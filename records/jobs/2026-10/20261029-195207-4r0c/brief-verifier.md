# Job 20261029-195207-4r0c: verify "Give OLY-4-03 a neighboring-case check and record its mechanism"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-4-03 a mechanism
> 
> Requirement OLY-4-03 in docs/spec.toml is active and has no mechanism.
> Requirement: Neighboring cases are tested where applicable, including nested navigation, browser history, direct links and interrupted flows.
> Owner: Ares
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-4-03
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-4-03 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small pure check in athena-core verify.rs that, for a fix touching navigation, reports which neighboring cases (nested navigation, browser history, direct links, interrupted flows) its added tests do not cover. Record it as OLY-4-03's mechanism and measure in docs/spec.toml, with a test that fails without it. Audience: Ares/Apollo and the verifier judging fixes.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (pure helpers such as is_test_path, finding_is_content_only, with a tests module at the end) and crates/athena-core/src/status.rs (labels like producers). Entry point for the new code is verify.rs next to judge; it does no probing or resolution and adds no record or data file under crates/, so check_record_file and AnyRecord are not touched; the property is measured directly by the function over test text.
- Existing code: extends crates/athena-core/src/verify.rs and docs/spec.toml, both already present; neighbouring requirements OLY-4-01/02 use the same mechanism/measure format.
- Goal is a real, minimal mechanism; wiring it as a blocking verdict rule is not required, so it is a helper plus test, with measure `status: not shown` and the test name unless a status label is added.
- Governance files (athena.toml, defaults.toml, workflows) are unchanged; CHANGELOG.md is only optionally touched.

## Acceptance criteria

1. [content] In docs/spec.toml, requirement OLY-4-03 has a mechanism other than "none" that names the new function in crates/athena-core/src/verify.rs, keeps owner "Ares", and its measure ends with `status: <label>` where the label is a line `athena status` prints, or `status: not shown` plus the test name that shows it.
2. [content] verify.rs defines a pure function that, given the added test text of a navigation-related fix, returns the neighboring cases (nested navigation, browser history, direct links, interrupted flows) that no test covers; Grep finds it and it is reachable from non-test code or is named in the mechanism.
3. [behavior] Run `cargo test -p athena-core neighbor`: the new test asserts a fix whose tests cover only the direct case reports all four missing, and one covering all four reports none; the test fails when the function body is reverted to return an empty list.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.
5. [content] Outcome-check: the spec measure states how neighboring-case coverage is shown, so a reader of docs/spec.toml can find the test that demonstrates it; the change adds no refusal rule to the gate and changes no verdict for existing jobs.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Making the check a blocking verdict rule or changing prompts
- Changing other requirements in docs/spec.toml
- Edits to athena.toml, defaults.toml, workflows

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused helper and test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Test plus fmt, clippy, test pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion on the spec measure.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b39801fc43d3 → candidate 20be3fcb6868. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  3 +++
 crates/athena-core/src/verify.rs | 45 ++++++++++++++++++++++++++++++++++++++++
 docs/spec.toml                   |  4 ++--
 3 files changed, 50 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-195207-4r0c/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-195207-4r0c/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42905976648 for 20be3fcb68686e090f75ad88ab366c2b4c3b59ab)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42905976648 for 20be3fcb68686e090f75ad88ab366c2b4c3b59ab)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42905976648 for 20be3fcb68686e090f75ad88ab366c2b4c3b59ab)
- ci-result: `not a check` passed (Check results for 20be3fcb68686e090f75ad88ab366c2b4c3b59ab come from the CI run https://github.com/doodla/athena/actions/runs/42905976648 (704s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
