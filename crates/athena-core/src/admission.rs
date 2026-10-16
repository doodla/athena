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
//!   fails here with a repairable finding instead of at landing. The file is judged by the candidate's
//!   schema: when the candidate changes `crates/athena-data/`, its own `athena-data` is written from the
//!   candidate commit to a temporary directory, built offline with the candidate's `Cargo.lock` and run on
//!   the file, so a job may add a section and its schema field together. If that check cannot be built or
//!   run, the result is a `configuration` violation, never a pass. Otherwise the running Athena's schema
//!   is the candidate's schema and parses the file directly;
//! - a governance file changed without the accepted reading listing it as a deliverable and naming it in
//!   an acceptance criterion. For a `Cargo.toml`, only its `[lints]` and `[workspace.lints]` tables are
//!   governance content.

use anyhow::Result;
use athena_data::config::PersonalData;
use athena_data::records::{ChangeStatus, PolicyViolation, Reading};
use athena_data::{Config, DefaultsModel};
use athena_gate::content::disallowed_emails;
use athena_gate::{CONFIG_PATH, DEFAULTS_PATH};
use athena_git::Git;
use athena_git::attrs::{PathClass, classification_changes, classify};
use std::collections::BTreeSet;

/// The crate holding the configuration and defaults schemas.
const SCHEMA_DIR: &str = "crates/athena-data/";

