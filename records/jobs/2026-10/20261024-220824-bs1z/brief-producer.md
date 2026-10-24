# Job 20261024-220824-bs1z: Add incremental attribution checking with checkpoint records

## Request (verbatim)

> Build job 6 of the records-history plan (docs/design/records-history.md section 9, decision D111): incremental checking. After a passing local attribution check (athena check --attribution, and athena land after a landing record), write a checkpoint record to athena/records with the records writer, naming the last passing main commit, the records-branch tip it was judged against, and the check's version; a later check judges only main's commits and records added since the newest checkpoint. A full-replay mode ignores checkpoints, judges everything from the start, and fails if it disagrees with the newest checkpoint. CI's token is read-only, so CI never writes checkpoints: its attribution job reads the newest one and runs incrementally on pushes, and as a full replay on its weekly schedule. Each run reports seconds taken and the number of commits and records judged. Tests: an incremental run judges only the new commits; a tampered checkpoint (naming a commit the full replay fails) is caught by the replay; a check-version change forces a full replay; a failing check writes no checkpoint. Timing: a producer's working copy has no history of this repository, so time a full replay and an incremental run on a synthetic repository of about 200 landings in a test or bench and report them as measured on that fixture; the timing on this repository is taken after landing, outside this job. This repeats job 20261024-220048-engm, which stopped because its request asked for timing here.

## Reading (accepted by the user)

Implement job 6 of the records-history plan (D111). A passing local `athena check --attribution` (and `athena land` after its landing record) writes a checkpoint record. Later checks judge only newer commits and records. A full-replay mode ignores checkpoints and fails on disagreement. CI reads checkpoints only, and the weekly run is a full replay. Timing is measured on a synthetic ~200-landing fixture, not on this repository. The audience is the maintainer and the CI attribution job.

Assumptions:
- The Checkpoint record type already exists in athena-data history.rs (main_sha, records_sha, verifier_version), and the records writer exists.
- audit_history in records_branch.rs has no range form, so a range or since-checkpoint audit is added there.
- The user has accepted the synthetic-fixture timing; timing on this repository is out of scope.
- CI is detected through the existing environment or a flag and forces read-only behaviour, with the workflow's permissions staying read-only.
- The newest checkpoint is chosen by records-branch order, and no checkpoint is written past a pending commit.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows an incremental run after a checkpoint judges only the commits and records added since it, with the reported counts equal to the new ones.
2. [behavior] Run `cargo test`: a tampered checkpoint that names a commit the full replay fails is caught by the full replay, which exits with failure.
3. [behavior] Run `cargo test`: a change in the check version forces a full replay, and the report says why.
4. [behavior] Run `cargo test`: a failing attribution check writes no checkpoint, and the records-branch tip is unchanged.
5. [behavior] Run `cargo test`: a passing `check --attribution` and a passing `land` each write a checkpoint naming the last passing main commit, the records tip judged against, and the check version.
6. [behavior] Run `cargo test`: the full-replay mode ignores checkpoints and judges everything, and a CI or read-only mode never writes a checkpoint.
7. [behavior] Run `cargo test`: each run output reports seconds taken and the number of commits and records judged.
8. [behavior] Run `cargo test`: a test or bench on a synthetic repository of about 200 landings times a full replay and an incremental run and prints both numbers, marked as measured on that fixture.
9. [content] .github/workflows/ci.yml keeps the token read-only. Pushes run the incremental check and the weekly schedule runs the full replay.
10. [content] The CHANGELOG entry describes the feature and the fixture timings, marked as measured on the fixture. It states that the timing on this repository is taken after landing.
11. [behavior] `cargo clippy` and `cargo fmt --check` are clean, and `cargo test` passes across the workspace.

## Expected deliverables

- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-data/src/history.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## Out of scope

- Timing on this repository itself
- Other jobs of the records-history plan
- Changes to the records branch format beyond using the existing checkpoint record

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each required behavior; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests to see counts, tamper detection, and no-write on failure.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: CI read-only is enforced by the workflow permission and by code.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to incremental checking.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Timings are measured on the fixture and labeled.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
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

Run `cargo test`: a test shows an incremental run after a checkpoint judges only the commits and records added since it, with the reported counts equal to the new ones.
Run `cargo test`: a tampered checkpoint that names a commit the full replay fails is caught by the full replay, which exits with failure.
Run `cargo test`: a change in the check version forces a full replay, and the report says why.
Run `cargo test`: a failing attribution check writes no checkpoint, and the records-branch tip is unchanged.
Run `cargo test`: a passing `check --attribution` and a passing `land` each write a checkpoint naming the last passing main commit, the records tip judged against, and the check version.
Run `cargo test`: the full-replay mode ignores checkpoints and judges everything, and a CI or read-only mode never writes a checkpoint.
Run `cargo test`: each run output reports seconds taken and the number of commits and records judged.
Run `cargo test`: a test or bench on a synthetic repository of about 200 landings times a full replay and an incremental run and prints both numbers, marked as measured on that fixture.
.github/workflows/ci.yml keeps the token read-only. Pushes run the incremental check and the weekly schedule runs the full replay.
The CHANGELOG entry describes the feature and the fixture timings, marked as measured on the fixture. It states that the timing on this repository is taken after landing.
`cargo clippy` and `cargo fmt --check` are clean, and `cargo test` passes across the workspace.

## Report

End with the producer report as your structured output.
