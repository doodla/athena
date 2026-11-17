# Job 20261106-043228-80zr: Make the xtask repin test and the drain pid-file test deterministic

## Request (verbatim)

> Two jobs failed in CI on a test their diff did not touch. In each case the verifier reran that one test and it passed. (1) In job 20261106-022423-vdtn, xtask tests::repin_sets_only_the_version_and_is_idempotent (xtask/src/main.rs:845-856) panicked because '<tmp>/claude --version failed'. Every criterion was met and the agent verdict was pass, but the final verdict was fail for 'check test failed' alone. This is the third rejection of the same ceiling change. (2) In job 20261106-012852-h7de, athena-cli drain::tests::status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain failed. Job 20261105-205932-ppfv added that test, which spawns processes. Find out why each test can fail and make it deterministic. My unverified guess for (1) is a race between writing the stand-in script and running it while other test threads fork (ETXTBSY). Name the root cause in the CHANGELOG. Acceptance: each test passes 200 runs in a row under the full parallel test binary (say how that was measured); no test coverage is removed; cargo fmt --check, cargo clippy --all-targets and cargo test pass.
> 
> Acceptance:
> - The change is made as described: Tests fail now and then in CI on code the job did not touch and pass on rerun, yet the failure still rejects the job
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: xtask/src/main.rs:845-856, tmp>/claude

## Reading (accepted by the user)

Find why xtask tests::repin_sets_only_the_version_and_is_idempotent (job 20261106-022423-vdtn) and athena-cli drain::tests::status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain (job 20261106-012852-h7de, test added by 20261105-205932-ppfv) fail intermittently under the parallel test binary, fix each root cause without removing coverage, and name both causes in CHANGELOG.md. Audience: the person running Athena, whose unrelated jobs are rejected by these flakes.

