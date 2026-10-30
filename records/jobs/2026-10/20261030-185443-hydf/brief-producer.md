# Job 20261030-185443-hydf: Commit and print the reader's error; re-read an unread job once

## Request (verbatim)

> try again on 20261030-061329-m30w

## Reading (accepted by the user)

Retry of job 20261030-061329-m30w (dispatched, never verified, no candidate or findings), so the earlier reading is rebuilt unchanged in intent. When the reader gives no usable reading, its error is committed with the failed-reading record and shown on the drain's line for the item. A Proposed job with no reading is read once more, then skipped with its reason, without counting twice toward the repeated-failure stop. The audience is the person running `athena drain`.

Assumptions:
- Job 20261030-061329-m30w: it had no candidate, no changed files and no findings, so nothing is kept from it and nothing needs fixing. The whole original request is redone, with the same deliverables and scope.
- Job 20261029-233021-e5sg's rejected landing-stage change is not retried or kept. status.rs, store.rs, athena-git/src/lib.rs and that job's CHANGELOG entry stay as on main.
- Code survey: crates/athena-core/src/reading.rs (`Proposal` at about line 61, the read path that builds the 'Read a request (the reader failed)' commit at about lines 858-877, and the retry path at about line 1046). crates/athena-core/src/run/mod.rs (`go` for a Proposed job: `None => bail!("job {id} has no reading yet (the reader failed)...")` at about line 331). crates/athena-cli/src/drain.rs: `run_job` builds the `JobLine` and returns `RunStep::Failed { read: false }` when the read has an error. `attempts` counts `failures` and returns `ItemEnd::Failed`, which sets `Stop::RepeatedFailure` at about line 1656. `Steps::read` for `ReadOf::Queued` calls `athena.go`, whose bail is what drain sees as the item's error.
- Code survey: the change adds no record or data file under crates/, so the records allow-list (`check_record_file`, `AnyRecord`, the round-trip test) is untouched. The error text goes into the existing failed-reading commit message body and the item's log line, not into a new record type. If it needs a field in an existing record, the round-trip test must still pass.
- The drain's re-read of a Proposed job with no reading is a second `Steps::read`/`athena correct`-style read inside the drain item's attempt. It is bounded to one re-read per item. When both reads fail, the item ends as skipped with its reason. It uses a skip-style end, not `ItemEnd::Failed`, so `Stop::RepeatedFailure` is not set and the next item runs.
- Run/mod.rs and run_pin_tests.rs stay as listed deliverables from the earlier scope (a possible new `go` error shape and pin tests), but are likely unchanged. Only the files that need to change are edited.
- The tests use the existing drain test harness in crates/athena-cli/tests/drain.rs with a fake reader that returns unparseable output, and reuse the harness's way of capturing the drain log.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test drives the drain entry point with a reader that returns unparseable output for the first item. It asserts that the reader's error text appears in the drain log line for that item and in the committed failed-reading record (a `git show` of the commit), that the reader ran exactly twice for the item (one re-read), and that the drain then ran the next item.
2. [behavior] In that test the skipped item is reported with its reason, and the drain's stop reason is not 'repeated failure'. A second test (or a second part of the first) shows that when the re-read succeeds the item runs. Run with `cargo test -p athena-cli --test drain`.
3. [behavior] A test in athena-core (run with `cargo test -p athena-core`) shows that a read whose reader output cannot be parsed commits a record or commit body that contains the `Proposal.reader_error` text.
4. [content] In crates/athena-core/src/reading.rs, the failed-reading commit is built using `reader_error`. `git diff` shows the error flowing into the commit body or record.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has a new entry saying the reader's error is committed and printed by drain, and that a Proposed job with no reading is read once more and then skipped with its reason.
7. [content] `git diff` touches only crates/athena-core/src/reading.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, optionally crates/athena-core/src/run/mod.rs and crates/athena-core/src/run_pin_tests.rs, and CHANGELOG.md. athena.toml, defaults.toml, .github/workflows and records/ are unchanged, as are status.rs, store.rs and athena-git/src/lib.rs.
8. [behavior] The existing repeated-failure behavior is unchanged for items that failed after being read: the existing drain tests for `Stop::RepeatedFailure` still pass under `cargo test -p athena-cli --test drain`.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Retrying or keeping job 20261029-233021-e5sg's rejected landing-stage change: crates/athena-core/src/status.rs, crates/athena-core/src/store.rs and crates/athena-git/src/lib.rs stay as on main, and its CHANGELOG entry is not touched
- Finding out why the reader produced no reading in the failed job (a reader or prompt fix)
- Changing athena.toml, defaults.toml, workflow files or records/
- Changing the repeated-failure rule for failures that are not unread jobs
- Repairing or re-reading existing job records by hand

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain and athena-core tests; fmt, clippy -D warnings and cargo test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the error text in the log and the committed record, the one re-read and the drain moving on, not just that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The error is carried and committed at its source (the failed-reading record), and the repeated-failure counting is fixed for unread jobs, not just logged.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in reading.rs and drain.rs with tests and a changelog entry.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog and commit text are written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the drain moving on with the reason visible, not the activity of editing.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the drain tests and the full suite.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...

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

Run `cargo test -p athena-cli --test drain`. A new test drives the drain entry point with a reader that returns unparseable output for the first item. It asserts that the reader's error text appears in the drain log line for that item and in the committed failed-reading record (a `git show` of the commit), that the reader ran exactly twice for the item (one re-read), and that the drain then ran the next item.
In that test the skipped item is reported with its reason, and the drain's stop reason is not 'repeated failure'. A second test (or a second part of the first) shows that when the re-read succeeds the item runs. Run with `cargo test -p athena-cli --test drain`.
A test in athena-core (run with `cargo test -p athena-core`) shows that a read whose reader output cannot be parsed commits a record or commit body that contains the `Proposal.reader_error` text.
In crates/athena-core/src/reading.rs, the failed-reading commit is built using `reader_error`. `git diff` shows the error flowing into the commit body or record.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has a new entry saying the reader's error is committed and printed by drain, and that a Proposed job with no reading is read once more and then skipped with its reason.
`git diff` touches only crates/athena-core/src/reading.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, optionally crates/athena-core/src/run/mod.rs and crates/athena-core/src/run_pin_tests.rs, and CHANGELOG.md. athena.toml, defaults.toml, .github/workflows and records/ are unchanged, as are status.rs, store.rs and athena-git/src/lib.rs.
The existing repeated-failure behavior is unchanged for items that failed after being read: the existing drain tests for `Stop::RepeatedFailure` still pass under `cargo test -p athena-cli --test drain`.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
