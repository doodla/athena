# Job 20261114-112646-s4bn: verify "Tell the planner its deadline and to answer within it"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The planner stage has a fixed 600s wall-clock deadline. It keeps no partial plan, gives no early sig
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: planner-deadline-exceeded-no-plan-no-partial-result (20261114-052225-bn5c)
> Cause: The planner stage has a fixed 600s wall-clock deadline. It keeps no partial plan, gives no early signal and has no resume or retry path. Nothing sizes the request against that deadline before planning starts, or records what the planner was...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261114-052207-pj25 (job 20261114-041123-hvyh); prevention it proposed: Make the planner stage survive its deadline. It should write its draft plan to the job record as it goes. On a deadline stop it should keep the draft and hand it to a resume or retry round, with the reason recorded as a timeout, not a plain...
> Introspection 20261114-112627-efkg (job 20261030-113229-kgrg); prevention it proposed: Make the planner stage keep partial output and report progress. Persist the draft plan and the last activity at each step. On a deadline stop, hold the job as resumable with that draft instead of failing it, and record the reason. Check the...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the planner's 600s `planner_timeout_s` kills it with no plan, and nothing in its brief states that deadline or tells it to size its reading to it (jobs 20261114-041123-hvyh and 20261030-113229-kgrg both ended 'the planner did not complete: deadline of 600s reached'). Fix by giving the planner the deadline in its brief with a rule to return the plan within it. Adds no refusal rule and no resume machinery.

