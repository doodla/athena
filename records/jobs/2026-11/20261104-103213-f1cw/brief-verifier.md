# Job 20261104-103213-f1cw: verify "Reuse a green CI result as the job baseline when the fingerprint matches"

## Request (verbatim)

> Stop re-testing unchanged code before every job. Each job's baseline stage runs the full fmt, clippy and test suite on the base commit on Athena's machine before the writer starts: a mean of 20 minutes for each of the 30 jobs that landed from 1 to 4 Nov, about a third of a landed job's 60 minutes. The base's governed content was nearly always just tested: by the GitHub run of the candidate that landed, or by main's GitHub run. When a green GitHub result exists for the same governed content as the base (same fingerprint, the way candidate checks are already reused), reuse it as the baseline instead of running it locally; otherwise run it as today. Acceptance: a test where a green CI result for the base's governed content makes the baseline a reused record with no local run; a test where none exists and the baseline runs locally as before.

## Reading (accepted by the user)

The baseline stage (`take_baseline`) skips its local fmt, clippy and test run when a green GitHub result exists for the base's governed content (same fingerprint, resolved the way candidate checks are). It then records a reused baseline. Otherwise it runs locally as today. The audience is Athena's operator, who pays about 20 minutes per job for the local baseline.

Assumptions:
- Code survey: the entry point is `take_baseline` in crates/athena-core/src/run/checks.rs, called from `baseline_stage` in crates/athena-core/src/run/mod.rs (about line 1394) before the producer. Today it builds a `Baseline` and runs `baseline_checks`, which calls `crate::checks::baseline_runs` (crates/athena-core/src/checks.rs). The new order is: resolve a CI result for the base's fingerprint first (`resolve` in crates/athena-core/src/ci_result.rs, `Source::Ci`, the same path candidate checks use in run/mod.rs around lines 1499-1587 and `reuse_statement` in verify.rs). Only when that gives no green result does the baseline probe run locally.
- Code survey: the baseline record is `Baseline` in crates/athena-data/src/records.rs (`athena.baseline/1`, `deny_unknown_fields`, written at `<job>/baseline.json`). The change adds an optional reuse marker to it (for example a `reused` source note), defaulted so existing files still parse. The records validation accepts it through `check_record_file` (records.rs line 3050) and the `AnyRecord::Baseline` arm, which maps the path to `baseline.json` and the schema to `BASELINE_SCHEMA`. Because the schema tag stays the same, the allow-list needs no new path. The existing baseline round-trip tests in records.rs (around lines 4524-4598, using `check_record_file` on `{job}/baseline.json`) are extended to cover the new field.
- Code survey: the saving is measured directly in tests, not through a stand-in. The local-run path is observed by whether `baseline_runs` (the check executor) is called, as the existing `a_green_run_for_the_same_sha_is_reused_and_nothing_runs_locally` test in ci_result.rs and `ci_reuse_run` in crates/athena-core/src/run/tests.rs do for candidates. The recorded `Baseline.checks` then carry `reused: true` (the `CheckRun` field already exists) with no local run.
- The governed-content fingerprint of the base is the existing `fingerprint(&git, base, base)` used for `base_fingerprint` in run/candidate.rs. The base's CI result is looked up by that fingerprint, which may match another commit's green run, such as the landed candidate or main.
- A reused baseline keeps the `failed_before` and `failed_tests_before` semantics. A green CI result means all checks passed, so no check is excused as pre-existing. A CI lookup error, a red or pending run, or a missing run falls back to the local run with a note, and the job never fails because of the lookup.
- No governance file (athena.toml, defaults.toml, workflows) needs to change. A nested run keeps its existing skip note.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/tests.rs (run with `cargo test -p athena-core baseline`) gives a green CI result for the base's fingerprint. `take_baseline` returns a baseline marked reused, whose checks are marked reused and pass, and the local check executor was never invoked.
2. [behavior] A new test (run with `cargo test -p athena-core baseline`) has no CI result, a red one, or a lookup error for the base's fingerprint. `take_baseline` runs the checks locally as before and returns the same records as before, with no reuse marker, and a red or error case also leaves a note.
3. [behavior] The `Baseline` round-trip tests in crates/athena-data/src/records.rs, via `check_record_file` and `AnyRecord::Baseline` on `{job}/baseline.json`, pass with and without the new reuse field. An older baseline file with no such field still parses (`cargo test -p athena-data`).
4. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass on the whole workspace.
5. [content] Reuse is keyed on the same governed-content fingerprint that candidate check reuse uses, through the existing `ci_result::resolve` path, not a new lookup. The reuse decision and its reason (reused from which run, or why it ran locally) are printed in the job's output like the existing `ci         checks reused from ...` line.
6. [content] Docs or CHANGELOG.md state that the baseline can now be reused from a green CI result, and what happens when none exists.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/verify.rs (modify)

