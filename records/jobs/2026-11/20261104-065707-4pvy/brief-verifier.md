# Job 20261104-065707-4pvy: verify "Fix firstmate study Reopened state: cited return to Queued, tests green"

## Request (verbatim)

> try again on 20261104-062715-yv4m

## Reading (accepted by the user)

Retry of job 20261104-062715-yv4m: make 'Reopened' in the firstmate study a non-terminal state that returns to Queued, with only Done ending the task. Add a cited claim for the Queued return, fix the t12 guard-'none' test failure, and fix the fmt failure in xtask/src/site.rs.

Assumptions:
- Job 20261104-062715-yv4m findings are addressed: (1) the t12 guard-'none' test failure is fixed by giving the return a non-'none' evaluated guard or by pointing t9 at queued; (2) the uncited Queued return gets a new claim in claims.json; (3) the cargo fmt failure at xtask/src/site.rs:963 is fixed; (4) validation.json is updated if needed; (5) the behavior criterion names the commands to run. The earlier t9/t12 double-event unsoundness is removed.
- xtask/src/site.rs exists at the base commit. The earlier candidate changed it, and that is kept as a deliverable: it is limited to the tests that asserted Reopened's old tag or counted transitions or cards, and it must be formatted. Its other changes in CHANGELOG.md, intro.json and model.json are kept and reworked as described.
- Goal: a correct 'Reopened' state on the firstmate study page, so the post-landing source check stops refusing it. The audience is the study page reader and that check. The reading is docs-class: study data is edited and Rust tests are only adjusted to match.

## Acceptance criteria

1. [content] In studies/firstmate-work-handling/model.json the 'reopened' state no longer has kind 'terminal-success' (Grep confirms); it is a non-terminal kind with a way back to 'queued', matching bin/fm-teardown.sh sending the backlog row back to Queued.
2. [content] studies/firstmate-work-handling/claims.json gains a new claim citing the pinned fm-teardown.sh lines (around 3894-3896) that show the backlog row returning to Queued. Every new or changed Queued-return text in model.json and intro.json cites that claim, not only c23-c25.
3. [content] Reopened's way out of the model is sound: t9 still reaches Reopened and the return to Queued is either t9 pointing straight at queued or a transition with a distinct cited trigger. It is not the t9 event repeated with guard 'none'. Any guard of 'none' has no guard_evaluated_by value, as the study rule test requires.
4. [content] studies/firstmate-work-handling/intro.json: the 'states' lead-in says only Done ends the task (Grep for 'ends the task' shows no Reopened). The 'reopened' glossary definition says the item goes back to Queued for the captain's answer and cites the new claim.
5. [content] studies/firstmate-work-handling/validation.json is updated to cover the new claim or transition if its structure requires it. CHANGELOG.md has an entry. Changes to xtask/src/site.rs are limited to the tests that asserted the old 'ends the task' tag on Reopened or counted transitions or cards. They are rustfmt-clean, including the starts_with call near line 963.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`, and `cargo test --locked`. All pass, including athena_cap_study rules::tests::a_guard_of_none_is_never_said_to_be_evaluated_by_code and the xtask site_page and policy tests, and the verifier sees the commands run.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- CHANGELOG.md (modify)
- xtask/src/site.rs (modify)

## Out of scope

- Changing the firstmate source code or the pinned commit
- Other states or study mechanisms
- Changing the study rules in crates/ or the source-check logic

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check Reopened's kind, its cited Queued return, and the full fmt, clippy and test run.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The new Queued-return claim cites the pinned source lines.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the model, the intro and the uncited Queued return, not just the tag.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior criterion names the commands to run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 26329fa42b18 → candidate c8415f97b2a4. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  8 ++++++++
 studies/firstmate-work-handling/claims.json |  7 +++++++
 studies/firstmate-work-handling/intro.json  |  5 +++--
 studies/firstmate-work-handling/model.json  |  9 ++++++++-
 xtask/src/site.rs                           | 10 +++++++---
 5 files changed, 33 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41791212239 for c8415f97b2a4ac3c2a3298c0db335bb7e1ac5681)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41791212239 for c8415f97b2a4ac3c2a3298c0db335bb7e1ac5681)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41791212239 for c8415f97b2a4ac3c2a3298c0db335bb7e1ac5681)
- ci-result: `not a check` passed (Check results for c8415f97b2a4ac3c2a3298c0db335bb7e1ac5681 come from the CI run https://github.com/doodla/athena/actions/runs/41791212239 (606s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`, and `cargo test --locked`. All pass, including athena_cap_study rules::tests::a_guard_of_none_is_never_said_to_be_evaluated_by_code and the xtask site_page and policy tests, and the verifier sees the commands run.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
