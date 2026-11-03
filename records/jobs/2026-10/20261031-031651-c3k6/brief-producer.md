# Job 20261031-031651-c3k6: Refuse plan open questions that reading the code would settle

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The planner designs from the reading and from the code paths it first finds. It does not enumerate e
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-leaves-code-answerable-questions-open-and-skips-adjacent-existing-path-fails-planning-rounds (20261031-030110-hnnx)
> Cause: The planner designs from the reading and from the code paths it first finds. It does not enumerate every existing caller and mode that reaches the function it changes, such as the `[parallel] max_concurrency` path into landing. It also may ...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261031-030054-6wkt (job 20261030-013555-f5qm); prevention it proposed: Add a plan-structure check that runs before each critic round. (a) A plan may not list an open question that is marked answerable by reading code. The planner must settle it by reading the code and cite the file and line. (b) Any plan that ...
> Introspection 20261031-031632-fdjf (job 20261031-030119-tw3v); prevention it proposed: Add a required step before the critic in planning for any change to a core flow such as landing. The planner must list every state the change touches, including records and job state read from the landing branch, and every existing caller a...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: Athena's own plan checks (plan_findings in admission.rs) accept open questions the planner could settle by reading code (e.g. whether a path lands today, or what an old job's records look like), so the critic rounds were spent on them; jobs 20261030-013555-f5qm and 20261031-030119-tw3v both stopped at round 3 this way. Fix: add a mechanical major finding in plan_findings, run before each critic round, for code-answerable open questions, and tell the planner in its brief to settle them with a file and line.

Assumptions:
- Code survey: read crates/athena-core/src/admission.rs (plan_findings at line 170 and dependents_findings at 314, which already run before each critic call), crates/athena-core/src/run/planning.rs (planning_round runs plan_findings, then up to MAX_MECHANICAL_PASSES revisions with no critic call), crates/athena-data/src/records.rs (JobPlan has open_questions: Vec<String> and dependents; no new record or data file is added, so check_record_file/AnyRecord/round-trip tests are unaffected). Entry point for the test: plan_findings with a JobPlan built like the existing dependents_findings_for fixture. The property is measured directly on the Finding list it returns.
- Job 20261030-013555-f5qm (finding: a plan left code-answerable questions open and skipped the [parallel] max_concurrency path into landing): addressed by the rule that refuses code-answerable open questions, and a dependents check that a plan for a core-flow file names every caller/mode; that job's own landing work is not redone here and its scope (drain.rs, landing.rs, CHANGELOG.md, tests/drain.rs, async-drain.md) is excluded; it changed no files, so nothing is kept or dropped.
- Job 20261031-030119-tw3v (finding: the plan left record-loss and ordering questions open): same fix, and it no longer applies as a retry; the same scope is excluded and it changed no files.
- Existing code: crates/athena-core/src/admission.rs, crates/athena-core/src/run/planning.rs, crates/athena-context/src/briefs.rs, docs/design/job-planning.md.
- The detection is a deterministic heuristic: an open question that cites a source path (a crates/ path or a .rs file or file:line) or says the answer is found by reading or checking the code is code-answerable. Questions for the user that name no code path stay allowed.
- The planner brief in briefs.rs gains one short sentence (planner has 334 tokens of headroom); the critic and other prompt sets are not touched.
- Check (b) from the introspections is kept small: when a plan touches a file whose dependents section is present, no new schema field is added; the existing dependents rule and brief wording carry the 'list every caller and mode' instruction.

## Acceptance criteria

1. [content] The change description names the root cause: plan_findings in crates/athena-core/src/admission.rs let code-answerable open questions through to the critic, and the planner brief did not require settling them by reading code; Grep finds the doc comment for the new rule in admission.rs.
2. [behavior] Run `cargo test -p athena-core admission`: a new test builds a plan whose open_questions contains a question citing a crates/ path (e.g. whether `crates/athena-cli/src/main.rs` passes max_concurrency into landing) and asserts plan_findings returns a Major finding located at open questions naming the question; the test fails on the base commit and passes with the change.
3. [behavior] Run `cargo test -p athena-core admission`: a new test shows a plan whose open question is a user decision naming no code path produces no open-question finding, and a plan with no open questions is unchanged, so the rule is not a blanket refusal.
4. [behavior] Run `cargo test -p athena-core run`: a planning test shows a plan with a code-answerable open question is sent back to the planner without a critic call (mechanical pass), not counted as a critic round.
5. [content] The planner brief in crates/athena-context/src/briefs.rs tells the planner to settle questions answerable from code by reading it and citing file and line, and to list every existing caller and mode that reaches a changed function; docs/design/job-planning.md and CHANGELOG.md describe the new rule.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including prompt token budget tests for the planner set.
7. [content] The change refuses more plans: expected 5% of newly refused plans, and the higher refusal rate is intended because those plans spent critic rounds on questions code could answer.

## Expected deliverables

- crates/athena-core/src/admission.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Landing queue work of jobs 20261030-013555-f5qm and 20261031-030119-tw3v (drain.rs, landing.rs, tests/drain.rs, async-drain.md): not redone here; those jobs changed no files.
- Changing athena.toml, defaults.toml, .github/workflows, or the JobPlan record schema in crates/athena-data
- Adding model calls or changing the critic prompt
- Conflict return, send_back and old-job record handling

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The rule and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the new rule; fmt, clippy and test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the shared plan_findings gate, not for one job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a code check, not prose only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run plan_findings on real plans.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule plus one brief sentence.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 41 cases in 41 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a different value, and the test must assert that each correspon...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

26 more qualifying classes are left out.

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

The change description names the root cause: plan_findings in crates/athena-core/src/admission.rs let code-answerable open questions through to the critic, and the planner brief did not require settling them by reading code; Grep finds the doc comment for the new rule in admission.rs.
Run `cargo test -p athena-core admission`: a new test builds a plan whose open_questions contains a question citing a crates/ path (e.g. whether `crates/athena-cli/src/main.rs` passes max_concurrency into landing) and asserts plan_findings returns a Major finding located at open questions naming the question; the test fails on the base commit and passes with the change.
Run `cargo test -p athena-core admission`: a new test shows a plan whose open question is a user decision naming no code path produces no open-question finding, and a plan with no open questions is unchanged, so the rule is not a blanket refusal.
Run `cargo test -p athena-core run`: a planning test shows a plan with a code-answerable open question is sent back to the planner without a critic call (mechanical pass), not counted as a critic round.
The planner brief in crates/athena-context/src/briefs.rs tells the planner to settle questions answerable from code by reading it and citing file and line, and to list every existing caller and mode that reaches a changed function; docs/design/job-planning.md and CHANGELOG.md describe the new rule.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including prompt token budget tests for the planner set.
The change refuses more plans: expected 5% of newly refused plans, and the higher refusal rate is intended because those plans spent critic rounds on questions code could answer.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
