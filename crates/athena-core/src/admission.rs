//! Admission: what a candidate may change (design §7.3, D38, D44).
//!
//! The core refuses a change if any of the following hold:
//!
//! - a path is under `records/`, or is marked `athena-exclude` or `athena-readonly` under the base's rules;
//! - a submodule or gitlink appears;
//! - a file exceeds the size limit;
//! - an added line contains an email address outside the allow-list, or gitleaks finds a secret;
//! - the candidate would change how any path is classified;
//! - a changed `athena.toml` or `defaults.toml` does not parse (rule `configuration`), so an unknown field
//!   fails here with a repairable finding instead of at landing. When the candidate changes
//!   `crates/athena-data/`, the file is left to the test in the candidate's own `athena-data`
//!   (`tests/repo_config.rs`), which runs where the candidate is built; candidate code is never built
//!   here, so a job may add a section and its schema field together. Otherwise the running Athena's
//!   schema is the parent's schema and parses the file directly (`athena_gate::defers_to_schema_test`
//!   decides, for admission and the landing gate alike);
//! - a governance file changed without the accepted reading listing it as a deliverable and naming it in
//!   an acceptance criterion. For a `Cargo.toml`, only its `[lints]` and `[workspace.lints]` tables are
//!   governance content.

use anyhow::Result;
use athena_agent::policy::{Decision, Policy, evaluate};
use athena_data::config::PersonalData;
use athena_data::records::{
    ChangeStatus, CriterionKind, DeliverableAction, Demonstration, Finding, FindingSource, JobPlan,
    PolicyViolation, Reading, Severity,
};
use athena_data::{Config, DefaultsModel};
use athena_gate::content::disallowed_emails;
use athena_gate::{CONFIG_PATH, DEFAULTS_PATH, defers_to_schema_test};
use athena_git::Git;
use athena_git::attrs::{PathClass, classification_changes, classify};
use std::collections::BTreeSet;

/// Root files that govern how Athena works in this repository.
const ROOT_GOVERNANCE: [&str; 4] = [
    "athena.toml",
    "defaults.toml",
    "setup.sh",
    "rust-toolchain.toml",
];

/// File names that configure the check tools, wherever they appear.
const TOOL_CONFIG_NAMES: [&str; 4] = [
    "rustfmt.toml",
    ".rustfmt.toml",
    "clippy.toml",
    ".clippy.toml",
];

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Whether `path` configures a check tool (cargo, rustfmt, clippy).
pub fn is_tool_config(path: &str) -> bool {
    TOOL_CONFIG_NAMES.contains(&file_name(path))
        || path.ends_with(".cargo/config.toml")
        || path.ends_with(".cargo/config")
}

/// Whether `path` is a governance file (whole-file); `Cargo.toml` lints tables are handled separately.
pub fn is_governance(path: &str) -> bool {
    ROOT_GOVERNANCE.contains(&path)
        || file_name(path) == ".gitattributes"
        || (path.starts_with(".github/workflows/") && !path.ends_with('/'))
        || is_tool_config(path)
}

/// Whether `path` is a Cargo manifest.
pub fn is_manifest(path: &str) -> bool {
    file_name(path) == "Cargo.toml"
}

/// The `[lints]` and `[workspace.lints]` tables of a manifest, normalized for comparison.
pub fn lints_tables(manifest: &str) -> Option<String> {
    let doc: toml_edit::DocumentMut = manifest.parse().ok()?;
    let lints = doc.get("lints").map(|i| i.to_string()).unwrap_or_default();
    let workspace = doc
        .get("workspace")
        .and_then(|w| w.get("lints"))
        .map(|i| i.to_string())
        .unwrap_or_default();
    Some(format!("{lints}\n--\n{workspace}"))
}

/// One changed path of the candidate, with what admission needs to know.
#[derive(Debug, Clone)]
pub struct Changed {
    pub path: String,
    pub status: ChangeStatus,
    pub mode: String,
    pub size: u64,
}