Assumptions:
- Goal and audience: the planner agent (it receives the brief) and the owner who reads failed-job records. The job lets the planner see the deadline it runs under, so it stops reading and returns a plan in time.
- Previous jobs: 20261114-041123-hvyh and 20261030-113229-kgrg each failed in planning at the 600s deadline and have no candidate, so no prior files are kept or dropped. Their own requests (follow-up queueing, kill-switch resume) are not addressed here: this change is the shared cause only, and their scopes (followups.rs, steward.rs, manage.rs, state.rs, status.rs, commands.rs, docs/decisions.md) are out of scope and dropped. Their original requests can be retried afterwards, and the planner deadline will then be stated to the planner.
- Code survey: crates/athena-core/src/run/planning.rs (planning_round passes cfg.planner_timeout_s to agent_spec at line 162; a non-Success outcome calls stop_planning with 'the planner did not complete: <detail>' at line 190; plan_round seeds a PlanRevision, so a kept plan can be revised), crates/athena-context/src/briefs/planner.rs (PlannerInputs at line 33 and planner_brief at line 95 carry no deadline; the brief ends with rule constants), crates/athena-context/src/briefs.rs (tests for the planner brief, for example near lines 1818, 1835 and 2816), crates/athena-data/src/config.rs (planner_timeout_s at line 229, default 86400 in tests, 600 in athena.toml). The records allow-list is not touched: no new record or data file is added, so check_record_file and AnyRecord are unaffected. The property is measured directly: the text of planner_brief output built from PlannerInputs.
- Existing code: crates/athena-context/src/briefs/planner.rs (PlannerInputs, planner_brief and its rule constants such as CODE_CLAIMS_RULE) and crates/athena-core/src/run/planning.rs (planning_round builds PlannerInputs) are extended, with a new DEADLINE rule constant that follows the existing pattern; nothing is rebuilt.
- Field meaning: PlannerInputs.deadline_s is a new field, not a reuse. It is filled from the existing `[job_plan] planner_timeout_s` (crates/athena-data/src/config.rs:229, a wall-clock limit on one planner call, live athena.toml value 600) and is shown only to the planner role.
- No refusal rule is added, so no expected-refusal-share figure applies. A refusal rule is not needed here: the planner failed for lack of the deadline information, and giving it that information can prevent the failure.
- Prompt files are not edited. The rule lives as a Rust constant in the brief (like the existing rules), so the planner prompt token headroom (95 tokens) is not used.
- Config: athena.toml and defaults.toml are not changed. The 600s value stays and the brief reports whatever the config says.
- Resume or retry of a timed-out plan, persisting partial drafts, and the kill-switch resume are out of scope. They need agent streaming, and the brief information is the smaller fix that addresses the cause.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs/planner.rs adds a `deadline_s` field to PlannerInputs and a rule constant with a doc comment that names the root cause (the planner's fixed deadline was never stated to it; jobs 20261114-041123-hvyh and 20261030-113229-kgrg); `planner_brief` includes the rule text.
2. [content] The rule in the planner brief states the deadline in seconds and says that a planner killed at the deadline produces no plan. It tells the planner to sort its reading by what the criteria need, to stop exploring well before the deadline (the rule gives a fraction of the deadline), and to return the best complete plan it has, with unresolved items as `User:` or `Host:` open questions. Check with Grep on the constant in planner.rs.
3. [content] crates/athena-core/src/run/planning.rs fills PlannerInputs.deadline_s from `cfg.planner_timeout_s`, the same value passed to agent_spec, so the figure shown cannot differ from the enforced deadline. Check with git diff.
4. [behavior] A new unit test in crates/athena-context/src/briefs.rs builds planner briefs with two different `deadline_s` values (600 and 1200) for a first-round and a revision input. It asserts that each brief contains its own deadline figure and the stop-early instruction. It fails on the base tree, where no brief mentions the deadline. Run `cargo test -p athena-context`.
5. [behavior] A new test in crates/athena-core/src/run/tests.rs runs planning with the fake agent and `planner_timeout_s = 600`. It asserts the recorded planner invocation brief contains `600`. It also asserts that a planner stop at the deadline still ends the job as a planning stop with the reason 'deadline of …s reached', so existing behavior is unchanged. Run `cargo test -p athena-core`.
6. [content] CHANGELOG.md has a new entry for this change that names the root cause. No prompt file, athena.toml, defaults.toml, workflow, docs/decisions.md or records/ file is modified.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass. The recorded Athena checks on the final tree count as evidence (D115). Existing planner-brief tests still pass with the new field set in each PlannerInputs literal.

## Expected deliverables

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Persisting a draft plan, resume or retry after a planner deadline, and holding the job as resumable (needs streamed agent output; the brief change is the smaller fix).
- Job 20261114-041123-hvyh's follow-up queueing (followups.rs, steward.rs, manage.rs, reading.rs, lib.rs, drain.rs) and job 20261030-113229-kgrg's kill-switch stop and resume (run/mod.rs, state.rs, status.rs, commands.rs, introspection.rs, docs/decisions.md): neither job produced a candidate, so there is nothing to keep, and both stay as separate requests.
- New refusal rules or sizing gates in admission, changes to prompt files, athena.toml, defaults.toml, workflows, docs/decisions.md and records/.
- Changing the 600s value or the critic and producer deadlines.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the planner's brief shows the deadline and the stop-early rule, and that the value matches the enforced timeout, not that a field exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and run tests are added, and fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the brief that all planner calls use, so every planning job gets the deadline, not just the two named jobs.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is a prose rule in the brief, pinned by tests that check the brief text; it is marked as information, not enforcement.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of planner.rs. It shows `pub deadline_s: u64` in PlannerInputs with a doc comment, a `pub(super) const DEADLINE_RULE` whose doc comment names the root cause (the planner's fixed deadline was never stated to the planner, so it explored until killed and left no plan; jobs 20261114-041123-hvyh and 20261030-113229-kgrg ran under 600 s; 740a9e4 raised the live value to 86400, and this covers any configured deadline), and a `## Deadline` section in `planner_brief` that writes the rule. Run: `git diff -- crates/athena-context/src/briefs/planner.rs`. Observes: The field, the documented constant naming the root cause and both job ids, and its use in `planner_brief` are all in the diff.
- Criterion 2: Read the DEADLINE_RULE text in the diff. It must state the deadline in seconds (`deadline of {deadline_s} seconds`), say a planner stopped at the deadline `sends no plan`, say the planner has no clock of your own and count tool calls against the reading order, say to sort the reading by what the acceptance criteria need (deliverables, then the callers, tests and config each criterion names, then the rest only once every criterion has a demonstration and a seam), say `stop exploring by {stop_s} seconds` with `(half the deadline)`, and say to return the best complete plan with unresolved items as `User:` or `Host:` open questions. The constant has no newline. The criterion 4 test asserts the rendered text. Run: `git diff -- crates/athena-context/src/briefs/planner.rs`. Observes: Every required instruction is in the constant, including the no-clock sentence; the constant is one line.
- Criterion 3: Read the diff of planning.rs. The `PlannerInputs` literal at planning.rs:129 gains `deadline_s: cfg.planner_timeout_s`. `cfg` is the `JobPlanConfig` copy read at planning.rs:98-101, and the same `cfg.planner_timeout_s` goes to `agent_spec` at planning.rs:162. The criterion 5 test confirms they agree: the recorded spec timeout equals the figure in the brief for two configured values. Run: `git diff -- crates/athena-core/src/run/planning.rs`. Observes: One config value feeds both the shown figure and the enforced limit.
- Criterion 4: New unit test `planner_brief_states_its_deadline_and_the_stop_early_rule_in_both_rounds` in briefs.rs, next to the planner-brief tests (after briefs.rs:1859). For deadline 600 and 1200, and for `planner_inputs(None)` and `planner_inputs(Some(PlanRevision{ previous: plan(1), findings: vec![finding(Severity::Major, FindingSource::Critic, "F")], resumed_from: None }))`, set `inputs.deadline_s` and build the brief. Assert `brief.matches("## Deadline").count() == 1` and that the heading comes before `## Report`. Slice the section: the text after `## Deadline` up to `## Report`; every further assertion is on that slice, so the system facts, the history or the quoted plan cannot satisfy it. In the slice assert: `deadline of {n} seconds`; `stop exploring by {n/2} seconds` (300 for 600, 600 for 1200); `sends no plan`; `You have no clock of your own`; `User:`; `Host:`. It must NOT contain the other value's deadline phrase (`deadline of 1200 seconds` in the 600 brief and the reverse) nor the other value's stop phrase (`stop exploring by 600 seconds` in the 600 brief, `stop exploring by 300 seconds` in the 1200 brief). The rule text is absent from the part after `## Report`. Also assert `!DEADLINE_RULE.contains('\n')`, `!DEADLINE_RULE.contains("## Report")` and that the constant holds the `{deadline_s}` and `{stop_s}` placeholders. Run: `cargo test -p athena-context planner_brief_states_its_deadline`. Observes: Each of the four briefs carries its own deadline figure, its own stop figure, the no-clock sentence and the stop-early instruction in its Deadline section, and no brief carries the other deadline.
  Fails without the behavior: On the base tree the test does not compile, because `deadline_s` and `DEADLINE_RULE` do not exist. With the field but no rule text, every section assertion fails because no `## Deadline` section exists. If the figure were hard-coded to 600, the 1200 briefs would fail. If the stop figure were not computed, the 1200 briefs would fail on `stop exploring by 600 seconds`. If the no-clock sentence were dropped (finding 1), every brief would fail on `You have no clock of your own`. If the rule were left out of a revision brief, the revision cases would fail.
  Test seam: Production entry point: `athena_context::briefs::planner_brief(&PlannerInputs)` (planner.rs:95), reached from the tests module of briefs.rs, where `DEADLINE_RULE` is visible as `super::planner::DEADLINE_RULE` (`pub(super)`, the pattern of CODE_CLAIMS_RULE at briefs.rs:1836). Fixture: the existing `planner_inputs(revision)` helper (briefs.rs:2776-2813), `plan(1)` (briefs.rs:2704) and `finding(...)` (briefs.rs:2766), injected by setting `inputs.deadline_s` on the returned struct. Exists today; plan step 5 adds the `deadline_s` field to the helper literal. No double: the brief builder is production code.
