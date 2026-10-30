# Job 20261030-061329-m30w: Keep and show the reader's error; re-read once, then skip the item

## Request (verbatim)

> Priority: urgent
> When the reader runs but produces no usable reading, the reason is lost and the drain stalls on the item. Evidence: job 20261029-233021-e5sg, commit 21689b7 'Read a request (the reader failed)': both reader invocations report outcome success, no reading file was written, and Proposal.reader_error (crates/athena-core/src/reading.rs) is never printed by drain nor committed. The job is then Proposed without a reading, so each later drain attempt fails at once with 'has no reading yet (the reader failed)'; drain-24 and drain-25 each stopped on 'repeated failure' with $0 spent. A manual athena correct read it fine.
> Change: commit the reader error with the failed-reading record and print it in drain's log line for the item; in drain, a Proposed job with no reading is read again once (like a refused reading), and if that also fails the item is skipped with its reason rather than counted twice toward the repeated-failure stop.
> Acceptance:
> - A test through the drain entry point with a reader that returns unparseable output shows the error text in the drain log and in the committed record, one re-read, and the drain moving on to the next item.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/reading.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

When the reader produces no usable reading (job 20261029-233021-e5sg, commit 21689b7), the error is committed with the failed-reading record and printed on the drain's log line for the item. A Proposed job with no reading is read again once, like a refused reading. If that also fails, the drain skips the item with its reason and moves on, without counting it twice toward the repeated-failure stop. The audience is the person running `athena drain`, who needs to see why an item stalled.

Assumptions:
- Code survey: I read crates/athena-core/src/reading.rs (the read path around lines 840-878 builds the 'Read a request (the reader failed)' commit and returns Proposal.reader_error), crates/athena-core/src/run/mod.rs line 332 (the 'has no reading yet (the reader failed)' error from go), and crates/athena-cli/src/drain.rs (run_job around 1067, attempts around 1248 with ATTEMPTS and ItemEnd::Failed, the read-ahead near 1835, and the real Steps impl at 2512-2625).
- Code survey, drain order: attempts() calls run_job with ReadOf::Queued, which calls steps.read and then steps.run. For a Proposed job with no reading, run returns Err, so the line is 'not run' and the item fails twice (ATTEMPTS), producing the repeated-failure stop. The fix goes in the drain's Steps::read/run path, not the reader.
- Code survey, records: the error text is added to the failed-reading commit's body, or to an existing record already validated by the AnyRecord allow-list. No new record type or data file under crates/ is planned, so check_record_file and the AnyRecord round-trip test are unaffected. If the producer adds a field to a record, it must extend the round-trip test.
- Code survey, measurement: the tests drive the drain entry point with the existing fake-reader setup in crates/athena-cli/tests/drain.rs (a reader returning unparseable output) and read the drain's log text and the committed record directly. They do not check wiring or types.
- The reader error is committed as text in the failed-reading commit body, with the secrets redactor applied. If a field on an existing record is cleaner, the producer may use that instead.
- The re-read is a single extra read call. If it fails again, the item is skipped with the reader's error as its reason. The skip counts as one failure for that item, so a different later item can still be tried and the drain does not stop with 'repeated failure'.
- The fix is generic: it applies to any Proposed job with no reading, not only job 20261029-233021-e5sg. This request does not edit or repair that job's records.
- Job 20261029-233021-e5sg is the evidence job here, not a retry target. Its own rejected change (the landing-stage fix in status.rs, store.rs and athena-git) is not retried or kept by this request. Its findings are not addressed here, and the files it changed are listed as out_of_scope.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test drives the drain entry point with a reader that returns unparseable output. It asserts that the reader's error text appears in the drain's log line for the item and in the committed failed-reading record, that the reader was invoked exactly twice for that item (one re-read), and that the drain then went on to the next item and ran it.
2. [behavior] In that test, the skipped item is reported with its reason, and the drain's stop reason is not 'repeated failure' (the item is not counted twice toward the stop). A second test, or part of the first, shows that a re-read which succeeds lets the item run.
3. [content] In crates/athena-core/src/reading.rs, the failed-reading commit body or record now includes Proposal.reader_error. A Grep of the diff shows the error used when the commit is built. A test in athena-core checks that the committed record contains the error text.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
5. [content] CHANGELOG.md has a new entry saying the reader's error is committed and printed by drain, and that a Proposed job with no reading is read once more and then skipped with its reason.
6. [content] Governance files (athena.toml, defaults.toml, workflows, records/) are unchanged. The diff touches only crates/athena-core/src/reading.rs, crates/athena-cli/src/drain.rs, tests under crates/, and CHANGELOG.md.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Retrying or keeping job 20261029-233021-e5sg's rejected landing-stage change: crates/athena-core/src/status.rs, crates/athena-core/src/store.rs and crates/athena-git/src/lib.rs are left as they are on main, and its CHANGELOG entry is not touched
- Finding out why the reader produced no reading in the failed job (a reader or prompt fix)
- Changing athena.toml, defaults.toml, workflow files or records/
- Changing the repeated-failure rule for failures that are not unread jobs
- Repairing or re-reading existing job records by hand

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain-entry-point tests, with the fmt, clippy -D warnings and test suite criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check the error text in the log and the committed record, the single re-read and the drain moving on, not that wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix covers any unread Proposed job, not only the one that surfaced.
- thrash-stop: If the same item fails twice in a loop, the loop stops and reports thrashing. Prose-only: no loop code detects it yet. Here: The skipped item is not double-counted, but a different item that fails twice still stops the drain.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the drain continuing past an unreadable item with its reason shown.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across reading.rs, drain.rs and the changelog.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The committed error text goes through the redactor.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
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

Run `cargo test -p athena-cli --test drain`. A new test drives the drain entry point with a reader that returns unparseable output. It asserts that the reader's error text appears in the drain's log line for the item and in the committed failed-reading record, that the reader was invoked exactly twice for that item (one re-read), and that the drain then went on to the next item and ran it.
In that test, the skipped item is reported with its reason, and the drain's stop reason is not 'repeated failure' (the item is not counted twice toward the stop). A second test, or part of the first, shows that a re-read which succeeds lets the item run.
In crates/athena-core/src/reading.rs, the failed-reading commit body or record now includes Proposal.reader_error. A Grep of the diff shows the error used when the commit is built. A test in athena-core checks that the committed record contains the error text.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
CHANGELOG.md has a new entry saying the reader's error is committed and printed by drain, and that a Proposed job with no reading is read once more and then skipped with its reason.
Governance files (athena.toml, defaults.toml, workflows, records/) are unchanged. The diff touches only crates/athena-core/src/reading.rs, crates/athena-cli/src/drain.rs, tests under crates/, and CHANGELOG.md.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
