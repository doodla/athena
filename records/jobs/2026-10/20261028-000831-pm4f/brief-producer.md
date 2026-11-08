# Job 20261028-000831-pm4f: Make drain and status never skip a dispatched job with no outcome

## Request (verbatim)

> Find out why these six dispatched jobs have job.json and brief-producer.md but no producer invocation and no outcome.json: 20261026-232359-msh3, 20261026-232423-hvyb, 20261027-054148-g63s, 20261027-123701-77zh, 20261027-054206-w1y3 and 20261027-054209-z7sj. All were dispatched on 2026-10-27, and later jobs (for example 20261027-122909-vvvm and 20261027-225544-q7fe) were dispatched and reached an outcome after them. Outcome: every dispatched job ends with an outcome record, either finished, resumed or closed with a stated reason, and drain does not move past a dispatched job while it has no outcome. Close each of the six with an outcome that cites the commit if its work already landed through another job (for example the Transport-row move and the run.rs rename). Otherwise re-queue it. Acceptance: (1) a test dispatches a job, stops it before the producer runs, and shows that the next drain either resumes it or writes an outcome with a reason; (2) athena status lists any dispatched job with no outcome; (3) the six jobs above have outcome records. Do not change unrelated stages.
> 
> Acceptance:
> - The change is made as described: Six dispatched jobs have a producer brief but no producer run and no outcome; later jobs were dispatched and finished past them
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: job.json, brief-producer.md, outcome.json, run.rs

## Reading (accepted by the user)

Goal: stop dispatched jobs that stop before the producer runs from sitting with no outcome. The maintainer (the audience) should see them in `athena status`, and the next drain should resume them or close them with a reason. The six named jobs (msh3, hvyb, g63s, 77zh, w1y3, z7sj) then get outcomes: closed with a commit citation where the work landed elsewhere, otherwise re-queued.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs (queue view around line 4190-4232, which adds steward::interrupted_work items), crates/athena-core/src/steward.rs (interrupted_work, lines 443-480, restricted to DRAINED_CHANNELS and skipping jobs that a later request retries), crates/athena-core/src/status.rs (lines 885-975, where a Dispatched job with no process is shown as 'interrupted' with a hint) and crates/athena-core/src/reading/retry.rs (interrupted detection, lines 20-60). Entry point: drain builds the queue view, then retries interrupted items via `athena retry`. No new record or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched.
- Likely cause, unverified because records/ is absent from my copy: interrupted_work only covers the drained channels and leaves out any job that a later request names as a retry. A dispatched job on another channel, or one whose 'retry' was itself abandoned, is therefore never resumed or closed, and drain moves past it. The producer must confirm this cause against the code and fix it where it originates. If it differs, fix the actual cause.
- Referenced jobs 20261026-232359-msh3 and 20261026-232423-hvyb: their G2 (Transport row move to gh/transport.rs) and R1 (run.rs to run/mod.rs) work already landed through a re-ask and a retry. The repository shows crates/athena-core/src/run/mod.rs and gh/ exist. Their candidates changed no files, so the work is neither kept nor dropped here. The outcomes cite the landing commit. Job 20261027-054148-g63s (reading-scope sizing; deliverables reading.rs, outputs.rs, config.rs, CHANGELOG.md) was never run and has no candidate; its work is not in the tree unless a later job landed it, so it is re-queued rather than closed, and those files are not deliverables here. The producer must check git log for each of 77zh, w1y3 and z7sj the same way, then close or re-queue.
- Recording the six outcomes is an operator action (`athena close <job> --why ...` or `athena retry <job>`) run against the live records. records/ is never edited by the change, so the diff cannot contain them. This part is checked outside the diff and needs the live records.
- Environment: no CI workflow, push or PR is needed. The producer works in the local checkout. Closing the six jobs needs write access to the live job records and read access to git history; neither is in the sandbox copy.
- Standing defaults apply: tested (cargo test and clippy clean), verified (exercise the behavior), root-cause (fix the class, not six instances), small (one focused change).

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (or crates/athena-core/src/steward.rs unit tests) dispatches a job, stops it before the producer runs (job.json and brief-producer.md present, no invocation, no outcome), then runs a drain. It asserts the job is resumed or gets an outcome record with a stated reason. It covers a job on a non-drained channel and a job named by a later retry request. It fails on the old code and passes on the new. Run: cargo test --locked -p athena-cli --test drain.
2. [behavior] A test shows that drain does not end its run past a dispatched job that has no outcome and no process: that job is resumed, closed with a reason, or reported as held with a reason. Run: cargo test --locked.
3. [behavior] `athena status` lists every dispatched job with no outcome and no process, whatever its channel, with its job id and a next-step hint. A test in crates/athena-core/src/status.rs or crates/athena-cli/tests/ builds records with such a job and asserts the status text or line contains it. Run: cargo test --locked.
4. [content] The six jobs 20261026-232359-msh3, 20261026-232423-hvyb, 20261027-054148-g63s, 20261027-123701-77zh, 20261027-054206-w1y3 and 20261027-054209-z7sj each end with an outcome record (finished, resumed or closed with a reason). Closed jobs whose work landed cite the commit hash found in git log; the others are re-queued. This is checked against the live records (see assumptions); the diff does not contain it.
5. [content] CHANGELOG.md has an entry that names the cause (a dispatched job with no outcome was skipped by drain and not shown by status) and the fix.
6. [behavior] No unrelated behavior changes: existing tests pass, and the producer, verifier and planning stages are untouched. Run: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/reading/retry.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving other structure-pass rows (G3 onward) or redoing the Transport or run/ moves, which already landed
- The reading-scope sizing work of job 20261027-054148-g63s (reading.rs, outputs.rs, config.rs): it is re-queued, not built here
- Editing records/ files, athena.toml, defaults.toml or workflows
- Changing producer or verifier timeouts, planning, or any stage other than drain and status handling of dispatched jobs with no outcome

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for drain and status; fmt, clippy and test are acceptance criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the drain/status gap for the whole class, not the six jobs one by one.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that stop a job before the producer and check the drain and status results.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that every dispatched job ends with an outcome, not that code paths exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to drain, steward and status.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 10 cases in 10 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then required to re-measure the file at the end and report the fi...

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