- Criterion 5: New test `the_planner_brief_states_the_enforced_deadline_and_a_planner_deadline_still_stops_planning` in run/tests.rs, near the planning tests (after `run_planned`, run/tests.rs:1356-1373, and `prompts_of`, 1426). For each n in [600, 4321]: build the repository with the config text of `repo_with_planning(1)` (run/tests.rs:294-299) with `planner_timeout_s = n` through `repo_with_config` (run/tests.rs:302); keep an `Arc<Mutex<Vec<Duration>>>` for planner timeouts and a `Calls` log (run/tests.rs:1318); take `planning_script(vec![good_plan()], vec![], calls.clone())` (run/tests.rs:1322) as the inner script and wrap it in an outer `athena_agent::fake::Script`. For `Role::Planner` the wrapper pushes `spec.timeout`, calls the inner script, then sets `turn.outcome = InvocationOutcome::TimedOut`, `turn.structured = None` and `turn.detail = Some(format!("deadline of {n}s reached"))`, as `timing_out` does for the producer (run/tests.rs:7208-7219). Run it with `athena_with(&repo, &state, PASSES, crate::test_support::plenty_of_space, script)` and `finished_job(&athena)` (run/tests.rs:2439). NEW-BEHAVIOR assertions: the first planner prompt (`prompts_of(&calls, Role::Planner)`) contains `deadline of {n} seconds` and `stop exploring by {n/2} seconds`; the recorded planner timeout equals `Duration::from_secs(n)`. REGRESSION-GUARD assertions (existing behavior, the reason text comes from the double): the finished job's state is `OutcomeState::Failed`; the reason starts with `athena_data::records::PLANNING_STOPPED_PREFIX` and contains `the planner did not complete: deadline of {n}s reached`; no `Role::Producer` call is in `calls`. Run: `cargo test -p athena-core the_planner_brief_states_the_enforced_deadline`. Observes: For two configured values the brief the fake agent received carries that value, the enforced timeout is that many seconds, and a planner stop at the deadline still ends the job as a planning stop with the unchanged reason.
  Fails without the behavior: On the base tree the planner prompt has no `deadline of 600 seconds`, so the first assertion fails. If planning.rs filled the field from any value other than `cfg.planner_timeout_s` (for example a constant 600), the 4321 case would show a brief figure that differs from the `spec.timeout` of 4321 s. If the change altered the timeout path (planning.rs:190-198), the guard assertions on the reason or state would fail.
  Test seam: Production entry point: `Athena::go` into `planning_round` (planning.rs:129-198), with `stop_planning` (planning.rs:8-14) and `failure` producing the Failed state. Fixture: a fake agent script (`athena_agent::fake::Script`) injected through `athena_with(...)` (run/tests.rs:192), as in `run_planned` (run/tests.rs:1356-1373) and `timing_out` (7208-7219). The wrapper only records `spec.timeout` and rewrites the planner's turn outcome and detail. Brief building, the `agent_spec` timeout and the stop path stay production code. Exists today: all helpers are read at the cited lines. No plan step adds a helper; step 6 adds only the test function.