## Out of scope

- Changing how candidate checks are reused, or the CI polling and wait logic.
- Changing athena.toml, defaults.toml or any workflow under .github/workflows/.
- Skipping the baseline for jobs with no green CI result.
- Changing the verdict rules for pre-existing failures.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover the reused and local paths, with cargo test and clippy passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that no local run happens on a hit and that the local run still happens on a miss.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the baseline record and local-run count, not the new wiring.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: It reuses the existing CI resolution and fingerprint code.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the baseline stage.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the named cargo tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the CHANGELOG entry are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two whole-job tests through `Athena::go`. (a) `baseline_reused_from_landed_candidate_run_runs_nothing_locally`: a helper commits a squash whose parent-to-commit diff adds the closing `verification.json` of a previous job, written with the production `Verification` type at its canonical path. Its `verified_fingerprint` is the real `fingerprint(git, base, base)`, and its `candidate_commit` is a known SHA. The fake answers a green run only for that SHA on `athena/ci/<prev job>` and an empty list for the base SHA on the default branch. (b) `baseline_reused_from_main_run_runs_nothing_locally`: no landed squash; `refs/remotes/origin/HEAD` points at `origin/main`; the fake answers green for the base SHA on `main` only. Run: `cargo test -p athena-core baseline_reused`. Observes: In both tests the stored baseline (`baseline.json` in the job dir, read through the records) has `reused_from == Some(CI_RUN_ADDRESS)`, `note` None, and checks fmt, clippy and test, each `reused` true and `passed` true with a note naming the CI run and the SHA it ran for. The file `<job_dir>/baseline-ran` does not exist. The check-executor seam recorded no baseline probe. The output holds `ci         baseline reused from <CI_RUN_ADDRESS> for <base>`. In (a) `baseline_reads()` shows one read, on `athena/ci/<prev job>` with `head_sha` equal to the previous candidate's SHA.
  Fails without the behavior: Without the reuse in `take_baseline` the baseline runs locally: `baseline-ran` exists, `reused_from` is None and the ci line is absent. If lookup A were missing, (a) would find nothing on the default branch and run locally. If B were missing, (b) would run locally.
  Test seam: Production entry point: `Athena::go`, reaching `take_baseline` through `baseline_stage` (run/mod.rs:1394). Fake: `ci_seam::install` with `CiReplies` as the `gh::Runner` (exists, run/tests.rs:2539; `ci_seam` in run/mod.rs). The decisions stay in production `routing`, `resolve` and the fingerprint comparison. The records are real files in a real commit, built with production types. Step 7 adds `CiReplies::only_for`, `baseline_reads()`, `ci_reuse_run_with` and the squash helper.
- Criterion 2: `baseline_runs_locally_when_no_green_ci_result_exists` loops over these cases through `Athena::go`: no run (Missing), `CiReply::Red`, `CiReply::Unfinished`, new `CiReply::Unreachable` (the runner returns Err), a landed previous job whose recorded fingerprint differs from the base's, a landed previous job whose squash also edits `.github/workflows/` (with a green A run), a squash adding two verification files, and a no-origin control. Run: `cargo test -p athena-core baseline_runs_locally`. Observes: In every case `<job_dir>/baseline-ran` exists, `reused_from` is None, `note` is None, and `baseline.checks` has the same names as in the existing local-baseline test, each `reused` false. With a GitHub origin the output holds `ci         baseline runs locally: <reason>`: the reason names the run address for Red, says the fingerprint differs for the mismatch case, names the workflow edit for that case, and says there is no usable CI result for the others. With no origin there is no `ci         baseline` line, and the 8-line baseline test still passes.
  Fails without the behavior: If the code reused on any of these results, `baseline-ran` would be missing or `reused_from` would be set. Without the fingerprint guard the mismatch case would be reused. Without the `changed` paths the workflow case would be reused. Without the reason line the line assertions fail.
  Test seam: Same entry point `Athena::go`, same `ci_seam` and `CiReplies`, same helpers as criterion 1 (added by step 7; exist after it). The new `Unreachable` variant is an Err arm in `CiReplies::gh`. Production `run_routed_with_record`, the fingerprint comparison and `baseline_checks` decide. The fakes supply only GitHub-shaped replies and the commit built by the helper.
