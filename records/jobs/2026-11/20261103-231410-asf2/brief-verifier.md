# Job 20261103-231410-asf2: verify "Run planned 'fails without the fix' demos and show them to the verifier"

## Request (verbatim)

> Let the verifier see the 'fails without the fix' run. Many acceptance criteria require a recorded run showing the new test fails on the old code, but the verifier receives no producer report and may not change files, so it can never see that run. Two tries of 'don't count needs-accepting re-reads toward the job limit' (jobs 20261103-193428-df2m and 20261103-222737-zzsr) were rejected only on this, although the code was correct and all checks passed. Fix it in one place: after the producer finishes, Athena itself runs each planned 'fails without the fix' demonstration (apply the planned revert to a scratch copy of the candidate, run the named test, record the command, exit code and output tail) and puts that record in the verifier's brief under 'Checks Athena ran'. Acceptance: a test where a plan has one such demonstration and the verifier brief contains its recorded failing run; a test where the revert does not make the test fail and the record says so.

## Reading (accepted by the user)

After the producer finishes, Athena runs each planned demonstration that has a fails_without on a scratch copy of the candidate with the fix reverted, and records command, exit code and output tail under 'Checks Athena ran on this tree' in the verifier brief. Goal: stop correct changes (retries of jobs 20261103-193428-df2m and 20261103-222737-zzsr) failing because the verifier can never see the failing run; audience is the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/run/verification.rs (verification_stage builds VerifierInputs.checks from check_runs plus extra 'not a check' entries, then verifier_brief_with_plan), crates/athena-context/src/briefs.rs (line ~1140 renders 'Checks Athena ran on this tree' from inputs.checks), crates/athena-core/src/admission.rs (a behavior criterion's demonstration must set fails_without, free text) and crates/athena-core/src/run/tests.rs (plan fixtures with fails_without). The new run happens before the verifier starts; its result is measured directly by running the named test command in a scratch copy, not inferred. No record or data file is added under crates/, so check_record_file/AnyRecord need no change.
- The 'planned revert' is read as: in a scratch copy of the candidate, restore the base version of the candidate's changed non-test files, keep the new tests, run the demonstration's command, and record command, exit code and output tail. If the plan's demonstration has no fails_without, nothing runs. No new plan field or planner prompt change (the planner/verifier prompts have almost no token headroom).
- The record is added as an extra checks entry like 'criteria-left-to-athena-checks' (not a check, fails nothing); it says whether the test failed (expected) or passed (revert did not make it fail), and the final verdict still rests on the recorded fmt/clippy/test checks.
- Prior jobs: 20261103-193428-df2m and 20261103-222737-zzsr changed drain.rs, tests/drain.rs, main.rs and CHANGELOG.md for the needs-accepting job-limit behaviour. Their code was judged correct; their only finding (missing failing run) is addressed here by this fix, and re-running that work is a separate follow-up request, so those four files are excluded in out_of_scope (work not kept in this job).
- Scope kept to one deliverable (Athena running and briefing the demonstration); wall time forecast stays under verifier_timeout_s of 2400s.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core` (filtered to the new test): a test where a plan has one demonstration with fails_without and the reverted copy makes the named test fail; the verifier brief it builds contains that demonstration's recorded run (command, non-zero exit code, output tail) under 'Checks Athena ran on this tree'.
2. [behavior] Run `cargo test -p athena-core` (filtered to the new test): a test where reverting does not make the named test fail; the brief's record says the test passed without the fix, and it is not presented as a failing run.
3. [behavior] Run `cargo test -p athena-context`: briefs.rs renders the demonstration record in the brief; a rendering test checks the text appears with command, exit code and output tail.
4. [content] The demonstration runs in a scratch copy: Grep/diff of verification.rs (or the checks stage) shows the candidate tree and the candidate commit are not modified, and the scratch copy is removed afterwards.
5. [content] The record is an entry that fails nothing ('not a check'), so the final verdict still rests on the recorded fmt, clippy and test checks.
6. [behavior] Outcome-check/tested: `cargo fmt --check`, `cargo clippy` and `cargo test` pass as recorded under 'Checks Athena ran on this tree'.
7. [content] CHANGELOG.md has an entry saying the verifier now sees Athena's recorded 'fails without the fix' run.

## Expected deliverables

- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Re-doing the needs-accepting job-limit work of jobs 20261103-193428-df2m and 20261103-222737-zzsr: crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs and crates/athena-cli/src/main.rs are not changed here; that is a follow-up request once this lands.
- Giving the verifier a way to change files or run reverts itself.
- Passing the producer's report to the verifier.
- Adding a machine-readable revert field to plans or changing planner/critic/verifier prompts.
- Changes to athena.toml, defaults.toml, workflows or other governance files.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for both the failing-run and not-failing cases; fmt, clippy and test pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria are checked by cargo test runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change; drain work deferred.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the verifier's blind spot once, rather than per job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the record appearing in the brief, not wiring.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome is the verifier seeing the failing run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and comments written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by Athena running the demonstration, not by prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Job-level test through `Athena::go`: plan from `behavior_plan(seam)` (run/tests.rs:668) with command `cargo test --locked demo` and a fails_without. The fake producer (base_script, run/tests.rs:215) edits README.md from '# Demo' to a longer text. The test installs `demo_seam` mapping the parsed argv to an `sh -c` script that prints 'test result: FAILED. 0 passed; 1 failed' and exits 1 when README.md in the scratch copy still equals the base text, and prints 'test result: ok. 1 passed; 0 failed' and exits 0 otherwise. The candidate would pass; the reverted copy fails. Run: `cargo test -p athena-core a_planned_fails_without_demonstration_is_run_on_a_reverted_copy_and_shown_to_the_verifier`. Observes: The prompt captured for Role::Verifier contains, under '## Checks Athena ran on this tree', an entry `fails-without-1` with the command, 'exit code 1', the sentence 'failed without the fix', the reverted file README.md and the output tail ('1 failed'). The job is Landed, `<job_dir>/revert` no longer exists, and `show_file(candidate, "README.md")` is still the producer's text.
  Fails without the behavior: Without the stage, the verifier prompt has no `fails-without-1` entry and the text assertion fails. If the revert did not happen, README.md would still hold the candidate text, the script would exit 0 and the 'exit code 1' assertion would fail.
  Test seam: Production entry point `Athena::go` -> `check_and_verify` -> `verification_stage` -> new `Athena::demonstration_records` (run/checks.rs), guarded by `self.nested.is_none()`, a plan demonstration with a non-blank fails_without, a plain `cargo test` command and an installed mapper. Fixtures `repo_with_planning` (tests.rs:283), `run_planned_behavior` (tests.rs:632), `behavior_plan` (tests.rs:668) and the recording planning script exist. `demo_seam::install` (cfg(any(test, feature="test-support"))) is added by plan step 4 and injected by the test as `check_seam` is today. The scratch copy, revert, process and classification are real code.
- Criterion 2: Same job-level entry point and fixtures, with `demo_seam` mapping the argv to `sh -c 'echo "test result: ok. 1 passed; 0 failed"'`, which exits 0 on the reverted copy. Unit tests in verify.rs (`demonstration_view_*`) cover the other classes: exit 0 with 0 tests ('ran no tests'), non-zero with 'could not compile', no exit code ('did not finish'), and a case where the tail holds only trailing '0 passed' summaries but `tests_ran` is Some(1) (still 'passed without the fix'). Run: `cargo test -p athena-core a_demonstration_that_still_passes_without_the_fix_is_recorded_as_passing`. Observes: The verifier prompt has the `fails-without-1` entry with 'exit code 0' and 'passed without the fix: the revert did not make the test fail', and does not contain 'failed without the fix'. The job still lands, because the record fails nothing.
  Fails without the behavior: Without the classification the entry would be shown as a failing run or without a sentence and the 'passed without the fix' assertion fails. A classifier that read the tail instead of `tests_ran` fails the tail-outside unit test.
  Test seam: Same entry point, fixtures and `demo_seam` as criterion 1 (step 4). `verify::demonstration_view` is real code (step 3); no double contains it.
- Criterion 3: Rendering tests in briefs.rs call `verifier_brief_with_plan` with `VerifierInputs.checks` holding (a) an entry with a `demonstration` sub-record (criteria [1, 2], command `cargo test --locked demo`, exit code 101, reverted files, a `tests_removed` file, a multi-line tail containing a ``` fence) and (b) one with exit code 0; a third test checks an old-style entry (`..Default::default()`) renders exactly as before. Run: `cargo test -p athena-context demonstration_record`. Observes: Under '## Checks Athena ran on this tree' the text has the command in backticks, 'exit code 101', the result sentence, the criteria, the reverted files, the 'tests inside these files were removed with the revert' line, and the clipped tail in a fenced block (inner fence neutralised by `clip_log_tail`). The entry prints neither 'passed' nor 'failed' as its own status. The existing briefs.rs tests (the full literal ~3273 and ~3445) still pass.
  Fails without the behavior: Without the rendering branch the exit code and tail are absent and the assertions fail.
  Test seam: Production entry point `athena_context::briefs::verifier_brief_with_plan` (exists, ~briefs.rs:1075). No double: the test builds the input struct directly. `DemonstrationRunView` and the `demonstration` field are added by step 1.
