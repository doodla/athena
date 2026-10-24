//! The `athena` binary as other programs meet it: git hooks, the agent tool hook, and nested runs.

mod common;

use common::*;
use std::io::Write;
use std::process::{Command, Stdio};

fn athena() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_athena"));
    cmd.env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR");
    cmd
}

fn install_hooks(fx: &Fixture) {
    fx.git.run(["config", "--unset", "core.hooksPath"]).unwrap();
    let hooks = fx.root().join(".git/hooks");
    std::fs::create_dir_all(&hooks).unwrap();
    for name in ["pre-commit", "commit-msg"] {
        std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_athena"), hooks.join(name)).unwrap();
    }
}

fn commit(fx: &Fixture, message: &str) -> std::process::Output {
    fx.git.run(["add", "-A"]).unwrap();
    Command::new("git")
        .arg("-C")
        .arg(fx.root())
        .args(["commit", "-q", "-m", message])
        .output()
        .unwrap()
}

#[test]
fn the_pre_commit_hook_refuses_a_generated_token() {
    // M1 (blocks): a commit with a generated token. (allows): a clean commit.
    let fx = Fixture::new();
    install_hooks(&fx);
    let token = format!(
        "{}{}",
        ["gh", "p_"].concat(),
        "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aYuE2i"
    );
    write(fx.root(), &[("notes.txt", &format!("auth: {token}\n"))]);
    let out = commit(&fx, "notes\n\nAthena-Job: bootstrap");
    assert!(!out.status.success(), "the token is refused");
    assert!(String::from_utf8_lossy(&out.stderr).contains("no secrets"));
    write(fx.root(), &[("notes.txt", "nothing secret\n")]);
    let out = commit(&fx, "notes\n\nAthena-Job: bootstrap");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn the_commit_msg_hook_refuses_undeclared_hand_work() {
    // M6 (blocks): a hand change without an intervention record, through git itself.
    let fx = Fixture::new();
    // Close the bootstrap range in this fixture, so only verified or declared changes may follow.
    let through = fx.head();
    let outcome = format!(
        "{{\n  \"schema\": \"athena.outcome/1\",\n  \"job\": \"bootstrap\",\n  \"state\": \"closed\",\n  \"reason\": \"accepted\",\n  \"at\": \"2026-10-10T18:00:00Z\",\n  \"totals\": null,\n  \"branch\": null,\n  \"base_commit\": null,\n  \"candidate_commit\": null,\n  \"after_blocked\": false,\n  \"through_commit\": \"{through}\"\n}}\n"
    );
    write(
        fx.root(),
        &[("records/jobs/bootstrap/outcome.json", &outcome)],
    );
    install_hooks(&fx);
    let out = commit(&fx, "Close the bootstrap range\n\nAthena-Job: bootstrap");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    write(
        fx.root(),
        &[("src/lib.rs", "pub fn one() -> u32 {\n    2\n}\n")],
    );
    let out = commit(&fx, "Edit by hand\n\nAthena-Job: bootstrap");
    assert!(
        !out.status.success(),
        "a bootstrap commit after the range closed is refused"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("governed content"), "{stderr}");
}

#[test]
fn the_tool_hook_denies_escapes_and_stays_silent_otherwise() {
    // M11 (blocks/allows), through the binary as Claude Code runs it.
    let snapshot = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(snapshot.path().join("src")).unwrap();
    let policy = snapshot.path().join("../policy.json");
    let policy_json = serde_json::json!({
        "role": "verifier",
        "root": snapshot.path(),
        "commands": { "cargo": ["test"], "git": ["status", "diff", "log", "show"], "ls": ["*"] },
        "writes": false
    });
    std::fs::write(&policy, policy_json.to_string()).unwrap();
    let ask = |command: &str| -> String {
        let mut child = athena()
            .args(["hook", "pre-tool-use", "--policy"])
            .arg(&policy)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let event = serde_json::json!({ "tool_name": "Bash", "tool_input": { "command": command }, "cwd": snapshot.path() });
        child
            .stdin
            .take()
            .unwrap()
            .write_all(event.to_string().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    assert_eq!(ask("cargo test"), "", "no objection is silent");
    let denied = ask("git -C /home log");
    assert!(
        denied.contains("\"permissionDecision\":\"deny\""),
        "{denied}"
    );
    assert!(ask("cargo test -- --ignored").contains("deny"));
    assert!(
        ask("rm -rf src").contains("deny"),
        "the verifier does not write"
    );
    let _ = std::fs::remove_file(policy);
}

#[test]
fn a_nested_run_works_in_a_scratch_copy_with_fake_agents() {
    // M25 (blocks): a nested `ask` leaves the repository it ran in at its commit. (allows): a nested `ask`
    // and `go` run with fake agents in a scratch repository.
    let fx = Fixture::new();
    let before = fx.head();
    let scratch = tempfile::tempdir().unwrap();
    let out = athena()
        .current_dir(fx.root())
        .env("ATHENA_NESTED", "20261010-153012-k7q2")
        .env("ATHENA_SCRATCH_DIR", scratch.path())
        .args(["ask", "write down a note"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stderr}\n{stdout}");
    assert!(stderr.contains("scratch copy"), "{stderr}");
    let job = stdout
        .lines()
        .find_map(|l| l.strip_prefix("athena · job "))
        .and_then(|l| l.split(' ').next())
        .expect("a job id")
        .to_string();
    let out = athena()
        .current_dir(fx.root())
        .env("ATHENA_NESTED", "20261010-153012-k7q2")
        .env("ATHENA_SCRATCH_DIR", scratch.path())
        .args(["go", &job])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(fx.head(), before, "the repository it ran in did not move");
    let status = fx.git.run(["status", "--porcelain"]).unwrap();
    assert_eq!(status, "", "and has no stray files");
    let scratch_git = athena_git::Git::at(scratch.path().join("repo"));
    let landed = scratch_git.run(["log", "-1", "--format=%B"]).unwrap();
    assert!(
        landed.contains("Athena-Verified-Content"),
        "the scratch copy has the landing commit"
    );
    assert!(scratch.path().join("repo/FAKE-CHANGE.md").exists());
}

#[test]
fn status_and_check_work_from_the_repository_alone() {
    let fx = Fixture::new();
    let out = athena()
        .current_dir(fx.root())
        .args(["status", "--json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let view: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(view["bootstrap"]["range_open"], true);
    let out = athena()
        .current_dir(fx.root())
        .args(["check", "--history"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn ask_later_queues_a_request_without_calling_a_model() {
    // Capability 10, point 3 of the fix round after acceptance verification 1: a request queued for later
    // is open work in `athena status`, from the records alone, with the command that picks it up.
    let fx = Fixture::new();
    let state = tempfile::tempdir().unwrap();
    let out = athena()
        .current_dir(fx.root())
        .env("ATHENA_STATE_DIR", state.path())
        .args(["ask", "--later", "write down a note"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.starts_with("queued"), "{stdout}");
    let out = athena()
        .current_dir(fx.root())
        .env("ATHENA_STATE_DIR", state.path())
        .args(["status", "--json"])
        .output()
        .unwrap();
    let view: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let open = view["open"].as_array().unwrap();
    assert_eq!(open.len(), 1, "{view}");
    assert_eq!(open[0]["state"], "queued");
    assert_eq!(open[0]["title"], "write down a note");
    let id = open[0]["id"].as_str().unwrap();
    assert!(
        view["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.as_str().unwrap().starts_with(&format!("athena go {id}"))),
        "{view}"
    );
}

#[test]
fn check_reports_a_configuration_that_does_not_parse() {
    // M6 (blocks), N1 of acceptance verification 2: a prefix exemption committed without hooks does not
    // stop the history check, which reports it and still judges the commits after it.
    let fx = Fixture::new();
    let config = fx.file("athena.toml").unwrap();
    let broken: String = config
        .lines()
        .map(|l| {
            if l.starts_with("exempt = ") {
                "exempt = [\"0\"]".to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    write(fx.root(), &[("athena.toml", &broken)]);
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run([
            "commit",
            "-q",
            "-m",
            "Exempt everything\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
    let state = tempfile::tempdir().unwrap();
    let out = athena()
        .current_dir(fx.root())
        .env("ATHENA_STATE_DIR", state.path())
        .args(["check", "--history"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(2),
        "violations, not an error: {}{stdout}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("is not a full commit id"), "{stdout}");
}

#[test]
fn check_help_lists_attribution_and_it_conflicts_with_the_other_modes() {
    let fx = Fixture::new();
    let help = athena()
        .current_dir(fx.root())
        .args(["check", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(
        String::from_utf8_lossy(&help.stdout).contains("--attribution"),
        "{}",
        String::from_utf8_lossy(&help.stdout)
    );
    for args in [
        vec!["check", "--attribution", "--history"],
        vec!["check", "--attribution", "--pr-head", "a", "b"],
    ] {
        let out = athena()
            .current_dir(fx.root())
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("cannot be used with"), "{args:?}: {stderr}");
    }
}

#[test]
fn check_attribution_without_a_records_branch_fails_explicitly() {
    let fx = Fixture::new();
    let state = tempfile::tempdir().unwrap();
    let out = athena()
        .current_dir(fx.root())
        .env("ATHENA_STATE_DIR", state.path())
        .args(["check", "--attribution"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(2), "{stdout}");
    assert!(stdout.contains("records branch missing"), "{stdout}");
    assert!(stdout.contains("attribution FAIL"), "{stdout}");
}

// ---- Incremental attribution checking with checkpoints (docs/design/records-history.md §9; D111) ----

mod incremental {
    use super::*;
    use athena_data::history::{
        Checkpoint, HistoryRecord, Landing, LandingCommit, LandingIntent, LandingKind, MergeMethod,
    };
    use athena_data::{JobId, RecordId};
    use athena_git::records_branch::{self, Change, RECORDS_REF, RecordsWriter};

    fn ts() -> jiff::Timestamp {
        "2026-10-24T10:00:00Z".parse().unwrap()
    }

    fn put(fx: &Fixture, dir: &str, id: &RecordId, record: HistoryRecord) {
        RecordsWriter::new(&fx.git)
            .append(
                "record",
                &[Change::put(
                    format!("{dir}/{id}.json"),
                    serde_json::to_string(&record).unwrap(),
                )],
            )
            .unwrap();
    }

    /// Adds a squash commit on main with its landing-intent and landing record; returns the commit.
    fn land(fx: &Fixture, n: u32) -> String {
        let base = fx.head();
        write(fx.root(), &[(&format!("landed{n}.txt"), "x\n")]);
        assert!(commit(fx, "Squash of a pull request").status.success());
        let sha = fx.head();
        let intent_id = RecordId::from_parts(ts(), 2 * n);
        let intent = LandingIntent {
            id: intent_id.clone(),
            created_at: ts(),
            pr_number: 1,
            head_sha: "1".repeat(40),
            head_tree_sha: fx.git.tree_of(&sha).unwrap(),
            expected_base: base,
            job_ids: vec![JobId::parse("20261024-100000-aaaa").unwrap()],
        };
        put(
            fx,
            "landing-intents",
            &intent_id,
            HistoryRecord::LandingIntent(intent),
        );
        let landing_id = RecordId::from_parts(ts(), 2 * n + 1);
        let landing = Landing {
            id: landing_id.clone(),
            created_at: ts(),
            pr_number: 1,
            head_sha: "1".repeat(40),
            host_commit: sha.clone(),
            merge_method: MergeMethod::Squash,
            commits: vec![LandingCommit {
                sha: "1".repeat(40),
                tree: "2".repeat(40),
                kind: LandingKind::RecordsOnly,
                job: None,
                verification: None,
            }],
            intent: intent_id,
        };
        put(fx, "landings", &landing_id, HistoryRecord::Landing(landing));
        sha
    }

    fn tip(fx: &Fixture) -> String {
        fx.git.rev_parse(RECORDS_REF).unwrap().unwrap()
    }

    /// `athena check --attribution` with the given extra arguments and environment.
    fn check(fx: &Fixture, args: &[&str], ci: Option<&str>) -> std::process::Output {
        let mut cmd = athena();
        cmd.current_dir(fx.root())
            .env("ATHENA_STATE_DIR", fx.state.path())
            .env_remove("CI")
            .args(["check", "--attribution"])
            .args(args);
        if let Some(value) = ci {
            cmd.env("CI", value);
        }
        cmd.output().unwrap()
    }

    fn newest_checkpoint(fx: &Fixture) -> Checkpoint {
        let tip = tip(fx);
        let path = records_branch::newest_added_under(&fx.git, &tip, "checkpoints")
            .unwrap()
            .expect("a checkpoint exists");
        let bytes = fx.git.show_file(&tip, &path).unwrap().unwrap();
        match HistoryRecord::parse(&String::from_utf8(bytes).unwrap()).unwrap() {
            HistoryRecord::Checkpoint(cp) => cp,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn check_attribution_writes_checkpoint() {
        let fx = Fixture::new();
        land(&fx, 1);
        land(&fx, 2);
        let before = tip(&fx);
        let out = check(&fx, &[], None);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(out.status.code(), Some(0), "{stdout}");
        assert!(stdout.contains("attribution pass"), "{stdout}");
        assert!(stdout.contains("checkpoint: written"), "{stdout}");
        let cp = newest_checkpoint(&fx);
        assert_eq!(cp.main_sha, fx.head());
        assert_eq!(cp.records_sha, before);
        assert_eq!(cp.verifier_version, athena_gate::attribution::CHECK_VERSION);
        assert_ne!(tip(&fx), before);

        // A failing check writes none.
        write(fx.root(), &[("rogue.txt", "r\n")]);
        assert!(commit(&fx, "Hand-made commit").status.success());
        let before = tip(&fx);
        let out = check(&fx, &[], None);
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(tip(&fx), before);
    }

    #[test]
    fn check_attribution_in_ci_writes_nothing() {
        let fx = Fixture::new();
        land(&fx, 1);
        let before = tip(&fx);
        let out = check(&fx, &[], Some("true"));
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(out.status.code(), Some(0), "{stdout}");
        assert!(stdout.contains("not written (CI is read-only)"), "{stdout}");
        assert_eq!(tip(&fx), before, "CI never writes the records branch");
        // The flag does the same where CI is not set.
        let out = check(&fx, &["--no-checkpoint"], None);
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(tip(&fx), before);
        // An empty CI variable is not CI.
        let out = check(&fx, &[], Some(""));
        assert_eq!(out.status.code(), Some(0));
        assert_ne!(tip(&fx), before);
    }

    #[test]
    fn check_attribution_reports_seconds_and_counts() {
        let fx = Fixture::new();
        land(&fx, 1);
        land(&fx, 2);
        let out = check(&fx, &[], None);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("mode: full replay, because no checkpoint"),
            "{stdout}"
        );
        assert!(
            stdout.contains(" s; judged 2 commit(s) and 4 record(s)"),
            "{stdout}"
        );
        assert!(stdout.contains("took "), "{stdout}");

        land(&fx, 3);
        let out = check(&fx, &[], None);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("mode: incremental"), "{stdout}");
        assert!(
            stdout.contains(" s; judged 1 commit(s) and 2 record(s)"),
            "{stdout}"
        );

        land(&fx, 4);
        let out = check(&fx, &["--json"], None);
        let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(json["mode"], "incremental");
        assert_eq!(json["commits_judged"], 1);
        assert_eq!(json["records_judged"], 2);
        assert!(json["seconds"].as_f64().unwrap() >= 0.0);
        assert!(json["checkpoint_written"].is_string());
        assert_eq!(json["passed"], true);

        // A full replay judges everything and says so.
        let out = check(&fx, &["--full-replay", "--no-checkpoint", "--json"], None);
        let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(json["mode"], "full replay");
        assert_eq!(json["commits_judged"], 4);
        assert_eq!(json["records_judged"], 8);
        assert!(json["checkpoint_written"].is_null());
    }

    #[test]
    fn full_replay_tampered_checkpoint_exits_2() {
        let fx = Fixture::new();
        land(&fx, 1);
        write(fx.root(), &[("rogue.txt", "r\n")]);
        assert!(commit(&fx, "Hand-made commit").status.success());
        let forged = Checkpoint {
            id: RecordId::from_parts(ts(), 900),
            created_at: ts(),
            main_sha: fx.head(),
            records_sha: tip(&fx),
            verifier_version: athena_gate::attribution::CHECK_VERSION.to_string(),
        };
        put(
            &fx,
            "checkpoints",
            &forged.id.clone(),
            HistoryRecord::Checkpoint(forged),
        );

        // The incremental run trusts the forged checkpoint; the full replay does not.
        let out = check(&fx, &["--no-checkpoint"], None);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
        let out = check(&fx, &["--full-replay", "--no-checkpoint"], None);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert_eq!(out.status.code(), Some(2), "{stdout}");
        assert!(
            stdout.contains("checkpoint disagrees with the full replay"),
            "{stdout}"
        );
    }
}
