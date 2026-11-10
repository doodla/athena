# Job 20261028-140324-y4fm: Give a planning-round budget stop its own budget-reserve outcome

## Request (verbatim)

> Add a budget reserve check. I read this in code: check_budget in crates/athena-core/src/reading.rs checks only the next invocation's cap against what is left. Planning can therefore spend the budget down until the producer or verifier cannot start. That happened on 20261028-081326-zzxg and its retry 20261028-082151-67fa ($5.48 left, $5.75 needed). Change: at admission, at `athena retry`, and before each planning round, compare the remaining job budget with the sum of the producer and verifier caps plus the next planner call's cap. If the budget falls short, hold or refuse the job with a message that names the remaining budget, the reserve and the shortfall, and make no model call. Do not change the budget amounts in athena.toml. Acceptance: (1) a test with $5.48 left and a $5.75 reserve shows no planner call and the shortfall message; (2) a test with enough budget shows planning goes ahead; (3) a job that cannot afford another planning round ends with a distinct budget-reserve status, not a failed planning stage.
> 
> Acceptance:
> - The change is made as described: Planning rounds and retries spend the job budget below what the producer and verifier need, so the job fails unjudged and the operator re-asks it as a new request
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/reading.rs

## Reading (accepted by the user)

The reserve check already exists (budget_reserve and preflight in reading.rs, called at admission, in retry, and before each planner and critic call). The gap is acceptance (3): a job stopped by it before a planning round still closes as a planning-stage failure. I read the request as giving that stop a distinct budget-reserve outcome; jobs 20261028-081326-zzxg and 20261028-082151-67fa are the incidents it came from, and this change adds no code for their disk and resume work.

Assumptions:
- Code survey: I read crates/athena-core/src/reading.rs (budget_reserve, ReserveStage, BudgetShort, Athena::preflight, Athena::retry), crates/athena-core/src/run/planning.rs (stop_planning, plan_round preflight calls at the planner and critic) and crates/athena-core/src/run/mod.rs (preflight at Dispatch and Run). Admission uses Config::budget_required in crates/athena-data/src/config.rs. The entry points are admission, Athena::retry, dispatch, then plan_round. Existing tests: reading.rs shipped_budget_covers_the_reserve, run/tests.rs lines 1658 and 1709, run_pin_tests.rs line 994.
- Existing code: this extends crates/athena-core/src/reading.rs, crates/athena-core/src/run/planning.rs and crates/athena-data/src/records/introspection.rs (PLANNING_STOPPED_PREFIX). The reserve sum is not rebuilt; the check already compares the remaining budget with the producer, verifier and next planner caps.
- Job 20261028-081326-zzxg (disk and resume work) and job 20261028-082151-67fa (retry that stopped on budget) produced no candidate. Their findings concern disk cleanup and resuming from a kept candidate, which this request does not ask for, so they do not apply; the changed-file set of both is empty, so no earlier work is kept or dropped.
- Today the planning-round stop calls stop_planning, which closes the job with PLANNING_STOPPED_PREFIX, the planning-stage failure the request wants distinguished. I assume a new prefix constant for the budget-reserve outcome, which retry_findings and introspection treat as an environment-like hold and not as a failure of the work.
- Budget amounts in athena.toml stay unchanged.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-core: a test with $5.48 left and a $5.75 reserve before a planning round makes no planner call and closes with a message naming the remaining budget, the reserve and the shortfall ($0.27).
2. [behavior] Run cargo test -p athena-core: a test with enough budget shows the planner is invoked and planning goes ahead.
3. [behavior] Run cargo test -p athena-core: a job that cannot afford another planning round ends with a distinct budget-reserve outcome. Its reason does not start with PLANNING_STOPPED_PREFIX, and the test asserts that.
4. [content] Grep of crates/athena-core/src/run/planning.rs shows the preflight failure path uses the new budget-reserve stop and no longer calls stop_planning with the planning-stage prefix.
5. [behavior] Run cargo test -p athena-core: retry and admission checks still refuse a short budget before any model call (existing run_pin_tests.rs and reading.rs tests pass).
6. [content] git diff shows no change to athena.toml, and CHANGELOG.md has an entry for the new budget-reserve outcome.
7. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass, with no unrelated behavior changes.

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/retry.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing budget amounts in athena.toml
- Changing the 10 GB free-disk minimum, disk cleanup before the verifier, or resume from a kept candidate (jobs 20261028-081326-zzxg and 20261028-082151-67fa)
- Changing verifier behavior or prompts
- Adding a partial-plan resume for the planner

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the no-model-call hold and the distinct outcome by running tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo fmt, clippy and test are acceptance criteria.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
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

- path: <state>/jobs/20261028-140324-y4fm/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-140324-y4fm/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run cargo test -p athena-core: a test with $5.48 left and a $5.75 reserve before a planning round makes no planner call and closes with a message naming the remaining budget, the reserve and the shortfall ($0.27).
Run cargo test -p athena-core: a test with enough budget shows the planner is invoked and planning goes ahead.
Run cargo test -p athena-core: a job that cannot afford another planning round ends with a distinct budget-reserve outcome. Its reason does not start with PLANNING_STOPPED_PREFIX, and the test asserts that.
Grep of crates/athena-core/src/run/planning.rs shows the preflight failure path uses the new budget-reserve stop and no longer calls stop_planning with the planning-stage prefix.
Run cargo test -p athena-core: retry and admission checks still refuse a short budget before any model call (existing run_pin_tests.rs and reading.rs tests pass).
git diff shows no change to athena.toml, and CHANGELOG.md has an entry for the new budget-reserve outcome.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass, with no unrelated behavior changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