Assumptions:
- Code survey: read xtask/src/main.rs (tests at 855+, stand_in_claude, repin_at at 726, found at 168, retry_busy at 146-166), crates/athena-testkit/src/lib.rs (write_executable: writes via a child sh to path.writing, chmod, mv), crates/athena-agent/src/procs.rs (retry_busy, 20 retries x 50 ms, ETXTBSY only) and crates/athena-cli/src/drain.rs (test helpers stand_in_drain 5460, sleeper 5376, settled 5411, quick 5400, ReleaseAfterFirstLookup 5477, the test at 5524). The change adds no record or data file under crates/, so check_record_file and AnyRecord are not touched; no refusal rule is added or tightened.
- Finding on the user's guess: ETXTBSY is already partly mitigated. write_executable renames a finished file into place and found() retries ETXTBSY for about 1 s. But found() collapses every failure (ETXTBSY after the retries are spent, any other spawn error, a non-zero exit) into None, so the message '<tmp>/claude --version failed' hides the real cause. The producer must reproduce the flake first (loop the full xtask test binary 200 times with --test-threads high and a CPU or fork load, capturing the underlying io error) and name the cause that was observed. If ETXTBSY is not the observed cause, the CHANGELOG names the one that was. A guess is not a root cause.
- For the drain test the likely causes to check are: the test spawns real sleep and sh processes and reads their start marker, with fixed timing (settled waits 10 s, quick(20 s)); a decoy or stand-in that exits or is reaped early; stat-based start marker reads racing process start; sh -c polling with sleep 0.05 under load; and ppid or pid reuse against other tests that also spawn. The producer reproduces under the same loop and fixes the observed cause, for example by waiting on an explicit readiness signal instead of a time.
- Previous results. Job 20261106-022423-vdtn (rejected, check test failed on the xtask repin test; the ceiling raise 8 to 16 itself met every criterion): its finding 'xtask test failed on a path the diff did not touch' is addressed by fixing that test here. Its other finding (CHANGELOG filed under Added not Changed) belongs to the ceiling change, which is a separate retry. Its six changed files (CHANGELOG.md, athena.toml, crates/athena-data/src/config.rs, the three design docs) are dropped from this job, not kept. Job 20261106-012852-h7de (rejected): its flaky drain test failure is addressed here; its major group_sizes findings and the ten changed files are dropped and belong to the separate retry of 20261105-180339-w5h7. Job 20261105-205932-ppfv (landed): its work is kept as landed; only its test in drain.rs is made deterministic, and its drain-status, drain-stop and drain-restart behavior is not changed.
- Environment: the flakes were seen in CI (GitHub Actions running `cargo test --locked`, sharded by `xtask test-shard`), which the job cannot read or write. The producer works only locally with read/write access to the working copy. No CI workflow, athena.toml or defaults.toml change is needed; if the cause turns out to need one, the producer reports it rather than editing.
- Measuring the 200 runs: the loop is the full test binary of each crate (cargo test -p xtask --bin xtask and cargo test -p athena-cli --lib), run 200 times in a row with the default parallel threads, filtered to its test name in the pass count, plus the same under added CPU load. The CHANGELOG states the exact command, thread count and load used, and the observed failure rate before and after, labeled as measured on the producer's machine. The verifier cannot run 200 loops and does not rely on a producer report; it checks the CHANGELOG text and runs each crate's tests once.
- Out of scope: the 8 to 16 ceiling and the group_sizes record (separate retries), the xtask test-shard 'did not finish' symptom beyond what these two fixes explain, other tests, and any change to drain behavior, athena.toml, defaults.toml or workflow files.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry (a Fixed or Changed section, per the file's existing style) that names the measured root cause of each flake: for xtask repin_sets_only_the_version_and_is_idempotent and for the drain pid-file test. Each cause is specific (the syscall, race or timing assumption), not 'flaky'. Check with Grep for both test names in CHANGELOG.md.
2. [content] CHANGELOG.md states how the 200-run check was measured: the exact command, the test-thread count, any added load, and the before and after failure counts. Check by reading the entry; a number without a command fails this criterion.
3. [content] No coverage is removed: `git diff` shows no deleted #[test] function, and Grep still finds repin_sets_only_the_version_and_is_idempotent in xtask/src/main.rs and status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain in crates/athena-cli/src/drain.rs, with every assertion of the latter still present (the decoy untouched, the pid in status, Exited after the wait, the Dead state).
4. [content] The xtask failure is no longer opaque: the diff makes repin_at or found report the underlying io error or exit status rather than only 'claude --version failed', or removes the source of the failure, and a test or the test body covers it. Check by reading the diff in xtask/src/main.rs.
5. [behavior] Run `cargo test -p xtask --bin xtask` and `cargo test -p athena-cli --lib` as separate commands; both pass, including each named test, which runs in the parallel binary with default threads.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` as separate commands; all pass, or a passing Athena-recorded check on the same final tree is cited (D115).
7. [content] No unrelated behavior changes: `git diff --stat` lists only the deliverables; production code of drain.rs and xtask (outside test modules and any minimal error-reporting change in found/repin_at) is unchanged; athena.toml, defaults.toml, workflows and records/ are untouched.

## Expected deliverables

- xtask/src/main.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-testkit/src/lib.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The agent ceiling raise from 8 to 16 and its files (athena.toml, crates/athena-data/src/config.rs, docs/design/async-drain.md, docs/design/parallel-jobs.md, docs/design/agent-and-build-slots.md): work of 20261106-022423-vdtn is dropped and retried separately
- Recording plan group sizes and the files of 20261106-012852-h7de (display.rs, tests/drain.rs, plan.rs, main.rs help text): dropped, belongs to the retry of 20261105-180339-w5h7
- Behavior of athena drain, drain-status, drain-stop and drain-restart from 20261105-205932-ppfv: kept as landed
- Changes to athena.toml, defaults.toml, .github/workflows, setup.sh or records/
- Other tests and the xtask test-shard timeout beyond what these two fixes explain

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the tests passing under the real parallel binary, not that helper code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer must reproduce and name the observed cause of each flake, not only retry or sleep longer.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria included.
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

- path: <state>/jobs/20261106-043228-80zr/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-043228-80zr/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md has a new entry (a Fixed or Changed section, per the file's existing style) that names the measured root cause of each flake: for xtask repin_sets_only_the_version_and_is_idempotent and for the drain pid-file test. Each cause is specific (the syscall, race or timing assumption), not 'flaky'. Check with Grep for both test names in CHANGELOG.md.
CHANGELOG.md states how the 200-run check was measured: the exact command, the test-thread count, any added load, and the before and after failure counts. Check by reading the entry; a number without a command fails this criterion.
No coverage is removed: `git diff` shows no deleted #[test] function, and Grep still finds repin_sets_only_the_version_and_is_idempotent in xtask/src/main.rs and status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain in crates/athena-cli/src/drain.rs, with every assertion of the latter still present (the decoy untouched, the pid in status, Exited after the wait, the Dead state).
The xtask failure is no longer opaque: the diff makes repin_at or found report the underlying io error or exit status rather than only 'claude --version failed', or removes the source of the failure, and a test or the test body covers it. Check by reading the diff in xtask/src/main.rs.
Run `cargo test -p xtask --bin xtask` and `cargo test -p athena-cli --lib` as separate commands; both pass, including each named test, which runs in the parallel binary with default threads.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` as separate commands; all pass, or a passing Athena-recorded check on the same final tree is cited (D115).
No unrelated behavior changes: `git diff --stat` lists only the deliverables; production code of drain.rs and xtask (outside test modules and any minimal error-reporting change in found/repin_at) is unchanged; athena.toml, defaults.toml, workflows and records/ are untouched.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
