# Job 20261106-185246-gpma: Record main's CI test failures and attach them to flaky-test requests

## Request (verbatim)

> When a CI test check fails on main, store the failing test names and their panic or assertion text in a record Athena can read (not the log tail). Attach that record to the request of any job that diagnoses or fixes that flaky test, so the job starts from the recorded failure. Add a test that feeds a CI log with a panic in the middle and cleanup at the end, and checks that the record and the attached request contain the test name and panic text.
> 
> Acceptance:
> - The change is made as described: Flaky-test jobs start without main's CI failure text, so the cause is guessed or found by the the person running Athena by hand
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-11/20261106-145054-040d.json

## Reading (accepted by the user)

When the CI test check fails on main, Athena would store the failing test names and their panic or assertion text in a typed record it can read. Any request that diagnoses or fixes that flaky test would get the record attached, so the job starts from the recorded failure. The goal is that the person running Athena no longer has to paste CI text by hand. A test feeds a log with a panic in the middle and cleanup at the end.

Assumptions:
- Goal and audience: the person running Athena, and the jobs that diagnose or fix flaky tests, should not have to paste main's CI failure text by hand. The verifier judges from the diff and the test.
- Code survey: read crates/athena-core/src/ci_result.rs (CiSource::failure_detail, FailureDetail with failed_tests and output_tail), crates/athena-core/src/gh/client.rs (detail_from_job_logs, failing_test_name), crates/athena-core/src/checks.rs (failed_test_names, failure_section, FAILED_TAIL_LINES) and crates/athena-core/src/steward.rs and hestia.rs (StewardRequest, the request builder). Existing code cuts cleanup text and keeps a log tail, but nothing reads main's own CI failure or attaches it to a request. The entry point to add is where a request is built or queued. The records validation allow-list lives in crates/athena-data/src/records.rs. A new record kind there would need `check_record_file` or `AnyRecord` and its round-trip test. Whether the record can instead reuse an existing kind is the producer's call.
- Existing code: crates/athena-core/src/ci_result.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/gh/client.rs and crates/athena-core/src/steward.rs are extended or reused, not recreated.
- Environment: main's CI is read through the existing injected CiSource over GitHub's API, read-only. The record is written to Athena's own records area, and no workflow file or push is needed. The test uses a fake source and a string log, with no network access.
- Field meaning: no existing field is reused; the record's fields are new.
- The request names records/introspections/2026-11/20261106-145054-040d.json as its source. That is the evidence for the problem and is not changed, because records are append-only. It was not readable in the copy.
- Whether a request 'diagnoses or fixes' a flaky test is decided by matching the failing test name in the request text, or the request's cause. This keeps the match mechanical and testable. The producer states the rule it chose.

## Acceptance criteria

1. [behavior] A new unit test feeds a CI log with a test panic in the middle (`thread 'x' panicked at ... assertion `left == right` failed`) and runner cleanup lines at the end. It asserts that the stored main-CI failure record contains the failing test name and the panic or assertion text, and that the request text built for a flaky-test job contains both too. Run with `cargo test -p athena-core main_ci_failure`.
2. [behavior] The test fails without the change: with the attach step removed or the log tail used instead of the panic text, the request assertion fails, because the cleanup lines at the end of the log would displace the panic. Checked by `cargo test -p athena-core main_ci_failure`.
3. [content] The record stores failing test names plus panic or assertion text as typed fields. It does not store a raw log tail. Redaction and clipping, as the existing `FailureDetail` path does, are applied before storing. Diff of `crates/athena-core/src/main_ci_failure.rs`.
4. [content] The new code reuses `crate::checks::failed_test_names` and `crate::ci_result::FailureDetail`/`CiSource::failure_detail` where they fit, and does not duplicate them. A diff line shows the call.
5. [content] The attach step applies only to requests that diagnose or fix the flaky test named in the record, and leaves other requests unchanged. A test asserts an unrelated request text is unchanged.
6. [content] Any new record or data file is covered by the records validation allow-list and its round-trip test (`check_record_file` or `AnyRecord`). CHANGELOG.md records the change; records/ is untouched.
7. [behavior] No unrelated behavior changes: `cargo test`, `cargo clippy` and `cargo fmt --check` pass, citing Athena's recorded checks on the final tree or the commands run.

## Expected deliverables

- crates/athena-core/src/main_ci_failure.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Rerunning or retrying flaky tests
- Changing workflow files or athena.toml
- Editing existing records or docs/decisions.md entries
- Changing how a job's own candidate checks are resolved

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the test name and panic text in both the record and the attached request.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses existing log-parsing code instead of a new parser.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

- path: <state>/jobs/20261106-185246-gpma/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-185246-gpma/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## How the verifier checks each criterion

The verifier sees only the diff and the commands it runs itself. Before you report, check each criterion yourself the way its line below says, and give that evidence in its criteria_check entry. This is guidance: nothing here refuses or holds your work.

Criterion 1: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 2: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 3: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 4: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 5: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 6: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 7: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test feeds a CI log with a test panic in the middle (`thread 'x' panicked at ... assertion `left == right` failed`) and runner cleanup lines at the end. It asserts that the stored main-CI failure record contains the failing test name and the panic or assertion text, and that the request text built for a flaky-test job contains both too. Run with `cargo test -p athena-core main_ci_failure`.
The test fails without the change: with the attach step removed or the log tail used instead of the panic text, the request assertion fails, because the cleanup lines at the end of the log would displace the panic. Checked by `cargo test -p athena-core main_ci_failure`.
The record stores failing test names plus panic or assertion text as typed fields. It does not store a raw log tail. Redaction and clipping, as the existing `FailureDetail` path does, are applied before storing. Diff of `crates/athena-core/src/main_ci_failure.rs`.
The new code reuses `crate::checks::failed_test_names` and `crate::ci_result::FailureDetail`/`CiSource::failure_detail` where they fit, and does not duplicate them. A diff line shows the call.
The attach step applies only to requests that diagnose or fix the flaky test named in the record, and leaves other requests unchanged. A test asserts an unrelated request text is unchanged.
Any new record or data file is covered by the records validation allow-list and its round-trip test (`check_record_file` or `AnyRecord`). CHANGELOG.md records the change; records/ is untouched.
No unrelated behavior changes: `cargo test`, `cargo clippy` and `cargo fmt --check` pass, citing Athena's recorded checks on the final tree or the commands run.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
