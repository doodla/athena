# Job 20261103-222737-zzsr: verify "Don't count needs-accepting re-reads toward the drain job limit"

## Request (verbatim)

> try again on 20261103-193428-df2m

## Reading (accepted by the user)

Retry of job 20261103-193428-df2m: when a drain re-read ends as needs accepting, give its --job-limit slot back so the drain goes on to the next item. Audience: the operator running `athena drain`. This time the report must cite a command and its output for every behavior criterion, including the run against the old counter.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs. `run_job` checks `st.started >= self.limits.max_jobs` (~line 1433) and then increments `st.started` (~1439). The `_ if run.needs_accepting` arm (~1537) never gives the slot back. `needs_accepting: true` comes only from the re-read result (~3572). Other `st.started` uses are at ~1889, 2008, 2555 and 2896. The tests are in crates/athena-cli/tests/drain.rs; the help text is in crates/athena-cli/src/main.rs. No record or data file is added.
- Job 20261103-193428-df2m is retried. Its candidate (decrement of `st.started` in the needs-accepting arm, docs, test, CHANGELOG) was sound. It was rejected only because behavior criterion 2 cited no command that ran (D43). So the same four files are kept as deliverables and the retry must record the commands. The nit about driving `Source::Reread` is optional; the stub's `needs_accepting` run result is accepted.
- The old-counter demonstration is done with git-free means the verifier can see: the test's assertions (b must land, stop != JobLimit) and the recorded `cargo test` output. The producer should run the new test with the decrement temporarily removed, record the failing output, then restore the fix.
- No governance files change; athena.toml, defaults.toml and workflows are untouched.
- The cost of the re-read still counts against the money budget.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test with `max_jobs: 1` has item `a` end as needs accepting and item `b` runnable. It asserts `jobs[0].outcome == "needs accepting"`, `landed == 1` and `stop != Stop::JobLimit`, and it passes.
2. [behavior] Fails without the fix, with a command that ran: with the `st.started` decrement in the needs-accepting arm of `run_job` removed, `cargo test -p athena-cli --test drain` fails on the new test (b not run, Stop::JobLimit). The recorded check output shows that failing run and the passing run with the fix.
3. [content] `git diff` of crates/athena-cli/src/drain.rs shows the `_ if run.needs_accepting` arm of `run_job` returning the slot (`st.started` decremented under the state lock). The JobLimit check and the passed, held and failed outcomes are unchanged.
4. [behavior] Run `cargo test -p athena-cli --test drain`: `the_job_limit_stops_the_loop`, the urgent-queued-request test and the two existing needs-accepting tests still pass. `git diff` shows no assertion removed or weakened.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`; all are clean.
6. [content] The drain module docs, the `Limits::max_jobs` doc and the `--job-limit` help in crates/athena-cli/src/main.rs say a re-read that only needs accepting does not count toward the job limit. Grep finds that wording. CHANGELOG.md has an entry.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Avoiding the re-read itself, or changing which config changes trigger a re-read.
- Changing the money budget accounting; the re-read's cost still counts against the budget.
- Changing how failed, held or budget-short outcomes count toward the job limit.
- Auto-accepting readings whose scope differs.
- Changes to athena.toml, defaults.toml, workflows or any other governance file.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New regression test, plus cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name the test assertions and the commands to run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the job counter in run_job, not in the one stop reason.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in four files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criterion 2 requires recorded runs with and without the fix.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, help and CHANGELOG are updated for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A new test in crates/athena-cli/tests/drain.rs builds `Stub { items: vec!["a","b"], needs_accepting: vec!["a"], outcomes: passing(), ratings: RefCell::new(vec![3,3]), ..Stub::default() }` and calls `drain::drain(&stub, Limits { max_jobs: 1, ..limits(9.0) }, &Quiet)`. It asserts `jobs[0].outcome == "needs accepting"`, `landed == 1`, `stop != Stop::JobLimit`, that the log has `run b` and `land b`, and that `a` is still in `report.queued`. Run: `cargo test -p athena-cli --test drain a_re_read_that_needs_accepting_does_not_use_a_job_limit_slot`. Observes: The test passes: `a` ends as needs accepting without using the one slot, `b` runs and lands, and the stop is not JobLimit.
  Fails without the behavior: Without the decrement the test fails on `stop != Stop::JobLimit` and `landed == 1`: `b` is never run because `a`'s slot is never returned.
  Test seam: Entry point `drain::drain`, the public function the existing `the_job_limit_stops_the_loop` (tests/drain.rs:1422) calls; it reaches `run_job` (drain.rs:1433-1547). Fixture: the existing `Stub` with its `needs_accepting` field (tests/drain.rs:89, 223), passed as the `Steps` argument; `limits()` and `Quiet` exist. The stub only reports `Run.needs_accepting`; the slot accounting under test is the production `run_job`. Exists today; plan step 3 adds only the test function.
- Criterion 2: The producer comments out the `st.started` decrement in the needs-accepting arm, runs the command below and then `cargo test -p athena-cli --test drain` as a whole, and records each command with its failing output. It restores the decrement, reruns both, and records the passing output. The report quotes all four runs verbatim, command first. The verifier reruns the passing command and uses `git diff -- crates/athena-cli/src/drain.rs` to confirm the decrement is in the final tree. Run: `cargo test -p athena-cli --test drain a_re_read_that_needs_accepting_does_not_use_a_job_limit_slot`. Observes: Old-counter run: 1 failed, naming the new test, with `stop` equal to JobLimit and `b` not run; the other tests in the binary pass. Fixed run: the new test ok, 0 failed.
  Fails without the behavior: The recorded failing run is the evidence: with the decrement removed, the new test fails with Stop::JobLimit and `b` not run. If the report quotes no command and output for that run, the criterion is not met.
  Test seam: Same as criterion 1: `drain::drain` with the existing `Stub` (exists). The only temporary change is deleting one production line in `run_job` (drain.rs, `_ if run.needs_accepting` arm), which is the logic under test, not a test double. No seam is added.
- Criterion 3: Read the drain.rs diff. The `_ if run.needs_accepting` arm of `run_job` must lock the state and decrement `st.started`. The `if st.started >= self.limits.max_jobs` check (1433), the increment (1439), the `Some(OutcomeState::Landed)` arm, the `run.held` arm and the `other` arm must show no changed lines. Run: `git diff -- crates/athena-cli/src/drain.rs`. Observes: The only code hunk is the decrement in that arm (plus doc and comment lines). No hunk touches the JobLimit check, the increment or the other arms.
- Criterion 4: Run the whole drain test binary and read the results for `the_job_limit_stops_the_loop` (tests/drain.rs:1422), `a_run_that_needs_accepting_leaves_the_item_queued_without_a_failure` (1305), the passed-over test with the "Job limit" case (~5044-5059), the `Stop::JobLimit` tests at ~1585, 4698 and 4732, and the urgent-queued-request test (found by grepping `urgent`). Then read the tests/drain.rs diff: it must add lines only, with no `-` line on an assertion. Run: `cargo test -p athena-cli --test drain`. Observes: Every test in the binary passes. `git diff -- crates/athena-cli/tests/drain.rs` shows no removed or weakened assertions.
  Fails without the behavior: A counter change that broke the JobLimit stop or the needs-accepting handling would fail `the_job_limit_stops_the_loop`, `a_run_that_needs_accepting_leaves_the_item_queued_without_a_failure` or the passed-over test. For example, a decrement applied in the passed arm would let `the_job_limit_stops_the_loop` run past its limit.
  Test seam: Entry point `drain::drain` with the existing `Stub` and the other local `Steps` implementations in tests/drain.rs. Exists today; no step adds a seam.
- Criterion 5: After the fix is restored, run each of the three commands separately from the workspace root, in the same form as the checks Athena runs (fmt `--all --check`, clippy `--all-targets --locked -- -D warnings`, test `--locked`), and quote each command with its result. They are run as separate whole commands, not chained. Run: `cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --workspace --locked`. Observes: fmt prints no diff and exits 0. clippy reports no warnings and exits 0. The workspace tests report 0 failed in every test binary.
  Fails without the behavior: fmt fails if the new test or the decrement block is not rustfmt-formatted. clippy `-D warnings` fails if the change adds a warning (an unused `mut`, say). `cargo test --workspace` fails if the change breaks any existing test in any crate, such as the drain tests or a help-text or docs test. A fix that was only described and never compiled would also fail here.
  Test seam: Entry points are the cargo toolchain commands over the whole workspace, the same ones Athena runs as its fmt, clippy and test checks. No fake or fixture: they run the real source and the real tests, including the new test from step 3. Exists today (the commands are in the verifier's allowed list); no plan step adds a seam.
- Criterion 6: Read the diff. The drain.rs module docs (`A configuration change` bullet ~176-181 and the job-limit mentions at ~23 and ~49), the `Limits::max_jobs` doc (~253), and the main.rs `--job-limit` help (the `job limit` stop-reason line ~256 and the arg doc ~288) must each say that a re-read that only needs accepting does not count toward the job limit. CHANGELOG.md gets one entry under `## [Unreleased]` / `### Fixed`. The wording search is done on the added lines of the diff. Run: `git diff -- crates/athena-cli/src/drain.rs crates/athena-cli/src/main.rs CHANGELOG.md`. Observes: The added lines carry the wording in the module docs, the `Limits::max_jobs` doc and both main.rs help texts. The CHANGELOG diff shows one new entry in Unreleased.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 53b68fbf0ddf → candidate b3e381696a71. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  5 +++++
 crates/athena-cli/src/drain.rs   | 15 ++++++++++++---
 crates/athena-cli/src/main.rs    |  4 ++--
 crates/athena-cli/tests/drain.rs | 34 ++++++++++++++++++++++++++++++++++
 4 files changed, 53 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41735308199 for b3e381696a71f3b6020804a992a628242318b989)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41735308199 for b3e381696a71f3b6020804a992a628242318b989)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41735308199 for b3e381696a71f3b6020804a992a628242318b989)
- ci-result: `not a check` passed (Check results for b3e381696a71f3b6020804a992a628242318b989 come from the CI run https://github.com/doodla/athena/actions/runs/41735308199 (570s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`; all are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
