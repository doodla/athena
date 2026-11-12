# Job 20261030-013717-3r2v: verify "Give OLY-B-STD-03 a buy-first dependency check and spec mechanism"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-STD-03 a mechanism
> 
> Requirement OLY-B-STD-03 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena adopts good, maintained options where they exist and builds only for needs they do not meet, naming those needs.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-STD-03
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-STD-03 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a test that every external crate in the workspace Cargo.toml is named in the buy-before-build decision log (D36) and that a stray dependency is refused. Then record that check as OLY-B-STD-03's mechanism and a measure ending in `status: not shown` in docs/spec.toml. The audience is the Athena owner and the verifier, who check the spec entry and run the test.

Assumptions:
- Code survey: I read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, which lists the allowed `status: <label>` values), crates/athena-core/tests/model_roster.rs (the pattern for a spec-entry mechanism test, with repo_file, Config::parse and a mutated-copy refusal) and docs/spec.toml (OLY-B-STD-01 to 05). The measure uses `status: not shown` plus the test command, as OLY-B-STD-04 and 05 do. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly on Cargo.toml text and D36 text, not through a stand-in.
- Existing code: the change reuses and extends docs/decisions.md D36 (Buy before build), the model_roster.rs test pattern, and the docs/spec.toml mechanism/measure format. It creates only the new test file.
- Mechanism choice: the existing prose-only process rule is docs/enforcement.md line 60 and D36. The smallest mechanism is a test tying the workspace dependency list to the D36 record, so a new crate cannot arrive without a recorded call. This enforces 'naming those needs' for dependencies only; it cannot judge whether a maintained option exists, and that stays a verifier judgement.
- Term ladder: none used. The old measure 'The spec gap list; status: requirement' is replaced because the requirement is no longer a gap.
- The test can parse Cargo.toml with the toml crate if athena-core already depends on it, otherwise with a simple line scan, so athena-core/Cargo.toml probably stays unchanged. Consumers: none found for any config change.

## Acceptance criteria

1. [content] In docs/spec.toml, the OLY-B-STD-03 entry no longer has `mechanism = "none"`. It names the buy-first check (the test file and the D36 decision log), keeps `owner = "Athena"`, and its measure ends with `status: not shown` and names the test command, for example `cargo test -p athena-core --test buy_first; status: not shown`.
2. [behavior] Run `cargo test -p athena-core --test buy_first`. It passes on the change. It reads the workspace Cargo.toml and docs/decisions.md and checks that every external `[workspace.dependencies]` crate is named in the buy-before-build record (D36). Another test in the same file adds a fake dependency to a copy of Cargo.toml and shows the check refusing it, so the check fails without the mechanism.
3. [behavior] A spec test in the same file fails if OLY-B-STD-03 still has `mechanism = "none"`, lacks owner `Athena`, or has a measure that does not end with `status: not shown`. It follows the pattern of spec_entry_has_a_real_mechanism in crates/athena-core/tests/model_roster.rs.
4. [content] Crates currently in workspace Cargo.toml but missing from D36 (proc-macro2, syn, toml_edit, any others the test finds) get their call recorded in docs/decisions.md with the unmet need named. No dependency is added or removed in Cargo.toml (the governance file); Cargo.toml stays unchanged.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass at the workspace root. The new test imports only existing crates, and docs/spec.toml still passes the xtask spec test (`cargo test -p xtask --test spec`).
6. [content] The change adds no refusal rule to the job pipeline. It only adds a test, so replay refusal rates do not change (expected 0% of newly refused readings).

## Expected deliverables

- crates/athena-core/tests/buy_first.rs (create)
- docs/spec.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/Cargo.toml (modify)
- xtask/tests/buy_first.rs (create)

## Out of scope

