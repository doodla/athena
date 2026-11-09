# Job 20261028-043810-h0ws: Skip Priority lines in the Queue a request commit subject

## Request (verbatim)

> When building the 'Queue a request:' commit subject in crates/athena-core/src/reading.rs (and the 'Dropped:' subject, wherever it is built from the request text), skip leading lines that match the 'Priority: <level>' marker parsed in crates/athena-core/src/plan.rs and use the first remaining non-empty line. Add a test with a request whose first line is 'Priority: urgent' showing that the subject names the request, not the priority. Add a CHANGELOG.md entry.
> 
> Acceptance:
> - The change is made as described: Queued and dropped request commits are titled only 'Priority: <level>' because the subject takes the request's first line
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/reading.rs, crates/athena-core/src/plan.rs, CHANGELOG.md

## Reading (accepted by the user)

`Athena::queue` in reading.rs builds "Queue a request: <first line>" from the request's raw first line, so a request starting "Priority: urgent" is titled by its priority. The fix uses the existing `plan::title_line` helper, which skips Priority and blank lines, and adds a test and a CHANGELOG entry. The audience is anyone reading the records log.

Assumptions:
- Code survey: crates/athena-core/src/reading.rs `queue` (about lines 895-932) takes `request.text.lines().next()` truncated to 60 chars for the subject. crates/athena-core/src/plan.rs already has `pub fn title_line` (line 114), which skips Priority: lines and blanks and has its own tests. crates/athena-core/src/store.rs `title()` already uses it. crates/athena-core/src/manage.rs close builds its subjects from `job.title()`, which is `title_line`-based when there is no reading. No records or data file is added, so no record allow-list is affected.
- The 'Dropped:' subject is probably already correct, because it comes from `job.title()` in store.rs, which uses `title_line`. The producer should confirm this with a Grep and change it only if some path still takes the raw first line. Nothing under crates/athena-gate needs to change.
- The 60-character truncation stays. `title_line` returns the first line when every line is a Priority or blank line, and that fallback is kept.
- The test goes in the existing tests module of reading.rs. It calls `queue` through the existing test helpers, with a request like 'Priority: urgent\n\nFix the widget', and reads the commit subject back with git log.
- No governance files change. CHANGELOG.md is the only non-code deliverable.

## Acceptance criteria

1. [content] In crates/athena-core/src/reading.rs, the `queue` subject uses `crate::plan::title_line(&request.text)` (then the 60-char cap) instead of `lines().next()`; check the diff.
2. [behavior] A new test in reading.rs queues a request whose first line is 'Priority: urgent' followed by a real request line. It asserts the commit subject is 'Queue a request: <real line>' and does not contain 'Priority'. Check with `cargo test -p athena-core` and the test name in the diff.
3. [content] The diff shows no raw first-line subject left for a Dropped commit. The Dropped subject path (manage.rs via `job.title()`) is either shown to use `title_line` or changed to use it.
4. [content] CHANGELOG.md has a new entry under the unreleased section saying that queued and dropped request commit subjects skip leading Priority lines.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test` all pass, and the existing `title_line` tests in plan.rs still pass unchanged.
6. [content] The diff touches only reading.rs, CHANGELOG.md, and plan.rs or manage.rs if needed; no other behavior changes.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/manage.rs (modify)

## Out of scope

- Changing how Priority is parsed or how priority affects plan ordering
- Changing the 60-character subject cap
- Changing titles taken from a reading
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test and clippy criteria
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that reuses the existing title_line helper
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Checks that the Dropped path also uses the shared helper, not only the Queue path
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test asserts the actual commit subject
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are in Rust
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria are commands the verifier can run
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 12 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must size the slice to reach the limit. The producer must also run...
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

- path: <state>/jobs/20261028-043810-h0ws/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-043810-h0ws/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In crates/athena-core/src/reading.rs, the `queue` subject uses `crate::plan::title_line(&request.text)` (then the 60-char cap) instead of `lines().next()`; check the diff.
A new test in reading.rs queues a request whose first line is 'Priority: urgent' followed by a real request line. It asserts the commit subject is 'Queue a request: <real line>' and does not contain 'Priority'. Check with `cargo test -p athena-core` and the test name in the diff.
The diff shows no raw first-line subject left for a Dropped commit. The Dropped subject path (manage.rs via `job.title()`) is either shown to use `title_line` or changed to use it.
CHANGELOG.md has a new entry under the unreleased section saying that queued and dropped request commit subjects skip leading Priority lines.
`cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test` all pass, and the existing `title_line` tests in plan.rs still pass unchanged.
The diff touches only reading.rs, CHANGELOG.md, and plan.rs or manage.rs if needed; no other behavior changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
