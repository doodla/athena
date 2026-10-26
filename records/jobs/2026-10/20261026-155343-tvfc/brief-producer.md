# Job 20261026-155343-tvfc: Add per-reader worktree/fallback tests for records branch move (D111)

## Request (verbatim)

> try again on 20261026-150607-85mv

## Reading (accepted by the user)

Retry of job 20261026-150607-85mv: the production move of records/ readers to the athena/records worktree is kept as in the accepted reading; the rejection was for missing per-reader tests and a missing forecast run. Audience: Athena operators and the verifier. The retry adds a test per reader with the worktree present and with it absent, a status and forecast scratch-checkout test, and fixes the minor findings.

Assumptions:
- The earlier attempt's production changes are not in the repository, so the producer redoes them per the accepted reading before adding tests.
- The forecast test can run offline against fixtures.
- The site generator stays unchanged and out of scope.

## Acceptance criteria

1. [behavior] In crates/athena-cli/tests/records_branch_readers.rs, for each moved reader (Records::load_current, RecordSet::load_current, read_records, xtask walltime reader, introspect walks, status, forecasting, scoring, retry carry, auditor, audit, proposals, backtest, land_pr) a named test puts different records in the worktree and in in-tree records/ and asserts the worktree's records are returned. Run cargo test --locked.
2. [behavior] For each of the same readers, a named test with no worktree asserts it falls back to in-tree records/ and returns the same result as before. Run cargo test --locked.
3. [behavior] A test runs the athena binary's status and a forecast on a scratch checkout whose working tree has no records/, and asserts both succeed with output matching the records/ run. Run cargo test --locked.
4. [content] The change description lists every reader found by grep of the code and maps each to its test names, and cites the commands actually run.
5. [behavior] backtest.rs and xtask walltime.rs find the git common dir via git's common-dir, not <root>/.git, and a test from a linked worktree checkout finds the records worktree. Run cargo test --locked.
6. [content] Tests create the records worktree under a temporary state directory, not the real state root; the scoring closures' use of the worktree regardless of the passed head is documented in docs/decisions.md and tested.
7. [behavior] The xtask records_reader_scan still fails on planted reads, and allow-list entries are narrowed to call sites where practical. Run cargo test -p xtask --locked records_reader_scan and confirm a nonzero number of tests ran.
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and the full cargo test --locked pass; xtask/src/site.rs is untouched.
9. [content] CHANGELOG.md records the reader move, the fallback, the allow-list and that the site generator is out of scope, citing D111.

## Expected deliverables

- crates/athena-cli/tests/records_branch_readers.rs (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/paths.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- xtask/src/walltime.rs (modify)
- xtask/src/main.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-gate/src/audit.rs (modify)

## Out of scope

- Rerooting xtask/src/site.rs
- Changing existing tests
- Changing governance files or athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Full cargo test, fmt, clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require tests that show worktree versus in-tree results per reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check readers return worktree records and status/forecast succeed.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Common-dir derivation fixed in all readers, not just one.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decisions and changelog written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261026-150607-85mv raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-cli/tests/records_branch_readers.rs): Criterion 1's named readers (forecasting, scoring, retry carry, auditor, audit, introspect, proposals, backtest) have no worktree-versus-in-tree test The new file tests only the shared loaders, read_records, status/trace/mistakes and land_pr; walltime is tested in xtask. The callers in auditor.rs:300, reading.rs:701 (retry), manage.rs:194 and run.rs:2392 (scoring), introspect.rs:1246/1283/1314, proposals.rs:556 and backtest.rs:510 are never exercised with a worktree.
> major (crates/athena-cli/tests/records_branch_readers.rs): Criterion 4's forecast run on a scratch checkout is missing Only `athena status` is run as a binary. No test or command produces a forecast with records/ absent from the working tree.

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

In crates/athena-cli/tests/records_branch_readers.rs, for each moved reader (Records::load_current, RecordSet::load_current, read_records, xtask walltime reader, introspect walks, status, forecasting, scoring, retry carry, auditor, audit, proposals, backtest, land_pr) a named test puts different records in the worktree and in in-tree records/ and asserts the worktree's records are returned. Run cargo test --locked.
For each of the same readers, a named test with no worktree asserts it falls back to in-tree records/ and returns the same result as before. Run cargo test --locked.
A test runs the athena binary's status and a forecast on a scratch checkout whose working tree has no records/, and asserts both succeed with output matching the records/ run. Run cargo test --locked.
The change description lists every reader found by grep of the code and maps each to its test names, and cites the commands actually run.
backtest.rs and xtask walltime.rs find the git common dir via git's common-dir, not <root>/.git, and a test from a linked worktree checkout finds the records worktree. Run cargo test --locked.
Tests create the records worktree under a temporary state directory, not the real state root; the scoring closures' use of the worktree regardless of the passed head is documented in docs/decisions.md and tested.
The xtask records_reader_scan still fails on planted reads, and allow-list entries are narrowed to call sites where practical. Run cargo test -p xtask --locked records_reader_scan and confirm a nonzero number of tests ran.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and the full cargo test --locked pass; xtask/src/site.rs is untouched.
CHANGELOG.md records the reader move, the fallback, the allow-list and that the site generator is out of scope, citing D111.

## Report

End with the producer report as your structured output.