- Criterion 4: Content check of the diff plus the criterion 1 test, which asserts that after the run `<job_dir>/revert` is gone and the candidate commit still holds README.md as the candidate wrote it. A unit test of the outer fn with an inner error (the scratch directory cannot be made) shows 'not run' and no leftover directory. Run: `git diff -- crates/athena-core/src/run/checks.rs crates/athena-core/src/run/verification.rs && cargo test -p athena-core a_planned_fails_without_demonstration_is_run_on_a_reverted_copy_and_shown_to_the_verifier`. Observes: The only writes are inside the directory created by `verifier_snapshot(&job_dir.join("revert"), ..)`; `pin::remove_tree` runs in the outer function on every path (as in `take_baseline`); no git write runs against the job repository or the candidate.
- Criterion 5: Content check of the diff plus the criterion 1 and 2 tests, which land the job with a failing demonstration (exit 1) and a passing one (exit 0). Run: `git diff -- crates/athena-core/src/run/verification.rs && cargo test -p athena-core a_demonstration_that_still_passes_without_the_fix_is_recorded_as_passing`. Observes: The entries are pushed only to `inputs.checks` with `command: "not a check"`, `passed: true`; not to `check_runs`, `Verification.checks` or `VerdictInputs.checks`; `judge` is not edited. The criterion 1 job lands although the demonstration exits 1.
- Criterion 6: The recorded Athena checks on the final tree. The producer also runs `cargo test -p athena-core --test refusal_corpus` and `cargo test -p athena-cli` as no-regression checks. The new unit tests include `revert_keeping_tests` cases: fix plus inline `mod tests`; a base with its own old test module (replaced); no test module; an unparsable file (None, whole-file fallback); a candidate with a bodiless `#[cfg(test)] mod x_tests;` whose file is kept and whose declaration is carried over; a declared module with a non-test file name that is kept; an added file with inline tests (listed in tests_removed); the `is_test_path` forms. A job-level test `a_planned_job_without_a_demo_runner_records_no_demonstration` (no mapper installed, plan with fails_without, command `cargo test --locked`) lands and its verifier prompt has no `fails-without` entry. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three pass; the existing tests.rs:861 job, which uses `behavior_plan`, and the athena-cli integration tests that use the fake planner behave as before.
  Fails without the behavior: A compile error, a clippy warning or a failing test fails the matching check; a `revert_keeping_tests` that dropped the candidate's inline module or declaration fails its unit test; a stage that ran real cargo with no mapper would make `a_planned_job_without_a_demo_runner_records_no_demonstration` find an entry and fail.
  Test seam: Whole workspace, no double. The `demonstration` field has a Default; the one full literal in briefs.rs (~3273) is updated in step 1. The no-mapper test uses existing fixtures and installs nothing.
