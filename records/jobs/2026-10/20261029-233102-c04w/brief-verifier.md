# Job 20261029-233102-c04w: verify "Give OLY-10-03 a mechanism: later-task prevention check in status"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b853a0e5c21b → candidate 7d1ebf67fdab. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |   6 +
 crates/athena-cli/src/display.rs                   |   1 +
 .../athena-cli/tests/status_build_measurement.rs   |  26 +++
 crates/athena-core/src/status.rs                   |  44 ++++
 crates/athena-data/src/records/introspection.rs    | 226 +++++++++++++++++++++
 docs/spec.toml                                     |   4 +-
 6 files changed, 305 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-233102-c04w/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233102-c04w/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (2975 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` pass across the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
