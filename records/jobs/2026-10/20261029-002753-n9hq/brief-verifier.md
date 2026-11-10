# Job 20261029-002753-n9hq: verify "Treat a weekly/session usage-limit reply as an environment stop"

## Request (verbatim)

> Priority: high
> First small slice of handling model usage limits. When the agent driver's reply says the weekly or session usage limit was hit (text such as "You've hit your weekly limit" with a reset time), classify the invocation as an environment stop using the existing environment-reason path in disk.rs, so the job is not counted as a failure, and have athena drain stop with the reason 'usage limit' and the reset time. Nothing else: no status display, no new record kind, no changes to retry or resume.
> Acceptance:
> - A test with a fake driver reply carrying the limit text shows the job ends as an environment stop, not a failure.
> - A drain test shows the stop reason 'usage limit' with the reset time.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-agent/src/claude.rs, crates/athena-core/src/disk.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Teach the driver and disk.rs's environment-reason path to recognise "You've hit your weekly/session limit … resets <time>" replies (not only "API Error: 429") as a usage-limit environment stop, keeping the reset time. Make athena drain stop at once with reason "usage limit" and that reset time. Audience: the athena operator reading drain output and the verifier running tests.

Assumptions:
- Code survey: read crates/athena-agent/src/claude.rs (api_error_status reads only 'API Error: NNN' details; output_from_result builds the detail as '<subtype>: <first 300 chars>'), crates/athena-core/src/disk.rs (USAGE_LIMIT, StopCause::of, producer_stop_reason, is_environment_reason), crates/athena-core/src/run/producer.rs (a non-success invocation goes through producer_stop_reason, then self.failure for usage limits, closing failed with an environment reason), crates/athena-core/src/scope_guard.rs (environment reasons are not counted as attempts) and crates/athena-cli/src/drain.rs (Stop enum; ItemEnd::Failed sets Stop::Environment only after an item fails twice).
- No record file or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched. The Stop::label and Stop::meaning tables, and any test that iterates Stop::ALL, may need updating if a variant is added.
- Existing code: the usage-limit path already exists for 'API Error: 429' (disk::USAGE_LIMIT, StopCause::UsageLimit, producer_stop_reason). This job extends it with text detection and a reset time; it does not add a parallel path.
- Assumption: the limit text looks like "You've hit your weekly limit · resets Nov 12, 3pm" (or 'session limit'). The detector matches 'hit your weekly limit' / 'hit your session limit' case-insensitively, and takes any text after 'resets' as the reset time. A reply without a reset time still counts as a usage limit, with no time shown.
- Assumption: 'drain stop with the reason usage limit' means a new Stop variant labelled 'usage limit' (not the generic 'environment'). It ends the run on the first such job, since every later item would hit the same limit. The reset time goes in the drain's stop output. The existing 'fails twice' Environment rule for logins and disk is unchanged.
- Reset time is carried inside the existing closing-reason string (USAGE_LIMIT prefix plus detail), so no new record kind is needed. The drain reads it back from the last reason.
- No governance files (athena.toml, defaults.toml, workflows) change; the only non-source deliverable is CHANGELOG.md.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-agent/src/claude.rs (fake driver script emitting an is_error result with 'You've hit your weekly limit · resets <time>') passes under `cargo test -p athena-agent`, and shows the classifier reports a usage limit with the reset time. A 'session limit' reply is covered too.
2. [behavior] A test in crates/athena-core/src/disk.rs shows producer_stop_reason on the limit text gives a reason starting with disk::USAGE_LIMIT that includes the reset time, and that is_environment_reason is true and StopCause::of gives UsageLimit. Run `cargo test -p athena-core`.
3. [behavior] A run-level test with a fake driver reply carrying the limit text (beside the existing usage-limit test in crates/athena-core/src/run/tests.rs, or an equivalent in a file listed as a deliverable) shows the job closes as an environment stop, not a counted failure, and keeps nothing. Run `cargo test -p athena-core`.
4. [behavior] A drain test in crates/athena-cli (drain.rs unit test or tests/drain.rs) shows the run stops after the first job whose reply says the limit was hit, with the stop reason 'usage limit' and the reset time in the output. Run `cargo test -p athena-cli`.
5. [behavior] Controls: text that only mentions a limit in a model's own prose (under a subtype other than success/error) and a normal success reply are not classified as usage limits. Existing 429 and 401 tests still pass.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has a new entry, in the file's existing format, describing the usage-limit environment stop and the drain stop with reset time. `git diff` shows it.
8. [content] The diff limits itself to the deliverables: no status display, no new record kind and no change to retry or resume logic (the resumable_stop and keeps_unfinished_work behaviour is unchanged). Checked with `git diff --stat`.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)

