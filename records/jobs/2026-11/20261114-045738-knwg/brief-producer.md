# Job 20261114-045738-knwg: Keep the failing section, never CI cleanup text, in failed check records

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The check result stores the tail of the CI log, and the tail is post-job cleanup text. The failing t
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: failed-check-output-is-ci-noise-cause-of-test-failure-invisible (20261106-042835-rxkn)
> Cause: The check result stores the tail of the CI log, and the tail is post-job cleanup text. The failing test's output is cut off, so the record cannot be used to diagnose anything. Nothing extracts failing test names from the test runner, and th...
> Seen 6 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261106-042824-j07h (job 20261106-022423-vdtn); prevention it proposed: When the test check fails, Athena should capture and store the failing test names and assertion output from the test runner itself (for example the `failures:` section of cargo test), not the tail of the CI log. If no test name can be extra...
> Introspection 20261106-081000-96mr (job 20261106-045349-f6x6); prevention it proposed: When a test check fails, Athena should save the failing section of the log rather than its tail. That means the lines from "test result", "FAILED", "failures:" and "panicked at", with the test names and assertion messages. The save should a...
> Introspection 20261107-081426-m469 (job 20261027-054225-ycks); prevention it proposed: When a test check fails, Athena should extract the failing test names, assertion messages and the cargo test summary from the full CI log (for example the "failures:" section and "test result:" lines) and store them as the check output, ahe...
> Introspection 20261111-022107-xcgz (job 20261029-204942-2zqw); prevention it proposed: When a test check fails, the kept evidence must include the failing test name and its assertion output, taken from the failed shard and not the tail of the log. A code check should refuse to record a failed test check whose kept output has ...
> Introspection 20261113-023248-yk33 (job 20261030-033234-vm1m); prevention it proposed: Add a code check that, when a test shard fails, keeps the failing test's name and assertion output (the lines around 'FAILED' or 'panicked') as the check's kept output, and runs the failing test once on the base tree to record whether it fa...
> Introspection 20261114-022748-gmm5 (job 20261114-013056-pkhp); prevention it proposed: When a sharded test check fails, keep the failing unit's own output (test names, assertion messages) in the check record and not just the shard summary. Rerun the failed unit once on the unchanged base so the record says whether the job cau...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Failed CI check records still store runner cleanup text (git credential removal, "Cleaning up orphan processes", Node warnings) when the log has no recognised failing section, e.g. a shard that "did not finish". I extend `checks::failure_section` and its callers so cleanup is dropped at the one place the output is stored, and a log with no test section says so and keeps the last pre-cleanup lines. Prior jobs 20261106-022423-vdtn, 20261106-045349-f6x6 and 20261027-054225-ycks hit this noise; their own changes are not kept.

