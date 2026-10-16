# Job 20261016-011904-mzhw: Send finished work to the verifier after an incomplete send-back report

## Request (verbatim)

> Six jobs failed at the producer self-check (k2ej, 5qem, be8x, qb41, 5wwj, 582g). In 20261016-005953-582g the producer had done the work, but on the send-back pass its report covered only 3 of the 5 criteria, so the job failed over unreported ones. The send-back should make finished work reach the verifier, not fail it over an incomplete report.

## Reading (accepted by the user)

When the producer's report is still incomplete after the one send-back, run.rs fails the job ("still had problems when sent back"). Change it so that if the producer made changes, the work and last report go to the verifier, with the unreported criteria flagged, and only a no-change result fails. Audience: Athena operators whose jobs (e.g. 582g) failed over report gaps.

Assumptions:
- The fix is in the send-back loop in crates/athena-core/src/run.rs (~lines 714-723).
- Missing real full-test result after send-back also goes to the verifier, which can run cargo test itself; only 'no change' fails.
- Pinned-study jobs behave as before.
- I did not read the other five jobs' records; they are absent from my copy.

## Acceptance criteria

1. [behavior] A lifecycle test in which the producer, on the send-back pass, reports only some criteria but leaves real changes reaches the verifier and does not end in the 'still had problems when sent back' failure; run `cargo test`.
2. [behavior] A test where the producer makes no change after send-back still fails the job; run `cargo test`.
3. [content] The unreported or unmet criteria and any missing test result from the last report are kept in the attempt invocation's detail and passed to the verifier as context, so the verifier judges them rather than the self-check.
4. [content] The send-back prompt text in run.rs no longer says a still-missing criterion fails the job, and matches the new behavior.
5. [content] A CHANGELOG line and a decision entry record the change and why, citing job 582g.
6. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-context/prompts/producer.md (modify)

## Out of scope

- Changing how many send-backs are allowed (PRODUCER_SEND_BACKS)
- Diagnosing the other five failed jobs individually
- Changing the verifier's own criteria checks

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for the new path; cargo test and clippy criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the lifecycle scenario rather than checking wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Change the send-back's give-up policy rather than special-casing job 582g.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry and CHANGELOG written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the send-back path.
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
