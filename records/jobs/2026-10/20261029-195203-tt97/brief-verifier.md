# Job 20261029-195203-tt97: verify "Give OLY-3-06 a mechanism: count items per feedback dimension"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-3-06 a mechanism
> 
> Requirement OLY-3-06 in docs/spec.toml is active and has no mechanism.
> Requirement: Each dimension may produce several hypotheses or work items.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-3-06
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-3-06 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a function in crates/athena-core/src/introspect.rs, beside filled_feedback_dimensions, that reports how many hypotheses or work items an IntrospectionRecord holds for each of the ten FEEDBACK_DIMENSIONS. It counts several where a field is a list. Then record it as OLY-3-06's mechanism and measure in docs/spec.toml, with a test. The audience is the Athena maintainers and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/introspect.rs (FEEDBACK_DIMENSIONS, filled_feedback_dimensions and its test around line 1943); the record type is IntrospectionRecord. Calls to the new function come from tests only. No record or data file is added under crates/, so check_record_file and AnyRecord are untouched. The property is measured directly by calling the new function on records.
- Assumption: 'several' is shown by per-dimension counts that can exceed 1, taken from list fields such as decisions_matched and earlier_cases (one count per entry); single-valued fields count 0 or 1.
- The measure ends with 'status: not shown' and names the test, as for OLY-3-05, because no athena status line prints this.
- Governance: docs/spec.toml is the only config-like deliverable. athena.toml and defaults.toml do not change.
- Existing code: extends crates/athena-core/src/introspect.rs (filled_feedback_dimensions and FEEDBACK_DIMENSIONS) and docs/spec.toml; creates nothing new.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-3-06 has a mechanism naming crates/athena-core/src/introspect.rs and the new function (not 'none'), owner 'Athena', and a measure naming the test and ending with 'status: not shown'. Check with Grep on docs/spec.toml.
2. [behavior] Run cargo test -p athena-core on the new test. It builds a record with several decisions_matched and earlier_cases entries and asserts the per-dimension counts exceed 1 for those dimensions and are 0 for blank ones. The test fails if the function is stubbed to return one per filled dimension.
3. [behavior] cargo fmt --check, cargo clippy and cargo test all pass from the repository root.
4. [content] The new function exists in introspect.rs, is named in the spec mechanism text, and is reached by a test in the same file.

## Expected deliverables

- crates/athena-core/src/introspect.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Printing a per-dimension line in athena status
- Changing other OLY requirements
- Changing athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test, with fmt, clippy and test passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion checks counts above 1 from a real record, not just that the function exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One function and one spec entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 65b0507fbb71 → candidate af6b557e9955. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  2 ++
 crates/athena-core/src/introspect.rs | 37 ++++++++++++++++++++++++++++++++++++
 docs/spec.toml                       |  4 ++--
 3 files changed, 41 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-195203-tt97/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-195203-tt97/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42900315189 for af6b557e99550bc176345365a9defe355ba17782)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42900315189 for af6b557e99550bc176345365a9defe355ba17782)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42900315189 for af6b557e99550bc176345365a9defe355ba17782)
- ci-result: `not a check` passed (Check results for af6b557e99550bc176345365a9defe355ba17782 come from the CI run https://github.com/doodla/athena/actions/runs/42900315189 (800s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy and cargo test all pass from the repository root.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
