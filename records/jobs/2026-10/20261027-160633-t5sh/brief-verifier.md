# Job 20261027-160633-t5sh: verify "Treat a model usage limit as a hold, not a job failure"

## Request (verbatim)

> Priority: urgent
> Treat a model usage limit as the environment, not a job failure. On 2026-10-27 the verifier call for job 20261027-123701-77zh returned in 2 seconds with "You've hit your weekly limit · resets Oct 29, 6pm (UTC)" and no usage; Athena recorded an error invocation with measured tokens but no usage for the requested model, its own gate refused that record, the job ended with no outcome, and drain counted it as a failure twice and stopped. Fix: recognise a usage-limit reply from the driver (the limit text and its reset time), record the invocation as limited with zero cost (so the gate accepts it), keep the job and its candidate for retry, and have drain stop with a 'usage limit' stop reason naming the reset time instead of 'repeated failure'. athena status shows the limit and its reset time while it holds.
> Acceptance:
> - A test with a fake driver reply carrying the limit text shows the invocation record passes the gate, the job stays retryable with its candidate kept, and no failure is counted.
> - A drain test shows the stop reason 'usage limit' with the reset time.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-agent/src/claude.rs, crates/athena-core/src/run/mod.rs, crates/athena-cli/src/drain.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

Recognise the driver's usage-limit reply (limit text plus reset time), record the invocation as limited at zero cost so the gate accepts it, keep the job and its candidate retryable, and make drain stop with 'usage limit' naming the reset time. Job 20261027-123701-77zh, which hit the limit, is retried by this fix and is not re-done; its site-rating work is out of scope.

Assumptions:
- Code survey: read crates/athena-agent/src/claude.rs (the driver reply parse and the invocation record built from it), crates/athena-cli/src/drain.rs (Stop::RepeatedFailure maps to 'repeated failure' near line 504; the drain loop counts failures), crates/athena-core/src/run/mod.rs (run pipeline, the verifier call and the gate on invocation records) and crates/athena-core/src/status.rs. No new record or data file under crates/ is added; if the invocation record gains a 'limited' state, the existing records validation and round-trip test must still accept it.
- Job 20261027-123701-77zh (site rating fixes) never got a verdict and changed no files; its candidate is kept and retried, not dropped. Its own deliverables (xtask/src/site.rs, studies/, docs/design/explainer-site.md) are not part of this change.
- The audience is Athena maintainers running drain; they judge the change by the stop reason, status output and a retryable job.
- The limit text has the form "You've hit your weekly limit · resets Oct 29, 6pm (UTC)"; the parser tolerates other limit kinds (session, daily) that carry the same 'limit' and 'resets' wording.

## Acceptance criteria