- Judging whether a better maintained option exists for each dependency
- Auditing dependencies of crates other than the workspace root
- Changes to Cargo.toml, athena.toml or defaults.toml
- Changes to other requirements in docs/spec.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require a new test that fails without the mechanism, plus cargo test and clippy.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the check and show it refusing a stray dependency, rather than checking that files exist.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: The change uses existing toml parsing and builds no new tooling; the call is recorded in D36.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule gets a test as its mechanism, and the spec entry names it.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the spec test, which parses the real docs/spec.toml with the production parser and checks the OLY-B-STD-03 entry. The verifier can also open the entry in docs/spec.toml. Run: `cargo test -p athena-core --test buy_first spec_entry`. Observes: The entry no longer says `none`. Its mechanism names `buy_first` and `D36`, owner is `Athena`, `spec_gaps` does not list OLY-B-STD-03, `measure_problem` is None, and `measure_label_problem` is None, so the measure ends `status: not shown` and names `cargo test -p athena-core --test buy_first`.
  Fails without the behavior: With the old entry (`mechanism = "none"`, measure `The spec gap list; status: requirement`), `spec_gaps` lists the id, the mechanism assertions fail, and `measure_label_problem` is Some. `measure_problem` alone would pass on the old measure, because `status: requirement` is a valid label (status.rs line 321), so it is not what catches this.
  Test seam: Production entry points: `athena_core::hestia::parse_spec`, `spec_gaps` and `measure_problem` (hestia.rs 1172, 1291, 1219; exist, `pub mod hestia` at lib.rs line 106). New pure helpers `owner_problem` and `measure_label_problem` in buy_first.rs, added by the step that creates the file. Fixture: the real docs/spec.toml read through a `repo_file` helper copied from model_roster.rs lines 11-16 (exists). Nothing is injected, and no logic sits in a double.
- Criterion 2: Run the whole test file. The real-file test calls `unrecorded` on the real Cargo.toml and docs/decisions.md and asserts it is empty. The negatives pass in-memory mutated copies to the same function: (a) `left-pad-rs = "1"` appended under `[workspace.dependencies]` is returned; (b) `proptest = "1"`, named only under D36's Alternatives, is returned; (c) a fixed manifest with the real D36 section alone returns exactly proc-macro2, syn and toml_edit and not `toml`; (d) an entry headed 'Buy before build' with no Decision bullet records nothing; (e) athena-* path entries, both `{ path = ... }` inline and table form, are not external. Run: `cargo test -p athena-core --test buy_first`. Observes: The real-file test passes, and each of the stray crate, the rejected-alternative crate, the D36-only gaps and the heading-only entry is refused. This also shows the check reads inline-table and plain-version entries the same way.
  Fails without the behavior: Without the check, `unrecorded` does not exist and the file does not compile. Without the new decision, the real-file test fails naming proc-macro2, syn and toml_edit. With whole-entry or substring matching, the proptest and D36-only tests fail.
  Test seam: Production entry point: `unrecorded(cargo_toml, decisions)` in crates/athena-core/tests/buy_first.rs, with `external_crates` and `adopted_record` behind it, all added by the file-creating step. It parses with `toml_edit::DocumentMut`, an existing athena-core dependency. Fixture: the real Cargo.toml and docs/decisions.md read through `repo_file`, and in-memory mutated copies passed as function arguments. The logic under test is the function, not the fixture.
- Criterion 3: Run the spec tests. On the real entry they pass. Then negatives edit a cloned parsed `SpecRequirement`: (a) mechanism `none`, so `spec_gaps` reports it; (b) measure ending `status: requirement`, so `measure_label_problem` returns Some while production `measure_problem` returns None (the test asserts both, to show which function does the refusing); (c) measure ending `status: bogus`, so production `measure_problem` returns Some; (d) owner set to another role, so `owner_problem` returns Some; (e) measure lacking the test command, so `measure_label_problem` returns Some. Run: `cargo test -p athena-core --test buy_first spec_`. Observes: The real entry passes. Each mutated copy is refused by the function named for it: mechanism none, owner not Athena, measure not ending `status: not shown`, and an invalid label.
  Fails without the behavior: Without the helpers, a wrong owner or a `status: requirement` measure passes silently, because `spec_gaps` ignores the owner and `measure_problem` accepts `status: requirement`. Without the mechanism text in docs/spec.toml, the real entry is flagged.
  Test seam: Production entry points `parse_spec`, `spec_gaps`, `measure_problem` (exist). New `owner_problem` and `measure_label_problem` in buy_first.rs (added by the test-writing step). Patterns: spec_entry_has_a_real_mechanism in model_roster.rs lines 50-68 and the OLY-B-RUN-02 test at hestia.rs lines 4367-4392 (exist). Fixture: the real docs/spec.toml parsed, with fields changed on a cloned `SpecRequirement` in the test.
