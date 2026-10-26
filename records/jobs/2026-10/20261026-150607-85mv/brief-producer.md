# Job 20261026-150607-85mv: Move all records/ readers to the athena/records worktree, with fallback

## Request (verbatim)

> try again on 20261026-145955-s7e2

## Reading (accepted by the user)

Define the athena/records worktree path once in athena-data paths.rs beside repo_state_dir. Add load_current forms to Records::load and RecordSet::load, and move every live reader to them, including gh.rs land_pr and the introspect.rs directory walks. Readers fall back to in-tree records/ only when the worktree is absent. The site generator reads studies/ and stays unchanged. Audience: Athena operators and the verifier.

Assumptions:
- Goal: after the D111 cutover, main's HEAD has no records/, so every live reader must read the worktree of athena/records in Athena's state directory.
- Decided up front from the earlier critic findings: gh.rs land_pr (gh.rs:2513, qualified crate::store::Records::load) moves to load_current. The introspect.rs walks (~300/318/339/360) move to the records root. Neither goes on the allow-list.
- A first grep of the snapshot (Records::load, RecordSet::load, join("records"), read_records) found hits in about 44 files, many of them tests. The producer must enumerate non-test hits from a real scan and classify each as move, allow-listed with a reason, or out of scope. Every moved file is a deliverable.
- Allow-list candidates are limited to citation strings (steward.rs, work_decisions.rs), the gate invariant BOOTSTRAP_OUTCOME path, treeish judges that take a given ref (gate history, attribution, reconcile, content), the fallback itself, and paths.rs.
- The scan matches the Records::load( / RecordSet::load( suffix whatever the path prefix is, so crate::store::Records::load( is caught.
- The site generator xtask/src/site.rs reads studies/ and is unchanged. It is shown to build identically with records/ present and absent.
- The producer may touch other files the inventory finds, such as the gate reconcile, attribution, history and content modules. Only the ones that turn out to be live readers change.

## Acceptance criteria

1. [behavior] A test puts different records in the worktree and in the in-tree records/ and checks that each moved reader returns the worktree's records: Records::load_current, RecordSet::load_current, read_records, the xtask walltime reader, introspect walks, status, forecasting, scoring, retry carry, auditor, audit, and the other moved readers in the inventory. Run cargo test --locked.
2. [behavior] A real-git fixture whose HEAD has no records/ loads records through the worktree. A negative control shows the same assertion fails with the old HEAD-based load. Run cargo test --locked.
3. [behavior] A test calls land_pr on a checkout whose HEAD has no records/ and shows it resolves the job's verification and candidate from the worktree. Run cargo test --locked.
4. [behavior] With no worktree present, each moved reader falls back to records/ and returns the same result as before. athena status and a forecast succeed on a scratch checkout whose working tree has no records/, with output matching the records/ run. Run cargo test --locked.
5. [content] The worktree path is defined once, in crates/athena-data/src/paths.rs beside repo_state_dir. No non-test reader joins records/ itself outside paths.rs and the fallback.
6. [behavior] An xtask scan test named records_reader_scan fails on planted reads (join("records"), read_records(<repo dir>), Records::load(git, head), and crate::store::Records::load(git, head)). It passes on the real tree with an allow-list in which every entry has a reason. The scan was first run against the real tree to produce the call-site inventory. Run cargo test -p xtask --locked records_reader_scan and confirm a nonzero number of tests ran.
7. [behavior] xtask/src/site.rs is untouched, and the site output is identical with records/ present and absent. Run cargo run -p xtask -- site both ways and compare the outputs.
8. [behavior] cargo fmt --check, cargo clippy --all-targets and the full cargo test --locked pass, with existing tests unchanged.
9. [content] docs/decisions.md and CHANGELOG.md record the reader move, the fallback, the allow-list, and that the site generator is out of scope, citing D111.

## Expected deliverables

- crates/athena-data/src/paths.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
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
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/records_import.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/audit.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/history.rs (modify)
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

- xtask/src/site.rs and the study site generator, which reads studies/ and stays unchanged
- Writers of records/ and the cutover itself (job 20261025-001232-vesh)
- Citation strings in steward.rs and work_decisions.rs
- Changes to athena.toml, defaults.toml or CI workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture, fallback, land_pr and scan tests are added, and fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check readers returning worktree records, not that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that readers still work when HEAD has no records/.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared load_current path plus a scan covers all call sites, not only the ones found so far.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria use commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Allow-list reasons and decision docs are written for an outside reader.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: The scan test run must confirm a nonzero number of tests ran.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move; the site generator is excluded.
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

A test puts different records in the worktree and in the in-tree records/ and checks that each moved reader returns the worktree's records: Records::load_current, RecordSet::load_current, read_records, the xtask walltime reader, introspect walks, status, forecasting, scoring, retry carry, auditor, audit, and the other moved readers in the inventory. Run cargo test --locked.
A real-git fixture whose HEAD has no records/ loads records through the worktree. A negative control shows the same assertion fails with the old HEAD-based load. Run cargo test --locked.
A test calls land_pr on a checkout whose HEAD has no records/ and shows it resolves the job's verification and candidate from the worktree. Run cargo test --locked.
With no worktree present, each moved reader falls back to records/ and returns the same result as before. athena status and a forecast succeed on a scratch checkout whose working tree has no records/, with output matching the records/ run. Run cargo test --locked.
The worktree path is defined once, in crates/athena-data/src/paths.rs beside repo_state_dir. No non-test reader joins records/ itself outside paths.rs and the fallback.
An xtask scan test named records_reader_scan fails on planted reads (join("records"), read_records(<repo dir>), Records::load(git, head), and crate::store::Records::load(git, head)). It passes on the real tree with an allow-list in which every entry has a reason. The scan was first run against the real tree to produce the call-site inventory. Run cargo test -p xtask --locked records_reader_scan and confirm a nonzero number of tests ran.
xtask/src/site.rs is untouched, and the site output is identical with records/ present and absent. Run cargo run -p xtask -- site both ways and compare the outputs.
cargo fmt --check, cargo clippy --all-targets and the full cargo test --locked pass, with existing tests unchanged.
docs/decisions.md and CHANGELOG.md record the reader move, the fallback, the allow-list, and that the site generator is out of scope, citing D111.

## Report

End with the producer report as your structured output.
