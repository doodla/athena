# Job 20261024-074924-919r: verify "Tell verifier to put cited Athena checks in the commands list"

## Request (verbatim)

> In crates/athena-cap-change/prompts/verifier.md, the instruction to cite the fmt, clippy and test results under 'Checks Athena ran on this tree' does not say where: job 20261024-033935-c4w4's verifier quoted them in its evidence text with an empty commands list, and D43 (verify.rs) reads only the commands list, so the criterion was rejected. Say that each recorded check cited goes in that criterion's commands list exactly as shown under 'Checks Athena ran on this tree'. Keep the change verifier inside its token budget (cargo test --locked -p athena-cli --test budgets -- --nocapture prints the sizes) and edit only that prompt and CHANGELOG.md.

## Reading (accepted by the user)

Edit the verifier prompt so each recorded fmt/clippy/test check it cites goes in that criterion's commands list, exactly as shown under "Checks Athena ran on this tree", not only in evidence text. Add a CHANGELOG entry. Audience: the change verifier and the maintainer reading rejections.

Assumptions:
- The existing sentence is edited in place with a short addition; no other prompt text changes.
- The prompt's wording on 'a failed or missing record' stays as is.

## Acceptance criteria

1. [content] verifier.md says that each recorded check cited goes in that criterion's commands list, exactly as shown under 'Checks Athena ran on this tree', and that quoting it only in evidence text is not enough (D43 reads only the commands list).
2. [content] CHANGELOG.md has an entry under Unreleased naming the cause (job 20261024-033935-c4w4, empty commands list, D43) and the prompt change.
3. [content] Only crates/athena-cap-change/prompts/verifier.md and CHANGELOG.md differ from the base (git diff --stat).
4. [behavior] The budgets test passes and shows the change verifier inside its token budget: run cargo test --locked -p athena-cli --test budgets -- --nocapture.
5. [behavior] Existing tests that read the verifier prompt still pass: run cargo test -p athena-cap-change.

## Expected deliverables

- crates/athena-cap-change/prompts/verifier.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing verify.rs or D43 logic
- Other prompts
- Design docs

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One sentence added to one prompt plus a changelog entry.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Budget and crate tests are behavior criteria.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion checks the instruction names the commands list, which is what D43 reads.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: content read Run: `grep -n "commands list" crates/athena-cap-change/prompts/verifier.md, then read the same line`. Observes: The sentence names the commands list, says each recorded check cited goes there exactly as shown under 'Checks Athena ran on this tree', and says evidence-text-only citation is not enough because D43 reads only the commands list.
- Criterion 2: content read Run: `Read CHANGELOG.md lines 1-20 and grep for 'c4w4' and 'D43'`. Observes: An entry under the [Unreleased] heading names job 20261024-033935-c4w4, the empty commands list, D43, and the verifier prompt change.
- Criterion 3: diff scope check Run: `git diff --stat`. Observes: Only crates/athena-cap-change/prompts/verifier.md and CHANGELOG.md appear in the diff against the base.
- Criterion 4: run the budgets test Run: `cargo test --locked -p athena-cli --test budgets -- --nocapture`. Observes: The every_role_set_is_within_its_budget test passes, and the printed verifier size is inside the verifier budget.
  Fails without the behavior: If the added clause pushed the change verifier set over its budget, check_budget returns an error and the test fails.
- Criterion 5: run the crate tests Run: `cargo test -p athena-cap-change`. Observes: All tests in athena-cap-change pass, including any that read the verifier prompt text.
  Fails without the behavior: If the edit broke a test that reads the prompt, or removed the sentence a test looks for, the crate test run fails.

Planned files:

- crates/athena-cap-change/prompts/verifier.md (modify)
- CHANGELOG.md (modify)

## The change

Base c0047999bd2a → candidate bd05fc786531. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 | 5 +++++
 crates/athena-cap-change/prompts/verifier.md | 4 ++--
 2 files changed, 7 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1033 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
