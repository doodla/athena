# Job 20261107-120506-2ms5: Fix the flaky main-CI shard 1 test and record red main CI baselines

## Request (verbatim)

> Find the test that fails in main CI 'test (shard 1)'. It failed on the push runs after the landings of jobs 20261107-031902-ttnw and 20261107-031925-mrpk, and after the 2026-11-06 landing 'Check hand-kept mirrors against their sources before handover'. Each of those changes had passed its pull request run first. Read the failing test name and panic message from those runs' logs (gh run view --log-failed). State whether the failure is intermittent and give its root cause. Then fix the cause, not the symptom. Also record a red main CI result after a landing in Athena's records, so the next job's baseline notes it. Acceptance: name the test and its cause from the log, show the fix with a repeat run, and keep cargo fmt, clippy and test passing.
> 
> Acceptance:
> - The change is made as described: Main CI test shard 1 failed after three landings in a row whose pull request runs passed, and nothing recorded it
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/

## Reading (accepted by the user)

Find the test failing in main CI 'test (shard 1)' after landings 20261107-031902-ttnw, 20261107-031925-mrpk and the 2026-11-06 mirror-check landing. Name it and its cause from the logs, say whether it is intermittent, fix the root cause, and make Athena record a red main CI result after a landing so the next job's baseline notes it.

Assumptions:
- Goal and audience: Athena maintainers and the next job's producer and verifier. They need a green main CI shard 1 and a baseline that shows a red main run after a landing.
- Environment: the CI site is GitHub Actions on main. The workflow 'test (shard 1)' logs are read with `gh run view --log-failed` (read-only) for the push runs after the three named landings. The verifier cannot run gh, so the test name and panic message are quoted in the CHANGELOG entry and the code comment. Real records and history are reached read-only through `athena status --snapshot` and `git log`.
- Code survey: I did not read the code in this reading. The producer must read the failing test's crate under crates/, the baseline and records code in crates/athena-core/src/ (hestia.rs, status.rs, snapshot.rs) and crates/athena-data/src/records.rs. It must also read the records validation allow-list. A new record kind must be covered by check_record_file, AnyRecord and the round-trip test.
- Existing code: extend crates/athena-core/src/status.rs, crates/athena-core/src/snapshot.rs and crates/athena-data/src/records.rs. They already hold the baseline and records notes. Do not add a parallel recording path.
- Intermittency is a claim to settle from the logs: compare the failing runs with their passing PR runs (shard assignment, ordering, shared state, time, ports, environment). The cause is recorded with its evidence. The fix targets that cause, not a retry or a skip.
- Prior jobs: 20261107-031902-ttnw (drain improvement turn: plan.rs, drain.rs, display.rs, hestia.rs, steward.rs, tests/drain.rs, main.rs) and 20261107-031925-mrpk (throughput waste measures: hestia.rs, status.rs, records.rs, display.rs, status_snapshot.rs, docs/design/hestia.md) landed with verdict pass. Their landed work is kept. Their findings (person-filed requests counted by content, the slowdown request without a turn, a coarse noise bar, window cut mismatch, retries counted only by the 'try again on' text) are not addressed here and stay open. A file of theirs is touched only if the root cause lies in it (hestia.rs, status.rs, plan.rs, drain.rs and tests/drain.rs are listed with low probability for that case).
- The 2026-11-06 landing 'Check hand-kept mirrors against their sources before handover' is also kept. It is touched only if the log shows the failing test or its cause lies there.
- Governance files (athena.toml, defaults.toml, workflows) are not expected to change; the fix is code or test only.
- Records under records/ are never edited by hand. The new red-CI note is produced by code that Athena runs and appended as a new record; existing entries stay unchanged.

## Acceptance criteria

