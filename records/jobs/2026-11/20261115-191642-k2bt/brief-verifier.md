# Job 20261115-191642-k2bt: verify "Drop deadline/pacing text from briefs; check queued fixes still apply"

## Request (verbatim)

> From Sri (the owner), word for word: "the planner is now told its time limit and asked to finish within it. That's pointless" and "It's not human, plus it'll take however long it takes. Yeah, also do that"
> 
> Background (not Sri's own words; added so the request can be read on its own). Job 6ee6c79d844c "Tell the planner its deadline and to answer within it" landed on 15 Nov after the time limits were raised to 24 hours; it was queued when the planner limit was 600 s. "Also do that" agrees to two things:
> 1. Remove the deadline and pacing text from the planner brief (and from the producer brief added by "Tell the producer its deadline and pacing in its brief", 64a92cc76d29), since a model does not pace itself like a person and the work takes as long as it takes.
> 2. Before a queued fix runs, check whether the problem it was filed for still exists under current configuration and code, and close it with the reason if not. Example still queued: "Scale planner time limit when revision escalates effort" (20261115-113508-66w1, retry of 20261114-052309-w3zt), which no longer applies now that limits are 24 hours.

## Reading (accepted by the user)

Per Sri: remove the "## Deadline" section from the planner brief (job 6ee6c79d844c) and the "## Time budget" section from the producer brief (64a92cc76d29), keeping the enforced limits themselves. Also add a planner-prompt rule to first check whether a queued fix's problem still exists under current config and code, and say so instead of building if it does not. Job 20261115-113508-66w1 is addressed as dropped-from-scope: see assumptions.

Assumptions:
- Code survey: read crates/athena-context/src/briefs/planner.rs (DEADLINE_RULE, deadline_section at lines 94-114, call at 181, PlannerInputs.deadline_s at 44), crates/athena-context/src/briefs.rs (time_budget_section 763, call 696, producer inputs deadline_s 373, tests ~1865-2072), crates/athena-core/src/run/planning.rs (deadline_s: timeout_s at 156, escalated factor 145/527), run/producer.rs:51 producer_deadline_s, run/mod.rs:1244, run/tests.rs:1438. No records or data files are added, so no record round-trip applies.
- Goal and audience: Sri, who judges whether the planner and producer briefs stop telling models a clock to pace against; models do not pace like humans and work takes as long as it takes. The deadline is still enforced, only no longer announced.
- Job 20261115-113508-66w1 (landed, pass; findings: minor fails-without check for criterion 2 could not show it, nit about string-inequality escalation detection): the scaling it added is kept as is, and its two findings concern test demonstration and style of code this job does not touch, so they no longer need fixing here. Its files planning.rs, tests.rs and CHANGELOG.md are kept as deliverables of this job only for the deadline_s plumbing removal and the changelog entry. Job 20261114-052309-w3zt (dispatched, never verified) is superseded by 66w1; its candidate files are the same three. The queued retry of that scaling is moot now that limits are 24 hours; it is not reworked here.
- Part 2 is read as an agent-behavior change (planner prompt rule), not a queue-closing mechanism in Rust; closing already-queued jobs such as 66w1's siblings is an owner action outside this repository change. The reader prompt has 2 tokens spare so it is not edited.
- Existing code: extends crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs.rs and crates/athena-context/prompts/planner.md; no new module.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs/planner.rs no longer defines DEADLINE_RULE or deadline_section, and the planner brief has no '## Deadline' section (git diff shows removal; Grep for 'Deadline' in briefs/planner.rs finds none).
2. [content] crates/athena-context/src/briefs.rs no longer defines time_budget_section and the producer brief has no '## Time budget' section; Grep for 'Time budget' in crates/ finds none; the deadline_s input fields that only fed these sections are removed from the planner and producer brief inputs and their callers (planning.rs, producer.rs, run/mod.rs), while the enforced agent-spec deadline is unchanged.
3. [behavior] Brief tests assert the rendered planner brief and producer brief contain neither 'deadline of' / 'seconds of wall-clock' nor 'Time budget', and the old tests pinning those sections (briefs.rs ~1865-1880, run/tests.rs:1438) are removed or inverted; run `cargo test -p athena-context` and `cargo test -p athena-core`; the new absence assertions fail if either section is restored.
4. [content] crates/athena-context/prompts/planner.md gains one short rule (within its 95-token headroom): when the request is a queued fix or retry, first check from the code and config whether its problem still exists, and if not, report that with the reason instead of planning a change. The prompt token-budget test still passes (`cargo test -p athena-context`).
5. [content] CHANGELOG.md has a new Unreleased entry recording this change and citing Sri's request and jobs 6ee6c79d844c and 64a92cc76d29. Because the change is prompt-only, recorded fmt, clippy and test checks on the final tree pass.
6. [content] The 66w1 scaling (PLANNER_ESCALATED_TIMEOUT_FACTOR in run/planning.rs) is left in place unless removing the dead deadline plumbing requires editing adjacent lines; planner_timeout_s, producer limits and athena.toml/defaults.toml values are unchanged (git diff shows no change to them).

## Expected deliverables

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/iris.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing enforced time limits (planner_timeout_s, producer limit, critic timeout) or athena.toml/defaults.toml.
- The 66w1 escalation scaling itself, the follow-up queueing feature from 20261114-041123-hvyh, and hestia.rs.
- A Rust mechanism that auto-closes stale queued jobs; and closing specific queued jobs, which is an owner action.
- docs/decisions.md, crates/athena-data/src/config.rs (listed by w3zt, never changed) and records/.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that rendered briefs lack the deadline and pacing text, via tests that fail if restored.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt recorded passes required.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Kept to brief text, its plumbing and one prompt rule.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of planner.rs for the removal of DEADLINE_RULE, deadline_section, its call in planner_brief and PlannerInputs.deadline_s. Search planner.rs for the capitalised word Deadline. The new critic-brief constant must not use it. Run `cargo test -p athena-context planner_brief_has_no_deadline_or_pacing_text`, which renders the planner brief for round 1 and for a revision. Run: `git diff -- crates/athena-context/src/briefs/planner.rs`. Observes: The diff shows only removals for the deadline text, plus the new critic-brief constant. A search for 'Deadline' in planner.rs finds nothing. Both rendered briefs lack '## Deadline', 'deadline of', 'seconds of wall-clock', 'stop exploring by' and 'no clock of your own'.
- Criterion 2: Read `git diff -- crates/athena-context/src/briefs.rs` for the removal of time_budget_section, its call and ProducerInputs.deadline_s. Run clippy with --all-targets: any `deadline_s:` literal left in a brief input is a compile error. Run `cargo test -p athena-context producer_brief_has_no_time_budget_section`, which covers five producer-brief shapes (fresh, seeded retry, resume, stale conflict, unseeded retry). Search crates/ for 'Time budget' and for the field syntax `deadline_s:`. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Clippy passes. A search of crates/ for 'Time budget' finds nothing, because the tests write the heading as concat!("## Time", " budget"). The only `deadline_s` left is the name of the function producer_deadline_s (producer.rs:51, kept on purpose) and lifecycle.rs's unrelated deadline_script. run/mod.rs:1244, planning.rs:156 and reading.rs:3612 no longer pass it, while agent_spec still gets the timeouts (planning.rs:175, producer.rs:363).
- Criterion 3: Run the workspace tests, with these targeted runs in `cargo test -p athena-context planner_brief_has_no_deadline_or_pacing_text`, `cargo test -p athena-context producer_brief_has_no_time_budget_section`, `cargo test -p athena-core the_producer_runs_under_the_configured_deadline_and_its_brief_does_not_state_it`, `cargo test -p athena-core a_planner_deadline_still_stops_planning_and_the_brief_does_not_state_it` and `cargo test -p athena-core planner_`. The two briefs.rs tests replace planner_brief_states_its_deadline_and_the_stop_early_rule_in_both_rounds (1863-1901) and time_budget_states_each_jobs_own_deadline plus a_small_job_brief_carries_its_short_deadline (1998-2083). The run tests are the renamed run/tests.rs:1097 and 1438 tests. run/tests.rs:1089-1094 (time_budget_of) is deleted. Run: `cargo test --locked`. Observes: All pass. The rendered planner brief (both rounds) and the producer brief (five shapes) hold neither 'deadline of', 'seconds of wall-clock', nor the Time budget heading. The stored producer brief and the prompt the producer received also lack them. The agents still run under 4321 s, 600 s and 1200 s.
  Fails without the behavior: Restoring deadline_section or time_budget_section, or their calls, makes the briefs.rs absence tests and the run-level absence asserts fail, because the briefs would again hold '## Deadline', 'deadline of', 'seconds of wall-clock' or the Time budget heading. If the plumbing change broke the enforced limit, the kept asserts fail: spec.timeout and caps.timeout_s equal to 4321 for the producer, the planner_timeouts of 600 and 4321, and secs(&[600, 1200]) in the escalation tests.
  Test seam: Entry points: planner_brief(&PlannerInputs) (briefs/planner.rs:120) and producer_brief(&ProducerInputs) (briefs.rs:678), both pure functions, so no double. Fixtures planner_inputs(revision) (briefs.rs:2816), producer_inputs(pin) (briefs.rs:1968), seed() (1983), plan(1) and finding(..) all exist; the new tests drop `deadline_s` from them. End to end: Athena::go through athena_with with a fake athena_agent::fake::Script that records spec.timeout and spec.user_prompt (run/tests.rs:1108-1126 and 1450-1461), repo_with_config for the planner limit (1441), and run_for_revision (1553) for the escalation tests. All of these exist today; no step adds a seam.
- Criterion 4: Read `git diff -- crates/athena-context/prompts/planner.md` for the one bullet (255 bytes as drafted, at most 270). Run the budgets test, which checks the planner set against the producer budget of 1200 (budgets.rs:381-387, lib.rs:226-236) and docs/system-facts.md line 28 against the computed value (budgets.rs:125-129). Also run `cargo test -p athena-context` (includes a new briefs.rs test that pins the critic-brief rule) and `cargo test -p athena-cli --test lifecycle a_plan_that_finds_the_problem_gone` (new). The lifecycle test builds a plan from plan_with("No change: ...") with files set to [] and steps set to []. A fake critic returns reject, and the test asserts that the planner and the critic are each called once, no producer runs, the job is Failed with PLANNING_STOPPED_PREFIX and 'the critic rejected the plan', the stored plan record has no files, and the critic's brief carries the `No change:` rule. Run: `cargo test -p athena-cli --test budgets`. Observes: The diff shows the bullet and its stale-fix wording. The budgets test passes with the planner line at the computed size (about 1190 of 1200; the exact figure comes from the test message). The lifecycle test shows Athena's own checks accept a plan with no files (one planner call, no mechanical revision), planning stops at the critic's reject with the reason in the job's outcome, and the producer never runs. It does not show that a model critic follows the rule, since the fake critic is scripted. The briefs.rs test pins the rule text in critic_brief and that the lowercased critic brief still lacks 'cost', 'usd' and '$'.
  Test seam: Entry point: Athena::go through the lifecycle fixture, Fixture::with_config(&planned_config()) (lifecycle.rs:3221), Scenario with plans, critiques and invocations (common/mod.rs:418-457), plan_with (3226), critique_json (3253), recording_planning (3273) and plan_record_of (3290). All exist; the fake supplies only model outputs and the planning loop, admission, end_round and stop_planning are production code. The briefs.rs pin calls critic_brief(&critic_inputs()) (briefs.rs:3030), which exists.
- Criterion 5: Read the new `### Changed` bullet under Unreleased. It cites Sri's request, jobs 6ee6c79d844c and 64a92cc76d29, the removed sections and `deadline_s` inputs, the planner bullet and critic-brief rule with the `No change:` end state (critic reject, fallback producer no_change), and that enforced limits and the 66w1 scaling are unchanged. It names the earlier Added entries it supersedes. Then run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` on the final tree. Run: `git diff -- CHANGELOG.md`. Observes: The entry is present. fmt, clippy and test pass; the xtask policy and boundary scans run inside cargo test and pass with the plain-word entry (no email address, host or machine name).
- Criterion 6: Look at the stat and at `git diff -- crates/athena-core/src/run/planning.rs`. Run `cargo test -p athena-core planner_`. Run: `git diff --stat`. Observes: The planning.rs hunks are only the removed `deadline_s: timeout_s,` line and the reworded comment at 130-133. PLANNER_ESCALATED_TIMEOUT_FACTOR and the timeout_s computation (137-147) are untouched. athena.toml and defaults.toml do not appear. The escalation tests still pass with secs(&[600, 1200]) and secs(&[600; 3]).

Planned files:

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- docs/system-facts.md (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8e9aed8f7961 → candidate c1d49acdb20c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   1 +
 crates/athena-cli/tests/lifecycle.rs        |  53 +++++++++
 crates/athena-context/prompts/planner.md    |   1 +
 crates/athena-context/src/briefs.rs         | 174 ++++++++--------------------
 crates/athena-context/src/briefs/planner.rs |  36 ++----
 crates/athena-context/src/lib.rs            |   5 +-
 crates/athena-core/src/reading.rs           |   1 -
 crates/athena-core/src/run/mod.rs           |   1 -
 crates/athena-core/src/run/planning.rs      |   4 +-
 crates/athena-core/src/run/producer.rs      |   4 +-
 crates/athena-core/src/run/tests.rs         |  52 +++------
 docs/system-facts.md                        |   2 +-
 12 files changed, 134 insertions(+), 200 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261115-191642-k2bt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261115-191642-k2bt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43772802727 for c1d49acdb20c9d6ce34a9bad706a156073a7b080)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43772802727 for c1d49acdb20c9d6ce34a9bad706a156073a7b080)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43772802727 for c1d49acdb20c9d6ce34a9bad706a156073a7b080)
