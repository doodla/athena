# Job 20261030-044531-ncyj: verify "Give OLY-B-S4-01 a mechanism: check mechanism explanations' evidence"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-S4-01 a mechanism
> 
> Requirement OLY-B-S4-01 in docs/spec.toml is active and has no mechanism.
> Requirement: For each instance the study explains how each mechanism works through the prompts, code and observed runs, using Athena's own records as primary evidence for Athena.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-S4-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-S4-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a pure check in athena-core, `mechanism_explanation.rs`. It flags any mechanism explanation that lacks a prompt, code or observed-run source, or, for the Athena instance, lacks a source from Athena's own records. It is tested like OLY-B-S3-02. Then fill the mechanism, owner and measure of OLY-B-S4-01 in docs/spec.toml, with the measure ending `status: not shown`. The audience is Athena's spec-gap check and the verifier.

Assumptions:
- Code survey: I read crates/athena-core/src/divergence.rs (a pure `reconcile` function with Claim, Observation and Divergence types), crates/athena-core/tests/study_instance_split.rs (the S1-02 pattern: it uses parse_spec, spec_gaps and measure_problem from hestia.rs, checks the real docs/spec.toml entry, and has negative tests), crates/athena-core/tests/divergence.rs (the S3-02 pattern), crates/athena-core/src/lib.rs (module registration with docs; divergence is at line 112), and crates/athena-cap-study/src/rules.rs (the study validation rules, with the evidence rules uncited_claim, bad_citation and stated_without_source).
- Code survey, continued: the new module is reached through `athena_core::mechanism_explanation::...`, so the integration test imports it. It adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly: the function receives explanation records and returns a gap for each missing evidence kind.
- Existing code: extend the S3-02/S1-02 pattern in crates/athena-core/src/divergence.rs, crates/athena-core/tests/study_instance_split.rs and `hestia::{parse_spec, spec_gaps, measure_problem}`. The new module sits beside them and reuses those spec helpers for the entry check.
- Goal: the requirement says a study explains each mechanism through prompts, code and observed runs, and that for Athena its own records are primary evidence. The mechanism is a code check that refuses an explanation missing one of those evidence kinds. The instance is the pure check inputs, not the athena study, because no Athena study exists yet (only studies/firstmate-work-handling).
- Because no Athena study exists, the measure honestly ends `status: not shown`. It names `cargo test -p athena-core --test mechanism_explanation`, as OLY-B-S1-02 does.
- Owner stays `Athena`. No governance file (athena.toml, defaults.toml, workflows) changes. The change adds a CHANGELOG.md entry by default.
- Term ladder: none used. 'Instance' means a studied system (firstmate, Hermes, Athena); 'evidence kind' means one of prompt, code, run, record.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-S4-01 has a mechanism that is not `none`. It names the new module crates/athena-core/src/mechanism_explanation.rs and the test crates/athena-core/tests/mechanism_explanation.rs. The owner is `Athena`. The measure ends with `status: not shown` and names `cargo test -p athena-core --test mechanism_explanation`. Check it with Grep on docs/spec.toml.
2. [content] crates/athena-core/src/mechanism_explanation.rs exists and is registered as `pub mod mechanism_explanation` in crates/athena-core/src/lib.rs, with a doc line. It exposes a pure function that takes explanation records, each with an instance, a mechanism and typed evidence sources (prompt, code, run, record), and returns a gap for each mechanism missing a prompt, code or run source. For the Athena instance it also returns a gap when no source is one of Athena's own records.
3. [behavior] Run `cargo test -p athena-core --test mechanism_explanation`. Tests pass for a complete explanation (no gaps) and fail on planted defects: a missing prompt source, a missing code source, a missing run source, and an Athena explanation with only non-record sources. Each test asserts the specific gap. A non-Athena instance without record evidence is not flagged. The real spec entry test passes: `parse_spec` finds OLY-B-S4-01, the mechanism is not `none`, `measure_problem` returns None, the label check passes, and `spec_gaps` no longer lists it.
4. [behavior] A test fails without the mechanism. A spec-check test sets the mechanism back to `none` and asserts `spec_gaps` reports `no_mechanism`. A test asserts that a wrong status label, a measure without the test command, and a wrong owner are each refused. Run `cargo test -p athena-core --test mechanism_explanation`.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace`. All pass, or the recorded Athena check on the final tree shows them passing (D115). Other tests that read docs/spec.toml, such as crates/athena-gate/tests/bootstrap_requirement.rs, still pass.
6. [content] CHANGELOG.md gains an entry for this change. No existing entry of docs/decisions.md or anything under records/ is edited.

## Expected deliverables

- crates/athena-core/src/mechanism_explanation.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/mechanism_explanation.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Writing the Athena study instance under studies/ or running a study of Athena.
- Changing the study capability in crates/athena-cap-study or its prompts.
- Mechanisms for other requirements (OLY-B-S2-01, OLY-B-S5-01).
- Edits to athena.toml, defaults.toml or workflows.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check the refusal behavior by running the test, not only that files exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The fmt, clippy and test criterion covers the whole workspace.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The module and test carry doc comments that name the requirement and its limits.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: There is one pure module and one test file, as for S3-02.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: The real-spec test parses docs/spec.toml with parse_spec and finds OLY-B-S4-01. A Grep of docs/spec.toml shows the same entry. Run: `cargo test -p athena-core --test mechanism_explanation spec_entry_has_a_real_mechanism`. Observes: The mechanism is not `none` and contains crates/athena-core/src/mechanism_explanation.rs and crates/athena-core/tests/mechanism_explanation.rs. The owner is `Athena`. The measure ends `status: not shown`, names `cargo test -p athena-core --test mechanism_explanation`, and states the two limits.
  Fails without the behavior: With the entry left at mechanism `none` and measure `...; status: requirement`, the not-none, file-name and label assertions fail.
  Test seam: Entry point: hestia::parse_spec(&str) (crates/athena-core/src/hestia.rs:1172) on the real docs/spec.toml, read via a CARGO_MANIFEST_DIR-relative path as study_instance_split.rs does. Fixture: the real file; nothing injected. The seam exists.
- Criterion 2: The test file imports athena_core::mechanism_explanation::{check, Explanation, Evidence, EvidenceKind, Gap}, so it compiles only if the module is `pub mod mechanism_explanation` in lib.rs. A Grep of lib.rs shows the registration between `manage` and `method_experiment`, and the doc bullet. The tests drive the pure function. Run: `cargo test -p athena-core --test mechanism_explanation`. Observes: The module compiles and is public. check returns [] for a complete explanation, one MissingEvidence for each absent Prompt, Code or Run, and NoOwnRecords for an Athena explanation with no Record source.
  Fails without the behavior: Without the module or its registration the integration test does not compile. Without the Athena rule the NoOwnRecords test fails.
  Test seam: Entry point: mechanism_explanation::check(&[Explanation]) -> Vec<Gap>, added by steps 1-2. Fixture: in-test Explanation values passed as the argument. No double.
- Criterion 3: The tests are: complete_explanation_has_no_gaps; missing_prompt_is_reported; missing_code_is_reported; missing_run_is_reported; blank_source_does_not_count; athena_with_only_prompt_code_run_is_reported; athena_with_a_record_is_complete; athena_match_ignores_case; non_athena_without_records_is_not_flagged (instance `hermes`); several_defects_give_gaps_in_order; empty_input_gives_no_gaps (documents the stated limit); and spec_entry_has_a_real_mechanism. That last test asserts mechanism != none, measure_problem(&r.measure) == None, the label check passes, and spec_gaps(&[r]) does not list OLY-B-S4-01. Run: `cargo test -p athena-core --test mechanism_explanation`. Observes: Each defect test asserts equality with the exact Gap value, for example Gap::MissingEvidence{instance:"hermes", mechanism:"m", kind:EvidenceKind::Code}. The non-Athena case asserts an empty result. The spec test asserts spec_gaps omits OLY-B-S4-01.
  Fails without the behavior: If check ignored a kind, the matching exact-gap assertion fails. If it flagged non-Athena instances for missing records, the not-flagged test fails. If a blank source counted, the blank test fails.
  Test seam: Entry points: mechanism_explanation::check (added by steps 1-2) and hestia::{parse_spec, measure_problem, spec_gaps} (exist, hestia.rs:1172, 1219, 1291). Fixtures: in-test Explanation values and the real docs/spec.toml. Nothing faked.
- Criterion 4: The negative tests copy the real requirement and change one field. spec_check_refuses_a_none_mechanism sets mechanism `none` and asserts spec_gaps(&[r]) has one gap with `no_mechanism`. Three more tests assert that a measure ending `status: requirement`, a measure without the test command, and owner `Hestia` are each refused by the copied measure_label_problem and owner_problem helpers, as in study_instance_split.rs:294-312. Run: `cargo test -p athena-core --test mechanism_explanation`. Observes: The none-mechanism entry is reported as `no_mechanism`; the wrong label, the missing command and the wrong owner each give Some(problem).
  Fails without the behavior: Without the real mechanism text the positive spec test fails. Without the spec_gaps rule or the helpers the negative tests fail. The none-mechanism test shows the original entry was a gap.
  Test seam: Entry point: hestia::spec_gaps (exists, hestia.rs:1291) over a SpecRequirement cloned from the parsed real entry with one field set to a bad value. Fixture: parse_spec on the real file, then an in-test field change. The seam exists, as used in study_instance_split.rs:284-312.
- Criterion 5: Run the three Athena checks on the final tree, one at a time if they cannot be chained. The run includes crates/athena-gate/tests/bootstrap_requirement.rs, xtask/tests/spec.rs (validator of the whole spec.toml), xtask/tests/ci_shards.rs, and the other spec readers listed in dependents. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. checked_in_timings_cover_every_workspace_test_unit passes only because the shard-timings line is added. xtask/tests/spec.rs passes because the edited entry keeps its source quote and has a non-empty mechanism, an owner and a measure containing `status:`. fmt passes because the module sits in sorted position between `manage` and `method_experiment`.
  Fails without the behavior: Omitting the shard-timings line fails checked_in_timings_cover_every_workspace_test_unit. A clippy lint or unformatted code fails the matching check. Placing the mod line out of order fails `cargo fmt --check`.
  Test seam: Entry points are the cargo commands themselves; no double. New files under crates/athena-core/tests/ are discovered by cargo.
- Criterion 6: Read the diff of CHANGELOG.md and docs/decisions.md. The CHANGELOG bullet goes first under `### Added` in [Unreleased], like the S3-02 bullet. The decision is appended after the last line of docs/decisions.md. A fresh-eyes `git diff` shows only added lines in both files and nothing under records/. The append-only and next-id rules are the mirrors checks. Run: `git diff --stat`. Observes: CHANGELOG.md gains a bullet naming OLY-B-S4-01, the module, the test command, `status: not shown` and the timings registration. docs/decisions.md gains only a final `## D<n> · ...` section, where n is the next id after the last heading in the base (D176 at this snapshot). No line is removed from either file.
  Fails without the behavior: Removing or editing an earlier line of docs/decisions.md makes the mirrors append-only check report 'docs/decisions.md is append-only' (mirrors.rs:265). A wrong id fails Rule::DecisionEntries (mirrors.rs:1054). A missing CHANGELOG bullet shows as an empty diff of that file.
  Test seam: Entry points: the mirrors checks Rule::DecisionEntries and the append-only check (crates/athena-core/src/mirrors.rs:143, 241-265; exercised by its tests near mirrors.rs:1817-1916), plus the git diff of CHANGELOG.md. Fixture: the base and final trees. The seam exists.

