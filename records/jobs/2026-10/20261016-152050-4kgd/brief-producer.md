# Job 20261016-152050-4kgd: Make interrupted jobs retryable and picked up by drain

## Request (verbatim)

> A job interrupted by a machine restart (20261016-071045-mx7e, container restart at 15:18 UTC) cannot be retried: retry accepts only rejected or failed jobs, so it had to be closed and asked again by hand. An interrupted job should be resumable or retryable, and a drain should pick it back up.

## Reading (accepted by the user)

A job that is dispatched with no live process and no closing record (e.g. after a container restart) is accepted by `athena retry`, and `athena drain` lists it as work and retries it. Audience: the user running athena on machines that can restart, who today must close the job by hand and re-ask.

Assumptions:
- Interrupted means JobState::Dispatched with no process in the registry and no closing record.
- Retrying an interrupted job leaves the old job unclosed unless the existing flow closes it; the retry proposal carries the original request.
- The earlier attempt for an interrupted job may lack verification/outcome records, so the check requiring them must be relaxed for this case.
- Drain's queue selection is per channel, as for other retryable jobs.

## Acceptance criteria

1. [behavior] A test (cargo test) makes a job dispatched with no live process, runs retry on it, and gets a proposal whose request is 'try again on <id>' carrying the original request, not the error 'only a rejected or failed job can be tried again'.
2. [behavior] A test (cargo test) shows a job whose process is still running is refused by retry with a clear message saying it is running.
3. [behavior] A drain test (cargo test) with an interrupted job on the drained channel shows it listed as work and retried, with no empty-queue report; an interrupted job on another channel is left alone.
4. [content] The `athena status` next-step hint for interrupted jobs (status.rs) and the run.rs interrupted message name retry as a way to resume, alongside close.
5. [content] drain.rs module docs and CHANGELOG.md describe interrupted jobs being retryable and drained.
6. [behavior] cargo test, cargo clippy and cargo fmt --check all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/manage.rs (modify)

## Out of scope

- Changing how running jobs are tracked or supervised
- Automatic restart of jobs without drain or retry
- Changes to the plan/read state bug in the referenced request

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for retry and drain on interrupted jobs; cargo test, clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run retry and drain on an interrupted job.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix retry's state check and drain's selection, not just this job.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change.
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
