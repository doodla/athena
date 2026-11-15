# Job 20261114-045803-0hj9: Make scope guard read each attempt's stop reason and near-complete

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The scope guard treats all spend on a line as one number. It does not record why an attempt stopped,
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: scope-guard-counts-environment-stop-as-line-spend-the person running Athena-lifts-pause (20261028-065858-bztk)
> Cause: The scope guard treats all spend on a line as one number. It does not record why an attempt stopped, and it does not know how much work is left. A stop caused by the environment (low disk) looks the same as a stop caused by the work being t...
> Seen 5 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261028-065847-kmx7 (job 20261028-055601-hg0n); prevention it proposed: Record a stop reason on every attempt. Have the scope guard exclude attempts that ended on an environment stop (low disk, container reclaim) from the spend and attempt counts, or count them separately. When an attempt has a kept candidate t...
> Introspection 20261028-103312-b53e (job 20261028-065911-m6m5); prevention it proposed: Make the scope guard record the stop reason for each attempt and the state of the kept candidate. Spend from an environment stop, such as low disk, must not count toward the line's progress-less spend. If a candidate is kept and has passed ...
> Introspection 20261028-124847-gf2n (job 20261028-103330-3d47); prevention it proposed: Compute the pause instead of storing it. On each guard evaluation (and whenever the guard's rule changes), recompute the line's spend from the per-attempt stop reasons under the current rule, and clear or keep the pause on the result. If a ...
> Introspection 20261029-195138-3pqb (job 20261029-111202-2bpc); prevention it proposed: Record a stop reason with each attempt's spend (environment, mechanical plan finding, real critic or verifier failure, producer failure). The guard should count only the reasons that signal runaway work. Store with each pause the rule versi...
> Introspection 20261103-101210-m2aa (job 20261103-092929-s7ts); prevention it proposed: When the guard is about to pause a line, it should first read the latest verdict. If most criteria are met (for example 8 of 9) and every gap is a named, bounded finding, the guard should not pause. It should queue a targeted retry limited ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the scope guard (crates/athena-core/src/scope_guard.rs) reduces an attempt to a verdict, a cost and minutes, so it does not know how far a rejected attempt got. Environment stops are already excluded (disk.rs StopCause), so this job adds the missing fact for the last introspection (20261103-092929-s7ts), a near-complete rejection, and pauses on spend only when work is not converging.

Assumptions:
- Goal and audience: the person running Athena, and the drain, should not see a line paused on spend when the last attempt stopped for a reason that does not signal runaway work. The verifier judges the diff and tests.
- Code survey: read crates/athena-core/src/scope_guard.rs (AttemptRecord, Verdict::Environment, t1_retries, t3_basis, t3_cost, t5_wall_time, verdict_of, attempt_of, request_records, evaluate_before_attempt, and the stale-pause recheck near line 1776) and crates/athena-core/src/disk.rs (StopCause, is_environment_reason, resumable_stop). Entry point: evaluate_before_attempt over attempts built by request_records from job records; attempt_of reads job.verification.criteria. No record or data file is added, so check_record_file, AnyRecord and the round-trip test are untouched. The property is measured directly on AttemptRecord lists in unit tests in scope_guard.rs.
- Existing code: crates/athena-core/src/scope_guard.rs (AttemptRecord, t3_basis, t3_cost, attempt_of) and crates/athena-core/src/disk.rs (StopCause, is_environment_reason) are extended, not replaced. Environment stops (low disk, check timeout, producer auth, usage limit, build slot) are already excluded from T1, T2, T5 and kept-work T3 spend, which covers introspections 20261028-065847-kmx7 (job 20261028-055601-hg0n), 20261028-103312-b53e (job 20261028-065911-m6m5), 20261028-124847-gf2n (job 20261028-103330-3d47) and 20261029-195138-3pqb. The remaining uncovered case is 20261103-101210-m2aa (job 20261103-092929-s7ts).
- Previous jobs 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47 were an unrelated replay feature (athena check-reading, reading replay) that stopped on low disk or an expired login. Their findings were environment stops, not code findings, and are now classified as environment by the guard. That replay feature is not part of this request. Their candidates' changed files (CHANGELOG.md, athena-cli main.rs, briefs.rs, lib.rs, reading.rs, run/mod.rs, store.rs, verify.rs, records.rs, docs/records.md) are dropped and not kept here, except CHANGELOG.md, which is a deliverable.
- Design: AttemptRecord gains the count of criteria met and the count of criteria in the last verification (None when unknown). The guard's T3 spend pause does not fire when the last attempt was rejected with at least 80% of criteria met (an estimate, a code constant) and its stop is a bounded gap; it downgrades to a flag so a targeted retry is allowed. A line with no met-criteria data behaves as before. The 80% threshold is a judgment call, labeled as such in the docs comment.
- This adds a rule that holds less work, not more, so no refusal-rule share applies. It refuses no readings.
- Term line: a line of work is a request, its retries and re-asks, grouped by group_lines in scope_guard.rs.
- Environment: no CI workflow, push, PR or history read is involved.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/scope_guard.rs builds a line whose last attempt is Rejected with 8 of 9 criteria met and spend over T3_PAUSE_RATIO times the forecast, and asserts t3_cost returns a level below Pause (checked by running cargo test); the same test with 3 of 9 met asserts Pause, so it fails without the change.
2. [behavior] A unit test shows an attempt record built by attempt_of from a rejected job's verification carries the met and total criterion counts (checked by running cargo test), and a Failed or Environment attempt carries none.
3. [content] The doc comments of AttemptRecord and t3_cost in crates/athena-core/src/scope_guard.rs name the root cause (the guard treated every attempt's spend as one number and did not record how far a rejected attempt got), and say that environment stops stay excluded through crate::disk::is_environment_reason, with no second list of environment reasons.
4. [behavior] Existing scope_guard tests for environment stops (environment_stop_is_not_counted_but_a_rejection_is, every_kind_of_environment_stop_is_an_environment_attempt) still pass under cargo test.
5. [content] CHANGELOG.md has a new entry for this change, and no refusal rule is added to crates/athena-core/src/reading.rs.
6. [behavior] cargo fmt --check, cargo clippy and cargo test pass on the candidate tree.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/disk.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- The reading replay and athena check-reading from jobs 20261028-055601-hg0n, 20261028-065911-m6m5 and 20261028-103330-3d47; their candidate files (store.rs, verify.rs, records.rs, briefs.rs, reading.rs, run/mod.rs, main.rs, lib.rs, docs/records.md) are dropped
- Automatic disk cleanup or disk reservation before the verifier
- Adding any refusal rule for readings
- Changing the thresholds of T1, T2, T4 or T5

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new unit tests plus cargo test, clippy and fmt passing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix is in the guard's attempt record, not at one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name tests that fail without the change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Time budget

Each producer run, the first and any send-back, is stopped after 86400 seconds of wall-clock time. A stopped run sends no report, so what is on disk when it stops must already be a usable change.

Pace the work by its order. Do the smallest change that meets every acceptance criterion first, one file at a time, and leave extras for afterwards; report as soon as the criteria are met. Leave no half-edited file and no broken build between steps. Use targeted tests while you work (`cargo test -p <crate> <filter>`). The full suite runs at most once, and Athena runs it again on your result (see "Checks before the verifier" below), so a second full run only spends this limit.

You have no clock of your own. If the commands Bash may run (below) include `date`, note the time when you start and read it between steps; without it, count steps against the order above.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

36 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261114-045803-0hj9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045803-0hj9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/scope_guard.rs builds a line whose last attempt is Rejected with 8 of 9 criteria met and spend over T3_PAUSE_RATIO times the forecast, and asserts t3_cost returns a level below Pause (checked by running cargo test); the same test with 3 of 9 met asserts Pause, so it fails without the change.
A unit test shows an attempt record built by attempt_of from a rejected job's verification carries the met and total criterion counts (checked by running cargo test), and a Failed or Environment attempt carries none.
The doc comments of AttemptRecord and t3_cost in crates/athena-core/src/scope_guard.rs name the root cause (the guard treated every attempt's spend as one number and did not record how far a rejected attempt got), and say that environment stops stay excluded through crate::disk::is_environment_reason, with no second list of environment reasons.
Existing scope_guard tests for environment stops (environment_stop_is_not_counted_but_a_rejection_is, every_kind_of_environment_stop_is_an_environment_attempt) still pass under cargo test.
CHANGELOG.md has a new entry for this change, and no refusal rule is added to crates/athena-core/src/reading.rs.
cargo fmt --check, cargo clippy and cargo test pass on the candidate tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
