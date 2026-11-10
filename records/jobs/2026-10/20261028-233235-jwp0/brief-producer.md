# Job 20261028-233235-jwp0: Report per-rule reading refusal counts in status; flag rules over 30%

## Request (verbatim)

> Priority: high
> Reading rules have multiplied (boundary words, governance files, previous result, criteria resting on the producer's report, size, repository files, code survey) and each refusal costs a reader call and time; on 2026-10-28 one retry needed three corrections to satisfy two rules in turn. Nothing measures how often each rule refuses a first reading or how many re-reads it causes. Add, per reading rule, the count of first readings it refused and of second readings it still refused over the last 50 readings, to athena status, and have the Hestia sweep flag any rule that refuses more than 30% of first readings, naming the rule and its most common refusal text, so the reader's brief or the rule can be fixed.
> Acceptance:
> - A test with fixture reading records shows status reports per-rule first and second refusal counts.
> - A test shows the sweep flags a rule refusing more than 30% of first readings and not one under it.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/status.rs, crates/athena-core/src/hestia.rs, CHANGELOG.md

## Reading (accepted by the user)

Add to `athena status` a per-reading-rule count of first readings refused and second readings still refused over the last 50 readings. Make the Hestia sweep flag any rule refusing more than 30% of first readings, naming the rule and its most common refusal text. Goal: measure which reading rules cause re-reads, so the reader brief or the rule can be fixed. Audience: the owner and the Hestia sweep.

Assumptions:
- Code survey: I read crates/athena-core/src/status.rs (StatusView, `status` at line 985), crates/athena-core/src/hestia.rs (`default_checks`, `run_sweep`, `repeated_causes`), crates/athena-core/src/reading.rs (rule functions named check_*, `ReadingCheck::Accept` and `Refuse(String)`) and crates/athena-core/src/run_pin_tests.rs (the refused-then-reread test at line 1855). Resolution order: the reader runs, rules run in turn, and the first refusal triggers one re-read whose brief carries the refusal text. A second refusal is the failure, and the reader does not run a third time. Reader invocations are recorded under `<job>/invocations/`. I found no stored refusal text or rule name per reading, so the producer must first find where the first refusal's text can be read back from records and add the smallest record field if there is none. A new record or field under crates/ must be covered by `check_record_file`, `AnyRecord` and the round-trip test. The property is measured directly from per-reading refusal data, not from a stand-in such as invocation counts alone.
- Rule identification: a refusal text is mapped to a rule by the check function name (for example `check_governance_coverage`) or by a stable text prefix. Text that matches no rule is counted under 'other'.
- Window: the last 50 readings means the 50 most recent jobs that have a reading, counted per job. A first refusal counts once per rule that refused it. A second refusal counts the rules that refused the re-read. The sweep rate is first refusals by that rule divided by first readings in the window.
- Fewer than a minimum number of readings in the window, for example 10, does not trigger a flag, so an empty or tiny window is never reported as healthy or as a failure. This is a design choice the producer states in docs or tests.
- Existing deliverables beyond the three named files (reading.rs, run/mod.rs, athena-data records.rs, a docs page) are possible only if the refusal data is not already recorded; they are listed with lower probability.
- The change adds no refusal rule. It only measures refusals, so no 'higher refusal rate is intended' or replay percentage criterion is needed. Governance files (athena.toml, defaults.toml and others) are untouched. Prompts are untouched; the reader prompt has 2 tokens of headroom.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core status`. A test in crates/athena-core/src/status.rs builds fixture reading records where rule A refuses 3 first readings and 1 second reading, and rule B refuses 1 first reading. It asserts that the status output reports, per rule, the first-refusal count and the second-refusal count, over at most the last 50 readings.
2. [behavior] Run `cargo test -p athena-core status`. A test with fixture records of more than 50 readings asserts that only the most recent 50 are counted.
3. [behavior] Run `cargo test -p athena-core hestia`. A test in crates/athena-core/src/hestia.rs asserts that the sweep flags a rule refusing more than 30% of first readings. The flag names the rule and its most common refusal text. The same test asserts that a rule at or under 30% is not flagged.
4. [behavior] Run `cargo test -p athena-core hestia`. A test asserts that a window with no readings, or fewer than the stated minimum, produces no flag and no claim that the rules are healthy.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
6. [content] CHANGELOG.md has a new entry. It names the per-rule first and second refusal counts in `athena status` and the Hestia sweep flag above 30%.
7. [content] The diff in crates/athena-core/src/status.rs shows each reading rule listed with its two counts. The diff in crates/athena-core/src/hestia.rs shows the 30% threshold as a named constant, not a bare number.
8. [content] If the diff adds a record type or field under crates/, it is covered by `check_record_file`, `AnyRecord` parsing and the round-trip test. Run `cargo test -p athena-data` to confirm.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status/throughput.rs (modify)

## Out of scope

- Changing, adding or loosening any reading rule or its refusal text
- Editing the reader prompt or brief; the sweep only names which rule or brief to fix
- Automatically filing fix requests for flagged rules
- Editing athena.toml, defaults.toml or workflows
- Changing the retry limit of one re-read

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus fmt, clippy -D warnings and cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests on fixture reading records and check the reported counts and flags.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty or tiny window gives no flag and is not reported as healthy.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry and code docs are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The change measures rule refusals by rule rather than counting retries only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261028-233235-jwp0/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-233235-jwp0/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core status`. A test in crates/athena-core/src/status.rs builds fixture reading records where rule A refuses 3 first readings and 1 second reading, and rule B refuses 1 first reading. It asserts that the status output reports, per rule, the first-refusal count and the second-refusal count, over at most the last 50 readings.
Run `cargo test -p athena-core status`. A test with fixture records of more than 50 readings asserts that only the most recent 50 are counted.
Run `cargo test -p athena-core hestia`. A test in crates/athena-core/src/hestia.rs asserts that the sweep flags a rule refusing more than 30% of first readings. The flag names the rule and its most common refusal text. The same test asserts that a rule at or under 30% is not flagged.
Run `cargo test -p athena-core hestia`. A test asserts that a window with no readings, or fewer than the stated minimum, produces no flag and no claim that the rules are healthy.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
CHANGELOG.md has a new entry. It names the per-rule first and second refusal counts in `athena status` and the Hestia sweep flag above 30%.
The diff in crates/athena-core/src/status.rs shows each reading rule listed with its two counts. The diff in crates/athena-core/src/hestia.rs shows the 30% threshold as a named constant, not a bare number.
If the diff adds a record type or field under crates/, it is covered by `check_record_file`, `AnyRecord` parsing and the round-trip test. Run `cargo test -p athena-data` to confirm.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
