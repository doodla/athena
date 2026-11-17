# Job 20261103-015135-qhhy: Record failed reads as failed invocations and fix the cause

## Request (verbatim)

> Find out why the reader returns no reading, and stop it happening again. What I read: GIT-LOG.txt has 5 'Read a request (the reader failed)' or 'again (the reader failed)' commits between 2026-11-02 and 2026-11-03. Job 20261102-205332-qv3s has 4 reader invocations and no reading-*.json. Job 20261102-171907-ab85 has 4 reader invocations and one reading. Every reader invocation I opened has outcome "success" and detail null, though no reading was produced, and each cost about $0.12-0.17 (reported cost). In crates/athena-core/src/reading.rs (first_reading, read_unread), the reader's error is written only into the commit body, and no record file stores it, so the records alone cannot show or count why reads fail. Work to do: (1) when a read gives no reading, record the invocation's outcome as a failure and put the reader's error in its detail; (2) find the cause in the stored errors (for example, no structured output returned) and fix it where it starts; (3) add a test with a fake reader that returns no structured reading, showing the invocation record has a failure outcome and the error text; (4) add a CHANGELOG entry. Note: job qv3s is the queued fix that hands the verifier the failing tests from a reused red CI run. Its reads keep failing, so that fix is stuck too.
> 
> Acceptance:
> - The change is made as described: The reader keeps returning no reading (5 'reader failed' commits in about a day), and each such invocation is recorded as outcome 'success' with no detail
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/reading.rs

## Reading (accepted by the user)

Make a read that yields no reading record its invocation as a failure with the reader's error in detail (reading.rs first_reading/read_unread path), find why the reader returns no structured output and fix it at its origin, add a fake-reader test and a CHANGELOG entry. Jobs 20261102-205332-qv3s and 20261102-171907-ab85 are cited as evidence only and are not reworked.

Assumptions:
- Goal: Athena's operator needs failed reads to be visible and countable in invocation records, and the failures to stop. Audience: the operator and the verifier reading records.
- Code survey: read crates/athena-core/src/reading.rs lines 640-775 (the read path: invoke_and_record, then a non-success outcome returns ReadError::Failed with record.detail, and missing structured output returns ReadError::Failed('the reader returned no structured reading') with the record untouched, so the record keeps outcome success and detail null). first_reading (~1199) and read_unread (~1185) write the error only into the commit subject/body. Also saw crates/athena-agent/src/stream.rs:192 (structured_output parsing) and claude.rs result handling. No new record type or file is added, so no records allow-list change is needed; the invocation record's existing outcome and detail fields are reused.
- Field meaning: InvocationOutcome and the invocation record's detail already carry a failure and its reason for other roles; reusing them for the reader fits. No config key changes.
- The cause is unknown from the repository copy because records and git history are absent; the producer must find it from the code path (why the reader exits successfully without structured output, e.g. turn or budget limits near the $0.12-0.17 spend, or schema mismatch). I assume one fix at the origin plus the recording change.
- Existing tests for the reader live in reading.rs or run/tests.rs; the new test goes next to the existing fake-reader tests.
- Out of reach: the verifier cannot read records/ or the git log, so GIT-LOG.txt counts are not criteria.

## Acceptance criteria

1. [content] In crates/athena-core/src/reading.rs, when the reader gives no reading (no structured output, or a non-success outcome), the invocation record written for that read has a failure outcome (not Success) and its detail holds the reader's error text, e.g. 'the reader returned no structured reading'. A diff line in first_reading/the read path sets record.outcome and record.detail before the record is stored.
2. [behavior] A new test with a fake reader that returns no structured reading (run with cargo test -p athena-core, test name containing 'no_structured') reads the stored invocation record and asserts a failure outcome and the error text in detail; it fails on the old code, where the outcome is success and detail is null.
3. [content] The root cause of the failed reads is named in the CHANGELOG entry and fixed where it starts in the diff (the reader's schema, prompt, budget or turn limit, or the way the agent layer reports a missing structured output), with the stored error text cited, not just the symptom recorded; if the stored errors do not show a single cause, the entry says so and what was checked.
4. [content] CHANGELOG.md has a new entry saying that failed reads are recorded as failed invocations with the error in detail, and giving the cause and its fix.
5. [content] Job 20261102-205332-qv3s (landed, verdict pass; its changed files in gh/, ci_result.rs, checks.rs, routing.rs, run/checks.rs, run/tests.rs, verify.rs, briefs.rs, CHANGELOG.md are kept as landed, not reworked) and job 20261102-171907-ab85 (landed; its files stay as landed) are not re-done. The diff touches none of those files except CHANGELOG.md and tests.
6. [behavior] Athena's recorded fmt, clippy and test checks on the final tree pass, and the existing reading tests still pass (cargo test -p athena-core).

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-agent/src/stream.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-agent/src/claude.rs (modify)

## Out of scope

- Rework of job 20261102-205332-qv3s (CI failing-test hand-off to the verifier): its 12 changed files stay as landed.
- Rework of job 20261102-171907-ab85 (homepage/README audience wording): README.md, studies/firstmate-work-handling/intro.json, xtask/src/site.rs and xtask/tests/site_page.rs stay as landed.
- New record types or data files; changes to athena.toml or defaults.toml.
- Changing verifier rules or the verdict rules.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New fake-reader test; fmt, clippy and test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the cause where it starts, plus record the failure.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts the stored record's outcome and detail.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

36 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261103-015135-qhhy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261103-015135-qhhy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In crates/athena-core/src/reading.rs, when the reader gives no reading (no structured output, or a non-success outcome), the invocation record written for that read has a failure outcome (not Success) and its detail holds the reader's error text, e.g. 'the reader returned no structured reading'. A diff line in first_reading/the read path sets record.outcome and record.detail before the record is stored.
A new test with a fake reader that returns no structured reading (run with cargo test -p athena-core, test name containing 'no_structured') reads the stored invocation record and asserts a failure outcome and the error text in detail; it fails on the old code, where the outcome is success and detail is null.
The root cause of the failed reads is named in the CHANGELOG entry and fixed where it starts in the diff (the reader's schema, prompt, budget or turn limit, or the way the agent layer reports a missing structured output), with the stored error text cited, not just the symptom recorded; if the stored errors do not show a single cause, the entry says so and what was checked.
CHANGELOG.md has a new entry saying that failed reads are recorded as failed invocations with the error in detail, and giving the cause and its fix.
Job 20261102-205332-qv3s (landed, verdict pass; its changed files in gh/, ci_result.rs, checks.rs, routing.rs, run/checks.rs, run/tests.rs, verify.rs, briefs.rs, CHANGELOG.md are kept as landed, not reworked) and job 20261102-171907-ab85 (landed; its files stay as landed) are not re-done. The diff touches none of those files except CHANGELOG.md and tests.
Athena's recorded fmt, clippy and test checks on the final tree pass, and the existing reading tests still pass (cargo test -p athena-core).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