## Out of scope

- Status display of the usage limit or reset time
- A new record kind
- Changes to retry or resume (resumable_stop, keeps_unfinished_work)
- Waiting until the reset time or automatic restart of the drain
- Changes to the existing login (401) or low-disk handling

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus cargo test, clippy -D warnings and fmt --check to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name tests that show the job closing as an environment stop and the drain stopping with the reason and reset time.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the four named files, plus tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The classifier is extended in the shared disk.rs path that the scope guard and drain already read, not patched in one caller.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New unit tests in `stand_in_tests` of crates/athena-agent/src/claude.rs. The STAND_IN script gets modes `weekly_limit` and `session_limit`, each emitting `{"type":"result","subtype":"success","is_error":true,...,"result":"You've hit your weekly limit · resets Nov 12, 3pm"}`. The tests call `driver.invoke` with `counted(dir, "weekly_limit", ..)` as `a_usage_limit_is_not_retried` does, then `usage_limit_hit(&out.detail.unwrap())`. Run: `cargo test -p athena-agent usage_limit`. Observes: The outcome is Error and the driver is called once (no 401-style retry). `usage_limit_hit` gives Some with `resets == Some("Nov 12, 3pm")`; the session variant gives the same; a reply with no 'resets' gives Some with `resets == None`.
  Fails without the behavior: Without `usage_limit_hit` the tests do not compile. A version returning None for these texts fails the Some(..) assertions.
  Test seam: Production entry points: `ClaudeCodeDriver::invoke` and `usage_limit_hit`. The fake is the shell stand-in `claude`, injected through `driver(stand_in(dir), Registry::new(state))` with MODE in `spec.env`. The seam exists (claude.rs:799-861); plan step 2 adds the two modes.
- Criterion 2: New tests in the `tests` module of crates/athena-core/src/disk.rs. They build the detail with the existing `driver_detail(&result_line("success", "You've hit your weekly limit · resets Nov 12, 3pm"))`, which goes through `StreamState::on_line` and `output_from_result`. Then they call `producer_stop_reason`, `StopCause::of`, `is_environment_reason`, `resumable_stop`, `keeps_unfinished_work` and `usage_limit_reset`, also on the stored unprefixed form `the producer did not complete: <detail>`. Run: `cargo test -p athena-core usage_limit_text`. Observes: The reason starts with `USAGE_LIMIT` and contains 'resets Nov 12, 3pm'. `StopCause::of` is UsageLimit; `is_environment_reason` is true; `usage_limit_reset` gives Some("Nov 12, 3pm"), None for a bare 429; `resumable_stop(Failed)` is true and `keeps_unfinished_work` false, as for 429 today.
  Fails without the behavior: Without the new branch the reason is `the producer did not complete: ...` and `StopCause::of` gives Work.
  Test seam: Production entry points: `producer_stop_reason`, `StopCause::of`, `usage_limit_reset`. The fixture is the existing `#[cfg(test)]` helpers `result_line` and `driver_detail` (disk.rs:526-554), which exist. No double. `usage_limit_reset` is added by plan step 3.
