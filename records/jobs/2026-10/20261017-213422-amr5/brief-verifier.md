# Job 20261017-213422-amr5: verify "Add planner and critic prompts, schemas and briefs (job 2)"

## Request (verbatim)

> Build job 2 of docs/design/job-planning.md (revision 2, section 8): the planner and critic prompts, their output schemas and the briefs, as pure functions with no change in behavior. Read the design (sections 3 and 4) and its review, docs/design/reviews/job-planning-review-1.md (findings F5 and F6 apply here). Job 1 added the records, roles and config.
> Plan: (1) Add crates/athena-context/prompts/planner.md and critic.md, short and stable, and register them in crates/athena-context/src/lib.rs as the other prompts are. (2) In crates/athena-context/src/outputs.rs add the planner's output schema (approach; decisions with choice, reason and rejected options; files with action and purpose; ordered steps; risks with mitigations; one demonstration per criterion with method, command or test, what it observes, and fails_without; open questions) and the critic's (verdict accept, revise or reject; findings with severity, location, evidence and fix; whether the reading misread the request); both forbid extra properties at every level. (3) In crates/athena-context/src/briefs.rs add planner_brief (numbers each criterion with its kind, flags a listed athena.toml or defaults.toml as a governance file, lists the commands the verifier may run; a revision brief quotes the previous plan and every finding), critic_brief (carries the plan's demonstrations and Athena's own findings, and nothing about the planner's cost or commands), and render_plan_section, which renders the accepted plan as the producer's "Job plan (accepted, round r)" section deterministically. (4) verifier_brief gains an optional plan input that adds only the critic-accepted demonstrations and the planned file list, never the approach, reasons or rejected options (F5); without a plan its output is byte-identical to today's.
> Acceptance:
> - cargo test: planner_brief numbers every criterion with its kind, marks a listed athena.toml as governance, and lists the verifier's commands; a revision brief contains the previous plan and each finding's text.
> - cargo test: critic_brief contains every demonstration and each Athena finding, and contains no planner cost or command list (assert on absence).
> - cargo test: both output schemas reject an extra property at the top level and inside a nested item, and the critic's verdict accepts exactly accept, revise and reject.
> - cargo test: render_plan_section gives the same bytes for the same record twice, and verifier_brief with a plan holds the demonstrations and planned files but not the plan's approach or reasons; without a plan it equals a stored copy of today's output.
> - The prompt budget tests in crates/athena-cli/tests/budgets.rs pass; cargo clippy --all-targets -- -D warnings, cargo fmt --check and xtask's boundary test pass; CHANGELOG.md has an entry.
> Out of scope: calling the planner or critic, run.rs, status display, and athena.toml.

## Reading (accepted by the user)

Implements job 2 of the job-planning design as pure, unused functions in athena-context. It adds the planner and critic prompts, their strict output schemas, and the planner, critic and plan-section briefs. It also gives verifier_brief an optional plan input that shows only demonstrations and planned files. Audience: the maintainer and the verifier checking the cargo tests.

Assumptions:
- Plan record types (JobPlan, PlanDecision, PlanFile, PlanRisk, Demonstration) from job 1 live in crates/athena-data/src/records.rs and are reused rather than redefined.
- Output schemas are generated from typed structs, as the existing outputs are, with unknown fields denied at every level.
- The 'stored copy of today's output' is a fixture or inline string in a briefs.rs test, captured before the change.
- Existing verifier_brief callers get a None plan input. cap-change or cap-study change only if the signature requires it.
- budgets.rs changes only if the new frames need budget entries.

## Acceptance criteria

1. [behavior] Run cargo test: planner_brief numbers every criterion with its kind, marks a listed athena.toml (and defaults.toml) as a governance file, and lists the commands the verifier may run. A revision brief contains the previous plan and the text of each finding.
2. [behavior] Run cargo test: critic_brief contains every demonstration and each Athena finding. Assertions on absence show it has no planner cost and no planner command list.
3. [behavior] Run cargo test: the planner and critic output schemas each reject an extra property at the top level and inside a nested item (additionalProperties false at every level). The critic verdict accepts exactly accept, revise and reject.
4. [behavior] Run cargo test: render_plan_section returns identical bytes for the same record called twice, under the heading 'Job plan (accepted, round r)'. verifier_brief with a plan contains the demonstrations and the planned file list but not the plan's approach, decision reasons or rejected options. With no plan its output equals a stored copy of today's output byte for byte.
5. [behavior] Run cargo test (including crates/athena-cli/tests/budgets.rs), cargo clippy --all-targets -- -D warnings, and cargo fmt --check. xtask's boundary test is among the tests. All pass with no change to existing behavior.
6. [content] prompts/planner.md and prompts/critic.md exist, are short, and are registered in crates/athena-context/src/lib.rs as PromptAssets (id, version, include_str!) like the existing frames.
7. [content] CHANGELOG.md has an entry for this change. Nothing calls the planner or critic, and run.rs, status display and athena.toml are unchanged.

## Expected deliverables

- crates/athena-context/prompts/planner.md (create)
- crates/athena-context/prompts/critic.md (create)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)

## Out of scope

- Calling the planner or critic
- run.rs
- status display
- athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover each new function and schema; clippy and fmt are required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the contents and absences of rendered briefs and schema rejections, not that items exist.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change confined to athena-context plus CHANGELOG.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The F5 restriction (no approach or reasons in the verifier brief) is enforced by absence tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base d94497ae84e9 → candidate dd7da256245f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   9 +
 crates/athena-cli/tests/budgets.rs       |  13 +-
 crates/athena-context/prompts/critic.md  |  12 +
 crates/athena-context/prompts/planner.md |  14 +
 crates/athena-context/src/briefs.rs      | 656 ++++++++++++++++++++++++++++++-
 crates/athena-context/src/lib.rs         |  25 ++
 crates/athena-context/src/outputs.rs     | 222 +++++++++++
 7 files changed, 948 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (900 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