- Criterion 4: Run the real-file test, then the log-structure tests, then `git diff`. The new decision's Decision bullet names `proc-macro2` and `syn` (unmet need: function-length and design-surface scans in code_health.rs and design_surface.rs, where brace counting breaks on strings and macros; the D132 note near decisions.md line 3356) and `toml_edit` (unmet need: rewriting configuration and proposals while keeping comments and layout, which `toml` does not do). Turned-down options (brace counting, a lint tool, the `toml` crate) sit under Alternatives. The producer reads the dependency use in the code before writing these needs. Run: `cargo test -p athena-core --test buy_first every_external_workspace_crate`. Observes: The real-file test passes. `git diff Cargo.toml` is empty and docs/decisions.md changes only by appended lines. `cargo test -p athena-cli --test backtest` and the mirror tests in `cargo test -p athena-core` also pass, showing the new wording parses as no supersede or amend link.
  Fails without the behavior: If the new decision is missing, or names a crate only under Alternatives, the real-file test fails naming that crate. If the entry uses 'Amends D36' with no [[link]], the athena-cli backtest real-log test fails.
  Test seam: Production entry point: `unrecorded` (added in the file-creating step). The append-only, heading and Provenance rules are enforced by crates/athena-core/src/mirrors.rs and its tests; the link rule by crates/athena-cli/tests/backtest.rs `the_real_log_has_no_condition_a_finding_against_the_confirmed_list`. All exist and run in `cargo test`.
- Criterion 5: Run the three Athena checks at the workspace root, then the three xtask tests that read files this change touches. The new test imports only std, `toml_edit` and `athena_core`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three checks pass. Then `cargo test -p xtask --test spec`, `cargo test -p xtask --test ci_shards` and `cargo test -p xtask --test policy` pass.
  Fails without the behavior: Without the timings line, `checked_in_timings_cover_every_workspace_test_unit` fails. Without the spec edit, the spec tests in buy_first.rs fail.
  Test seam: The existing check commands and xtask tests (exist). The timings line is added by the plan step for xtask/shard-timings.txt.
