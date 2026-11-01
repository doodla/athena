# Job 20261030-060822-ztj9: verify "Require plans to name and wait for external-service state"

## Request (verbatim)

> Priority: normal
> 
> Add a planning-stage convention: when a plan calls an external service (GitHub API, CI runs, deploys), it must name the state that call requires (for example, that a run has completed) and say how the plan checks or waits for that state before calling. The critic should reject a plan that leaves this out in its first round. Add a test or a prompt fixture that covers this. Failure class 20261018-100239-xc20 is still 'proposed' and has three introspections, the latest from 2026-10-26.
> 
> Introspections that record the repeated cause: 20261018-100226-1mfq, 20261026-002830-390w, 20261026-091638-kqx6
> 
> Acceptance:
> - The change is made as described: Planning designs external-service calls without checking the state they need, and the critic still finds the gap after the last round
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261018-100239-xc20.json

## Reading (accepted by the user)

Add a planner-prompt convention that any plan step calling an external service (GitHub API, CI, deploys) names the state it needs and how the plan checks or waits for it. Add a critic-prompt rule to reject such a plan in round one when this is missing. A prompt test in athena-context covers both. Audience: Athena's planner and critic agents, and the maintainers who judge plans.

Assumptions:
- Code survey: I read crates/athena-context/src/lib.rs (PLANNER_FRAME and CRITIC_FRAME, the planner_and_critic_frames_are_short_and_fit_the_budgets test at about line 394, and the reader/producer guidance-text test pattern at about line 437) and crates/athena-context/prompts/planner.md and critic.md. The entry point is compose() plus check_budget(); the prompt text is static, so the property is measured by asserting the frame text directly. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not involved.
- The planner and critic prompt files have room: planner 494 tokens left, critic 517 left. Each frame must stay under 500 estimated tokens (existing test). Added text must be short, about 60 tokens each. The contract and the other sets are untouched.
- The named record records/failure-classes/2026-10/20261018-100239-xc20.json is read-only owner data and does not change. It is the evidence for the request. Athena handles its status.
- The prompt fixture is a unit test in crates/athena-context/src/lib.rs, in the style of reader_checks_for_governance_files_and_producer_stops_on_unlisted_ones. It asserts that the planner frame requires naming the needed state and the wait or check, and that the critic frame says to reject in the first round. No LLM is run.
- No governance files (athena.toml, defaults.toml, workflows) need to change.

## Acceptance criteria

1. [content] crates/athena-context/prompts/planner.md has a bullet saying that any step calling an external service (GitHub API, CI runs, deploys) must name the state the call requires (e.g. a run has completed) and say how the plan checks or waits for it before the call.
2. [content] crates/athena-context/prompts/critic.md has a bullet saying to raise a major finding, in the first round, against a plan whose external-service call omits the required state or how it checks or waits for it.
3. [behavior] A new test in crates/athena-context/src/lib.rs fails if either bullet is removed from its frame. It asserts planner and critic frame text contain the external-service state/wait wording and the critic's first-round rejection wording. Run `cargo test -p athena-context`.
4. [behavior] The existing budget test planner_and_critic_frames_are_short_and_fit_the_budgets and all other tests pass, so the prompts stay within token budgets. Run `cargo test`.
5. [behavior] No unrelated behavior changes: `git diff` touches only the two prompt files and the athena-context test module. The reader, producer, verifier and contract prompts are unchanged. `cargo fmt --check` and `cargo clippy --all-targets` are clean.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing records/ files, including the failure class record and its introspections
- Changing the reader, producer, verifier or contract prompts
- Changing athena.toml, defaults.toml, workflows or token budgets
- Running a live planner or critic model to test the convention
- Enforcing the rule in code beyond the prompt text and its test

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A new unit test covers the convention; cargo test, clippy and fmt must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Two short prompt bullets and one test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test asserts the actual frame wording, and the verifier runs cargo test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The test guards the prompt wording. Whether the critic follows it is prose-only guidance.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The prompt wording and the test are plain, with examples.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The goal is that plans stop calling services without their needed state and the critic catches it in round one. The test checks the wording that causes this.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Inspect the planner.md diff, then run the new test, which reads PLANNER_FRAME.text (an include_str of this file). Run: `cargo test -p athena-context external_service_calls_need_a_named_state_and_a_wait`. Observes: The planner.md diff adds a bullet naming an external service (GitHub API, CI run, deploy), the state the call needs (e.g. run completed), and how the plan checks or waits for it before the call. The test passes on the planner phrases.
- Criterion 2: Inspect the critic.md diff, then run the new test, which reads CRITIC_FRAME.text. Run: `cargo test -p athena-context external_service_calls_need_a_named_state_and_a_wait`. Observes: The critic.md diff adds a bullet saying to raise a major finding in round one when an external-service call omits the needed state or how the plan checks or waits for it. The test passes on the critic phrases.
- Criterion 3: A new unit test in the athena-context test module reads the public constants PLANNER_FRAME.text and CRITIC_FRAME.text. It loops over the required phrases and asserts each with a message naming the missing one ('planner frame lacks: ...', 'critic frame lacks: ...'). Both frames are static include_str text, so no precondition or guard applies. The producer also removes a bullet temporarily, sees the failure, and restores it. Run: `cargo test -p athena-context external_service_calls_need_a_named_state_and_a_wait`. Observes: The test passes with both bullets. With the planner bullet removed it fails with 'planner frame lacks: ...'. With the critic bullet removed, or 'in round one' dropped from it, it fails with 'critic frame lacks: ...'.
  Fails without the behavior: Removing the planner bullet, the critic bullet, or the words 'in round one' or 'major finding' from the critic bullet makes the matching assertion fail.
  Test seam: Production entry points: PLANNER_FRAME.text and CRITIC_FRAME.text in crates/athena-context/src/lib.rs. They exist today (static include_str text, read in lib.rs). The fixture is the frame text itself, with no double. Plan step 4 adds the test.