Assumptions:
- Goal and audience: a producer, verifier or person reading a failed check record (send-back brief, verdict findings, introspection) must see the failing test names and assertion output, or an explicit statement that none was found, never runner cleanup text.
- Code survey: read crates/athena-core/src/checks.rs (tail, tail_lines, is_cleanup_line, is_runner_marker, failure_section, with_failure_detail, runs_from_record), crates/athena-core/src/gh/client.rs (plain_log_line, failing_units, detail_from_job_logs, failure_detail), crates/athena-core/src/routing.rs:280 (with_failure_detail applied to reused red runs), crates/athena-core/src/run/send_back.rs:297 (3000-byte tail of output_tail) and crates/athena-core/src/ci_result.rs:99. failure_section already drops a trailing cleanup block only when it finds a recognised cleanup line (Post job cleanup, Post Run, Cleaning up orphan processes) after the last runner marker or ##[error] line at index > 0; no records file or AnyRecord type is added, CheckRun fields are unchanged so the records round-trip test is unaffected. The property is measured directly on failure_section and detail_from_job_logs output.
- Root cause as read: the failing section is found only by recognising a cleanup start line and a runner marker; a log with neither (a cancelled or timed-out shard, a cleanup header form not matched, or a header cut off) falls through to a plain tail, which is cleanup text. Nothing tells a reader when no failing section exists. The fix gives the storing stage that knowledge: the cleanup block is recognised by its content (git config unset, 'Removing HTTP extra header', 'Removing includeIf', 'Cleaning up orphan processes', '##[warning]Node.js' lines, Complete job) as well as by header, and a log with no test section is stored as a short explicit note plus the last non-cleanup lines.
- Prior job 20261106-022423-vdtn (rejected on an unrelated xtask test whose CI output was cleanup text): its candidate files CHANGELOG.md, athena.toml, crates/athena-data/src/config.rs and three design docs are not kept here; its ceiling change is a separate request. Its finding about invisible output is what this change fixes.
- Prior job 20261106-045349-f6x6 (rejected; failed shard shown as cleanup text): its 17 candidate files are not kept, and its mirrored_sources feature is dropped from this change. Its finding that the drain test failure was invisible is addressed by this fix.
- Prior job 20261027-054225-ycks (rejected; check output was cleanup text; flaky drain test): its 5 candidate files (prompts, lib.rs, docs/system-facts.md, CHANGELOG.md) are not kept. The flaky drain test is a separate issue.
- No refusal rule is added; nothing is newly refused, so no refused-share figure applies. The change only alters what a failed check record stores.
- Environment: no CI workflow, push or records read is changed; the tests use inline log strings only, so no host access is needed.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/checks.rs feeds failure_section the log shape seen in the failed jobs (a shard log with no runner marker or ##[error], ending in the git credential-removal lines, 'Cleaning up orphan processes' and a '##[warning]Node.js 20' line, with the 'Post job cleanup.' header absent) and asserts the result contains none of those lines. It fails on the base tree and passes with the change; run `cargo test -p athena-core failure_section`.
2. [behavior] A new test in crates/athena-core/src/gh/client.rs calls detail_from_job_logs with a cancelled shard whose log has only progress lines then cleanup, and asserts output_tail holds the progress lines and a note that no failing test section was found, with no cleanup text; it also asserts a shard log with a `failures:` section still keeps the test name and panicked line. Run `cargo test -p athena-core detail_from_job_logs`.
3. [behavior] A test over the stored path (with_failure_detail, plus a locally run failed check via the checks.rs run path) asserts the stored output_tail and the send_back CheckFailure tail never contain the cleanup lines, so the fix holds for every producer of a failed check record and not one job. Run `cargo test -p athena-core with_failure_detail`.
4. [content] A reading of the diff in crates/athena-core/src/checks.rs shows the cleanup recognition (is_cleanup_line or a successor) lives in one function used by the local run, the reused-CI detail and the per-shard log paths; no second copy is added in gh/client.rs or send_back.rs.
5. [content] The CHANGELOG.md entry names the root cause (a log with no recognised failing section fell through to a plain tail of cleanup text) and what the record stores instead, and states only what the diff does.
6. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (recorded Athena checks on the same tree count under D115).
7. [content] No refusal or admission rule is added or tightened: git diff shows no change to admission.rs or any gate; expected 0% of newly refused readings.

## Expected deliverables

- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Candidate changes of job 20261106-022423-vdtn: CHANGELOG.md range entry, athena.toml, crates/athena-data/src/config.rs, docs/design/async-drain.md, docs/design/parallel-jobs.md, docs/design/agent-and-build-slots.md (dropped; the max_concurrency ceiling is a separate request)
- Candidate changes of job 20261106-045349-f6x6: mirrored_sources plan feature, planner.md, critic.md, admission.rs, plan.rs, records.rs, briefs.rs, outputs.rs, reading.rs, run/mod.rs, run/planning.rs, run_pin_tests.rs, status.rs, store.rs, refusal_corpus.rs, job_plan.rs, docs/design/job-planning.md, docs/system-facts.md (dropped)
- Candidate changes of job 20261027-054225-ycks: commit-shape prompt rules in planner.md, critic.md, crates/athena-context/src/lib.rs, docs/system-facts.md (dropped)
- The flaky drain test status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain and the xtask repin test
- Adding any refusal rule, or changing athena.toml, defaults.toml or CI workflows
- Re-running a failing test on the base tree to record whether the job caused it (proposed by some introspections; a separate request)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in the shared failure_section path, not per job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail on the base tree.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in checks.rs and its callers.
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

- path: <state>/jobs/20261114-045738-knwg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045738-knwg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/checks.rs feeds failure_section the log shape seen in the failed jobs (a shard log with no runner marker or ##[error], ending in the git credential-removal lines, 'Cleaning up orphan processes' and a '##[warning]Node.js 20' line, with the 'Post job cleanup.' header absent) and asserts the result contains none of those lines. It fails on the base tree and passes with the change; run `cargo test -p athena-core failure_section`.
A new test in crates/athena-core/src/gh/client.rs calls detail_from_job_logs with a cancelled shard whose log has only progress lines then cleanup, and asserts output_tail holds the progress lines and a note that no failing test section was found, with no cleanup text; it also asserts a shard log with a `failures:` section still keeps the test name and panicked line. Run `cargo test -p athena-core detail_from_job_logs`.
A test over the stored path (with_failure_detail, plus a locally run failed check via the checks.rs run path) asserts the stored output_tail and the send_back CheckFailure tail never contain the cleanup lines, so the fix holds for every producer of a failed check record and not one job. Run `cargo test -p athena-core with_failure_detail`.
A reading of the diff in crates/athena-core/src/checks.rs shows the cleanup recognition (is_cleanup_line or a successor) lives in one function used by the local run, the reused-CI detail and the per-shard log paths; no second copy is added in gh/client.rs or send_back.rs.
The CHANGELOG.md entry names the root cause (a log with no recognised failing section fell through to a plain tail of cleanup text) and what the record stores instead, and states only what the diff does.
`cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (recorded Athena checks on the same tree count under D115).
No refusal or admission rule is added or tightened: git diff shows no change to admission.rs or any gate; expected 0% of newly refused readings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
