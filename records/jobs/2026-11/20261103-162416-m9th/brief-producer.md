# Job 20261103-162416-m9th: Pause an item that fails twice; stop the batch only on shared causes

## Request (verbatim)

> Keep a batch running when one item fails twice. Today a batch with $16 and 3 jobs left stopped because a single item (judge candidate PR heads as pending) failed planning twice, even though the scope guard had already paused that item. A batch should stop on repeated failure only when the failures point to something shared (the environment, the build, Athena itself); a single item's repeated failure should pause that item and let the batch go on to the next one. Acceptance: a test where one item fails twice and the batch then runs the next queued item; a test where two different items fail for the same shared cause and the batch still stops.

## Reading (accepted by the user)

In the drain, an item failing twice currently sets Stop::RepeatedFailure and ends the batch. Change it so one item's double failure passes that item over (paused, noted, failure visible) and the batch continues, while two different items failing for the same cause still stop the batch with RepeatedFailure. Audience: the operator running `athena drain` with budget left.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs. `Drain::attempts` (~line 1725) loops: checks stop_requested, then the scope-guard pause (returns ItemEnd::Paused), then run_job; a failed attempt (rejected job or NotLanded) increments `failures`, and at ATTEMPTS returns ItemEnd::Failed. `Drain::finish` (~2379) maps ItemEnd::Failed to set_stop(Stop::RepeatedFailure, Some(job)), which ends every worker via take_until_done. ItemEnd::Paused/Skipped already use pass_over and let the run continue; that is the model for the new behaviour. Stop::RepeatedFailure is declared ~line 944 with text near 983/1001, and rendered in crates/athena-cli/src/display.rs ~619.
- Code survey: existing tests in crates/athena-cli/tests/drain.rs that assert Stop::RepeatedFailure for a single failing item (~lines 939, 1060, 1083, 1272, 4293, 6435, 6864, 7194) and the `Before::RepeatedFailure` seeding (~6628) encode the old rule and must be updated to the new rule; display tests assert 'repeated failure: item a failed twice'. No new record or data file is added, so records validation (check_record_file, AnyRecord) is untouched.
- Shared cause is judged by comparing a normalized failure reason (the rejection/not-landed text, with job and item names stripped) between two different items; the same reason on a second distinct item means shared and stops the batch with Stop::RepeatedFailure naming the later item. Different reasons, or the same item, only pause that item. Item failures from the environment/build (rebuild failure, unreadable config, disk hold) keep their existing stops.
- A paused-by-failure item is recorded in report jobs/notes with its reason so the failure stays visible, and is not retried again in the same run.
- This adds no new refusal rule that rejects more readings; it loosens a stop, so no refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows) are not changed.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs has item `a` fail twice and then asserts the drain runs the next queued item `b` (stub log shows b read/run after a's retry), and report.stop is not Stop::RepeatedFailure. Run `cargo test -p athena-cli --test drain`.
2. [behavior] A new test has two different items fail twice with the same failure reason and asserts the drain stops with Stop::RepeatedFailure, naming an item, and does not start a further queued item. Run `cargo test -p athena-cli --test drain`.
3. [behavior] A new test has two different items fail with different reasons and asserts the batch continues past both (not a shared cause). Run `cargo test -p athena-cli --test drain`.
4. [content] In crates/athena-cli/src/drain.rs `Drain::finish` no longer sets Stop::RepeatedFailure unconditionally for ItemEnd::Failed; a lone item failure goes through pass_over with a reason that names the item and its failures, and the stop only occurs when the failure reason matches a different earlier failed item.
5. [behavior] Existing drain tests that asserted the old single-item stop are updated to the new rule, and `cargo test` passes across the workspace, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes.
6. [content] The doc comments in drain.rs (module header ~line 103, Stop::RepeatedFailure docs ~942 and its description ~1001) and the text in display.rs describe the new rule: a repeated failure of one item pauses that item; only a shared cause stops the batch. CHANGELOG.md has an entry.
7. [behavior] Running the drain display test for a paused-by-failure item (`cargo test -p athena-cli`) shows the output still names the failed item and its rejection, so the failure is not hidden when the batch continues.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- docs (modify)

## Out of scope

- Changing the scope guard itself or how it pauses lines
- Changing the ATTEMPTS count or the reader's skip-after-two-unread rule
- Changes to athena.toml, defaults.toml, or workflows
- Changing environment stops such as rebuild failure, budget, job limit, or reinstall

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain with stub items and check the next item runs or the batch stops.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Changes the stop rule in finish, not the one item that surfaced.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the batch continuing past a lone failure, and stopping on a shared cause.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog updated for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in drain code and tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
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

A new test in crates/athena-cli/tests/drain.rs has item `a` fail twice and then asserts the drain runs the next queued item `b` (stub log shows b read/run after a's retry), and report.stop is not Stop::RepeatedFailure. Run `cargo test -p athena-cli --test drain`.
A new test has two different items fail twice with the same failure reason and asserts the drain stops with Stop::RepeatedFailure, naming an item, and does not start a further queued item. Run `cargo test -p athena-cli --test drain`.
A new test has two different items fail with different reasons and asserts the batch continues past both (not a shared cause). Run `cargo test -p athena-cli --test drain`.
In crates/athena-cli/src/drain.rs `Drain::finish` no longer sets Stop::RepeatedFailure unconditionally for ItemEnd::Failed; a lone item failure goes through pass_over with a reason that names the item and its failures, and the stop only occurs when the failure reason matches a different earlier failed item.
Existing drain tests that asserted the old single-item stop are updated to the new rule, and `cargo test` passes across the workspace, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes.
The doc comments in drain.rs (module header ~line 103, Stop::RepeatedFailure docs ~942 and its description ~1001) and the text in display.rs describe the new rule: a repeated failure of one item pauses that item; only a shared cause stops the batch. CHANGELOG.md has an entry.
Running the drain display test for a paused-by-failure item (`cargo test -p athena-cli`) shows the output still names the failed item and its rejection, so the failure is not hidden when the batch continues.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
