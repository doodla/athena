# Job 20261030-013409-rwgy: verify "Give OLY-B-O1-01 a mechanism: bootstrap-range gate, spec entry and test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-O1-01 a mechanism
> 
> Requirement OLY-B-O1-01 in docs/spec.toml is active and has no mechanism.
> Requirement: The v0 walking skeleton was built by the automation engineer and was the only work done outside Athena, recorded as such; v0 stays small.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-O1-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-O1-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Point OLY-B-O1-01 in docs/spec.toml at the existing history-invariant bootstrap range (`athena_gate::invariant::bootstrap_range`, rule M6), which accepts `Athena-Job: bootstrap` commits only until records/jobs/bootstrap/outcome.json closes the range. Add a test that checks the spec entry and exercises that gate, with a `status: not shown` measure naming the test.

Assumptions:
- Goal and audience: the spec gap list (shown by `athena status`) and the verifier, who need OLY-B-O1-01 to name a real mechanism, an owner and a measure.
- Code survey: read crates/athena-gate/src/invariant.rs (BOOTSTRAP_OUTCOME, bootstrap_range, rule M6, test bootstrap_commits_pass_while_the_range_is_open_and_not_after), crates/athena-data/src/id.rs (JobId::Bootstrap reserved, dir records/jobs/bootstrap), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, no 'bootstrap' label) and crates/athena-core/tests/model_roster.rs (precedent: spec_entry_has_a_real_mechanism).
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. No refusal rule is added or tightened; the gate already exists, so no refusal-rate criterion applies.
- Mechanism is the existing bootstrap-range gate; 'v0 stays small' is not mechanically enforced and the measure says 'status: not shown' and names the test; this is stated in the measure.
- Existing code: crates/athena-gate/src/invariant.rs, crates/athena-data/src/id.rs, docs/records.md (Closing the bootstrap), docs/design/requirements-map.md (OM-3); the new test reuses model_roster.rs's spec-entry parsing approach.
- Consumers: none found for docs/spec.toml other than xtask spec tests and status.rs measure labels; no governance file (athena.toml, defaults.toml, workflows) changes.

## Acceptance criteria

1. [content] In docs/spec.toml the OLY-B-O1-01 entry no longer has mechanism = "none"; it names athena_gate::invariant::bootstrap_range (rule M6), keeps owner = "Athena", and its measure ends with `status: not shown` and names the test that shows it.
2. [behavior] A new test file under crates/athena-gate/tests/ reads docs/spec.toml, fails if the OLY-B-O1-01 entry has mechanism "none", lacks the mechanism name, owner or the `status: not shown` ending; run `cargo test -p athena-gate`. It fails on the base tree and passes with the change.
3. [behavior] The same test (or the existing invariant test) shows the gate accepts an `Athena-Job: bootstrap` commit while the range is open and refuses one after records/jobs/bootstrap/outcome.json closes it; run `cargo test -p athena-gate bootstrap`.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root, including the xtask spec test that checks measure markers.
5. [content] CHANGELOG.md gets an entry if the repository convention requires one for spec changes; no other path (athena.toml, defaults.toml, workflows, records/) changes.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-gate/tests/bootstrap_requirement.rs (create)
- crates/athena-gate/src/invariant.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/Cargo.toml (modify)

## Out of scope

- Changing the gate's behavior or the bootstrap range rules
- Adding a new `athena status` label
- Editing records/ or owner files
- Enforcing 'v0 stays small' by a size limit

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the gate test and check the spec entry text.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Requirement points at an enforced gate; the unenforced 'stays small' part is marked not shown.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: The test reads the real docs/spec.toml and runs `check_entry` on it. A second command, `cargo test -p athena-core oly_b_o1_01`, runs the real hestia gap rule on the same entry. `git diff docs/spec.toml` shows the edited lines. Run: `cargo test -p athena-gate --test bootstrap_requirement spec_entry_names_the_bootstrap_range_gate`. Observes: The OLY-B-O1-01 entry has no mechanism "none". Its mechanism contains `athena_gate::invariant::bootstrap_range`. Its owner is "Athena". Its one-line measure names `cargo test -p athena-gate bootstrap`, the test file and the hestia test, and ends `status: not shown`. The statement and source lines are unchanged. `measure_problem` returns None and `spec_gaps` omits the id.
  Fails without the behavior: On the base tree the entry has mechanism "none" (docs/spec.toml:2164) and measure 'The spec gap list; status: requirement', so `check_entry` returns Err and the hestia test's assert on the mechanism fails.
  Test seam: Entry points: the new pure `check_entry(&str)` in crates/athena-gate/tests/bootstrap_requirement.rs, fed docs/spec.toml via `Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")` (the model_roster.rs:11-16 approach), and the production `athena_core::hestia::{parse_spec, measure_problem, spec_gaps}`, fed `include_str!("../../../docs/spec.toml")`. There is no double. `check_entry` and the hestia test are added by plan steps 2 and 3. The spec file and the hestia functions exist (read).
