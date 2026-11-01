# Job 20261101-114004-358q: verify "Count CI-reused passing checks as commands run in the verdict"

## Request (verbatim)

> When the checks stage reused GitHub's CI result for the candidate commit, the verdict does not count those check commands as seen running, so a behavior criterion citing cargo fmt, clippy or test is marked unmet (D43) and a passing job is rejected. Count a passing check whose result was reused from a green CI run for the same candidate commit as a command run, as a passing local check on the same tree already counts (D115).
> - In crates/athena-core/src/verify.rs judge(): a reused passing check record for the candidate commit counts toward the seen commands; a failed, skipped or other-commit record still does not.
> - A test in verify.rs builds VerdictInputs with checks reused from CI (exit code absent, the reuse note set, same tree) and a behavior criterion citing cargo test, and shows the criterion is met; it fails on today's code.
> - A test shows a reused record for a different commit, or a failed one, still leaves the criterion unmet.
> - Mark reused records with a field rather than matching note text, if the CheckRun record needs one; keep old records loading.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion; the checks stage covers it.
> - CHANGELOG.md gets an entry.
> Priority: urgent

## Reading (accepted by the user)

When the checks stage reused a green CI run for the candidate commit, judge() in verify.rs should count those passing records as seen commands, so a behavior criterion citing cargo fmt, clippy or test is met. Goal: stop rejecting passing jobs; audience is the verdict and whoever reads its criteria results. Reused records get an explicit CheckRun field, with old records still loading.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (judge() at ~line 308 builds `ran` from commands_run, then adds passing records only when `candidate_tree == checks_tree` and `exit_code == Some(0)`; a reused CI record has no exit code, so it never counts), crates/athena-core/src/checks.rs (~line 230-254 builds reused records from a CiRecord with note 'result reused from the ... for {sha}'; that is the producer to mark), crates/athena-core/src/routing.rs (~267 runs_from_record, resolves reuse before local runs) and crates/athena-data/src/records.rs (CheckRun at ~751, `deny_unknown_fields`, existing optional fields use `#[serde(default, skip_serializing_if)]`). Records validation: `check_record_file` (records.rs ~2927) parses into `AnyRecord` (~2613), so a new `#[serde(default, skip_serializing_if = "...")] reused: bool` on CheckRun is accepted by deny_unknown_fields; old records without it load via default. The existing round-trip test `failing_test_names_round_trip_and_are_read_per_check` (records.rs ~4227) and the CheckRun literal at ~4188 are the pattern for covering it; the producer adds a similar old-record-still-loads test. Behavior is measured directly by calling judge() on VerdictInputs in verify.rs unit tests (existing helper judge_recorded ~816), not through a stand-in.
- Commit match: the record counts only when it is for the candidate commit. CheckRun has no commit field, so the producer decides how judge() knows the commit (for example the existing checks_tree/candidate_tree equality, or a recorded sha next to the reused flag); the test must show a different-commit record stays unmet.
- No governance file (athena.toml, defaults.toml, workflows) needs to change.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/verify.rs builds VerdictInputs with passing checks reused from CI (exit_code None, reused field set, same tree and commit) and a behavior criterion citing `cargo test`, and asserts the criterion is met with commands_verified true. Run `cargo test -p athena-core verify`; the test passes, and it fails on the base code (judge() ignores records without exit code 0).
2. [behavior] Unit tests in verify.rs show a reused record for a different commit/tree, a failed reused record, and a skipped record each leave the `cargo test` behavior criterion unmet. Run `cargo test -p athena-core verify`.
3. [content] CheckRun in crates/athena-data/src/records.rs gains a serde-defaulted boolean field marking reused records (not note-text matching), crates/athena-core/src/checks.rs sets it where reused records are built, and judge() reads the field, not the note string (Grep for the note text in verify.rs finds no matching on it).
4. [behavior] A records.rs test deserializes a CheckRun JSON without the new field and it loads with the field false, and a reused record round-trips through check_record_file/AnyRecord. Run `cargo test -p athena-data`.
5. [content] CHANGELOG.md has an entry describing that checks reused from a green CI run for the candidate commit now count as commands run in the verdict (D43, D115).

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Changing how or when CI results are fetched or reused
- Changing the baseline or pre-existing failure logic
- Listing a repository-wide fmt, clippy or test run as its own acceptance criterion (the checks stage covers it)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests in verify.rs and records.rs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in judge()'s seen-command rule, not by loosening the criterion check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run judge() on reused-check inputs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is a passing job no longer rejected for unseen commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New unit test `a_passing_check_reused_from_ci_on_the_same_commit_counts_as_run` in verify.rs. It builds a `check(true)` with `exit_code: None`, `reused: true` and note 'result reused from the CI run ...', and passes it through `judge_recorded(&[reused], Some("abc"), Some("abc"), vec!["cargo test"])`. It also covers all three checks (`cargo fmt --all --check`, `cargo clippy ...`, `cargo test --locked`) with one reused record each and cited commands that match the check's command text. Run: `cargo test -p athena-core verify`. Observes: `criteria[1].met` and `commands_verified` are true and no D43 finding exists.
  Fails without the behavior: On today's code the filter `exit_code == Some(0)` drops the reused record. `ran` stays empty, the criterion is unmet, commands_verified is false and a D43 finding appears, so the test's assertions fail.
  Test seam: Production entry point: `verify::judge(&VerdictInputs)`. Guard: `candidate_tree` Some and equal to `checks_tree`. Fixture: records built by the existing `check()` helper with fields overridden, injected via the existing `judge_recorded` helper (verify.rs ~817, exists). The `check()` literal needs `reused: false` (step 1). The judge logic is not in the test.
