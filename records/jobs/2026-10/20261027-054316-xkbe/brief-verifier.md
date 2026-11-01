# Job 20261027-054316-xkbe: verify "Compare failing tests to the base so job-caused failures are separate"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Nothing separates a failure the job caused from one already on the base, or from a clash with a conc
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: failing-check-blocks-all-criteria-unjudged-no-baseline (20261017-075355-rwrh)
> Cause: Nothing separates a failure the job caused from one already on the base, or from a clash with a concurrent config and test change. The check stage has no baseline run and no mapping from tests to the change. A failing check also ends the jo...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261017-075336-m1y1 (job 20261017-044408-wcwy); prevention it proposed: Add a code check at admission and before handover. Admission would run the full test suite on the base commit and record which tests fail. Before handover the producer must run the same suite, and the system would compare the two. A test th...
> Introspection 20261017-184719-0s4h (job 20261017-182453-a5sf); prevention it proposed: Before the producer starts, run the repo's full check suite on the base commit and record which checks already fail. At verification, compare the failures after the job to that baseline. Failures that already existed are reported separately...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the baseline records only whether a whole check (e.g. "test") failed on the base, so a failure is either wholly pre-existing or wholly the job's; a different test failing, or a clash with a concurrent config/test change, cannot be told apart (jobs 20261017-044408-wcwy and 20261017-182453-a5sf). Fix it where it originates by recording failing test names in the baseline and comparing per test at verification.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (is_pre_existing, mark_pre_existing, VerdictInputs.baseline), crates/athena-core/src/run/checks.rs (take_baseline runs the configured checks on the base snapshot via crates/athena-core/src/checks.rs baseline_runs), crates/athena-core/src/run/verification.rs (verifier always runs; failed check fails the final verdict unless pre-existing) and crates/athena-data/src/records.rs (Baseline::failed_before matches a check by name only; Baseline::validate). The Baseline record is already in AnyRecord, so a new optional field on CheckRun/Baseline must keep serde defaults so old records still parse and the record round-trip test and check_record_file still pass.
- Cause as read: pre-existing detection is per check name, so (a) a check failing on the base masks a new, different test failure in the same check, and (b) a failure from a clash with a concurrent change is judged as the job's own. The name-only comparison is the origin; the fix is test-level.
- Fix shape: parse failing test names from cargo test output (the 'failures:' list) into CheckRun for baseline and candidate runs; is_pre_existing treats a failed check as pre-existing only if every failing test also failed on the base; failures new relative to the base are reported separately in the finding with their names. Where no test names can be parsed, behaviour falls back to the current name-level rule.
- Jobs 20261017-044408-wcwy and 20261017-182453-a5sf: their candidate changes are not retried or kept here; their findings are addressed by the general mechanism (failing test names classified against the base), not by editing their files. The athena.toml / docs/design/job-planning.md clashes are not edited.
- Audience: the verifier and the user reading a rejected job, who need to see which failures the job caused. Check names in the stage are the existing ones (fmt, clippy, test).

## Acceptance criteria