- Criterion 3: New test in crates/athena-core/src/run/tests.rs beside `a_login_expiry_with_no_edit_keeps_nothing_and_a_usage_limit_keeps_nothing` (line 5382). It builds `limit = driver_detail(result_line("success", "You've hit your weekly limit · resets Nov 12, 3pm"))`, runs `athena_with(&repo, &state, PASSES, plenty_of_space, producer_ending(InvocationOutcome::Error, &limit, true, Arc::default()))`, then `finished_job(&athena)`. Loops over weekly and session texts. Run: `cargo test -p athena-core weekly_limit_reply`. Observes: `done.state == Failed`; `done.reason` starts with `disk::USAGE_LIMIT` and contains the reset time; `is_environment_reason(&done.reason)` is true; the record has no candidate and no progress-note; `scope_guard::counted` skips an attempt with that reason, so it is not a counted failure.
  Fails without the behavior: Without the classifier change the reason is `the producer did not complete: ...`, so `is_environment_reason` is false and the starts_with assertion fails.
  Test seam: Production entry point: the producer stage in `Athena::run` (run/producer.rs:391-410). The fake driver is the `producer_ending` Script injected through `athena_with`. Both exist (run/tests.rs:5277 and the existing test at 5416-5444). The detail comes from the real `driver_detail` helper.
- Criterion 4: New test in crates/athena-cli/tests/drain.rs near line 1213. It uses `Stub { items: ["a","b"], outcomes: [Failed, Failed], run_reason: Some(reason) }` and calls `drain::drain(&stub, limits(5.0), &ui)`. `reason` comes from `athena_core::disk::producer_stop_reason("success: You've hit your weekly limit · resets Nov 12, 3pm")`, leaked with `Box::leak` because `Stub.run_reason` is `&'static str`. It asserts on `report.stop`, `stub.log()` and `display::drain(&report)`. Cases: weekly text, session text, no reset time, bare 429 reason. Run: `cargo test -p athena-cli --test drain usage_limit`. Observes: `report.stop == Stop::UsageLimit`; `stub.log() == ["read a", "run a"]` (no retry, b never read); `display::drain` contains `stopped     usage limit: ` and a `note        usage limit reached; it resets Nov 12, 3pm` line; the no-reset case says no reset time was given; the 429 reason also stops after one run. `Stop::ALL` has 10 entries each with non-empty label and meaning, and the `drain --help` test (tests/drain.rs:2436) finds 'usage limit'.
  Fails without the behavior: Without the `run_job` check, a is retried (the log has 'reread a') and the stop is Environment or QueueEmpty.
  Test seam: Production entry point: `drain::drain`, which uses the `Steps` trait. The fake is the existing `Stub` (tests/drain.rs:41, `fn run` at 218), injected as `&stub`. It exists. The `Stop` variant and the `run_job` check are added by plan step 6.
- Criterion 5: New tests `not_a_usage_limit_*` in claude.rs and disk.rs using `driver_detail(result_line(subtype, text))`. Subtype `error_max_turns` or `error_during_execution` carries the exact limit text; subtype `success` carries prose such as 'I fixed the weekly limit handler' or 'Note: you've hit your weekly limit earlier'; one case is a normal success reply with is_error false. Existing 429 and 401 tests run unchanged. Run: `cargo test -p athena-agent not_a_usage_limit && cargo test -p athena-core -- not_a_usage_limit a_failure_of_the_work_is_never_an_environment_stop the_real_driver_detail_of_an_expired_login every_kind_of_environment_stop && cargo test -p athena-agent a_usage_limit_is_not_retried`. Observes: `usage_limit_hit` is None, `StopCause::of(producer_stop_reason(..))` is Work, and `is_environment_reason` is false. The existing 401 and 429 tests still pass.
  Fails without the behavior: A detector that searched the words anywhere or ignored the subtype would flag the prose and `error_max_turns` cases, and these tests would fail.
  Test seam: Production entry points: `usage_limit_hit`, `producer_stop_reason`, `StopCause::of`. The fixture is `driver_detail`/`result_line`, which exist. No double.
