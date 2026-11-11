# Job 20261029-233102-c04w: Give OLY-10-03 a mechanism: later-task prevention check in status

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-10-03 a mechanism
> 
> Requirement OLY-10-03 in docs/spec.toml is active and has no mechanism.
> Requirement: A later similar task shows the improved process either prevents the defect or detects it earlier.
> Owner: Apollo
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-10-03
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-10-03 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a records-based check that, for an accepted defaults proposal, looks at later same-class work and reports whether the defect was prevented (no recurrence in later landed jobs) or detected earlier. Athena status prints it as a line. Fill the mechanism and measure for OLY-10-03 in docs/spec.toml, with a test that fails without the check.

Assumptions:
- Goal and audience: the owner (Apollo) and the verifier judge whether OLY-10-03 is shown by records, via a line `athena status` prints, as OLY-10-02 is.
- Code survey: read crates/athena-data/src/records/introspection.rs (ProposalRecord, ProposalStatus, ProposalEvaluation, improvement_cycle and its tests), crates/athena-core/src/status.rs (improvement_cycle_line, landed_job_ids, StatusView construction near line 1310) and crates/athena-cli/src/display.rs line 1330 (prints the line). Entry point is the status view build; the new function is pure over records, like improvement_cycle. No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched; existing record types are reused with no schema change.
- Existing code: extend crates/athena-data/src/records/introspection.rs (IntrospectionRecord, ProposalRecord with failure_class and accepted status, improvement_cycle), crates/athena-core/src/status.rs (improvement_cycle_line pattern, StatusView) and crates/athena-cli/tests/status_build_measurement.rs; the measure line follows the improvement_cycle pattern.
- Term later similar task: a landed job or introspection dated after an accepted proposal was decided, of the same failure class (or class_of_work in applies_to). Prevented means such a later landed job has no recorded introspection of that class; detected earlier means a later introspection of the class exists but at an earlier category/stage than the earlier cases. The planner may narrow this to what the records can measure directly.
- Nothing recorded or no accepted proposal gives `not shown`, never success (no-empty-success).
- The new status line has a fixed prefix (for example `later similar task:`) and a `shown` or `not shown (...)` form; the spec measure ends with `status: <that label>` or `status: not shown` plus the test name.
- Governance files athena.toml, defaults.toml and workflows are not changed; docs/spec.toml is the only non-crate deliverable.

## Acceptance criteria

1. [content] docs/spec.toml OLY-10-03 has mechanism naming the new function path in crates/athena-data/src/records/introspection.rs and the status.rs line function (no longer "none"), owner still Apollo, and a measure ending in `status: not shown` or a label printed by `athena status`; Grep of the entry shows it.
2. [behavior] A new unit test in crates/athena-data/src/records/introspection.rs shows: with an accepted proposal and a later same-class landed job with no recurrence, the check reports prevented; with a later recurrence at an earlier stage it reports detected earlier; with a recurrence at the same or later stage, or with no records, it reports not shown. Run `cargo test -p athena-data introspection`; the test fails if the function body is stubbed to always report not shown.
3. [behavior] `athena status` output includes the new later-similar-task line, with the status.rs unit test and a crates/athena-cli test asserting the line equals the StatusView field for both shown and not-shown outcomes. Run `cargo test -p athena-core status` and `cargo test -p athena-cli status`.
4. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass across the workspace.
5. [content] A recurrence of the same class before the proposal was accepted, or a proposal that is only pending or rejected, does not count as the improved process (checked by a negative test in the same test module).

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_build_measurement.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing OLY-10-01, OLY-10-02 or other requirements
- Changing the proposal, evaluation or introspection record schemas
- Changing athena.toml, defaults.toml or CI workflows
- Writing real records or running a live demonstration job

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the prevented/detected-earlier outcome via tests and the status line, not that the code exists.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Empty records yield not shown.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test pass.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 18 cases in 18 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: When a criterion says a test must assert a property of rendered output, the producer's brief should require two things before the job is done. The test must run against the real rendered page or the real source data. And it must be shown to fail on the known bad input, here the current duplicated te...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

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

- path: <state>/jobs/20261029-233102-c04w/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233102-c04w/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml OLY-10-03 has mechanism naming the new function path in crates/athena-data/src/records/introspection.rs and the status.rs line function (no longer "none"), owner still Apollo, and a measure ending in `status: not shown` or a label printed by `athena status`; Grep of the entry shows it.
A new unit test in crates/athena-data/src/records/introspection.rs shows: with an accepted proposal and a later same-class landed job with no recurrence, the check reports prevented; with a later recurrence at an earlier stage it reports detected earlier; with a recurrence at the same or later stage, or with no records, it reports not shown. Run `cargo test -p athena-data introspection`; the test fails if the function body is stubbed to always report not shown.
`athena status` output includes the new later-similar-task line, with the status.rs unit test and a crates/athena-cli test asserting the line equals the StatusView field for both shown and not-shown outcomes. Run `cargo test -p athena-core status` and `cargo test -p athena-cli status`.
`cargo fmt --check`, `cargo clippy` and `cargo test` pass across the workspace.
A recurrence of the same class before the proposal was accepted, or a proposal that is only pending or rejected, does not count as the improved process (checked by a negative test in the same test module).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
