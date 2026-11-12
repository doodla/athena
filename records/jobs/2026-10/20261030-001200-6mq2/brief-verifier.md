# Job 20261030-001200-6mq2: verify "Give OLY-A-07 a mechanism: a spec-and-vocabulary owner check"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4110578f5641 → candidate d3d2457cd3ec. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   3 +
 crates/athena-core/src/hestia.rs | 136 +++++++++++++++++++++++++++++++++++++++
 docs/spec.toml                   |   4 +-
 3 files changed, 141 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-001200-6mq2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-001200-6mq2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43124987282 for d3d2457cd3ecceeb5d2b0b5e72c65ed463cc6abf)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43124987282 for d3d2457cd3ecceeb5d2b0b5e72c65ed463cc6abf)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43124987282 for d3d2457cd3ecceeb5d2b0b5e72c65ed463cc6abf)
- ci-result: `not a check` passed (Check results for d3d2457cd3ecceeb5d2b0b5e72c65ed463cc6abf come from the CI run https://github.com/doodla/athena/actions/runs/43124987282 (820s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
