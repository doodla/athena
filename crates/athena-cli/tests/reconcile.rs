//! `athena status` and `athena land` run the reconcile step first (`docs/design/records-history.md` §8.1, D111):
//! a main commit that no landing record names, that no landing intent matches and that the host cannot name a
//! pull request for is reported as unattributed, and nothing is written. The host is a fake `gh` first on PATH
//! that refuses every call, so no test depends on the network.

mod common;

use athena_core::gh::{GitLandingRecords, LandingRecords};
use athena_git::Git;
use athena_git::records_branch::RECORDS_REF;
use common::*;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

struct World {
    fx: Fixture,
    _hosts: tempfile::TempDir,
    bare: Git,
    bin: std::path::PathBuf,
    stray: String,
}

/// A repository whose `origin` looks like a host repository (`.../github.com/example/project.git`), with a
/// landing intent for a pull request that never merged, and one commit on main that nothing explains.
fn world() -> World {
    let fx = Fixture::new();
    let hosts = tempfile::tempdir().unwrap();
    let origin = hosts.path().join("github.com/example/project.git");
    std::fs::create_dir_all(&origin).unwrap();
    let bare = Git::at(&origin);
    bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
    let git = &fx.git;
    git.run(["remote", "add", "origin", origin.to_str().unwrap()])
        .unwrap();
    git.run(["push", "-q", "origin", "main"]).unwrap();
    git.run(["remote", "set-head", "origin", "main"]).unwrap();
    let base = fx.head();

    // A pull request head one commit above the base, carrying a job trailer; its intent is on the records branch.
    git.run(["checkout", "-q", "--detach", &base]).unwrap();
    write(fx.root(), &[("pr1.txt", "pr1\n")]);
    git.run(["add", "-A"]).unwrap();
    git.run([
        "commit",
        "-q",
        "-m",
        "Pull request\n\nAthena-Job: 20261024-114125-7z2t",
    ])
    .unwrap();
    let head = fx.head();
    git.run(["push", "-q", "origin", &format!("{head}:refs/heads/pr1")])
        .unwrap();
    git.run(["checkout", "-q", "main"]).unwrap();
    GitLandingRecords::for_remote(git, "origin")
        .unwrap()
        .intent(1, &head, &base)
        .unwrap();

    // A commit by hand on main after the intent's base.
    write(fx.root(), &[("by-hand.txt", "by hand\n")]);
    git.run(["add", "-A"]).unwrap();
    git.run(["commit", "-q", "-m", "By hand"]).unwrap();
    git.run(["push", "-q", "origin", "main"]).unwrap();
    let stray = fx.head();

    // A host that refuses every call.
    let bin = hosts.path().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let gh = bin.join("gh");
    std::fs::write(
        &gh,
        "#!/bin/sh\necho 'gh: Not Found (HTTP 404)' >&2\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).unwrap();
    World {
        fx,
        _hosts: hosts,
        bare,
        bin,
        stray,
    }
}

fn athena(w: &World, args: &[&str]) -> std::process::Output {
    let path = format!(
        "{}:{}",
        w.bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    Command::new(env!("CARGO_BIN_EXE_athena"))
        .current_dir(w.fx.root())
        .env("ATHENA_STATE_DIR", w.fx.state.path())
        .env("PATH", path)
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_JEV_SHADOW")
        .args(args)
        .output()
        .unwrap()
}

fn landings_written(w: &World) -> usize {
    let tip = w.bare.rev_parse(RECORDS_REF).unwrap().unwrap();
    w.bare.ls_tree_under(&tip, "landings").unwrap().len()
}

#[test]
fn status_reports_an_unattributed_main_commit_and_writes_nothing() {
    let w = world();
    let tip_before = w.bare.rev_parse(RECORDS_REF).unwrap();
    let out = athena(&w, &["status"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("unattributed"), "{stdout}");
    assert!(stdout.contains(&w.stray), "{stdout}");
    assert_eq!(landings_written(&w), 0);
    assert_eq!(w.bare.rev_parse(RECORDS_REF).unwrap(), tip_before);

    let json = athena(&w, &["status", "--json"]);
    let text = String::from_utf8_lossy(&json.stdout);
    assert!(text.contains("\"outcome\": \"unattributed\""), "{text}");
}

#[test]
fn land_reports_an_unattributed_main_commit_before_it_does_anything_else() {
    let w = world();
    // The job does not exist, so the landing fails; the reconcile lines come first.
    let out = athena(&w, &["land", "no-such-job", "--pr"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unattributed"), "{stderr}");
    assert!(stderr.contains(&w.stray), "{stderr}");
    assert_eq!(landings_written(&w), 0);
}