/// Everything admission reads.
#[derive(Debug)]
pub struct AdmissionInput<'a> {
    pub git: &'a Git,
    /// The base commit P (its rules judge).
    pub base: &'a str,
    /// The candidate commit C, already built.
    pub candidate: &'a str,
    pub changes: &'a [Changed],
    pub reading: &'a Reading,
    pub max_file_bytes: u64,
    pub personal_data: &'a PersonalData,
    /// The base's configuration, whose `[secrets]` judges the candidate.
    pub config: &'a athena_data::Config,
}

fn violation(rule: &str, path: Option<&str>, detail: String) -> PolicyViolation {
    PolicyViolation {
        rule: rule.into(),
        path: path.map(str::to_string),
        detail,
    }
}

/// Whether the reading lists `path` as a deliverable and names it in an acceptance criterion.
///
/// This is the one definition of a listed governance change. Admission applies it to the paths a candidate
/// changed, and reading acceptance (`reading::check_governance_coverage`) applies it to the paths a request's
/// outcome needs, so the same omission is refused at reading time and not only after the producer has finished.
pub fn governance_change_is_listed(reading: &Reading, path: &str) -> bool {
    let listed = reading.deliverables.iter().any(|d| d.path == path);
    let named = reading
        .acceptance_criteria
        .iter()
        .any(|c| c.text.contains(path) || c.text.contains(file_name(path)));
    listed && named
}

fn athena_finding(
    severity: Severity,
    location: String,
    summary: String,
    evidence: String,
) -> Finding {
    Finding {
        severity,
        summary,
        evidence,
        location: Some(location),
        source: FindingSource::Athena,
    }
}

/// Athena's own checks on a job plan (`docs/design/job-planning.md` section 3), which cost no model call:
///
/// - every acceptance criterion has exactly one demonstration;
/// - a demonstration of a behavior criterion names what fails without the behavior and cites a command the
///   verifier's grant allows (`verifier` is that grant as a policy; its root must exist);
/// - a file to modify or delete exists in the base, and a file to create does not;
/// - a governance file must be listed by the reading and named in a criterion (`governance_change_is_listed`);
///   that is a major finding otherwise;
/// - any other path outside the reading's deliverables is a minor finding.
///
/// - every planned path is a plain relative file path: not empty, not absolute, no `..`, no trailing `/`, and
///   not a directory in the base (`is_directory` says whether a path is one); a major finding otherwise.
///
/// A major finding sends the plan back to the planner without a critic call.
pub fn plan_findings(
    plan: &JobPlan,
    reading: &Reading,
    verifier: &Policy,
    is_directory: &dyn Fn(&str) -> bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let count = reading.acceptance_criteria.len();
    for (i, criterion) in reading.acceptance_criteria.iter().enumerate() {
        let number = (i + 1) as u32;
        let demos: Vec<&Demonstration> = plan
            .demonstrations
            .iter()
            .filter(|d| d.criterion == number)
            .collect();
        let location = format!("demonstrations, criterion {number}");
        if demos.len() != 1 {
            findings.push(athena_finding(
                Severity::Major,
                location.clone(),
                format!(
                    "Criterion {number} has {} demonstrations; it needs exactly one",
                    demos.len()
                ),
                format!("Criterion {number} is: {}", criterion.text),
            ));
        }
        if criterion.kind != CriterionKind::Behavior {
            continue;
        }
        for d in demos {
            if d.fails_without
                .as_deref()
                .is_none_or(|f| f.trim().is_empty())
            {
                findings.push(athena_finding(
                    Severity::Major,
                    location.clone(),
                    format!("The demonstration of behavior criterion {number} does not say what fails without the behavior"),
                    "A demonstration of a behavior criterion must set fails_without, so that it would fail if the behavior were missing.".into(),
                ));
            }
            if let Decision::Deny(why) = evaluate(
                verifier,
                "Bash",
                &serde_json::json!({ "command": d.command }),
                None,
            ) {
                findings.push(athena_finding(
                    Severity::Major,
                    location.clone(),
                    format!(
                        "The demonstration of behavior criterion {number} cites a command the verifier may not run: `{}`",
                        d.command.trim()
                    ),
                    why,
                ));
            }
        }
    }
    for d in plan
        .demonstrations
        .iter()
        .filter(|d| d.criterion == 0 || d.criterion as usize > count)
    {
        findings.push(athena_finding(
            Severity::Major,
            format!("demonstrations, criterion {}", d.criterion),
            format!(
                "A demonstration names criterion {}, but the reading has {count} criteria",
                d.criterion
            ),
            format!("Its command is `{}`.", d.command.trim()),
        ));
    }
    for f in &plan.files {
        let location = format!("files, {}", f.path);
        if let Some(problem) = path_problem(&f.path, is_directory) {
            findings.push(athena_finding(
                Severity::Major,
                location,
                format!("The plan lists {:?}, which {problem}", f.path),
                "List each file by its plain relative path, such as `src/lib.rs`; never a directory.".into(),
            ));
            continue;
        }
        match f.action {
            DeliverableAction::Create if f.exists => findings.push(athena_finding(
                Severity::Major,
                location.clone(),
                format!(
                    "The plan creates {}, which already exists in the base",
                    f.path
                ),
                "Plan a modification, or choose another path.".into(),
            )),
            DeliverableAction::Modify | DeliverableAction::Delete if !f.exists => {
                findings.push(athena_finding(
                    Severity::Major,
                    location.clone(),
                    format!(
                        "The plan changes {}, which does not exist in the base",
                        f.path
                    ),
                    "Plan to create it, or name the file that exists.".into(),
                ))
            }
            _ => {}
        }
        if is_governance(&f.path) {
            if !governance_change_is_listed(reading, &f.path) {
                findings.push(athena_finding(
                    Severity::Major,
                    location,
                    format!(
                        "{} is a governance file and the accepted reading does not both list it as a deliverable and name it in a criterion",
                        f.path
                    ),
                    "Admission would refuse the change; leave the file out of the plan.".into(),
                ));
            }
        } else if !reading.deliverables.iter().any(|d| d.path == f.path) {
            findings.push(athena_finding(
                Severity::Minor,
                location,
                format!("{} is not among the reading's deliverables", f.path),
                "A change outside the listed deliverables widens the scope.".into(),
            ));
        }
    }
    findings
}

