# Job 20261026-232446-rhxw: Move run test modules from mod.rs into run/tests.rs, text unchanged

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

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
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

20 more qualifying classes are left out.

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

crates/athena-core/src/run/tests.rs exists and holds the test modules removed from run/mod.rs; mod.rs declares them with `#[cfg(test)] mod tests;` and the `git diff` shows the moved test bodies as identical text apart from import/use lines and indentation.
No non-test item in run/mod.rs changes in the diff except the module declaration. Any visibility change is limited to pub(super) or pub(crate) needed by the tests. Grep for `run::` in crates/athena-core/src/main.rs, drain.rs and the other callers returns the same set of paths before and after (visible in the diff, which does not touch those files).
For each name moved or re-exported, Grep of the repository snapshot shows its callers still resolve through the same `run::` path. The diff contains no edits to caller files.
Run `cargo test --locked`: it passes and the test count equals the pre-change count (1737 in the R8 recorded check, or the count on the base commit). Compare names with `cargo test --locked -- --list` before and after, where the only difference is the module path prefix if any.
Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both exit 0.
CHANGELOG.md has a new R9 entry describing the move of the run test modules to run/tests.rs with no behavior change, and Grep finds it.
The diff touches only crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/tests.rs and CHANGELOG.md. It does not touch athena.toml, defaults.toml, workflows, or docs/design/structure-pass.md.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