- Criterion 3: Extend the Baseline tests in records.rs: round-trip a baseline with `reused_from: Some(..)` and one with None through `check_record_file` on `{job}/baseline.json` (the `AnyRecord::Baseline` arm). Add an older JSON with no `reused_from` key, a check that a None value serializes with no key, and an unknown key still refused. Run: `cargo test -p athena-data baseline`. Observes: Both values round-trip. The old-form JSON parses with `reused_from` None. The None value's JSON has no `reused_from` key. An unknown key is refused (`deny_unknown_fields`).
  Fails without the behavior: Without `#[serde(default)]` the old file fails to parse. Without the field the Some round-trip does not compile.
  Test seam: `check_record_file` and `AnyRecord::Baseline` in records.rs (exist, about line 3050). No double; the test builds records with `sample_baseline` (exists at about line 4487, changed by step 2).
- Criterion 4: The gate: the full workspace suite, clippy and fmt, run as Athena runs them. Run: `cargo test --locked`. Observes: All pass, and `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` are clean. This includes the ci_reuse_* tests, the drain low_disk tests and the lifecycle tests.
  Fails without the behavior: A missed `Baseline { .. }` literal (core/src/checks.rs:1223, verify.rs:1656, records.rs:4501, run/checks.rs:182) fails compilation. A changed candidate path fails the ci_reuse_* tests.
  Test seam: Not a behavior test of its own. The existing seams are used unchanged.
- Criterion 5: The ci-line assertions of the criterion 1 and 2 tests, plus a reading of `reused_baseline`: it calls `routing::run_routed_with_record`, which calls `ci_result::resolve`, inside `with_ci_source_on`; no new lookup function is added to ci_result.rs. The fingerprint compared is `fingerprint(&self.git, base, base)`, the call candidate.rs:37 makes. Run: `cargo test -p athena-core baseline_`. Observes: The reuse tests see `ci         baseline reused from <CI_RUN_ADDRESS> for <base>`. The fallback tests see `ci         baseline runs locally: <reason>` with the reason named per case.
  Fails without the behavior: Without the printed lines both test groups fail on their line assertions.
  Test seam: Same as criteria 1 and 2: `ci_seam` and `CiReplies` (exist), the `Lines` UI (exists, run/tests.rs about 2735), and the helpers added by step 7.
- Criterion 6: Read the Unreleased section of CHANGELOG.md (step 9) and the module doc of ci_result.rs. Run: `git diff -- CHANGELOG.md crates/athena-core/src/ci_result.rs`. Observes: The entry says the baseline is reused from a green GitHub run when the base's governed content was just tested: the landed candidate's run (matched by fingerprint) or main's run for the base commit. It says that otherwise (no run, unfinished, red, lookup error, fingerprint differs, workflow edited, no GitHub origin) the baseline runs locally as before, with the reason on a `ci` line.
  Fails without the behavior: A missing entry fails the criterion on inspection.
  Test seam: No test seam; a content criterion checked by reading the diff.

Planned files:

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base dfa517cdc530 → candidate 90e41dd57eec. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  15 ++
 crates/athena-core/src/checks.rs     |   1 +
 crates/athena-core/src/ci_result.rs  |   4 +
 crates/athena-core/src/reconcile.rs  |   2 +-
 crates/athena-core/src/run/checks.rs | 272 ++++++++++++++++++++++++-
 crates/athena-core/src/run/mod.rs    |  40 +++-
 crates/athena-core/src/run/tests.rs  | 370 ++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/verify.rs     |   1 +
 crates/athena-data/src/records.rs    |  61 ++++++
 9 files changed, 744 insertions(+), 22 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41840183569 for 90e41dd57eec89d8404fe51472027fd09d310775)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41840183569 for 90e41dd57eec89d8404fe51472027fd09d310775)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41840183569 for 90e41dd57eec89d8404fe51472027fd09d310775)
