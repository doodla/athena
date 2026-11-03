# Job 20261103-181027-bx8y: verify "Don't stop a batch on repeated failure of an item already paused"

## Request (verbatim)

> When a batch's repeated-failure stop is about an item that the scope guard has already paused, do not stop the batch: record it and move on to the next queued item. Keep the existing stop for every other repeated failure. This is the smallest version of the closed request 20261103-162416-m9th (whose plan grew too large); do not reclassify failures by cause here. Batches stopped this way twice today with money and job slots left. Acceptance: a test where an item fails twice, the scope guard pauses it, and the batch runs the next item; the existing repeated-failure test for an unpaused item still stops the batch.

## Reading (accepted by the user)

In the drain, when an item reaches its second failure (ATTEMPTS) and the scope guard has paused its line, end it as a passed-over paused item and go on to the next queued item, instead of Stop::RepeatedFailure. Unpaused items keep the existing stop. Job 20261103-162416-m9th (cause classification, key masking) is dropped; this is its smallest version. Audience: operators running batches.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs. Drain::attempts (about line 1725) counts failures and returns ItemEnd::Failed at ATTEMPTS (1811-1814). It already checks Steps::paused before each read (1745-1752) and returns ItemEnd::Paused. Drain::finish (2348) turns ItemEnd::Failed into set_stop(Stop::RepeatedFailure) unconditionally (2379). The gap: an item's second failure can itself pause its line, and no check follows the last failure. The fix is one re-check of Steps::paused before returning Failed (or in finish); no failure classification. Tests: crates/athena-cli/tests/drain.rs, with the existing paused-line test at line 644, repeated-failure tests at 1047 and 1073, and the Stub steps with a `paused` field. No record or data file is added.
- Job 20261103-162416-m9th: its four findings (Err classed as shared, invented stub text, missing id-wrapping unit test, round-3 planning stop) concern the cause classification and failure_key masking. The request tells us not to reclassify by cause, so they no longer apply and that plan is dropped. It left no candidate, so there are no changed files to keep or drop.
- A paused item is recorded through the existing ItemEnd::Paused path (a 'passed over' line), and not counted as a failure.
- The test Stub needs a paused list that changes after the item's runs fail; this may need a small Stub extension in tests/drain.rs.
- CHANGELOG.md gets a one-line entry if the repo convention requires it.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs: item a fails twice, the scope guard then reports its line paused, and drain::drain runs the next queued item. It asserts the next item is read, run and landed, report.stop is not Stop::RepeatedFailure, and report.failed_item is None. Check with `cargo test -p athena-cli --test drain`.
2. [behavior] The existing repeated-failure tests for an unpaused item (the_same_item_failing_twice_is_a_repeated_failure_naming_the_item and the not-landed variant) still pass unchanged: Stop::RepeatedFailure naming the item. Check with `cargo test -p athena-cli --test drain`.
3. [behavior] `cargo test` passes, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
4. [content] The diff to crates/athena-cli/src/drain.rs is small and has no failure-cause classification or key masking. The module docs and the Stop::RepeatedFailure description say that a failure of an item whose line is paused does not stop the batch. Verifiable with `git diff`.
5. [content] The paused item's second failure is recorded in the run output (a passed-over line with the pause reason) and not silently dropped. The new test asserts that line.
6. [content] No change to athena.toml, defaults.toml, workflows, the scope guard itself, ATTEMPTS, or the reader's skip-after-two-unread rule.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Reclassifying failures by cause, shared vs per-item, or failure_key masking (job 20261103-162416-m9th's plan)
- Changing the scope guard or how it pauses lines
- Changing ATTEMPTS or the reader's skip-after-two-unread rule
- Changes to athena.toml, defaults.toml, or workflows
- Changing environment stops such as rebuild failure, budget, job limit, or reinstall
- Changes to docs/ beyond doc comments in drain.rs, and to crates/athena-cli/tests/job_plan.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus existing tests; cargo test and clippy pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the drain, with no cause classification.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the drain through Stub and check the stop reason and the next item's run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The unconditional RepeatedFailure stop in finish/attempts is fixed, not one instance.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test for the drain tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two new tests in tests/drain.rs, both driving drain::drain with the Stub and plan entries. (1) `drain_passes_over_an_item_whose_second_failure_pauses_its_line_and_runs_the_next_item`: items ["a","b"], plan [planned("a",Ladder::Implementation,&["src/a.rs"],&[]), planned("b",Ladder::Implementation,&["src/b.rs"],&[])], outcomes [Rejected, Rejected, Landed], ratings [1], pause_on_retry Some(("a", pause_text("a"))), limits(9.0), Stamped Ui. It asserts stub.log() == ["read a","run a","reread a","run a-retry","read b","run b","land b","rate"], report.stop == Stop::QueueEmpty (so not RepeatedFailure), report.failed_item == None, report.landed == 1. (2) `drain_passes_over_an_item_whose_second_failure_pauses_its_line_and_holds_back_what_depends_on_it`: items ["a","b","c"], plan a, b, and c with depends_on ["a"], same outcomes and pause_on_retry. It asserts the log is the same as in (1) (c is never read or run), c has exactly one passed-over line containing `depends on a, which is held back: its line is paused`, b lands, stop is not RepeatedFailure, and failed_item is None. Run: `cargo test -p athena-cli --test drain drain_passes_over_an_item_whose_second_failure_pauses_its_line`. Observes: Both tests pass: after a's second failure, b is read, run and landed, the batch ends QueueEmpty with no failed item, and in the dependency test c is held back by the per-wave re-plan and never read or run.
  Fails without the behavior: Without the re-check before `return ItemEnd::Failed`, attempts returns Failed for a, finish sets Stop::RepeatedFailure naming a, the log ends at `run a-retry`, and b is neither read nor run, so the log, stop, failed_item and landed assertions fail in both tests.
  Test seam: Production entry point drain::drain (drain.rs 2663), which reaches Drain::work, attempts and finish (serial) and drain_concurrent (2674) through the same drain_with. The seam is the Steps trait with the existing Stub in tests/drain.rs injected as `&stub`; Stub::paused (156-165) already reads `paused_now`. The new `pause_on_retry` field and its few lines in Stub::run are added by plan step 5. The Stub only records state; no drain logic is in it. The planned() and pause_text() helpers exist.