- Criterion 6: Run the three checks Athena runs on the result, in this order, over the whole workspace after all steps are done. The tests that break if the behavior is missing or half-done are in this run: `drain_help_documents_the_flags_and_lists_every_stop_reason` (tests/drain.rs:2436), the `Stop::ALL` test in src/drain.rs (~5193), the edited and new tests in tests/drain.rs, the scope-guard and disk tests that call `producer_stop_reason`, and `system_facts_budgets_match_computed`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0: formatting is clean; clippy reports no warning (no dead code, no non-exhaustive or unreachable match arm after adding `Stop::UsageLimit` and removing it from the `ItemEnd::Failed` arm); every test in the workspace passes, including the new tests from criteria 1-5.
  Fails without the behavior: If `Stop::UsageLimit` were missing from main.rs's drain help, `drain_help_documents_the_flags_and_lists_every_stop_reason` fails. If the `ItemEnd::Failed` arm or the old test at tests/drain.rs:1214 were left as is, the old test fails (the run now stops with UsageLimit after one run). If the variant were not added to `Stop::label`/`meaning`/`ALL`, the build or the Stop::ALL test fails. If the classifier change is absent, the new tests of criteria 1-4 fail; if code is left unused or badly formatted, clippy -D warnings or fmt --check fails.
  Test seam: Production entry points: the cargo fmt, clippy and test commands over the workspace, which Athena itself runs as its fmt, clippy and test checks. The fixtures are the existing and new test suites (claude.rs stand-in script, disk.rs helpers, run/tests.rs `producer_ending`, tests/drain.rs `Stub`). Nothing is injected for the commands. The seam exists today; the new test code it runs is added by plan steps 2, 4, 5 and 8.
- Criterion 7: Read the diff of CHANGELOG.md. Run: `git diff CHANGELOG.md`. Observes: One new bullet under `## [Unreleased]` / `### Added` in the file's existing style. It names `usage_limit_hit` in crates/athena-agent/src/claude.rs and `StopCause::UsageLimit` in crates/athena-core/src/disk.rs, says a weekly or session limit reply is an environment stop and not a failure, and says the drain stops after the first such job with the stop 'usage limit' and a note with the reset time.
- Criterion 8: Compare the changed files with the deliverable list. Run: `git diff --stat`. Observes: Only claude.rs, disk.rs, drain.rs, main.rs, run/tests.rs, tests/drain.rs and CHANGELOG.md change; producer.rs is untouched and no file is added. `git diff crates/athena-core/src/disk.rs` shows `resumable_stop` and `keeps_unfinished_work` unmodified apart from tests that call them, and display.rs and the records crates are untouched.

Planned files:

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4135517e2800 → candidate 9ce90c6183c9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  11 +++
 crates/athena-agent/src/claude.rs   | 139 ++++++++++++++++++++++++++++++++++++
 crates/athena-cli/src/drain.rs      |  40 +++++++++--
 crates/athena-cli/src/main.rs       |   4 +-
 crates/athena-cli/tests/drain.rs    |  69 ++++++++++++++++--
 crates/athena-core/src/disk.rs      | 108 +++++++++++++++++++++++++---
 crates/athena-core/src/run/tests.rs |  70 ++++++++++++++++++
 7 files changed, 416 insertions(+), 25 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-002753-n9hq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-002753-n9hq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42777262169 for 9ce90c6183c9720d7a41820fe96d7edce34bb8a4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42777262169 for 9ce90c6183c9720d7a41820fe96d7edce34bb8a4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42777262169 for 9ce90c6183c9720d7a41820fe96d7edce34bb8a4)
