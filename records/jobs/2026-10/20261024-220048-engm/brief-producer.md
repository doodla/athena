# Job 20261024-220048-engm: Add incremental attribution checking with checkpoints and full replay

## Request (verbatim)

> Build job 6 of the records-history plan (docs/design/records-history.md section 9, decision D111): incremental checking. After a passing attribution check, write a checkpoint record to athena/records (the record type exists in athena-data history.rs) naming the last passing main commit, the records-branch tip it was judged against, and the check's version; a later check judges only main's commits and records added since the newest checkpoint. A full replay mode ignores checkpoints, judges everything from the start, and fails if it disagrees with the newest checkpoint; the CI attribution job runs incrementally on pushes and as a full replay on its weekly schedule. Report per run the seconds taken and the number of commits and records judged. Tests: the incremental run judges only the new commits; a tampered checkpoint (one that names a commit the full replay fails) is caught by the replay; a check-version change forces a full replay. Measure on this repository: time a full replay and an incremental run one landing behind, and put both numbers in the CHANGELOG entry, marked measured.

## Reading (accepted by the user)

Extend `athena check --attribution` to read the newest checkpoint and judge only newer main commits and records, add `--full-replay` that ignores checkpoints and fails on disagreement, and write checkpoints from local runs (check pass, land). CI stays read-only: incremental on push, full replay on schedule. Audience: maintainers and CI.

Assumptions:
- Checkpoint record type already exists in athena-data history.rs and is reused; fields map to main sha, records tip, verifier_version.
- Per the user's reply, CI is read-only and never writes checkpoints; local runs write them and they are pushed with the records writer.
- Timings are measured on a synthetic ~200-landing fixture in a test or small bench, since the producer has no real records; real-repo timing is deferred.
- Flag name is --full-replay on the existing `check --attribution` command.

## Acceptance criteria

1. [behavior] Run `cargo test`: passes, including a test where an incremental run after a checkpoint judges only the new commits (judged count equals the number of new commits and is less than a full replay's).
2. [behavior] Run `cargo test`: a test with a tampered checkpoint (naming a main commit the full replay fails) shows the full replay reports a finding and fails because it disagrees with the newest checkpoint.
3. [behavior] Run `cargo test`: a test shows a checkpoint with a different check version forces a full replay judging everything from the start.
4. [behavior] Run `cargo test`: a passing local check writes a checkpoint record to athena/records with main sha, records tip judged against, and check version; a failing check writes none; `athena land` writes one after the landing record; a read-only (CI) mode never writes one.
5. [behavior] Run `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
6. [behavior] Run `cargo run -- check --help`: documents incremental and --full-replay modes and the read-only/no-write behavior.
7. [content] Every run's text and --json output reports seconds taken, commits judged, and records judged.
8. [content] .github/workflows/ci.yml keeps permissions contents: read; the attribution job runs incrementally on push and as a full replay on the schedule event, and writes no checkpoint.
9. [content] CHANGELOG.md has an entry with full-replay and incremental (one landing behind) timings from a synthetic fixture (about 200 landings), each marked measured-on-fixture, stating that the real-repository timing is taken after landing; the numbers come from an actual test or bench run.
10. [content] Checkpoint is skipped as a trust source in full replay, and mismatch is reported as a finding naming the checkpoint.

## Expected deliverables

- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-data/src/history.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## Out of scope

- Timing the real repository (done after landing)
- Giving CI write access or pushing checkpoints from CI
- Other jobs of the records-history plan

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for incremental, tampered checkpoint, version change; clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check judged counts and failure behavior via tests.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: CHANGELOG numbers labeled measured-on-fixture from real runs.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: CI read-only enforced by workflow permissions and a no-write mode.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the attribution check.
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

Run `cargo test`: passes, including a test where an incremental run after a checkpoint judges only the new commits (judged count equals the number of new commits and is less than a full replay's).
Run `cargo test`: a test with a tampered checkpoint (naming a main commit the full replay fails) shows the full replay reports a finding and fails because it disagrees with the newest checkpoint.
Run `cargo test`: a test shows a checkpoint with a different check version forces a full replay judging everything from the start.
Run `cargo test`: a passing local check writes a checkpoint record to athena/records with main sha, records tip judged against, and check version; a failing check writes none; `athena land` writes one after the landing record; a read-only (CI) mode never writes one.
Run `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
Run `cargo run -- check --help`: documents incremental and --full-replay modes and the read-only/no-write behavior.
Every run's text and --json output reports seconds taken, commits judged, and records judged.
.github/workflows/ci.yml keeps permissions contents: read; the attribution job runs incrementally on push and as a full replay on the schedule event, and writes no checkpoint.
CHANGELOG.md has an entry with full-replay and incremental (one landing behind) timings from a synthetic fixture (about 200 landings), each marked measured-on-fixture, stating that the real-repository timing is taken after landing; the numbers come from an actual test or bench run.
Checkpoint is skipped as a trust source in full replay, and mismatch is reported as a finding naming the checkpoint.

## Report

End with the producer report as your structured output.