- Criterion 2: Three unit tests in verify.rs, same helper. (a) A reused passing record with candidate 'abc' and checks_tree 'def' or None is unmet with a D43 finding. (b) A reused record with `passed: false` and `exit_code: None` is unmet. (c) A skipped record (no exit code, passed true, `reused: false`, a 'not run: ...' note, like `checks::skipped_run`) is unmet. The existing placeholder test stays and passes. Run: `cargo test -p athena-core verify`. Observes: `criteria[1].met` and `commands_verified` are false and `d43_finding` is Some.
  Fails without the behavior: If judge() counted every reused record regardless of tree, passed state or the reuse flag, the different-tree, failed and skipped cases would be met and these assertions would fail.
  Test seam: `verify::judge`, via `judge_recorded` (exists). Fixtures are built by `check()` with overrides. A skipped record may also come from `checks::skipped_run` (exists) if that is reachable from the test module.
- Criterion 3: Inspect records.rs `CheckRun`, which gains `pub reused: bool` with serde default. In checks.rs, extend the test around line 1096 to assert `reused` is true on every record built from a CiRecord and false on `skipped_run` and locally run records. Extend one routing.rs test (reuse path, ~534 or ~672) to assert reused records have `reused` true and locally run or skipped ones false. A Grep of verify.rs for the note text (non-test code) finds nothing. Run: `Grep pattern 'result reused' on crates/athena-core/src/verify.rs (expect no match in judge()); then cargo test -p athena-core runs_from_record  and  cargo test -p athena-core routing`. Observes: The field exists with `#[serde(default)]`, `runs_from_record` and the routing reuse path set it, and judge() has no note-text matching.
  Fails without the behavior: If `runs_from_record` did not set the flag, the assertion `runs.iter().all(|r| r.reused)` in the checks.rs test (~1096) and the routing.rs reuse tests would fail. The unit tests in criterion 1 only prove judge()'s rule; this proves the producer sets the flag.
  Test seam: Production entry points: `checks::runs_from_record` and `routing::run_routed_with_record`. Fixtures: existing CiRecord and fake CiSource/Exec doubles in those test modules (exist). The doubles supply input only, not the logic under test.
- Criterion 4: Two new tests in records.rs, patterned on `a_baseline_written_before_failed_tests_existed_still_parses_and_validates` and `failing_test_names_round_trip_and_are_read_per_check`. (a) Build a Baseline JSON from `sample_baseline()`, assert it does not contain "reused", and parse it back. Also deserialize a literal CheckRun JSON string with no `reused` key via `serde_json::from_str::<CheckRun>`; assert `reused == false`. (b) Set `b.checks[0].reused = true`, `exit_code = None`, `passed = true`, serialize with `AnyRecord::to_json` and assert it contains "reused", `AnyRecord::parse` equals the original, and `check_record_file(<job dir>/baseline.json, bytes)` is Ok. Run: `cargo test -p athena-data`. Observes: The old record loads with `reused` false, and the reused record round-trips and validates.
  Fails without the behavior: Without `#[serde(default)]` the old-shape JSON fails with a missing-field error. Without the field on the struct, `deny_unknown_fields` rejects the serialized reused record, so check_record_file fails.
  Test seam: `AnyRecord::parse`, `AnyRecord::to_json`, `check_record_file` (exist). Fixture: `sample_baseline()` (exists, ~4187). The literal there needs `reused: false` (step 1).
- Criterion 5: Read the new bullet under `## [Unreleased]` / `### Fixed`. Run: `Grep pattern '^- .*reused' on CHANGELOG.md under '## [Unreleased]' / '### Fixed'`. Observes: One bullet states that checks reused from a green CI run for the candidate commit now count as commands run in the verdict. It names D43 and D115, the root cause (judge() counted only records with exit code 0, and reused records carry none), and the new `CheckRun.reused` field, optional in old records.
  Fails without the behavior: Not a behavior criterion.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/status_durations.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 65a1bc0140f6 → candidate 1eb41ee1b3ab. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  8 +++
 crates/athena-cli/tests/status_durations.rs |  1 +
 crates/athena-core/src/checks.rs            | 11 ++++
 crates/athena-core/src/introspect.rs        |  1 +
 crates/athena-core/src/reading.rs           |  1 +
 crates/athena-core/src/routing.rs           |  7 +++
 crates/athena-core/src/run/mod.rs           |  1 +
 crates/athena-core/src/run/send_back.rs     |  1 +
 crates/athena-core/src/run/tests.rs         |  2 +
 crates/athena-core/src/status.rs            |  1 +
 crates/athena-core/src/verify.rs            | 91 ++++++++++++++++++++++++++++-
 crates/athena-data/src/records.rs           | 38 ++++++++++++
 12 files changed, 160 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41305363138 for 1eb41ee1b3ab59a7d503584e82a1dfae294693f4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41305363138 for 1eb41ee1b3ab59a7d503584e82a1dfae294693f4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41305363138 for 1eb41ee1b3ab59a7d503584e82a1dfae294693f4)
- ci-result: `not a check` passed (Check results for 1eb41ee1b3ab59a7d503584e82a1dfae294693f4 come from the CI run https://github.com/doodla/athena/actions/runs/41305363138 (545s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