1. [content] CHANGELOG.md has an entry that names the failing test and its panic message as read from the 'test (shard 1)' logs of the push runs after 20261107-031902-ttnw, 20261107-031925-mrpk and the 2026-11-06 mirror-check landing. It states whether the failure is intermittent and gives the root cause, and it states only what the diff does.
2. [content] The diff changes the cause of the failure in the test or the code it covers (for example shared state, ordering or timing), not a retry, ignore or skip attribute. A regression test in the diff fails on the old code and passes on the new.
3. [behavior] Run `cargo test --locked` on the changed tree and check the named test passes on the candidate. Run the named test a second time by name with `cargo test <test name>` to show a repeat pass. Athena's recorded test check on the same final tree also counts.
4. [behavior] Red main CI recording: a test in the diff feeds sample records holding a failed main CI result after a landing and checks the baseline notes it. Run it with `cargo test -p athena-core` or `cargo test -p athena-data`. The test fails when the new recording code is reverted.
5. [behavior] Real records check: run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records. The landing time it prints must stay within the expected range from the commit times of landings in history.json and `git log`, which are not computed by this change. Landings take a few minutes each, so a figure of 0.1 or hundreds of minutes is a failure. The other throughput figures (attempts per landed change, wasted spend, first-attempt share) must equal their output before the change, since this change must not move them.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`; all pass, or the recorded fmt, clippy and test checks on the same final tree show them passed.
7. [content] No unrelated behavior changes: the diff lists no change to athena.toml, defaults.toml or .github/workflows, and no edit to existing entries under records/ or docs/decisions.md.

## Expected deliverables

- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/snapshot.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/status_snapshot.rs (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Rewriting or annotating existing records under records/ or docs/decisions.md.
- Editing athena.toml, defaults.toml, .gitattributes, workflows or prompt files.
- Reverting or redoing jobs 20261107-031902-ttnw and 20261107-031925-mrpk; their landed work is kept, and their listed findings are not addressed here.
- Changing the throughput review measures, the improvement-turn ordering or the 25% slowdown rule.
- Marking the test ignored or adding CI retries to hide the failure.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that the shard 1 test passes on repeat runs and that the baseline notes a red main CI, not that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix removes the cause found in the log, and the criteria require it to be named.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test pass, and a regression test covers the cause.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the test and status commands and check the output.
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

- path: <state>/jobs/20261107-120506-2ms5/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-120506-2ms5/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## How the verifier checks each criterion

The verifier sees only the diff and the commands it runs itself. Before you report, check each criterion yourself the way its line below says, and give that evidence in its criteria_check entry. This is guidance: nothing here refuses or holds your work.

Criterion 1: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 2: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 3: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 4: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 5: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 6: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 7: a named file or diff line: cite the path, and the diff line, that shows it.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md has an entry that names the failing test and its panic message as read from the 'test (shard 1)' logs of the push runs after 20261107-031902-ttnw, 20261107-031925-mrpk and the 2026-11-06 mirror-check landing. It states whether the failure is intermittent and gives the root cause, and it states only what the diff does.
The diff changes the cause of the failure in the test or the code it covers (for example shared state, ordering or timing), not a retry, ignore or skip attribute. A regression test in the diff fails on the old code and passes on the new.
Run `cargo test --locked` on the changed tree and check the named test passes on the candidate. Run the named test a second time by name with `cargo test <test name>` to show a repeat pass. Athena's recorded test check on the same final tree also counts.
Red main CI recording: a test in the diff feeds sample records holding a failed main CI result after a landing and checks the baseline notes it. Run it with `cargo test -p athena-core` or `cargo test -p athena-data`. The test fails when the new recording code is reverted.
Real records check: run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records. The landing time it prints must stay within the expected range from the commit times of landings in history.json and `git log`, which are not computed by this change. Landings take a few minutes each, so a figure of 0.1 or hundreds of minutes is a failure. The other throughput figures (attempts per landed change, wasted spend, first-attempt share) must equal their output before the change, since this change must not move them.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`; all pass, or the recorded fmt, clippy and test checks on the same final tree show them passed.
No unrelated behavior changes: the diff lists no change to athena.toml, defaults.toml or .github/workflows, and no edit to existing entries under records/ or docs/decisions.md.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
