# Job 20261031-184832-4gah: Pass over queued items on a paused line instead of failing them

## Request (verbatim)

> Priority: high
> Skip, do not fail, a queued item whose line the scope guard has paused
> 
> On 2026-10-31 three drains stopped on items of the paused line 20261030-060751-5389 (gdgv, then 7ym7, then 487a): each item ended 'no outcome · not attempted · line ... is paused' twice, which counts as a repeated failure and stops the drain before any other work.
> 
> An item on a paused line is not a failure of that item. In athena drain, pass over it before reading it, print one line naming the line and how to lift the pause, and go on to the next item; it must not count toward the repeated-failure stop. athena plan should list such items as held, with the same reason.
> 
> Acceptance:
> - A drain test with a pause file for a line and a queued item on that line shows the item is passed over with one printed line, is not read, does not count as a failure, and the next item runs.
> - A plan test shows the item listed as held with the pause reason.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

`athena drain` passes over a queued item whose line the scope guard has paused (before reading it), prints one line naming the line and how to lift the pause, and runs the next item without counting a failure. `athena plan` lists such items as held with the same reason. The audience is whoever runs drains; it is the follow-up to job 20261030-060751-5389, whose work is unrelated and dropped.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs: `Drain::attempts` (about line 1642) runs `run_job`, and a paused line reaches it as a not-attempted result that `failures += 1` counts toward `ATTEMPTS`, so `ItemEnd::Failed` and `Stop::RepeatedFailure` follow. `ItemEnd::Held` and `ItemEnd::Skipped` are the existing non-failure ends, and `pass_over` (about line 2214) prints the one-line notes. `plan(steps, ...)` at about line 2748 builds `PlanReport`. crates/athena-core/src/scope_guard.rs: `Athena::line_pause(records, job_id)` returns the `LineMark` while the pause still holds, `pause_refusal` gives the text with the lift instruction and pause file, and `pause_path` gives the file. No record or data file is added under crates/, so `check_record_file`, `AnyRecord` and the round-trip test are not touched. The property is measured directly: the number of `read` calls and failure counts on a fake `Steps`, and the printed notes, in crates/athena-cli/tests/drain.rs and the drain tests module.
- Job 20261030-060751-5389 is a failed root-cause job on reading references. Its deliverables (reading.rs, CHANGELOG.md, docs/decisions.md) concern a different cause. Its work is dropped, not kept: this change does not touch reading.rs or docs/decisions.md, and it has no candidate files to keep. Its findings (Part 3 false positives, symbol-shape rule, crate-relative paths) do not apply to this request. The two jobs only share the paused line.
- The pause check is a new Steps method, or an extension of an existing one, that the real implementation backs with `line_pause`. The fake Steps used in tests lets a test mark a line paused.
- The passed-over item ends as a new non-failure ItemEnd, or the existing Held/Skipped path. The drain's final report lists it as passed over, with the same reason shown for `athena plan`.
- This adds no new refusal of readings, so no newly-refused-readings percentage applies. It removes a failure count; the drain still stops on a real repeated failure of any other item.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli drain`. A new drain test queues an item on a line with a pause file, then a second ordinary item. It asserts that the paused item is not read (zero read calls for it), that exactly one printed line names the line and how to lift the pause, that no failure is counted and the stop is not `repeated-failure`, and that the next item runs and lands.
2. [behavior] Run `cargo test -p athena-cli drain`. A new drain test queues the same paused-line item three times, followed by an ordinary item. It asserts that the drain does not stop on repeated failure and that the ordinary item still runs.
3. [behavior] Run `cargo test -p athena-cli plan`. A new plan test with a pause file for a line shows its queued item listed as held, with the pause reason and lift instruction from `pause_refusal`. The same test shows a queued item on an unpaused line is not listed as held.
4. [content] CHANGELOG.md has an entry that says drain passes over items on a paused line, prints one line, does not count them as failures, and plan lists them as held.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
6. [content] A stale pause (one `line_pause` no longer reports) does not hold an item. Pass-over uses `Athena::line_pause`, not just the existence of the pause file, as shown by the diff in the real Steps implementation.
7. [content] Job 20261030-060751-5389's candidate files and deliverables (reading.rs, docs/decisions.md) are unchanged in the diff.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing crates/athena-core/src/reading.rs or the reading-reference check of job 20261030-060751-5389; that job's work is dropped
- Changing how the scope guard decides to pause or lift a line
- Changing athena.toml, defaults.toml, or planner and critic prompts
- Changing how `athena go` or other commands handle a paused line

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain and plan tests; fmt, clippy -D warnings and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The paused line is stopped before it is read and counted as a non-failure end, rather than hiding the failure in the stop rule.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that check reads, printed lines, failure counts and the next item running.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that a drain no longer stops on a paused line and does other work.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria name commands the verifier can run.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced in code and tests, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in drain and plan.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 15 cases in 13 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make Athena own the build-folder lifecycle in one place that every run path calls, not only drain start. Before any retry or new build starts, and before the checks stage, Athena removes the build folders of jobs that are interrupted, abandoned, finished or superseded. A job's retry deletes its own ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

23 more qualifying classes are left out.

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

Run `cargo test -p athena-cli drain`. A new drain test queues an item on a line with a pause file, then a second ordinary item. It asserts that the paused item is not read (zero read calls for it), that exactly one printed line names the line and how to lift the pause, that no failure is counted and the stop is not `repeated-failure`, and that the next item runs and lands.
Run `cargo test -p athena-cli drain`. A new drain test queues the same paused-line item three times, followed by an ordinary item. It asserts that the drain does not stop on repeated failure and that the ordinary item still runs.
Run `cargo test -p athena-cli plan`. A new plan test with a pause file for a line shows its queued item listed as held, with the pause reason and lift instruction from `pause_refusal`. The same test shows a queued item on an unpaused line is not listed as held.
CHANGELOG.md has an entry that says drain passes over items on a paused line, prints one line, does not count them as failures, and plan lists them as held.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
A stale pause (one `line_pause` no longer reports) does not hold an item. Pass-over uses `Athena::line_pause`, not just the existence of the pause file, as shown by the diff in the real Steps implementation.
Job 20261030-060751-5389's candidate files and deliverables (reading.rs, docs/decisions.md) are unchanged in the diff.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
