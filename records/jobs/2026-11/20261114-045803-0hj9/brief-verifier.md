# Job 20261114-045803-0hj9: verify "Make scope guard read each attempt's stop reason and near-complete"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The scope guard treats all spend on a line as one number. It does not record why an attempt stopped,
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: scope-guard-counts-environment-stop-as-line-spend-the person running Athena-lifts-pause (20261028-065858-bztk)
> Cause: The scope guard treats all spend on a line as one number. It does not record why an attempt stopped, and it does not know how much work is left. A stop caused by the environment (low disk) looks the same as a stop caused by the work being t...
> Seen 5 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261028-065847-kmx7 (job 20261028-055601-hg0n); prevention it proposed: Record a stop reason on every attempt. Have the scope guard exclude attempts that ended on an environment stop (low disk, container reclaim) from the spend and attempt counts, or count them separately. When an attempt has a kept candidate t...
> Introspection 20261028-103312-b53e (job 20261028-065911-m6m5); prevention it proposed: Make the scope guard record the stop reason for each attempt and the state of the kept candidate. Spend from an environment stop, such as low disk, must not count toward the line's progress-less spend. If a candidate is kept and has passed ...
> Introspection 20261028-124847-gf2n (job 20261028-103330-3d47); prevention it proposed: Compute the pause instead of storing it. On each guard evaluation (and whenever the guard's rule changes), recompute the line's spend from the per-attempt stop reasons under the current rule, and clear or keep the pause on the result. If a ...
> Introspection 20261029-195138-3pqb (job 20261029-111202-2bpc); prevention it proposed: Record a stop reason with each attempt's spend (environment, mechanical plan finding, real critic or verifier failure, producer failure). The guard should count only the reasons that signal runaway work. Store with each pause the rule versi...
> Introspection 20261103-101210-m2aa (job 20261103-092929-s7ts); prevention it proposed: When the guard is about to pause a line, it should first read the latest verdict. If most criteria are met (for example 8 of 9) and every gap is a named, bounded finding, the guard should not pause. It should queue a targeted retry limited ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the scope guard (crates/athena-core/src/scope_guard.rs) reduces an attempt to a verdict, a cost and minutes, so it does not know how far a rejected attempt got. Environment stops are already excluded (disk.rs StopCause), so this job adds the missing fact for the last introspection (20261103-092929-s7ts), a near-complete rejection, and pauses on spend only when work is not converging.

Assumptions:
- Goal and audience: the person running Athena, and the drain, should not see a line paused on spend when the last attempt stopped for a reason that does not signal runaway work. The verifier judges the diff and tests.
- Code survey: read crates/athena-core/src/scope_guard.rs (AttemptRecord, Verdict::Environment, t1_retries, t3_basis, t3_cost, t5_wall_time, verdict_of, attempt_of, request_records, evaluate_before_attempt, and the stale-pause recheck near line 1776) and crates/athena-core/src/disk.rs (StopCause, is_environment_reason, resumable_stop). Entry point: evaluate_before_attempt over attempts built by request_records from job records; attempt_of reads job.verification.criteria. No record or data file is added, so check_record_file, AnyRecord and the round-trip test are untouched. The property is measured directly on AttemptRecord lists in unit tests in scope_guard.rs.
- Existing code: crates/athena-core/src/scope_guard.rs (AttemptRecord, t3_basis, t3_cost, attempt_of) and crates/athena-core/src/disk.rs (StopCause, is_environment_reason) are extended, not replaced. Environment stops (low disk, check timeout, producer auth, usage limit, build slot) are already excluded from T1, T2, T5 and kept-work T3 spend, which covers introspections 20261028-065847-kmx7 (job 20261028-055601-hg0n), 20261028-103312-b53e (job 20261028-065911-m6m5), 20261028-124847-gf2n (job 20261028-103330-3d47) and 20261029-195138-3pqb. The remaining uncovered case is 20261103-101210-m2aa (job 20261103-092929-s7ts).
- Previous jobs 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47 were an unrelated replay feature (athena check-reading, reading replay) that stopped on low disk or an expired login. Their findings were environment stops, not code findings, and are now classified as environment by the guard. That replay feature is not part of this request. Their candidates' changed files (CHANGELOG.md, athena-cli main.rs, briefs.rs, lib.rs, reading.rs, run/mod.rs, store.rs, verify.rs, records.rs, docs/records.md) are dropped and not kept here, except CHANGELOG.md, which is a deliverable.
- Design: AttemptRecord gains the count of criteria met and the count of criteria in the last verification (None when unknown). The guard's T3 spend pause does not fire when the last attempt was rejected with at least 80% of criteria met (an estimate, a code constant) and its stop is a bounded gap; it downgrades to a flag so a targeted retry is allowed. A line with no met-criteria data behaves as before. The 80% threshold is a judgment call, labeled as such in the docs comment.
- This adds a rule that holds less work, not more, so no refusal-rule share applies. It refuses no readings.
- Term line: a line of work is a request, its retries and re-asks, grouped by group_lines in scope_guard.rs.
- Environment: no CI workflow, push, PR or history read is involved.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/scope_guard.rs builds a line whose last attempt is Rejected with 8 of 9 criteria met and spend over T3_PAUSE_RATIO times the forecast, and asserts t3_cost returns a level below Pause (checked by running cargo test); the same test with 3 of 9 met asserts Pause, so it fails without the change.
2. [behavior] A unit test shows an attempt record built by attempt_of from a rejected job's verification carries the met and total criterion counts (checked by running cargo test), and a Failed or Environment attempt carries none.
3. [content] The doc comments of AttemptRecord and t3_cost in crates/athena-core/src/scope_guard.rs name the root cause (the guard treated every attempt's spend as one number and did not record how far a rejected attempt got), and say that environment stops stay excluded through crate::disk::is_environment_reason, with no second list of environment reasons.
4. [behavior] Existing scope_guard tests for environment stops (environment_stop_is_not_counted_but_a_rejection_is, every_kind_of_environment_stop_is_an_environment_attempt) still pass under cargo test.
5. [content] CHANGELOG.md has a new entry for this change, and no refusal rule is added to crates/athena-core/src/reading.rs.
6. [behavior] cargo fmt --check, cargo clippy and cargo test pass on the candidate tree.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/disk.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- The reading replay and athena check-reading from jobs 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47; their candidate files (store.rs, verify.rs, records.rs, briefs.rs, reading.rs, run/mod.rs, main.rs, lib.rs, docs/records.md) are dropped
- Automatic disk cleanup or disk reservation before the verifier
- Adding any refusal rule for readings
- Changing the thresholds of T1, T2, T4 or T5

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new unit tests plus cargo test, clippy and fmt passing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix is in the guard's attempt record, not at one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name tests that fail without the change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New tests whose names all contain `near_complete`, in two layers. (A) Pure unit tests in `mod tests` of crates/athena-core/src/scope_guard.rs call `t3_cost` and `evaluate_before_attempt` on `AttemptRecord` lists. (B) Whole-path tests in `mod watcher_tests` drive `Athena::go` through `watch_line`, `line_state` and `line_pause` on the Rig, whose fake verifier judges 5 criteria.

Test A1 `a_near_complete_rejection_does_not_pause_t3` uses a forecast p90 of 2.0 throughout:
- one Rejected attempt, 8 of 9 met, cost 3.0 (1.5 times the forecast): `t3_cost` is below Pause (Flag).
- the same with 3 of 9: Pause.
- 7 of 9: Pause.
- 4 of 5: Flag (the 0.8 boundary).
- 9 of 9 with a Rejected verdict: Pause.
- 8 of 9 with bounded false: Pause.
- 8 of 9 with no progress: Pause.
- a Failed attempt that carries a progress value: Pause.
- [8 of 9, then 3 of 9] over the limit: Pause (the last attempt decides).
- [3 of 9, then 8 of 9] over the limit: Flag.
- [8 of 9, environment stop]: Flag (the stop is skipped).
- `evaluate_before_attempt` on the 8-of-9 line: t3 is Flag and overall is Flag.

Test A1b `near_complete_targeted_retry_is_used_once_when_over_the_limit` (finding 2):
- [8 of 9 at 0.5, 8 of 9 at 3.0]: Flag, because spend was under the limit at the earlier attempt, so the allowance is unused.
- [8 of 9 at 3.0, 8 of 9 at 0.1]: Pause.
- [8 of 9 at 3.0, 8 of 9 at 3.0, 8 of 9 at 0.1]: Pause.
- [8 of 9 at 3.0, Failed, 8 of 9 at 0.1]: Flag, because the counted attempt before is not near-complete.
- [8 of 9 at 3.0, environment stop, 8 of 9 at 0.1]: Pause, because the stop is skipped.

Test A2 `near_complete_rejections_still_pause_through_t4_and_t5`: two near-complete rejections of one criterion pause through T4 with a large forecast, and a near-complete attempt of 130 agent minutes pauses through T5.

Test A3 `near_complete_t3_detail_says_why_no_pause`: `facts` for the 8-of-9 line has a T3 flag whose detail names 'met 8 of 9' and says the pause was withheld. A flag at 1.0 times the forecast has no such clause.

B1 `a_near_complete_rejection_is_flagged_and_the_next_attempt_is_dispatched`: `rig_with_criteria(1.0, 0.8, &[Some(0), None], 5)`. `r.ask("describe the demo in the readme")` then `r.go` returns Rejected with 4 of 5 met and spend 1.6 against p90 1.0. Assert there is no pause file, and a flag file whose `signal_names()` is 'T3', whose spend is about 1.6 and whose reason contains 'met 4 of 5'. Assert `line_pause` is None. Then `r.ask("try again on <a>")` and `r.go` returns Landed (not refused).

B2 `a_second_near_complete_rejection_over_the_spend_limit_pauses` (round-1 finding 4): `rig_with_criteria(1.0, 0.8, &[Some(0), Some(1)], 5)` and `r.two_rejections()`. The second attempt's T4 is No (criteria 0 and 1 differ), T1 flags (3rd attempt next), and T3 is Pause (the earlier attempt was near-complete and already at 1.6 of a 1.5 limit). The pause file exists with a T3 fact at level pause, and `r.go` of a third retry returns an error whose text contains 'T3'.

B3 `a_t3_pause_stored_before_the_change_for_a_near_complete_line_does_not_block`: after the first rejection of B1, delete the flag file and write `write_mark(r.pause(&a), &mark("pause", a.as_str(), "T3"))`. `line_pause` is None. `line_flag` is None and `status().lines` is empty (the documented gap). `go` of the retry returns Landed. Run: `cargo test -p athena-core near_complete`. Observes: `t3_cost` returns Flag for 8 of 9 and for 4 of 5, and Pause for 3 of 9, 7 of 9, 9 of 9, unbounded, no progress, a Failed last attempt and a second consecutive near-complete rejection that is over the limit. The under-limit-then-over-limit sequence flags. On the real path, a near-complete line is flagged not paused and its retry runs. The second consecutive one pauses with T3. A stored old pause stops blocking.
  Fails without the behavior: On the base, `AttemptRecord` has no `progress` field, so the tests do not compile. If only the field were added and `t3_cost` left alone, `t3_cost(&[near(8,9,3.0)], 2.0)` returns Pause and the 'below Pause' assertion fails, and B1 would find a pause file and a refused `go` (run/mod.rs:1040). If the downgrade ignored the earlier attempt, the A1 and A1b over-the-limit sequences and B2 fail. If it ignored the spend at the earlier attempt, the under-then-over case fails. If it counted environment stops, the environment-stop sequences fail.
  Test seam: Production entry points `t3_cost` and `evaluate_before_attempt` (scope_guard.rs:321, 415), and for the whole path `Athena::go` -> `watch_line` -> `try_watch_line` -> `line_state` (scope_guard.rs:1005-1087, 735-744) plus `Athena::line_pause` (:822). Fixtures: the unit tests use the existing helper `att` (scope_guard.rs:2091), which step 8 extends with `progress: None`, and a new helper `near(met, total, cost)` (a Rejected attempt with a bounded `Progress`) added in step 8. The whole-path tests use the existing Rig (scope_guard.rs:1169-1396), driven by `FakeAgentDriver`, injected through the `script` closure in `rig_build` (:1254-1340). Step 9 adds a `criteria: usize` parameter to `rig_build` and a `rig_with_criteria` wrapper. The two existing callers (:1186, :1192) pass 2, so their behaviour is unchanged. The fake verifier only chooses which criterion fails; no guard logic lives in it. No admission rule caps the criteria count in a reading (admission.rs:184 reads it only for planning, which is off in the rig).
- Criterion 2: Two new tests in `mod watcher_tests`; both call the private production function `attempt_of(&job, resumed, unlanded)` (scope_guard.rs:580), which is reachable from the in-module test.

Test 1 `attempt_of_records_how_far_a_near_complete_rejection_got`: `rig(1.0, 0.8, &[Some(0)])`, `r.ask`, `r.go` (Rejected), then clone the job from `r.records().jobs`.
- `attempt_of(&job, false, false)` gives verdict Rejected and `progress == Some(Progress { met: 1, total: 2, bounded: true })`.
- Set `verification` to None: progress None.
- Set `outcome.state` to Failed with reason `the producer returned no valid report`: verdict Failed, progress None, though the verification is still present.
- Set `outcome.state` to Failed with the real `crate::disk::LowDisk { free, required }.text()` as in scope_guard.rs:2337-2341: verdict Environment, progress None.
- Set `outcome.state` to Landed: Accepted, progress None.

Test 2 `near_complete_gap_is_bounded_only_by_findings_of_the_gap` (finding 1) overwrites `verification.criteria`, `.findings` and `.agent_verdict` on the cloned rejected job with the output of the real `crate::verify::judge(&VerdictInputs { .. })`. It is fed `VerdictOutput`, `CriterionVerdictOutput` and `FindingOutput` literals (athena_context::outputs, outputs.rs:245-300). The rest of `VerdictInputs` is empty or None, and `map_base` and `map_candidate` are set only in case (b). Cases, each checked by `attempt_of(..).progress` and `near_complete`:
- 9 content criteria, 8 met, no findings: 8 of 9, bounded, near_complete true.
- The same plus one verifier Major on the gap: bounded.
- The same plus two verifier Majors: unbounded.
- (a, critic case) 9 criteria of which one is Behavior, unmet, citing a command that was not seen running, plus a verifier Major and the Athena Major that judge adds: two Majors for one gap, bounded.
- (b, critic case) 9 content criteria, 8 met, commands verified, plus one Athena Major from `map_base` `| A1-1 | x | Open |` and `map_candidate` `| A1-1 | x | Covered |` (verify.rs:249-290): unbounded.
- An unassessed criterion (the output lists 8 of 9 criteria): bounded, with its single Athena Major covered by `judged_gaps`.
- A verifier Blocker: unbounded.
- `request_misread` true in the output: unbounded.
- `agent_verdict` None: unbounded.
- The verifier invocation's `outcome` set to Killed in the clone: unbounded. Its `seq` removed from `job.invocations`: unbounded.
- A `CheckRun` with passed false and no note (built field by field as in status.rs:2662-2675): unbounded. With a note starting `PRE_EXISTING_NOTE`: bounded again. Run: `cargo test -p athena-core attempt_of_records_how_far && cargo test -p athena-core near_complete_gap_is_bounded`. Observes: A Rejected attempt carries (met, total, bounded). `bounded` drops for a misread, a Blocker, surplus verifier Majors, an unrelated Athena Major, a missing verdict, an unfinished verifier and a blocking failed check. It stays true for one gap that produced two Majors (verifier plus Athena). A Failed, Environment or Accepted attempt and a Rejected one with no verification carry None.
  Fails without the behavior: The field does not exist on the base, so the tests do not compile. If `attempt_of` filled progress for Failed or Environment attempts, the None assertions fail. If `gaps_only` counted all Majors together, case (a) fails. If it ignored the Athena Major's source, case (b) fails. If it ignored the misread, Blocker, check, verdict or invocation outcome, the matching unbounded assertion fails.
  Test seam: Production entry point `attempt_of` (scope_guard.rs:580) with `progress_of` and `gaps_only` (new, step 3) behind it. The Rig (exists, scope_guard.rs:1169) produces a real job with a real verification and a real verifier invocation. `JobRecords` fields are pub (store.rs:22-52) and the type is Clone and Default, so the clone is edited in place, and `job.state()` follows `outcome.state` (store.rs:140-149). The verification fields in the judge-based cases come from the real `crate::verify::judge` (verify.rs:409, pub), not from a test double. The `Progress` type and the field are added in step 1.
- Criterion 3: Read the doc comments in the diff for `AttemptRecord`, the new `progress` field, `Progress`, `NEAR_COMPLETE_SHARE`, `near_complete`, `t3_cost` and the module header paragraph. Run: `git diff -- crates/athena-core/src/scope_guard.rs`. Observes: The `AttemptRecord` and `t3_cost` doc comments each state the root cause: the guard treated every attempt's spend as one number and did not record how far a rejected attempt got. Each says environment stops stay excluded through `crate::disk::is_environment_reason` (via `verdict_of`), with no second list of reasons. The `t3_cost` doc also states the allowance exactly: a near-complete last attempt is flagged, not paused, unless the counted attempt before it was also near-complete and the spend was already at the pause limit at that attempt. It also says a pause stored earlier lifts by recompute and the line shows no flag until its next attempt. The diff adds no string literal in non-test code that lists environment reasons and does not touch `verdict_of` or disk.rs logic.
- Criterion 4: Run the whole `scope_guard::tests` module. It holds `environment_stop_is_not_counted_but_a_rejection_is` (scope_guard.rs:2389) and `every_kind_of_environment_stop_is_an_environment_attempt` (:2332). The filter `environment_stop` alone would also match `run::tests::a_check_killed_at_its_deadline_closes_as_an_environment_stop`, so the module path is used. Both test bodies stay as they are; only the helper `att` gains `progress: None`. Run: `cargo test -p athena-core scope_guard::tests::`. Observes: Both tests pass, together with `only_a_job_closed_failed_is_an_environment_attempt`, `a_line_of_expired_login_stops_raises_no_signal` and `resumed_retry_is_judged_on_remaining_stages`.
  Fails without the behavior: If the change counted an environment stop as a rejection, or gave an Environment attempt progress or near-complete status, `environment_stop_is_not_counted_but_a_rejection_is` (all five levels None for Environment, Pause for the same line as Rejected) or `every_kind_of_environment_stop_is_an_environment_attempt` would fail.
  Test seam: Exists: production `verdict_of` (scope_guard.rs:561), `evaluate_before_attempt`, `t1_retries`, with the in-file fixtures `att` and `failed_with` (:2268). No new seam.
- Criterion 5: Read the stat and the CHANGELOG.md hunk. Run: `git diff --stat`. Observes: The stat lists crates/athena-core/src/scope_guard.rs, status.rs, disk.rs, crates/athena-cli/src/drain.rs and CHANGELOG.md only, and not crates/athena-core/src/reading.rs. CHANGELOG.md has one new bullet at the top of `## [Unreleased]` / `### Changed`. It names the root cause and the fix in AttemptRecord and t3_cost, says environment stops stay excluded through is_environment_reason with no second list, gives the 0.8 estimate and states the allowance rule exactly (a near-complete rejection that trips T3 is flagged once; a second one straight after with spend still over the limit pauses; an attempt under the limit does not use the allowance). It says a pause stored before the change lifts by recompute and the line shows no flag until its next attempt. It lists the limits: findings carry no criterion index, so verifier Majors are tied to unmet criteria by count; criteria differ in weight; the effect on past lines was not measured because records/ is absent from the planning copy. It names the follow-up that docs/design/scope-guard.md section 5 is not updated here, and states that no refusal rule is added (0% of past accepted readings refused). It names the test commands.
- Criterion 6: Run the three checks as Athena runs them (fmt, clippy with -D warnings, test), as one chained command, or as three separate commands. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0 on the candidate tree. The tests include the new near_complete tests, the existing scope_guard tests, and the athena-cli drain tests (crates/athena-cli/tests/drain.rs:6044-6170) that read pauses and flags through the real Steps.
  Fails without the behavior: A missed `progress` field in a constructor fails the build. A private helper left unused (`progress_of`, `gaps_only`, `targeted_retry`, `t3_by_spend`, `counted_at`) fails clippy -D warnings with dead_code. A broken intra-doc link in the doc-only edits fails clippy -D warnings, so names that are not in scope are written with plain backticks. A banned word in CHANGELOG.md or the docs fails the policy test (xtask/tests/policy.rs).
  Test seam: Exists: the repository's own fmt, clippy and test checks.

Planned files:

- crates/athena-core/src/scope_guard.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/disk.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base fa5bd1dfbe93 → candidate 6a6e9c8a4bad. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |   1 +
 crates/athena-cli/src/drain.rs        |   4 +-
 crates/athena-core/src/disk.rs        |   3 +
 crates/athena-core/src/scope_guard.rs | 693 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/status.rs      |   3 +-
 5 files changed, 684 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045803-0hj9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045803-0hj9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43717966625 for 6a6e9c8a4bade88482fea99295c6a1112ddee701)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43717966625 for 6a6e9c8a4bade88482fea99295c6a1112ddee701)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43717966625 for 6a6e9c8a4bade88482fea99295c6a1112ddee701)
