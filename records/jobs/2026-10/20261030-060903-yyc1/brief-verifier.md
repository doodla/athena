# Job 20261030-060903-yyc1: verify "Give OLY-B-S7-01 a mechanism: check a study's comparison"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-S7-01 a mechanism
> 
> Requirement OLY-B-S7-01 in docs/spec.toml is active and has no mechanism.
> Requirement: The study maps all three systems onto shared concepts, compares them mechanism by mechanism, and closes with a short essay on building orchestration from prompts.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-S7-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-S7-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a pure check in athena-core, modelled on mechanism_explanation.rs, that flags a comparison study which lacks shared concepts mapped across all three systems, a per-mechanism comparison, or a closing essay on building orchestration from prompts. Record it as the mechanism in docs/spec.toml with a measure ending `status: not shown`, since no real three-system study exists yet. Audience: the owner and verifier judging the spec gap list.

Assumptions:
- Code survey: I read crates/athena-core/src/mechanism_explanation.rs, crates/athena-core/tests/mechanism_explanation.rs and crates/athena-core/src/lib.rs (line 131 `pub mod mechanism_explanation;`). The entry point is a pure `check` function that reads and runs nothing. The test parses the real docs/spec.toml with `hestia::parse_spec` and checks the entry's owner and measure. I also skimmed crates/athena-cap-study/src/rules.rs, which validates a study's claims. It has no rule about comparison or essay content.
- The new code adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not involved.
- Existing code: the job extends the S4-01 pattern in crates/athena-core/src/mechanism_explanation.rs and crates/athena-core/tests/mechanism_explanation.rs, and reuses hestia::parse_spec and SpecRequirement. It creates crates/athena-core/src/comparison.rs and crates/athena-core/tests/comparison.rs.
- The check takes a comparison record as input: a list of shared concepts, each mapped for every one of three named systems; a list of mechanism comparisons, each covering all three systems; and an essay with its text and a prompt-orchestration topic. It returns a Gap for each of these: a concept missing a system, no concepts, a mechanism comparison missing a system, no comparisons, a missing or blank essay, and an essay that is not short. 'Short' is a word cap the producer picks and documents.
- Measure: the test file checks complete and defective comparisons plus this spec entry, and names `cargo test -p athena-core --test comparison`. The measure ends `status: not shown`, because the check has not yet run on a real three-system study. S3-01, S3-02 and S4-01 use the same wording.
- Governance: neither athena.toml, defaults.toml nor any workflow file needs to change. CHANGELOG.md gets the default entry.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-S7-01 has a mechanism naming crates/athena-core/src/comparison.rs and its `check` function (not "none"), keeps owner "Athena", and has a measure that names `cargo test -p athena-core --test comparison` and ends with `status: not shown`.
2. [content] crates/athena-core/src/comparison.rs exists, is declared in crates/athena-core/src/lib.rs, and has a doc comment that lists its limits. Its `check` function reports a gap for each of: a concept not mapped for all three systems, no concepts, a mechanism comparison missing a system, no comparisons, a missing or blank essay, and an over-long essay.
3. [behavior] Run `cargo test -p athena-core --test comparison`. A complete comparison yields no gaps. Each defective input (a missing system in a concept mapping, a missing system in a mechanism comparison, an absent essay, a blank essay, an over-long essay) yields exactly its expected gap. The test also parses the real docs/spec.toml and fails when the S7-01 mechanism is "none" or the measure lacks the command or the `status: not shown` ending, so it fails without the mechanism and passes with it.
4. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test` as separate commands, or cite the Athena-recorded checks on the final tree. All pass.
5. [content] CHANGELOG.md has a new entry for this change, and the diff touches no records/ path and no existing entry of docs/decisions.md.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/comparison.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/comparison.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Writing the real three-system comparison study or the essay itself
- Running any of the three systems
- Changes to other spec requirements, athena.toml, defaults.toml or workflow files
- Wiring `check` into the athena CLI or the study capability's rule set

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new module gets an integration test, and the criteria require fmt, clippy and test to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test drives `check` with defective inputs and checks the real spec entry.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a pure check function with tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module, one test file and one spec entry, with no wiring beyond lib.rs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: The test reads the real docs/spec.toml with `parse_spec` and finds OLY-B-S7-01. It asserts: the mechanism is not `none` and contains `crates/athena-core/src/comparison.rs`, `crates/athena-core/tests/comparison.rs` and `check`; owner == "Athena"; `measure_problem` is None; the measure contains `cargo test -p athena-core --test comparison` and, once trimmed, ends with `status: not shown`; `spec_gaps` no longer lists OLY-B-S7-01. Run: `cargo test -p athena-core --test comparison spec_entry_has_a_real_mechanism`. Observes: The test passes on the edited file. On the current entry (spec.toml:3037-3039: mechanism `none`, measure `The spec gap list; status: requirement`) it fails.
  Fails without the behavior: With mechanism `none`, the old measure label `requirement`, a measure missing the command, or an owner other than Athena, the assertions fail. The negative tests `spec_check_refuses_a_none_mechanism`, `spec_check_refuses_a_wrong_status_label`, `spec_check_refuses_a_measure_without_the_command` and `spec_check_refuses_a_wrong_owner` show each helper rejects a bad entry.
  Test seam: Production entry points: `athena_core::hestia::parse_spec`, `measure_problem` and `spec_gaps` (hestia.rs:1219, 1291), run over the real docs/spec.toml via `CARGO_MANIFEST_DIR/../..`. No double. The `repo_file` helper pattern exists at crates/athena-core/tests/mechanism_explanation.rs:16-22; the new test file copies it (plan step 3). The spec edit is plan step 4.
- Criterion 2: A test calls `check` on in-memory records, one per defect, and asserts the exact gap list. The defects are: a concept lacking one system (each of the three systems in turn), `NoConcepts`, a mechanism comparison lacking one system, `NoComparisons`, `NoEssay` (absent), `NoEssay` (whitespace-only), `EssayTooLong` at 801 words, and `EssayOffTopic`. A test also reads crates/athena-core/src/comparison.rs with `repo_file` and asserts its doc comment holds a `Limits:` section. `cargo build -p athena-core` shows `pub mod comparison;` is declared, because tests/comparison.rs imports `athena_core::comparison`. Run: `cargo test -p athena-core --test comparison`. Observes: Each defective input gives exactly its own `Gap` and no other. An entry for an unknown system name, or one with blank text, does not satisfy a required system. The 800-word essay gives no gap. Several defects together give gaps in the fixed order.
  Fails without the behavior: Without comparison.rs or the lib.rs declaration the test file does not compile. A `check` that skips one branch leaves that defect's test with an empty or different gap list.
  Test seam: Production entry point: `athena_core::comparison::check(&Comparison)`. Fixtures are in-memory `Comparison` values built by helper functions in the test. `check` is pure, so nothing is injected. The seam is added by plan step 1 and declared by step 2.
- Criterion 3: Same test file. `complete_comparison_has_no_gaps` builds a record with all three systems in every concept and every mechanism comparison, and a short essay naming both terms, and asserts an empty Vec. One test per defect asserts exactly its gap. `essay_at_the_cap_passes_and_one_over_fails` checks the 800/801 boundary. `blank_entry_text_does_not_count` and `unknown_system_name_does_not_count` check the guards. `spec_entry_has_a_real_mechanism` and the four negative spec tests cover the real spec. Run: `cargo test -p athena-core --test comparison`. Observes: All tests pass with the module and the spec entry in place. The test target is `comparison` in athena-core.
  Fails without the behavior: Reverting the spec edit makes `spec_entry_has_a_real_mechanism` fail (mechanism `none`; measure lacks the command and ending). Deleting a `check` branch makes the matching defect test fail with a gap list that is empty or differs from the expected one. Deleting comparison.rs makes the file fail to compile.
  Test seam: Production entry points: `comparison::check`, and `hestia::parse_spec`, `measure_problem` and `spec_gaps`. Fixtures are in-memory structs and the real docs/spec.toml; there is no double. The test file is created in plan step 3, comparison.rs in step 1.
- Criterion 4: Run three separate commands on the final tree, as Athena's checks run them: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Or cite the Athena-recorded fmt, clippy and test checks on the final tree. The full `cargo test --locked` includes `checked_in_timings_cover_every_workspace_test_unit` (xtask/tests/ci_shards.rs:411), `the_real_repo_has_no_mirror_drift` (crates/athena-cli/tests/mirrors.rs:22), `every_active_requirement_has_a_mechanism_an_owner_and_a_measure` (xtask/tests/spec.rs:627) and the boundary scan in xtask/tests/policy.rs. Run: `cargo test --locked`. Observes: All three commands exit 0. The new public types carry `#[derive(Debug)]`, so clippy under `-D warnings` raises no `missing_debug_implementations`. The timings file lists `athena-core --test comparison`, so the ci_shards and mirrors checks agree with the workspace's test targets.
  Fails without the behavior: Without the `athena-core --test comparison 5.00` line, `checked_in_timings_cover_every_workspace_test_unit` fails and the mirrors test reports drift, so `cargo test --locked` fails. Without `pub mod comparison;` in lib.rs, tests/comparison.rs does not compile, so both `cargo test --locked` and `cargo clippy --all-targets` fail. A public type without Debug fails clippy under `-D warnings`. Unformatted code fails `cargo fmt --all --check`.
  Test seam: Production entry points: the `cargo fmt`, `cargo clippy` and `cargo test` commands over the whole workspace, the same commands Athena records as its fmt, clippy and test checks. The fixture is the real final working tree, read by the tests listed above with no double. The seam exists today (I read ci_shards.rs:410-421 and mirrors.rs:58-59, 206-234). The timings line is added by plan step 6 and the Debug derives by step 1.
