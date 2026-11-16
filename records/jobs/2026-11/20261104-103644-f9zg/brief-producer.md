# Job 20261104-103644-f9zg: Report per-role spend, time, rounds and cap stops over last 30 jobs

## Request (verbatim)

> For every finished job, record per role (reader, producer, verifier, planner, critic) the USD spent, seconds elapsed and planning rounds used, next to the configured cap, plus which cap ended the run, if any. Use the existing typed job records. Add a read-only report command that prints, per role over the last 30 jobs, the median and 90th percentile of spend and time, and how many runs and what share of runs each cap stopped. Acceptance: tests with fixture records built from real job records show correct numbers, including a planner budget stop and a producer deadline stop. Do not change any cap values in athena.toml in this job.
> 
> Acceptance:
> - The change is made as described: Agent spend, time and round caps are raised by hand after repeated cap stops, with no record of which cap ended each job
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: athena.toml

## Reading (accepted by the user)

Add a read-only `athena` report command that, from existing typed job and invocation records, prints per role (reader, producer, verifier, planner, critic) the median and p90 of USD and seconds over the last 30 finished jobs, with the configured cap and the count and share of runs each cap stopped. Fixture-based tests cover a planner budget stop and a producer deadline stop. Audience: the owner, who tunes caps by hand and needs to see which cap ended each job. No cap value in athena.toml changes.

Assumptions:
- Code survey: read crates/athena-data/src/records.rs (Job.caps: Caps with job_usd, reader/producer/verifier_usd and timeouts; Invocation with role, caps: InvocationCaps{max_budget_usd,timeout_s}, cost, wall_time, outcome: InvocationOutcome Killed (budget) / TimedOut (deadline), detail; OutcomeState::TimedOut), crates/athena-cli/src/main.rs (clap Command enum dispatching to commands::*, e.g. Status, Audit), crates/athena-core/src/status.rs and status/planning.rs (existing planning-round aggregation to reuse). The new records need no new file type, so the records validation allow-list (AnyRecord, check_record_file and the round-trip test) is unchanged unless a field is added to an existing record; any added field must be Option with serde(default, skip_serializing_if) so old records still parse under deny_unknown_fields.
- Existing code: extend crates/athena-core/src/status/ aggregation helpers and the athena-cli command table; derive the cap that ended a run from Invocation.outcome plus Invocation.caps and the planner's round limit (athena.toml [plan] max_rounds = 5) instead of inventing a new record family.
- Field meaning: Invocation.outcome Killed is documented as 'Terminated by a budget, the kill switch, or the death of its parent' and TimedOut as 'its own deadline passed; the detail names the deadline' (crates/athena-data/src/records.rs:137-140). Killed is therefore only a budget stop when detail names the budget; the report must classify by detail and not count kill-switch or parent-death kills as cap stops. Config: live athena.toml max_rounds = 5.
- Consumers: none found for a new config key; athena.toml is not a deliverable and no cap value changes there (the 'Files: athena.toml' line is read as the file whose caps are being observed, not edited).
- Planning rounds are taken from the existing planner/critic records of a job; roles that do not run for a job (for example planner on non-hard jobs) are omitted from that job's sample, not counted as zero.
- 'Last 30 jobs' means the 30 most recent jobs with a terminal outcome by dispatch time; fewer jobs print what exists, and zero jobs print an explicit 'no finished jobs' message, not an empty success.
- Fixture records are built by copying the shape of real job, invocation and outcome records already used as fixtures in crates/athena-core tests, and not written from scratch.

## Acceptance criteria

1. [behavior] Run `cargo test`: new tests feed fixture records (shaped from real job records) with a planner budget stop and a producer deadline stop. They assert exact median and p90 USD and seconds per role, and exact stop counts and shares per cap (for example 1 of N runs = the computed percent). The tests fail if cap-stop classification is removed.
2. [behavior] Run `cargo run -- <report command> --help` (the new subcommand name, as added to the Command enum in crates/athena-cli/src/main.rs). It lists the command and shows it takes no write flags. The report test also asserts the records directory is byte-identical before and after a run.
3. [content] The report output has one row per role (reader, producer, verifier, planner, critic) showing runs, median and p90 USD, median and p90 seconds, the configured cap (USD, seconds, rounds), and for each cap the count and percent of runs it stopped. Show it in a test's expected-output string.
4. [content] A Killed invocation whose detail is not a budget stop (kill switch, parent death) is not counted as a cap stop; a test covers this case.
5. [content] The window is the last 30 finished jobs; a test with more than 30 fixture jobs shows older ones excluded, and a test with zero jobs shows the explicit no-finished-jobs message.
6. [content] Any field added to an existing record type is optional with serde default so existing records still parse; the existing records round-trip tests in crates/athena-data pass unchanged.
7. [content] `git diff` shows athena.toml unchanged; no cap value is edited anywhere in the change.
8. [content] CHANGELOG.md has a new entry for the report command, and docs name the command (README.md or the docs page that lists commands).
9. [behavior] Athena's recorded `cargo fmt --check`, `cargo clippy` and `cargo test` results on the final tree pass, and all pre-existing tests still pass.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/status/caps.rs (create)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Changing any cap value in athena.toml
- Changing how caps are enforced or how runs are stopped
- Backfilling or rewriting existing records
- Writing to records/ or any other state from the report command

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests with fixture records plus cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check computed numbers from fixtures, not the existence of the code.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One report command reusing existing records and status aggregation.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog entry written for an outside reader.
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

- path: <state>/jobs/20261104-103644-f9zg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261104-103644-f9zg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test`: new tests feed fixture records (shaped from real job records) with a planner budget stop and a producer deadline stop. They assert exact median and p90 USD and seconds per role, and exact stop counts and shares per cap (for example 1 of N runs = the computed percent). The tests fail if cap-stop classification is removed.
Run `cargo run -- <report command> --help` (the new subcommand name, as added to the Command enum in crates/athena-cli/src/main.rs). It lists the command and shows it takes no write flags. The report test also asserts the records directory is byte-identical before and after a run.
The report output has one row per role (reader, producer, verifier, planner, critic) showing runs, median and p90 USD, median and p90 seconds, the configured cap (USD, seconds, rounds), and for each cap the count and percent of runs it stopped. Show it in a test's expected-output string.
A Killed invocation whose detail is not a budget stop (kill switch, parent death) is not counted as a cap stop; a test covers this case.
The window is the last 30 finished jobs; a test with more than 30 fixture jobs shows older ones excluded, and a test with zero jobs shows the explicit no-finished-jobs message.
Any field added to an existing record type is optional with serde default so existing records still parse; the existing records round-trip tests in crates/athena-data pass unchanged.
`git diff` shows athena.toml unchanged; no cap value is edited anywhere in the change.
CHANGELOG.md has a new entry for the report command, and docs name the command (README.md or the docs page that lists commands).
Athena's recorded `cargo fmt --check`, `cargo clippy` and `cargo test` results on the final tree pass, and all pre-existing tests still pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