- Criterion 6: Read the diff stat. It touches only crates/athena-core/tests/buy_first.rs, docs/spec.toml, docs/decisions.md, CHANGELOG.md and xtask/shard-timings.txt. No parser, stem list or reading check is added to the pipeline; the key-parsing in the test applies to Cargo.toml and the decision log, not to readings. Run: `git diff --stat`. Observes: No change under crates/*/src and no pipeline rule added. The refusal_corpus case ids are unaffected, so no reading is newly refused (0%). The verifier may also run `cargo test -p athena-core --test refusal_corpus`; the planner did not run it and cites no result.

Planned files:

- crates/athena-core/tests/buy_first.rs (create)
- docs/spec.toml (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base acd449d74cc1 → candidate 6dcf7ffb4e33. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |   1 +
 crates/athena-core/tests/buy_first.rs | 232 ++++++++++++++++++++++++++++++++++
 docs/decisions.md                     |  31 +++++
 docs/spec.toml                        |   4 +-
 xtask/shard-timings.txt               |   1 +
 5 files changed, 267 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-013717-3r2v/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013717-3r2v/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43223370890 for 6dcf7ffb4e33c7dc1e92795d0e18745daae6cdf2)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43223370890 for 6dcf7ffb4e33c7dc1e92795d0e18745daae6cdf2)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43223370890 for 6dcf7ffb4e33c7dc1e92795d0e18745daae6cdf2)
- ci-result: `not a check` passed (Check results for 6dcf7ffb4e33c7dc1e92795d0e18745daae6cdf2 come from the CI run https://github.com/doodla/athena/actions/runs/43223370890 (795s): fmt success, clippy success, test success.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test buy_first spec_entry`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, docs/decisions.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling thiserror-impl v2.0.21
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling ref-cast v1.0.27
   Compiling linux-raw-sys v0.12.1
   Compiling jiff v0.2.38
   Compiling bitflags v2.13.2
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling anyhow v1.0.104
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 39.03s
     Running tests/buy_first.rs (<job>/target/verify/debug/deps/buy_first-d55d0629d2238394)

running 1 test
test spec_entry_has_a_real_mechanism ... FAILED

failures:

---- spec_entry_has_a_real_mechanism stdout ----

thread 'spec_entry_has_a_real_mechanism' (25172) panicked at crates/athena-core/tests/buy_first.rs:181:5:
assertion `left != right` failed
  left: "none"
 right: "none"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    spec_entry_has_a_real_mechanism

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.06s

error: test failed, to rerun pass `-p athena-core --test buy_first`
error: 1 target failed:
    `-p athena-core --test buy_first`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test buy_first`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, docs/decisions.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/buy_first.rs (<job>/target/verify/debug/deps/buy_first-d55d0629d2238394)

running 12 tests
test a_buy_first_heading_without_a_decision_bullet_records_nothing ... ok
test d36_alone_leaves_proc_macro2_syn_and_toml_edit_unrecorded ... ok
test path_crates_are_not_external ... ok
test a_crate_named_only_as_a_rejected_alternative_is_refused ... FAILED
test every_external_workspace_crate_is_recorded_in_the_buy_first_decisions ... FAILED
test a_stray_dependency_is_refused ... FAILED
test spec_check_refuses_a_measure_without_the_test_command ... ok
test spec_check_refuses_a_none_mechanism ... ok
test spec_check_refuses_a_wrong_owner ... ok
test spec_check_refuses_a_wrong_status_label ... ok
test spec_check_refuses_an_invalid_label ... ok
test spec_entry_has_a_real_mechanism ... FAILED

failures:

---- a_crate_named_only_as_a_rejected_alternative_is_refused stdout ----

thread 'a_crate_named_only_as_a_rejected_alternative_is_refused' (25201) panicked at crates/athena-core/tests/buy_first.rs:139:5:
assertion `left == right` failed
  left: ["proptest", "proc-macro2", "syn", "toml_edit"]
 right: ["proptest"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- every_external_workspace_crate_is_recorded_in_the_buy_first_decisions stdout ----

thread 'every_external_workspace_crate_is_recorded_in_the_buy_first_decisions' (25204) panicked at crates/athena-core/tests/buy_first.rs:106:5:
external crates with no Buy before build decision: ["proc-macro2", "syn", "toml_edit"]

---- a_stray_dependency_is_refused stdout ----

thread 'a_stray_dependency_is_refused' (25202) panicked at crates/athena-core/tests/buy_first.rs:122:5:
assertion failed: unrecorded(&cargo, &decisions).is_empty()

---- spec_entry_has_a_real_mechanism stdout ----

thread 'spec_entry_has_a_real_mechanism' (25211) panicked at crates/athena-core/tests/buy_first.rs:181:5:
assertion `left != right` failed
  left: "none"
 right: "none"


failures:
    a_crate_named_only_as_a_rejected_alternative_is_refused
    a_stray_dependency_is_refused
    every_external_workspace_crate_is_recorded_in_the_buy_first_decisions
    spec_entry_has_a_real_mechanism

test result: FAILED. 8 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

error: test failed, to rerun pass `-p athena-core --test buy_first`
error: 1 target failed:
    `-p athena-core --test buy_first`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test buy_first spec_`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, docs/decisions.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/buy_first.rs (<job>/target/verify/debug/deps/buy_first-d55d0629d2238394)

running 6 tests
test spec_check_refuses_a_wrong_owner ... ok
test spec_check_refuses_a_wrong_status_label ... ok
test spec_check_refuses_a_none_mechanism ... ok
test spec_check_refuses_a_measure_without_the_test_command ... ok
test spec_check_refuses_an_invalid_label ... ok
test spec_entry_has_a_real_mechanism ... FAILED

failures:

---- spec_entry_has_a_real_mechanism stdout ----

thread 'spec_entry_has_a_real_mechanism' (25282) panicked at crates/athena-core/tests/buy_first.rs:181:5:
assertion `left != right` failed
  left: "none"
 right: "none"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    spec_entry_has_a_real_mechanism

test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.09s

error: test failed, to rerun pass `-p athena-core --test buy_first`
error: 1 target failed:
    `-p athena-core --test buy_first`
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core --test buy_first every_external_workspace_crate`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
