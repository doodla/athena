# Job 20261030-001204-nkgn: verify "Add Olympus re-triage of queued proposals as OLY-A-09 mechanism"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-A-09 a mechanism
> 
> Requirement OLY-A-09 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena re-triages the requests and proposals queued before Olympus, keeping what still matters and closing the rest with a reason.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-A-09
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-A-09 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a pure re-triage function in athena-core that, for each still-open work-item proposal queued before Olympus (2026-10-17), keeps it if the steward still raises its finding key and otherwise closes it with a recorded rejection reason. Record it as OLY-A-09's mechanism in docs/spec.toml with a measure ending `status: not shown` and naming the test.

Assumptions:
- Goal and audience: the owner (Sri) and the spec verifier need OLY-A-09 to name a real, tested mechanism; the xtask spec test and Hestia spec_gaps pass over it.
- Code survey: read crates/athena-core/src/work_decisions.rs (open_proposals, decision_of, rejection, reject_work, which appends an athena.work-decision/1 record), crates/athena-data/src/records/work.rs (WorkItem, WorkItemKind::Proposal, WorkDecisionState::Rejected{reason}), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS; there is no label for triage), and crates/athena-core/src/hestia.rs (spec_gaps). No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched; the existing WorkDecision record is reused.
- Existing code: extend crates/athena-core/src/work_decisions.rs (or a small sibling module athena-core/src/retriage.rs) reusing open_proposals, decision_of and rejection; docs/spec.toml OLY-A-09 entry (lines ~1724-1733) is edited.
- Scope reading: 'requests and proposals queued before Olympus' is read as open work items of kind Proposal with `at` before the Olympus date 2026-10-17 (the heading date in the spec source). Queued job requests are not a separate queue in the records, so they are out of this mechanism; this assumption is stated in the mechanism text.
- Keep rule: a pre-Olympus open proposal is kept if its `key` is in the set of finding keys the steward still raises; every other one is closed with a rejection whose reason says it is no longer raised after Olympus. The function takes that key set as input so the test is deterministic.
- The measure keeps `status: not shown` because no `athena status` label shows re-triage; the measure names the test (cargo test -p athena-core retriage). No change to STATUS_MEASURE_LABELS.
- No governance files (athena.toml, defaults.toml, workflows) change.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-A-09 has `mechanism` naming the new function and its file under crates/athena-core/src (no longer "none"), keeps owner = "Athena", and its `measure` ends with `status: not shown` and names the test command `cargo test -p athena-core retriage`.
2. [behavior] Run `cargo test -p athena-core retriage`: a test builds records with open Proposal work items before and after 2026-10-17; the re-triage keeps those whose key is still raised and after-Olympus ones, and closes the rest with a non-empty reason, appearing as Rejected work decisions so open_proposals no longer lists them. The test fails if the function is stubbed to keep everything.
3. [behavior] Run `cargo test -p athena-core retriage`: a second test shows re-running re-triage is idempotent (already decided items are not decided again) and an empty queue yields no decisions and is not reported as success of closing anything.
4. [behavior] Run `cargo test -p xtask` and `cargo test -p athena-core hestia`: the spec test and spec_gaps accept OLY-A-09 with the new mechanism and measure.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
6. [content] CHANGELOG.md is not required to change; the deliverables are limited to docs/spec.toml and the athena-core source file(s) holding the function and its tests.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/retriage.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Re-triaging queued job requests that are not work items
- Adding a new `athena status` label or changing STATUS_MEASURE_LABELS
- Changing athena.toml, defaults.toml or workflows
- Calling the new function from the drain loop or a CLI command

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests that fail without the mechanism; fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the retriage test and check decisions recorded, not that the file exists.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A criterion covers the empty queue not counting as success.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module and a spec edit.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The requirement is backed by code and a test, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 05a4ef1d2cbb → candidate a7809eba6103. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   4 ++
 crates/athena-core/src/lib.rs      |   1 +
 crates/athena-core/src/retriage.rs | 120 +++++++++++++++++++++++++++++++++++++
 docs/spec.toml                     |   4 +-
 4 files changed, 127 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-001204-nkgn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-001204-nkgn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43128477843 for a7809eba6103e5b98a7af774401b910707e35772)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43128477843 for a7809eba6103e5b98a7af774401b910707e35772)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43128477843 for a7809eba6103e5b98a7af774401b910707e35772)
- ci-result: `not a check` passed (Check results for a7809eba6103e5b98a7af774401b910707e35772 come from the CI run https://github.com/doodla/athena/actions/runs/43128477843 (708s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
