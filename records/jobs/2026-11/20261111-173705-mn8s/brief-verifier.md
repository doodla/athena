# Job 20261111-173705-mn8s: verify "Make usage measures see every role's runs, with a standing real-records"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The retire rule was written against one data source, the per-job invocation list. Nothing required i
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: usage-measure-blind-to-roles-whose-runs-are-not-in-job-records (20261111-105030-a2bc)
> Cause: The retire rule was written against one data source, the per-job invocation list. Nothing required it to be run on the real records, or compared with known facts such as the steward, audience and auditor being active, before the job was jud...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261111-105017-5k6m (job 20261029-233051-jq6k); prevention it proposed: Any job that adds a usage, need or retire measure must replay it over the real records before it is judged. The replay must be checked against a list of roles known to be active. A role whose runs are not in the measured source must be show...
> Introspection 20261111-122618-q1ha (job 20261029-233058-fveq); prevention it proposed: Add a code check that every new status or measure stage is run over the real records and must give both outcomes: it must not be true for all items or false for all items, unless a stated reason is recorded. It must also be compared with kn...
> Introspection 20261111-145341-v9p4 (job 20261111-122637-aawj); prevention it proposed: Add a standing code check that runs every role-usage or need measure over the real job and non-job run records. It must fail when any role the records show as active (steward, audience, auditor) would be flagged for retirement. Make the all...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Retry of 20261029-233051-jq6k, 20261029-233058-fveq and 20261111-122637-aawj. Root cause: a usage/need measure read only the per-job invocation list, while steward, audience and auditor runs live in separate Records lists and nobody gave the measure author that fact. Fix by giving the author the fact (docs on JobRecords::invocations, system facts), keeping one all-runs accessor, and adding a standing test that replays the measure over a known-active-roles fixture and every loaded run source.

Assumptions:
- Code survey: read crates/athena-core/src/store.rs (JobRecords::invocations doc at ~l.29-32, Records steward_runs/audience_runs/auditor_runs, all_invocations ~l.520 and its test all_invocations_covers_every_loaded_invocation ~l.713), crates/athena-core/src/role_need.rs (WINDOW_JOBS, ON_DEMAND_ROLES, counts_as_work, RoleNeed::compute and its replay tests), crates/athena-core/src/status.rs (role_need field, STATUS_MEASURE_LABELS) and docs/spec.toml OLY-9-11. No new record or data file is added under crates/, so check_record_file/AnyRecord round-trip is not touched. Usage is measured directly from Records, not a stand-in.
- Existing code: crates/athena-core/src/store.rs (Records::all_invocations), crates/athena-core/src/role_need.rs and docs/system-facts.md are extended, not rebuilt; the working copy already holds much of the earlier fix, so the producer first diffs it against the verifier findings below and fills only the gaps.
- Prior job 20261029-233051-jq6k (blocker: steward/audience/auditor flagged; minor: compute untested, no window, display test) is addressed by the all_invocations read, a windowed per-class count and Records-level tests; its files role_need.rs, status.rs, lib.rs, display.rs, spec.toml and CHANGELOG.md are kept as deliverables.
- Prior job 20261029-233058-fveq (improvement_cycle evaluated-stage check always true; misplaced doc comment in status.rs; unlinked chain) is a different measure of the same class (a status stage true for every item); its stage is checked by the new all-outcomes test only for both-outcomes, and its files introspection.rs and tests/status_build_measurement.rs are excluded in out_of_scope unless the both-outcomes test shows the stage is always true, in which case the stage is renamed honestly. status.rs is kept.
- Prior job 20261111-122637-aawj (not replayed against known-active roles; invocations field undocumented; system facts silent; dropped/stale jobs counted; extra files) is addressed by documenting the split on JobRecords::invocations and in docs/system-facts.md, a known-active replay test that includes the auditor as on-demand/not measured, and counting only jobs that ran a model. Its files status_role_need.rs and xtask/shard-timings.txt are kept because the new test binary must be registered.
- Goal: stop future usage/need/retire measures from being blind to roles whose runs are outside job records. Audience: measure authors (producers) and verifiers, and whoever reads athena status. The preferred remedy is information (doc comments and system facts), not a refusal rule; no refusal of readings is added, so no refusal-share number applies.
- Environment: none; no CI, push or history reads.
- Governance files: docs/spec.toml (OLY-9-11 mechanism/measure text) may change and is named in a criterion; athena.toml, defaults.toml and workflows do not change.

## Acceptance criteria

