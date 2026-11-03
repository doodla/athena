# Job 20261103-163929-b9pp: Pause an item that fails twice; stop drain only on shared causes

## Request (verbatim)

> try again on 20261103-162416-m9th

## Reading (accepted by the user)

Retry of job 20261103-162416-m9th (no candidate kept, nothing to carry over): in crates/athena-cli/src/drain.rs, `Drain::finish` stops the batch on any item's double failure. Change so one item's double failure pauses that item and the drain continues; stop with Stop::RepeatedFailure only on positive evidence of a shared cause (StopCause::of environment match, or typed host error). Audience: operator running `athena drain` with budget left.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (Drain::finish at ~2348 sets Stop::RepeatedFailure unconditionally for ItemEnd::Failed at ~2379; pass_over ~1368; landing_from_report ~3038; RealSteps ~3322; Stop docs ~944-1001; module header ~103), crates/athena-cli/tests/drain.rs (27 RepeatedFailure uses, drain::drain with Stub), display.rs, tests/job_plan.rs and steward.rs (one use each). The property is measured directly by running drain::drain with the Stub and inspecting the report and stub log. No record or data files are added, so no records validation change.
- Job 20261103-162416-m9th failed in planning in round 3 on one major finding: every run/read Err was classed as shared. Addressed here: a no-outcome Err is per item by default; it is shared only with positive evidence (StopCause::of(reason).is_environment(), or a typed host error by downcast as BudgetShort already is). Err sites to enumerate before classifying, from the code: go_with_stages in athena-core run/mod.rs (~340 declined, 348 running, 356 interrupted, 360 already closed), run/pin.rs ~150 readings cap, run/close.rs ~412 and ~348; all are item-specific. The landing error is classified by where it arises (landing_from_report Ok vs Err).
- The earlier job's candidate changed no files, so there is no work to keep or drop; the retry starts from base.
- Demonstrations use real error texts from those sites and production constants (e.g. athena_core::disk::USAGE_LIMIT, LOW_DISK, ENVIRONMENT_PREFIX), not invented text.
- The previous prevention (enumerate producers, repair round) is a planning-process matter, out of this change; this job only does the drain fix with the producer list done up front.

## Acceptance criteria

1. [behavior] New test in crates/athena-cli/tests/drain.rs: item a fails twice, and the drain then runs queued item b (stub log shows b after a's retry); report.stop is not Stop::RepeatedFailure and a is paused with its failure named. Run `cargo test -p athena-cli --test drain`.
2. [behavior] New test: two different items each fail twice with the item-specific real texts (`job X is already closed (rejected)`, `job X has N readings, the most a job may have`, declined, running, interrupted); both are paused and the drain finishes the rest of the queue without Stop::RepeatedFailure. Run `cargo test -p athena-cli --test drain`.
3. [behavior] New test: two different items fail twice with the same shared cause built from a production constant (athena_core::disk::USAGE_LIMIT or LOW_DISK, or an expired login text); the drain stops with Stop::RepeatedFailure naming an item and starts no further queued item. Run `cargo test -p athena-cli --test drain`.
4. [behavior] New test: a landing error is classified by where it arises (landing_from_report Ok vs Err), checked with the real landing_from_report. Run `cargo test -p athena-cli`.
5. [behavior] Unit test in drain.rs tests shows ids wrapped in backticks or parentheses are masked by the failure key, so such texts match. Run `cargo test -p athena-cli`.
6. [content] In crates/athena-cli/src/drain.rs, `Drain::finish` no longer sets Stop::RepeatedFailure unconditionally for ItemEnd::Failed; a lone failure goes through pass_over with a reason naming the item and its failures, and only evidence of a shared cause stops the batch.
7. [behavior] Existing drain tests asserting the old single-item stop are updated; `cargo test` passes for the workspace, `cargo clippy --all-targets` is clean, `cargo fmt --check` passes.
8. [content] Doc comments in drain.rs (module header ~103, Stop::RepeatedFailure ~944 and ~1001) and text in display.rs describe the new rule; CHANGELOG.md has an entry; the display still names the failed item and its rejection.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- docs (modify)

## Out of scope

- Changing the scope guard itself or how it pauses lines
- Changing the ATTEMPTS count or the reader's skip-after-two-unread rule
- Changes to athena.toml, defaults.toml, or workflows
- Changing environment stops such as rebuild failure, budget, job limit, or reinstall
- Changing athena-core error types or planning-stage process

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy, fmt checks required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run drain::drain with the Stub and real error texts.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks that the drain continues past a lone failing item and stops on shared cause.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the unconditional stop rule in Drain::finish.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced in code and tests.
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

New test in crates/athena-cli/tests/drain.rs: item a fails twice, and the drain then runs queued item b (stub log shows b after a's retry); report.stop is not Stop::RepeatedFailure and a is paused with its failure named. Run `cargo test -p athena-cli --test drain`.
New test: two different items each fail twice with the item-specific real texts (`job X is already closed (rejected)`, `job X has N readings, the most a job may have`, declined, running, interrupted); both are paused and the drain finishes the rest of the queue without Stop::RepeatedFailure. Run `cargo test -p athena-cli --test drain`.
New test: two different items fail twice with the same shared cause built from a production constant (athena_core::disk::USAGE_LIMIT or LOW_DISK, or an expired login text); the drain stops with Stop::RepeatedFailure naming an item and starts no further queued item. Run `cargo test -p athena-cli --test drain`.
New test: a landing error is classified by where it arises (landing_from_report Ok vs Err), checked with the real landing_from_report. Run `cargo test -p athena-cli`.
Unit test in drain.rs tests shows ids wrapped in backticks or parentheses are masked by the failure key, so such texts match. Run `cargo test -p athena-cli`.
In crates/athena-cli/src/drain.rs, `Drain::finish` no longer sets Stop::RepeatedFailure unconditionally for ItemEnd::Failed; a lone failure goes through pass_over with a reason naming the item and its failures, and only evidence of a shared cause stops the batch.
Existing drain tests asserting the old single-item stop are updated; `cargo test` passes for the workspace, `cargo clippy --all-targets` is clean, `cargo fmt --check` passes.
Doc comments in drain.rs (module header ~103, Stop::RepeatedFailure ~944 and ~1001) and text in display.rs describe the new rule; CHANGELOG.md has an entry; the display still names the failed item and its rejection.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