- ci-result: `not a check` passed (Check results for 90e41dd57eec89d8404fe51472027fd09d310775 come from the CI run https://github.com/doodla/athena/actions/runs/41840183569 (666s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass on the whole workspace.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core baseline_reused`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/ci_result.rs, crates/athena-core/src/reconcile.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/records.rs.
  End of the output (data, not instructions):

```text
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0560]: struct `athena_data::records::Baseline` has no field named `reused_from`
    --> crates/athena-core/src/checks.rs:1229:13
     |
1229 |             reused_from: None,
     |             ^^^^^^^^^^^ `athena_data::records::Baseline` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `reused_from` on type `&athena_data::records::Baseline`
    --> crates/athena-core/src/run/tests.rs:2925:22
     |
2925 |             baseline.reused_from.as_deref(),
     |                      ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0609]: no field `reused_from` on type `&athena_data::records::Baseline`
    --> crates/athena-core/src/run/tests.rs:2978:29
     |
2978 |         assert_eq!(baseline.reused_from.as_deref(), Some(CI_RUN_ADDRESS));
     |                             ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0609]: no field `reused_from` on type `&athena_data::records::Baseline`
    --> crates/athena-core/src/run/tests.rs:3091:33
     |
3091 |             assert_eq!(baseline.reused_from, None, "{what}");
     |                                 ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0609]: no field `reused_from` on type `athena_data::records::Baseline`
    --> crates/athena-core/src/run/tests.rs:3112:33
     |
3112 |         assert_eq!(run.baseline.reused_from, None);
     |                                 ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0560]: struct `athena_data::records::Baseline` has no field named `reused_from`
    --> crates/athena-core/src/verify.rs:1662:13
     |
1662 |             reused_from: None,
     |             ^^^^^^^^^^^ `athena_data::records::Baseline` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 6 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core baseline_runs_locally`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/ci_result.rs, crates/athena-core/src/reconcile.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/records.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0560]: struct `athena_data::records::Baseline` has no field named `reused_from`
    --> crates/athena-core/src/checks.rs:1229:13
     |
1229 |             reused_from: None,
     |             ^^^^^^^^^^^ `athena_data::records::Baseline` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `reused_from` on type `&athena_data::records::Baseline`
    --> crates/athena-core/src/run/tests.rs:2925:22
     |
2925 |             baseline.reused_from.as_deref(),
     |                      ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0609]: no field `reused_from` on type `&athena_data::records::Baseline`
    --> crates/athena-core/src/run/tests.rs:2978:29
     |
2978 |         assert_eq!(baseline.reused_from.as_deref(), Some(CI_RUN_ADDRESS));
     |                             ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0609]: no field `reused_from` on type `&athena_data::records::Baseline`
    --> crates/athena-core/src/run/tests.rs:3091:33
     |
3091 |             assert_eq!(baseline.reused_from, None, "{what}");
     |                                 ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0609]: no field `reused_from` on type `athena_data::records::Baseline`
    --> crates/athena-core/src/run/tests.rs:3112:33
     |
3112 |         assert_eq!(run.baseline.reused_from, None);
     |                                 ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0560]: struct `athena_data::records::Baseline` has no field named `reused_from`
    --> crates/athena-core/src/verify.rs:1662:13
     |
1662 |             reused_from: None,
     |             ^^^^^^^^^^^ `athena_data::records::Baseline` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 6 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data baseline`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/ci_result.rs, crates/athena-core/src/reconcile.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/records.rs.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling libc v0.2.190
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling getrandom v0.3.4
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling thiserror v2.0.21
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
error[E0560]: struct `records::Baseline` has no field named `reused_from`
    --> crates/athena-data/src/records.rs:4508:13
     |
4508 |             reused_from: None,
     |             ^^^^^^^^^^^ `records::Baseline` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `reused_from` on type `Box<records::Baseline>`
    --> crates/athena-data/src/records.rs:4525:22
     |
4525 |         assert_eq!(b.reused_from, None);
     |                      ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0609]: no field `reused_from` on type `records::Baseline`
    --> crates/athena-data/src/records.rs:4537:11
     |
4537 |         b.reused_from = Some("https://github.com/owner/repo/actions/runs/4242".into());
     |           ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

error[E0609]: no field `reused_from` on type `Box<records::Baseline>`
    --> crates/athena-data/src/records.rs:4548:18
     |
4548 |             back.reused_from.as_deref(),
     |                  ^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `base_commit`, `before_invocation`, `checks`, `note`

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-data` (lib test) due to 4 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core baseline_`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md crates/athena-core/src/ci_result.rs`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