/// Why `path` cannot be a planned file, or `None` if it is a plain relative file path. The shape checks need
/// no filesystem; `is_directory` is asked only for a path of a good shape.
fn path_problem(path: &str, is_directory: &dyn Fn(&str) -> bool) -> Option<&'static str> {
    if path.trim().is_empty() {
        Some("is empty")
    } else if path.starts_with('/')
        || path.starts_with('\\')
        || path.as_bytes().get(1) == Some(&b':')
    {
        Some("is an absolute path")
    } else if path.split(['/', '\\']).any(|part| part == "..") {
        Some("contains `..`")
    } else if path.ends_with('/') {
        Some("ends in `/`, so it names a directory")
    } else if is_directory(path) {
        Some("is a directory in the base")
    } else {
        None
    }
}

/// The core's admission rules. Returns every violation found.
pub fn admit(input: &AdmissionInput<'_>) -> Result<Vec<PolicyViolation>> {
    let mut violations = admit_without_secrets(input)?;
    // Secrets: the candidate is scanned now, so a secret fails the job here instead of at landing.
    for finding in athena_gate::secrets::check_commit(
        input.git,
        input.config,
        Some(input.base),
        input.candidate,
    )? {
        violations.push(violation(
            "secret",
            finding.path.as_deref(),
            finding.detail.clone(),
        ));
    }
    Ok(violations)
}

