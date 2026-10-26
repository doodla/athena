# Job 20261026-145955-s7e2: Route all records/ readers to the athena/records worktree

## Request (verbatim)

> try again on 20261025-001236-brde

## Reading (accepted by the user)

Define the records worktree path once in athena-data paths.rs beside repo_state_dir; add load_current forms to Records::load and RecordSet::load and move the live readers to them, falling back to in-tree records/ only when the worktree is absent. The site generator reads studies/, so it stays unchanged and is shown to still build identically. Audience: Athena operators and the verifier.

Assumptions:
- The user's reply governs: site generator untouched, scan targets only real readers, and the scan run command executes the scan test.
- The worktree of athena/records is created or located under Athena's state directory via the path in paths.rs; fallback to in-tree records/ applies only when it is absent.
- I did not re-open the repository; the file list derives from the earlier failed plan's inventory and the request's Files line, and the implementer must re-enumerate call sites by search.

## Acceptance criteria

1. [behavior] A test puts different records in the worktree and in the in-tree records/ and checks that each moved reader (Records::load_current, RecordSet::load_current, read_records, xtask walltime, status, auditor, reading, scope_guard, run, manage, proposals, audience, review, hestia, steward, code_health, work_decisions, backtest, drain) returns the worktree's records. Run cargo test --locked.
2. [behavior] With no worktree present, each moved reader falls back to records/ and returns the same result as before. athena status and a forecast succeed on a scratch checkout whose working tree has no records/, reading only the worktree, with output matching the records/ run. Run cargo test --locked.
3. [content] The worktree path is defined once, in crates/athena-data/src/paths.rs next to repo_state_dir. No non-test reader joins records/ itself outside paths.rs and the fallback.
4. [behavior] An xtask scan test named records_reader_scan fails on a planted direct read of records/ in non-test code and passes on the real tree. It matches reader shapes (join("records"), read_records(<repo dir>), Records/RecordSet::load() only, so citation strings in steward.rs and work_decisions.rs and the gate invariant.rs BOOTSTRAP_OUTCOME path are not flagged. Run cargo test -p xtask --locked records_reader_scan, and confirm it executes (nonzero tests run).
5. [behavior] The study site generator is unchanged: xtask/src/site.rs is untouched and not rooted at the worktree, and its output is identical when built with records/ present and with it absent. Run cargo run -p xtask -- site both ways and compare outputs.
6. [behavior] cargo fmt --check, cargo clippy --all-targets and the full cargo test --locked pass, including existing tests unchanged before the branch exists.
7. [content] docs/decisions.md and CHANGELOG.md record the reader move, the fallback, and that the site generator is out of scope, citing D111.

## Expected deliverables

- crates/athena-data/src/paths.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-core/src/audience.rs (modify)
- crates/athena-core/src/review.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/records_import.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/audit.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/records_branch_readers.rs (create)
- xtask/src/main.rs (modify)
- xtask/src/walltime.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Rerooting the study site generator (xtask/src/site.rs), which reads studies/firstmate-work-handling
- The cutover job 20261025-001232-vesh itself
- Creating or populating the athena/records branch
- Flagging citation strings such as 'records/jobs' labels in steward.rs and work_decisions.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests plus clean fmt and clippy are required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria compare reader outputs from worktree vs in-tree records.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A scan test enforces the single-path rule.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One path definition and load_current forms rather than per-caller patches.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Status and forecast must work on a checkout without records/.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decisions and changelog document the move for outside readers.
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

A test puts different records in the worktree and in the in-tree records/ and checks that each moved reader (Records::load_current, RecordSet::load_current, read_records, xtask walltime, status, auditor, reading, scope_guard, run, manage, proposals, audience, review, hestia, steward, code_health, work_decisions, backtest, drain) returns the worktree's records. Run cargo test --locked.
With no worktree present, each moved reader falls back to records/ and returns the same result as before. athena status and a forecast succeed on a scratch checkout whose working tree has no records/, reading only the worktree, with output matching the records/ run. Run cargo test --locked.
The worktree path is defined once, in crates/athena-data/src/paths.rs next to repo_state_dir. No non-test reader joins records/ itself outside paths.rs and the fallback.
An xtask scan test named records_reader_scan fails on a planted direct read of records/ in non-test code and passes on the real tree. It matches reader shapes (join("records"), read_records(<repo dir>), Records/RecordSet::load() only, so citation strings in steward.rs and work_decisions.rs and the gate invariant.rs BOOTSTRAP_OUTCOME path are not flagged. Run cargo test -p xtask --locked records_reader_scan, and confirm it executes (nonzero tests run).
The study site generator is unchanged: xtask/src/site.rs is untouched and not rooted at the worktree, and its output is identical when built with records/ present and with it absent. Run cargo run -p xtask -- site both ways and compare outputs.
cargo fmt --check, cargo clippy --all-targets and the full cargo test --locked pass, including existing tests unchanged before the branch exists.
docs/decisions.md and CHANGELOG.md record the reader move, the fallback, and that the site generator is out of scope, citing D111.

## Report

End with the producer report as your structured output.
