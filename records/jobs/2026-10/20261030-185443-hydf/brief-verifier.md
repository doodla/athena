# Job 20261030-185443-hydf: verify "Commit and print the reader's error; re-read an unread job once"

## Request (verbatim)

> try again on 20261030-061329-m30w

## Reading (accepted by the user)

Retry of job 20261030-061329-m30w (dispatched, never verified, no candidate or findings), so the earlier reading is rebuilt unchanged in intent. When the reader gives no usable reading, its error is committed with the failed-reading record and shown on the drain's line for the item. A Proposed job with no reading is read once more, then skipped with its reason, without counting twice toward the repeated-failure stop. The audience is the person running `athena drain`.

Assumptions:
- Job 20261030-061329-m30w: it had no candidate, no changed files and no findings, so nothing is kept from it and nothing needs fixing. The whole original request is redone, with the same deliverables and scope.
- Job 20261029-233021-e5sg's rejected landing-stage change is not retried or kept. status.rs, store.rs, athena-git/src/lib.rs and that job's CHANGELOG entry stay as on main.
- Code survey: crates/athena-core/src/reading.rs (`Proposal` at about line 61, the read path that builds the 'Read a request (the reader failed)' commit at about lines 858-877, and the retry path at about line 1046). crates/athena-core/src/run/mod.rs (`go` for a Proposed job: `None => bail!("job {id} has no reading yet (the reader failed)...")` at about line 331). crates/athena-cli/src/drain.rs: `run_job` builds the `JobLine` and returns `RunStep::Failed { read: false }` when the read has an error. `attempts` counts `failures` and returns `ItemEnd::Failed`, which sets `Stop::RepeatedFailure` at about line 1656. `Steps::read` for `ReadOf::Queued` calls `athena.go`, whose bail is what drain sees as the item's error.
- Code survey: the change adds no record or data file under crates/, so the records allow-list (`check_record_file`, `AnyRecord`, the round-trip test) is untouched. The error text goes into the existing failed-reading commit message body and the item's log line, not into a new record type. If it needs a field in an existing record, the round-trip test must still pass.
- The drain's re-read of a Proposed job with no reading is a second `Steps::read`/`athena correct`-style read inside the drain item's attempt. It is bounded to one re-read per item. When both reads fail, the item ends as skipped with its reason. It uses a skip-style end, not `ItemEnd::Failed`, so `Stop::RepeatedFailure` is not set and the next item runs.
- Run/mod.rs and run_pin_tests.rs stay as listed deliverables from the earlier scope (a possible new `go` error shape and pin tests), but are likely unchanged. Only the files that need to change are edited.
- The tests use the existing drain test harness in crates/athena-cli/tests/drain.rs with a fake reader that returns unparseable output, and reuse the harness's way of capturing the drain log.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test drives the drain entry point with a reader that returns unparseable output for the first item. It asserts that the reader's error text appears in the drain log line for that item and in the committed failed-reading record (a `git show` of the commit), that the reader ran exactly twice for the item (one re-read), and that the drain then ran the next item.
2. [behavior] In that test the skipped item is reported with its reason, and the drain's stop reason is not 'repeated failure'. A second test (or a second part of the first) shows that when the re-read succeeds the item runs. Run with `cargo test -p athena-cli --test drain`.
3. [behavior] A test in athena-core (run with `cargo test -p athena-core`) shows that a read whose reader output cannot be parsed commits a record or commit body that contains the `Proposal.reader_error` text.
4. [content] In crates/athena-core/src/reading.rs, the failed-reading commit is built using `reader_error`. `git diff` shows the error flowing into the commit body or record.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has a new entry saying the reader's error is committed and printed by drain, and that a Proposed job with no reading is read once more and then skipped with its reason.
7. [content] `git diff` touches only crates/athena-core/src/reading.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, optionally crates/athena-core/src/run/mod.rs and crates/athena-core/src/run_pin_tests.rs, and CHANGELOG.md. athena.toml, defaults.toml, .github/workflows and records/ are unchanged, as are status.rs, store.rs and athena-git/src/lib.rs.
8. [behavior] The existing repeated-failure behavior is unchanged for items that failed after being read: the existing drain tests for `Stop::RepeatedFailure` still pass under `cargo test -p athena-cli --test drain`.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Retrying or keeping job 20261029-233021-e5sg's rejected landing-stage change: crates/athena-core/src/status.rs, crates/athena-core/src/store.rs and crates/athena-git/src/lib.rs stay as on main, and its CHANGELOG entry is not touched
- Finding out why the reader produced no reading in the failed job (a reader or prompt fix)
- Changing athena.toml, defaults.toml, workflow files or records/
- Changing the repeated-failure rule for failures that are not unread jobs
- Repairing or re-reading existing job records by hand

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain and athena-core tests; fmt, clippy -D warnings and cargo test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the error text in the log and the committed record, the one re-read and the drain moving on, not just that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The error is carried and committed at its source (the failed-reading record), and the repeated-failure counting is fixed for unread jobs, not just logged.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in reading.rs and drain.rs with tests and a changelog entry.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog and commit text are written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the drain moving on with the reason visible, not the activity of editing.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the drain tests and the full suite.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in crates/athena-cli/tests/drain.rs. Entry point: `drain::drain(&steps, limits(50.0), &lines)` with `real_steps_with(&athena, &lander, &lines)`; `athena` from `fx.athena(script)` (exists today). Precondition: two queued requests, each with an acceptance bullet (like the text at tests/drain.rs ~2020) so the plan has no missing parts and the first read happens in `attempts`. Seam: wrap `rated_scenario().script()` in a closure that, for Role::Reader whose `user_prompt` contains the first request's text, counts the call and sets `turn.structured = None` (FakeTurn.structured is pub; the Script seam exists in `Fixture::athena`); every other call is the unchanged scenario. `lines` is `common::Lines` (exists). The logic under test (go, read_queued, drain) is real; only the model output is faked. Run: `cargo test -p athena-cli --test drain a_reader_that_gives_no_reading_is_read_once_more_then_skipped_with_its_error`. Observes: Reader calls for item one equal 2; a `Lines` entry starting `drain` contains 'the reader returned no structured reading'; `fx.git.run(["log","--format=%B","--grep","the reader failed"])` output contains the same text; the lander recorded the second item's job (the drain ran the next item).
  Fails without the behavior: Without the commit-body change the `git log` text has no error text; without the `go` re-read the reader runs once, not twice; without the Skipped end the item ends RepeatedFailure and the second item is never run, so the assertions on reader count, lander and the next item fail.
