# Job 20261026-104801-0pwx: verify "Add code-health check, status table, and 10-landing design review"

## Request (verbatim)

> try again on 20261026-100656-h9wq

## Reading (accepted by the user)

Retry of job 20261026-100656-h9wq, keeping its accepted design. Add code_health.rs (syn-measured production lines and longest function, 7-day churn, failure citations, test run time), show it in athena status, queue one deduplicated structural-pass request per breach after each landing, and queue a self-contained high-priority design review every 10 landings, anchored by a typed baseline record. Tests must not hit the disk guard (set the minimum to 0, as fixed in 3947eb3).

Assumptions:
- Audience: Sri (user) and the verifier; goal is that code health is watched automatically.
- The prior design is kept as accepted; no design-review role exists, so the queued request carries its full content.
- Disk-guard fix 3947eb3 is present in the base; nested-run tests set the minimum to 0.
- Adding syn to athena-core is the buy-over-build choice for function length.

## Acceptance criteria

1. [behavior] Run cargo test: a code_health test with fixture source files and landing records asserts per-file production line count (excluding #[cfg(test)] modules), longest function length, landings in the last 7 days, and failure/rejection citations.
2. [behavior] Run cargo test: tests show each threshold breach (file over 2000 lines, function over 200 lines, over 1500 lines with 5 or more landings in a week) is reported, and a file under the thresholds is not flagged.
3. [behavior] Run cargo test: on a breach exactly one structural-pass request is queued for that file, citing the measured numbers; a second run, or an open structural-pass request for the file, queues none.
4. [behavior] Run cargo test: a baseline record (new AnyRecord variant in crates/athena-data/src/records.rs, with a schema round-trip test) is written, and a test reloads Records from git into a fresh state directory and shows the baseline survives. The first landing sets the baseline and fires no review.
5. [behavior] Run cargo test: the design-review request is queued through Athena::queue at high priority on the 10th landing after the baseline, and not on the 9th.
6. [behavior] Run cargo test: the queued design-review text names what to review (code-health table, files most changed and most broken since the last review, open structural-pass requests, recent rejections and planning stops) and what to produce (a short design note with proposed changes, as a design-only job).
7. [behavior] Run cargo test: tests show the check runs after a landing in both Athena::land_pr (after resync_onto_merge and the Pages watch) and note_finished in steward.rs, and each test fails without its hook. Tests set the disk-guard minimum to 0.
8. [behavior] Run cargo test: the rendered athena status output contains the code-health table with each file's lines, longest function, weekly landings, citations, thresholds, whether over, and test suite run time, with aligned headers.
9. [behavior] Run cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked: all three exit 0.
10. [content] CHANGELOG.md has an entry, and docs/decisions.md records the thresholds, the buy-versus-build call on measuring function length (syn), the baseline record, and the 10-landing cadence.

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
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-cli/tests/code_health_hooks.rs (create)

## Out of scope

- Building a design-review role or running the structural pass itself
- Changing athena.toml or defaults.toml
- Backfilling the cadence from existing landing history

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests; fmt, clippy, test run.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use syn and record the call in docs/decisions.md.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise queuing, hooks, and status output.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Status output verified by rendered-output test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test with fixture tree (cfg(test) mod, long and short fn), dated Athena-Job-trailer landings in a scratch git repo, fixture failed/rejected records. Run: `cargo test code_health`. Observes: Production lines exclude the test module; longest fn, 7-day landings and citations equal known values.
  Fails without the behavior: Test-module lines are counted, the longest function is wrong, or churn/citation counts differ.
- Criterion 2: Threshold tests: 2001-line file, 201-line fn, 1501 lines with 5 landings, plus under-limit and boundary files. Run: `cargo test code_health::tests::breach`. Observes: Each breach is reported with its reason; small and boundary files are not flagged.
  Fails without the behavior: Nothing is reported or the small file is flagged.
- Criterion 3: Scratch repo; run check, read Records, rerun, then with an open request present. Run: `cargo test code_health::tests::structural_pass`. Observes: Exactly one request citing the measured numbers; the rerun and open-request cases queue none.
  Fails without the behavior: A second request is queued, or none exists.
- Criterion 4: Round-trip test in records.rs; core test writes the baseline on the first landing, then Records::load from git into a fresh StateDir. Run: `cargo test baseline`. Observes: Record parses with the right path and schema; baseline survives reload; no review on the first landing.
  Fails without the behavior: Without the variant the baseline fails to parse/reload; without first-landing logic a review fires.
- Criterion 5: Set baseline, add 9 landings and check, then a 10th; min_free_gb 0. Run: `cargo test code_health::tests::design_review_cadence`. Observes: No review after 9; one high-priority request after the 10th, queued via Athena::queue; none again right after.
  Fails without the behavior: An off-by-one queues on the 9th or misses the 10th.
- Criterion 6: Read the queued review text from Records and assert phrases. Run: `cargo test code_health::tests::design_review_text`. Observes: Names the code-health table, files most changed and most broken, open structural-pass requests, recent rejections and planning stops, and a short design note with proposed changes as a design-only job.
  Fails without the behavior: A missing phrase fails the assertions.
- Criterion 7: Integration tests in code_health_hooks.rs plus a gh.rs unit test: land via land_pr (fake runner) and via note_finished, min_free_gb 0. For order, assert the Ui lines in sequence (resync, Pages watch, then the code-health line) and that the queued request's records commit follows the resync commit in git log of the records branch. A Rejected outcome test asserts nothing is queued. Run: `cargo test code_health_hooks`. Observes: Both paths queue the request for a breaching file, in the stated order; Rejected queues none.
  Fails without the behavior: Removing either hook leaves no queued request, failing that test; moving the hook earlier breaks the order assertion.
- Criterion 8: Render test builds a StatusView with rows and suite time and renders it in display.rs. Run: `cargo test status_renders_code_health`. Observes: Table has file, lines, longest fn, weekly landings, cited, limits, over columns and the suite time; header and cell columns align.
  Fails without the behavior: Without the table the header assertions fail.
- Criterion 9: Run the three checks on the finished change. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0, including every new test and the new AnyRecord variant handled in all matches.
  Fails without the behavior: A format slip, clippy warning, unhandled variant, stale Cargo.lock or failing test makes the chain exit non-zero.
- Criterion 10: Read the diff. Run: `git diff -- CHANGELOG.md docs/decisions.md`. Observes: Unreleased entry; decision with thresholds 2000/200/1500 with 5 per week, syn call and version, baseline record, landing definition, 10-landing cadence.

Planned files:

- crates/athena-core/src/code_health.rs (create)
- Cargo.toml (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/tests/code_health_hooks.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base 5e350839c857 → candidate 266c799e522d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |    7 +
 Cargo.lock                                   |    2 +
 Cargo.toml                                   |    2 +
 crates/athena-cli/src/display.rs             |   76 ++
 crates/athena-cli/tests/binary.rs            |    4 +-
 crates/athena-cli/tests/code_health_hooks.rs |  165 ++++
 crates/athena-cli/tests/cutover_fixture.rs   |   11 +-
 crates/athena-cli/tests/lifecycle.rs         |    3 +-
 crates/athena-core/Cargo.toml                |    2 +
 crates/athena-core/src/code_health.rs        | 1357 ++++++++++++++++++++++++++
 crates/athena-core/src/gh.rs                 |  192 +++-
 crates/athena-core/src/lib.rs                |    4 +
 crates/athena-core/src/status.rs             |    7 +
 crates/athena-core/src/steward.rs            |   58 ++
 crates/athena-core/src/store.rs              |    6 +
 crates/athena-data/src/records.rs            |   57 ++
 docs/decisions.md                            |   39 +
 docs/records.md                              |    1 +
 18 files changed, 1971 insertions(+), 22 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1314 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked: all three exit 0.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
