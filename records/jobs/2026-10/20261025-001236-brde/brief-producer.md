# Job 20261025-001236-brde: Point all records/ readers at a worktree of athena/records

## Request (verbatim)

> Priority: high
> Build job 7, part 2, of the records-history plan (docs/design/records-history.md section 13, decision D111): move every reader of records/ to the athena/records branch. Depends on 20261025-001232-vesh (the cutover). Keep a worktree of athena/records at a fixed path in Athena's state directory and point the readers there.
> Acceptance:
> - A test shows each reader (Records::load, RecordSet::load, read_records, the walltime reader, status, forecasting, scoring, retry carry, audit, introspection, the study site generator) returns the same result from the branch as from records/ at C0.
> - athena status and a forecast work on a checkout whose working tree has no records/.
> Files: crates/athena-core, crates/athena-data, crates/athena-gate, crates/athena-cli, xtask

## Reading (accepted by the user)

Add a managed worktree of the athena/records branch at a fixed path in Athena's state directory, and route every reader of records/ (Records::load, RecordSet::load, read_records, walltime, status, forecasting, scoring, retry carry, audit, introspection, study site generator) to it. Audience: Athena operators and the verifier; a checkout with no records/ must still run status and forecast.

Assumptions:
- The cutover job (20261025-001232-vesh) has landed, so athena/records exists with C0 and the import commit.
- The worktree is created or refreshed (detached at the branch tip) on demand by a single helper in athena-core, and all readers take their records root from it.
- When the branch is absent (before cutover), readers fall back to records/ in the working tree so existing behavior is unchanged.
- Equivalence is tested in a scratch repository: import records/ at C0, then compare each reader's output from the branch worktree with its output from records/.
- The wiring edits live mainly in the listed crates; docs/decisions.md and CHANGELOG.md get short entries.

## Acceptance criteria

1. [behavior] A test in the workspace shows each reader (Records::load, RecordSet::load, read_records, the walltime reader, status, forecasting, scoring, retry carry, audit, introspection, study site generator) returns the same result from the athena/records worktree as from records/ at C0. Run cargo test.
2. [behavior] A test runs athena status and a forecast on a scratch checkout whose working tree has no records/ and both succeed, with output matching the records/ run. Run cargo test.
3. [content] The worktree path is a single fixed location under Athena's state directory, defined in one place, and no reader still hard-codes a records/ path in the working tree except the pre-cutover fallback.
4. [behavior] Before the branch exists, existing tests pass unchanged: cargo test, cargo clippy --all-targets, and cargo fmt --check are clean.
5. [content] docs/decisions.md and CHANGELOG.md record the reader move and the fallback, citing D111.

## Expected deliverables

- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/records_import.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/audit.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- xtask/src/main.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Ending commit-message trailers
- Removing records/ from the main branch
- Changing the cutover, importer, or history/attribution checks
- Writers of records (only readers move)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Equivalence and no-records/ tests; cargo test and clippy clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria compare reader outputs, not existence of wiring.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and decision entries written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Single helper for the worktree path makes the rule structural.
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

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A test in the workspace shows each reader (Records::load, RecordSet::load, read_records, the walltime reader, status, forecasting, scoring, retry carry, audit, introspection, study site generator) returns the same result from the athena/records worktree as from records/ at C0. Run cargo test.
A test runs athena status and a forecast on a scratch checkout whose working tree has no records/ and both succeed, with output matching the records/ run. Run cargo test.
The worktree path is a single fixed location under Athena's state directory, defined in one place, and no reader still hard-codes a records/ path in the working tree except the pre-cutover fallback.
Before the branch exists, existing tests pass unchanged: cargo test, cargo clippy --all-targets, and cargo fmt --check are clean.
docs/decisions.md and CHANGELOG.md record the reader move and the fallback, citing D111.

## Report

End with the producer report as your structured output.
