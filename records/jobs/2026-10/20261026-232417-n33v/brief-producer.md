# Job 20261026-232417-n33v: Move gh test modules into their own files, text unchanged (G8)

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

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

18 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Running `cargo test --locked -p athena-core -- --list` on the candidate gives the same sorted test names as the base (after normalising any path prefix) and the same count; run on both and compare.
`cargo test --locked` passes, `cargo clippy --all-targets --locked -- -D warnings` is clean, and `cargo fmt --all --check` passes.
`git diff -M --color-moved=zebra` shows crates/athena-core/src/gh/mod.rs test bodies as moved to gh/tests.rs (and any sibling test file) with every line identical; the only non-moved lines are `mod`, `use` and `#[cfg(test)]` lines.
mod.rs no longer contains `mod tests {`, `mod records_flow {` or `mod after_merge_tests {` bodies, only `#[cfg(test)] mod ...;` declarations; its public re-exports (`pub use client/early/landing/pages/records/remote/transport`) are unchanged, and `git diff --stat` lists only files under crates/athena-core/src/gh/ and CHANGELOG.md.
CHANGELOG.md has a new G8 entry stating what moved, the test count before and after, and a caller check (Grep count over crates/ of each moved name's `gh::` path, equal before and after); it names both gh sibling links correctly (early->landing for LandPr and work_branch; pages->landing for Poll) and does not change the G7 entry.
No test file under crates/*/tests/ and no test line other than `use` lines is edited; no Rust behavior or visibility change in non-test code.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
