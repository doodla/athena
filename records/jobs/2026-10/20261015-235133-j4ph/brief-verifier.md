# Job 20261015-235133-j4ph: verify "Make the drain source check judge model behavior against pinned code"

## Request (verbatim)

> For the fourth time Athena's own outcome check scored the explainer a point above an independent reader (4 against 3). The independent reader compares the study's model with what the pinned code does and found behavior errors: a reopened backlog item shown as finished although the source returns it to Queued, the counter missing that a failed close already freed the place, and a red-PR rule that contradicts a waiver on the same page. Athena's source check only confirms that quotes exist. The check that ends drain must judge whether the model's behavior matches the pinned code.

## Reading (accepted by the user)

The drain's page source check (`check_study_text` in pin.rs, called from drain.rs) only verifies per-statement claims and quoted text. I read the request as: extend it so the check that ends drain also compares the study's model (states, transitions, counters, rules, waivers) with what the pinned code does, and fails on behavior errors and on contradictions within the page. The audience is the maintainer, and the independent reader who rates the page.

Assumptions:
- Goal: the check that gates 'target met' in drain must catch behavior errors in the study model (wrong end state after reopen, counter not freed by a failed close, rule contradicting a waiver), not only quote existence. The audience is the maintainer and the independent reader who rates the page.
- Chosen approach: keep one model call per study, but rewrite SOURCE_CHECK_PROMPT and source_check_brief in crates/athena-core/src/pin.rs to require tracing each model.json state, transition, counter and rule through the pinned code, and cross-checking rules on the page against each other. The prompt must also say what counts as a behavior error. Add a problem kind such as 'behavior' or 'inconsistent' to the checker schema and to its handling.
- The existing model-free floor (model_contradictions in drain.rs) stays. It may gain a check for the red-PR/waiver kind of self-contradiction only if that can be done mechanically.
- Tests use the fake agent, so a new unit/integration test shows a behavior-kind problem fails check_page and the drain does not report target met. No live model is run.
- Prompt-version bump (Component version) and a decisions.md entry and CHANGELOG line are expected for the changed behavior; athena.toml and defaults.toml are unchanged.
- Higher cost per check is possible because the checker reads more; the existing verifier cap and the budget reserve still apply.

## Acceptance criteria

1. [content] SOURCE_CHECK_PROMPT and source_check_brief in crates/athena-core/src/pin.rs instruct the checker to trace the study's model (states and where transitions end, counters and every event that changes them, rules and their waivers) through the pinned code, and to report a model that behaves differently from the code or a page that contradicts itself. The report schema has a problem kind for this, and it is distinct from 'contradicted' and 'omitted'.
2. [content] The drain.rs module docs, the CHANGELOG and docs/decisions.md describe that the source check now judges model behavior against the pinned code, and why. The reason cited is the repeated 4-vs-3 score gap and the three named errors: reopen shown as finished, the counter freed by a failed close, and a red-PR rule contradicting a waiver.
3. [behavior] Run `cargo test`. A new test using the fake agent returns a behavior-kind problem from the checker, and shows check_study_text reports it as a problem. A drain test shows that the run does not report 'target met' and that Report::held_back names the statement. The test fails if the new kind is ignored.
4. [behavior] Run `cargo test`. A test shows the checker's report is still rejected when it omits model.json from files_checked, so the model file cannot be skipped.
5. [behavior] Run `cargo clippy` and `cargo fmt --check`. Both are clean, and the existing drain and pin tests still pass.

## Expected deliverables

- crates/athena-core/src/pin.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/fake.rs (modify)
- README.md (modify)

## Out of scope

- Correcting the studies' content (the reopened item, the counter, the red-PR rule) in studies/.
- Changing the audience rater or its target rating.
- Changing athena.toml budgets or defaults.toml.
- Running a live model check against the real pinned repository.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the added problem kind and the drain refusing 'target met' on a behavior problem; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that a behavior error in a model fails the check and holds back 'target met', not merely that a prompt string exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the check that ends drain, not in correcting the three listed errors in one study.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments, CHANGELOG and decisions entry are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the source check and its prompt, schema and tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria target the outcome, which is that the gate rejects a model whose behavior differs from the pinned code.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 5c082cf7e4cc → candidate a0d4d7e042bc. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |  6 +++
 README.md                        |  3 +-
 crates/athena-cli/src/drain.rs   | 10 ++++-
 crates/athena-cli/tests/drain.rs | 97 ++++++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/pin.rs    | 22 +++++++--
 docs/decisions.md                | 13 ++++++
 6 files changed, 144 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (715 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
