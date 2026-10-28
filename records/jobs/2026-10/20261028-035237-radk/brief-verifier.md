# Job 20261028-035237-radk: verify "Add a shared soundness question to the critic and verifier briefs"

## Request (verbatim)

> Priority: urgent
> Judge soundness, not only criteria, in both the critic and the verifier (Sri, 2026-10-28). Today the critic checks a plan against the reading and the code, and the verifier checks a candidate against its criteria, so a crude design can pass both and land (2026-10-27: a refusal rule that matched common words in requests was verified and landed, then refused almost every request). Add one soundness question to the critic's review of a plan and to every change verification: would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives? Each must name every such problem with the simpler or more correct alternative. In the critic, a blocker-level soundness finding makes the verdict revise; in the verifier, it fails the verdict like an unmet criterion. Keep it to the approach, not style. Share the question's wording between the two prompts so they cannot drift.
> Acceptance:
> - The critic and verifier briefs both carry the same soundness question and stay within their size limits; the size test passes.
> - A test with a fake critic output carrying a blocker soundness finding shows the plan verdict is revise, and a test with a fake verifier output carrying one shows the verdict fails even with all criteria met; minor findings change neither.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-context/prompts/critic.md, crates/athena-context/prompts/verifier.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/verify.rs, crates/athena-core/src/run/mod.rs, CHANGELOG.md

## Reading (accepted by the user)

Define one soundness-question string in briefs.rs and put it in both the critic brief and every change-verifier brief. It asks whether a careful senior engineer would reject the approach as a crude heuristic, an unneeded special case, a guess where code or data could be read, or a rule with obvious false positives, and it names the better alternative. Tests with fake critic and verifier outputs show that a blocker soundness finding makes the plan verdict revise and fails the verifier verdict even when all criteria are met, and that minor findings change neither. Audience: the Athena maintainer.

Assumptions:
- Existing rules already make blocker or major findings block a verifier pass (verify.rs) and force revise from the critic (run/mod.rs), so the work is the shared question text plus tests that pin this behaviour for soundness findings.
- The question text lives as one shared const in briefs.rs and is rendered into both briefs, so the verifier prompt file (2 tokens of headroom) grows little or not at all; critic.md has 517 tokens of room.
- Soundness findings are ordinary findings with a blocker, major, minor or nit severity; no new severity or output schema field is added.
- The soundness check covers the approach only, not style, and the question is added only to change verification (the study verifier is untouched).
- The CHANGELOG entry follows the existing format.

## Acceptance criteria

1. [content] briefs.rs defines the soundness question once, and both the critic brief and the change-verifier brief render that same constant. It asks about a crude heuristic, a special case that should be general, a guess where code or data could be read, and a rule with obvious false positives. It requires naming every such problem with the simpler or more correct alternative, and it is limited to approach, not style.
2. [behavior] Run `cargo test`: the brief tests assert the critic brief and the change-verifier brief both contain the identical soundness question text, and the existing prompt size/token-limit test passes.
3. [behavior] Run `cargo test`: a test with a fake critic output carrying a blocker soundness finding yields plan verdict revise, and one with only a minor soundness finding does not change the verdict.
4. [behavior] Run `cargo test`: a test with a fake verifier output carrying a blocker soundness finding, with all criteria met, yields a failed verdict, and a minor soundness finding leaves a passing verdict passing.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both are clean. `cargo test` passes for the whole workspace.
6. [content] CHANGELOG.md has a new entry describing the soundness question in the critic and verifier. No governance files (athena.toml, defaults.toml) change.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the study capability's verifier
- Adding a new severity level or output schema field
- Changing token budgets in athena.toml
- Style or lint review rules

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add fake-output tests for critic and verifier; require fmt, clippy -D warnings and cargo test to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the verdict outcomes from fake outputs, not that the text exists.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The shared const makes the wording drift-proof, and the existing blocker/major gates enforce the verdict.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared question covers both gates instead of separate wording in each.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a crude-but-criteria-meeting design is no longer passed.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change over the six named files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the brief unit tests and read the diff. `SOUNDNESS_QUESTION` is defined once in briefs.rs and only `soundness_section` renders it. Test `soundness_question_is_one_const_with_the_four_problems_and_the_alternative` asserts the const contains each of: crude heuristic, special case, guess, false positives, the simpler-or-more-correct alternative requirement, and the approach-not-style limit. Entry points: the const, `critic_brief(&critic_inputs())`, `verifier_brief(&inputs, true)`. Seams: `critic_inputs()` exists today; the `soundness` field is added in step 2. Run: `cargo test --locked -p athena-context soundness`. Observes: The soundness tests pass: the const holds all four problem phrases, the alternative requirement and the style limit, and each brief contains the const text exactly once. critic.md and verifier.md carry no copy.
  Fails without the behavior: The test fails if the const lacks any of the four problem phrases, the alternative requirement or the style limit, or if a brief renders the text zero times or twice.
