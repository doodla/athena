# Job 20261031-094756-8cjd: verify "Classify environment stops in one place so the scope guard ignores them"

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The scope guard treats all spend on a line as one number. It does not record why an attempt stopped,
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: scope-guard-counts-environment-stop-as-line-spend-the person running Athena-lifts-pause (20261028-065858-bztk)
> Cause: The scope guard treats all spend on a line as one number. It does not record why an attempt stopped, and it does not know how much work is left. A stop caused by the environment (low disk) looks the same as a stop caused by the work being t...
> Seen 4 time(s) in 4 jobs. The introspections that record it:
> Introspection 20261028-065847-kmx7 (job 20261028-055601-hg0n); prevention it proposed: Record a stop reason on every attempt. Have the scope guard exclude attempts that ended on an environment stop (low disk, container reclaim) from the spend and attempt counts, or count them separately. When an attempt has a kept candidate t...
> Introspection 20261028-103312-b53e (job 20261028-065911-m6m5); prevention it proposed: Make the scope guard record the stop reason for each attempt and the state of the kept candidate. Spend from an environment stop, such as low disk, must not count toward the line's progress-less spend. If a candidate is kept and has passed ...
> Introspection 20261028-124847-gf2n (job 20261028-103330-3d47); prevention it proposed: Compute the pause instead of storing it. On each guard evaluation (and whenever the guard's rule changes), recompute the line's spend from the per-attempt stop reasons under the current rule, and clear or keep the pause on the result. If a ...
> Introspection 20261029-195138-3pqb (job 20261029-111202-2bpc); prevention it proposed: Record a stop reason with each attempt's spend (environment, mechanical plan finding, real critic or verifier failure, producer failure). The guard should count only the reasons that signal runaway work. Store with each pause the rule versi...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the scope guard learns why an attempt stopped from a free-text closing-reason prefix, and two lists of "environment" reasons disagree (`scope_guard::ENVIRONMENT_PREFIX` vs `disk::is_environment_reason`). A producer stop on an expired login (401, job 20261028-103330-3d47) is closed as a work failure and counted as line spend, and a stored pause is never recomputed. Fix: one classifier of stop reasons that the guard and retry both use, with auth and usage-limit stops as environment, and the pause computed from the attempts on each evaluation.

Assumptions:
- Code survey: I read crates/athena-core/src/scope_guard.rs (`attempt_of`, `counted`, `t3_basis`, `line_pause`, `try_watch_line`, `list_lines`), crates/athena-core/src/disk.rs (`LOW_DISK`, `CHECK_TIMEOUT`, `is_environment_reason`, `resumable_stop`) and crates/athena-core/src/run/producer.rs (the 'the producer did not complete' failure). The entry point is `try_watch_line` after each attempt, which writes the pause file, and `line_pause` plus `list_lines`, which read it back. No new record or data file under crates/ is planned, so `check_record_file`, `AnyRecord` and the round-trip test are untouched. The property is measured directly by running `evaluate_before_attempt` and `line_pause` over attempts built from real closing reasons.
- Goal and audience: the person running Athena, whose lines get paused by stops that are not runaway work. Success is that an environment stop (low disk, check timeout, expired producer login, usage limit) never raises a signal, and that a pause that no longer holds under the current rule does not block a retry.
- Previous jobs 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47 were requests for a different feature (the reading replay). Their findings were environment stops (low disk, 401), not defects in that work. This request does not retry them and does not fix their findings. Their candidate files (CHANGELOG.md, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, reading.rs, store.rs, verify.rs, crates/athena-data/src/records.rs, docs/records.md) are dropped here and listed in out_of_scope; crates/athena-core/src/run/mod.rs is kept as a possible deliverable.
- Most of the line-spend exclusion already exists for 'environment:' prefixed reasons. The remaining gap is classification: a producer that stops on a 401 or an expired OAuth token is closed without that prefix. I assume a small typed `StopCause` (or one shared function) in disk.rs or scope_guard.rs is enough and no new record schema is needed.
- No governance file (athena.toml, defaults.toml, workflows) needs to change; no refusal rule on readings is added, so no replay percentage applies.

## Acceptance criteria

1. [content] The change description names the root cause: the guard infers the stop reason from free-text closing-reason prefixes and has two disagreeing environment lists, so an auth-expiry stop counted as line spend; the fix is in the shared classifier in crates/athena-core/src/disk.rs and scope_guard.rs, not in one job's data.
2. [content] `scope_guard::attempt_of` and `disk::is_environment_reason`/`resumable_stop` use one classifier (no second hard-coded list); Grep shows a single definition of which closing reasons are environment stops.
3. [behavior] Run `cargo test -p athena-core scope_guard`: a new test builds a line of several attempts that all closed with the producer-auth-expired reason (the 'the producer did not complete: ... 401 OAuth access token has expired' text) and asserts T1, T3 and T5 give Level::None; the same test fails when the classifier change is reverted. A second test covers each environment reason kind (low disk, check timeout, auth, usage limit), not one job's text.
4. [behavior] Run `cargo test -p athena-core scope_guard`: a test shows a stored pause whose signals no longer hold under the current attempts is not returned by `line_pause` (the pause is recomputed from attempts), while a pause for real repeated rejections still is.
5. [behavior] Run `cargo test -p athena-core disk`: a producer stop on an expired login is a resumable stop, so `athena retry` can resume it.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
7. [content] CHANGELOG.md is not required; no existing reading rule, athena.toml, defaults.toml or workflow changes (git diff shows none).

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Retrying or completing the reading-replay feature of jobs 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47; their candidate changes to crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/store.rs, crates/athena-core/src/verify.rs, crates/athena-data/src/records.rs and docs/records.md are dropped
- Automatic disk cleanup or disk reservation before the verifier
- Automatic re-authentication of the producer's login
- Changing the thresholds T1 to T5 or the reading refusal rules
- Editing or deleting existing pause files in the state directory

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The classifier is fixed where the stop reason is derived, for the whole class.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise the guard over attempts.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the classifier for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to stop classification and pause recompute.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that an environment stop does not pause a line, not that the code exists.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check of the diff: the doc comment on `StopCause` in disk.rs, the module doc of scope_guard.rs, and the CHANGELOG.md Unreleased entry each name the root cause. Run: `git diff -- crates/athena-core/src/disk.rs crates/athena-core/src/scope_guard.rs CHANGELOG.md`. Observes: The diff says the stop reason was inferred from free-text closing-reason prefixes held in two disagreeing lists, so an expired-login stop counted as line spend; the fix is the shared classifier in disk.rs used by scope_guard.rs, and the pause computed from attempts with the forecast in force when it was written.
- Criterion 2: Test plus Grep. `disk::tests::one_classifier_decides_environment_reasons` asserts `is_environment_reason(r) == StopCause::of(r).is_environment()` and that `resumable_stop` agrees, for every kind (low disk, check timeout, auth, usage limit, work). A Grep of `ENVIRONMENT_PREFIX` and `starts_with("environment` under crates/ finds one definition and its use only in disk.rs; `attempt_of` reaches the classifier through `verdict_of`. Run: `cargo test -p athena-core disk::tests::one_classifier_decides_environment_reasons`. Observes: The test passes; the Grep shows scope_guard.rs holds no list of environment reasons.
- Criterion 3: Unit tests in scope_guard.rs `mod tests`. Test A: four attempts of 40 minutes and $1.50. Map the expired-login reason through `verdict_of(JobState::Closed(OutcomeState::Failed), Some(reason))`, with the reason built three ways: the new form from `producer_stop_reason(driver_detail(json))`; the stored form 'the producer did not complete: ' + `driver_detail(json)` (the real driver path of finding 2, via StreamState::on_line and output_from_result); and a literal legacy string as a control for the read path. Build `AttemptRecord`s with the existing `att` helper (producer confirms it exists in mod tests before use) and assert `evaluate_before_attempt(&attempts, 2.0)` gives Level::None for t1, t3 and t5. Controls: 'the producer returned no valid report' on the same attempts gives t1 Flag and t5 Pause; an `error_max_turns` detail whose text mentions '401' and 'token' behaves as the work failure. Test B: table over each environment kind (`LowDisk::text`, `check_timeout_reason`, producer-auth and usage-limit reasons) maps to Verdict::Environment and raises no signal. Test C: a TimedOut state with the producer prefix and API-error form maps to a non-Environment verdict. Run: `cargo test -p athena-core scope_guard`. Observes: The expired-login line raises no signal at t1, t3 or t5 for every construction of the reason; the controls do raise signals; every environment kind is Verdict::Environment; a TimedOut job still counts as work.
  Fails without the behavior: Without the classifier change the 401 reasons map to Verdict::Failed, so t1 flags at the third attempt and t5 reaches Pause at 120 minutes, and test A fails; the auth and usage-limit rows of test B fail too. Without the state gate, test C fails.
  Test seam: Production entry points `scope_guard::verdict_of` (added in step 3) and `evaluate_before_attempt` (exists, scope_guard.rs:392). Input is closing-reason text from production helpers (`LowDisk::text`, `check_timeout_reason`, `disk::producer_stop_reason` added in step 1) and a detail built by the real `StreamState::on_line` + `output_from_result` (exist, athena-agent stream.rs:77 and claude.rs:249) through `driver_detail` (test helper added in step 2). No fake; nothing under test sits in the helper.
- Criterion 4: Rig tests in scope_guard.rs `watcher_tests`, using `rig`, `two_rejections`, `write_mark`, `pause_path`, `Athena::line_pause`, `Athena::retry`, `Athena::status`. Test 1: `rig(0.8, 0.01, &[Some(0), Some(1)])` (two rejections of different criteria); a hand-written T4 pause makes `line_pause` None and `retry` is not refused. Test 2: `rig(0.8, 0.01, &[Some(0), Some(0)])` pauses naturally; `line_pause` is Some and the retry is refused. Test 3: `status().lines` omits the stale mark, keeps the real pause, keeps a mark for a line not in records, keeps a flag mark while the recompute is Flag or Pause. Test 4 (finding 1): `rig(1.0, 0.8, &[Some(0)])` gives a real T3 pause on attempt `a`. Then `ask("try again on a")` adds request `b`, which has no reading and no attempt. Assert `line_pause(records, b)` is still Some with signal T3; `r.go(&b)` is refused with T3, 1.60 and 1.00 (go call site); `r.athena.retry(a, &Quiet)` is refused with T3 (reading.rs:1034 call site) and commits nothing. A control where the stale mark is hand-written for a line that has only environment-stop attempts is dropped. Run: `cargo test -p athena-core scope_guard`. Observes: A stale stored pause is not returned and does not block the retry; pauses for real repeated rejections and a real T3 pause are returned and refuse `retry` and `go` even after a new request joined the line; status shows only marks that still hold.
  Fails without the behavior: Without the recompute, test 1 sees Some(pause) and the retry is refused. If the recompute read the newest request's forecast (the previous plan), test 4 fails: `line_pause` drops the T3 pause and `go` dispatches.
  Test seam: Production entry points `Athena::line_pause`, `Athena::retry`, `Athena::go` and `Athena::status`. The fake driver rig (`watcher_tests::rig`, `FakeAgentDriver`, scope_guard.rs:848+) exists and injects the fake driver into Athena. `line_state`, `line_forecast_p90` and `live_marks` are added in step 4. The hand-written pause file in test 1 is the stored state under test, not logic; test 4 uses a pause written by the real `try_watch_line`.
- Criterion 5: Unit tests in disk.rs `mod tests`. `resumable_stop(JobState::Closed(OutcomeState::Failed), Some(reason))` is true for: the stored legacy text 'the producer did not complete: ' + `driver_detail(json)` (401 JSON through StreamState::on_line and output_from_result, finding 2); `producer_stop_reason` of that detail; and a 429 detail. It is false for 'the producer returned no valid report', an admission refusal, an `error_max_turns` detail that mentions 401 and token, Rejected and Landed states, and a TimedOut state with the same expired-login reason. Also: `StopCause::of(...) == ProducerAuth` for the real-driver detail. Run: `cargo test -p athena-core disk`. Observes: An expired-login producer stop is resumable by `athena retry`; work failures and non-Failed states are not.
  Fails without the behavior: Without the change `is_environment_reason` matches only LOW_DISK and CHECK_TIMEOUT, so the 401 reason is not resumable and the first assertion fails.
  Test seam: Production entry points `disk::resumable_stop` (exists, disk.rs), `disk::StopCause::of` and `disk::producer_stop_reason` (added in step 1), `StreamState::on_line` and `output_from_result` (exist). `driver_detail` test helper added in step 2 only chains those two. No double.
- Criterion 6: Run the three Athena checks as separate whole commands. The full test run covers the new disk and scope_guard tests and the existing tests updated in steps 2, 3 and 7. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: Formatting is clean, clippy reports no warning, every workspace test passes.
  Fails without the behavior: Without the change the new criterion 3, 4 and 5 tests fail. A partial change also fails: removing `ENVIRONMENT_PREFIX` without updating its users breaks compilation, and recomputing the pause without fixing hand-made-pause tests (`retry_refuses_paused_line`) fails them.
  Test seam: The cargo workspace commands run every test through the real code paths of criteria 3 to 5. Seam exists; the new tests are added by steps 2, 6 and 7.
- Criterion 7: Show the diff touches no governance file and no reading rule. Run: `git diff --stat`. Observes: No athena.toml, defaults.toml or workflow file appears; only crates/athena-core/src files and CHANGELOG.md change.

Planned files:

- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4960964b887d → candidate 3107e5503ad5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  12 ++
 crates/athena-core/src/disk.rs         | 297 +++++++++++++++++++++++++-
 crates/athena-core/src/run/producer.rs |   6 +-
 crates/athena-core/src/scope_guard.rs  | 378 +++++++++++++++++++++++++++++----
 crates/athena-core/src/status.rs       |   3 +-
 5 files changed, 652 insertions(+), 44 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1846 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
