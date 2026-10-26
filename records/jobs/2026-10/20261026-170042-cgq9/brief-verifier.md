# Job 20261026-170042-cgq9: verify "Move records/ readers to athena/records worktree with per-reader tests"

## Request (verbatim)

> try again on 20261026-155343-tvfc

## Reading (accepted by the user)

Retry of job tvfc: move every records/ reader to the athena/records worktree with in-tree fallback, with a worktree test and a fallback test per reader. Test the xtask walltime reader inside xtask, test land_pr in gh.rs, and use distinct job ids so status, audit and land_pr tests show which record was read. Audience: Athena operators and the verifier.

Assumptions:
- Goal: complete D111 job 7 part 2 (readers of records/ move to the athena/records worktree with in-tree fallback); audience is Athena operators and the verifier.
- Production move is as in the previously accepted reading; the retry's key change is test placement: walltime tests in xtask, land_pr tests in gh.rs.
- I could not inspect the repo in detail; deliverable list follows the earlier accepted list.
- site.rs reads studies/, not records/, and stays untouched.
- Depends on the cutover job 20261025-001232-vesh having landed.

## Acceptance criteria

1. [behavior] In crates/athena-cli/tests/records_branch_readers.rs, each athena-cli-reachable reader (Records::load_current, RecordSet::load_current, read_records, introspect walks, scoring, forecasting, retry carry, auditor, proposals, backtest) has a named worktree test planting different records in the worktree and in-tree and asserting the worktree's are returned. Run cargo test --locked.
2. [behavior] Each of those readers has a named fallback test with no worktree asserting in-tree records/ is returned, equal to the pre-change result. Run cargo test --locked.
3. [behavior] xtask's own tests contain worktree_walltime and fallback_walltime (not in the athena-cli test file) plus a linked-worktree test finding the records worktree via git's common dir. Run cargo test -p xtask --locked and confirm a nonzero number of tests ran.
4. [behavior] land_pr tests (in gh.rs, where Rig/FakeHost are reachable) use distinct job ids in worktree and in-tree records: resolve succeeds for the worktree-only id and fails with 'unknown job' for the in-tree-only id; under fallback the reverse. Run cargo test --locked land_pr.
5. [behavior] A test runs the built athena binary's status, audit and a forecast on a scratch checkout whose working tree has no records/; worktree and in-tree records have distinct job ids, and status and audit output contain the worktree's id and not the in-tree one; all succeed. Run cargo test --locked.
6. [behavior] Tests create the records worktree under a temporary state directory; scoring's use of the worktree regardless of the head passed is documented in docs/decisions.md and tested.
7. [behavior] xtask records_reader_scan still fails on planted reads, including crate::store::Records::load( paths, ignores citation strings in steward.rs and work_decisions, and allow-list entries are narrowed to call sites. Run cargo test -p xtask --locked records_reader_scan and confirm a nonzero number of tests ran.
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test --locked pass; git diff shows xtask/src/site.rs unchanged.
9. [content] The change description lists every reader found by grep, maps each to its worktree and fallback test names and the observed marker, states the walltime (in xtask) and land_pr (in gh.rs) placements, says common-dir derivation is new code and not a repair, and cites commands actually run.
10. [content] CHANGELOG.md records the reader move, the fallback, the allow-list, and that the site generator is out of scope, citing D111.

## Expected deliverables

- crates/athena-cli/tests/records_branch_readers.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/paths.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- xtask/src/walltime.rs (modify)
- xtask/src/main.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-gate/src/audit.rs (modify)

## Out of scope

- Rerooting the study site generator xtask/src/site.rs
- Changing the cutover itself (job vesh)
- Scanning citation strings in steward.rs or work_decisions

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Per-reader tests; fmt, clippy and full test criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Binary status/audit/forecast exercised on a scratch checkout.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests plant distinct records and assert the worktree's are returned.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: records_reader_scan enforces the reader rule with a planted-read test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Change description and docs written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check readers returning worktree data, not wiring.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Macro-generated worktree_<reader> integration tests plant W in the worktree and T in-tree. They cover records_load_current, recordset_load_current, read_records, introspect_walks, scoring, forecasting, retry_carry, auditor, proposals and backtest, plus worktree_status and worktree_audit via the binary. Run: `cargo test --locked`. Observes: Every worktree_ test passes, asserting W is returned and T is not.
  Fails without the behavior: A reader still reading in-tree or the passed head returns T and fails.
- Criterion 2: Macro-generated fallback_<reader> tests for the same readers with no worktree. They assert T is returned and equals the pre-change result (Records::load at head, read_records(root)). Run: `cargo test --locked`. Observes: All fallback_ tests pass with T, equal to the pre-change result.
  Fails without the behavior: A reader that errors or returns empty without a worktree fails.
- Criterion 3: xtask unit tests worktree_walltime, fallback_walltime and linked_worktree_walltime. The last runs git worktree add for a linked checkout and finds the records worktree via Git::common_dir(). Run: `cargo test -p xtask --locked`. Observes: A nonzero number of tests ran. The worktree test returns W only, the fallback test T only, and the linked test W.
  Fails without the behavior: Reading <root>/.git or in-tree only fails the worktree and linked tests.
- Criterion 4: gh.rs tests worktree_land_pr and fallback_land_pr with Rig/FakeHost, distinct ids, and a refusing runner. They call Athena::land_pr and inspect the error. The wording difference ('no job matches' vs 'unknown job') is noted. Run: `cargo test --locked land_pr`. Observes: Worktree: the W id errors but not with no-match, and the T id errors with no-match. Fallback: the reverse.
  Fails without the behavior: A land_pr still reading in-tree would give no-match for W under the worktree and the test fails.
- Criterion 5: scratch_checkout runs the built athena status, audit and a forecast on a scratch checkout with no records/ in its tree, with ATHENA_STATE_DIR set to a tempdir that holds the worktree (W) and in-tree records (T). Run: `cargo test --locked scratch_checkout`. Observes: All exit 0; status and audit stdout contain W and not T.
  Fails without the behavior: A reader needing working-tree records/ fails or prints T or nothing.
- Criterion 6: The fixture builds the worktree path with repo_state_dir under a tempdir ATHENA_STATE_DIR and asserts it is under that tempdir. scoring_ignores_passed_head passes a head unlike the worktree tip. docs/decisions.md records the rule. Run: `cargo test --locked scoring_ignores_passed_head`. Observes: The test passes, nothing is created under the real state root, and the decision text is in the diff.
  Fails without the behavior: A closure honouring the passed head returns main's records and the test fails.
- Criterion 7: xtask tests records_reader_scan_reports_planted_read (including a crate::store::Records::load( path), records_reader_scan_ignores_citations, records_reader_scan_real_tree_clean (with the call-site allow-list) and an allow-list narrowness check. Run: `cargo test -p xtask --locked records_reader_scan`. Observes: At least 3 tests ran and passed.
  Fails without the behavior: A scan missing planted reads fails the planted-read test.
- Criterion 8: Run the three checks, then inspect the diff for site.rs. Run: `cargo test --locked`. Observes: cargo fmt --all --check and clippy -D warnings pass, the tests pass, and git diff --stat lists no xtask/src/site.rs.
  Fails without the behavior: A broken test, lint error or site.rs edit fails one of these.
- Criterion 9: Read the change description for the grep list, the reader-to-test map with W/T markers, the placements, the new-wiring statement, the unknown-job wording note, the follow-up allow-list entries and the commands run. Check that the test names exist. Run: `cargo test --locked -- --list`. Observes: Every cited test name appears in the list; each grep hit is mapped or explained.
- Criterion 10: Read the CHANGELOG.md Unreleased entry. Run: `git diff CHANGELOG.md`. Observes: The entry names the reader move, the fallback, the allow-list with D111 follow-ups, the site generator out of scope, and D111.

Planned files:

- crates/athena-cli/tests/records_branch_readers.rs (create)
- crates/athena-data/src/paths.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- xtask/src/walltime.rs (modify)
- xtask/src/main.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base 6387943d6a29 → candidate 36d095702955. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                      |  10 +
 crates/athena-cli/src/backtest.rs                 |   7 +-
 crates/athena-cli/tests/records_branch_readers.rs | 645 ++++++++++++++++++++++
 crates/athena-core/src/auditor.rs                 |   2 +-
 crates/athena-core/src/gh.rs                      | 115 +++-
 crates/athena-core/src/introspect.rs              |  23 +-
 crates/athena-core/src/manage.rs                  |  13 +-
 crates/athena-core/src/proposals.rs               |  15 +-
 crates/athena-core/src/reading.rs                 |  14 +-
 crates/athena-core/src/run.rs                     |   4 +-
 crates/athena-core/src/status.rs                  |   8 +-
 crates/athena-core/src/store.rs                   |  12 +-
 crates/athena-data/src/lib.rs                     |   3 +-
 crates/athena-data/src/paths.rs                   |  90 +++
 crates/athena-gate/src/records.rs                 |  17 +
 docs/decisions.md                                 |  50 ++
 xtask/src/main.rs                                 | 342 ++++++++++++
 xtask/src/walltime.rs                             | 136 ++++-
 18 files changed, 1470 insertions(+), 36 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1383 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test --locked pass; git diff shows xtask/src/site.rs unchanged.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