Planned files:

- crates/athena-core/src/mechanism_explanation.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/mechanism_explanation.rs (create)
- docs/spec.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f19d0a1df86a → candidate 0b7c49496a6c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                      |   1 +
 crates/athena-core/src/lib.rs                     |   3 +
 crates/athena-core/src/mechanism_explanation.rs   | 100 ++++++++++
 crates/athena-core/tests/mechanism_explanation.rs | 226 ++++++++++++++++++++++
 docs/decisions.md                                 |  21 ++
 docs/spec.toml                                    |   4 +-
 xtask/shard-timings.txt                           |   1 +
 7 files changed, 354 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-044531-ncyj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-044531-ncyj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43379903767 for 0b7c49496a6cc65f9618df13dde37c13e8eb1cc5)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43379903767 for 0b7c49496a6cc65f9618df13dde37c13e8eb1cc5)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43379903767 for 0b7c49496a6cc65f9618df13dde37c13e8eb1cc5)
- ci-result: `not a check` passed (Check results for 0b7c49496a6cc65f9618df13dde37c13e8eb1cc5 come from the CI run https://github.com/doodla/athena/actions/runs/43379903767 (688s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace`. All pass, or the recorded Athena check on the final tree shows them passing (D115). Other tests that read docs/spec.toml, such as crates/athena-gate/tests/bootstrap_requirement.rs, still pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test mechanism_explanation spec_entry_has_a_real_mechanism`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/lib.rs, crates/athena-core/src/mechanism_explanation.rs, docs/decisions.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling cfg-if v1.0.5
   Compiling zmij v1.0.23
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling thiserror v2.0.21
   Compiling winnow v1.0.4
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling winnow v0.7.15
   Compiling itoa v1.0.18
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling linux-raw-sys v0.12.1
   Compiling jiff v0.2.38
   Compiling ref-cast v1.0.27
   Compiling bitflags v2.13.2
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
error[E0432]: unresolved import `athena_core::mechanism_explanation`
  --> crates/athena-core/tests/mechanism_explanation.rs:10:18
   |
10 | use athena_core::mechanism_explanation::{Evidence, EvidenceKind, Explanation, Gap, check};
   |                  ^^^^^^^^^^^^^^^^^^^^^ could not find `mechanism_explanation` in `athena_core`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-core` (test "mechanism_explanation") due to 1 previous error
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3, 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test mechanism_explanation`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/lib.rs, crates/athena-core/src/mechanism_explanation.rs, docs/decisions.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `athena_core::mechanism_explanation`
  --> crates/athena-core/tests/mechanism_explanation.rs:10:18
   |
10 | use athena_core::mechanism_explanation::{Evidence, EvidenceKind, Explanation, Gap, check};
   |                  ^^^^^^^^^^^^^^^^^^^^^ could not find `mechanism_explanation` in `athena_core`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-core` (test "mechanism_explanation") due to 1 previous error
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
