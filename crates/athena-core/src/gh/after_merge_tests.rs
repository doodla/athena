use super::*;
use crate::Athena;
use crate::code_health::tests::{big_file, land, open_requests, set_up};
use std::cell::RefCell;
use std::rc::Rc;

/// One log that the host fake and the UI both write to, so their order can be read back.
type Log = Rc<RefCell<Vec<String>>>;

struct Pages(Log);

impl PagesHost for Pages {
    fn pages_runs(&self, _: &str) -> Result<Vec<PagesRun>> {
        self.0.borrow_mut().push("pages: looked for a run".into());
        Ok(vec![])
    }
    fn rerun_pages(&self, _: u64) -> Result<()> {
        Ok(())
    }
}

struct Said(Log);

impl Ui for Said {
    fn line(&self, text: &str) {
        self.0.borrow_mut().push(text.to_string());
    }
}

fn report(state: LinkState, merged: Option<&str>) -> PrReport {
    PrReport {
        link: PrLink {
            pr_number: 7,
            host: "github.com".into(),
            repository: "example/project".into(),
            head: "h".repeat(40),
            job: "j1".into(),
            state,
            merged_commit: merged.map(str::to_string),
            base_jobs: Vec::new(),
        },
        checks: None,
        merge_requested: true,
        reruns: 0,
        failing_test: None,
        warnings: vec![],
    }
}

fn poll() -> Poll<'static> {
    Poll {
        attempts: 1,
        delay: Duration::ZERO,
        sleep: &|_| {},
    }
}

#[test]
fn after_a_merge_the_branch_is_resynced_then_pages_watched_then_code_health_checked() {
    let (_dir, _state, athena) = set_up();
    let now = crate::now();
    let base = athena.head().unwrap();
    land(
        &athena,
        &[("crates/x/src/big.rs", big_file())],
        "j1",
        now,
        0,
    );
    let landed = athena.head().unwrap();
    // The host's squash: another commit with the landed content, on the base.
    let tree = athena.git.tree_of(&landed).unwrap();
    let merged = athena
        .git
        .run([
            "commit-tree",
            tree.trim(),
            "-p",
            &base,
            "-m",
            "Land j1 (#7)\n\nAthena-Job: j1\nAthena-Verified-Content: fp-j1\n",
        ])
        .unwrap()
        .trim()
        .to_string();

    let log: Log = Rc::default();
    athena
        .after_merge(
            &report(LinkState::Landed, Some(&merged)),
            Some(&landed),
            &Pages(log.clone()),
            &poll(),
            &Said(log.clone()),
        )
        .unwrap();

    let seen = log.borrow().clone();
    let at = |needle: &str| {
        seen.iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("no line with {needle:?} in {seen:?}"))
    };
    assert!(at("merged the host's commit") < at("pages: looked for a run"));
    assert!(at("pages: looked for a run") < at("code health"));
    // The branch sits on the merged commit, and the request was queued after it.
    assert!(
        athena
            .git
            .is_ancestor(&merged, &athena.head().unwrap())
            .unwrap()
    );
    let requests = open_requests(&athena);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert!(requests[0].starts_with("Structural pass: crates/x/src/big.rs"));
}

#[test]
fn a_pull_request_that_did_not_land_is_not_checked() {
    let (_dir, _state, athena) = set_up();
    land(
        &athena,
        &[("crates/x/src/big.rs", big_file())],
        "j1",
        crate::now(),
        0,
    );
    let log: Log = Rc::default();
    for state in [LinkState::Open, LinkState::MergedUnverified] {
        athena
            .after_merge(
                &report(state, Some(&"m".repeat(40))),
                None,
                &Pages(log.clone()),
                &poll(),
                &Said(log.clone()),
            )
            .unwrap();
    }
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());
    assert!(open_requests(&athena).is_empty());
}

/// A bare repository added to `athena`'s work repository as `origin`.
fn add_bare_origin(athena: &Athena) -> (tempfile::TempDir, athena_git::Git) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    athena.git.run(["init", "--bare", "-q", path]).unwrap();
    athena.git.run(["remote", "add", "origin", path]).unwrap();
    let bare = athena_git::Git::at(dir.path());
    (dir, bare)
}

/// The host's squash of the landed commit `landed`: its tree on top of `base`.
fn squash_of(athena: &Athena, landed: &str, base: &str) -> String {
    let tree = athena.git.tree_of(landed).unwrap();
    athena
        .git
        .commit_tree(&tree, &[base], "Land j1 (#7)\n", false)
        .unwrap()
}

