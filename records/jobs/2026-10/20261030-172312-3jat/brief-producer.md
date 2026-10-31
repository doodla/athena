# Job 20261030-172312-3jat: Make plans name each demonstration's test seam and check it exists

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The planning stage writes demonstrations for each criterion without checking that the code under tes
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-demonstration-assumes-nonexistent-test-seam-fails-planning-rounds (20261018-113458-gqky)
> Cause: The planning stage writes demonstrations for each criterion without checking that the code under test can be driven that way. Nothing in the plan template makes it name the seam, fixture or harness each demonstration uses, or confirm that i...
> Seen 5 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261018-113436-cxg9 (job 20261018-095726-r3bn); prevention it proposed: Change the plan template and planning flow in three ways. (1) Each demonstration must name its test seam: the exact function or constructor it calls, how the fake or fixture is injected, and whether that seam exists today or is a planned ch...
> Introspection 20261018-114052-07tm (job 20261018-113512-gvcs); prevention it proposed: Change the plan template and the planning gate. (1) Each demonstration must name its test seam: the fixture or harness, the production function it runs, and whether the seam exists or which step adds it. Logic under test must not sit behind...
> Introspection 20261024-115821-jzkw (job 20261024-115052-bzw6); prevention it proposed: Change the planning stage in three ways. (1) The plan template requires, for each criterion's demonstration, the production entry point called, the fake or seam used, and a check that the seam exists in the code (or the step that adds it). ...
> Introspection 20261025-012621-vezp (job 20261025-001234-2hax); prevention it proposed: Change the planning template and the planning loop. (1) For each criterion the plan must name the production entry point its demonstration calls, that entry point's preconditions, the seam or fake it uses, and the merge method and commit sh...
> Introspection 20261026-024143-4yat (job 20261026-015419-4j37); prevention it proposed: Give the planning stage a required step for each criterion's demonstration. It must name the entry function, quote the state guards and thresholds on the path (such as go refusing Dispatched, or t3_cost needing p90 > 0), and show that the s...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a plan demonstration (DemonstrationOutput) has only method, command, observes and fails_without. Nothing makes the planner name the entry point, fake or harness it drives, or whether that seam exists, so critics catch nonexistent seams only in round 3. Fix it at the plan schema and the planning gate: add a required `seam` field for behavior criteria, reject plans without it, and show it to the critic. Jobs 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6 stopped at planning round 3 with no candidate. Their code changes are dropped and not redone here; their seam findings are the class this change prevents.

Assumptions:
- Code survey: entry point is crates/athena-core/src/run/planning.rs, where PlanOutput becomes a JobPlan (the function around line 500 that maps DemonstrationOutput to Demonstration), so the new check runs after the planner output parses and before the critic sees the plan. The output schema is crates/athena-context/src/outputs.rs (DemonstrationOutput, line 418), and the critic and planner briefs render demonstrations in crates/athena-context/src/briefs.rs (line 1175).
- Code survey: the stored Demonstration record is in crates/athena-data/src/records.rs (line 2345). The change adds an optional `seam` field there with a serde default, so old plan records still parse. That file is accepted by `check_record_file` in crates/athena-data/src/records.rs, which crates/athena-core/src/run/tests.rs:1117 and crates/athena-cli/tests/lifecycle.rs:71 call, and by the `AnyRecord` round-trip; the new field must pass both, with an old record lacking it still valid.
- Code survey: the property is measured directly by a unit test in crates/athena-core/src/run/tests.rs that calls the production plan-conversion and validation function with a plan whose behavior demonstration has no seam, and one that has it. It does not use a stand-in.
- Audience: the planner, which gets the new required field, and the critic and verifier, who read it. Old stored plans without `seam` still load.
- Seam is required only when a criterion is a behavior criterion (the same condition as fails_without). It states the production entry point, the fake or fixture and how it is injected, and either 'exists' or the plan step that adds it. Whether the seam exists in the code is checked by the critic from the rendered field, not by Athena mechanically.
- Prompt files are out of the crates/ scope given and the planner prompt has room (494 tokens), but the schema field description carries the instruction, so no prompt edit is planned.

## Acceptance criteria

1. [content] The change description names the root cause: plan demonstrations carry no field for the test seam, so the planning flow never requires naming or confirming the seam. Cites jobs 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6.
2. [content] DemonstrationOutput in crates/athena-context/src/outputs.rs and Demonstration in crates/athena-data/src/records.rs gain a `seam` field, with a doc comment naming entry point, fake injection and exists-or-added-by-step. The records field has a serde default so old records parse.
3. [content] crates/athena-context/src/briefs.rs renders the seam beside each demonstration, so the critic sees it (check with Grep for the seam label near the Demonstrations: heading).
4. [behavior] Run `cargo test -p athena-core`: a new test shows planning refuses a plan whose behavior-criterion demonstration has an empty or missing seam, with an error naming the criterion, and accepts the same plan with a seam. The test fails on the code before the change.
5. [behavior] Run `cargo test -p athena-data` and `cargo test -p athena-cli`: a plan record without `seam` still passes check_record_file and round-trips as AnyRecord, and a record with the seam round-trips unchanged.
6. [behavior] The change refuses more plans, and the higher refusal rate is intended: expected 10% of newly refused plans, namely behavior criteria with no named seam.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root; all pass.
8. [content] CHANGELOG.md has an entry for the new required seam field and the planning refusal.

## Expected deliverables

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/store.rs (modify)

## Out of scope

- Re-attempting the work of jobs 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6 (merge/resync, records-branch landing). Their candidates changed no files, so nothing is kept.
- Changing the three-round planning cap or the critic's severity rules
- Mechanically proving a seam exists in the target code
- Changes to athena.toml, defaults.toml, CI workflows or other governance files
- Edits to records/ or owner files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the plan schema and gate rather than one job's plan.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run a test that refuses a seamless plan.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a validation in planning, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused schema and gate change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 7 cases in 7 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Land b5hm first and extend it. Before the checks stage, and again when free disk drops below the guard, Athena should delete regenerable artifacts itself: the verify-side build folder of finished baseline checks, the workspace target, and the uv and npm caches. If that is still not enough, the job s...

22 more qualifying classes are left out.

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

The change description names the root cause: plan demonstrations carry no field for the test seam, so the planning flow never requires naming or confirming the seam. Cites jobs 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6.
DemonstrationOutput in crates/athena-context/src/outputs.rs and Demonstration in crates/athena-data/src/records.rs gain a `seam` field, with a doc comment naming entry point, fake injection and exists-or-added-by-step. The records field has a serde default so old records parse.
crates/athena-context/src/briefs.rs renders the seam beside each demonstration, so the critic sees it (check with Grep for the seam label near the Demonstrations: heading).
Run `cargo test -p athena-core`: a new test shows planning refuses a plan whose behavior-criterion demonstration has an empty or missing seam, with an error naming the criterion, and accepts the same plan with a seam. The test fails on the code before the change.
Run `cargo test -p athena-data` and `cargo test -p athena-cli`: a plan record without `seam` still passes check_record_file and round-trips as AnyRecord, and a record with the seam round-trips unchanged.
The change refuses more plans, and the higher refusal rate is intended: expected 10% of newly refused plans, namely behavior criteria with no named seam.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root; all pass.
CHANGELOG.md has an entry for the new required seam field and the planning refusal.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
