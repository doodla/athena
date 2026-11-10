# Job 20261029-180927-z06n: verify "Give OLY-3-05 a mechanism: ten-dimension introspection coverage"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-3-05 a mechanism
> 
> Requirement OLY-3-05 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena analyses each feedback item along ten dimensions: observation, expectation, reproduction, immediate defect, scope, cause, escape, systemic implication, prevention and validation.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-3-05
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-3-05 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a pure function in athena-core introspect.rs that reports which of the ten feedback dimensions (observation, expectation, reproduction, immediate defect, scope, cause, escape, systemic implication, prevention, validation) an IntrospectionRecord fills, with a test. Record it as OLY-3-05's mechanism and a measure in docs/spec.toml. Audience: the spec verifier and maintainers reading docs/spec.toml.

Assumptions:
- Code survey: I read crates/athena-data/src/records/introspection.rs (IntrospectionRecord fields: what_happened, cause, prevention_kind, prevention, proposed_request, evidence, failure_class, earlier_cases, status), crates/athena-core/src/introspect.rs (pub fns and its tests module at line 1673) and crates/athena-core/src/status.rs (label table around line 302-312, which includes 'mistakes'). The change adds a function in crates/athena-core/src/introspect.rs and adds NO new record or data file and does not touch crates/athena-data/src/records.rs, so check_record_file and AnyRecord need no change; the existing round-trip tests of IntrospectionRecord keep covering the record unchanged. The property is measured directly: the function takes an IntrospectionRecord and returns the dimensions with non-empty content, and the test calls it on a full record and a sparse record.
- Goal: OLY-3-05 gets a real mechanism and a measure ending in 'status: not shown' plus the test name, because no `athena status` line prints a per-dimension share; adding a status line is out of scope.
- Mapping of dimensions to existing fields is my assumption: observation=what_happened, expectation/reproduction/scope=from evidence where present, immediate defect=category, cause=cause, escape=failure_class/earlier_cases, systemic implication=decisions_matched, prevention=prevention, validation=proposed_request. The producer may adjust the mapping if the fields read differently.
- Consumers: none found for a new config key; no config file changes.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-3-05 has mechanism naming the new function by path (crates/athena-core/src/introspect.rs), owner 'Athena' kept, and a measure that ends with 'status: not shown' and names the test; Grep of that function name finds it in introspect.rs.
2. [behavior] Run `cargo test -p athena-core introspect`: the new test passes, asserting a full IntrospectionRecord yields all ten dimensions and a record with only what_happened yields only observation; the test fails if the function returns an empty or fixed set.
3. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root; all pass, including the existing spec-consistency test that checks mechanism paths and measure labels.
4. [content] git diff shows no change to crates/athena-data/src/records.rs, athena.toml, defaults.toml, or any records/ path.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/records/introspection.rs (modify)

## Out of scope

- Adding an `athena status` line that prints per-dimension coverage
- Changing the introspection model prompt or the record schema
- OLY-3-06 and other requirements
- Changes to athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new function has a passing test; fmt, clippy and test are acceptance criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test asserts the dimensions returned for specific records.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One function and one spec entry.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The requirement is tied to code via docs/spec.toml.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c3199cd80f6d → candidate 0c65269346ae. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  4 +++
 crates/athena-core/src/introspect.rs | 70 ++++++++++++++++++++++++++++++++++++
 docs/spec.toml                       |  4 +--
 3 files changed, 76 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-180927-z06n/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-180927-z06n/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42877616157 for 0c65269346aeb2c88d049dfd612e3d18084db44d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42877616157 for 0c65269346aeb2c88d049dfd612e3d18084db44d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42877616157 for 0c65269346aeb2c88d049dfd612e3d18084db44d)
- ci-result: `not a check` passed (Check results for 0c65269346aeb2c88d049dfd612e3d18084db44d come from the CI run https://github.com/doodla/athena/actions/runs/42877616157 (910s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root; all pass, including the existing spec-consistency test that checks mechanism paths and measure labels.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