1. [behavior] Run `cargo test`: a claude.rs test feeds a fake driver reply with the text "You've hit your weekly limit · resets Oct 29, 6pm (UTC)" and no usage, and asserts the reply is recognised as a usage limit with the reset time parsed.
2. [behavior] Run `cargo test`: a run-level test with a fake driver returning the limit reply asserts the invocation record is marked limited with zero cost, passes the gate, the job stays retryable with its candidate kept, and no failure is counted.
3. [behavior] Run `cargo test`: a drain test with a job hitting the limit asserts drain stops with stop reason 'usage limit' that names the reset time, not 'repeated failure', and the failure counter is unchanged.
4. [behavior] Run `cargo test`: a status.rs test asserts `athena status` output shows the usage limit and its reset time while the limit holds, and shows neither once the reset time has passed.
5. [content] A genuine driver error without limit text still records as an error and counts as a failure, shown by an existing or new test, so ordinary failure handling is unchanged.
6. [content] CHANGELOG.md has an entry describing the usage-limit handling, with the 'usage limit' stop reason and the status display.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- The site-rating changes of job 20261027-123701-77zh (xtask/src/site.rs, studies/, docs/design/explainer-site.md)
- Automatic waiting or sleeping until the reset time
- Changing the gate's rules for other records
- Records under records/, athena.toml, defaults.toml and workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-driver, drain and status tests plus fmt, clippy -D warnings and cargo test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Handles the limit reply at the driver and record level rather than patching drain's failure count alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise the limit reply, gate, retry and stop reason.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The gate accepts the limited record and drain enforces the stop through code.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A limited job is never counted as success; it stays retryable.
- thrash-stop: If the same item fails twice in a loop, the loop stops and reports thrashing. Prose-only: no loop code detects it yet. Here: A limit hit stops drain at once with its own reason, rather than as a repeated failure.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry and messages are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit and driver-process tests in claude.rs. (a) Feed the result line (is_error true, text "You've hit your weekly limit · resets Oct 29, 6pm (UTC)", zero usage, empty modelUsage) to `StreamState::on_line`, then call the production function `output_from_result` and assert the output. (b) Add a `MODE=limit` branch to the existing `STAND_IN` shell script, placed before its assistant echo, and run it through `ClaudeCodeDriver::invoke` with the existing `stand_in`, `driver` and `spec` helpers. (c) A third case parses 'session limit · resets 3pm (UTC)'. The stand-in and helpers exist today (claude.rs lines 585 to 646); step 2 adds the branch and tests. Run: `cargo test -p athena-agent usage_limit`. Observes: Outcome is Limited; the detail contains 'resets at 2026-10-29T18:00:00Z' and the reply text; `cost_usd` is Some(0.0); usage is zero; `usage_measured` is false; `models_used` is empty. This holds through both the pure function and the real driver process path, and a session-kind reply is also recognised.
  Fails without the behavior: Without the recogniser, `output_from_result` returns outcome Error with `usage_measured` true, so the outcome, detail and measured assertions fail.
- Criterion 2: Run test in crates/athena-core/src/run/mod.rs, patterned on `low_disk_before_the_verifier_resumes_without_planner_or_producer`. Build the Athena with the existing `athena_with(.., PASSES, .., counting(..))`, then replace its `driver` field (an `Arc<dyn AgentDriver>` the harness already builds) with a test-local `LimitedVerifier`. That wrapper holds an inner `FakeAgentDriver` made with the same `counting` script and registry and delegates every role but the verifier. For the verifier it passes the limit line through `StreamState::on_line` and the real `output_from_result` and returns that output, so no recognition logic is in the double. Entry points: `Athena::go`, then `Athena::retry` with the plain driver restored. The `LimitedVerifier` wrapper is added in step 5; the `Athena.driver` field and the harness exist today. Confirm the `AgentDriver` trait's method set in step 1. Run: `cargo test -p athena-core usage_limit`. Observes: (1) `go` returns Failed with a reason starting 'environment: usage limit', `is_environment_reason` is true, and `usage_limit_of(&reason)` returns the reset time (and None for a low-disk reason). (2) The stored invocation record has outcome Limited and cost 0, and `athena_gate::reconcile::check_invocation(path, &record)` returns no violations, called on the stored record. (3) The resume mark exists and the candidate fingerprint is recorded. (4) With the plain driver, `retry` resumes, the job passes, and the planner and producer counters stay at (0, 1, 0). (5) A control run with an Error verifier turn closes without the usage-limit reason. The drain half of 'no failure counted' is criterion 3.
  Fails without the behavior: Without the Limited handling, the verifier branch sets `verifier_failure`, the job closes Rejected, the resume mark is removed, and the state, reason and resume assertions fail. If the output were measured, `check_invocation` would return the 'measured tokens, but no usage for the requested model' violation.
- Criterion 3: Drain test in crates/athena-cli/src/drain.rs tests. Add a `LimitedStub` implementing `Steps`, modelled on the existing `Stub` (line 2074), with two queued items. Its `run` logs the call and returns `Run { outcome: Some(OutcomeState::Failed), reason: <the reason string the run pipeline writes, built with the same builder run/mod.rs exports>, .. }`. Call `drain_with` (line 1303) with the existing `limits()` and `Quiet` ui. The new stub, `RunStep::UsageLimit`, `ItemEnd::UsageLimit` and `Stop::UsageLimit` are added in step 7. Run: `cargo test -p athena-cli usage_limit`. Observes: `report.stop == Stop::UsageLimit` and its label is 'usage limit'; `report.usage_limit` contains the reset time; `report.failed_item` is None; `run` was called once (so `failures` was never incremented and no retry was tried); the second item never started; the job line outcome reads 'usage limit'; `display::drain(&report)` contains 'usage limit' and the reset time and does not contain 'repeated failure'. The existing `Stop::ALL` loops (drain.rs line 2348 and crates/athena-cli/tests/drain.rs line 1279) now cover the new variant.
  Fails without the behavior: Without the new branch, the failed Run falls into the 'other' arm of `run_job`, is retried, and the stop becomes RepeatedFailure; the stop and call-count assertions fail.
