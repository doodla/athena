# Job 20261107-110045-yp1e: Enforce realistic-branch fixtures for landing/history readings by check

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The planning stage derives a commit-shape rule from the intended happy path instead of from the full
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-tested-only-against-job-commits-not-realistic-branch-history (20261024-140616-afw6)
> Cause: The planning stage derives a commit-shape rule from the intended happy path instead of from the full set of shapes the system produces. Here it missed the retry-after-block landing. The plan's demonstrations also use a fixture that does not...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261024-154443-51wt (job 20261024-153908-1rve); prevention it proposed: Change the plan template and critic checklist for any landing or history-check rule. The plan must include an enumerated table of every commit shape the real branch contains. The table must cover job commits, records-only commits, owner-pat...
> Introspection 20261024-161503-q0pp (job 20261024-154552-e65v); prevention it proposed: Before handover, run a check that runs every test in each crate that calls the changed history or classification functions. For this change that means `athena-core` reconcile and `athena-cli --test reconcile`, not only `land::`. The check s...
> Introspection 20261025-012736-2q6w (job 20261025-012657-hac3); prevention it proposed: Add a default to the reading stage. Any criterion on a landing, history or attribution check must name a test fixture that uses a realistic branch. That branch has at least one intervention commit, one records-only commit and the job landin...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (jobs 20261024-153908-1rve, 20261024-154552-e65v, 20261025-012657-hac3; introspections 51wt, q0pp, 2q6w): landing, history and attribution rules are derived from the happy-path commit shape, and the guard against it exists only as planner/critic prose. Fix at the reading stage with a mechanical check in reading.rs, added to refusing_check, that refuses such readings unless a criterion names a realistic-branch fixture with an intervention commit, a records-only commit and the job landing. Earlier jobs' source changes are not retried: that work is dropped.

Assumptions:
- Code Survey placeholder replaced below.
- Code survey: read crates/athena-core/src/reading.rs: refusing_check (about line 3227) runs check_prior_result_and_scope, check_criteria_evidence, check_code_survey (line 3912) and the later checks in order and returns the first failure by name; check_code_survey is the model (applies only to code/tests readings with a behavior criterion, reads assumptions and deliverables, never the repo). Also read crates/athena-context/prompts/planner.md line 16 and critic.md line 14, which already hold the commit-shape table and realistic-fixture guidance as prose only, and crates/athena-context/src/lib.rs test landing_rules_need_a_commit_shape_table_and_a_realistic_fixture. No record or data file is added, so check_record_file and AnyRecord are not involved; the property is measured directly by calling the new check on Reading values.
- Goal and audience: the Athena operator and the planner/verifier stages; the goal is that landing, history and attribution rules are no longer tested only against job commits. The check triggers when deliverables include a landing/history/attribution source (land.rs, history.rs, attribution.rs, or gh.rs check functions) or a criterion text mentions landing, history check or attribution check, and the reading is code/tests with a behavior criterion.
- Environment: none; the check is pure over the Reading, with no model call, no network, no repository read and no write.
- Prior jobs: 20261024-153908-1rve and 20261024-154552-e65v changed (or tried to change) land.rs, gh.rs, invariant.rs, records-history.md and CHANGELOG.md; 20261025-012657-hac3 changed commits.rs, manage.rs, athena-git, status.rs, auditor.rs, athena-gate files, decisions.md. Their specific pr_commits and trailer-cutover candidates are dropped, not retried: their findings (reconcile fixtures not updated, operator/ fixture, missing post-C0 tests) are the instances, and this job fixes the class they share rather than re-landing them.
- The reader prompt has 3 tokens of headroom, so no reader prompt growth; the planner (175 left) and critic prompts already carry the guidance and are left unchanged. The existing mechanical gap is the only thing added.

## Acceptance criteria

1. [content] crates/athena-core/src/reading.rs gains a pub fn check_history_fixture(reading: &Reading) -> Result<()> whose doc comment names the root cause (rules derived from the happy-path commit shape, jobs 20261024-153908-1rve, 20261024-154552-e65v, 20261025-012657-hac3) and states 'expected N% of newly refused readings' with a whole-percent N.
2. [content] refusing_check in reading.rs calls the new check with step!(check_history_fixture(reading)), so the first-failure name and order stay derived from stringify!.
3. [behavior] Run `cargo test -p athena-core reading`: a new unit test shows a code reading with a behavior criterion for a landing/history/attribution rule and no criterion naming a realistic-branch fixture with an intervention commit, a records-only commit and the job landing is refused with an error that says what to add; the same test shows the reading is accepted once such a criterion exists. The refusal test fails if the check is removed from refusing_check.
4. [behavior] Run `cargo test -p athena-core reading`: a test shows readings unrelated to landing/history/attribution (docs, prompts, a code reading touching none of those) are not refused by the new check, and the existing replay/refusal-rate tests still pass.
5. [content] Any reading fixtures in crates/ tests that legitimately touch landing or history rules are updated with the realistic-fixture criterion so existing tests stay valid, and the CHANGELOG.md entry names the root cause.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass (the tested default).
7. [content] The change description names the root cause and explains why it was fixed at the reading-stage check, where readings are accepted, not at one pr_commits or trailer instance.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Re-landing the pr_commits content classification (land.rs, gh.rs, invariant.rs) from jobs 20261024-153908-1rve and 20261024-154552-e65v; that candidate is dropped
- Re-landing the trailer cutover work (commits.rs, manage.rs, athena-git, status.rs, auditor.rs, athena-gate files, decisions.md, lifecycle tests) from job 20261025-012657-hac3; that candidate is dropped
- Editing reader, planner or critic prompt files (reader has 3 tokens left; planner and critic already carry the guidance)
- Changing athena.toml, defaults.toml or CI workflows
- Running the full replay beyond the existing reading tests

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New check has unit tests; fmt, clippy and test criterion.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The prose-only planner/critic rule gets a mechanical enforcement in refusing_check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the reading gate that all landing/history jobs pass through.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test to see refusal and acceptance.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and changelog explain the cause.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

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

- path: <state>/jobs/20261107-110045-yp1e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-110045-yp1e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading.rs gains a pub fn check_history_fixture(reading: &Reading) -> Result<()> whose doc comment names the root cause (rules derived from the happy-path commit shape, jobs 20261024-153908-1rve, 20261024-154552-e65v, 20261025-012657-hac3) and states 'expected N% of newly refused readings' with a whole-percent N.
refusing_check in reading.rs calls the new check with step!(check_history_fixture(reading)), so the first-failure name and order stay derived from stringify!.
Run `cargo test -p athena-core reading`: a new unit test shows a code reading with a behavior criterion for a landing/history/attribution rule and no criterion naming a realistic-branch fixture with an intervention commit, a records-only commit and the job landing is refused with an error that says what to add; the same test shows the reading is accepted once such a criterion exists. The refusal test fails if the check is removed from refusing_check.
Run `cargo test -p athena-core reading`: a test shows readings unrelated to landing/history/attribution (docs, prompts, a code reading touching none of those) are not refused by the new check, and the existing replay/refusal-rate tests still pass.
Any reading fixtures in crates/ tests that legitimately touch landing or history rules are updated with the realistic-fixture criterion so existing tests stay valid, and the CHANGELOG.md entry names the root cause.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass (the tested default).
The change description names the root cause and explains why it was fixed at the reading-stage check, where readings are accepted, not at one pr_commits or trailer instance.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
