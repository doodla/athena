# Job 20261104-194933-s8v3: verify "Record lifted budget caps; keep job_usd 50 so tests pass unchanged"

## Request (verbatim)

> try again on 20261104-171104-yb47

## Reading (accepted by the user)

Retry of job 20261104-171104-yb47, whose candidate was rejected for editing six test and code files to pin job_usd 60. This reading keeps job_usd at 50, which already covers the 47.00 stage-cap reserve, so no crates/ file changes. It confirms the lifted caps in athena.toml and records them in CHANGELOG.md for the owner.

Assumptions:
- Goal: the owner (4 Nov) wants Athena's per-agent budgets and timeouts lifted; the audience is the owner and the verifier checking athena.toml and the changelog.
- Job 20261104-171104-yb47 addressed: its major finding (six files outside scope edited: config.rs, admission.rs, reading.rs, run/tests.rs, run_pin_tests.rs, drain.rs) is fixed by dropping all of those edits. Its job_usd 60 and test-pin edits are dropped, not kept. Its CHANGELOG.md and athena.toml edits are kept only as redone on the base.
- Read from athena.toml in the snapshot: reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, producer_timeout_s 5400, verifier_timeout_s 3600, planner_usd 3.00, critic_usd 3.00, max_rounds 5 already hold the target values; job_usd is 50.00. Reserve is 2+10+5+5*(3+3)=47, so 50 covers it and admission accepts.
- Tests pin job_usd = 50.00 (admission.rs, run_pin_tests.rs, reading.rs, config.rs tests), so raising it to 60 would force test edits. Keeping 50 meets the goal without that. The earlier 60 came from a 40 refusal that no longer applies.
- If the snapshot's athena.toml differs from the values above, the producer corrects only value lines; if a test or admission rule then refuses, it stops and reports rather than editing crates/.
- The change is a CHANGELOG.md Unreleased entry naming each raised cap with old and new value; the old values are the ones in the earlier request.

## Acceptance criteria

1. [content] git diff file list contains only CHANGELOG.md, plus athena.toml only if a value line needed correcting; nothing under crates/ (including config.rs, admission.rs, reading.rs, tests) changes.
2. [content] athena.toml [budget] has reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, job_usd 50.00, producer_timeout_s 5400, verifier_timeout_s 3600, and [job_plan] has planner_usd 3.00, critic_usd 3.00, max_rounds 5; no key is added or removed (Grep athena.toml).
3. [content] job_usd (50) is at least the stage-cap reserve 2+10+5+5*(3+3)=47, checkable by arithmetic from athena.toml, so admission accepts.
4. [content] CHANGELOG.md Unreleased section names each lifted cap with its old and new value (reader 0.50->2.00, producer 3.00->10.00, verifier 2.00->5.00, job 12.00->50.00, producer timeout 3000->5400, verifier timeout 2400->3600, planner 0.75->3.00, critic 0.75->3.00, max_rounds 3->5) and says it follows the owner's 4 Nov direction.
5. [behavior] Run `cargo test --locked` with no test file edited; it passes, including athena-data repo_config tests and the admission budget tests.

## Expected deliverables

- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Raising job_usd to 60 or any change to tests that pin job_usd 50.00
- Any change under crates/, including crates/athena-data/src/config.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/tests.rs, crates/athena-core/src/run_pin_tests.rs, crates/athena-cli/tests/drain.rs
- Adding or removing config keys
- Changing other budget values such as reader_timeout_s or check_timeout_s

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the caps are in place, admission arithmetic holds, and cargo test passes unchanged.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; no code or test edits.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Drops the job_usd 60 choice that forced test edits, rather than patching tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog entry lists old and new values for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test and reads the diff file list.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b8eda8fd4578 → candidate 158689b99526. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md | 20 +++++++++++++++-----
 1 file changed, 15 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41900945050 for 158689b99526334bb298662a3fef9335d431537c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41900945050 for 158689b99526334bb298662a3fef9335d431537c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41900945050 for 158689b99526334bb298662a3fef9335d431537c)
- ci-result: `not a check` passed (Check results for 158689b99526334bb298662a3fef9335d431537c come from the CI run https://github.com/doodla/athena/actions/runs/41900945050 (400s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked` with no test file edited; it passes, including athena-data repo_config tests and the admission budget tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
