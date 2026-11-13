# Job 20261030-033229-hb3a: verify "Give OLY-B-A9-01 a mechanism: automation register with retire rule"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A9-01 a mechanism
> 
> Requirement OLY-B-A9-01 in docs/spec.toml is active and has no mechanism.
> Requirement: The automation engineer builds v0 and stays on call, takes toil and repeat causes, delivers reusable capabilities, and registers each automation with its savings and retires those that cost more than they save.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A9-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A9-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small athena-core module holding an automation register (each entry with its cost and measured savings) and a rule that flags automations costing more than they save for retirement. Record it as the mechanism of OLY-B-A9-01 in docs/spec.toml, with a measure ending `status: not shown` that names the test, following the OLY-B-A3-02 pattern.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, 'status: not shown' rule), crates/athena-core/tests/learnings_recorded.rs (the OLY-B-A3-02 pattern: parse_spec, measure_problem, spec_gaps tests), crates/athena-data/src/records.rs (only an 'automation engineer' role string, no register). No record or data file under crates/ is added, so check_record_file/AnyRecord/round-trip are not touched; the register is Rust code with in-test data.
- Existing code: extend the pattern in crates/athena-core/tests/learnings_recorded.rs and use athena_core::hestia::{parse_spec, measure_problem, spec_gaps}; docs/spec.toml is modified, not created.
- Goal: make the spec gap for OLY-B-A9-01 close with a real, tested rule; audience is the Hestia spec pass and the verifier. The full engineer role (on-call, building v0) is prose-level; the enforceable core is savings registration and retirement.
- The measure uses `status: not shown` because no athena status label shows automations; wiring a status line is out of scope.
- No new refusal rule applies to past readings; the retire rule is a standalone function not wired into drain or gates.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [content] In docs/spec.toml, OLY-B-A9-01 has mechanism no longer "none", naming crates/athena-core/src/automation.rs and the test file crates/athena-core/tests/automation_register.rs; owner stays Athena.
2. [content] The OLY-B-A9-01 measure in docs/spec.toml says how the requirement is shown, names `cargo test -p athena-core --test automation_register`, and ends with `status: not shown` (no label from STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs is claimed that athena status does not print).
3. [content] crates/athena-core/src/automation.rs defines an automation register entry (name, owner, build/run cost, measured savings, source toil or repeat cause) and a pure function returning the entries whose cost exceeds their savings, plus entries lacking savings registration; it is exported from crates/athena-core/src/lib.rs.
4. [behavior] Run `cargo test -p athena-core --test automation_register`: tests show an automation costing more than it saves is listed for retirement, one saving more is kept, an entry with no savings is refused, and the real docs/spec.toml entry for OLY-B-A9-01 has a non-none mechanism, owner Athena, a passing measure_problem, and is absent from spec_gaps. The spec test fails if the mechanism reverts to "none".
5. [behavior] Run `cargo fmt --check`, `cargo clippy`, and `cargo test` (or the Athena-recorded passing checks on the same tree): all pass.
6. [content] CHANGELOG.md records the change; no existing entry of docs/decisions.md or records/ is edited.

## Expected deliverables

- crates/athena-core/src/automation.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/automation_register.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding an `athena status` line or label for automations
- Creating an automation engineer pipeline role or prompt
- Wiring the retire rule into the drain or any gate
- Editing docs/decisions.md or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy, test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the named cargo test and check retire/keep outcomes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One module, one test file, one spec edit.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 0f3d5d0e7dd2 → candidate c4b62841f4a8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                    |   1 +
 crates/athena-core/src/automation.rs            |  68 ++++++++++++++
 crates/athena-core/src/lib.rs                   |   3 +
 crates/athena-core/tests/automation_register.rs | 113 ++++++++++++++++++++++++
 docs/spec.toml                                  |   4 +-
 xtask/shard-timings.txt                         |   1 +
 6 files changed, 188 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-033229-hb3a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-033229-hb3a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43339933332 for c4b62841f4a8306692b00054029cf0970c1d7d90)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43339933332 for c4b62841f4a8306692b00054029cf0970c1d7d90)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43339933332 for c4b62841f4a8306692b00054029cf0970c1d7d90)
- ci-result: `not a check` passed (Check results for c4b62841f4a8306692b00054029cf0970c1d7d90 come from the CI run https://github.com/doodla/athena/actions/runs/43339933332 (931s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy`, and `cargo test` (or the Athena-recorded passing checks on the same tree): all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
