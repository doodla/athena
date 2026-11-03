# Job 20261103-181027-bx8y: Don't stop a batch on repeated failure of an item already paused

## Request (verbatim)

> When a batch's repeated-failure stop is about an item that the scope guard has already paused, do not stop the batch: record it and move on to the next queued item. Keep the existing stop for every other repeated failure. This is the smallest version of the closed request 20261103-162416-m9th (whose plan grew too large); do not reclassify failures by cause here. Batches stopped this way twice today with money and job slots left. Acceptance: a test where an item fails twice, the scope guard pauses it, and the batch runs the next item; the existing repeated-failure test for an unpaused item still stops the batch.

## Reading (accepted by the user)

In the drain, when an item reaches its second failure (ATTEMPTS) and the scope guard has paused its line, end it as a passed-over paused item and go on to the next queued item, instead of Stop::RepeatedFailure. Unpaused items keep the existing stop. Job 20261103-162416-m9th (cause classification, key masking) is dropped; this is its smallest version. Audience: operators running batches.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs. Drain::attempts (about line 1725) counts failures and returns ItemEnd::Failed at ATTEMPTS (1811-1814). It already checks Steps::paused before each read (1745-1752) and returns ItemEnd::Paused. Drain::finish (2348) turns ItemEnd::Failed into set_stop(Stop::RepeatedFailure) unconditionally (2379). The gap: an item's second failure can itself pause its line, and no check follows the last failure. The fix is one re-check of Steps::paused before returning Failed (or in finish); no failure classification. Tests: crates/athena-cli/tests/drain.rs, with the existing paused-line test at line 644, repeated-failure tests at 1047 and 1073, and the Stub steps with a `paused` field. No record or data file is added.
- Job 20261103-162416-m9th: its four findings (Err classed as shared, invented stub text, missing id-wrapping unit test, round-3 planning stop) concern the cause classification and failure_key masking. The request tells us not to reclassify by cause, so they no longer apply and that plan is dropped. It left no candidate, so there are no changed files to keep or drop.
- A paused item is recorded through the existing ItemEnd::Paused path (a 'passed over' line), and not counted as a failure.
- The test Stub needs a paused list that changes after the item's runs fail; this may need a small Stub extension in tests/drain.rs.
- CHANGELOG.md gets a one-line entry if the repo convention requires it.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs: item a fails twice, the scope guard then reports its line paused, and drain::drain runs the next queued item. It asserts the next item is read, run and landed, report.stop is not Stop::RepeatedFailure, and report.failed_item is None. Check with `cargo test -p athena-cli --test drain`.
2. [behavior] The existing repeated-failure tests for an unpaused item (the_same_item_failing_twice_is_a_repeated_failure_naming_the_item and the not-landed variant) still pass unchanged: Stop::RepeatedFailure naming the item. Check with `cargo test -p athena-cli --test drain`.
3. [behavior] `cargo test` passes, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
4. [content] The diff to crates/athena-cli/src/drain.rs is small and has no failure-cause classification or key masking. The module docs and the Stop::RepeatedFailure description say that a failure of an item whose line is paused does not stop the batch. Verifiable with `git diff`.
5. [content] The paused item's second failure is recorded in the run output (a passed-over line with the pause reason) and not silently dropped. The new test asserts that line.
6. [content] No change to athena.toml, defaults.toml, workflows, the scope guard itself, ATTEMPTS, or the reader's skip-after-two-unread rule.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Reclassifying failures by cause, shared vs per-item, or failure_key masking (job 20261103-162416-m9th's plan)
- Changing the scope guard or how it pauses lines
- Changing ATTEMPTS or the reader's skip-after-two-unread rule
- Changes to athena.toml, defaults.toml, or workflows
- Changing environment stops such as rebuild failure, budget, job limit, or reinstall
- Changes to docs/ beyond doc comments in drain.rs, and to crates/athena-cli/tests/job_plan.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus existing tests; cargo test and clippy pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the drain, with no cause classification.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the drain through Stub and check the stop reason and the next item's run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The unconditional RepeatedFailure stop in finish/attempts is fixed, not one instance.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test for the drain tests.
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

A new test in crates/athena-cli/tests/drain.rs: item a fails twice, the scope guard then reports its line paused, and drain::drain runs the next queued item. It asserts the next item is read, run and landed, report.stop is not Stop::RepeatedFailure, and report.failed_item is None. Check with `cargo test -p athena-cli --test drain`.
The existing repeated-failure tests for an unpaused item (the_same_item_failing_twice_is_a_repeated_failure_naming_the_item and the not-landed variant) still pass unchanged: Stop::RepeatedFailure naming the item. Check with `cargo test -p athena-cli --test drain`.
`cargo test` passes, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
The diff to crates/athena-cli/src/drain.rs is small and has no failure-cause classification or key masking. The module docs and the Stop::RepeatedFailure description say that a failure of an item whose line is paused does not stop the batch. Verifiable with `git diff`.
The paused item's second failure is recorded in the run output (a passed-over line with the pause reason) and not silently dropped. The new test asserts that line.
No change to athena.toml, defaults.toml, workflows, the scope guard itself, ATTEMPTS, or the reader's skip-after-two-unread rule.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