- Criterion 2: Run the existing, unmodified repeated-failure tests: the_same_item_failing_twice_is_a_repeated_failure_naming_the_item (tests/drain.rs 1047) and a_job_that_does_not_land_twice_still_ends_as_repeated_failure_when_the_config_is_readable (1073, not-landed variant), plus the other tests with `repeated_failure` in the name. Their Stub has no paused, pause_on_run or pause_on_retry entry, so the new check finds nothing. Run: `cargo test -p athena-cli --test drain repeated_failure`. Observes: The tests pass unchanged: Stop::RepeatedFailure naming a, report.jobs.len() == 2, and the text `repeated failure: item a failed twice`.
  Fails without the behavior: If the re-check passed over every failure, not only a paused one, these tests would fail on the stop reason and failed_item.
  Test seam: drain::drain with the existing Stub (exists); the tests are not edited.
- Criterion 3: Run the three checks Athena runs on the result. Run: `cargo test --locked`. Observes: All tests pass. Also run `cargo clippy --all-targets --locked -- -D warnings` (clean) and `cargo fmt --all --check` (no diff).
  Fails without the behavior: A compile error, a lint (for example an unused Stub field), or a changed existing test would show here.
  Test seam: The whole workspace test suite; nothing injected.
- Criterion 4: Read the drain.rs diff: the helper, one re-check before `return ItemEnd::Failed`, and the doc and meaning() text. Look for cause classification or key masking (none expected). Run: `git diff -- crates/athena-cli/src/drain.rs`. Observes: A diff of a few dozen lines. The module-docs bullet on a paused line being passed over (lines 26-32) says the check is also made after the last failed attempt. The Stop::RepeatedFailure doc comment (942-944) and meaning() (about 1001) say that a failure of an item whose line is paused does not stop the batch. The Stop::ALL test (drain.rs about 4191) still sees a non-empty meaning.
- Criterion 5: The first new test also asserts the passed-over line with the Stamped Ui: `ui.lift_lines("a")` has exactly one entry; it starts with `plan       a passed over: line a is paused`, contains `To lift it: narrow the request`, and its runs-before count is 2 (printed after a's two runs, before b ran). It asserts display::drain(&report) does not repeat it (lift_mentions(&text, "a") == 0) and report.jobs holds two lines for item a. The second test asserts c's single passed-over line as in criterion 1. Run: `cargo test -p athena-cli --test drain drain_passes_over_an_item_whose_second_failure_pauses_its_line`. Observes: One passed-over line with the pause reason, printed once, after a's two runs and before b ran.
  Fails without the behavior: Without the fix no passed-over line is printed for a, because finish sets the stop instead, so the one-line assertion fails.
  Test seam: Same drain::drain entry point and the existing `Stamped` Ui (tests/drain.rs about 552) injected as `&ui`; it exists today.
- Criterion 6: List the changed files and confirm none is athena.toml, defaults.toml, a workflow or the scope guard, and that `const ATTEMPTS: u32 = 2;` (drain.rs about 243) and the reader's `unread >= 2` rule (1770) are untouched. Run: `git diff --stat`. Observes: Only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs and CHANGELOG.md are changed, and `git diff -- crates/athena-cli/src/drain.rs` shows no edit to ATTEMPTS or the unread branch.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ee4b71c6cadc → candidate caa3cedfc727. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   6 ++
 crates/athena-cli/src/drain.rs   |  37 +++++++----
 crates/athena-cli/tests/drain.rs | 135 +++++++++++++++++++++++++++++++++++++--
 3 files changed, 159 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41699494631 for caa3cedfc7277669d73691ee11697a6795f55040)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41699494631 for caa3cedfc7277669d73691ee11697a6795f55040)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41699494631 for caa3cedfc7277669d73691ee11697a6795f55040)
- ci-result: `not a check` passed (Check results for caa3cedfc7277669d73691ee11697a6795f55040 come from the CI run https://github.com/doodla/athena/actions/runs/41699494631 (554s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
