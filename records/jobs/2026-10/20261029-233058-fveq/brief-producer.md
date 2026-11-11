# Job 20261029-233058-fveq: Add an improvement-cycle check for OLY-10-02 and record its mechanism

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-10-02 a mechanism
> 
> Requirement OLY-10-02 in docs/spec.toml is active and has no mechanism.
> Requirement: Olympus investigates and resolves the defect on its own, finds a supported process weakness, develops a candidate improvement, evaluates it on representative cases, and records an evidence-based adoption decision.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-10-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-10-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small, tested check in the introspection records layer that decides whether one job's chain shows a repair, a supported process finding, an evaluated candidate and an evidence-based adoption decision. Surface it in `athena status`, then set the OLY-10-02 mechanism and measure in docs/spec.toml. Goal: Athena's owner can see whether the final acceptance chain is shown.

Assumptions:
- Code survey: read crates/athena-data/src/records/introspection.rs (IntrospectionRecord, FailureClass, ProposalRecord, ProposalStatus, ClassStatus types), crates/athena-cli/src/commands.rs (status entry point at status/status_snapshot) and crates/athena-cli/src/display.rs (status renderer, StatusView). I did not read how records validation is wired in detail; the producer must check check_record_file / AnyRecord and the round-trip test if any new record type is added under crates/. Prefer reusing existing ProposalRecord/ProposalStatus as the adoption decision rather than adding a new record type.
- Existing code: crates/athena-data/src/records/introspection.rs (proposals, classes, evidence), crates/athena-cli/src/display.rs status and crates/athena-cli/src/commands.rs status; docs/spec.toml OLY-10-02 entry at line ~1534. No existing 'adoption' term in crates/, so the decision is modelled with ProposalStatus.
- The mechanism is a pure function that, given a job's linked records, reports each of four stages (repair landed, supported finding, candidate evaluated on cases, adoption decision with evidence) as present or missing; `athena status` prints a line such as 'improvement cycle: shown' or 'improvement cycle: not shown'.
- The measure label will be a line `athena status` prints; the producer chooses the exact wording and cites it in the spec measure, ending with `status: <label>`.
- No athena.toml, defaults.toml or workflow change is needed. Consumers: none found (no config deliverable).

## Acceptance criteria

1. [content] docs/spec.toml OLY-10-02 has a mechanism naming an existing crates/ path and function (not "none"), owner Athena kept, and a measure ending in `status: <label>` where the label is a line `athena status` prints, or `status: not shown` plus the test name. Check with Grep on docs/spec.toml and the named path.
2. [behavior] A unit test for the cycle check passes with the mechanism: a record set with repair, supported finding, evaluated candidate and adoption decision reports shown; each set missing one stage reports that stage missing. Run `cargo test -p athena-data introspection` (or the crate holding the check). The test fails if the check is stubbed to always return shown or always missing.
3. [behavior] `athena status` output includes the improvement-cycle line, and a display test in crates/athena-cli asserts it for both shown and not-shown. Run `cargo test -p athena-cli status`.
4. [behavior] An empty record set is reported as not shown, never as success (no-empty-success). Covered by a test.
5. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
6. [content] Any new record or data type is covered by the records validation allow-list and its round-trip test (check_record_file/AnyRecord), or no new record type is added. Check by Grep in crates/athena-data.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Running a real end-to-end demonstration with a live defect
- OLY-10-01, OLY-10-03 and other requirements
- Changing athena.toml, defaults.toml or workflows
- Prompt changes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New check has unit tests; fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise shown and missing cases.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Empty record set reports not shown.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check reusing existing proposal records.
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

- path: <state>/jobs/20261029-233058-fveq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233058-fveq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml OLY-10-02 has a mechanism naming an existing crates/ path and function (not "none"), owner Athena kept, and a measure ending in `status: <label>` where the label is a line `athena status` prints, or `status: not shown` plus the test name. Check with Grep on docs/spec.toml and the named path.
A unit test for the cycle check passes with the mechanism: a record set with repair, supported finding, evaluated candidate and adoption decision reports shown; each set missing one stage reports that stage missing. Run `cargo test -p athena-data introspection` (or the crate holding the check). The test fails if the check is stubbed to always return shown or always missing.
`athena status` output includes the improvement-cycle line, and a display test in crates/athena-cli asserts it for both shown and not-shown. Run `cargo test -p athena-cli status`.
An empty record set is reported as not shown, never as success (no-empty-success). Covered by a test.
`cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
Any new record or data type is covered by the records validation allow-list and its round-trip test (check_record_file/AnyRecord), or no new record type is added. Check by Grep in crates/athena-data.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
