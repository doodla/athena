# Job 20261026-232426-91cn: verify "Move Send-back row and left_to_athena_tests to run/send_back.rs"

## Request (verbatim)

> Priority: high
> Structure pass job R2 (docs/design/structure-pass.md, section 4). Move the Send-back row with left_to_athena_tests. No behavior change. Depends on 20261026-232423-hvyb.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/send_back.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job R2: move unmet_criteria, left_to_athena, marked_left, is_athena_check_criterion, send_back_text, criterion_key and mod left_to_athena_tests from run/mod.rs into new run/send_back.rs, re-exported so paths like crate::run::send_back_text are unchanged. Pure move, no behavior change. Prior job 20261026-232423-hvyb (R1, run.rs to run/mod.rs) was closed as superseded because R1 landed via its retry; run/mod.rs already exists, so nothing from it is redone or kept.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs holds unmet_criteria (~1676), left_to_athena (~1695), marked_left, is_athena_check_criterion, send_back_text (~1803), criterion_key (~1836) and mod left_to_athena_tests (~3718); docs/design/structure-pass.md section 4 row R2 and section 5 proof define the move. No record or data files are added, so check_record_file/AnyRecord are not involved.
- Job 20261026-232423-hvyb (R1) is closed as superseded; its scope (delete run.rs, create run/mod.rs) is already landed, its candidate changed no files, so no finding remains and none of its files are touched here.
- Audience: the structure-pass reviewer/verifier who checks the section 5 proof.
- The 'report lists the caller check' item is satisfied by the verifier's own Grep of callers, not by the producer's report.
- No athena.toml, defaults.toml or workflow change is needed.

## Acceptance criteria

1. [content] crates/athena-core/src/run/send_back.rs exists and contains fn unmet_criteria, left_to_athena, marked_left, is_athena_check_criterion, send_back_text, criterion_key and mod left_to_athena_tests; Grep shows none of these definitions remain in run/mod.rs.
2. [content] run/mod.rs declares mod send_back and re-exports the pub(crate) names (unmet_criteria, left_to_athena, send_back_text) so Grep for run::(unmet_criteria|left_to_athena|send_back_text) across crates/ is unchanged and no caller path was edited.
3. [content] git diff shows moved function and test text identical (only visibility/import/use lines differ); no test other than imports is edited.
4. [behavior] Run cargo test --locked: passes, and the test list for left_to_athena_tests is unchanged in names (now under run::send_back).
5. [behavior] Run cargo fmt --all --check and cargo clippy --all-targets --locked -- -D warnings: both clean.
6. [content] CHANGELOG.md has a new entry describing the Send-back move as no behavior change.

## Expected deliverables

- crates/athena-core/src/run/send_back.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any other row of run/mod.rs (producer, verifier, etc.; later R jobs)
- Editing tests beyond import lines
- Changing athena.toml, defaults.toml or workflows
- Redoing R1 (run.rs to run/mod.rs), already landed
- Any behavior change or deduplicating test helpers

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust move.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single row move only.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests and caller Grep verify unchanged behavior.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests run and Grep of callers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d13a68a0edbd → candidate 800a6f024481. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   6 +
 crates/athena-core/src/run/mod.rs       | 299 +-------------------------------
 crates/athena-core/src/run/send_back.rs | 290 +++++++++++++++++++++++++++++++
 3 files changed, 305 insertions(+), 290 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --all --check and cargo clippy --all-targets --locked -- -D warnings: both clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