1. [content] The doc comment on `pub invocations` in JobRecords (crates/athena-core/src/store.rs) states that steward, audience and auditor runs are not in it and names Records::all_invocations; checked with Grep.
2. [content] docs/system-facts.md states where each role's runs are recorded (job invocations vs steward_runs, audience_runs, auditor_runs) and that usage measures must read Records::all_invocations; checked with Grep for those names.
3. [behavior] Run `cargo test -p athena-core role_need`: a fixture with steward, audience and auditor runs outside job records yields no retire candidate for them (auditor listed as on-demand/not measured), while a genuinely unused in-job role is a candidate, so the measure gives both outcomes. The test fails if role_need reads job.invocations instead of all_invocations.
4. [behavior] Run `cargo test -p athena-core store`: a Records::from_parts test fails if any loaded Invocation (including a hypothetical fourth list) is missing from all_invocations.
5. [content] Dropped, stale and abandoned jobs that never ran a model are excluded from the 8-job window and class totals; a unit test in role_need.rs covers it.
6. [content] docs/spec.toml OLY-9-11 mechanism and measure match the code (all_invocations, on-demand roles not measured, ends with `status: role need`); CHANGELOG.md has an entry naming the root cause; xtask/shard-timings.txt lists any new test binary.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass. The new tests fail on the pre-change code (verifier checks by reading the diff for assertions on the auditor/steward/audience outcomes).

## Expected deliverables

- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/role_need.rs (modify)
- docs/system-facts.md (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_role_need.rs (modify)
- xtask/shard-timings.txt (modify)
- crates/athena-core/src/lib.rs (modify)

## Out of scope

- Adding a generic refusal rule or gate on new measures
- Automatically creating or retiring roles
- OLY-9-12 (value of a new role against a simpler alternative)
- crates/athena-data/src/records/introspection.rs improvement_cycle and tests/status_build_measurement.rs from job 20261029-233058-fveq (dropped unless the both-outcomes test shows its stage is always true)
- Changes to prompts, athena.toml, defaults.toml or workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the measure's real outcomes on known-active roles, not that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the information gap at the records source and its docs, not one measure.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion included.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check by Grep of the doc comment above `pub invocations: Vec<Invocation>` in JobRecords. Run: `Grep pattern 'PARTIAL source|Records::all_invocations' in crates/athena-core/src/store.rs`. Observes: Lines 29-32 say the steward's, audience's and auditor's runs are never in the list, and name `Records::all_invocations`. Present in the base; edit only if the Grep fails.
- Criterion 2: Content check by Grep of the 'Where invocations are recorded' fact. Run: `Grep pattern 'JobRecords::invocations|steward_runs|audience_runs|auditor_runs|Records::all_invocations' in docs/system-facts.md`. Observes: All five names appear in one fact (l.44-48), which states where each role's runs are recorded and that a usage measure reads `Records::all_invocations`. Present in the base; edit only if the Grep fails.
- Criterion 3: Run the role_need unit tests, including the new `every_role_loaded_through_from_parts_is_counted_and_classified` and `every_role_is_measured_or_on_demand`. The new output test builds a Records with `Records::from_parts(&RecordSet{files}, &[])`. The files are one Invocation record per `ALL_ROLES` entry (distinct seq per role) in each of 10 jobs, plus one Outcome per job covering Landed, Rejected, Failed and TimedOut. Every invocation's `started_at` is that job's time. It then calls `RoleNeed::compute`. Run: `cargo test -p athena-core role_need`. Observes: For every role in `ALL_ROLES`, `need.roles` has an entry with count > 0. For "steward", "audience" and "auditor", the entry has source SOURCE_OUTSIDE_JOBS and flagged == false, appears in `not_measured`, and is absent from `retire_candidates`. `retire_candidates` is empty, since every measured role ran. The existing base tests still give the other outcome: `known_active_roles_are_never_retire_candidates_and_an_unused_role_is` (role_need.rs:335) has planner, critic and an unused custom role as candidates, and `the_measure_yields_both_outcomes_on_a_mixed_fixture` (l.410) gives both empty and non-empty candidate lists.
  Fails without the behavior: If `compute` read `job.invocations`, steward, audience and auditor would have count 0 and no not_measured entries, so the assertions fail. A role added to `ALL_ROLES` but not routed or chained into `all_invocations` fails on count == 0. A role in neither `MEASURED_ROLES` nor `ON_DEMAND_ROLES` fails the classification test.
  Test seam: Production entry point `RoleNeed::compute(&Records)` (role_need.rs:82), with guards of 8 jobs that ran a model and `counts_as_work` (l.68). Fixture: `Records::from_parts` (store.rs:429-437, exists) fed `RecordFile` values built in the test as at role_need.rs:455-464, with invocations from `status::planning_cost_tests::invocation` (exists). It is injected as the argument. The measure's logic is not in the fixture. `ALL_ROLES` and `MEASURED_ROLES` are added by plan steps 2 and 4; the new tests by steps 5 and 6.
- Criterion 4: Run the store tests: `all_invocations_covers_every_loaded_invocation`, rewritten to load one Invocation record per `ALL_ROLES` entry through `Records::from_parts`, plus the new `all_roles_lists_every_role_variant`. Run: `cargo test -p athena-core store`. Observes: Each (seq, role) loaded comes back out of `all_invocations`. The three out-of-job roles are not in any `job.invocations`. `ALL_ROLES` holds every variant named in the `Role` enum source. A hypothetical fourth list needs a routed role, which `ALL_ROLES` forces into the from_parts test, and an unchained list fails on the missing (seq, role) pair.
  Fails without the behavior: If `all_invocations` dropped any list (for example `auditor_runs`), `seen` lacks that role's (seq, role) and the equality assertion fails. A new Role variant missing from `ALL_ROLES` fails the completeness test on count and membership.
  Test seam: Production entries `Records::from_parts(&RecordSet, &[])` and `Records::all_invocations` (store.rs:524), both exist. Fixture: `RecordFile` values built in the test (store.rs:728-745, exists). The `ALL_ROLES` const and the completeness test are added by plan steps 2 and 3. The completeness test reads the enum source with `include_str!("../../athena-data/src/records.rs")`.
- Criterion 5: Run the existing window test (role_need.rs:535-566). It adds newer jobs closed as dropped, stale, abandoned, declined and closed, each with planner and critic runs, plus a landed job with no invocation. Run: `cargo test -p athena-core role_need::tests::jobs_that_never_ran_a_model_are_left_out_of_the_window`. Observes: window_jobs is 8, retire_candidates is still [planner, critic], and failed_jobs is 2. This is in the base and is not edited.
  Fails without the behavior: If `counts_as_work` dropped the state check or the ran-a-model check, dropped and stale jobs would enter the window, planner and critic would count as used, and the candidates assertion would fail.
  Test seam: `RoleNeed::compute` with the in-module `job()` fixture (role_need.rs:252), which is injected as Records. `counts_as_work` (l.68-78) requires a Closed landed, rejected, failed or timed-out state and a non-on-demand invocation under the job id. Exists.
- Criterion 6: Content check by Grep, then read the new CHANGELOG bullet. Run: `Grep 'all_invocations|not measured|status: role need' in docs/spec.toml; Grep 'Root cause' in CHANGELOG.md; Grep 'status_role_need' and 'athena-core --lib' in xtask/shard-timings.txt`. Observes: spec.toml OLY-9-11 (l.1495-1501) names `Records::all_invocations`, says steward, audience and auditor are not measured, and ends `status: role need`; edit only if a Grep shows a mismatch. The new CHANGELOG bullet names the root cause, 41539a7, b670592 and the guards. shard-timings.txt already lists `athena-cli --test status_role_need` (l.41) and `athena-core --lib` (l.51). The new tests are unit tests in the core lib binary, so no new binary and no timing line is added.
- Criterion 7: Run the three checks Athena runs. Then read `git diff` for the outcome assertions in the new role_need test `every_role_loaded_through_from_parts_is_counted_and_classified`. Those are: count > 0 per `ALL_ROLES` role; steward, audience and auditor with source SOURCE_OUTSIDE_JOBS, unflagged and in `not_measured`; none of the three in `retire_candidates`. Also read the rewritten store test and the classification test. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three pass. The diff holds the auditor, steward and audience outcome assertions, driven by `ALL_ROLES` and `RoleNeed::compute` output. On the pre-change code these tests would not compile, since `ALL_ROLES` and `MEASURED_ROLES` do not exist. The substantive reason to trust them is the fails-without below. The base tests (role_need.rs:335-378, :433-479; status_role_need.rs through `athena status`) remain as further evidence.
  Fails without the behavior: With `compute` changed to read `job.invocations`, the new output test fails on count == 0 for steward, audience and auditor, and on their absence from `not_measured`. With a role dropped from `all_invocations`, both the store test and the output test fail. With a role added to `Role` but not to `ALL_ROLES`, the completeness test fails. With a role in neither classification list, the classification test fails.
  Test seam: CLI path: `athena.status()` then `athena_cli::display::status(&view)` over records seeded into a temporary git repo by `seed()` in status_role_need.rs:96, where steward, audience and auditor runs are separate Invocation records routed by `from_parts`. Exists; edited only if a Grep shows a gap. Unit path: as in criterion 3.

Planned files:

- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/role_need.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_role_need.rs (modify)
- xtask/shard-timings.txt (modify)
- crates/athena-core/src/lib.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b1f31eee80c9 → candidate 98853029eea3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  10 +++
 crates/athena-core/src/role_need.rs | 172 +++++++++++++++++++++++++-----------
 crates/athena-core/src/store.rs     |  57 +++++++++---
 3 files changed, 178 insertions(+), 61 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-173705-mn8s/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-173705-mn8s/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43078837334 for 98853029eea37f7472d147ce6d01fd307b9e69ba)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43078837334 for 98853029eea37f7472d147ce6d01fd307b9e69ba)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43078837334 for 98853029eea37f7472d147ce6d01fd307b9e69ba)
