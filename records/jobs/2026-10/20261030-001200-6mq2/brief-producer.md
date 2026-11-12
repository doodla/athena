# Job 20261030-001200-6mq2: Give OLY-A-07 a mechanism: a spec-and-vocabulary owner check

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-A-07 a mechanism
> 
> Requirement OLY-A-07 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena has a requirements engineer that owns the spec and the shared vocabulary, and a planner then decides how to build it.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-A-07
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-A-07 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Build the requirements-engineer function as a code path in the existing Hestia spec review (`Athena::spec_review`). It owns both docs/spec.toml and the shared vocabulary docs/ontology.toml and reports gaps in either. Then record it as OLY-A-07's mechanism and a measure ending `status: requirement`. The audience is Sri and the verifier, who can check it with cargo test.

Assumptions:
- Code survey: I read crates/athena-core/src/hestia.rs (`Athena::spec_review` at line 1449, called at line 3195; its tests start at line 4054), crates/athena-core/src/status.rs (`STATUS_MEASURE_LABELS` at line 305, which includes the `requirement` label backed by the `spec_gaps` field), xtask/tests/spec.rs (`check_how_it_is_met`) and crates/athena-data/src/records.rs. Entry point: the Hestia sweep calls `spec_review`, which parses docs/spec.toml and queues a request for each active requirement whose mechanism is `none` or whose measure lacks a valid `status:` marker. The new code extends that review path, so the mechanism, owner and measure checks keep running in the order they run now. No existing function reads docs/ontology.toml, so the new check adds that read after the spec checks.
- Records validation: the change adds no record or data file under crates/ and does not touch crates/athena-data/src/records.rs. A new file would need `check_record_file` and the `AnyRecord` enum (records.rs lines 1415 and 1089) to accept it, plus a round-trip test in that file. The change only adds a function and tests in hestia.rs, so none of that applies. Records remain checked by `check_record_file`.
- Direct measurement: the property is that the review names gaps in the spec and the vocabulary. A unit test in hestia.rs builds a fixture spec and ontology with a gap and asserts that the new function reports it. The same test asserts nothing is reported for a clean fixture, so it fails without the mechanism. The measure is read from the `spec_gaps` field that `athena status` prints under the `requirement` label.
- Design choice: the 'requirements engineer' is a code path inside the Hestia spec review, not a new LLM pipeline role. A new role would need a prompt set and a prompt-token budget, and the reader prompt has 2 tokens left. The planner already exists, so the second half of the requirement, a planner then deciding how to build it, is met by the current pipeline. The new check reports each gap as a request for a planner to act on.
- Existing code: this extends the existing `Athena::spec_review` and `SpecReport` in crates/athena-core/src/hestia.rs and reuses `STATUS_MEASURE_LABELS` in crates/athena-core/src/status.rs. It also reuses docs/ontology.toml as the vocabulary source. It builds no parallel spec parser.
- Governance files: athena.toml, defaults.toml and .github/workflows/ are not needed, because the check runs inside the existing sweep. Consumers: none found for any new config. The only non-code file that changes is docs/spec.toml.
- Refusal rule: the new check queues requests and refuses no readings, so no refusal-rate replay applies.
- Measure text will be: `Spec requirements without a mechanism, owner or measure, and ontology terms without a source; status: requirement`. The mechanism text will name `Athena::spec_review` and the new function in crates/athena-core/src/hestia.rs.

## Acceptance criteria

1. [content] In docs/spec.toml, requirement OLY-A-07 has a mechanism other than `none` that names `Athena::spec_review` and the new function in crates/athena-core/src/hestia.rs. Grep confirms that function exists in that file. The owner is still Athena, and the measure ends with `status: requirement` or `status: not shown` plus a named test.
2. [behavior] Run `cargo test -p athena-core hestia`. A new test passes: the spec-and-vocabulary check reports a fixture spec requirement with an empty owner or measure, and an ontology term with no source, and reports nothing for a clean fixture. The verifier can confirm the test fails when the new function's body is removed or stubbed to return no gaps.
3. [behavior] Run `cargo test -p xtask --test spec`. It passes with the edited OLY-A-07 entry, so the mechanism, owner, measure marker and verbatim source quotes are all valid.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root.
5. [content] The diff touches only docs/spec.toml, crates/athena-core/src/hestia.rs, possibly crates/athena-core/src/status.rs, and CHANGELOG.md. It adds no record file and does not edit athena.toml, defaults.toml or the prompt files.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- docs/ontology.toml (modify)

## Out of scope

- Adding a new LLM pipeline role, prompt set or config entry for a requirements engineer
- Changing any other requirement in docs/spec.toml
- Editing BRIEF.md, athena.toml, defaults.toml or records/
- Building a formal ontology; OLY-A-08 says none is built for now

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require a new test that fails without the mechanism, plus cargo test and clippy passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run tests that exercise the gap report rather than checking that the code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the existing spec review path, with no new role.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The measure points at an `athena status` label, so the requirement's outcome is shown.
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

- path: <state>/jobs/20261030-001200-6mq2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-001200-6mq2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In docs/spec.toml, requirement OLY-A-07 has a mechanism other than `none` that names `Athena::spec_review` and the new function in crates/athena-core/src/hestia.rs. Grep confirms that function exists in that file. The owner is still Athena, and the measure ends with `status: requirement` or `status: not shown` plus a named test.
Run `cargo test -p athena-core hestia`. A new test passes: the spec-and-vocabulary check reports a fixture spec requirement with an empty owner or measure, and an ontology term with no source, and reports nothing for a clean fixture. The verifier can confirm the test fails when the new function's body is removed or stubbed to return no gaps.
Run `cargo test -p xtask --test spec`. It passes with the edited OLY-A-07 entry, so the mechanism, owner, measure marker and verbatim source quotes are all valid.
`cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root.
The diff touches only docs/spec.toml, crates/athena-core/src/hestia.rs, possibly crates/athena-core/src/status.rs, and CHANGELOG.md. It adds no record file and does not edit athena.toml, defaults.toml or the prompt files.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