1. [content] The change description names the root cause (baseline compares whole checks by name, with no per-test comparison) and the fix is in crates/athena-data/src/records.rs and crates/athena-core/src/verify.rs / checks.rs, not in any one job's files; check with git diff.
2. [behavior] A unit test in crates/athena-core/src/verify.rs fails without the change and passes with it: a baseline where test X failed and a candidate where tests X and Y fail is NOT pre-existing and the verdict fails naming Y; where only X fails it is pre-existing. Run cargo test -p athena-core.
3. [behavior] A parser test shows failing test names are extracted from real cargo test output of the shapes quoted in the two jobs (the 'failures:' list), and output with no list falls back to the name-level rule. Run cargo test -p athena-core.
4. [behavior] A Baseline/CheckRun record written before the change (no test-name field) still parses and validates; the records round-trip test and check_record_file pass. Run cargo test.
5. [behavior] The finding or note for a failed check separates tests already failing on the base from tests newly failing, so the user can see what the job caused; covered by a test on the produced finding text. Run cargo test -p athena-core.
6. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test --locked pass.
7. [content] No governance file (athena.toml, defaults.toml, workflows) is changed; CHANGELOG.md has an entry describing the root cause.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Re-landing the auditor role or the ontology file from the two earlier jobs; their candidate files (auditor.rs, ontology.toml, tests, docs) are dropped, not kept
- Fixing the max_concurrency test or the banned-word wording in docs, which are separate instances
- Changing athena.toml, defaults.toml or workflows
- Rebasing or merging concurrent changes automatically

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix the per-check-name baseline comparison itself
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: tests for the parser and the verdict; fmt, clippy, test pass
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: all changes in Rust
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria run cargo test on the new tests
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: enforced in verdict code, not prose
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one focused change to baseline comparison
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff. The doc comment on `is_pre_existing`, `Baseline::failed_tests_before`, the `failed_tests` field doc, the new `failed_test_names` and `cargo_test_args`, and the CHANGELOG Unreleased/Fixed entry must name the cause (baseline compares whole checks by name, no per-test comparison) and say the change covers case (a) only. Check that no job's candidate file (auditor.rs, ontology.toml) appears. Run: `git diff --stat`. Observes: The diff touches records.rs, verify.rs, checks.rs, run/checks.rs, run/verification.rs, CHANGELOG.md and docs/design/job-planning.md, plus only one-line `failed_tests: Vec::new()` additions in the compiler-reported literals. The CHANGELOG text names the root cause and states the (a)-only scope.
- Criterion 2: Unit tests in the verify.rs tests module. They use the existing helpers `baseline_with`, `judge_against` and `check` (`check` gains an empty `failed_tests`; each test sets the field on real records). Entry points: `is_pre_existing`, `mark_pre_existing`, `judge`. Cases: base {X} vs candidate {X,Y} is not pre-existing, the verdict is Fail and Y is named; base {X} vs {X} is pre-existing and passes; base {X} vs {Y} is not pre-existing; base {A X} vs candidate {A X, B X} (two binaries) is not pre-existing; base {A X} vs candidate {A X, `B (did not finish)`} (a crashed binary) is not pre-existing, while base and candidate both holding the same marker is pre-existing. Run: `cargo test -p athena-core verify::tests`. Observes: The tests pass: {X} vs {X,Y} is not pre-existing, the verdict is Fail and Y is named; {X} vs {X} stays pre-existing; the crashed-binary case blocks.
  Fails without the behavior: Without the per-test comparison `is_pre_existing` matches by check name, so base {X} vs candidate {X,Y} is marked pre-existing and the verdict passes; the assertions that the check is not pre-existing, the verdict is Fail and Y is named all fail.
  Test seam: Production entry points `verify::is_pre_existing`, `mark_pre_existing` and `judge` (exist: verify.rs lines 134-151 and the judge function). Fixtures: existing `baseline_with` (line 731), `judge_against` (line 741) and `check` (line 678); step 3 adds the empty field to `check`. Real records, no double; the comparison is production code.
- Criterion 3: Unit tests in checks.rs call `failed_test_names(&str)` with literal text in cargo's exact format (`Running unittests src/lib.rs (target/debug/deps/<crate>-<hash>)`, the blank-line-led stdout `failures:` block with `---- name stdout ----`, the 4-space-indented `failures:` list, `test result: FAILED. N passed; M failed`), written by the producer from a real local run with a deliberately failing test. Cases: (a) two binaries, two failing tests, names as `<binary> <test>`; (b) a doc-test failure; (c) only a `---- name stdout ----` block and no list yields the `(did not finish)`-free empty or count-mismatch result as the parser rule defines, and output with no `Running` sections yields an empty list; (d) no failures yields empty; (e) a crashed binary (a `Running` line, `running N tests`, then the next `Running` line with no `test result:`) yields `<binary> (did not finish)`; (f) cargo's early-stop shape. A pure `cargo_test_args` test: `--no-fail-fast` goes right after `test`, before any `--`, is not duplicated; `--fail-fast`, a non-cargo program or `cargo +nightly test` gives records_names=false. A `run_check` test uses a temp script named `cargo` (absolute path as program, Test-kind CheckSpec) that prints canned output, echoes its args and exits 101: `failed_tests` is filled, the script saw `--no-fail-fast`, `command` is unchanged and the note mentions the flag. A second `run_check` case uses `sh` (the existing `sh_spec`): the list is empty. A verify.rs test shows an empty list leads to the name-level rule (base fails, candidate fails, pre-existing). Run: `cargo test -p athena-core failed_test_names`. Observes: Names are extracted and qualified by binary for multi-binary and doc-test output; a crashed binary produces the marker; output with no list gives an empty result; the cargo-named script run records names, the flag and the note; sh runs record none; the empty result leads to the name-level rule.
  Fails without the behavior: Without the parser `failed_test_names` does not exist and the tests cannot compile. A parser that reads only the first `failures:` stdout section, or `test ... FAILED` lines, gives wrong names and fails the exact-name assertions. Without the completeness rule the crashed-binary test returns an empty list. Without the flag insertion the script would not see `--no-fail-fast`.
  Test seam: Production entry points `checks::failed_test_names` and `checks::cargo_test_args` (added in step 2) and `checks::run_check` (exists, checks.rs line 190), and `verify::is_pre_existing` (exists). Fixture for run_check: a script written and chmod-ed in a per-test tempdir, named `cargo`, passed by absolute path in `CheckSpec.run`, with the existing `CheckContext` setup used by other run_check tests (redactor from builtin config, `Registry`). The script supplies only canned text; parsing and flag logic run in production code.