- path: <state>/jobs/20261028-000831-pm4f/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-000831-pm4f/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-cli/tests/drain.rs (or crates/athena-core/src/steward.rs unit tests) dispatches a job, stops it before the producer runs (job.json and brief-producer.md present, no invocation, no outcome), then runs a drain. It asserts the job is resumed or gets an outcome record with a stated reason. It covers a job on a non-drained channel and a job named by a later retry request. It fails on the old code and passes on the new. Run: cargo test --locked -p athena-cli --test drain.
A test shows that drain does not end its run past a dispatched job that has no outcome and no process: that job is resumed, closed with a reason, or reported as held with a reason. Run: cargo test --locked.
`athena status` lists every dispatched job with no outcome and no process, whatever its channel, with its job id and a next-step hint. A test in crates/athena-core/src/status.rs or crates/athena-cli/tests/ builds records with such a job and asserts the status text or line contains it. Run: cargo test --locked.
The six jobs 20261026-232359-msh3, 20261026-232423-hvyb, 20261027-054148-g63s, 20261027-123701-77zh, 20261027-054206-w1y3 and 20261027-054209-z7sj each end with an outcome record (finished, resumed or closed with a reason). Closed jobs whose work landed cite the commit hash found in git log; the others are re-queued. This is checked against the live records (see assumptions); the diff does not contain it.
CHANGELOG.md has an entry that names the cause (a dispatched job with no outcome was skipped by drain and not shown by status) and the fix.
No unrelated behavior changes: existing tests pass, and the producer, verifier and planning stages are untouched. Run: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
