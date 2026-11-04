# Job 20261104-081830-js1p: Plan jobs whose criteria ask for a 'fails without the fix' run

## Request (verbatim)

> Plan every job whose acceptance asks for a 'fails without the fix' run. Athena now runs planned 'fails without the fix' demonstrations and shows them to the verifier (landed 4 Nov, 64cb889), but only when the job has a plan; jobs judged easy skip planning (job_plan mode hard). Job 20261103-231422-8dzf had such a criterion, no plan, so no demonstration ran and the verifier rejected correct code a third time. When any accepted criterion asks for a failing run without the fix (the reading's behavior criteria with fails_without, or criterion text naming 'fails without the fix'), run the planner and critic even if the job is judged easy. Acceptance: a test that an easy job with such a criterion gets a plan and its demonstration record reaches the verifier brief; a test that an easy job without one still skips planning.

## Reading (accepted by the user)

Retry context: job 20261103-231422-8dzf was rejected because an easy job had a 'fails without the fix' criterion, got no plan, and so no demonstration reached the verifier. In dispatch, a job whose accepted criteria name a failing run without the fix is planned even in `hard` mode when the reading is not hard; ordinary easy jobs still skip planning. The audience is the verifier and the maintainer running the drain.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs computes `planned` from `job_plan.plans(reading.hard_reason.is_some())` and `!capability.pins_repo()`, then calls `plan_round` (crates/athena-core/src/run/planning.rs); `JobPlanConfig::plans` in crates/athena-data/src/config.rs holds the enabled/mode rule. Demonstrations are run in crates/athena-core/src/run/checks.rs and rendered for the verifier in crates/athena-core/src/verify.rs. The reading's Criterion (crates/athena-data/src/records.rs) has only `text` and `kind`, with no fails_without field; fails_without exists only on the plan's Demonstration, so the trigger is a Behavior criterion whose text names 'fails without the fix' (case-insensitive; also 'fails without the change/mechanism' variants if the producer judges them the same ask). The property is measured directly by running a job through `Athena::go` with the fake agents, as the existing demonstration tests in crates/athena-core/src/run/tests.rs do. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are untouched. The change adds no refusal rule, only widens planning.
- Job 20261103-231422-8dzf: its candidate's work (drain job-limit slot give-back in crates/athena-cli/src/drain.rs, tests/drain.rs, main.rs, and its CHANGELOG.md entry) is a different feature, retried as 20261103-193428-df2m; it is not part of this change and is dropped here. The finding that evidence was missing is addressed at its cause: the planning gate, not the candidate. Its dependency 20261103-231410-asf2 (landed 4 Nov, 64cb889) already runs and shows demonstrations, so it is not touched.
- The existing docs/design/job-planning.md section on modes (section 9) is updated to state the new rule; docs/job-planning.md does not exist.
- `mode = "all"` and `enabled = false` behave as today: disabled still plans nothing; planning is still skipped for a pinned-repo capability (study).

## Acceptance criteria

1. [behavior] A test in crates/athena-core/src/run/tests.rs (or crates/athena-cli/tests/job_plan.rs) runs an easy job (hard_reason null, `[job_plan] mode = "hard"`) whose Behavior criterion says a test 'fails without the fix': the planner and critic are called, and the verifier prompt contains the 'fails-without-1' record from Athena's revert run. Run `cargo test --locked -p athena-core run::tests` and `cargo test --locked -p athena-cli --test job_plan`.
2. [behavior] A test runs an easy job in `hard` mode with no such criterion and asserts no planner or critic call and no plan record, so ordinary easy jobs still skip planning. Run `cargo test --locked -p athena-core run::tests` and `cargo test --locked -p athena-cli --test job_plan`.
3. [behavior] The new 'plans' test fails without the fix: with the criterion check in crates/athena-core/src/run/mod.rs reverted to `plans(reading.hard_reason.is_some())` alone, `cargo test --locked -p athena-core run::tests` fails on it.
4. [content] The planning gate in crates/athena-core/src/run/mod.rs (grep `planned`) also holds when any accepted Behavior criterion text names 'fails without the fix', and the trigger is a small named helper with a doc comment, not inline string matching.
5. [content] docs/design/job-planning.md (an existing file) and CHANGELOG.md state that a job with a 'fails without the fix' criterion is planned even when judged easy.
6. [behavior] Outcome check: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass, and the existing mode tests in crates/athena-cli/tests/job_plan.rs still pass unchanged.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Job 20261103-231422-8dzf's candidate files crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs and crates/athena-cli/src/main.rs: the drain job-limit work is not part of this change and is dropped here.
- Changing how demonstrations are run or shown to the verifier (landed in 64cb889; job 20261103-231410-asf2).
- Changing the planner, critic or reader prompts.
- Governance files: athena.toml, defaults.toml and workflows; the repository's `mode = "hard"` setting stays.
- Adding a fails_without field to the reading's Criterion record.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for both branches; cargo test and clippy must be clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check a planned job reaching the verifier brief, not wiring.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The final criterion is the outcome check of fmt, clippy and the full test run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the planning gate, not a retry of the one job.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the gate plus tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the named cargo test commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 10 cases in 8 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code-checked evidence path for "fails without the fix" criteria. Athena would apply the reverse of the fix to a scratch copy, run the named test command, and record the output among the checks the verifier sees. The reading's criterion would then name the check, not a producer claim. Separatel...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

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

A test in crates/athena-core/src/run/tests.rs (or crates/athena-cli/tests/job_plan.rs) runs an easy job (hard_reason null, `[job_plan] mode = "hard"`) whose Behavior criterion says a test 'fails without the fix': the planner and critic are called, and the verifier prompt contains the 'fails-without-1' record from Athena's revert run. Run `cargo test --locked -p athena-core run::tests` and `cargo test --locked -p athena-cli --test job_plan`.
A test runs an easy job in `hard` mode with no such criterion and asserts no planner or critic call and no plan record, so ordinary easy jobs still skip planning. Run `cargo test --locked -p athena-core run::tests` and `cargo test --locked -p athena-cli --test job_plan`.
The new 'plans' test fails without the fix: with the criterion check in crates/athena-core/src/run/mod.rs reverted to `plans(reading.hard_reason.is_some())` alone, `cargo test --locked -p athena-core run::tests` fails on it.
The planning gate in crates/athena-core/src/run/mod.rs (grep `planned`) also holds when any accepted Behavior criterion text names 'fails without the fix', and the trigger is a small named helper with a doc comment, not inline string matching.
docs/design/job-planning.md (an existing file) and CHANGELOG.md state that a job with a 'fails without the fix' criterion is planned even when judged easy.
Outcome check: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass, and the existing mode tests in crates/athena-cli/tests/job_plan.rs still pass unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