- Criterion 6: Read the CHANGELOG.md hunk and the diff stat. The new bullet goes under `## [Unreleased]` / `### Added` (CHANGELOG.md:8 and 30), above the entry at line 32. It names the root cause (the planner's fixed deadline was never stated to it; jobs 20261114-041123-hvyh and 20261030-113229-kgrg), that both ran under 600 s, that 740a9e4 raised the live value to 86400 and this covers any configured deadline, the fix (`deadline_s`, `DEADLINE_RULE`, the `## Deadline` section with the no-clock sentence), that it is information only with no refusal rule (0% of past readings refused) and no resume or draft persistence, and the test names. The stat lists only the five deliverables. Run: `git diff --stat`. Observes: Only the five deliverable files appear. No prompt file, athena.toml, defaults.toml, workflow, docs/decisions.md or records/ path is in the stat, and the changelog hunk (`git diff -- CHANGELOG.md`) names the root cause and 740a9e4.
- Criterion 7: Run the three checks Athena runs, on the final tree. Existing planner-brief tests and planning run tests are the regression guard for the shared paths this change touches. They pass only if (a) the `planner_inputs` helper literal carries `deadline_s` (the only test construction site of `PlannerInputs`; the other is planning.rs:129), (b) the new `## Deadline` section leaves the Report text and its size pin intact (briefs.rs:3963-4004) and does not disturb the `split("## Report").nth(1)` tests, (c) the prompt-set budgets are unchanged (`system_facts_budgets_match_computed`, crates/athena-cli/tests/budgets.rs), (d) the planning round-trip tests in run/tests.rs that use `run_planned` still run with the field filled from `cfg`, and (e) the refusal-corpus tests that `include_str!` CHANGELOG.md (crates/athena-core/tests/refusal_corpus.rs:1138, 1217, 1250, 1749, 2986, 3033, 3073, 3128, 3187) still find their needles, since an added entry removes none. The recorded Athena checks on the final tree count as evidence (D115). Run: `cargo test --locked`. Observes: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` exit 0, so every existing planner-brief and planning run test passes with the new field set.
  Fails without the behavior: A `PlannerInputs` literal missing `deadline_s` fails to compile (E0063), so `cargo test` and clippy fail. Text added inside the Report, or a rule constant that contains `## Report` or a newline, fails the pins at briefs.rs:1837-1838 or 3988. An unformatted new line fails `cargo fmt --check`. An unused or needlessly cloned helper in planner.rs fails `clippy -D warnings`. A prompt-file edit would fail `system_facts_budgets_match_computed`.
  Test seam: Production entry points: `planner_brief(&PlannerInputs)` (planner.rs:95) through the existing tests in briefs.rs at 1767, 1802, 1819, 1845, 2817, 2853, 2880, 2999, 3027, 3970 and 4009; and `Athena::go` into `planning_round` through the existing planning run tests that use `run_planned` (run/tests.rs:1356). Fixtures: the `planner_inputs(revision)` helper (briefs.rs:2776-2813) and `repo_with_planning`, `planning_script` and `athena_with` (run/tests.rs:294, 1322, 192), injected as in those tests. The seam exists today (read at the cited lines). Plan step 5 adds `deadline_s: 600` to the helper literal. The fmt and clippy checks run the real cargo commands on the real final tree with no double.

Planned files:

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 26a94984f70f → candidate e0b4626e3a55. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  1 +
 crates/athena-context/src/briefs.rs         | 41 ++++++++++++++++++
 crates/athena-context/src/briefs/planner.rs | 26 ++++++++++++
 crates/athena-core/src/run/planning.rs      |  1 +
 crates/athena-core/src/run/tests.rs         | 64 +++++++++++++++++++++++++++++
 5 files changed, 133 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-112646-s4bn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-112646-s4bn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43746113165 for e0b4626e3a55aa3f8cbc621dc200dfd7dc6ace84)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43746113165 for e0b4626e3a55aa3f8cbc621dc200dfd7dc6ace84)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43746113165 for e0b4626e3a55aa3f8cbc621dc200dfd7dc6ace84)
