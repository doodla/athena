# Job 20261029-233025-6ayn: Refuse records-measure readings lacking a real-records criterion

## Request (verbatim)

> Priority: high
> Measures and reports that Athena computes from its own records (throughput, keep rule, planning cost, refusal rates, check durations) have twice in two days been verified on fixture records alone and then shown wrong figures on the real records (2026-10-29: landing time 0.1 minutes, then 273.8 minutes, both against real landings of a few minutes). Require that any change to a measure computed from records includes a criterion run against this repository's real records with an expected range taken from an independent source (git log, recorded check durations), and give the verifier that real output.
> Acceptance:
> - The reader brief and the planner brief state the rule in one line each and stay within their size limits; the size test passes.
> - A test shows a reading for a change to a records-computed measure with no real-records criterion is refused at reading time naming the missing criterion.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/reading.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md

## Reading (accepted by the user)

Add a reading-time check that refuses a code change to a measure computed from Athena's records unless a criterion runs it against this repository's real records with an expected range from an independent source, plus one-line rules in the reader and planner briefs. Audience: Athena maintainers and the verifier, who need the real output.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (check_reading chain with step! calls, build_reading, size test helpers), crates/athena-core/src/reading/surveys.rs (check_history_fixture is the closest existing check: it fires on code readings with a behavior criterion and names the missing phrase in its error), and crates/athena-context/src/briefs.rs (reader and planner rule lines and the size test). The new check is called from the same chain as check_history_fixture; the property (a records-computed measure change lacking a real-records criterion) is measured directly from reading text: summary, criteria and deliverable paths. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected.
- Existing code: crates/athena-core/src/reading/surveys.rs check_history_fixture and changes_code_with_behavior_criterion are extended or mirrored; crates/athena-context/src/briefs.rs reader and planner rule lists get one line each.
- Environment: the check runs inside Athena's reader step on the producer's working copy snapshot, with read-only access to the reading text and no git, network or records access; it needs no origin, so with no origin, a non-GitHub origin or a GitHub origin it behaves identically. The tests run under cargo test with no records branch. The rule it enforces asks later jobs' criteria to run against this repository's real records, which the verifier sees as recorded check output; the producer working copy does not contain records/. No data or doc file is read by the new check.
- Because the request asks for the rule: the check refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings (estimate, not measured; few past jobs changed records-computed measures). The replay at verification measures the real share.
- The rule is detected by keywords for records-computed measures (throughput, keep rule, planning cost, refusal rate, check duration, landing time) in a code or tests reading's summary or criteria, and satisfied by a criterion naming real records and an independent source (git log or recorded check durations) with an expected range.
- Brief growth is tight: reader has 2 tokens left and planner 95, so the reader line must replace or shorten existing wording to stay within the size limit.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs.rs has exactly one added rule line in the reader brief and one in the planner brief stating that a change to a records-computed measure needs a criterion run against real records with an expected range from an independent source; Grep the diff shows one line each.
2. [behavior] Run `cargo test -p athena-context`: the brief size test passes with both new lines, so the reader and planner briefs stay within their token budgets.
3. [behavior] Run `cargo test -p athena-core`: a new test builds a code reading whose summary changes a records-computed measure (for example landing time) with only fixture-based criteria and asserts check_reading returns an error naming the missing real-records criterion.
4. [behavior] Run `cargo test -p athena-core`: a companion test shows the same reading accepted once it adds a criterion naming real records, an independent source (git log or recorded check durations) and an expected range, and a reading not touching a records measure is unaffected.
5. [content] The new check's doc comment in crates/athena-core/src/reading/surveys.rs or reading.rs states that it refuses more readings, the higher rate is intended, and declares `expected 3% of newly refused readings`.
6. [content] CHANGELOG.md has a new entry describing the real-records criterion rule, citing the 2026-10-29 landing-time figures 0.1 and 273.8 minutes.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Making the verifier itself fetch real records or changing verifier prompts
- Fixing the landing-time measure itself
- Changing the reader, planner token budgets in config

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus cargo test, clippy -D warnings and fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Refusal and acceptance are checked by running tests, not by file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check plus two brief lines and a changelog entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 17 cases in 17 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) against it. A mismatch refuses the reading and prints the re...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

34 more qualifying classes are left out.

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

- path: <state>/jobs/20261029-233025-6ayn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233025-6ayn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-context/src/briefs.rs has exactly one added rule line in the reader brief and one in the planner brief stating that a change to a records-computed measure needs a criterion run against real records with an expected range from an independent source; Grep the diff shows one line each.
Run `cargo test -p athena-context`: the brief size test passes with both new lines, so the reader and planner briefs stay within their token budgets.
Run `cargo test -p athena-core`: a new test builds a code reading whose summary changes a records-computed measure (for example landing time) with only fixture-based criteria and asserts check_reading returns an error naming the missing real-records criterion.
Run `cargo test -p athena-core`: a companion test shows the same reading accepted once it adds a criterion naming real records, an independent source (git log or recorded check durations) and an expected range, and a reading not touching a records measure is unaffected.
The new check's doc comment in crates/athena-core/src/reading/surveys.rs or reading.rs states that it refuses more readings, the higher rate is intended, and declares `expected 3% of newly refused readings`.
CHANGELOG.md has a new entry describing the real-records criterion rule, citing the 2026-10-29 landing-time figures 0.1 and 273.8 minutes.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