- Criterion 4: A new records.rs test builds Baseline JSON in the old shape (`sample_baseline().to_json()` with the `failed_tests` keys removed, or a literal) with no `failed_tests` key on its CheckRuns. It parses it via `AnyRecord::parse` and `check_record_file` at `baseline.json`, then calls `Baseline::validate`. The existing round-trip tests and `a_baseline_belongs_at_baseline_json_and_nowhere_else` run unchanged apart from the field in `sample_baseline`. A second case round-trips a record with names, and a record without names serialises with no `failed_tests` key. Run: `cargo test`. Observes: Old JSON parses and validates with an empty `failed_tests`; a record with names round-trips; the round-trip tests and check_record_file pass across the workspace.
  Fails without the behavior: Without `#[serde(default)]` old JSON fails to parse under `deny_unknown_fields`/required fields; without `skip_serializing_if` existing records change shape and the old-shape and round-trip assertions fail.
  Test seam: Production entry points `AnyRecord::parse`, `check_record_file` and `Baseline::validate` (all exist). Fixture: JSON derived from `sample_baseline` (exists; updated in step 1). No double.
- Criterion 5: A verify.rs test judges base {X} vs candidate {X,Y} through `judge_against` and reads the Athena finding. It asserts severity Minor, evidence lists Y under newly failing and X under already failing on the base, and the result has exactly one `check test failed` reason and one such finding. Further cases: only new names gives no 'already' part; a check with no names gives no such finding; a fully pre-existing check gives none; a failed run that got the flag has the flag note. Run: `cargo test -p athena-core verify::tests::the_finding_for_a_failed_check_separates_new_from_base_failures`. Observes: The finding text separates 'newly failing: Y' from 'already failing on the base: X'; the verdict stays Fail with the single existing reason.
  Fails without the behavior: Without the finding, `judge` returns only the `check test failed` reason; the assertions on the finding's severity, summary and evidence fail.
  Test seam: Production entry point `verify::judge` via the existing `judge_against` (exists). Text built by production `failure_split` (added in step 3). Fixtures are real CheckRun and Baseline records from `baseline_with` and `check`. No double.
- Criterion 6: Run the three checks Athena runs on the finished workspace; every CheckRun literal must compile, and `cargo test --locked` also runs the budget, boundary-word and record tests that scan the edited docs. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` each exit 0.
  Fails without the behavior: Without the `failed_tests: Vec::new()` additions the build fails with 'missing field'; failing new tests make `cargo test` exit non-zero; unformatted code fails fmt.
  Test seam: The cargo commands of Athena's check stage, run on the whole workspace (exist today as configured checks). No double.
- Criterion 7: Read the changed file list and the CHANGELOG [Unreleased] / Fixed entry: it names the root cause, the `--no-fail-fast` and did-not-finish completeness rules, and that case (b) is not covered. Run: `git status`. Observes: No athena.toml, defaults.toml or workflow file is changed; the CHANGELOG entry states the root cause (baseline compared whole checks by name, no per-test comparison).

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/status_durations.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base a7b1d637ffb3 → candidate 41efbfbf1579. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  15 ++
 crates/athena-cli/tests/status_durations.rs |   1 +
 crates/athena-core/src/checks.rs            | 395 +++++++++++++++++++++++++++-
 crates/athena-core/src/introspect.rs        |   1 +
 crates/athena-core/src/reading.rs           |   1 +
 crates/athena-core/src/routing.rs           |   1 +
 crates/athena-core/src/run/checks.rs        |   4 +-
 crates/athena-core/src/run/mod.rs           |   1 +
 crates/athena-core/src/run/send_back.rs     |   1 +
 crates/athena-core/src/run/tests.rs         |   1 +
 crates/athena-core/src/run/verification.rs  |   3 +-
 crates/athena-core/src/status.rs            |   1 +
 crates/athena-core/src/verify.rs            | 175 +++++++++++-
 crates/athena-data/src/records.rs           |  66 ++++-
 docs/design/job-planning.md                 |  10 +
 15 files changed, 670 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1883 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets and cargo test --locked pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
