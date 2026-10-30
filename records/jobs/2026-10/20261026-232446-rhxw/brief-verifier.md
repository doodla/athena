# Job 20261026-232446-rhxw: verify "Move run test modules from mod.rs into run/tests.rs, text unchanged"

## Request (verbatim)

> Priority: high
> Structure pass job R9 (docs/design/structure-pass.md, section 4). Move the run test modules, test text unchanged. No behavior change. Depends on 20261026-232443-f4d7.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/tests.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job R9: move the #[cfg(test)] modules in crates/athena-core/src/run/mod.rs into a new run/tests.rs, with no behavior change and test text unchanged. Follows job 20261026-232443-f4d7 (R8, landed, verification pass), whose work is kept as landed. The audience is the verifier and maintainers of the structure pass.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs has #[cfg(test)] items at lines ~1000, 1592 and 1663 (grep result); other submodules (planning.rs, send_back.rs) keep their own test modules and are untouched. The tests are moved as a `#[cfg(test)] mod tests;` declaration plus run/tests.rs. No records or data file is added, so the records allow-list (check_record_file, AnyRecord) is not touched. The property is measured directly by the test list from cargo test.
- Prior job 20261026-232443-f4d7 (R8): its two findings are a CHANGELOG line-count nit (68 vs 67) and a note that check_and_verify is still ~220 lines. Neither applies to R9: the R8 CHANGELOG entry is not retro-edited, and splitting check_and_verify is follow-up work. R8's files (verification.rs, mod.rs, CHANGELOG.md) are kept as landed; mod.rs and CHANGELOG.md are edited again here, and verification.rs is out of scope.
- Section 5 'proof' means: tests pass with only import lines edited, the test list is unchanged, moved text is identical, and public paths are unchanged. The caller check is made by Grep, not by a report.
- Test modules may need `use super::*` or pub(super) visibility on private items they use; minimal visibility widening for tests only is acceptable.
- Priority high and depends on R8, which has landed.

## Acceptance criteria

1. [content] crates/athena-core/src/run/tests.rs exists and holds the test modules removed from run/mod.rs; mod.rs declares them with `#[cfg(test)] mod tests;` and the `git diff` shows the moved test bodies as identical text apart from import/use lines and indentation.
2. [content] No non-test item in run/mod.rs changes in the diff except the module declaration. Any visibility change is limited to pub(super) or pub(crate) needed by the tests. Grep for `run::` in crates/athena-core/src/main.rs, drain.rs and the other callers returns the same set of paths before and after (visible in the diff, which does not touch those files).
3. [content] For each name moved or re-exported, Grep of the repository snapshot shows its callers still resolve through the same `run::` path. The diff contains no edits to caller files.
4. [behavior] Run `cargo test --locked`: it passes and the test count equals the pre-change count (1737 in the R8 recorded check, or the count on the base commit). Compare names with `cargo test --locked -- --list` before and after, where the only difference is the module path prefix if any.
5. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both exit 0.
6. [content] CHANGELOG.md has a new R9 entry describing the move of the run test modules to run/tests.rs with no behavior change, and Grep finds it.
7. [content] The diff touches only crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/tests.rs and CHANGELOG.md. It does not touch athena.toml, defaults.toml, workflows, or docs/design/structure-pass.md.

## Expected deliverables

- crates/athena-core/src/run/tests.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving non-test code or other stages
- Any behavior change, or test edits beyond import lines
- Editing docs/design/structure-pass.md, including done-markers
- Editing verification.rs or other run submodules landed by R8 and earlier (candidate.rs, send_back.rs, close.rs, checks.rs, producer.rs, planning.rs)
- Retro-editing the R8 or R7 CHANGELOG entries, or splitting check_and_verify
- Editing athena.toml, defaults.toml, workflows, or callers in main.rs or drain.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move of the test modules.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs fmt, clippy and test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test list and count are compared by running cargo test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry is written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 6578db450a48 → candidate 9ff9839f921c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |    6 +
 crates/athena-core/src/run/mod.rs   | 1528 +----------------------------------
 crates/athena-core/src/run/tests.rs | 1528 +++++++++++++++++++++++++++++++++++
 3 files changed, 1535 insertions(+), 1527 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1737 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
