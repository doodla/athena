//! Content checks on a change (design §7.4 checks 2–5): append-only records, valid records in place,
//! file size, personal data on added lines, and configuration that parses.

use crate::{
    CONFIG_PATH, Change, DEFAULTS_PATH, Judged, Result, Rule, Violation, defers_to_schema_test,
};
use athena_data::config::PersonalData;
use athena_data::records::check_record_file;
use athena_data::{Config, DefaultsModel};
use athena_git::Git;
use athena_git::attrs::PathClass;
use regex::Regex;
use std::sync::OnceLock;

/// Runs every content check on `change`, judged by `judged` (the parent's rules).
///
/// `allow` overrides the personal-data allow-list. The full-history check passes `HEAD`'s, as it does for
/// the secrets allow-list: allow-listing an old false positive at `HEAD` is legitimate.
pub fn check(
    git: &Git,
    change: &Change,
    judged: &Judged,
    allow: Option<&PersonalData>,
) -> Result<Vec<Violation>> {
    let commit = change.commit.as_deref();
    let mut violations = Vec::new();
    let max = judged.config.records.max_file_bytes;

    let new_blobs: Vec<String> = judged
        .diff
        .iter()
        .filter(|d| d.status != 'D' && d.new_mode != "160000")
        .map(|d| d.new_oid.clone())
        .collect();
    let sizes = git.blob_sizes(&new_blobs)?;

    let defer_to_schema_test = defers_to_schema_test(
        judged
            .diff
            .iter()
            .map(|d| (d.path.as_str(), d.status == 'D')),
    );

    for d in &judged.diff {
        let class = judged.class(&d.path);
        if class == PathClass::Records && d.status != 'A' {
            let what = match d.status {
                'D' => "deleted",
                'T' => "changed in type",
                _ => "modified",
            };
            violations.push(
                Violation::new(
                    Rule::AppendOnly,
                    format!("{} was {what}", d.path),
                    "records are written once; add a new record that references this one (a re-score, a later \
                     outcome, or a resolution) instead",
                )
                .at_path(&d.path)
                .in_commit(commit),
            );
        }
        if d.status == 'D' || d.new_mode == "160000" {
            continue;
        }
        let size = sizes.get(&d.new_oid).copied().unwrap_or(0);
        if size > max {
            violations.push(
                Violation::new(
                    Rule::Size,
                    format!("{} is {size} bytes, over the limit of {max}", d.path),
                    "keep large artifacts out of the repository, or raise [records] max_file_bytes in a commit of its own",
                )
                .at_path(&d.path)
                .in_commit(commit),
            );
            continue;
        }
        if class == PathClass::Records && d.status == 'A' {
            let bytes = git.cat_blob(&d.new_oid)?;
            if let Err(error) = check_record_file(&d.path, &bytes) {
                violations.push(
                    Violation::new(
                        Rule::ValidRecords,
                        error.to_string(),
                        "write records through athena, which writes them valid and in place",
                    )
                    .at_path(&d.path)
                    .in_commit(commit),
                );
            }
        }
        if d.path == CONFIG_PATH || d.path == DEFAULTS_PATH {
            let text = String::from_utf8_lossy(&git.cat_blob(&d.new_oid)?).into_owned();
            let problem = if defer_to_schema_test {
                // The candidate's own athena-data test judges the file; the running schema would not.
                None
            } else if d.path == CONFIG_PATH {
                Config::parse(&text).err().map(|e| e.to_string())
            } else {
                DefaultsModel::parse(&text).err().map(|e| e.to_string())
            };
            if let Some(problem) = problem {
                violations.push(
                    Violation::new(
                        Rule::Configuration,
                        format!("{}: {problem}", d.path),
                        "fix the file before committing it",
                    )
                    .at_path(&d.path)
                    .in_commit(commit),
                );
            } else if d.path == CONFIG_PATH {
                violations.extend(new_exemptions(git, change, judged, &text)?);
            }
        }
    }

    let allow = allow.unwrap_or(&judged.config.personal_data);
    violations.extend(personal_data(git, judged, allow, &change.tree, commit)?);
    Ok(violations)
}

/// An exemption added to `[history] exempt` must name a commit already in this history: an ancestor of
/// the commit that adds it (acceptance verification 2, N1). Entries the parent's rules already list are
/// judged where they were added.
fn new_exemptions(
    git: &Git,
    change: &Change,
    judged: &Judged,
    text: &str,
) -> Result<Vec<Violation>> {
    let Ok(config) = Config::parse(text) else {
        return Ok(Vec::new());
    };
    let mut violations = Vec::new();
    for entry in config
        .history
        .exempt
        .iter()
        .filter(|e| !judged.config.history.exempt.contains(e))
    {
        let in_the_past = match change.parents.first() {
            Some(parent) => {
                git.rev_parse(entry)?.as_deref() == Some(entry.as_str())
                    && git.is_ancestor(entry, parent)?
            }
            None => false,
        };
        if !in_the_past {
            violations.push(
                Violation::new(
                    Rule::Configuration,
                    format!(
                        "{CONFIG_PATH}: [history] exempt adds {entry}, which is not a commit already in this history"
                    ),
                    "an exemption can only name a commit made before it, such as one that predates Athena",
                )
                .at_path(CONFIG_PATH)
                .in_commit(change.commit.as_deref()),
            );
        }
    }
    Ok(violations)
}