- ci-result: `not a check` passed (Check results for e0b4626e3a55aa3f8cbc621dc200dfd7dc6ace84 come from the CI run https://github.com/doodla/athena/actions/runs/43746113165 (715s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass. The recorded Athena checks on the final tree count as evidence (D115). Existing planner-brief tests still pass with the new field set in each PlannerInputs literal.)
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context planner_brief_states_its_deadline`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
   Compiling generic-array v0.14.7
   Compiling zmij v1.0.23
   Compiling serde v1.0.229
   Compiling libc v0.2.190
   Compiling syn v3.0.7
   Compiling serde_json v1.0.151
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling winnow v1.0.4
   Compiling cfg-if v1.0.5
   Compiling getrandom v0.3.4
   Compiling itoa v1.0.18
   Compiling thiserror v2.0.21
   Compiling memchr v2.8.3
   Compiling ref-cast v1.0.27
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_derive_internals v0.30.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling thiserror-impl v2.0.21
   Compiling serde_spanned v1.1.2
   Compiling digest v0.10.7
   Compiling cpufeatures v0.2.17
   Compiling jiff-core v0.1.1
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling sha2 v0.10.9
   Compiling toml v0.9.12+spec-1.1.0
   Compiling schemars_derive v1.2.2
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0432]: unresolved import `super::planner::DEADLINE_RULE`
    --> crates/athena-context/src/briefs.rs:1865:13
     |
1865 |         use super::planner::DEADLINE_RULE;
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `DEADLINE_RULE` in `briefs::planner`

error[E0609]: no field `deadline_s` on type `planner::PlannerInputs`
    --> crates/athena-context/src/briefs.rs:1877:24
     |
1877 |                 inputs.deadline_s = n;
     |                        ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `request`, `reading`, `checks`, `verifier_commands` ... and 2 others

error[E0560]: struct `planner::PlannerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2852:13
     |
2852 |             deadline_s: 600,
     |             ^^^^^^^^^^ `planner::PlannerInputs` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0432, E0560, E0609.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core the_planner_brief_states_the_enforced_deadline`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.00s

     Running tests/context_compression.rs (<job>/target/verify/debug/deps/context_compression-6112e9d39a3b55ee)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running tests/divergence.rs (<job>/target/verify/debug/deps/divergence-1ae4db8e62dcf162)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/lab_runs.rs (<job>/target/verify/debug/deps/lab_runs-6a6e3469b51eff19)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/learnings_recorded.rs (<job>/target/verify/debug/deps/learnings_recorded-ac4f2643c7b6c897)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

     Running tests/mechanism_explanation.rs (<job>/target/verify/debug/deps/mechanism_explanation-4037148411066e09)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-7: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 7 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
   Compiling colorchoice v1.0.5
   Compiling heck v0.5.0
   Compiling clap_lex v1.1.1
   Compiling anstream v1.0.0
   Compiling strsim v0.11.1
   Compiling crc32fast v1.5.2
   Compiling clap_builder v4.6.7
   Compiling simd-adler32 v0.3.10
   Compiling adler2 v2.0.1
   Compiling miniz_oxide v0.9.1
   Compiling xattr v1.6.1
   Compiling filetime v0.2.29
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling ref-cast v1.0.27
   Compiling thiserror v2.0.21
   Compiling clap_derive v4.6.7
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling clap v4.6.7
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
error[E0432]: unresolved import `super::planner::DEADLINE_RULE`
    --> crates/athena-context/src/briefs.rs:1865:13
     |
1865 |         use super::planner::DEADLINE_RULE;
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `DEADLINE_RULE` in `briefs::planner`

error[E0609]: no field `deadline_s` on type `planner::PlannerInputs`
    --> crates/athena-context/src/briefs.rs:1877:24
     |
1877 |                 inputs.deadline_s = n;
     |                        ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `request`, `reading`, `checks`, `verifier_commands` ... and 2 others

error[E0560]: struct `planner::PlannerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2852:13
     |
2852 |             deadline_s: 600,
     |             ^^^^^^^^^^ `planner::PlannerInputs` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0432, E0560, E0609.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