- Criterion 7: Content check of CHANGELOG.md. Run: `git diff -- CHANGELOG.md`. Observes: Under '## [Unreleased]' / '### Added' an entry says the verifier now sees Athena's recorded 'fails without the fix' run: Athena reverts the fix in a scratch copy of the candidate (keeping tests, including inline and declared test modules), runs the named test once per distinct command, and shows command, exit code and output tail under 'Checks Athena ran on this tree'. It names the root cause: the verifier gets no producer report and cannot change files.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1c848ebf7b5a → candidate c4d7cdd2d0ca. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  10 +
 crates/athena-context/src/briefs.rs        | 170 ++++++++
 crates/athena-core/src/code_health.rs      |  49 +++
 crates/athena-core/src/run/checks.rs       | 239 +++++++++++
 crates/athena-core/src/run/tests.rs        | 215 ++++++++++
 crates/athena-core/src/run/verification.rs |  13 +
 crates/athena-core/src/verify.rs           | 654 ++++++++++++++++++++++++++++-
 7 files changed, 1348 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41776051549 for c4d7cdd2d0caf5807822fc784e4488547621a5cd)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41776051549 for c4d7cdd2d0caf5807822fc784e4488547621a5cd)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41776051549 for c4d7cdd2d0caf5807822fc784e4488547621a5cd)
- ci-result: `not a check` passed (Check results for c4d7cdd2d0caf5807822fc784e4488547621a5cd come from the CI run https://github.com/doodla/athena/actions/runs/41776051549 (618s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Outcome-check/tested: `cargo fmt --check`, `cargo clippy` and `cargo test` pass as recorded under 'Checks Athena ran on this tree'.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
