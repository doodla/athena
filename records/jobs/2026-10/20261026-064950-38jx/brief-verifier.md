# Job 20261026-064950-38jx: verify "Add code-health check, status table, and 10-landing design review"

## Request (verbatim)

> try again on 20261026-042558-5p65

## Reading (accepted by the user)

Retry of 20261026-042558-5p65. Add code_health.rs measuring per-file production lines, longest function, 7-day churn, failure citations and test run time; show it in athena status with thresholds; queue one structural-pass request per breaching file; run after each landing; queue a self-contained design-review request every 10 landings. Audience is Sri and the verifier.

Assumptions:
- No design-review role exists in the repo; the queued request carries its whole content, as the user said.
- Dependence on request 20261017-205558-dn2e is not assumed; this job defines the review text.
- Dedup and cadence are derived from records and open requests, not a counter, so running both hooks for one job does not double count.
- With no earlier design-review request, the cadence baseline is the current landing count, so the first review does not fire at once on existing history.
- Function length is measured by parsing source (syn if added, or a line-based scan); the choice is recorded in docs/decisions.md.
- No athena.toml field is added; thresholds are constants.
- The check hooks into both the finished-job path (steward note_finished) and Athena::land_pr.

## Acceptance criteria

1. [behavior] Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in the last 7 days, and failure/rejection citations.
2. [behavior] Run cargo test: tests show a breach of each threshold (file over 2000 lines, function over 200 lines, over 1500 lines with 5 or more landings in a week) is reported, and a file under the thresholds is not flagged.
3. [behavior] Run cargo test: on a breach exactly one structural-pass request is queued for that file, citing the measured numbers; running again, or with an open structural-pass request for that file, queues none.
4. [behavior] Run cargo test: the design-review request is queued at high priority at the 10th landed job since the last review and not at the 9th.
5. [behavior] Run cargo test: the queued design-review text names what to review (the code-health table, files most changed and most broken since the last review, open structural-pass requests, recent rejections and planning stops) and what to produce (a short design note with proposed changes, as a design-only job).
6. [behavior] Run cargo test: with no earlier review request, the first design review does not fire at once on existing history.
7. [behavior] Run cargo test: a test of the Athena::land_pr path shows the code-health check is invoked and would fail without the hook; the finished-job hook is likewise shown.
8. [behavior] Run cargo test: the rendered athena status output contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, whether over, and test suite run time.
9. [behavior] cargo clippy with -D warnings and cargo fmt --check are clean and cargo test passes.
10. [content] CHANGELOG.md has an entry, and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length, and the 10-landing cadence.

## Expected deliverables

