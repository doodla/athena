# Job 20261023-181804-j42y: Report space reclaimed when a closed job's build cache is deleted

## Request (verbatim)

> Athena never deletes a job's build cache. Each job's state directory keeps its own Rust target directory (11 to 12 GB each), and on 2026-10-23 two of them, one from an interrupted job of 2026-10-17, filled the session's disk allowance mid-run, so job 20261019-013306-zvmr died committing its candidate. Delete a job's target directory once the job closes (landed, failed, rejected, abandoned or interrupted and then closed), and report the space reclaimed; consider sharing one target directory across jobs if the checks allow it.

## Reading (accepted by the user)

Closing already calls StateDir::clean_job, which silently removes the job directory (including its 11-12 GB target) on some paths. The change makes cleanup cover every closing path, including a close of an interrupted job, report bytes reclaimed, and stop swallowing removal errors; sharing one target directory is evaluated against the checks and recorded as a decision (expected: not adopted). Audience: the Athena operator who ran out of disk.

Assumptions:
- Goal: stop closed jobs from leaving 11-12 GB caches that fill the disk; audience is the Athena operator and the verifier reading output and tests.
- Reading of the code: clean_job (state.rs) is already called after close_with, close_dispatched, land_or_block (non-blocked), retry_landing, but it ignores errors, reports nothing, and is skipped when an earlier step errors (e.g. commit out of disk), so leftovers persist.
- Sharing one target directory is considered but not adopted by default: review 2 R14 shows a shared dir reuses another job's stale test binaries; per-job isolation is deliberate (D46).
- Deleting caches of already-closed historical jobs on this machine is an operational action, not a code deliverable; optionally a sweep could be added but is not assumed.

## Acceptance criteria

1. [behavior] A test creates a job directory containing a target/ tree of known size, closes the job through each closing path (landed, failed, rejected, abandoned, interrupted-then-`athena close`, closed while blocked), and asserts the job directory no longer exists and the reported reclaimed byte count equals the known size. Run: cargo test.
2. [content] The closing line shown to the user (athena go / close output) states the space reclaimed in human units, e.g. 'reclaimed 11.4 GB'; a job with nothing to remove reports no failure.
3. [behavior] If removal of the job directory fails, the user sees a warning naming the path rather than silent success (test with an unremovable path or injected error). Run: cargo test.
4. [behavior] A job that is still running or blocked keeps its directory; cleanup runs only after the closing commit succeeds. Run: cargo test.
5. [content] docs/decisions.md records the shared-target-directory question with alternatives and the evidence (docs/design/reviews/athena-v0-review-2.md R14: a shared target dir executed another job's test binaries), and the decision taken; the per-job CARGO_TARGET_DIR in athena.toml stays unless the decision says otherwise.
6. [behavior] cargo test, cargo clippy with warnings denied, and cargo fmt --check all pass.
7. [content] CHANGELOG.md has an entry describing that closing a job deletes its build cache and reports space reclaimed.

## Expected deliverables

- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/pin.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Switching to a shared or persistent cross-job CARGO_TARGET_DIR
- Manually deleting directories on the operator's machine
- Fixing the land --pr head-checking behavior of job 20261019-013306-zvmr
- Capping disk use during a running job

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each closing path and removal failure; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert directory gone and byte count reported, not that a function exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Covers all closing paths and error cases instead of the one that surfaced.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Shared-target alternative written down in decisions.md with evidence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to cleanup and reporting.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
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
