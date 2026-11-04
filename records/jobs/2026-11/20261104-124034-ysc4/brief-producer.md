# Job 20261104-124034-ysc4: Pass over an item that fails twice instead of stopping the batch

## Request (verbatim)

> Pass over any item that fails twice instead of stopping the batch. Two landed fixes already pass over an item that fails twice when the scope guard has paused or flagged it, but at 12:39pm UTC on 4 Nov a batch with $43 and 3 job slots left stopped again because the item (job 20261029-160024-80qj) had failed twice with no pause or flag. Remove the item-level repeated-failure stop: after an item's second failure, record it, leave it for the scope guard and Hestia, and run the next item. Keep the batch stops that are not about a single item (disk, login, settings, the kill switch). Acceptance: a test where an unflagged, unpaused item fails twice and the batch then runs the next item; the existing tests for the other stops still pass.

## Reading (accepted by the user)

Remove the item-level repeated-failure batch stop in the drain: after an item's second failure with no pause or flag, record it as passed over (left for the scope guard and Hestia) and run the next item. Disk, login, settings and kill-switch stops stay. Job 20261029-160024-80qj (the 4 Nov stopped batch's item) is the trigger, not the work to redo.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs. The item loop counts failures against ATTEMPTS (around line 1829). After the last failure it returns ItemEnd::Paused when paused_reason finds a pause, ItemEnd::Flagged when flagged_reason finds a flag, and otherwise ItemEnd::Failed. The item-end handler (around line 2426) maps ItemEnd::Failed to set_stop(Stop::RepeatedFailure, Some(job)). Paused and Flagged go to pass_over, which records the item and runs on. The Stop enum (around lines 957-1071) has a RepeatedFailure variant, a name, a display and a repeated-failure item field. crates/athena-cli/src/display.rs line 620 prints 'stopped repeated failure: item X failed twice'. crates/athena-cli/src/main.rs line 254 documents the stop in help text. crates/athena-cli/tests/drain.rs asserts the stop at lines 934, 1300 and 1322, and asserts it is absent at lines 909 and 1363.
- Design: the new outcome of an unflagged, unpaused double failure reuses the existing ItemEnd::Flagged/Paused pass_over path, with its own reason text such as 'failed twice; left for the scope guard and Hestia'. Failing items still get no retry beyond ATTEMPTS. The producer decides whether to delete the Stop::RepeatedFailure variant and its display and help text (preferred, as nothing else sets it) or keep it unused. Any stop reason or serialized stop name that reaches a record is checked so that old records still parse.
- The batch stops kept are the ones not about a single item: disk, login, settings (unreadable config / needs reinstall), the kill switch (Stop::Requested), and the budget and job-limit stops. Their existing tests in crates/athena-cli/tests/drain.rs stay unchanged and pass.
- Existing tests at tests/drain.rs lines 934, 1300 and 1322 assert 'repeated failure: item a failed twice' and must be rewritten to assert the new pass-over behavior (the batch goes on, the item is recorded as passed over). This is intended, not a regression.
- Job 20261029-160024-80qj (docs/design/warm-baseline-build.md, CHANGELOG.md entry) was rejected by verification. That design note does not exist in the base snapshot. Its findings (CI-result path omitted, mtime claims) concern a different design job and do not apply here. Its candidate files are dropped, not kept: this change neither creates the note nor touches that job's CHANGELOG entry, and a separate job should redo it.
- Goal and audience: the operator who runs `athena drain` batches (and Hestia and the scope guard, which handle the left-over item) needs a batch not to halt on one item's double failure. No expected refusal-rule change: this loosens a stop and adds no new refusal, so no refused-readings rate applies.
- No governance files change: athena.toml, defaults.toml, .github/workflows and rust-toolchain.toml are untouched. The thrash-stop default text in defaults.toml / crates/athena-data/src/defaults.rs is a user expectation and is left as is.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`: a new test where an item on a line that is neither paused nor flagged fails twice and the batch then runs the next item to completion (for example it lands). The test asserts the report has no repeated-failure stop and records the failed item as passed over, with a reason.
2. [behavior] Run `cargo test` (whole workspace): the existing drain tests for the disk, login, settings (unreadable config/needs reinstall), kill-switch, budget and job-limit stops, and the paused and flagged pass-over tests, pass unchanged.
3. [content] Grep of crates/athena-cli/src/drain.rs finds no code path that sets a batch stop because a single item failed twice: the `ItemEnd::Failed => set_stop(Stop::RepeatedFailure, ..)` arm is gone or becomes pass_over, and the comment at about line 1831 and the module docs at lines 32 and 110 are updated to match.
4. [content] The help text in crates/athena-cli/src/main.rs (about line 254) and the stop text in crates/athena-cli/src/display.rs (about line 620) no longer list 'repeated failure' as a batch stop, unless the variant is deliberately kept and unused.
5. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean, with no dead-code warnings from a leftover RepeatedFailure variant.
6. [content] CHANGELOG.md has an entry saying an item that fails twice is passed over instead of stopping the batch, naming the 4 Nov stop of job 20261029-160024-80qj.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/design/warm-baseline-build.md (create)

## Out of scope

- Re-doing the warm-baseline design note or its CHANGELOG entry from job 20261029-160024-80qj (that work is dropped)
- Changing the disk, login, settings, kill-switch, budget or job-limit stops
- Changing the scope guard's pause or flag logic, or Hestia
- Changing ATTEMPTS or the retry policy within an item
- athena.toml, defaults.toml, rust-toolchain.toml, .github/workflows/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New pass-over test plus the whole workspace test and clippy runs as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain tests and check that the batch runs the next item.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The stop arm is removed at its source in drain.rs, not special-cased for one job.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments, help text and CHANGELOG are updated for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the drain module and its tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the next item runs after a double failure.
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
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

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

Run `cargo test -p athena-cli --test drain`: a new test where an item on a line that is neither paused nor flagged fails twice and the batch then runs the next item to completion (for example it lands). The test asserts the report has no repeated-failure stop and records the failed item as passed over, with a reason.
Run `cargo test` (whole workspace): the existing drain tests for the disk, login, settings (unreadable config/needs reinstall), kill-switch, budget and job-limit stops, and the paused and flagged pass-over tests, pass unchanged.
Grep of crates/athena-cli/src/drain.rs finds no code path that sets a batch stop because a single item failed twice: the `ItemEnd::Failed => set_stop(Stop::RepeatedFailure, ..)` arm is gone or becomes pass_over, and the comment at about line 1831 and the module docs at lines 32 and 110 are updated to match.
The help text in crates/athena-cli/src/main.rs (about line 254) and the stop text in crates/athena-cli/src/display.rs (about line 620) no longer list 'repeated failure' as a batch stop, unless the variant is deliberately kept and unused.
Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean, with no dead-code warnings from a leftover RepeatedFailure variant.
CHANGELOG.md has an entry saying an item that fails twice is passed over instead of stopping the batch, naming the 4 Nov stop of job 20261029-160024-80qj.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
