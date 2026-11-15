# Job 20261114-045733-smbn: Prove the job budget reserve is checked before every spend, incl.

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Athena gives each job a fixed budget and only checks it after planning has already spent from it. Ad
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: job-budget-too-small-for-planning-plus-producer-verifier-caps-fails-before-code (20261028-091806-tng9)
> Cause: Athena gives each job a fixed budget and only checks it after planning has already spent from it. Admission and retry do not compare the remaining budget with the cap the producer and verifier need, which is fixed at $5.75. Planning rounds,...
> Seen 7 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261028-091754-xkyn (job 20261028-082151-67fa); prevention it proposed: Add a budget pre-check at admission, at retry and before each planning round. It should compute the reserve the producer and verifier need (the sum of their caps) and refuse to start or continue a planning round if the remaining budget minu...
> Introspection 20261030-151759-gmz2 (job 20261030-033434-3v30); prevention it proposed: Add a budget preflight in code. At admission, at retry, and before drain dispatches a job, compare the job's remaining budget with the planner cap plus the producer and verifier caps. If it falls short, do not start planning. Mark the job h...
> Introspection 20261030-153131-skzk (job 20261030-151827-48rm); prevention it proposed: Add one check, run at admission, when the queue picks a job, and before any retry or re-ask. It compares the job's remaining budget with the planning cost plus the producer and verifier caps. If the budget is short, it holds the job before ...
> Introspection 20261030-153335-xdtk (job 20261030-151810-0n3b); prevention it proposed: Add a code check that runs when a job fails at its budget cap, and again before a retry. It compares the remaining budget with the stage caps. If the cap is the cause, drain should hold that job with a clear reason, record the cap as the bl...
> Introspection 20261030-165437-3n07 (job 20261030-033434-3v30); prevention it proposed: Add a check at admission, before each retry and after each planning round. It compares the job's remaining budget with the producer cap plus the verifier cap. If the budget is short, it holds the job with a clear "budget below stage caps" r...
> Introspection 20261030-214133-cp86 (job 20261030-151818-2c8g); prevention it proposed: Define the reserve in one function that sums the stage caps from config, as planner + critic per round for max_rounds, plus producer and verifier. Use that same function in admission, retry, dispatch and the per-round check. Add a test that...
> Introspection 20261103-055632-pxgr (job 20261103-055218-kj1t); prevention it proposed: Before dispatching a retry, and at admission, check the job's remaining budget (cap minus all earlier attempts and planning spend) against the planning stage cap plus the producer and verifier caps. If it falls short, hold the job and show ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

The root cause (per-invocation budget check made after planning spent, with a reserve that left out the critic and later rounds) is already fixed in the repo by `reading::budget_reserve` and `Athena::preflight`. This change closes the remaining gap: a test per entry point (admission, retry, dispatch, planner, critic, run, drain, and a resumed job with prior spend) that fails when its preflight is removed. It supersedes the stalled jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm; the audience is the operator and verifier.

