//! The 'fmt, clippy, tests' job's reuse of its own earlier pass (docs/decisions.md D121): the fingerprint
//! command, and the decisions behind the workflow's lookup, mark and report steps. Every test runs the built
//! `xtask` binary, as the workflow does.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn xtask(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    assert!(
        out.status.success(),
        "xtask failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=tester", "-c", "user.email=tester"])
        .args([
            "-c",
            "core.attributesFile=/dev/null",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .env("GIT_ATTR_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(dir: &Path, path: &str, text: &str) {
    let file = dir.join(path);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

/// A repository with governed code, a records/ directory and owner paths (excluded and read-only).
fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    write(
        dir.path(),
        ".gitattributes",
        "/FRONT.md athena-exclude\n/SPEC.md athena-readonly\n",
    );
    write(dir.path(), "FRONT.md", "front door");
    write(dir.path(), "SPEC.md", "the spec");
    write(dir.path(), "records/jobs/x.json", "{}");
    write(dir.path(), "src/main.rs", "fn main() {}\n");
    commit(dir.path(), "first");
    dir
}

fn fingerprint(dir: &Path) -> String {
    stdout(&xtask(dir, &["fingerprint"])).trim().to_string()
}

#[test]
fn fingerprint_ignores_records_and_owner_paths() {
    let repo = repo();
    let dir = repo.path();
    let first = fingerprint(dir);
    assert!(first.starts_with("sha256:"), "{first}");

    write(dir, "records/jobs/y.json", "{\"more\":1}");
    commit(dir, "records only");
    assert_eq!(fingerprint(dir), first, "a records-only change");

    write(dir, "FRONT.md", "a new front door");
    write(dir, "SPEC.md", "an edited spec");
    commit(dir, "owner paths only");
    assert_eq!(fingerprint(dir), first, "an owner-path change");
}

#[test]
fn fingerprint_changes_with_governed_content() {
    let repo = repo();
    let dir = repo.path();
    let first = fingerprint(dir);
    write(dir, "src/main.rs", "fn main() { println!(); }\n");
    commit(dir, "code change");
    assert_ne!(fingerprint(dir), first);
}

#[test]
fn fingerprint_outside_a_repository_fails() {
    let dir = tempfile::tempdir().unwrap();
    let out = xtask(dir.path(), &["fingerprint"]);
    assert!(!out.status.success());
}

const FP: &str = "sha256:abc123";

fn check(cache: &Path, fingerprint: &str) -> String {
    let dir = cache.parent().unwrap();
    stdout(&xtask(
        dir,
        &["ci-cache", "check", cache.to_str().unwrap(), fingerprint],
    ))
    .trim()
    .to_string()
}

fn mark(cache: &Path, fingerprint: &str, outcome: &str) -> String {
    let dir = cache.parent().unwrap();
    stdout(&xtask(
        dir,
        &[
            "ci-cache",
            "mark",
            cache.to_str().unwrap(),
            fingerprint,
            "--outcome",
            outcome,
        ],
    ))
}

fn cache_dir(root: &Path) -> PathBuf {
    root.join("ci-pass")
}

#[test]
fn ci_cache_a_missing_entry_runs_the_full_job() {
    let root = tempfile::tempdir().unwrap();
    // No directory restored at all (a cache miss).
    assert_eq!(check(&cache_dir(root.path()), FP), "skip=false");
    // An empty directory (a hit on an entry that holds no marker).
    std::fs::create_dir_all(cache_dir(root.path())).unwrap();
    assert_eq!(check(&cache_dir(root.path()), FP), "skip=false");
}

#[test]
fn ci_cache_a_marker_for_another_fingerprint_does_not_skip() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache_dir(root.path());
    mark(&cache, "sha256:other", "success");
    assert_eq!(check(&cache, FP), "skip=false");
    assert_eq!(check(&cache, ""), "skip=false", "an empty fingerprint");
}

#[test]
fn ci_cache_a_failed_run_writes_no_marker_and_the_next_run_is_full() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache_dir(root.path());
    assert_eq!(check(&cache, FP), "skip=false", "first run: a miss");
    for outcome in ["failure", "cancelled", "skipped", ""] {
        let said = mark(&cache, FP, outcome);
        assert!(said.contains("not marked"), "{outcome}: {said}");
        assert!(!cache.exists(), "{outcome} must write nothing");
    }
    assert_eq!(
        check(&cache, FP),
        "skip=false",
        "after failures: still full"
    );
}

#[test]
fn ci_cache_a_pass_is_saved_and_the_same_fingerprint_then_skips() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache_dir(root.path());
    assert_eq!(check(&cache, FP), "skip=false");
    assert!(mark(&cache, FP, "success").contains("marked"));
    assert!(cache.join("passed").is_file());
    assert_eq!(check(&cache, FP), "skip=true", "a hit of a passing entry");
    assert_eq!(
        check(&cache, "sha256:changed"),
        "skip=false",
        "changed governed content"
    );
}

#[test]
fn ci_cache_check_appends_its_answer_to_the_output_file() {
    let root = tempfile::tempdir().unwrap();
    let cache = cache_dir(root.path());
    let out = root.path().join("github_output");
    std::fs::write(&out, "fingerprint=x\n").unwrap();
    let run = |dir: &Path| {
        stdout(&xtask(
            root.path(),
            &[
                "ci-cache",
                "check",
                dir.to_str().unwrap(),
                FP,
                "--output",
                out.to_str().unwrap(),
            ],
        ))
    };
    run(&cache);
    mark(&cache, FP, "success");
    run(&cache);
    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        "fingerprint=x\nskip=false\nskip=true\n"
    );
}

#[test]
fn ci_cache_report_names_the_fingerprint_and_the_matched_key() {
    let root = tempfile::tempdir().unwrap();
    let summary = root.path().join("summary.md");
    let said = stdout(&xtask(
        root.path(),
        &[
            "ci-cache",
            "report",
            FP,
            "ci-pass-abc123",
            "--summary",
            summary.to_str().unwrap(),
        ],
    ));
    let written = std::fs::read_to_string(&summary).unwrap();
    for text in [&said, &written] {
        assert!(text.contains(FP), "{text}");
        assert!(text.contains("ci-pass-abc123"), "{text}");
        assert!(text.contains("skipped"), "{text}");
    }
}

#[test]
fn ci_cache_bad_arguments_are_refused() {
    let root = tempfile::tempdir().unwrap();
    assert!(!xtask(root.path(), &["ci-cache"]).status.success());
    assert!(
        !xtask(root.path(), &["ci-cache", "mark", "d", FP])
            .status
            .success(),
        "mark without --outcome"
    );
}
