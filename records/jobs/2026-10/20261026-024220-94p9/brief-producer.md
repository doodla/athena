# Job 20261026-024220-94p9: Build scope-guard running-line watcher (job 5): flag, pause, refuse

## Request (verbatim)

> try again on 20261026-015419-4j37

## Reading (accepted by the user)

Retry of the failed watcher job. After each attempt of a line, evaluate T1–T5 from scope_guard.rs, record flags and pause records, show them in athena status, and make dispatch and Athena::retry refuse a paused line. Audience is the owner running Athena. Demonstrations use states the code accepts, per the user's two decisions.

Assumptions:
- Pauses only stop the next dispatch; no running process is killed.
- T3 p90 is read from the job's reading forecasts (user decision).
- Demonstrations avoid calling go on a Dispatched job (user decision).
- reading.rs is edited for the Athena::retry refusal.

## Acceptance criteria

1. [behavior] A test with two rejections on the same unmet criterion, with the third request made via ask('try again on <id>') (not retry), shows the third attempt is not dispatched and a pause record exists; a control with no pause file dispatches. Run cargo test.
2. [behavior] A test with rejections on differing criteria records a flag only: no pause record, and the next attempt is dispatched. Run cargo test.
3. [behavior] A test shows Athena::retry on a paused line refuses with the pause reason, and dispatch finds the line head by rebuilding RequestRecords and calling group_lines, then looking up the pause file by the line's first id. Run cargo test.
4. [behavior] A T3 test passes a real positive forecast p90 dollars read from the job's reading forecasts, pauses the line, and fails if the pause is removed. Run cargo test.
5. [behavior] A test registers a real child process (e.g. sleep) as a line's running job, drives the watcher to a pause on that line, and asserts the child is still alive and its registry entry unchanged. Run cargo test.
6. [behavior] A test shows athena status output lists flagged and paused lines with their signal (T1–T5). Run cargo test.
7. [content] A pause record names the signal, the numbers that tripped it, and what would lift it (a measurement, or a narrowed request for T4).
8. [content] CHANGELOG.md has an entry for the watcher; docs/design/scope-guard.md reflects job 5 and states that p90 comes from the reading's forecasts and is used by t3_cost.
9. [behavior] cargo clippy is clean, cargo fmt --check passes, and cargo test passes.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/design/scope-guard.md (modify)

## Out of scope

- Changing T1–T5 thresholds or rules
- Killing or interrupting running jobs
- Lifting pauses automatically

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing refusal, with controls that dispatch without the pause.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check criterion.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the listed files.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check lines being stopped, not code existing.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

A test with two rejections on the same unmet criterion, with the third request made via ask('try again on <id>') (not retry), shows the third attempt is not dispatched and a pause record exists; a control with no pause file dispatches. Run cargo test.
A test with rejections on differing criteria records a flag only: no pause record, and the next attempt is dispatched. Run cargo test.
A test shows Athena::retry on a paused line refuses with the pause reason, and dispatch finds the line head by rebuilding RequestRecords and calling group_lines, then looking up the pause file by the line's first id. Run cargo test.
A T3 test passes a real positive forecast p90 dollars read from the job's reading forecasts, pauses the line, and fails if the pause is removed. Run cargo test.
A test registers a real child process (e.g. sleep) as a line's running job, drives the watcher to a pause on that line, and asserts the child is still alive and its registry entry unchanged. Run cargo test.
A test shows athena status output lists flagged and paused lines with their signal (T1–T5). Run cargo test.
A pause record names the signal, the numbers that tripped it, and what would lift it (a measurement, or a narrowed request for T4).
CHANGELOG.md has an entry for the watcher; docs/design/scope-guard.md reflects job 5 and states that p90 comes from the reading's forecasts and is used by t3_cost.
cargo clippy is clean, cargo fmt --check passes, and cargo test passes.

## Report

End with the producer report as your structured output.
