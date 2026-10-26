# Job 20261026-170042-cgq9: Move records/ readers to athena/records worktree with per-reader tests

## Request (verbatim)

> try again on 20261026-155343-tvfc

## Reading (accepted by the user)

Retry of job tvfc: move every records/ reader to the athena/records worktree with in-tree fallback, with a worktree test and a fallback test per reader. Test the xtask walltime reader inside xtask, test land_pr in gh.rs, and use distinct job ids so status, audit and land_pr tests show which record was read. Audience: Athena operators and the verifier.

Assumptions:
- Goal: complete D111 job 7 part 2 (readers of records/ move to the athena/records worktree with in-tree fallback); audience is Athena operators and the verifier.
- Production move is as in the previously accepted reading; the retry's key change is test placement: walltime tests in xtask, land_pr tests in gh.rs.
- I could not inspect the repo in detail; deliverable list follows the earlier accepted list.
- site.rs reads studies/, not records/, and stays untouched.
- Depends on the cutover job 20261025-001232-vesh having landed.

## Acceptance criteria

1. [behavior] In crates/athena-cli/tests/records_branch_readers.rs, each athena-cli-reachable reader (Records::load_current, RecordSet::load_current, read_records, introspect walks, scoring, forecasting, retry carry, auditor, proposals, backtest) has a named worktree test planting different records in the worktree and in-tree and asserting the worktree's are returned. Run cargo test --locked.
2. [behavior] Each of those readers has a named fallback test with no worktree asserting in-tree records/ is returned, equal to the pre-change result. Run cargo test --locked.
3. [behavior] xtask's own tests contain worktree_walltime and fallback_walltime (not in the athena-cli test file) plus a linked-worktree test finding the records worktree via git's common dir. Run cargo test -p xtask --locked and confirm a nonzero number of tests ran.
4. [behavior] land_pr tests (in gh.rs, where Rig/FakeHost are reachable) use distinct job ids in worktree and in-tree records: resolve succeeds for the worktree-only id and fails with 'unknown job' for the in-tree-only id; under fallback the reverse. Run cargo test --locked land_pr.
5. [behavior] A test runs the built athena binary's status, audit and a forecast on a scratch checkout whose working tree has no records/; worktree and in-tree records have distinct job ids, and status and audit output contain the worktree's id and not the in-tree one; all succeed. Run cargo test --locked.
6. [behavior] Tests create the records worktree under a temporary state directory; scoring's use of the worktree regardless of the head passed is documented in docs/decisions.md and tested.
7. [behavior] xtask records_reader_scan still fails on planted reads, including crate::store::Records::load( paths, ignores citation strings in steward.rs and work_decisions, and allow-list entries are narrowed to call sites. Run cargo test -p xtask --locked records_reader_scan and confirm a nonzero number of tests ran.
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test --locked pass; git diff shows xtask/src/site.rs unchanged.
9. [content] The change description lists every reader found by grep, maps each to its worktree and fallback test names and the observed marker, states the walltime (in xtask) and land_pr (in gh.rs) placements, says common-dir derivation is new code and not a repair, and cites commands actually run.
10. [content] CHANGELOG.md records the reader move, the fallback, the allow-list, and that the site generator is out of scope, citing D111.

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

- Rerooting the study site generator xtask/src/site.rs
- Changing the cutover itself (job vesh)
- Scanning citation strings in steward.rs or work_decisions

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Per-reader tests; fmt, clippy and full test criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Binary status/audit/forecast exercised on a scratch checkout.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests plant distinct records and assert the worktree's are returned.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: records_reader_scan enforces the reader rule with a planted-read test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Change description and docs written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check readers returning worktree data, not wiring.
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

In crates/athena-cli/tests/records_branch_readers.rs, each athena-cli-reachable reader (Records::load_current, RecordSet::load_current, read_records, introspect walks, scoring, forecasting, retry carry, auditor, proposals, backtest) has a named worktree test planting different records in the worktree and in-tree and asserting the worktree's are returned. Run cargo test --locked.
Each of those readers has a named fallback test with no worktree asserting in-tree records/ is returned, equal to the pre-change result. Run cargo test --locked.
xtask's own tests contain worktree_walltime and fallback_walltime (not in the athena-cli test file) plus a linked-worktree test finding the records worktree via git's common dir. Run cargo test -p xtask --locked and confirm a nonzero number of tests ran.
land_pr tests (in gh.rs, where Rig/FakeHost are reachable) use distinct job ids in worktree and in-tree records: resolve succeeds for the worktree-only id and fails with 'unknown job' for the in-tree-only id; under fallback the reverse. Run cargo test --locked land_pr.
A test runs the built athena binary's status, audit and a forecast on a scratch checkout whose working tree has no records/; worktree and in-tree records have distinct job ids, and status and audit output contain the worktree's id and not the in-tree one; all succeed. Run cargo test --locked.
Tests create the records worktree under a temporary state directory; scoring's use of the worktree regardless of the head passed is documented in docs/decisions.md and tested.
xtask records_reader_scan still fails on planted reads, including crate::store::Records::load( paths, ignores citation strings in steward.rs and work_decisions, and allow-list entries are narrowed to call sites. Run cargo test -p xtask --locked records_reader_scan and confirm a nonzero number of tests ran.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test --locked pass; git diff shows xtask/src/site.rs unchanged.
The change description lists every reader found by grep, maps each to its worktree and fallback test names and the observed marker, states the walltime (in xtask) and land_pr (in gh.rs) placements, says common-dir derivation is new code and not a repair, and cites commands actually run.
CHANGELOG.md records the reader move, the fallback, the allow-list, and that the site generator is out of scope, citing D111.

## Report

End with the producer report as your structured output.