- ci-result: `not a check` passed (Check results for 98853029eea37f7472d147ce6d01fd307b9e69ba come from the CI run https://github.com/doodla/athena/actions/runs/43078837334 (847s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass. The new tests fail on the pre-change code (verifier checks by reading the diff for assertions on the auditor/steward/audience outcomes).)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core role_need`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/role_need.rs, crates/athena-core/src/store.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:491:35
    |
491 |         for role in crate::store::ALL_ROLES {
    |                                   ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `MEASURED_ROLES` in this scope
   --> crates/athena-core/src/role_need.rs:492:28
    |
492 |             let measured = MEASURED_ROLES.contains(&role);
    |                            ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:508:64
    |
508 |             .map(|(k, s)| job(k as u32 + 1, *s, &crate::store::ALL_ROLES))
    |                                                                ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:520:35
    |
520 |         for role in crate::store::ALL_ROLES {
    |                                   ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:641:13
    |
641 |             ALL_ROLES.len(),
    |             ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:647:21
    |
647 |             assert!(ALL_ROLES.contains(&role), "ALL_ROLES lacks Role::{name}");
    |                     ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:649:26
    |
649 |         for (n, role) in ALL_ROLES.iter().enumerate() {
    |                          ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:650:22
    |
650 |             assert!(!ALL_ROLES[..n].contains(role), "{role:?} is listed twice");
    |                      ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:751:21
    |
751 |         let roles = ALL_ROLES;
    |                     ^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 9 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core store`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/role_need.rs, crates/athena-core/src/store.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:491:35
    |
491 |         for role in crate::store::ALL_ROLES {
    |                                   ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `MEASURED_ROLES` in this scope
   --> crates/athena-core/src/role_need.rs:492:28
    |
492 |             let measured = MEASURED_ROLES.contains(&role);
    |                            ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:508:64
    |
508 |             .map(|(k, s)| job(k as u32 + 1, *s, &crate::store::ALL_ROLES))
    |                                                                ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:520:35
    |
520 |         for role in crate::store::ALL_ROLES {
    |                                   ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:641:13
    |
641 |             ALL_ROLES.len(),
    |             ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:647:21
    |
647 |             assert!(ALL_ROLES.contains(&role), "ALL_ROLES lacks Role::{name}");
    |                     ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:649:26
    |
649 |         for (n, role) in ALL_ROLES.iter().enumerate() {
    |                          ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:650:22
    |
650 |             assert!(!ALL_ROLES[..n].contains(role), "{role:?} is listed twice");
    |                      ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:751:21
    |
751 |         let roles = ALL_ROLES;
    |                     ^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 9 previous errors
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core role_need::tests::jobs_that_never_ran_a_model_are_left_out_of_the_window`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/role_need.rs, crates/athena-core/src/store.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:491:35
    |
491 |         for role in crate::store::ALL_ROLES {
    |                                   ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `MEASURED_ROLES` in this scope
   --> crates/athena-core/src/role_need.rs:492:28
    |
492 |             let measured = MEASURED_ROLES.contains(&role);
    |                            ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:508:64
    |
508 |             .map(|(k, s)| job(k as u32 + 1, *s, &crate::store::ALL_ROLES))
    |                                                                ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `ALL_ROLES` in module `crate::store`
   --> crates/athena-core/src/role_need.rs:520:35
    |
520 |         for role in crate::store::ALL_ROLES {
    |                                   ^^^^^^^^^ not found in `crate::store`

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:641:13
    |
641 |             ALL_ROLES.len(),
    |             ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:647:21
    |
647 |             assert!(ALL_ROLES.contains(&role), "ALL_ROLES lacks Role::{name}");
    |                     ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:649:26
    |
649 |         for (n, role) in ALL_ROLES.iter().enumerate() {
    |                          ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:650:22
    |
650 |             assert!(!ALL_ROLES[..n].contains(role), "{role:?} is listed twice");
    |                      ^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `ALL_ROLES` in this scope
   --> crates/athena-core/src/store.rs:751:21
    |
751 |         let roles = ALL_ROLES;
    |                     ^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 9 previous errors
```
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