/// Every admission rule except the secret scan, which needs the pinned gitleaks binary.
fn admit_without_secrets(input: &AdmissionInput<'_>) -> Result<Vec<PolicyViolation>> {
    let git = input.git;
    let mut violations = Vec::new();
    let paths: Vec<String> = input.changes.iter().map(|c| c.path.clone()).collect();
    let classes = classify(git, input.base, &paths)?;
    for change in input.changes {
        let path = change.path.as_str();
        match classes.get(path).copied().unwrap_or(PathClass::Governed) {
            PathClass::Records => violations.push(violation(
                "records",
                Some(path),
                format!("{path} is under records/, which only Athena writes"),
            )),
            PathClass::Excluded => violations.push(violation(
                "owner-private",
                Some(path),
                format!("{path} is private to the repository owner"),
            )),
            PathClass::Readonly => violations.push(violation(
                "owner-maintained",
                Some(path),
                format!("{path} is the owner's own document; no job may change it"),
            )),
            PathClass::Governed => {}
        }
        if change.mode == "160000" {
            violations.push(violation(
                "gitlink",
                Some(path),
                format!("{path} is a submodule or gitlink"),
            ));
        }
        if change.size > input.max_file_bytes {
            violations.push(violation(
                "size",
                Some(path),
                format!(
                    "{path} is {} bytes, over the limit of {}",
                    change.size, input.max_file_bytes
                ),
            ));
        }
    }

    // Added lines with private email addresses.
    let mut flagged = BTreeSet::new();
    for (path, line) in git.added_lines(input.base, input.candidate)? {
        if !disallowed_emails(&line, input.personal_data).is_empty() && flagged.insert(path.clone())
        {
            violations.push(violation(
                "personal-data",
                Some(&path),
                format!("{path} adds an email address outside the allow-list"),
            ));
        }
    }

    // Classification stability: no path may change class between the base's rules and the candidate's.
    let mut all: BTreeSet<String> = git
        .ls_tree(input.base)?
        .into_iter()
        .map(|e| e.path)
        .collect();
    all.extend(git.ls_tree(input.candidate)?.into_iter().map(|e| e.path));
    let all: Vec<String> = all.into_iter().collect();
    let moved = classification_changes(git, input.base, input.candidate, &all)?;
    if !moved.is_empty() {
        violations.push(violation(
            "classification",
            moved.first().map(String::as_str),
            format!("the candidate changes how {} path(s) are classified (athena-* declarations), starting with {}", moved.len(), moved[0]),
        ));
    }

    // Configuration: a changed athena.toml or defaults.toml must parse, so an unknown field fails the job
    // here with a finding the producer can repair, instead of at landing. When the candidate changes
    // `crates/athena-data/`, the running Athena's compiled schema is the wrong judge (a job adding a
    // section and its field together would always be refused) and candidate code is never built here:
    // the test in the candidate's own `athena-data` (`tests/repo_config.rs`) judges the file where the
    // candidate is built. The same rule, `defers_to_schema_test`, decides this at landing.
    let schema_changed = defers_to_schema_test(
        input
            .changes
            .iter()
            .map(|c| (c.path.as_str(), c.status == ChangeStatus::Deleted)),
    );
    for change in input.changes {
        let path = change.path.as_str();
        if change.status == ChangeStatus::Deleted || ![CONFIG_PATH, DEFAULTS_PATH].contains(&path) {
            continue;
        }
        let Some(bytes) = git.show_file(input.candidate, path)? else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        let problem = if schema_changed {
            None
        } else if path == CONFIG_PATH {
            Config::parse(&text).err().map(|e| e.to_string())
        } else {
            DefaultsModel::parse(&text).err().map(|e| e.to_string())
        };
        if let Some(problem) = problem {
            violations.push(violation(
                "configuration",
                Some(path),
                format!("{path} does not parse: {problem}. Repair it: remove or correct the offending field so the file parses"),
            ));
        }
    }

    // Governance files.
    for change in input.changes {
        let path = change.path.as_str();
        let governance = if is_manifest(path) {
            let before = git
                .show_file(input.base, path)?
                .map(|b| String::from_utf8_lossy(&b).into_owned());
            let after = git
                .show_file(input.candidate, path)?
                .map(|b| String::from_utf8_lossy(&b).into_owned());
            let lints = |text: &Option<String>| {
                text.as_deref()
                    .and_then(lints_tables)
                    .unwrap_or_else(|| "\n--\n".into())
            };
            lints(&before) != lints(&after)
        } else {
            is_governance(path)
        };
        if governance && !governance_change_is_listed(input.reading, path) {
            violations.push(violation(
                "governance",
                Some(path),
                format!(
                    "{path} is a governance file{}; a job may change it only when the accepted reading lists it as a deliverable and names it in an acceptance criterion",
                    if is_manifest(path) { " (its [lints] tables)" } else { "" }
                ),
            ));
        }
    }
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::JobId;
    use athena_data::records::{
        Criterion, CriterionKind, Deliverable, DeliverableAction, Forecast, Forecasts, Quantiles,
    };

    /// Athena's findings on a plan that lists only `path` (to create), where `dirs` are directories in the base.
    fn findings_for_path(path: &str, dirs: &[&str]) -> Vec<Finding> {
        use athena_data::records::{PlanFile, Role};
        let plan = JobPlan {
            job: JobId::parse("20261012-084936-n1p0").unwrap(),
            round: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 2,
            approach: "Approach.".into(),
            decisions: vec![],
            files: vec![PlanFile {
                path: path.into(),
                action: DeliverableAction::Create,
                purpose: "Purpose.".into(),
                governance: false,
                exists: false,
            }],
            steps: vec![],
            risks: vec![],
            demonstrations: vec![],
            open_questions: vec![],
            producer_section_sha256: "0".repeat(64),
            resumed_from: None,
        };
        let verifier = Policy {
            role: Role::Verifier,
            root: std::path::PathBuf::from("."),
            commands: Default::default(),
            writes: false,
            read_roots: Vec::new(),
        };
        plan_findings(&plan, &reading(), &verifier, &|p| dirs.contains(&p))
            .into_iter()
            .filter(|f| f.location.as_deref() == Some(&format!("files, {path}")))
            .collect()
    }

    #[test]
    fn a_planned_path_that_is_not_a_plain_relative_file_is_a_major_finding() {
        let dirs = ["docs", "src/deep"];
        for path in [
            "docs/",
            "docs",
            "src/deep",
            "",
            "   ",
            "/etc/passwd",
            "C:/x.txt",
            "../x.rs",
            "a/../b.rs",
            "a\\..\\b.rs",
            "src/",
        ] {
            let found = findings_for_path(path, &dirs);
            assert_eq!(found.len(), 1, "{path:?}: {found:?}");
            assert_eq!(found[0].severity, Severity::Major, "{path:?}");
            assert_eq!(found[0].source, FindingSource::Athena, "{path:?}");
        }
        // A plain file path is not a major finding (it is only outside the deliverables).
        for path in ["src/new.rs", "a..b.rs", "docs/new.md"] {
            let found = findings_for_path(path, &dirs);
            assert!(
                found.iter().all(|f| f.severity == Severity::Minor),
                "{path:?}: {found:?}"
            );
        }
    }

    const BASE_CONFIG: &str = include_str!("../../../athena.toml");
    const BASE_DEFAULTS: &str = include_str!("../../../defaults.toml");

    fn reading() -> Reading {
        let forecast = Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.7,
            p_user_intervenes: 0.1,
            cost_usd: Quantiles::new(0.2, 0.5, 1.5).unwrap(),
            wall_time_s: Quantiles::new(100.0, 300.0, 900.0).unwrap(),
        };
        Reading {
            job: JobId::parse("20261012-084936-n1p0").unwrap(),
            n: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            base_commit: "a".repeat(40),
            config_sha256: "b".repeat(64),
            defaults_sha256: "c".repeat(64),
            defaults_version: 1,
            title: "Edit configuration".into(),
            summary: "Edit configuration.".into(),
            class_of_work: "feature".into(),
            capability: Some("change".into()),
            decline_reason: None,
            producer_model: "producer".into(),
            verifier_model: "verifier".into(),
            defaults_applied: vec![],
            assumptions: vec![],
            question: None,
            acceptance_criteria: vec![Criterion {
                text: "athena.toml and defaults.toml change".into(),
                kind: CriterionKind::Content,
            }],
            deliverables: ["athena.toml", "defaults.toml"]
                .map(|path| Deliverable {
                    path: path.into(),
                    action: DeliverableAction::Modify,
                    p: 0.9,
                })
                .into(),
            out_of_scope: vec![],
            forecasts: Forecasts {
                reader: forecast.clone(),
                reference: forecast,
            },
            departures: None,
            urgent_reason: None,
            hard_reason: None,
            producer_effort: None,
        }
    }

    #[test]
    fn a_workflow_change_must_be_listed_and_named() {
        let path = ".github/workflows/ci.yml";
        assert!(is_governance(path));
        let mut r = reading();
        assert!(!governance_change_is_listed(&r, path));
        r.deliverables.push(Deliverable {
            path: path.into(),
            action: DeliverableAction::Modify,
            p: 0.9,
        });
        assert!(!governance_change_is_listed(&r, path), "listed but unnamed");
        r.acceptance_criteria.push(Criterion {
            text: format!("{path} runs the new check"),
            kind: CriterionKind::Content,
        });
        assert!(governance_change_is_listed(&r, path));
        let mut named_only = reading();
        named_only.acceptance_criteria.push(Criterion {
            text: format!("{path} runs the new check"),
            kind: CriterionKind::Content,
        });
        assert!(
            !governance_change_is_listed(&named_only, path),
            "named but unlisted"
        );
    }

    /// Commits the base files, then a candidate with `files` written over them; returns the violations of
    /// the rules admission applies to content, which do not need gitleaks.
    fn admit_files(files: &[(&str, &str)]) -> Vec<PolicyViolation> {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(dir.path().join("athena.toml"), BASE_CONFIG).unwrap();
        std::fs::write(dir.path().join("defaults.toml"), BASE_DEFAULTS).unwrap();
        std::fs::write(dir.path().join("a.txt"), "a\n").unwrap();
        std::fs::create_dir_all(dir.path().join("crates/athena-data/src")).unwrap();
        std::fs::write(dir.path().join(SCHEMA_FILE), "// schema\n").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "base"]).unwrap();
        let base = git.head().unwrap().unwrap();
        let mut changes = Vec::new();
        for (path, text) in files {
            std::fs::write(dir.path().join(path), text).unwrap();
            changes.push(Changed {
                path: path.to_string(),
                status: ChangeStatus::Modified,
                mode: "100644".into(),
                size: text.len() as u64,
            });
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "candidate"]).unwrap();
        let candidate = git.head().unwrap().unwrap();
        let config = athena_data::Config::parse(BASE_CONFIG).unwrap();
        admit_without_secrets(&AdmissionInput {
            git: &git,
            base: &base,
            candidate: &candidate,
            changes: &changes,
            reading: &reading(),
            max_file_bytes: 1 << 20,
            personal_data: &config.personal_data,
            config: &config,
        })
        .unwrap()
    }

    const SCHEMA_FILE: &str = "crates/athena-data/src/config.rs";

    #[test]
    fn a_new_section_and_its_schema_change_together_are_admitted_without_a_build() {
        // A section no schema knows (`[parallel]` was this example until the schema gained it).
        let text = format!("{BASE_CONFIG}\n[telemetry]\nsample_rate = 2\n");
        // The running Athena's schema is the wrong judge: it reproduces the refusal jobs
        // 20261016-071202-q0xy and 20261016-185517-qcrz met.
        let old = Config::parse(&text).unwrap_err().to_string();
        assert!(old.contains("unknown field `telemetry`"), "{old}");
        // The test repository has no Cargo.toml or lock file, so any attempt to build the candidate's
        // schema would fail and show up as a violation: none is the proof that nothing was built.
        let found = configuration(admit_files(&[
            ("athena.toml", &text),
            (SCHEMA_FILE, "// adds [telemetry]\n"),
        ]));
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn an_unknown_section_without_a_schema_change_is_still_refused() {
        let text = format!("{BASE_CONFIG}\n[bogus]\nfield = 1\n");
        let found = configuration(admit_files(&[("athena.toml", &text)]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].detail.contains("unknown field `bogus`"),
            "{found:?}"
        );
    }

    #[test]
    fn plan_admission_and_the_gate_agree_on_when_the_schema_judges_a_config_edit() {
        let text = format!("{BASE_CONFIG}\n[telemetry]\nsample_rate = 2\n");
        let cases: [(&str, &[(&str, &str)]); 2] = [
            ("config alone", &[("athena.toml", &text)]),
            (
                "config with the schema",
                &[("athena.toml", &text), (SCHEMA_FILE, "// adds it\n")],
            ),
        ];
        for (what, files) in cases {
            let changed: Vec<(&str, bool)> = files.iter().map(|(p, _)| (*p, false)).collect();
            // The plan rule flags the change exactly when admission parses the file itself and refuses it.
            let planned = crate::plan::schema_reference_problem(
                &changed
                    .iter()
                    .map(|(p, d)| (p.to_string(), *d))
                    .collect::<Vec<_>>(),
            );
            let admitted = configuration(admit_files(files));
            assert_eq!(
                planned.is_some(),
                !admitted.is_empty(),
                "{what}: {planned:?}"
            );
            // The gate's own deferral is the opposite of both.
            assert_eq!(
                defers_to_schema_test(changed.iter().copied()),
                planned.is_none(),
                "{what}"
            );
        }
        // And the plan message names the schema file to change.
        let problem =
            crate::plan::schema_reference_problem(&[("athena.toml".to_string(), false)]).unwrap();
        assert!(problem.contains(SCHEMA_FILE), "{problem}");
    }

    fn configuration(violations: Vec<PolicyViolation>) -> Vec<PolicyViolation> {
        violations
            .into_iter()
            .filter(|v| v.rule == "configuration")
            .collect()
    }

    #[test]
    fn an_unknown_field_in_athena_toml_is_a_configuration_violation() {
        let text = format!("{BASE_CONFIG}\n[bogus]\nfield = 1\n");
        let found = configuration(admit_files(&[("athena.toml", &text)]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].path.as_deref(), Some("athena.toml"));
        assert!(found[0].detail.contains("does not parse"), "{found:?}");
        assert!(found[0].detail.contains("bogus"), "{found:?}");
        assert!(found[0].detail.contains("Repair it"), "{found:?}");
    }

    #[test]
    fn an_unknown_field_in_defaults_toml_is_a_configuration_violation() {
        let text = format!("bogus_field = 1\n{BASE_DEFAULTS}");
        let found = configuration(admit_files(&[("defaults.toml", &text)]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].path.as_deref(), Some("defaults.toml"));
        assert!(found[0].detail.contains("does not parse"), "{found:?}");
    }

    #[test]
    fn parsing_configuration_and_untouched_configuration_add_no_violation() {
        let config = format!("# edited\n{BASE_CONFIG}");
        let defaults = format!("# edited\n{BASE_DEFAULTS}");
        let parsing = admit_files(&[("athena.toml", &config), ("defaults.toml", &defaults)]);
        assert!(configuration(parsing).is_empty());
        let untouched = admit_files(&[("a.txt", "b\n")]);
        assert!(configuration(untouched).is_empty());
    }

    #[test]
    fn governance_files_are_recognized() {
        for path in [
            "athena.toml",
            "defaults.toml",
            ".gitattributes",
            "crates/x/.gitattributes",
            ".github/workflows/ci.yml",
            "setup.sh",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            "crates/x/.cargo/config",
            "rustfmt.toml",
            ".rustfmt.toml",
            "clippy.toml",
            "sub/.clippy.toml",
        ] {
            assert!(is_governance(path), "{path}");
        }
        for path in [
            "README.md",
            "crates/x/Cargo.toml",
            "docs/athena.toml",
            "src/setup.sh",
        ] {
            assert!(!is_governance(path), "{path}");
        }
    }

    #[test]
    fn lints_tables_are_compared_alone() {
        let a = "[package]\nname = \"a\"\n\n[lints]\nworkspace = true\n";
        let b = "[package]\nname = \"a\"\nversion = \"2\"\n\n[lints]\nworkspace = true\n";
        let c = "[package]\nname = \"a\"\n\n[lints.rust]\nunsafe_code = \"allow\"\n";
        assert_eq!(lints_tables(a), lints_tables(b));
        assert_ne!(lints_tables(a), lints_tables(c));
    }
}