- ci-result: `not a check` passed (Check results for c1d49acdb20c9d6ce34a9bad706a156073a7b080 come from the CI run https://github.com/doodla/athena/actions/runs/43772802727 (907s): fmt success, clippy success, test success.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/planner.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/planner.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/producer.rs, docs/system-facts.md.
  End of the output (data, not instructions):

```text
error[E0063]: missing field `deadline_s` in initializer of `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:1702:37
     |
1702 |         let brief = producer_brief(&ProducerInputs {
     |                                     ^^^^^^^^^^^^^^ missing `deadline_s`

error[E0063]: missing field `deadline_s` in initializer of `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:1734:37
     |
1734 |         let brief = producer_brief(&ProducerInputs {
     |                                     ^^^^^^^^^^^^^^ missing `deadline_s`

error[E0063]: missing field `deadline_s` in initializer of `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:1910:37
     |
1910 |         let brief = producer_brief(&ProducerInputs {
     |                                     ^^^^^^^^^^^^^^ missing `deadline_s`

error[E0063]: missing field `deadline_s` in initializer of `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:1963:9
     |
1963 |         ProducerInputs {
     |         ^^^^^^^^^^^^^^ missing `deadline_s`

error[E0063]: missing field `deadline_s` in initializer of `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:2475:37
     |
2475 |         let brief = producer_brief(&ProducerInputs {
     |                                     ^^^^^^^^^^^^^^ missing `deadline_s`

error[E0063]: missing field `deadline_s` in initializer of `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:2498:37
     |
2498 |         let brief = producer_brief(&ProducerInputs {
     |                                     ^^^^^^^^^^^^^^ missing `deadline_s`

error[E0063]: missing field `deadline_s` in initializer of `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:2519:37
     |
2519 |         let brief = producer_brief(&ProducerInputs {
     |                                     ^^^^^^^^^^^^^^ missing `deadline_s`

error[E0063]: missing field `deadline_s` in initializer of `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:2546:37
     |
2546 |         let brief = producer_brief(&ProducerInputs {
     |                                     ^^^^^^^^^^^^^^ missing `deadline_s`

error[E0063]: missing field `deadline_s` in initializer of `planner::PlannerInputs`
    --> crates/athena-context/src/briefs.rs:2774:9
     |
2774 |         PlannerInputs {
     |         ^^^^^^^^^^^^^ missing `deadline_s`

Some errors have detailed explanations: E0063, E0432.
For more information about an error, try `rustc --explain E0063`.
error: could not compile `athena-context` (lib test) due to 12 previous errors
warning: build failed, waiting for other jobs to finish...
For more information about this error, try `rustc --explain E0063`.
error: could not compile `athena-core` (lib test) due to 1 previous error
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