#[test]
fn after_a_merge_the_working_branch_is_pushed_and_a_report_that_did_not_land_pushes_nothing() {
    let (_dir, _state, athena) = set_up();
    athena.git.run(["checkout", "-q", "-b", "work"]).unwrap();
    let base = athena.head().unwrap();
    let (_origin, bare) = add_bare_origin(&athena);
    athena.git.push_branch("origin", "work", &base).unwrap();
    land(
        &athena,
        &[("crates/x/src/a.rs", "pub fn a() {}\n".to_string())],
        "j1",
        crate::now(),
        0,
    );
    let landed = athena.head().unwrap();
    let merged = squash_of(&athena, &landed, &base);

    // A report that did not land leaves the remote copy where it was.
    let quiet: Log = Rc::default();
    athena
        .after_merge(
            &report(LinkState::Open, Some(&merged)),
            Some(&landed),
            &Pages(quiet.clone()),
            &poll(),
            &Said(quiet.clone()),
        )
        .unwrap();
    assert_eq!(
        bare.rev_parse("refs/heads/work").unwrap(),
        Some(base.clone())
    );
    assert!(quiet.borrow().is_empty(), "{:?}", quiet.borrow());

    let log: Log = Rc::default();
    athena
        .after_merge(
            &report(LinkState::Landed, Some(&merged)),
            Some(&landed),
            &Pages(log.clone()),
            &poll(),
            &Said(log.clone()),
        )
        .unwrap();
    let seen = log.borrow().clone();
    let at = |needle: &str| {
        seen.iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("no line with {needle:?} in {seen:?}"))
    };
    assert!(at("merged the host's commit") < at("pushed work to origin"));
    assert!(at("pushed work to origin") < at("pages: looked for a run"));
    // The remote copy now holds the merge: it descends from its earlier head and from the squash, and the
    // branch (which the code-health check may add a request commit to afterwards) descends from it.
    let pushed = bare.rev_parse("refs/heads/work").unwrap().unwrap();
    assert_ne!(pushed, base);
    assert!(athena.git.is_ancestor(&base, &pushed).unwrap());
    assert!(athena.git.is_ancestor(&landed, &pushed).unwrap());
    assert!(athena.git.is_ancestor(&merged, &pushed).unwrap());
    assert!(
        athena
            .git
            .is_ancestor(&pushed, &athena.head().unwrap())
            .unwrap()
    );
}

#[test]
fn a_failed_push_after_a_merge_is_a_warning_and_the_merge_stands() {
    let (_dir, _state, athena) = set_up();
    athena.git.run(["checkout", "-q", "-b", "work"]).unwrap();
    let base = athena.head().unwrap();
    // The remote is a path that does not exist.
    athena
        .git
        .run(["remote", "add", "origin", "/nonexistent/origin.git"])
        .unwrap();
    land(
        &athena,
        &[("crates/x/src/a.rs", "pub fn a() {}\n".to_string())],
        "j1",
        crate::now(),
        0,
    );
    let landed = athena.head().unwrap();
    let merged = squash_of(&athena, &landed, &base);
    let log: Log = Rc::default();
    athena
        .after_merge(
            &report(LinkState::Landed, Some(&merged)),
            Some(&landed),
            &Pages(log.clone()),
            &poll(),
            &Said(log.clone()),
        )
        .unwrap();
    let seen = log.borrow().clone();
    assert!(
        seen.iter()
            .any(|l| l.starts_with("warning: could not push work to origin")),
        "{seen:?}"
    );
    assert!(
        athena
            .git
            .is_ancestor(&merged, &athena.head().unwrap())
            .unwrap()
    );
}

#[test]
fn push_working_branch_pushes_once_refuses_a_divergent_remote_and_is_silent_without_an_origin() {
    let (_dir, _state, athena) = set_up();
    let log: Log = Rc::default();
    let ui = Said(log.clone());
    // No origin: nothing is pushed and nothing is said.
    athena.git.run(["checkout", "-q", "-b", "work"]).unwrap();
    assert!(!athena.push_working_branch(&ui));
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());

    let (_origin, bare) = add_bare_origin(&athena);
    let head = athena.head().unwrap();
    assert!(athena.push_working_branch(&ui));
    assert_eq!(
        bare.rev_parse("refs/heads/work").unwrap(),
        Some(head.clone())
    );
    assert_eq!(log.borrow().len(), 1, "{:?}", log.borrow());
    // The remote already holds the tip: no second push, no second line.
    assert!(athena.push_working_branch(&ui));
    assert_eq!(log.borrow().len(), 1, "{:?}", log.borrow());

    // A remote copy that holds a commit the branch does not contain is left alone, with a warning.
    let sibling = athena
        .git
        .commit_tree(
            &athena.git.tree_of(&head).unwrap(),
            &[&head],
            "elsewhere\n",
            false,
        )
        .unwrap();
    let ahead = athena
        .git
        .commit_tree(
            &athena.git.tree_of(&head).unwrap(),
            &[&head],
            "here\n",
            false,
        )
        .unwrap();
    athena.git.push_branch("origin", "work", &sibling).unwrap();
    athena.git.run(["reset", "-q", "--hard", &ahead]).unwrap();
    assert!(!athena.push_working_branch(&ui));
    assert_eq!(bare.rev_parse("refs/heads/work").unwrap(), Some(sibling));
    let seen = log.borrow().clone();
    assert!(
        seen.last()
            .is_some_and(|l| l.starts_with("warning: could not push work to origin")),
        "{seen:?}"
    );
}

#[test]
fn the_default_branch_is_not_pushed_after_a_merge() {
    let (_dir, _state, athena) = set_up();
    let base = athena.head().unwrap();
    let (_origin, bare) = add_bare_origin(&athena);
    land(
        &athena,
        &[("crates/x/src/a.rs", "pub fn a() {}\n".to_string())],
        "j1",
        crate::now(),
        0,
    );
    let landed = athena.head().unwrap();
    let merged = squash_of(&athena, &landed, &base);
    let log: Log = Rc::default();
    athena
        .after_merge(
            &report(LinkState::Landed, Some(&merged)),
            Some(&landed),
            &Pages(log.clone()),
            &poll(),
            &Said(log.clone()),
        )
        .unwrap();
    assert_eq!(bare.rev_parse("refs/heads/main").unwrap(), None);
    assert!(
        log.borrow()
            .iter()
            .any(|l| l.starts_with("not pushing main to origin")),
        "{:?}",
        log.borrow()
    );
}