- Criterion 4: Two tests. In status.rs tests, build `Records` with the existing `invocation(job, seq, role, usd)` helper (line 1615), set outcome Limited and the detail 'usage limit (resets at 2026-10-29T18:00:00Z): …', and call the production function `usage_limit_hold(&records, now)` with a `now` before the reset, one after it, and one before it with a later Success invocation present. In display.rs tests (added there if no test module exists), render `display::status` on a `StatusView` with `usage_limit` set and one with it None. `usage_limit_hold`, the field and the render line are added in steps 6 and 8. Run: `cargo test -p athena-core usage_limit_hold && cargo test -p athena-cli status_usage_limit`. Observes: The hold carries the limit text and reset time while `now` is before the reset. It is None once `now` is past the reset, and None when a later invocation succeeded. The rendered status contains 'usage limit' and the reset time only in the holding case, and neither when None.
  Fails without the behavior: Without the hold function and field, the tests do not compile; with a hold that ignored `now`, the after-reset case would still show the limit and fail.
- Criterion 5: claude.rs tests: a result with is_error true, subtype 'error_during_execution' and text 'API Error: 500', and one with is_error true and text 'rate limit exceeded' with no 'resets' wording, each give outcome Error with the detail unchanged from today. A successful result with usage that quotes 'hit your limit … resets' gives Success. The run/mod.rs control (Error verifier turn) closes without the usage-limit reason. Existing drain tests with a plain Failed Run (no usage-limit reason) still stop as RepeatedFailure after two attempts. Run: `cargo test -p athena-agent -p athena-core -p athena-cli`. Observes: Ordinary errors keep the Error outcome and measured tokens, closing reasons and drain failure counting are unchanged, and a drain with a plain failed `Run` still stops as 'repeated failure'.
  Fails without the behavior: If the matcher were too loose (any error, or a success quoting the words), these cases would show Limited or a changed outcome, and the failure-count tests would fail.
- Criterion 6: Add an Unreleased entry to CHANGELOG.md naming the cause (a weekly limit reply recorded as an error and counted as a failure twice), the Limited invocation outcome, the retryable job, the 'usage limit' stop reason with the reset time, and the `athena status` line. Run: `grep -n 'usage limit' CHANGELOG.md`. Observes: The entry sits under Unreleased and mentions the stop reason and the status display.
  Fails without the behavior: With no entry, grep finds no 'usage limit' line in the Unreleased section.
- Criterion 7: Run the three checks Athena runs on the final tree, which includes the new tests from criteria 1 to 5, the extended records round-trip test, the budgets test (`system_facts_budgets_match_computed`) and the help/`Stop::ALL` tests. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: fmt fails on unformatted new code; clippy fails on an unused import or dead variant; the tests fail if `Limited` is missing from the records round-trip, if `Stop::UsageLimit` is missing from `Stop::ALL` or the drain help list in main.rs, or if any of the new behavior tests of criteria 1 to 5 fails.

Planned files:

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8c98795e50ad → candidate e94425612346. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  19 +++
 crates/athena-agent/src/claude.rs | 305 ++++++++++++++++++++++++++++++++++++++
 crates/athena-cli/src/display.rs  |  33 +++++
 crates/athena-cli/src/drain.rs    | 180 +++++++++++++++++++++-
 crates/athena-cli/src/main.rs     |   2 +
 crates/athena-core/src/disk.rs    |  16 +-
 crates/athena-core/src/run/mod.rs | 284 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs  | 168 +++++++++++++++++++++
 crates/athena-data/src/records.rs |   9 ++
 9 files changed, 1012 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1630 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