- Criterion 2: Unit tests in briefs.rs call the production builders `critic_brief(&critic_inputs())` and `verifier_brief(&inputs, true)` with `soundness: true`. They assert each brief contains `SOUNDNESS_QUESTION` exactly once and that the text extracted from the `## Soundness` section of each, up to the role sentence, is equal. A second test asserts the study case (`soundness: false`) lacks it and that `VERIFIER_BRIEF_WITHOUT_A_PLAN` still matches. The existing budget tests run in the same command. Seams: `critic_inputs()` and `plain_verifier_inputs()` exist today; the `soundness` field is added by step 2. Run: `cargo test --locked`. Observes: The brief tests pass. `every_role_set_is_within_its_budget` and `system_facts_budgets_match_computed` in athena-cli/tests/budgets.rs pass because the prompt files are unchanged. A bound assertion keeps the const at most 700 bytes.
  Fails without the behavior: `soundness_is_in_both_briefs_with_identical_text` fails if either builder omits the section or the two texts differ. The study test fails if the field is ignored.
- Criterion 3: Unit tests in run/mod.rs build a `CriticOutput` with `serde_json::from_value`: verdict accept plus one finding with severity blocker and summary 'Soundness: matches common words, false positives'. They pass it through `critic_findings` and `plan_verdict`, the same functions the production round calls, and assert `CritiqueVerdict::Revise`. A second case has the same finding at minor and expects Accept. A third has a nit and expects Accept. Seam: the two functions are extracted in step 4; no agent or repository is needed. Run: `cargo test --locked -p athena-core soundness_finding`. Observes: The blocker case yields revise (agent verdict accept overridden). The minor and nit cases leave accept.
  Fails without the behavior: If the blocker-or-major override in `plan_verdict` is removed or the severity mapping in `critic_findings` is wrong, the blocker case returns accept and the test fails.
- Criterion 4: Unit tests in verify.rs call `judge_with` (existing helper wrapping `judge`) with `output(vec!["cargo test"], vec![finding])`, `ran = ["cargo test"]` and `check(true)`, so all criteria are met and the command is seen. Case A: a `FindingOutput` with severity blocker and a soundness summary gives `final_verdict == Verdict::Fail`, with the '1 blocker or major finding(s)' reason and zero unmet criteria. Case B: the same finding at minor leaves `Verdict::Pass`. The helpers `output`, `check` and `judge_with` exist today (verify.rs about lines 1224-1260). Run: `cargo test --locked -p athena-core soundness_finding`. Observes: The blocker soundness finding with all criteria met gives fail. The minor one leaves pass.
  Fails without the behavior: If `judge` ignored blocker findings when every criterion is met, case A would return pass and the test would fail.
- Criterion 5: Run the three Athena checks over the whole workspace, each as its own command. Entry points are the cargo tools on the changed crates. The tests cover every changed code path: brief builders, `VerifierInputs` constructors, extracted critic functions, `judge`, plus the CLI integration tests that read the verifier brief and run planning. The three commands are listed below; run each. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: fmt reports no diff, clippy reports no warnings, and the workspace test suite passes, including the new soundness tests, the stored verifier brief copy test, the budget tests and the CLI planning and lifecycle tests.
  Fails without the behavior: A missed `soundness` initializer in any `VerifierInputs` constructor fails compilation in clippy and test. Unformatted new code fails fmt. An unused or needlessly cloned helper in the extraction fails clippy under -D warnings. A changed prompt file or a brief edit that breaks the stored copy, the critic cost/usd/$ absence scan or the pinned budget lines fails cargo test. An accidental behavior change in the extracted critic code fails the job_plan.rs integration tests.
- Criterion 6: Read the diff. CHANGELOG.md gets one new bullet under `## [Unreleased]` / `### Added` in the existing prose style. It says the critic's brief and every change-verification brief carry one shared soundness question, a blocker finding makes the critic revise and fails the verifier, and the study verifier is unchanged. The diff lists no athena.toml, defaults.toml, workflow or lint file. Run: `git diff --stat`. Observes: Changed files are only briefs.rs, run/mod.rs, verify.rs and CHANGELOG.md, with the CHANGELOG hunk under Unreleased. No governance file appears.
  Fails without the behavior: Without the entry the diff stat lacks CHANGELOG.md; with a governance file touched, athena.toml or defaults.toml would show in the stat.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cf03abb96e40 → candidate d4b0787a4596. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   9 +++
 crates/athena-context/src/briefs.rs | 104 ++++++++++++++++++++++++++++-
 crates/athena-core/src/run/mod.rs   | 128 +++++++++++++++++++++++++++++-------
 crates/athena-core/src/verify.rs    |  50 ++++++++++++++
 4 files changed, 267 insertions(+), 24 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1569 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both are clean. `cargo test` passes for the whole workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