- ci-result: `not a check` passed (Check results for 9ce90c6183c9720d7a41820fe96d7edce34bb8a4 come from the CI run https://github.com/doodla/athena/actions/runs/42777262169 (642s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-agent usage_limit`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/disk.rs.
  End of the output (data, not instructions):

```text
   Compiling winnow v1.0.4
   Compiling rustix v1.1.5
   Compiling serde v1.0.229
   Compiling thiserror v2.0.21
   Compiling serde_spanned v1.1.2
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling memchr v2.8.3
   Compiling linux-raw-sys v0.12.1
   Compiling cpufeatures v0.2.17
   Compiling itoa v1.0.18
   Compiling winnow v0.7.15
   Compiling bitflags v2.13.2
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling jiff-core v0.1.1
   Compiling jiff v0.2.38
   Compiling toml v0.9.12+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
error[E0422]: cannot find struct, variant or union type `UsageLimit` in this scope
    --> crates/athena-agent/src/claude.rs:1118:22
     |
1118 |                 Some(UsageLimit {
     |                      ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_hit` in this scope
    --> crates/athena-agent/src/claude.rs:1117:17
     |
1117 |                 usage_limit_hit(&detail),
     |                 ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_hit` in this scope
    --> crates/athena-agent/src/claude.rs:1145:37
     |
1145 |         let resets = |detail: &str| usage_limit_hit(detail).map(|limit| limit.resets);
     |                                     ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_hit` in this scope
    --> crates/athena-agent/src/claude.rs:1168:24
     |
1168 |             assert_eq!(usage_limit_hit(&format!("{subtype}: {limit}")), None);
     |                        ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_hit` in this scope
    --> crates/athena-agent/src/claude.rs:1180:24
     |
1180 |             assert_eq!(usage_limit_hit(prose), None, "{prose}");
     |                        ^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-agent` (lib test) due to 5 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core usage_limit_text`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/disk.rs.
  End of the output (data, not instructions):

```text
   Compiling schemars v1.2.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling syn v2.0.119
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `usage_limit_reset` in module `crate::disk`
    --> crates/athena-core/src/run/tests.rs:5480:30
     |
5480 |                 crate::disk::usage_limit_reset(&done.reason).as_deref(),
     |                              ^^^^^^^^^^^^^^^^^ not found in `crate::disk`

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1471:24
     |
1471 |             assert_eq!(usage_limit_reset(&written).as_deref(), Some("Nov 12, 3pm"));
     |                        ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1476:24
     |
1476 |             assert_eq!(usage_limit_reset(&stored).as_deref(), Some("Nov 12, 3pm"));
     |                        ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1488:20
     |
1488 |         assert_eq!(usage_limit_reset(&no_time), None);
     |                    ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1491:20
     |
1491 |         assert_eq!(usage_limit_reset(&too_many), None);
     |                    ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1494:13
     |
1494 |             usage_limit_reset("the producer returned no valid report"),
     |             ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1498:13
     |
1498 |             usage_limit_reset(&producer_stop_reason(&expired_login_detail())),
     |             ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1511:24
     |
1511 |             assert_eq!(usage_limit_reset(&reason), None);
     |                        ^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 8 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core weekly_limit_reply`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/disk.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `usage_limit_reset` in module `crate::disk`
    --> crates/athena-core/src/run/tests.rs:5480:30
     |
5480 |                 crate::disk::usage_limit_reset(&done.reason).as_deref(),
     |                              ^^^^^^^^^^^^^^^^^ not found in `crate::disk`

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1471:24
     |
1471 |             assert_eq!(usage_limit_reset(&written).as_deref(), Some("Nov 12, 3pm"));
     |                        ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1476:24
     |
1476 |             assert_eq!(usage_limit_reset(&stored).as_deref(), Some("Nov 12, 3pm"));
     |                        ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1488:20
     |
1488 |         assert_eq!(usage_limit_reset(&no_time), None);
     |                    ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1491:20
     |
1491 |         assert_eq!(usage_limit_reset(&too_many), None);
     |                    ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1494:13
     |
1494 |             usage_limit_reset("the producer returned no valid report"),
     |             ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1498:13
     |
1498 |             usage_limit_reset(&producer_stop_reason(&expired_login_detail())),
     |             ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `usage_limit_reset` in this scope
    --> crates/athena-core/src/disk.rs:1511:24
     |
1511 |             assert_eq!(usage_limit_reset(&reason), None);
     |                        ^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 8 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-cli --test drain usage_limit`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-agent not_a_usage_limit && cargo test -p athena-core -- not_a_usage_limit a_failure_of_the_work_is_never_an_environment_stop the_real_driver_detail_of_an_expired_login every_kind_of_environment_stop && cargo test -p athena-agent a_usage_limit_is_not_retried`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