- Criterion 2: Run the new test file and `cargo test -p athena-core oly_b_o1_01`. Three tests in the gate file: `spec_entry_names_the_bootstrap_range_gate` (real entry gives Ok), `base_entry_is_rejected` (literal base entry gives Err), `bootstrap_range_is_open_then_closed_by_the_outcome_record`. The hestia test asserts that the real entry is not a gap and that the same entry with mechanism "none" is exactly one `no_mechanism` gap. Run: `cargo test -p athena-gate --test bootstrap_requirement`. Observes: All pass with the change. `base_entry_is_rejected` passing shows the checker rejects the base text, so `spec_entry_names_the_bootstrap_range_gate` fails on the base docs/spec.toml. The hestia test fails on the base tree because the real entry is a gap there. The gate test target does not exist on the base tree, so the run fails there with 'no test target'.
  Fails without the behavior: With the base docs/spec.toml, `spec_entry_names_the_bootstrap_range_gate` fails on mechanism "none" and the hestia test fails on its gap assert. If `check_entry` accepted the base entry, `base_entry_is_rejected` would fail. If the gap rule did not flag mechanism "none", the hestia test's second half would fail.
  Test seam: Entry points: `check_entry` (gate test) with a fixture string compiled into the test as the base entry wrapped in `[[requirement]]`, and the production `spec_gaps` (hestia.rs:1291). The fixture is data. The logic under test is the real spec file plus the checker, and the real gap rule, not a double. The range test calls the real `athena_gate::invariant::bootstrap_range(&Git, "HEAD")` (invariant.rs:68) on a `tempfile::tempdir` repo made with `athena_git::Git::isolated`. Both crates are normal dependencies (Cargo.toml:11-19). Precondition: a git binary. The range test is not counted as proof of failure without the change. Plan steps 2 and 3 add the tests. `bootstrap_range` and `Git::isolated` exist (read).
- Criterion 3: The `bootstrap` filter runs the existing in-source `bootstrap_commits_pass_while_the_range_is_open_and_not_after` (invariant.rs:1102-1122) and the new `bootstrap_range_is_open_then_closed_by_the_outcome_record`. Both are in the filter, because the new test name contains `bootstrap`. Criterion 3 is existing evidence for the refusal, plus a new direct range check. Run: `cargo test -p athena-gate bootstrap`. Observes: Bootstrap commits pass while the range is open and at the closing commit. A late `Athena-Job: bootstrap` commit after `through_commit` gets exactly one violation (assert at invariant.rs:1121). The new test sees `Open` and then `ClosedThrough(<commit id>)`.
  Fails without the behavior: If rule M6 accepted bootstrap commits after the outcome record, the `assert_eq!(judge(&git, "HEAD").len(), 1)` at invariant.rs:1121 would fail. If `bootstrap_range` ignored the outcome file, the new range test would fail. Neither fails on the base tree, because the gate is unchanged.
  Test seam: Entry points: `invariant::check` (invariant.rs:102) via the in-source `judge` helper (invariant.rs:885) and `owner_repo()` (invariant.rs:892), and `bootstrap_range` directly. Fixtures are real temporary git repos made inside the tests. All exist (read). The only new seam is the range test in step 2.
- Criterion 4: The verifier runs `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` as separate commands. Run: `cargo test --locked`. Observes: No fmt diff and no clippy warnings. Every test passes. That includes xtask `every_active_requirement_has_a_mechanism_an_owner_and_a_measure` (spec.rs:627), which uses the `status: <label>` check at spec.rs:194; `checked_in_timings_cover_every_workspace_test_unit` (ci_shards.rs:411); hestia `the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed` (hestia.rs:4286); and the xtask policy scans over the new text.
  Fails without the behavior: A measure without a trailing `status: <label>` fails the xtask spec test. A missing shard-timings line fails ci_shards.rs:411.
  Test seam: The workspace cargo commands, the xtask spec parse, the shard oracle in xtask/tests and the hestia tests. All exist (read).
- Criterion 5: Read the diff stat and the new CHANGELOG.md bullet. CHANGELOG.md is a 435 KB file in the Keep-a-Changelog style, and each spec mechanism has a bullet under [Unreleased] / Added (the OLY-B-SET-02 entry, CHANGELOG.md:26-32). Run: `git diff --stat`. Observes: Only docs/spec.toml, crates/athena-gate/tests/bootstrap_requirement.rs, crates/athena-gate/src/invariant.rs (doc comment only), crates/athena-core/src/hestia.rs (test only), CHANGELOG.md and xtask/shard-timings.txt change. athena.toml, defaults.toml, workflows, records/ and crates/athena-gate/Cargo.toml are untouched. The bullet is first under Added and labels the 5.00 s timing an estimate.
  Fails without the behavior: Content check; there is no behavior to fail.
  Test seam: None. This is a content check of the diff.