- crates/athena-core/src/code_health.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Adding athena.toml fields
- Changing steward::tick or steward-count behavior
- Building a general design-review role or prompt beyond the queued request text
- Refactoring files flagged by the check

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each behavior; clippy and tests pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and assert observed outcomes.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Thresholds and cadence enforced in code, not prose.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Function-length measurement weighs syn against a manual scan and records the call.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Breaches queue work automatically, so no one has to point them out.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A check with no files measured is not treated as healthy.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixtures and queued text contain no personal data.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test with tempdir fixture .rs (known fn lengths plus a #[cfg(test)] mod) and in-memory JobRecords (landings inside/outside 7 days, rejected jobs, a finding naming the path). Run: `cargo test --locked -p athena-core code_health::tests::measures`. Observes: Exact production lines excluding test module, exact longest fn, 7-day landings (old excluded), citation count; at least 1 test run.
  Fails without the behavior: Lines include the test module, longest fn wrong, landings and citations zero.
- Criterion 2: Fixtures: 2001-line file; 201-line fn; 1501 lines with 5 landings and with 4; small file. Run: `cargo test --locked -p athena-core code_health::tests::thresholds`. Observes: Each breach reported with its reason; small file and 4-landing case not flagged.
  Fails without the behavior: A missing threshold leaves its breach unreported, or an under-threshold file is flagged.
- Criterion 3: Scratch repo with a breaching file; run_code_health twice, then with an open request, then with a closed one. Run: `cargo test --locked -p athena-core code_health::tests::structural_pass`. Observes: Exactly one request naming file and numbers; repeat and open-request runs queue none; closed does not block.
  Fails without the behavior: No dedup gives duplicates; no queueing gives none.
- Criterion 4: Records with a prior review request then 9 and 10 landed jobs; run check each time. Run: `cargo test --locked -p athena-core code_health::tests::design_review_cadence`. Observes: None after 9; exactly one at Priority::High after 10; none added while open.
  Fails without the behavior: Wrong cadence fires at the wrong count or lower priority.
- Criterion 5: Read the queued review text in a test. Run: `cargo test --locked -p athena-core code_health::tests::design_review_text`. Observes: Text names code-health table, most changed and most broken files since last review, open structural-pass requests, recent rejections and planning stops, and a short design note with proposed changes as a design-only job.
  Fails without the behavior: Text lacks what to review or produce.
- Criterion 6: Records with 25 landings and no review request; check, then 9 more landings, then the 10th. Run: `cargo test --locked -p athena-core code_health::tests::first_review`. Observes: First check queues nothing and writes the baseline; review fires only 10 landings after.
  Fails without the behavior: Without the baseline, existing history queues a review at once.
- Criterion 7: Two tests in a code_health::hooks module: (a) drives the real Athena::land_pr Landed branch using a github.com-style origin URL and a fake Runner (following PagesFake in crates/athena-cli/tests/external_calls.rs or the gh.rs FakeRunner), with a breaching Rust file in the tree; (b) drives Athena::note_finished likewise. Each asserts the 'code health:' ui line and a structural-pass request. Producer confirms each fails with its hook call removed. The run must report exactly 2 passed for the filter; 0 tests run counts as failure. Run: `cargo test --locked -p athena-core code_health::hooks`. Observes: 2 tests passed: hooks::land_pr_runs_code_health and hooks::note_finished_runs_code_health; ui line and request present after each path.
  Fails without the behavior: Removing the hook call from land_pr or note_finished removes the ui line and request, failing that test.
- Criterion 8: display.rs test builds StatusView with CodeHealth (two files, one over, test run time) and renders display::status; core test that Athena::status fills the field. Run: `cargo test --locked status_renders_the_code_health_table`. Observes: Output has each file's lines, longest fn, weekly landings, citations, thresholds, over marker, test run time; zero files says none measured. Expect at least 1 test run.
  Fails without the behavior: Without the field and renderer the table text is absent.
- Criterion 9: Run the three checks Athena runs. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0 with the new tests listed passing.
  Fails without the behavior: Unformatted code, any warning, or any failing test makes the command exit nonzero.
- Criterion 10: Read the diff of the two files. Run: `git diff CHANGELOG.md docs/decisions.md`. Observes: CHANGELOG Unreleased entry; decisions.md entry with the three thresholds, syn-vs-scan call, 10-landing cadence, baseline rule and the resync-skip note.

Planned files:

- crates/athena-core/src/code_health.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base 775017d9552f → candidate 59ce6d362770. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |    8 +
 Cargo.lock                                   |    2 +
 crates/athena-cli/src/display.rs             |  111 ++
 crates/athena-cli/tests/code_health_hooks.rs |  138 +++
 crates/athena-core/Cargo.toml                |    3 +
 crates/athena-core/src/code_health.rs        | 1475 ++++++++++++++++++++++++++
 crates/athena-core/src/gh.rs                 |    5 +
 crates/athena-core/src/lib.rs                |    4 +
 crates/athena-core/src/status.rs             |    4 +
 crates/athena-core/src/steward.rs            |    2 +
 docs/decisions.md                            |   44 +
 11 files changed, 1796 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` failed (160 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo clippy with -D warnings and cargo fmt --check are clean and cargo test passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