- Criterion 5: Run `git status` for the untracked new files, `git diff --stat` for the changed paths, and `git diff -- CHANGELOG.md docs/decisions.md` for the changelog and decision log. Look for any path under records/. Run: `git diff --stat`. Observes: The changed paths are only docs/spec.toml, crates/athena-core/src/comparison.rs (new), crates/athena-core/src/lib.rs, crates/athena-core/tests/comparison.rs (new), CHANGELOG.md and xtask/shard-timings.txt. CHANGELOG.md gains one new bullet under [Unreleased] > Added. docs/decisions.md and records/ are unchanged.
  Fails without the behavior: Without the CHANGELOG bullet the diff for CHANGELOG.md is empty and the criterion fails. A decision-log edit or a records/ path in the diff would also fail it.
  Test seam: Production entry point: git's own diff of the working tree against the base commit. The fixture is the real tree; nothing is injected. The seam exists.

Planned files:

- crates/athena-core/src/comparison.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/comparison.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ba420e848f56 → candidate dfcba458b0c6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   1 +
 crates/athena-core/src/comparison.rs   | 158 +++++++++++++++++
 crates/athena-core/src/lib.rs          |   4 +
 crates/athena-core/tests/comparison.rs | 309 +++++++++++++++++++++++++++++++++
 docs/spec.toml                         |   4 +-
 xtask/shard-timings.txt                |   1 +
 6 files changed, 475 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-060903-yyc1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060903-yyc1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43536856444 for dfcba458b0c6eebff6dee2e473695e47ad8559ea)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43536856444 for dfcba458b0c6eebff6dee2e473695e47ad8559ea)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43536856444 for dfcba458b0c6eebff6dee2e473695e47ad8559ea)