Planned files:

- docs/spec.toml (modify)
- crates/athena-gate/tests/bootstrap_requirement.rs (create)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f8c855555d13 → candidate 09bd35dafce3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                      |   6 ++
 crates/athena-core/src/hestia.rs                  |  30 ++++++
 crates/athena-gate/src/invariant.rs               |   2 +-
 crates/athena-gate/tests/bootstrap_requirement.rs | 118 ++++++++++++++++++++++
 docs/spec.toml                                    |   4 +-
 xtask/shard-timings.txt                           |   1 +
 6 files changed, 158 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-013409-rwgy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013409-rwgy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43165754167 for 09bd35dafce31eb3db6305df4abc217d86ed1a7f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43165754167 for 09bd35dafce31eb3db6305df4abc217d86ed1a7f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43165754167 for 09bd35dafce31eb3db6305df4abc217d86ed1a7f)
- ci-result: `not a check` passed (Check results for 09bd35dafce31eb3db6305df4abc217d86ed1a7f come from the CI run https://github.com/doodla/athena/actions/runs/43165754167 (631s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root, including the xtask spec test that checks measure markers.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-gate --test bootstrap_requirement spec_entry_names_the_bootstrap_range_gate`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-gate/src/invariant.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling memchr v2.8.3
   Compiling getrandom v0.3.4
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling thiserror v2.0.21
   Compiling zmij v1.0.23
   Compiling digest v0.10.7
   Compiling cpufeatures v0.2.17
   Compiling winnow v1.0.4
   Compiling rustix v1.1.5
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling winnow v0.7.15
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling jiff-core v0.1.1
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling itoa v1.0.18
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling regex-automata v0.4.18
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling regex v1.13.1
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.30s
     Running tests/bootstrap_requirement.rs (<job>/target/verify/debug/deps/bootstrap_requirement-93a6853cbe33cf19)

running 1 test
test spec_entry_names_the_bootstrap_range_gate ... FAILED

failures:

---- spec_entry_names_the_bootstrap_range_gate stdout ----

thread 'spec_entry_names_the_bootstrap_range_gate' (28643) panicked at crates/athena-gate/tests/bootstrap_requirement.rs:72:47:
called `Result::unwrap()` on an `Err` value: "mechanism is blank or none: \"none\""
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    spec_entry_names_the_bootstrap_range_gate

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p athena-gate --test bootstrap_requirement`
error: 1 target failed:
    `-p athena-gate --test bootstrap_requirement`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-gate --test bootstrap_requirement`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-gate/src/invariant.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/bootstrap_requirement.rs (<job>/target/verify/debug/deps/bootstrap_requirement-93a6853cbe33cf19)

running 3 tests
test base_entry_is_rejected ... ok
test spec_entry_names_the_bootstrap_range_gate ... FAILED
test bootstrap_range_is_open_then_closed_by_the_outcome_record ... ok

failures:

---- spec_entry_names_the_bootstrap_range_gate stdout ----

thread 'spec_entry_names_the_bootstrap_range_gate' (28672) panicked at crates/athena-gate/tests/bootstrap_requirement.rs:72:47:
called `Result::unwrap()` on an `Err` value: "mechanism is blank or none: \"none\""
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    spec_entry_names_the_bootstrap_range_gate

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

error: test failed, to rerun pass `-p athena-gate --test bootstrap_requirement`
error: 1 target failed:
    `-p athena-gate --test bootstrap_requirement`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-gate bootstrap`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-gate/src/invariant.rs, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.10s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_gate-d4b52202855a4113)

running 2 tests
test invariant::tests::bootstrap_commits_pass_while_the_range_is_open_and_not_after ... ok
test history::tests::a_clean_bootstrap_history_passes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 139 filtered out; finished in 0.80s

     Running tests/bootstrap_requirement.rs (<job>/target/verify/debug/deps/bootstrap_requirement-93a6853cbe33cf19)

running 2 tests
test spec_entry_names_the_bootstrap_range_gate ... FAILED
test bootstrap_range_is_open_then_closed_by_the_outcome_record ... ok

failures:

---- spec_entry_names_the_bootstrap_range_gate stdout ----

thread 'spec_entry_names_the_bootstrap_range_gate' (29284) panicked at crates/athena-gate/tests/bootstrap_requirement.rs:72:47:
called `Result::unwrap()` on an `Err` value: "mechanism is blank or none: \"none\""
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    spec_entry_names_the_bootstrap_range_gate

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p athena-gate --test bootstrap_requirement`
error: 1 target failed:
    `-p athena-gate --test bootstrap_requirement`
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
