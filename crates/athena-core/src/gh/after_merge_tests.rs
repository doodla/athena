use super::*;
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
    assert!(at("rebuilt on the merged commit") < at("pages: looked for a run"));
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
