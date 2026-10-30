# Job 20261029-233021-e5sg: Match each job's landing stage to its own landing evidence

## Request (verbatim)

> Priority: urgent
> The throughput section's landing stage is still wrong after the fix that landed as 2270e80: it now reports a median of 273.8 minutes for landing and rebuild, while real landings take minutes (job 20261029-130605-mbpm: outcome 20:41:09 UTC, its commit on main at 20:44:56 UTC). No landing record for that job exists under the landings path the code searches, so the matching picks some other record. The tests used only fixture records, so the error reached main. Fix: match each job to its own landing evidence (its landing record if one exists, else the merge commit on main that carries the job id in its records or the landed job's commit time), and when no evidence is found say so for that job instead of borrowing another record. Validate against this repository's real records, not only fixtures.
> Acceptance:
> - A test with fixture records covers a job with its own landing record, a job without one, and a neighbouring job's record that must not be used.
> - A check run on this repository's real records (shown in the job report and repeatable by the verifier with athena status) gives, for the five most recent landed jobs, landing durations within 3 minutes of the time between each job's outcome and its commit on main as shown by git log.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

Fix the throughput section's landing-and-rebuild stage in status.rs so each landed job ends at its own landing evidence (its landing record, else its merge commit on main that carries the job id, else the job's commit time). When none is found, report no evidence for that job and don't borrow another record. This follows the landed 2270e80 fix, which still reported a 273.8-minute median. It must be validated on this repository's real records.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs. landing_record_time (~line 772) filters records.landings for a JobLanding commit with job == this job, taking the newest created_at. landing_end (~788) takes the later of the outcome and that time, else the outcome. It feeds busy_intervals, stage measurement (~859) and the summary text (~976). The fix is in this resolution order: own landing record, then a main-branch merge commit carrying the job id, then the job's commit time, then explicit none. Landing time is measured directly as that evidence's timestamp minus the verifier end or outcome, not via a stand-in. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected.
- Goal and audience: the owner reading `athena status` throughput needs a trustworthy landing stage. The request is urgent because the wrong median reached main.
- The newest-record rule in landing_record_time and any commit-kind matching may pick a record in which the job id appears as a neighbour. The producer must find the actual root cause on the real records, not only assume this one.
- Evidence outside records/ is read through the existing history data (athena_data::history); no new dependency is added. If the merge commit isn't reachable from the loaded data, the fallback is the landed job's commit time.
- Job-level 'no evidence' is shown in the report text, for example 'landing evidence not found for N of M', and that job's landing stage is excluded from the median rather than counted as zero or borrowed.
- Governance files (athena.toml, defaults.toml, workflows) are not needed; CHANGELOG.md is a deliverable per the request.

## Acceptance criteria

1. [content] crates/athena-core/src/status.rs has a fixture test with three jobs: one with its own landing record (its landing stage equals that record's time minus the outcome), one with no landing record (reported as no evidence, or resolved only by its own merge commit or commit time), and one whose neighbouring job's landing record exists and is not used for it.
2. [behavior] Run `cargo run -- status` (athena status) on this repository's real records. For the five most recent landed jobs the reported landing durations are each within 3 minutes of the time between the job's outcome and its commit on main from `git log`. For job 20261029-130605-mbpm the landing stage is about 3.8 minutes (outcome 20:41:09 to commit 20:44:56 UTC), not hours. The overall median for landing and rebuild is no longer 273.8 minutes. The producer's job report shows this check.
3. [content] When no landing evidence is found for a job, the status output says so for that job. No other job's record is used, and the code path is covered by a test.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
5. [content] CHANGELOG.md has an entry that names the landing-stage matching fix and the real-records validation.
6. [content] Job 20261029-130605-mbpm, a landed job that only supplies the evidence case, is not modified. Its changed files (CHANGELOG.md, crates/athena-core/src/hestia.rs, docs/enforcement.md, docs/spec.toml, xtask/tests/spec.rs) are kept as landed and not touched, and its two minor findings (a stale spec.toml comment and OLY-B-WHO-01 lacking an open-question note) are out of scope here.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Fixing the two minor findings of job 20261029-130605-mbpm (stale spec.toml comment about two late headings; OLY-B-WHO-01 open-question note)
- Editing job 20261029-130605-mbpm's landed files: docs/spec.toml, xtask/tests/spec.rs, docs/enforcement.md, crates/athena-core/src/hestia.rs (kept as landed; CHANGELOG.md is edited only to add the new entry)
- Changing athena.toml, defaults.toml, workflow files or records/
- Redesigning other throughput stages or the busy-interval model beyond what landing matching needs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture test plus fmt, clippy -D warnings and the full test suite.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the five most recent landed jobs on the real records against git log.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix replaces the matching rule rather than patching the one job.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier repeats the athena status check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is landing durations matching real commit times, not just that code compiles.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A job with no evidence is reported as such, not counted as success or zero.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 35 cases in 35 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Have the producer and checks code emit typed stage events (an enum with Producer, Checks and so on) at each transition, including the send-back transition, and have the in-flight table record only those. Do not parse display strings. Add a test that runs the real drain path through checks failing an...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

20 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

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

crates/athena-core/src/status.rs has a fixture test with three jobs: one with its own landing record (its landing stage equals that record's time minus the outcome), one with no landing record (reported as no evidence, or resolved only by its own merge commit or commit time), and one whose neighbouring job's landing record exists and is not used for it.
Run `cargo run -- status` (athena status) on this repository's real records. For the five most recent landed jobs the reported landing durations are each within 3 minutes of the time between the job's outcome and its commit on main from `git log`. For job 20261029-130605-mbpm the landing stage is about 3.8 minutes (outcome 20:41:09 to commit 20:44:56 UTC), not hours. The overall median for landing and rebuild is no longer 273.8 minutes. The producer's job report shows this check.
When no landing evidence is found for a job, the status output says so for that job. No other job's record is used, and the code path is covered by a test.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has an entry that names the landing-stage matching fix and the real-records validation.
Job 20261029-130605-mbpm, a landed job that only supplies the evidence case, is not modified. Its changed files (CHANGELOG.md, crates/athena-core/src/hestia.rs, docs/enforcement.md, docs/spec.toml, xtask/tests/spec.rs) are kept as landed and not touched, and its two minor findings (a stale spec.toml comment and OLY-B-WHO-01 lacking an open-question note) are out of scope here.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
