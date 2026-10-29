# Job 20261028-082419-d9y7: verify "Make Athena's deterministic plan findings not count as planning rounds"

## Request (verbatim)

> Priority: high
> Mechanical plan findings use up planning rounds. On 2026-10-28 a hard job's planning spent round 1 on a missing fails_without field and round 2 on a grouped command line, both flagged by Athena's own plan rules with no critic call, so the critic judged the design only once, in round 3, and its two real findings stopped the job at the cap. Fix: findings from Athena's deterministic plan rules go back to the planner for an immediate revision that does not count as a planning round and does not call the critic, up to two such passes; only critic reviews count toward the round cap.
> Acceptance:
> - A test with a fake planner whose first plan breaks a deterministic plan rule shows the planner revises it, the critic then sees the fixed plan in round 1, and the job can still use all three critic rounds.
> - A test shows a plan that still breaks a rule after two mechanical passes stops with that finding.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, CHANGELOG.md

## Reading (accepted by the user)

Findings from Athena's own plan rules (plan_findings, no critic call) go back to the planner for an immediate revision that does not use a planning round, up to two such passes per round; only critic reviews count toward max_rounds. Audience: maintainers and the job pipeline, whose hard jobs lose critic rounds to mechanical findings.

Assumptions:
- Code survey: read crates/athena-core/src/run/planning.rs (plan_round loops rounds 1..=max_rounds; planning_round runs the planner, then plan_findings, and on a Blocker/Major finding calls end_round with a Revise critique and no critic call, consuming the round) and grep of crates/athena-core/src/run/mod.rs (existing planning tests, plan_verdict at ~210, plan_round call at ~892). The loop change lands in planning.rs though the request names only mod.rs; tests go in mod.rs.
- Plan records and critiques are keyed by job and round; the mechanical pass must not collide with the critic's round numbering (e.g. mechanical plans are not filed as records, or are filed without a critique, but the final stop must still record the finding). The producer must check how closing.plans/critiques are validated and persisted; records validation allow-list is unchanged because no new record or data file is added.
- The two-pass cap applies per critic round (each critic round gets up to two mechanical passes); a plan still breaking a rule after two passes stops planning with that finding. If this should be a job-wide cap instead, that is a small change.
- Mechanical passes need planner budget; the existing check_budget before each planner call applies, and a budget stop ends planning as today.
- No new refusal reading rule is added to readings; this changes planning flow, so no refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows) are not changed; max_rounds config semantics stay the same.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/mod.rs uses a fake planner whose first plan breaks a deterministic plan rule (e.g. missing fails_without); run `cargo test -p athena-core`: the planner is invoked again with the finding, the critic is first called on the fixed plan labelled round 1, and no critic call precedes it.
2. [behavior] A test with max_rounds = 3 shows that after one mechanical revision the critic can still review in rounds 1, 2 and 3 (three critic calls) before the cap stops the job; run `cargo test -p athena-core`.
3. [behavior] A test shows a plan that still breaks a rule after two mechanical passes stops planning (job failed with the planning-stopped prefix) with that finding in the reason, and the critic is never called; run `cargo test -p athena-core`.
4. [content] The planning loop in crates/athena-core/src/run/planning.rs no longer ends a round on Blocker/Major plan_findings by consuming a round; the cap of two mechanical passes is a named constant or documented, and the plan_round doc comment states that only critic reviews count toward max_rounds.
5. [content] CHANGELOG.md has an entry describing that deterministic plan-rule findings are revised without using a planning round or a critic call, at most twice.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [behavior] Existing planning tests (soundness findings, revise in last round, resume with seed revision) still pass unchanged in behavior; run `cargo test -p athena-core`.

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Changing max_rounds defaults or [job_plan] values in athena.toml
- Changing what the plan rules or the critic flag
- Changing the planner or critic prompts

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the planner re-invocation and critic round numbering by test, not wiring.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the round accounting rather than raising max_rounds.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the planning loop.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The two-pass cap is enforced in code and tested.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in the produce_and_verify_order_tests module of crates/athena-core/src/run/mod.rs. It drives Athena::go, the production entry point, over a repo built by repo_with_planning(max_rounds = 3). The seam is the fake agent driver (FakeAgentDriver::new(script)), which exists today and is injected through athena_with. The script wraps base_script. It logs each role and user_prompt. The planner returns a bad plan first (README.md with action create although it exists, which breaks the real plan rule 'creates a file that exists') and a good plan after. The critic returns accept. Step 4 adds repo_with_planning, the planner and critic turns and the logging wrapper, all in the test module. The test does not stub plan_findings or the brief. Guards: [job_plan] enabled with mode all, and a capability that does not pin the repo (base_script's reader returns 'study'). Run: `cargo test -p athena-core a_mechanical_finding_is_revised_before_the_critic_and_the_critic_sees_round_1`. Observes: The role log is [Reader, Planner, Planner, Critic, Producer, Verifier]. The second planner prompt contains 'already exists in the base'. The first critic prompt contains '## Plan under review (round 1)'. The job state is Landed.
  Fails without the behavior: Without the new loop, the first plan's finding ends round 1 with no critic call. The critic's first prompt would say 'Plan under review (round 2)', so the round-1 assertion and the Reader, Planner, Planner, Critic order assertion fail.
- Criterion 2: New test in the same module, with the same entry point, seam and wrapper, using repo_with_planning(max_rounds = 3). The planner returns one bad plan, then good plans. The critic returns revise with a major finding every time. Run: `cargo test -p athena-core after_a_mechanical_revision_the_critic_still_reviews_in_three_rounds`. Observes: The role log is [Reader, Planner, Planner, Critic, Planner, Critic, Planner, Critic] with no Producer. The critic prompts name rounds 1, 2 and 3 in order. The job is Failed, and the reason starts with PLANNING_STOPPED_PREFIX and contains PLANNING_CAP_REASON and 'round 3 of 3'.
  Fails without the behavior: Under the old accounting the mechanical finding takes round 1, only two critic calls occur before the cap stop, and the reason says 'round 2 of 3'. The three-critic-call assertion fails.
- Criterion 3: New test in the same module, with the same entry point, seam and wrapper, using repo_with_planning(max_rounds = 5). The planner always returns the bad plan. The critic arm of the script panics, so any critic call fails the test. Run: `cargo test -p athena-core a_plan_that_still_breaks_a_rule_after_two_mechanical_passes_stops_without_the_critic`. Observes: The planner is called exactly three times (first plan plus two mechanical passes) and the critic never. The job is Failed. The reason starts with PLANNING_STOPPED_PREFIX and contains 'already exists in the base' and 'after 2 mechanical revisions', and does not contain PLANNING_CAP_REASON. No Producer or Verifier ran.
  Fails without the behavior: Without the pass cap the planner is called again each time and planning never stops on the mechanical finding. Under the old accounting max_rounds = 5 gives five planner calls and a cap-style reason. The exact planner-call count, the zero critic calls and the reason assertions fail.
- Criterion 4: Read the diff of planning.rs. Four things must be present: (1) the plan_findings branch no longer calls end_round with last_round; (2) a pub(super) const MAX_MECHANICAL_PASSES: u32 = 2 with a doc comment; (3) plan_round's doc comment says only critic reviews count toward max_rounds and describes the mechanical passes; (4) RoundEnd::Mechanical exists and plan_round's loop handles it without advancing the round. Criteria 1 to 3 show the behavior; this checks content. Run: `git diff crates/athena-core/src/run/planning.rs`. Observes: The four items above are present in the diff.
- Criterion 5: Read the CHANGELOG.md diff. Run: `git diff CHANGELOG.md`. Observes: The [Unreleased] section has an entry saying findings from Athena's own plan rules go back to the planner without using a planning round or calling the critic, at most two passes per round, and that only critic reviews count toward max_rounds. It names the 2026-10-28 cause: a missing fails_without and a grouped command line took rounds 1 and 2.
- Criterion 6: Run the three checks Athena runs on the result, in order, from the repository root. The test run includes the three new tests and the adjusted CLI job_plan tests; clippy covers the new Attempt struct and RoundEnd::Mechanical variant (unused fields or too many arguments would fail it); fmt covers formatting. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: If the loop change were made without adjusting the five CLI tests in job_plan.rs, cargo test --locked would fail on them, because critique-1 would be the critic's and not Athena-only. If the Attempt struct left an unused field or planning_round kept too many arguments, clippy -D warnings would fail. If the new code were unformatted, fmt --check would fail.
- Criterion 7: Run the existing planning tests: the critic-soundness tests in mod.rs, the revise, reject and last-round cases, the resume-seed tests, and the CLI job_plan and lifecycle tests. Only the five CLI tests that read an Athena-only critique-1 are adjusted, and they assert on the planner's revision brief and the critic call count instead, which covers the same behavior. Run: `cargo test -p athena-core && cargo test -p athena-cli --test job_plan --test lifecycle`. Observes: All pass. Critic-reviewed paths keep their record numbering (job-plan-N and critique-N for N = 1 to 3), the cap reason and the resume seed.
  Fails without the behavior: A regression in end_round, in the resume seed or in max_rounds accounting would break these tests, including the critic-soundness tests in mod.rs, the 'revise in the last round' case and a_retry_of_a_job_stopped_in_planning_carries_the_last_critiques_findings.

Planned files:

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 459389ba4514 → candidate e36c7a8f1acd. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   9 ++
 crates/athena-cli/tests/job_plan.rs    | 137 +++++++++---------
 crates/athena-core/src/run/mod.rs      | 252 ++++++++++++++++++++++++++++++++-
 crates/athena-core/src/run/planning.rs | 119 ++++++++++++----
 docs/design/job-planning.md            |  16 ++-
 5 files changed, 436 insertions(+), 97 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1661 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
