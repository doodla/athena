# Job 20261026-232417-n33v: verify "Move gh test modules into their own files, text unchanged (G8)"

## Request (verbatim)

> Priority: high
> Structure pass job G8 (docs/design/structure-pass.md, section 4). Move the three gh test modules to their own files, test text unchanged; compare the test list after normalising the path prefix. No behavior change. Depends on 20261026-232414-p8n4.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/tests.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure-pass job G8 moves the gh test modules (`tests` with its nested `records_flow`, and `after_merge_tests`) out of crates/athena-core/src/gh/mod.rs into gh/tests.rs and, if needed, a sibling file, with test text unchanged. Audience is the owner and verifier checking the section 5 proof. Follow-up to landed job 20261026-232414-p8n4 (G7), whose landing.rs move is kept as is.

Assumptions:
- Previous job 20261026-232414-p8n4 (G7, landed, pass): its two minor findings were about wording in its CHANGELOG G7 entry and the structure-pass.md note (the early/landing two-way link understated, pages->landing via Poll missed; LandPr vs work_branch contradiction). They concern G7's own entry, not the test move, so G8 does not edit that entry; the G8 entry must not repeat the errors and, where it mentions the cycles, names both links (early uses LandPr and work_branch from landing; pages uses Poll from landing while landing uses pages items).
- G7's changed files are kept as landed: landing.rs, structure-pass.md and the G7 CHANGELOG text are not touched; mod.rs and CHANGELOG.md are modified again (CHANGELOG with a new G8 entry only). early.rs and client.rs are not touched.
- Code survey: read crates/athena-core/src/gh/mod.rs. It holds `#[cfg(test)] mod after_merge_tests` (line 109), `#[cfg(test)] mod tests` (line 253, with `use super::client::TEST_NAME_LIMIT; use super::*;`) and nested `mod records_flow` (line 2230); `early_seam` is a test seam, not a test module, and stays. Tests reach private items through `use super::*`, so a file at gh/tests.rs with `mod tests;` in mod.rs keeps `super` the same and test paths become gh::tests::... as before. No entry point changes.
- The 'three test modules' are after_merge_tests, tests and records_flow. records_flow stays nested in tests.rs. after_merge_tests moves to gh/after_merge_tests.rs (a small extra file) so its names stay gh::after_merge_tests::*; if it is kept in tests.rs instead, its list prefix is normalised too. The producer decides and reports.
- The test list is compared as `cargo test --locked -p athena-core -- --list` before and after; names should be identical (the module paths gh::tests:: and gh::after_merge_tests:: do not change since the module names stay), and the count is recorded in the report.
- The only expected non-moved lines in the diff are `mod x;` declarations in mod.rs, `use` lines, and removal of mod.rs imports that become unused; docs citing gh.rs stay out of scope.

## Acceptance criteria

1. [behavior] Running `cargo test --locked -p athena-core -- --list` on the candidate gives the same sorted test names as the base (after normalising any path prefix) and the same count; run on both and compare.
2. [behavior] `cargo test --locked` passes, `cargo clippy --all-targets --locked -- -D warnings` is clean, and `cargo fmt --all --check` passes.
3. [content] `git diff -M --color-moved=zebra` shows crates/athena-core/src/gh/mod.rs test bodies as moved to gh/tests.rs (and any sibling test file) with every line identical; the only non-moved lines are `mod`, `use` and `#[cfg(test)]` lines.
4. [content] mod.rs no longer contains `mod tests {`, `mod records_flow {` or `mod after_merge_tests {` bodies, only `#[cfg(test)] mod ...;` declarations; its public re-exports (`pub use client/early/landing/pages/records/remote/transport`) are unchanged, and `git diff --stat` lists only files under crates/athena-core/src/gh/ and CHANGELOG.md.
5. [content] CHANGELOG.md has a new G8 entry stating what moved, the test count before and after, and a caller check (Grep count over crates/ of each moved name's `gh::` path, equal before and after); it names both gh sibling links correctly (early->landing for LandPr and work_branch; pages->landing for Poll) and does not change the G7 entry.
6. [content] No test file under crates/*/tests/ and no test line other than `use` lines is edited; no Rust behavior or visibility change in non-test code.

## Expected deliverables

- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (create)
- crates/athena-core/src/gh/after_merge_tests.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Any behavior change or edit to test text beyond use lines
- Editing landing.rs, early.rs, client.rs, pages.rs or other gh siblings
- Editing docs/design/structure-pass.md or the G7 CHANGELOG entry
- Updating docs that cite gh.rs (system-facts.md, architecture.md)
- R9 (run.rs test move) and any other row
- Editing introspect.rs, external_calls_guard.rs, athena.toml, defaults.toml or workflows
- Edits to callers in main.rs, drain.rs, run.rs, display.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy -D warnings and full cargo test must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the test list on base and candidate and compares.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the test list and passing tests, not file existence.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is an unchanged test list and diff that is only a move.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move; nothing else touched.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9bf2b06fc93f → candidate 52258d1e85be. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                   |   14 +
 crates/athena-core/src/gh/after_merge_tests.rs |  140 ++
 crates/athena-core/src/gh/mod.rs               | 2833 +-----------------------
 crates/athena-core/src/gh/tests.rs             | 2669 ++++++++++++++++++++++
 4 files changed, 2825 insertions(+), 2831 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test --locked` passes, `cargo clippy --all-targets --locked -- -D warnings` is clean, and `cargo fmt --all --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
