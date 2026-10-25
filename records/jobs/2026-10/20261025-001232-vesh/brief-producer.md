# Job 20261025-001232-vesh: Add records-history cutover with import and catch-up at C0

## Request (verbatim)

> Priority: high
> Build job 7, part 1, of the records-history plan (docs/design/records-history.md sections 4 and 10, decision D111): the cutover, decided on 2026-10-25 as follows. athena.toml's [history] gains cutover_job naming this job (its own id); C0 is the host commit of that job's landing record on athena/records. When athena land --pr lands the job that cutover_job names, after the landing record it runs the importer (crates/athena-core records_import) at C0 and writes the legacy-exempt record built from [history] exempt (six entries plus 02837e0, with their reasons) to athena/records. athena check --history judges commits up to and including C0 as today and hands commits after C0 to the attribution check (crates/athena-gate attribution.rs); before the records branch shows C0, it behaves exactly as today. The required CI job keeps its name, 'the full history', and fetches athena/records first. Commit-message trailers are still written; ending them is a later job.
> Acceptance:
> - A scratch-repository test lands a job named by cutover_job and shows the import commit (with import-manifest.json whose self-check passes) and the legacy-exempt record on the records branch, and C0 resolved from the landing record.
> - A test shows the history check judging a commit before C0 by the old rules and a commit after C0 only by records, so a later records-only commit with no trailer passes.
> - With no cutover_job set, or before its landing record exists, every existing history-check test passes unchanged.
> - athena.toml and .github/workflows/ci.yml are governance deliverables; docs/decisions.md records the design.
> Files: athena.toml, crates/athena-core/src/gh.rs, crates/athena-core/src/records_import.rs, crates/athena-gate/src/history.rs, crates/athena-gate/src/attribution.rs, crates/athena-data/src/config.rs, .github/workflows/ci.yml, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Add [history] cutover_job; on landing that job (and via a catch-up in the reconcile step of land and status, idempotent), run the importer at C0 and write the legacy-exempt record. The history check judges commits up to C0 by old rules and later ones by attribution, unchanged before C0 exists; CI keeps its name and fetches athena/records first.

Assumptions:
- Goal: make the records-history cutover work so maintainers and CI judge history by records after C0; audience is the Athena maintainer and the verifier.
- The job's own id is taken from the job context (the landing job id); cutover_job is set to it in athena.toml.
- The existing importer in records_import.rs and the attribution check in attribution.rs are reused; only wiring and idempotence markers are added.
- Idempotence is detected by presence of an import commit/manifest on athena/records.
- Trailers continue to be written.

## Acceptance criteria

1. [behavior] Run `cargo test`: a scratch-repository test lands the job named by [history] cutover_job via the land path and shows the records branch has the landing record, an import commit with import-manifest.json whose self-check passes, and the legacy-exempt record (six exempt entries plus 02837e0, each with a reason), with C0 resolved from the landing record's host commit.
2. [behavior] Run `cargo test`: a catch-up test shows that when the cutover job has a landing record on athena/records but no import commit, the reconcile step at the start of land and status runs the import at C0 and writes the legacy-exempt record; a second reconcile writes nothing (idempotent).
3. [behavior] Run `cargo test`: a history test shows a commit at or before C0 judged by the old trailer rules and a commit after C0 judged only by the attribution check against records, so a later records-only commit with no trailer passes.
4. [behavior] Run `cargo test`: with no cutover_job set, or before its landing record exists on the records branch, all pre-existing history-check tests pass with their expectations unedited.
5. [content] athena.toml [history] sets cutover_job to this job's own id; crates/athena-data/src/config.rs parses it as optional; .github/workflows/ci.yml keeps the job name 'the full history' and fetches athena/records before the history check.
6. [content] docs/decisions.md records the cutover design (C0 definition, import at C0, legacy-exempt record, catch-up reconcile, behavior before C0, trailers still written) and CHANGELOG.md has an entry.
7. [behavior] Run `cargo clippy` and `cargo fmt --check`: both clean; `cargo test` passes overall.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/records_import.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-data/src/config.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Ending commit-message trailers (a later job).
- Renaming the required CI job.
- Changing records/ contents by hand.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for land, catch-up, history; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests exercising the behavior.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decisions and changelog written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: CI and config enforce the cutover.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Catch-up test asserts the import actually wrote commits, not just an empty no-op.
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

Run `cargo test`: a scratch-repository test lands the job named by [history] cutover_job via the land path and shows the records branch has the landing record, an import commit with import-manifest.json whose self-check passes, and the legacy-exempt record (six exempt entries plus 02837e0, each with a reason), with C0 resolved from the landing record's host commit.
Run `cargo test`: a catch-up test shows that when the cutover job has a landing record on athena/records but no import commit, the reconcile step at the start of land and status runs the import at C0 and writes the legacy-exempt record; a second reconcile writes nothing (idempotent).
Run `cargo test`: a history test shows a commit at or before C0 judged by the old trailer rules and a commit after C0 judged only by the attribution check against records, so a later records-only commit with no trailer passes.
Run `cargo test`: with no cutover_job set, or before its landing record exists on the records branch, all pre-existing history-check tests pass with their expectations unedited.
athena.toml [history] sets cutover_job to this job's own id; crates/athena-data/src/config.rs parses it as optional; .github/workflows/ci.yml keeps the job name 'the full history' and fetches athena/records before the history check.
docs/decisions.md records the cutover design (C0 definition, import at C0, legacy-exempt record, catch-up reconcile, behavior before C0, trailers still written) and CHANGELOG.md has an entry.
Run `cargo clippy` and `cargo fmt --check`: both clean; `cargo test` passes overall.

## Report

End with the producer report as your structured output.