- ci-result: `not a check` passed (Check results for dfcba458b0c6eebff6dee2e473695e47ad8559ea come from the CI run https://github.com/doodla/athena/actions/runs/43536856444 (941s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy` and `cargo test` as separate commands, or cite the Athena-recorded checks on the final tree. All pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test comparison spec_entry_has_a_real_mechanism`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/comparison.rs, crates/athena-core/src/lib.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling zmij v1.0.23
   Compiling getrandom v0.3.4
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling serde v1.0.229
   Compiling winnow v1.0.4
   Compiling serde_json v1.0.151
   Compiling thiserror v2.0.21
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling itoa v1.0.18
   Compiling winnow v0.7.15
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
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
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `athena_core::comparison`
  --> crates/athena-core/tests/comparison.rs:10:18
   |
10 | use athena_core::comparison::{
   |                  ^^^^^^^^^^ could not find `comparison` in `athena_core`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-core` (test "comparison") due to 1 previous error
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test comparison`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/comparison.rs, crates/athena-core/src/lib.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `athena_core::comparison`
  --> crates/athena-core/tests/comparison.rs:10:18
   |
10 | use athena_core::comparison::{
   |                  ^^^^^^^^^^ could not find `comparison` in `athena_core`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-core` (test "comparison") due to 1 previous error
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/comparison.rs, crates/athena-core/src/lib.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling utf8parse v0.2.2
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-parse v1.0.0
   Compiling anstyle v1.0.14
   Compiling colorchoice v1.0.5
   Compiling clap_lex v1.1.1
   Compiling strsim v0.11.1
   Compiling anstream v1.0.0
   Compiling heck v0.5.0
   Compiling clap_builder v4.6.7
   Compiling crc32fast v1.5.2
   Compiling adler2 v2.0.1
   Compiling simd-adler32 v0.3.10
   Compiling miniz_oxide v0.9.1
   Compiling xattr v1.6.1
   Compiling filetime v0.2.29
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling clap_derive v4.6.7
   Compiling clap v4.6.7
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0432]: unresolved import `athena_core::comparison`
  --> crates/athena-core/tests/comparison.rs:10:18
   |
10 | use athena_core::comparison::{
   |                  ^^^^^^^^^^ could not find `comparison` in `athena_core`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-core` (test "comparison") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
