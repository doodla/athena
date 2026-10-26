# Job 20261026-015419-4j37: Add scope-guard running-line watcher: flags, pauses, status display

## Request (verbatim)

> Priority: urgent
> Build job 5 of the scope guard (docs/design/scope-guard.md section 13, decision D127): the running-line watcher, rules only. Depends on 20261026-015416-94rc. After each attempt of a line of work, evaluate T1 to T5 and apply their effects: flags recorded and shown in athena status, and pauses (never kills) that stop the next attempt being dispatched, with a pause record giving the signal, the numbers and what would lift it (a measurement or a narrowed request for T4). athena retry on a paused line refuses with that reason.
> Acceptance:
> - A test with two rejections on the same unmet criterion shows the third attempt is not dispatched and a pause record exists; a test with differing criteria shows a flag only.
> - athena status lists flagged and paused lines with their signal.
> Files: crates/athena-core/src/scope_guard.rs, crates/athena-core/src/run.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, CHANGELOG.md

## Reading (accepted by the user)

After each attempt of a line of work, evaluate T1–T5 (already built in scope_guard.rs). Record flags and pauses (with signal, numbers, and lift condition), show them in `athena status`, and make dispatch and `athena retry` refuse a paused line with the pause reason. The audience is the owner running Athena, who needs thrashing lines stopped without any process being killed.

Assumptions:
- Job 20261026-015416-94rc has landed, so scope_guard.rs already has the signal functions and evaluate_before_attempt.
- Pause records are stored with the existing job-record mechanism; the exact form is chosen by the producer from existing records code.
- Rules only: no value scoring or model scorer.

## Acceptance criteria

1. [behavior] A test with two rejections on the same unmet criterion shows the third attempt is not dispatched and a pause record exists; run `cargo test`.
2. [behavior] A test with rejections on differing criteria records a flag only: no pause record, and the next attempt is dispatched; run `cargo test`.
3. [behavior] A test shows `athena retry` on a paused line refuses with the pause reason; run `cargo test`.
4. [behavior] A test shows the status output lists flagged and paused lines with their signal (T1–T5); run `cargo test`.
5. [content] A pause record names the signal, the numbers that tripped it, and what would lift it (a measurement, or a narrowed request for T4).
6. [content] Pauses never kill a running job; they only stop the next dispatch. CHANGELOG.md has an entry for the watcher.
7. [behavior] `cargo clippy` is clean and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/design/scope-guard.md (modify)

## Out of scope

- Value scoring and the model scorer
- Escalations in section 8
- Killing running jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Behavior tests for pause and flag, plus clippy and fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check dispatch refusal and status output.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change over the listed files.
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

A test with two rejections on the same unmet criterion shows the third attempt is not dispatched and a pause record exists; run `cargo test`.
A test with rejections on differing criteria records a flag only: no pause record, and the next attempt is dispatched; run `cargo test`.
A test shows `athena retry` on a paused line refuses with the pause reason; run `cargo test`.
A test shows the status output lists flagged and paused lines with their signal (T1–T5); run `cargo test`.
A pause record names the signal, the numbers that tripped it, and what would lift it (a measurement, or a narrowed request for T4).
Pauses never kill a running job; they only stop the next dispatch. CHANGELOG.md has an entry for the watcher.
`cargo clippy` is clean and `cargo fmt --check` passes.

## Report

End with the producer report as your structured output.
