# Job 20261030-033238-b87q: verify "Give OLY-B-S1-02 a mechanism: study reuse/instance split test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-S1-02 a mechanism
> 
> Requirement OLY-B-S1-02 in docs/spec.toml is active and has no mechanism.
> Requirement: Reusable work lives in the studying capability, instance-specific work stays with its instance, each later instance is studied with a fresh context, and gaps are fixed in the capability and re-checked.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-S1-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-S1-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add an athena-core integration test that checks, on the real repo, that reusable study logic sits in crates/athena-cap-study (free of instance names) and instance data stays in studies/<id>/ (data only, pinned, no code); record the test as the mechanism in docs/spec.toml with a measure ending `status: not shown`. Audience: Athena's spec-gap check and the verifier.

Assumptions:
- Code survey: read crates/athena-core/tests/learnings_recorded.rs (model: real-file checks, owner_problem, measure_label_problem using parse_spec/measure_problem/spec_gaps from crates/athena-core/src/hestia.rs), crates/athena-core/src/run/pin.rs (studies/<id>/pin.json), crates/athena-cap-study/src/{lib,model,records,rules}.rs exist. The test adds no record or data file under crates/, so check_record_file/AnyRecord round trip are not touched.
- Existing code: crates/athena-core/src/hestia.rs (parse_spec, measure_problem, spec_gaps), crates/athena-core/tests/learnings_recorded.rs pattern, crates/athena-cap-study/src, studies/firstmate-work-handling/, docs/spec.toml; the new test reuses hestia functions.
- Only one instance (firstmate-work-handling) is studied so far; Hermes and Athena are not. The test therefore checks the split on existing instances generically (iterating every studies/*/), so later instances are covered automatically.
- Fresh context per instance and gap-fix-and-recheck cannot be shown by static file checks; the measure honestly ends `status: not shown` and names the test command, and the test docs state this limit.
- The mechanism path is crates/athena-core/tests/study_instance_split.rs; test command `cargo test -p athena-core --test study_instance_split`.
- Consumers: none found for new files; no config files change.
- Term ladder: not used.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-S1-02 has mechanism naming crates/athena-core/tests/study_instance_split.rs (not "none"), owner "Athena", and a measure that names `cargo test -p athena-core --test study_instance_split` and ends with `status: not shown`.
2. [content] crates/athena-core/tests/study_instance_split.rs asserts on real files that every studies/<id>/ holds only data (json/md, no .rs) and has a pin.json, and that crates/athena-cap-study/src does not contain any studies/ instance id string.
3. [behavior] Run `cargo test -p athena-core --test study_instance_split`: passes on the change; its negative-case unit tests (an instance dir with a .rs file, a capability source naming an instance, a missing pin, a measure lacking the label) each produce a problem, so the test fails without the mechanism.
4. [behavior] Run `cargo test -p athena-core --test study_instance_split`: the test also confirms the real OLY-B-S1-02 requirement no longer appears in spec_gaps (mechanism and measure present, measure_problem is None).
5. [behavior] cargo fmt --check, cargo clippy and cargo test pass on the final tree (Athena's recorded check on the same tree counts).
6. [content] CHANGELOG.md gets a job entry; no other governance file (athena.toml, defaults.toml, workflows) changes.

## Expected deliverables

- crates/athena-core/tests/study_instance_split.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Studying Hermes Agent or Athena itself
- Changing the study capability's runtime behavior or prompts
- Editing docs/decisions.md entries or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test file plus cargo test/clippy/fmt criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the named test command on real files.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by a test recorded in docs/spec.toml.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the spec-entry test, which parses the real docs/spec.toml and asserts the OLY-B-S1-02 fields. Run: `cargo test -p athena-core --test study_instance_split spec_entry_has_a_real_mechanism`. Observes: Mechanism is not `none`, contains `crates/athena-core/tests/study_instance_split.rs`, `StudyCapability::admit` and `every_study_follows_the_conventions`. Owner is `Athena`. The measure contains `cargo test -p athena-core --test study_instance_split` and ends with `status: not shown`. `measure_problem` is None.
  Fails without the behavior: Before the change the entry is mechanism `none`, measure `The spec gap list; status: requirement` (docs/spec.toml:2929-2931), so each assertion fails.
  Test seam: Production entry point `athena_core::hestia::parse_spec` (hestia.rs:1172) and `measure_problem` (hestia.rs:1219) on the real docs/spec.toml, read through CARGO_MANIFEST_DIR/../.. as `repo_file` in learnings_recorded.rs:24-29. No double. Exists; the test file is created by plan step 1.
- Criterion 2: Two real-file tests walk the real tree. `every_study_instance_holds_no_code_and_a_pin` lists studies/*/ recursively (asserting at least one instance) and calls `instance_problems`. `the_study_capability_names_no_instance` reads every file under crates/athena-cap-study/src and prompts/ and calls `capability_problems` with the directory names under studies/. Run: `cargo test -p athena-core --test study_instance_split`. Observes: Both pass on the real tree. firstmate-work-handling holds only json files including pin.json (Glob of studies/**). No cap-study file contains `studies/firstmate-work-handling` or `"firstmate-work-handling"` (grep of crates/athena-cap-study found no such hit).
  Fails without the behavior: Without the test file, cargo reports no such test target. With it, the criterion 3 negatives show each check bites.
  Test seam: `instance_problems(id, &[String] relative paths)` and `capability_problems(&[(String,String)] sources, &[String] ids)` are plain functions in the new file, fed from std::fs reads of the real repo. Added by plan steps 2 and 3. No double.
- Criterion 3: Negative unit tests call the same pure functions on in-memory input. Each asserts the clean input gives no problem, then the mutated input gives the specific problem: (a) a `helper.rs` and a nested `sub/run.sh` in an instance; (b) a listing without pin.json; (c) a source containing `studies/firstmate-work-handling/x.json`, and one containing `"firstmate-work-handling"`; (d) id `athena` against `athena_core`, `athena-cap-study` and prose 'Athena' gives none, against `studies/athena/` or `"athena"` gives one, and `studies/athena-extra` gives none for id `athena`; (e) a nested `sub/dir/x.json` and an `.md` file are not flagged; (f) a measure without `status: not shown`, a measure without the test command, a wrong owner, and mechanism `none` appearing in spec_gaps. Run: `cargo test -p athena-core --test study_instance_split`. Observes: Every mutated input yields exactly the expected problem string. Every clean or allowed input yields none. The test therefore fails if any check is removed or loosened, and does not refuse layouts that `admit` accepts (lib.rs:267).
  Fails without the behavior: If `instance_problems` stopped flagging code files or missing pins, or `capability_problems` ignored the two reference forms, the matching negative test would see an empty list and fail. If it reverted to substring matching, the `athena_core` case would fail.
  Test seam: The production checks are `instance_problems`, `capability_problems`, `owner_problem`, `measure_label_problem` and `athena_core::hestia::spec_gaps` (hestia.rs:1291). Fixtures are plain strings or a real requirement with one field changed. The logic under test is not in the fixtures. Added by plan steps 2 to 5; `spec_gaps` exists.
- Criterion 4: `spec_entry_has_a_real_mechanism` calls spec_gaps on the parsed real requirements and asserts none has id OLY-B-S1-02. A companion test sets mechanism to `none` on the real requirement and asserts spec_gaps returns one gap with `no_mechanism` true, so the check is not vacuous. Run: `cargo test -p athena-core --test study_instance_split spec_entry_has_a_real_mechanism`. Observes: OLY-B-S1-02 is absent from spec_gaps and `measure_problem(&r.measure)` is None on the changed spec. The `none` copy is reported as a gap.
  Fails without the behavior: The old entry (mechanism `none`) is a gap, so the assertion fails.
  Test seam: `athena_core::hestia::{parse_spec, spec_gaps, measure_problem}` (hestia.rs:1172, 1291, 1219) on the real docs/spec.toml. Exists. Same pattern as learnings_recorded.rs:289-318.
- Criterion 5: Run the three Athena checks on the final tree. The full test run includes xtask `checked_in_timings_cover_every_workspace_test_unit` (xtask/tests/ci_shards.rs:411), which needs the shard-timings line, and every other test that parses docs/spec.toml. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A missing timings line fails the xtask ci_shards test. A bad TOML edit fails every parse_spec user.
  Test seam: The cargo commands themselves. Exist.
- Criterion 6: Read the diff of the final tree. Run: `git diff --stat`. Observes: CHANGELOG.md gains one bullet under `## [Unreleased]` / `### Added` naming OLY-B-S1-02, the test file, the existing mechanisms it complements and the `status: not shown` limit. The other changed paths are the new test file, docs/spec.toml and xtask/shard-timings.txt (a data file). athena.toml, defaults.toml and .github/workflows are untouched.

Planned files:

- crates/athena-core/tests/study_instance_split.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4663152f6af7 → candidate 13ac77080027. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                     |   1 +
 crates/athena-core/tests/study_instance_split.rs | 312 +++++++++++++++++++++++
 docs/spec.toml                                   |   4 +-
 xtask/shard-timings.txt                          |   1 +
 4 files changed, 316 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-033238-b87q/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-033238-b87q/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43362882419 for 13ac7708002725bc4a3a7bfefccb8df149533e16)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43362882419 for 13ac7708002725bc4a3a7bfefccb8df149533e16)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43362882419 for 13ac7708002725bc4a3a7bfefccb8df149533e16)
- ci-result: `not a check` passed (Check results for 13ac7708002725bc4a3a7bfefccb8df149533e16 come from the CI run https://github.com/doodla/athena/actions/runs/43362882419 (922s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy and cargo test pass on the final tree (Athena's recorded check on the same tree counts).)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test study_instance_split spec_entry_has_a_real_mechanism`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling jiff v0.2.38
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast v1.0.27
   Compiling bitflags v2.13.2
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 30.20s
     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 1 test
test spec_entry_has_a_real_mechanism ... FAILED

failures:

---- spec_entry_has_a_real_mechanism stdout ----

thread 'spec_entry_has_a_real_mechanism' (20779) panicked at crates/athena-core/tests/study_instance_split.rs:263:5:
assertion `left != right` failed
  left: "none"
 right: "none"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    spec_entry_has_a_real_mechanism

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.06s

error: test failed, to rerun pass `-p athena-core --test study_instance_split`
error: 1 target failed:
    `-p athena-core --test study_instance_split`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test study_instance_split`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 12 tests
test a_capability_naming_an_instance_is_reported ... ok
test an_instance_with_code_is_reported ... ok
test an_instance_without_a_pin_is_reported ... ok
test every_study_instance_holds_no_code_and_a_pin ... ok
test short_ids_do_not_match_ordinary_text ... ok
test nested_data_files_are_allowed ... ok
test spec_check_refuses_a_wrong_owner ... ok
test spec_check_refuses_a_measure_without_the_command ... ok
test the_study_capability_names_no_instance ... ok
test spec_check_refuses_a_wrong_status_label ... ok
test spec_check_refuses_a_none_mechanism ... ok
test spec_entry_has_a_real_mechanism ... FAILED

failures:

---- spec_entry_has_a_real_mechanism stdout ----

thread 'spec_entry_has_a_real_mechanism' (20816) panicked at crates/athena-core/tests/study_instance_split.rs:263:5:
assertion `left != right` failed
  left: "none"
 right: "none"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    spec_entry_has_a_real_mechanism

test result: FAILED. 11 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

error: test failed, to rerun pass `-p athena-core --test study_instance_split`
error: 1 target failed:
    `-p athena-core --test study_instance_split`
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
