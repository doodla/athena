# Job 20261029-233058-fveq: verify "Add an improvement-cycle check for OLY-10-02 and record its mechanism"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b6f5109e731e → candidate 4e675782f02b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |   3 +
 crates/athena-cli/src/display.rs                   |   1 +
 .../athena-cli/tests/status_build_measurement.rs   |  31 +++
 crates/athena-core/src/status.rs                   |  20 ++
 crates/athena-data/src/records/introspection.rs    | 246 +++++++++++++++++++++
 docs/spec.toml                                     |   4 +-
 6 files changed, 303 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-233058-fveq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233058-fveq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43017870618 for 4e675782f02bb62a9dd5836a525cbdc5625dbbb3)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43017870618 for 4e675782f02bb62a9dd5836a525cbdc5625dbbb3)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43017870618 for 4e675782f02bb62a9dd5836a525cbdc5625dbbb3)
- ci-result: `not a check` passed (Check results for 4e675782f02bb62a9dd5836a525cbdc5625dbbb3 come from the CI run https://github.com/doodla/athena/actions/runs/43017870618 (730s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
