# Job 20261109-075120-7zqw: Refuse oversized multi-surface readings before dispatch

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Nothing before dispatch measures whether a request fits one producer run. This job touches a decisio
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: multi-surface-job-producer-makes-no-change-and-cannot-report (20261025-185414-aapc)
> Cause: Nothing before dispatch measures whether a request fits one producer run. This job touches a decision doc, the tool pins in athena.toml, the CI workflow, the check runner's output parsing in checks.rs and gh.rs, new unit tests and the chang...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261025-185359-yvbn (job 20261025-183644-t67q); prevention it proposed: Add a pre-dispatch size check and a no-change guard for the producer. (1) Before dispatch, count the distinct deliverable surfaces and criteria in the reading, and send any job over a set bound back to be split. (2) If the producer ends wit...
> Introspection 20261109-075105-011b (job 20261017-205554-zaj2); prevention it proposed: Add a check before dispatch that splits a request spanning separable surfaces (core snapshot and landing code with tests, versus config and changelog) into one job per change, ordered by dependency. Also require a producer that ends with an...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: no step in the reading gate (`refusing_check`, crates/athena-core/src/reading/checks.rs) measures whether a request fits one producer run. Jobs 20261025-183644-t67q (10 deliverables over docs, config, CI and code) and 20261017-205554-zaj2 (12 deliverables) ended with no change. I add a `check_job_size` step that sends back, to be split, any change reading spanning too many deliverables or separable surfaces. Both earlier attempts are addressed this way: their size problem no longer reaches dispatch, and neither left a candidate to keep or drop.

Assumptions:
- Code survey: crates/athena-core/src/reading/checks.rs (`refusing_check` runs the `step!` checks in order, and `check_scope_limit` is the nearest precedent), crates/athena-core/src/reading.rs (the gate tests, `gate_refuser`, the step-order tests) and crates/athena-core/src/reading/surveys.rs. The property is measured directly on `Reading.deliverables` and `Reading.acceptance_criteria`, not through a stand-in. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved.
- Goal and audience: the Athena operator and the gate's later readers; a job that is too large is stopped at reading time with a repair, instead of costing a producer run that ends with no change.
- Size bound: count deliverables other than the always-written CHANGELOG.md, and count distinct surfaces (docs, config such as athena.toml, .github/workflows, xtask, tests, and each crates/<crate>). Refuse a change reading above 8 deliverables or 4 surfaces. The exact numbers are a design choice, set so both failing jobs are refused. Study readings and readings with a question are skipped, as in check_scope_limit.
- The 'producer ends with no change and cannot report' half of the proposed prevention is out of this job. Pre-dispatch splitting prevents the class at its origin, and `run/producer.rs` already reports a no-change outcome.
- The retried jobs left no candidate (no recorded changes), so no earlier work is kept or dropped. Their deliverables (docs/decisions.md, athena.toml, ci.yml, xtask, athena-data config, the athena-git snapshot, and others) are not in this job's scope.
- Consumers: none found. No config file is a deliverable, and this job does not change athena.toml, defaults.toml or any workflow.
- Prompt files are untouched because the reader prompt has only 71 tokens of headroom; the refusal error message carries the repair text.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/checks.rs has `pub fn check_job_size(reading: &Reading) -> Result<()>` and one `step!(check_job_size(reading));` line in `refusing_check`. Its doc comment names the root cause (nothing before dispatch measured whether a request fits one producer run) and cites jobs 20261025-183644-t67q and 20261017-205554-zaj2.
2. [behavior] Run `cargo test -p athena-core`. A new test builds a change reading shaped like job 20261025-183644-t67q (docs, athena.toml, CI workflow, two crates files, xtask, tests, CHANGELOG) and one shaped like 20261017-205554-zaj2 (12 deliverables). `check_job_size` refuses both, and the error says to split the job by surface and dependency. The test fails when the `step!` line is removed.
3. [behavior] Run `cargo test -p athena-core`. Tests show the bound is general: a focused reading with a few deliverables in one or two surfaces passes. A reading with many deliverables in one crate is refused on count alone, and a study reading and a reading with a question are not refused.
4. [behavior] Run `cargo test -p athena-core`. The refusal corpus (crates/athena-core/tests/refusal_corpus.rs) has a case in which check_job_size is the refusing check, so the corpus test that every `step!` has a refusing case still passes. The existing gate-order tests still pass.
5. [content] The check refuses more readings and the higher refusal rate is intended: expected 6% of newly refused readings, an estimate and not a measurement. CHANGELOG.md has an entry that names the root cause and the new bound.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`: all pass.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Splitting or redoing job 20261025-183644-t67q (nextest adoption, docs/decisions.md, athena.toml, ci.yml, xtask) or job 20261017-205554-zaj2 (study snapshot isolation, Hermes Agent, the systems schema)
- A producer-side no-change guard or changes to run/producer.rs and run/send_back.rs
- Prompt files, athena.toml, defaults.toml and .github/workflows
- Automatic splitting of a request into several jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus a refusal corpus case; fmt, clippy and test are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a gate step at the origin, not a patch for either job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test on readings shaped like the two failed jobs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 12 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must size the slice to reach the limit. The producer must also run...
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

- path: <state>/jobs/20261109-075120-7zqw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-075120-7zqw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading/checks.rs has `pub fn check_job_size(reading: &Reading) -> Result<()>` and one `step!(check_job_size(reading));` line in `refusing_check`. Its doc comment names the root cause (nothing before dispatch measured whether a request fits one producer run) and cites jobs 20261025-183644-t67q and 20261017-205554-zaj2.
Run `cargo test -p athena-core`. A new test builds a change reading shaped like job 20261025-183644-t67q (docs, athena.toml, CI workflow, two crates files, xtask, tests, CHANGELOG) and one shaped like 20261017-205554-zaj2 (12 deliverables). `check_job_size` refuses both, and the error says to split the job by surface and dependency. The test fails when the `step!` line is removed.
Run `cargo test -p athena-core`. Tests show the bound is general: a focused reading with a few deliverables in one or two surfaces passes. A reading with many deliverables in one crate is refused on count alone, and a study reading and a reading with a question are not refused.
Run `cargo test -p athena-core`. The refusal corpus (crates/athena-core/tests/refusal_corpus.rs) has a case in which check_job_size is the refusing check, so the corpus test that every `step!` has a refusing case still passes. The existing gate-order tests still pass.
The check refuses more readings and the higher refusal rate is intended: expected 6% of newly refused readings, an estimate and not a measurement. CHANGELOG.md has an entry that names the root cause and the new bound.
Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`: all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