- ci-result: `not a check` passed (Check results for 6a6e9c8a4bade88482fea99295c6a1112ddee701 come from the CI run https://github.com/doodla/athena/actions/runs/43717966625 (1022s): fmt success, clippy success, test success.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core near_complete`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/scope_guard.rs, crates/athena-core/src/status.rs.
  End of the output (data, not instructions):

```text
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2279:57
     |
2279 |             attempt_of(&missing, false, false).unwrap().progress,
     |                                                         ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2300:53
     |
2300 |             attempt_of(&job, false, false).unwrap().progress
     |                                                     ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0560]: struct `scope_guard::AttemptRecord` has no field named `progress`
    --> crates/athena-core/src/scope_guard.rs:2473:13
     |
2473 |             progress: None,
     |             ^^^^^^^^ `scope_guard::AttemptRecord` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2480:11
     |
2480 |         a.progress = Some(Progress {
     |           ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2661:19
     |
2661 |         unbounded.progress.as_mut().unwrap().bounded = false;
     |                   ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2664:21
     |
2664 |         no_progress.progress = None;
     |                     ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0425]: cannot find function `near_complete` in this scope
    --> crates/athena-core/src/scope_guard.rs:2668:18
     |
2668 |         assert!(!near_complete(&failed));
     |                  ^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425, E0433, E0560, E0609.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 30 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core attempt_of_records_how_far && cargo test -p athena-core near_complete_gap_is_bounded`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core scope_guard::tests::`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/scope_guard.rs, crates/athena-core/src/status.rs.
  End of the output (data, not instructions):

```text
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2279:57
     |
2279 |             attempt_of(&missing, false, false).unwrap().progress,
     |                                                         ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2300:53
     |
2300 |             attempt_of(&job, false, false).unwrap().progress
     |                                                     ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0560]: struct `scope_guard::AttemptRecord` has no field named `progress`
    --> crates/athena-core/src/scope_guard.rs:2473:13
     |
2473 |             progress: None,
     |             ^^^^^^^^ `scope_guard::AttemptRecord` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2480:11
     |
2480 |         a.progress = Some(Progress {
     |           ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2661:19
     |
2661 |         unbounded.progress.as_mut().unwrap().bounded = false;
     |                   ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0609]: no field `progress` on type `scope_guard::AttemptRecord`
    --> crates/athena-core/src/scope_guard.rs:2664:21
     |
2664 |         no_progress.progress = None;
     |                     ^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `unmet_criterion`, `cost_dollars`, `agent_minutes`, `plan_size` ... and 2 others

error[E0425]: cannot find function `near_complete` in this scope
    --> crates/athena-core/src/scope_guard.rs:2668:18
     |
2668 |         assert!(!near_complete(&failed));
     |                  ^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425, E0433, E0560, E0609.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 30 previous errors
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
