# Job 20261030-033242-et3s: verify "Give OLY-B-S2-01 a prompt-surface inventory mechanism and test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-S2-01 a mechanism
> 
> Requirement OLY-B-S2-01 in docs/spec.toml is active and has no mechanism.
> Requirement: For each studied instance Athena inventories every prompt surface, and the prompt analysis is reproducible in code.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-S2-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-S2-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add reproducible code in the study capability that inventories every prompt surface of a studied instance, with a test on the real studies/ data. Record it as OLY-B-S2-01's mechanism, with owner Athena and a measure ending `status: not shown` that names the test.

Assumptions:
- Goal and audience: Athena's spec gap check and the verifier must be able to see that each studied instance has a prompt-surface inventory built by code, not by hand.
- Code survey: read docs/spec.toml (OLY-B-S2-01 entry, mechanism = "none"), crates/athena-core/tests/study_instance_split.rs (model: a test over the real files plus a spec-entry check using parse_spec and measure_problem), crates/athena-cap-study/src/lib.rs (the Capability impl; prompts in prompts/), records.rs, rules.rs and model.rs. Grep found no prompt-surface logic in crates/athena-cap-study or in studies/, so the producer must create it. Entry point is the study capability. Records validation: any new record or data file under crates/ must be covered by check_record_file, AnyRecord or the round-trip test; the producer should prefer no new record file. The property is measured directly by running the inventory over studies/<id>/ and asserting the result.
- Existing code: extend crates/athena-cap-study/src/lib.rs and rules.rs, and reuse the spec helpers parse_spec and measure_problem from athena-core as in crates/athena-core/tests/study_instance_split.rs. docs/spec.toml is modified but not created.
- Design choice: the inventory is a pure function over an instance's files that lists prompt surfaces, deterministic and sorted, so running it twice gives the same output (the reproducibility). The current instance is firstmate-work-handling. The capability must not name any instance (see the S1-02 test).
- The measure follows the S1-02 pattern: it names the test command and ends `status: not shown`, because that each later instance is actually inventoried is not visible in static files.
- Mechanism text in spec.toml should name real paths. Tests go in athena-core/tests or in the unit tests of athena-cap-study, wherever they can import the code.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-S2-01 has mechanism naming an existing code path and test file (not "none"), owner "Athena", and a measure that names the test command and ends with `status: not shown`.
2. [content] The diff adds a prompt-surface inventory function in crates/athena-cap-study/src that names no instance id, and the mechanism text in docs/spec.toml cites its path.
3. [behavior] A new test runs the inventory twice over every real studies/<id>/ directory, asserts non-empty identical output, and asserts the spec entry has a real mechanism, the owner and the measure label. Run `cargo test` and check the test passes. The test must fail on a spec entry with mechanism "none" or an empty inventory (negative cases in the same test file).
4. [behavior] `cargo fmt --check` passes.
5. [behavior] `cargo clippy` is clean.
6. [behavior] `cargo test` passes, including the existing spec-gap and record round-trip tests.
7. [content] CHANGELOG.md records the change.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cap-study/src/prompts_inventory.rs (create)
- crates/athena-core/tests/prompt_inventory.rs (create)
- crates/athena-cap-study/src/rules.rs (modify)
- CHANGELOG.md (modify)
- docs/design/requirements-map.md (modify)
- crates/athena-cap-study/Cargo.toml (modify)

## Out of scope

- Studying new instances or editing existing studies/ data
- Editing docs/decisions.md entries or anything under records/
- Changing other spec requirements

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A test runs the inventory on the real studies data.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The requirement is enforced by a test, recorded in spec.toml.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 91d64c5444ab → candidate f30abe5b5138. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                      |   1 +
 crates/athena-cap-study/src/lib.rs                |   1 +
 crates/athena-cap-study/src/prompts_inventory.rs  | 122 ++++++++++++++++++++++
 crates/athena-cap-study/tests/prompt_inventory.rs | 121 +++++++++++++++++++++
 docs/spec.toml                                    |   4 +-
 xtask/shard-timings.txt                           |   1 +
 6 files changed, 248 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-033242-et3s/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-033242-et3s/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43367478699 for f30abe5b51385a5260c3e976aa2ed2e65e3b9990)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43367478699 for f30abe5b51385a5260c3e976aa2ed2e65e3b9990)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43367478699 for f30abe5b51385a5260c3e976aa2ed2e65e3b9990)
- ci-result: `not a check` passed (Check results for f30abe5b51385a5260c3e976aa2ed2e65e3b9990 come from the CI run https://github.com/doodla/athena/actions/runs/43367478699 (917s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check` passes.; `cargo clippy` is clean.; `cargo test` passes, including the existing spec-gap and record round-trip tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