- Criterion 4: Run the whole workspace suite. It includes planner_and_critic_frames_are_short_and_fit_the_budgets (each frame under 500 estimated tokens and inside its budget set), planning_prompts_and_brief_require_a_named_test_seam, planner_prompt_and_brief_carry_the_refusal_corpus_rule, and system_facts_budgets_match_computed in crates/athena-cli/tests/budgets.rs. Run: `cargo test --locked`. Observes: All tests pass. Both frames stay under 500 estimated tokens. The planner and critic lines in docs/system-facts.md equal the computed sizes. The seam and refusal_corpus wording that existing tests assert survives the trim.
  Fails without the behavior: Without the trims, the frame-size test fails. Without the docs/system-facts.md update, system_facts_budgets_match_computed fails. If the trim removed asserted wording, the seam or refusal_corpus tests fail.
  Test seam: Existing tests that call compose() and check_budget() in crates/athena-context/src/lib.rs and computed_rows() in crates/athena-cli/tests/budgets.rs. All exist today (read in the code). No step adds a seam.
- Criterion 5: Run the budgets test, then check the diff and the lint commands. The budgets test computes the size of every prompt set from the real prompt files via computed_rows() and compares it to docs/system-facts.md. It passes only if the contract, reader, producer and verifier lines are unchanged and only the planner and critic lines moved, so a stray edit to another prompt would fail it. Then run `git diff --stat`, `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Run: `cargo test --locked -p athena-cli --test budgets`. Observes: The budgets test passes with the contract, reader, producer and verifier lines at their base values (326, 1997, 1257, 1298 in the base snapshot). `git diff --stat` lists only planner.md, critic.md, the athena-context src/lib.rs test module, docs/system-facts.md (two numbers) and CHANGELOG.md. fmt and clippy report nothing. No records/, athena.toml, defaults.toml or workflow file appears in the diff.
  Fails without the behavior: If any reader, producer, verifier or contract prompt were edited, its size would differ from its unchanged line in docs/system-facts.md and system_facts_budgets_match_computed would fail. An unformatted or lint-failing change fails fmt or clippy.
  Test seam: Existing: computed_rows() in crates/athena-cli/tests/budgets.rs reads the real prompt files and docs/system-facts.md (it exists, read in the code). The fixture is the repository itself, with no double and nothing injected. No step adds it.

Planned files:

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/lib.rs (modify)
- docs/system-facts.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d5c447df3532 → candidate dbcd34ff3771. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             | 10 ++++++++++
 crates/athena-context/prompts/critic.md  |  4 ++--
 crates/athena-context/prompts/planner.md |  6 +++---
 crates/athena-context/src/lib.rs         | 26 ++++++++++++++++++++++++++
 docs/system-facts.md                     |  4 ++--
 5 files changed, 43 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1884 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