/// The program built against the candidate's `athena-data`: parses the file named by its second argument
/// as `config` (athena.toml) or `defaults` (defaults.toml), prints the error and exits 1 if it fails.
const CHECK_MAIN: &str = r#"fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&args[2]).expect("read the file to check");
    let result = match args[1].as_str() {
        "config" => athena_data::Config::parse(&text).map(|_| ()).map_err(|e| e.to_string()),
        _ => athena_data::DefaultsModel::parse(&text).map(|_| ()).map_err(|e| e.to_string()),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
"#;

/// Parses `text` (`kind` is `config` or `defaults`) with the schema of `candidate`'s own `athena-data`.
///
/// The candidate's tree is written to a temporary directory and a small crate depending on its
/// `athena-data` is built offline with the candidate's `Cargo.lock`, then run on the file. The error
/// is the parse error, or the reason the check could not be built or run; it never reads as success.
fn check_with_candidate_schema(
    git: &Git,
    candidate: &str,
    kind: &str,
    text: &str,
) -> std::result::Result<(), String> {
    let fail = |what: &str, e: &dyn std::fmt::Display| format!("{what}: {e}");
    let dir =
        tempfile::tempdir().map_err(|e| fail("the candidate schema check has no workspace", &e))?;
    let tree = dir.path().join("tree");
    let entries = git
        .ls_tree(candidate)
        .map_err(|e| fail("cannot list the candidate's files", &e))?;
    for entry in entries {
        let wanted = entry.path.starts_with(SCHEMA_DIR)
            || entry.path == "Cargo.toml"
            || entry.path == "Cargo.lock";
        if !wanted || entry.mode == "160000" {
            continue;
        }
        let bytes = git
            .cat_blob(&entry.oid)
            .map_err(|e| fail("cannot read the candidate's files", &e))?;
        let target = tree.join(&entry.path);
        std::fs::create_dir_all(target.parent().unwrap_or(&tree))
            .and_then(|()| std::fs::write(&target, bytes))
            .map_err(|e| fail("cannot write the candidate's files", &e))?;
    }
    let check = dir.path().join("check");
    let write = |path: std::path::PathBuf, body: String| {
        std::fs::create_dir_all(path.parent().unwrap_or(&check))
            .and_then(|()| std::fs::write(path, body))
            .map_err(|e| fail("cannot write the schema check", &e))
    };
    write(
        check.join("Cargo.toml"),
        format!(
            "[package]\nname = \"candidate-schema-check\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
             [dependencies]\nathena-data = {{ path = {:?} }}\n\n[workspace]\n",
            tree.join("crates/athena-data").display().to_string()
        ),
    )?;
    write(check.join("src/main.rs"), CHECK_MAIN.to_string())?;
    if let Ok(lock) = std::fs::read_to_string(tree.join("Cargo.lock")) {
        write(check.join("Cargo.lock"), lock)?;
    }
    let input = dir.path().join("input.toml");
    std::fs::write(&input, text).map_err(|e| fail("cannot write the file to check", &e))?;
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| dir.path().join("target"));
    let output =
        std::process::Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(["run", "--quiet", "--offline", "--manifest-path"])
            .arg(check.join("Cargo.toml"))
            .arg("--")
            .arg(kind)
            .arg(&input)
            .env("CARGO_TARGET_DIR", target_dir)
            .env_remove("RUSTFLAGS")
            .current_dir(dir.path())
            .output()
            .map_err(|e| fail("the candidate schema check could not start", &e))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "under the candidate's own schema ({}): {}",
        SCHEMA_DIR.trim_end_matches('/'),
        stderr.trim()
    ))
}

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
pub fn governance_change_is_listed(reading: &Reading, path: &str) -> bool {
    let listed = reading.deliverables.iter().any(|d| d.path == path);
    let named = reading
        .acceptance_criteria
        .iter()
        .any(|c| c.text.contains(path) || c.text.contains(file_name(path)));
    listed && named
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

    // Configuration: a changed athena.toml or defaults.toml must parse under the candidate's own schema, so
    // an unknown field fails the job here with a finding the producer can repair, instead of at landing.
    // When the candidate changes `crates/athena-data/`, the running Athena's compiled schema is the wrong
    // judge (a job adding a section and its field together would always be refused), so the file is
    // checked by a build of the candidate's own `athena-data`; a check that cannot run is a violation.
    let schema_changed = input
        .changes
        .iter()
        .any(|c| c.path.starts_with(SCHEMA_DIR) && c.status != ChangeStatus::Deleted);
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
            let kind = if path == CONFIG_PATH {
                "config"
            } else {
                "defaults"
            };
            check_with_candidate_schema(git, input.candidate, kind, &text).err()
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
        write_schema_crate(dir.path());
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

    /// Copies the real workspace manifest, lock file and `athena-data` crate into a test repository, so a
    /// candidate can change the schema and the check can build it.
    fn write_schema_crate(dest: &std::path::Path) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for file in ["Cargo.toml", "Cargo.lock", "crates/athena-data/Cargo.toml"] {
            let to = dest.join(file);
            std::fs::create_dir_all(to.parent().unwrap()).unwrap();
            std::fs::copy(root.join(file), to).unwrap();
        }
        let src = dest.join("crates/athena-data/src");
        std::fs::create_dir_all(&src).unwrap();
        for entry in std::fs::read_dir(root.join("crates/athena-data/src")).unwrap() {
            let entry = entry.unwrap();
            std::fs::copy(entry.path(), src.join(entry.file_name())).unwrap();
        }
    }

    const CONFIG_RS: &str = include_str!("../../athena-data/src/config.rs");
    const CONFIG_RS_PATH: &str = "crates/athena-data/src/config.rs";
    const FIELD_ANCHOR: &str = "    pub personal_data: PersonalData,\n";

    /// The schema with a new optional `[parallel]` section added.
    fn config_rs_with_parallel() -> String {
        assert!(CONFIG_RS.contains(FIELD_ANCHOR));
        CONFIG_RS.replacen(
            FIELD_ANCHOR,
            &format!(
                "{FIELD_ANCHOR}    #[serde(default)]\n    pub parallel: Option<toml::Value>,\n"
            ),
            1,
        )
    }

    #[test]
    fn a_new_section_and_its_schema_field_together_are_admitted() {
        let text = format!("{BASE_CONFIG}\n[parallel]\nmax_jobs = 2\n");
        // The running Athena's schema is the wrong judge: it reproduces the refusal jobs
        // 20261016-071202-q0xy and 20261016-185517-qcrz met.
        let old = Config::parse(&text).unwrap_err().to_string();
        assert!(old.contains("unknown field `parallel`"), "{old}");
        let found = configuration(admit_files(&[
            ("athena.toml", &text),
            (CONFIG_RS_PATH, &config_rs_with_parallel()),
        ]));
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn an_unknown_section_without_a_schema_change_is_still_refused_by_the_candidate_schema() {
        let text = format!("{BASE_CONFIG}\n[bogus]\nfield = 1\n");
        // A candidate that touches the schema (here only a comment) is judged by its own build.
        let schema = format!("// touched\n{CONFIG_RS}");
        let found = configuration(admit_files(&[
            ("athena.toml", &text),
            (CONFIG_RS_PATH, &schema),
        ]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].detail.contains("unknown field `bogus`"),
            "{found:?}"
        );
        assert!(
            found[0].detail.contains("candidate's own schema"),
            "{found:?}"
        );
        // And with no schema change the section is refused as before.
        let found = configuration(admit_files(&[("athena.toml", &text)]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].detail.contains("unknown field `bogus`"),
            "{found:?}"
        );
    }

    #[test]
    fn a_candidate_schema_check_that_cannot_run_is_a_configuration_violation() {
        let text = format!("{BASE_CONFIG}\n# edited\n");
        let broken = format!("{CONFIG_RS}\nthis is not rust;\n");
        let found = configuration(admit_files(&[
            ("athena.toml", &text),
            (CONFIG_RS_PATH, &broken),
        ]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].detail.contains("does not parse"), "{found:?}");
        assert!(found[0].detail.contains("error"), "{found:?}");
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
