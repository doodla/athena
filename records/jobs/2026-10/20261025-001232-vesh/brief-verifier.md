# Job 20261025-001232-vesh: verify "Add records-history cutover with import and catch-up at C0"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Scratch-repo test in gh.rs driving the real Athena::land_pr path with a bare remote and the existing fake PrHost. Base commit's athena.toml has no cutover_job; the PR head commit sets cutover_job to the job id. The scratch repo commits a verification record for that job id, so the landing's commits name it. The sink's cutover_job is resolved by cutover_job_for from the landed commit, not passed in by hand. A second case shows cutover_job_for returns None for the base and Some(job) for the head. Run: `cargo test cutover_land`. Observes: On the remote athena/records: the landing record, an import commit with import-manifest.json for which verify_cutover_import passes, and legacy-exempt.json with 7 entries each with a non-empty reason. cutover_commit(git, job) returns the landing's host_commit and equals the manifest's C0. cutover_commit returns None for a landing of another job.
  Fails without the behavior: Without the land wiring (or if the config were read from the pre-merge HEAD) the branch holds only intent and landing; no manifest, no legacy-exempt.json, so the assertions fail.
- Criterion 2: Test on a remote whose records branch holds intent and landing for the cutover job but no import, and whose main tip config has cutover_job. Run Athena::reconcile (the step land and status call first), count commits on the remote records branch, then run it again. Run: `cargo test catch_up`. Observes: First reconcile adds exactly two commits (import with manifest, then legacy-exempt.json with 7 entries) and the tip moves, so it is a real write rather than an empty no-op. Second reconcile leaves the records tip identical. A case with only legacy-exempt.json missing writes just that file.
  Fails without the behavior: Without the catch-up call in try_reconcile the tip is unchanged and the manifest assertion fails; without the presence checks the second run adds commits or errors.
- Criterion 3: history.rs scratch-repo test reusing legacy.rs/attribution test helpers. A pre-C0 commit breaks the old rules; C0 sets cutover_job; records branch has the landing, import manifest naming C0 and legacy-exempt.json; a post-C0 commit has no trailer but intent and landing. Variants: pre-C0 commit not exempt gives a violation naming it; post-C0 commit with its landing gives none; post-C0 commit with no landing gives an attribution-derived violation. Run: `cargo test history::tests::cutover`. Observes: A pre-C0 commit judged by old trailer rules; the later records-only trailerless commit passes; the unattributed one is flagged.
  Fails without the behavior: Without the router the trailerless post-C0 commit fails the old rules; without the attribution hand-off an unattributed post-C0 commit passes.
- Criterion 4: Run the entire suite with existing history, legacy, attribution and lifecycle tests unedited. Add new tests: cutover_job set but no records branch; branch with no landing for the job; landing present but no manifest (pending). Each asserts the report equals today's check_history_with_exempt(None) result (the pending case also carries the visible note, not a violation). Run: `cargo test`. Observes: All pre-existing tests pass with unmodified expectations; the fallback reports are identical.
  Fails without the behavior: If the router engaged without C0 shown, existing tests with no records branch would gain attribution findings and fail.
- Criterion 5: config.rs tests parse the repo's athena.toml and small tomls with and without the field; inspect ci.yml diff with git diff. Run: `cargo test cutover_job`. Observes: cutover_job == Some("20261025-001232-vesh") from athena.toml, None when absent. ci.yml diff keeps the name 'the full history', and the fetch step precedes the history check step, with the persist-credentials comment.
  Fails without the behavior: Without the field, deny_unknown_fields rejects athena.toml, so the parse test and all config-reading tests fail.
- Criterion 6: Review the diff. Run: `git diff -- docs/decisions.md CHANGELOG.md`. Observes: decisions.md has a new entry covering C0 (landing host commit), which config commit supplies cutover_job, import at C0, the legacy-exempt record from config at C0, catch-up reconcile, behavior before C0 including the manifest-keyed activation and pending note, and trailers still written. CHANGELOG.md has an entry.
- Criterion 7: Run clippy, then cargo fmt --all --check, then cargo test --locked. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: No warnings, no format diff, all tests pass.
  Fails without the behavior: Unformatted code, unused imports or lints would fail these gates.

Planned files:

- athena.toml (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/records_import.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- .github/workflows/ci.yml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base 49f6c9b635c9 → candidate 13b19d9a571c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml                 |  11 +-
 CHANGELOG.md                             |  10 +
 athena.toml                              |   5 +
 crates/athena-cli/src/main.rs            |   3 +
 crates/athena-core/src/gh.rs             | 381 ++++++++++++++++++++++++++++-
 crates/athena-core/src/manage.rs         |   3 +
 crates/athena-core/src/reconcile.rs      | 206 +++++++++++++++-
 crates/athena-core/src/records_import.rs |  80 ++++--
 crates/athena-data/src/config.rs         |  21 ++
 crates/athena-gate/src/attribution.rs    |  83 +++++++
 crates/athena-gate/src/history.rs        | 402 ++++++++++++++++++++++++++++++-
 docs/decisions.md                        |  71 ++++++
 12 files changed, 1252 insertions(+), 24 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1180 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