Assumptions:
- Code survey: read crates/athena-data/src/config.rs (planning_usd, critic_usd, run_usd, budget_required, check_budget at 894-949), crates/athena-core/src/reading.rs (ReserveStage and budget_reserve 94-140, check_budget and preflight 467-503, first_reading 1187 with preflight at 1203, retry 1262 with preflight at 1303), crates/athena-core/src/run/mod.rs (dispatch preflight 1108, Run preflight 1367), crates/athena-core/src/run/planning.rs (preflight at 116 and 336), crates/athena-cli/src/drain.rs (budget short hold, doc at 266). The entry points are Athena::ask, retry, go and drain, and the order is admission, dispatch, each planner and critic, then run. No record or data file under crates/ is added, so no records allow-list or AnyRecord round-trip change. The property is measured by comparing the job's remaining budget (job_usd minus spent) with budget_reserve at each stage, and by the error type BudgetShort.
- Current state: CHANGELOG.md 2184-2200 already records the root-cause fix and the per-stage preflight, and athena.toml has job_usd = 50.00 against a reserve of 2 + planning + 15, so the shipped config no longer reproduces the $8.00 failures. The three previous jobs (20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm) stopped in planning with no candidate and no changed files. Their critic findings concern different features (an interrupted-job takeover, drain ordering), not this cause, so they no longer apply to this request. Their file scopes (run/mod.rs, disk.rs, manage.rs, drain.rs, plan.rs and the athena.toml budget) are excluded here except where listed as deliverables, and their work is dropped.
- Existing code: this extends reading::budget_reserve, Athena::preflight, ReserveStage and BudgetShort in crates/athena-core/src/reading.rs and Config::planning_usd and budget_required in crates/athena-data/src/config.rs. It builds no second reserve computation.
- The gap to close, if the audit finds one, is a path where spend happens without a preflight. The likely places are a resumed or interrupted job whose earlier spend is not counted, a retry judged at spend 0 while its seeded stage already ran, and a drain dispatch. If the audit finds no gap, the change is tests, a short doc note and a CHANGELOG entry only.
- Expected newly refused readings: 0%. The shipped job_usd of 50.00 exceeds the largest reserve, so no past accepted reading is refused. No new refusal rule is added. The preflight already exists, and the change only proves it and extends it to any uncovered entry point.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry naming the root cause (the budget checked per invocation, after planning had spent, with a reserve that omitted the critic and later rounds) and jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm, and stating that 0% of past accepted readings are newly refused.
2. [content] The reserve for every ReserveStage is still computed only in reading::budget_reserve, from Config methods. Grep for `planner_usd + ` or `producer_usd + ` under crates/athena-core/src shows no second sum of caps outside Config.
3. [behavior] A new test per entry point (ask, retry, dispatch, planner round, critic round, run stage, drain, and a resumed job with prior spend) gives a job a budget one cent below budget_reserve for that stage, asserts a BudgetShort error before any invocation is recorded, and asserts success at exactly the reserve. Run `cargo test` and check that these tests pass.
4. [behavior] The cause is tested, not one job: a table test varies each cap in a Config (reader, planner, critic, producer, verifier, max_rounds) and asserts the reserve and the refusal threshold move by exactly that amount at every stage. Removing the preflight call in any one entry point (shown by the verifier running `cargo test` on a mutated tree, or by reading the test) makes its test fail. Run `cargo test`.
5. [behavior] Through the drain entry point, a queued item whose remaining budget is below the reserve is left queued with one `item <id> is left queued` line and outcome `budget short`, and the next affordable item still runs. Run `cargo test -p athena-cli`.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, as a recorded Athena check on the final tree.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Changes to athena.toml, defaults.toml, .github/workflows or records/; the [budget] values stay as they are
- The interrupted-job takeover and the retry head-staleness work from job 20261028-082151-67fa (run/mod.rs timeout_seed, disk.rs, manage.rs)
- Drain ordering and plan.rs changes from jobs 20261030-033434-3v30 and 20261030-151827-48rm
- Any new refusal rule beyond the existing preflight; changing the producer, verifier or planner prompts
- A partial-plan resume for the planner

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that a short budget is held before spend, not that a function exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One reserve function and a cap-varying table test cover the whole class.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test pass is a criterion.
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

- path: <state>/jobs/20261114-045733-smbn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045733-smbn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md has a new entry naming the root cause (the budget checked per invocation, after planning had spent, with a reserve that omitted the critic and later rounds) and jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm, and stating that 0% of past accepted readings are newly refused.
The reserve for every ReserveStage is still computed only in reading::budget_reserve, from Config methods. Grep for `planner_usd + ` or `producer_usd + ` under crates/athena-core/src shows no second sum of caps outside Config.
A new test per entry point (ask, retry, dispatch, planner round, critic round, run stage, drain, and a resumed job with prior spend) gives a job a budget one cent below budget_reserve for that stage, asserts a BudgetShort error before any invocation is recorded, and asserts success at exactly the reserve. Run `cargo test` and check that these tests pass.
The cause is tested, not one job: a table test varies each cap in a Config (reader, planner, critic, producer, verifier, max_rounds) and asserts the reserve and the refusal threshold move by exactly that amount at every stage. Removing the preflight call in any one entry point (shown by the verifier running `cargo test` on a mutated tree, or by reading the test) makes its test fail. Run `cargo test`.
Through the drain entry point, a queued item whose remaining budget is below the reserve is left queued with one `item <id> is left queued` line and outcome `budget short`, and the next affordable item still runs. Run `cargo test -p athena-cli`.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, as a recorded Athena check on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
