# Job 20261010-154002-g61b: Retry once and log commit and record count when go can't resolve

## Request (verbatim)

> When athena go cannot resolve a job, log the commit and record count it read and retry once (m7, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

In `athena go` (go_job in crates/athena-core/src/run.rs), when `records.resolve` finds no job, log the HEAD commit and the number of records read via `ui.line`, reload records at the current HEAD, and resolve once more. Goal: make the unreproduced race in review item m7 diagnosable and self-healing for the maintainer.

Assumptions:
- Only `athena go` is changed; other callers of resolve are untouched.
- Record count means the number of record files in the loaded set (or jobs); the producer picks one and the log line labels it.
- The retry reloads HEAD, since the race is plausibly a stale head or a half-written commit.
- Logging goes through the existing Ui::line.
- The tests use the existing test harness; the test mechanism for injecting a first-load miss is the producer's choice.

## Acceptance criteria

1. [content] go_job in crates/athena-core/src/run.rs, on a 'no job matches' failure, emits a line through ui containing the HEAD commit id and the record count read, then re-reads HEAD and records and resolves once more.
2. [content] The retry happens exactly once: a second failure returns the original no-match error, and 'latest', ambiguous matches, and other errors are not retried.
3. [behavior] A new test shows a job that is absent from the first load but present at the retry resolves and runs, and a job that is never present fails after one retry with the log line emitted once. Run `cargo test`.
4. [behavior] `cargo clippy` is clean and `cargo fmt --check` passes.
5. [content] A CHANGELOG.md entry notes the log and retry, citing m7 in docs/design/reviews/athena-v0-acceptance-1.md.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- docs/design/reviews/athena-v0-acceptance-1.md (modify)

## Out of scope

- Finding the root cause of the m7 race.
- Changing resolve semantics for other commands.
- Edits to the review document.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A criterion checks the retry and the log happening, not that code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single change confined to go_job.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior criteria run with cargo test.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust.
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

## Report

End with the producer report as your structured output.