fn email_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9\-]+(?:\.[A-Za-z0-9\-]+)*\.[A-Za-z]{2,}")
            .expect("the email pattern compiles")
    })
}

/// Email addresses in `text` that the allow-list does not allow.
pub fn disallowed_emails<'a>(text: &'a str, allow: &PersonalData) -> Vec<&'a str> {
    email_regex()
        .find_iter(text)
        .map(|m| m.as_str())
        .filter(|e| !allow.allows(e))
        .collect()
}

/// Replaces every email address outside the allow-list with `[email]`.
pub fn redact_emails(text: &str, allow: &PersonalData) -> String {
    email_regex()
        .replace_all(text, |caps: &regex::Captures<'_>| {
            let found = &caps[0];
            if allow.allows(found) {
                found.to_string()
            } else {
                "[email]".to_string()
            }
        })
        .into_owned()
}

/// Shows an address without repeating it in full, for gate messages that may reach public CI logs.
fn masked(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) => format!("{}…@{domain}", local.chars().next().unwrap_or('?')),
        None => "…".into(),
    }
}

fn personal_data(
    git: &Git,
    judged: &Judged,
    allow: &PersonalData,
    tree: &str,
    commit: Option<&str>,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for (path, line) in git.added_lines(&judged.base, tree)? {
        if judged.class(&path).is_owner() {
            continue;
        }
        for email in disallowed_emails(&line, allow) {
            violations.push(
                Violation::new(
                    Rule::PersonalData,
                    format!("{path} adds the email address {}", masked(email)),
                    "remove it, or use a no-reply or documentation address (see [personal_data] in athena.toml)",
                )
                .at_path(&path)
                .in_commit(commit),
            );
        }
    }
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    fn gate_last(git: &Git) -> Vec<Violation> {
        let head = git.head().unwrap().unwrap();
        let change = Change::of_commit(git, &head).unwrap();
        let judged = Judged::new(git, &change).unwrap();
        check(git, &change, &judged, None).unwrap()
    }

    const READING_DIR: &str = "records/jobs/2026-10/20261010-153012-k7q2";

    fn valid_intervention() -> (String, String) {
        let path = "records/interventions/2026-10/20261010-153012-k7q2.json".to_string();
        let text = r#"{
  "schema": "athena.intervention/1",
  "id": "20261010-153012-k7q2",
  "at": "2026-10-10T15:30:12Z",
  "what": "Fixed a typo by hand",
  "why": "The verifier missed it",
  "class_of_work": "docs",
  "category": "did-work-themselves",
  "job": null,
  "source": "user",
  "with_staged_changes": false
}
"#
        .to_string();
        (path, text)
    }

    #[test]
    fn modifying_deleting_or_renaming_a_record_is_refused() {
        // M4 (blocks): modifying, deleting, or renaming a committed record.
        let (path, text) = valid_intervention();
        let (dir, git) = repo(&[(&path, &text), ("README.md", "x\n")]);
        write(
            dir.path(),
            &[(&path, &text.replace("by hand", "by hand, twice"))],
        );
        commit(&git, "edit\n\nAthena-Job: bootstrap\n");
        assert!(gate_last(&git).iter().any(|v| v.rule == Rule::AppendOnly));

        std::fs::remove_file(dir.path().join(&path)).unwrap();
        commit(&git, "delete\n\nAthena-Job: bootstrap\n");
        assert!(gate_last(&git).iter().any(|v| v.rule == Rule::AppendOnly));

        let (dir, git) = repo(&[(&path, &text)]);
        std::fs::rename(
            dir.path().join(&path),
            dir.path().join("records/interventions/2026-10/moved.json"),
        )
        .unwrap();
        commit(&git, "rename\n\nAthena-Job: bootstrap\n");
        assert!(gate_last(&git).iter().any(|v| v.rule == Rule::AppendOnly));
    }

    #[test]
    fn adding_a_valid_record_passes() {
        // M4/M5 (allows): adding new, valid records.
        let (dir, git) = repo(&[("README.md", "x\n")]);
        let (path, text) = valid_intervention();
        write(dir.path(), &[(&path, &text)]);
        commit(&git, "record\n\nAthena-Job: bootstrap\n");
        assert_eq!(gate_last(&git), vec![]);
    }

    #[test]
    fn malformed_misplaced_or_unknown_records_are_refused() {
        // M5 (blocks): malformed, unknown-field, and misplaced records.
        let (path, text) = valid_intervention();
        for (file, content) in [
            (path.clone(), "{ not json".to_string()),
            (
                path.clone(),
                text.replace("\"job\": null,", "\"job\": null,\n  \"extra\": 1,"),
            ),
            (
                "records/interventions/2026-11/20261010-153012-k7q2.json".to_string(),
                text.clone(),
            ),
            (format!("{READING_DIR}/notes.txt"), "free text".to_string()),
        ] {
            let (dir, git) = repo(&[("README.md", "x\n")]);
            write(dir.path(), &[(&file, &content)]);
            commit(&git, "bad record\n\nAthena-Job: bootstrap\n");
            assert!(
                gate_last(&git).iter().any(|v| v.rule == Rule::ValidRecords),
                "{file} should be refused"
            );
        }
    }

    #[test]
    fn oversized_files_are_refused() {
        // M5 (blocks): an oversized file.
        let (dir, git) = repo(&[("README.md", "x\n")]);
        let big = "a".repeat(1024 * 1024 + 1);
        write(dir.path(), &[("big.txt", &big)]);
        commit(&git, "big\n\nAthena-Job: bootstrap\n");
        assert!(gate_last(&git).iter().any(|v| v.rule == Rule::Size));
    }

    #[test]
    fn added_lines_with_private_email_addresses_are_refused() {
        // M22 (blocks): an added line with a non-allow-listed email address is refused.
        let (dir, git) = repo(&[("README.md", "x\n")]);
        write(
            dir.path(),
            &[("CONTRIBUTORS.md", &format!("Jane <{}>\n", private_email()))],
        );
        commit(&git, "contributors\n\nAthena-Job: bootstrap\n");
        let found = gate_last(&git);
        assert!(found.iter().any(|v| v.rule == Rule::PersonalData));
        assert!(
            !found.iter().any(|v| v.detail.contains(&private_email())),
            "messages do not repeat the address"
        );
    }

    #[test]
    fn no_reply_addresses_pass() {
        // M22 (allows): ordinary text and noreply@ addresses.
        let (dir, git) = repo(&[("README.md", "x\n")]);
        write(
            dir.path(),
            &[(
                "NOTES.md",
                "Co-Authored-By: Claude <noreply@anthropic.com>\nplain text\n",
            )],
        );
        commit(&git, "notes\n\nAthena-Job: bootstrap\n");
        assert_eq!(gate_last(&git), vec![]);
    }

    #[test]
    fn owner_paths_are_not_scanned_for_personal_data() {
        let (dir, git) = repo(&[(".gitattributes", "/OWNER.md athena-readonly\n")]);
        write(
            dir.path(),
            &[("OWNER.md", &format!("contact: {}\n", private_email()))],
        );
        commit(&git, "owner\n");
        assert_eq!(gate_last(&git), vec![]);
    }

    #[test]
    fn a_configuration_that_does_not_parse_is_refused() {
        let (dir, git) = repo(&[("README.md", "x\n")]);
        write(dir.path(), &[("athena.toml", "[athena]\nschema = 2\n")]);
        commit(&git, "bad config\n\nAthena-Job: bootstrap\n");
        assert!(
            gate_last(&git)
                .iter()
                .any(|v| v.rule == Rule::Configuration)
        );
    }

    #[test]
    fn a_new_section_with_its_schema_change_in_one_commit_is_not_refused() {
        let config = include_str!("../../../athena.toml");
        // A section no schema knows (`[parallel]` was this example until the schema gained it).
        let with_section = format!("{config}\n[telemetry]\nsample_rate = 2\n");
        // The running schema rejects the section; that is the refusal this change removes.
        assert!(Config::parse(&with_section).is_err());

        let (dir, git) = repo(&[
            ("athena.toml", config),
            ("crates/athena-data/src/lib.rs", ""),
        ]);
        write(
            dir.path(),
            &[
                ("athena.toml", &with_section),
                ("crates/athena-data/src/lib.rs", "// parallel section\n"),
            ],
        );
        commit(&git, "schema and section\n\nAthena-Job: bootstrap\n");
        assert!(
            !gate_last(&git)
                .iter()
                .any(|v| v.rule == Rule::Configuration)
        );

        // The same section without a schema change is still refused with the parse error.
        let (dir, git) = repo(&[
            ("athena.toml", config),
            ("crates/athena-data/src/lib.rs", ""),
        ]);
        write(dir.path(), &[("athena.toml", &with_section)]);
        commit(&git, "section only\n\nAthena-Job: bootstrap\n");
        let found = gate_last(&git);
        assert!(
            found
                .iter()
                .any(|v| v.rule == Rule::Configuration && v.detail.contains("unknown field")),
            "{found:?}"
        );
    }

    #[test]
    fn deleting_the_schema_does_not_defer_the_parse() {
        assert!(defers_to_schema_test([(
            "crates/athena-data/src/a.rs",
            false
        )]));
        assert!(!defers_to_schema_test([(
            "crates/athena-data/src/a.rs",
            true
        )]));
        assert!(!defers_to_schema_test([("athena.toml", false)]));
    }

    #[test]
    fn email_redaction_keeps_allowed_addresses() {
        let allow = crate::builtin_config().personal_data;
        let text = format!("mail {} or noreply@anthropic.com", private_email());
        assert_eq!(
            redact_emails(&text, &allow),
            "mail [email] or noreply@anthropic.com"
        );
    }
}
