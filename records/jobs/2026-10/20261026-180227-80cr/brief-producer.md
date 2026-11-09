# Job 20261026-180227-80cr: Add Records::load_current and move status, forecast, land_pr to it

## Request (verbatim)

> Records-history step 7b, first slice (D111). Define the athena/records worktree path once in crates/athena-data/src/paths.rs beside repo_state_dir, add Records::load_current and RecordSet::load_current that read the worktree and fall back to in-tree records/ only when the worktree is absent, and move only these readers to them: athena status, the forecast, and Athena::land_pr's job lookup. Every other reader stays as it is; list them by file and function in docs/design/records-history.md as named follow-ups for later slices. No scan test in this slice.
> Acceptance:
> - A test with distinct job ids in the worktree and in in-tree records/ shows status, the forecast and land_pr's lookup use the worktree's records, and with no worktree they use records/ as before.
> - docs/design/records-history.md lists every remaining reader by file and function as a follow-up.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-data/src/paths.rs, crates/athena-core/src/store.rs, crates/athena-gate/src/records.rs, crates/athena-core/src/status.rs, crates/athena-core/src/gh.rs, docs/design/records-history.md, CHANGELOG.md

## Reading (accepted by the user)

First D111 step-7b slice. It defines the athena/records worktree path once in paths.rs and adds Records::load_current and RecordSet::load_current, which read the worktree and fall back to in-tree records/ only when the worktree is absent. Only `athena status`, the forecast and land_pr's job lookup move to them. The design doc lists every other reader by file and function as a follow-up. Audience: the maintainer and the verifier.

Assumptions:
- Code survey: read crates/athena-data/src/paths.rs (repo_state_dir at line 43, so the worktree path helper goes beside it), crates/athena-core/src/store.rs (Records::load at line 349 and load_dir at 356, built from RecordSet::load plus landing_files), crates/athena-core/src/status.rs (status() at line 985 calls Records::load and athena_gate::records::RecordSet::load; the forecast code is also in status.rs; mistake_report at 906 and trace at 1119 also call Records::load), crates/athena-core/src/gh/landing.rs (Athena::land_pr at line 832 calls crate::store::Records::load at line 846). The new loaders check files through check_record_file, as the existing loaders do. No record or data file is added under crates/, so no AnyRecord or round-trip change is needed. Resolution order: use the worktree when its directory exists, else read records/ from the treeish as today. The worktree directory is probed first and records/ only on absence, not on an empty or invalid worktree.
- Existing code: crates/athena-data/src/paths.rs (repo_state_dir), crates/athena-core/src/store.rs (Records::load, load_dir, from_parts), crates/athena-gate/src/records.rs (RecordSet::load) are reused and extended. docs/design/records-history.md section 7 (lines 455-511) already names the worktree under the state directory, for example <state>/records-worktree, and lists the readers per seam.
- Environment: no network, CI or push is involved. Tests build a temporary worktree directory and a temporary git repository locally, so the test reads only local files and git. No workflow file changes.
- The forecast reader is the status.rs code that computes the forecast, which I located only by grep. The producer must find the exact function and move only it. Reading status() also loads Records at line 988, so status and forecast may share one load.
- Term worktree: a directory holding a checkout of the athena/records branch at a fixed path under the state directory. In-tree records/ is the records/ directory in the main checkout.
- Governance files (athena.toml, defaults.toml, workflows) are not needed and not changed. CHANGELOG.md is a deliverable. No scan test is added, per the request.

## Acceptance criteria

1. [behavior] A new test, run with `cargo test`, puts job ids that differ between a records worktree and in-tree records/. It shows that Athena::status, the forecast and land_pr's job lookup each see the worktree's job ids and not the in-tree ones.
2. [behavior] A test run with `cargo test` removes the worktree directory and shows that status, the forecast and land_pr's lookup read the in-tree records/ job ids, as before the change.
3. [content] crates/athena-data/src/paths.rs defines the athena/records worktree path in one function beside repo_state_dir. A Grep of crates/ shows no other place builds that path (no repeated 'records-worktree' literal).
4. [content] crates/athena-core/src/store.rs has Records::load_current and crates/athena-gate/src/records.rs has RecordSet::load_current. Each reads the worktree when it exists and falls back to the treeish's in-tree records/ only when the worktree is absent.
5. [content] The diff changes only three callers to load_current: status() (and the forecast) in crates/athena-core/src/status.rs, and Athena::land_pr in crates/athena-core/src/gh/landing.rs. A Grep for Records::load and RecordSet::load shows every other non-test caller unchanged. The request names landing.rs's land_pr as gh.rs.
6. [content] docs/design/records-history.md has a follow-ups list naming every remaining non-test caller of Records::load, RecordSet::load, GitRecords::load and the file-based readers by file and function, including status.rs mistake_report and trace, review.rs, proposals.rs, work_decisions.rs, run/mod.rs, run/close.rs, land.rs, auditor.rs, evidence_labels.rs, introspect.rs read_records and the athena-gate readers. The list is checkable against a Grep of the callers.
7. [content] CHANGELOG.md has a new entry for the load_current slice of D111 step 7b. The diff adds no scan test.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- crates/athena-data/src/paths.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any reader other than status, the forecast and land_pr's job lookup
- A scan test that checks all readers
- Creating, updating or syncing the worktree (the writer and the cutover)
- Changing athena.toml, defaults.toml or workflows
- Changing the records branch writer in athena-git

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus fmt, clippy -D warnings and cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test criteria use distinct job ids in the worktree and in-tree records/, so they check which source is read.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design doc follow-up list names each reader by file and function.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-180227-80cr/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-180227-80cr/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test, run with `cargo test`, puts job ids that differ between a records worktree and in-tree records/. It shows that Athena::status, the forecast and land_pr's job lookup each see the worktree's job ids and not the in-tree ones.
A test run with `cargo test` removes the worktree directory and shows that status, the forecast and land_pr's lookup read the in-tree records/ job ids, as before the change.
crates/athena-data/src/paths.rs defines the athena/records worktree path in one function beside repo_state_dir. A Grep of crates/ shows no other place builds that path (no repeated 'records-worktree' literal).
crates/athena-core/src/store.rs has Records::load_current and crates/athena-gate/src/records.rs has RecordSet::load_current. Each reads the worktree when it exists and falls back to the treeish's in-tree records/ only when the worktree is absent.
The diff changes only three callers to load_current: status() (and the forecast) in crates/athena-core/src/status.rs, and Athena::land_pr in crates/athena-core/src/gh/landing.rs. A Grep for Records::load and RecordSet::load shows every other non-test caller unchanged. The request names landing.rs's land_pr as gh.rs.
docs/design/records-history.md has a follow-ups list naming every remaining non-test caller of Records::load, RecordSet::load, GitRecords::load and the file-based readers by file and function, including status.rs mistake_report and trace, review.rs, proposals.rs, work_decisions.rs, run/mod.rs, run/close.rs, land.rs, auditor.rs, evidence_labels.rs, introspect.rs read_records and the athena-gate readers. The list is checkable against a Grep of the callers.
CHANGELOG.md has a new entry for the load_current slice of D111 step 7b. The diff adds no scan test.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
