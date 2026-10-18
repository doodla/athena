# Job 20261018-095724-dqht: Make land --pr rerun a failed check once and name the failing test

## Request (verbatim)

> athena land --pr gives up when the pull request's checks have not finished and when one check fails, so every landing needs someone to rerun it. Make land wait for the checks to finish (within a bounded time), re-run a failed check job once, and report a second failure with the failing test's name, so a landing completes without a person.

## Reading (accepted by the user)

`athena land --pr` already polls checks for about ten minutes (30 looks, 20 s apart), but a red check ends the run with no retry and no detail. I read the request as: keep the bounded wait, rerun the failed check jobs once when a check fails and wait again, and on a second failure report the failing test's name. The audience is the operator who wants a landing to finish unattended.

Assumptions:
- Waiting already exists (Poll, 30 x 20 s); that bound is kept, applied to the first wait and to the rerun wait separately or in total, and stated in the code.
- Only failed check jobs are re-run (GitHub rerun-failed-jobs), once per land invocation; a stored count is not needed beyond the invocation unless the design needs it.
- The failing test's name is extracted from the failed job's log or annotations for cargo test output (`test x::y ... FAILED`); if none is found the report says so rather than guessing.
- The merge gate is unchanged: nothing merges unless the checks are green and merge_via_rest is on.
- Exit code stays 2 unless the checks end green or the job lands.

## Acceptance criteria

1. [behavior] A unit test in gh.rs with a fake PrHost shows a first red check triggers exactly one rerun of the failed jobs, then a second wait; if the rerun goes green, land_pr reports Green and merges when merge_via_rest is on. Run `cargo test -p athena-core gh`.
2. [behavior] A unit test shows a second red result triggers no further rerun and the PrReport carries the failing test's name (parsed from the failed job's log or annotations). Run `cargo test -p athena-core gh`.
3. [behavior] A test shows that checks still pending after the bounded number of looks make land_pr return Pending without rerunning or merging, and that the total wait across the first wait, the rerun and the second wait stays bounded.
4. [content] The text printed by display::land_pr names the failing test on a second failure and says that the failed job was re-run once. The JSON report includes the same field.
5. [content] The real GhClient implements the rerun through the GitHub rerun-failed-jobs endpoint and reads the failing test name from a real host reply shape. This is covered by a parser test on a sample payload. Any new host calls go through the existing HostOperation retry and redaction path.
6. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass with no warnings.
7. [content] The design doc or CHANGELOG states the new behavior (wait, one rerun, failing-test report) so a stranger can find it.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/external_calls.rs (modify)

## Out of scope

- Changing the merge gate or the merge_via_rest setting
- Rerunning more than once or rerunning passing jobs
- Non-GitHub hosts
- Pages-run rerun logic

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests with fake host; cargo test and clippy must be clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rerun and the failing-test report happening, via fake-host tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The retry sits inside land_pr's check wait, not as a caller-side wrapper.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the gh.rs landing flow plus its display.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
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