- Criterion 2: Same test as criterion 1 (second part) plus a second new test. Entry point and seam as in 1. Test A asserts on the returned `Report`: `report.stop != Stop::RepeatedFailure`, `report.jobs[0].outcome == "not read"` with `reason` containing the error text, a `report.notes` entry containing 'skipped' and the error text, and `display::drain(&report)` showing the reason. Test B: the script fails the reader only on its first call for the item, then behaves normally. Run: `cargo test -p athena-cli --test drain -- a_reader_that_gives_no_reading a_re_read_that_succeeds_runs_the_item`. Observes: Test A: no repeated-failure stop, the skipped item reported with its reason. Test B: reader called twice, `report.landed == 1`, stop is QueueEmpty (or not RepeatedFailure), lander has the item's job.
  Fails without the behavior: Without the Skipped end the stop is RepeatedFailure and `report.jobs[0].reason` has no skip note; without the go re-read the second test's item never lands because the second attempt bails.
- Criterion 3: New tests in crates/athena-core/src/run_pin_tests.rs beside `a_reader_that_does_not_complete_is_not_read_again` (~line 1763). Entry point: `athena.ask("study the demo", &Quiet)` built with the existing `repo`, `athena_with` and a `Tweak` that sets `turn.structured = None` for Role::Reader (the same seam that test uses today). Then `go` on that job with a working reader: assert it returns `GoResult::Reread` with a reading, with no invocation seq collision and the reader called exactly twice in total. Also a `correct` variant is covered by the same helper. Run: `cargo test -p athena-core a_reader_that_returns_no_reading_commits_its_error`. Observes: `proposal.reader_error` is Some; `git.run(["show","-s","--format=%B",&proposal.commit])` contains the `reader_error` text; for the go re-read, the commit subject starts 'Read a request again'.
  Fails without the behavior: Without the helper in `first_reading` the commit body lacks the error and the assertion fails.
- Criterion 4: Read the diff of reading.rs. The helper that builds the failed-reading body takes `error` (from `read_for`'s third value, which becomes `Proposal.reader_error`) and is used in `first_reading` and in `correct`. The behavior is proved by criterion 3's test. Run: `git diff -- crates/athena-core/src/reading.rs`. Observes: The commit body is built from the error text; `message(&subject, &body, ...)` in the failed-reading branch now receives a body that contains it.
- Criterion 5: Run the three checks Athena will run. Each is a whole simple cargo command; the verifier may run them one at a time. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: Not applicable: this is the repository's own gate (fmt, clippy -D warnings, full tests including the existing RepeatedFailure tests and any test touching `go` on a Proposed job).
- Criterion 6: Read the diff of CHANGELOG.md. Add one entry under `## [Unreleased]` in its `### Fixed` or `### Changed` section (the first `### Added` at line 10 belongs to Unreleased; choose the Fixed section found after it). Entry in plain words for a person running `athena drain`: the reader's error is committed with the failed-reading record and printed on the drain line; a Proposed job with no reading is read once more and then skipped with its reason, not counted as a repeated failure. Run: `git diff -- CHANGELOG.md`. Observes: The new entry names both changes.
- Criterion 7: List changed files after the producer run. Expected only: crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs (one arm of `go_job`), crates/athena-core/src/run_pin_tests.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, CHANGELOG.md. Run: `git status --short`. Observes: No other path appears; athena.toml, defaults.toml, .github/workflows, records/, status.rs, store.rs and athena-git/src/lib.rs are absent.
- Criterion 8: Run the whole existing drain test file with the new tests. Entry point is `drain::drain` over the existing `Stub`; those items fail after being read (Stub reads give a job with no error), so they never take the new Unread branch. Run: `cargo test -p athena-cli --test drain`. Observes: All existing RepeatedFailure assertions still pass: stop RepeatedFailure and the failed item named.
  Fails without the behavior: If `failures`, ATTEMPTS or the Failed path were changed, `the_same_item_failing_twice_is_a_repeated_failure_naming_the_item`, `a_landing_failure_is_a_failure_and_no_rating_is_taken_as_success` and `a_pass_that_landed_nothing_cannot_report_target_met_from_a_stale_rating` (existing tests at ~751, ~884, ~648) would fail.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 6ecf9f6f4fa3 → candidate 7a4fceaf3ed1. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   9 ++
 crates/athena-cli/src/drain.rs          |  49 ++++++++++-
 crates/athena-cli/tests/drain.rs        | 141 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs       |  71 +++++++++++++---
 crates/athena-core/src/run/mod.rs       |   7 +-
 crates/athena-core/src/run_pin_tests.rs |  94 +++++++++++++++++++++
 6 files changed, 352 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1746 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
