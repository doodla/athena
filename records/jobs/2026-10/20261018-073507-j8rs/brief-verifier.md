# Job 20261018-073507-j8rs: verify "Record which checks already fail on the base commit before producing"

## Request (verbatim)

> Before the producer starts, run the job's checks on the base commit and record which ones already fail, as part of the job's records (part 1 of 2; part 2 uses them at verification). Keep it to recording: no change to how verification judges.

## Reading (accepted by the user)

Before the producer starts, Athena runs the capability's configured checks (fmt, clippy, test) on the base commit and writes a job record listing each check's name, pass/fail and exit code. Only recording is added. The verifier's judgment is unchanged, because part 2 will consume the record. The audience is the maintainer, who reads the job's records, and part 2's verifier.

Assumptions:
- The baseline reuses run_check from crates/athena-core/src/checks.rs on a snapshot of the base commit, with the capability's env and check_timeout_s, and stores the existing CheckRun shape (name, exit_code, passed, tests_ran, output tail, note).
- It is stored as part of the job's records (a new record kind or a field on an existing record), written with the job's other records; the exact schema is the producer's choice, following the pattern of the existing records in crates/athena-data/src/records.rs.
- It is recorded for every capability with configured checks (change and study), taken at the base commit before the first producer invocation; skip_when_only is not applied, since no change list exists yet.
- A baseline check that fails or times out is recorded as failing and never blocks or aborts the job. If the baseline cannot run at all, the job proceeds and the record says so.
- Nothing in verify.rs or routing.rs changes, and verdict logic is untouched. athena.toml and defaults.toml are not changed.
- Changing the records schema may require updating the docs and the record-validation or policy tests that enumerate record kinds.

## Acceptance criteria

1. [behavior] A test run by `cargo test --locked` drives a job with a fake or failing check on the base commit (for example `sh -c 'exit 1'`) and asserts that the job's records contain a baseline entry for that check with passed=false and its exit code, and passed=true for a check that succeeds.
2. [behavior] A test run by `cargo test --locked` shows the baseline is taken before the first producer invocation and from the base commit, not from the candidate. For example, the entry's SHA equals the job's base and its position precedes the producer's invocation record.
3. [behavior] A test run by `cargo test --locked` shows a baseline that fails, times out, or cannot start does not stop the job or change its verdict. The job continues and the record notes the failure.
4. [behavior] The existing verification tests still pass unchanged with no edits to the verdict logic. `git diff` shows no changes to crates/athena-core/src/verify.rs judgment code or to the routing of checks, and `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo fmt --all --check` all pass.
5. [content] The baseline record is part of the job's records: it lists each configured check by name with passed, exit code and a redacted output tail capped at the existing 4 KiB, and it is registered wherever record kinds are validated, so the records check accepts it.
6. [content] The docs or CHANGELOG state in a sentence or two that the baseline is recorded only and that part 2 will use it at verification.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)

## Out of scope

- Using the baseline at verification, or any change to how the verifier judges checks (part 2).
- Changing the checks themselves or their configuration in athena.toml.
- Fixing the checks that already fail on the base.
- Changing the producer's brief or the planner and critic.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is in Rust in the athena-core and athena-data crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover the new recording, and cargo test and clippy stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that a failing base check shows up as failed in the job's records, not only that a type exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Recording only; no consumption by the verifier (that is part 2).
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the design/CHANGELOG note are written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier can run.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a failing base check is recorded as failing.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New lifecycle test with `Fixture::with_config`. Two change checks are configured: `sh -c 'exit 1'` (name `bad`) and `sh -c 'exit 0'` (name `good`). A scenario drives the job through `athena.go`. The test then reads `baseline.json` from the closing commit with the helper that `job_plan.rs` uses (`read_record`). Run: `cargo test --locked -p athena-cli --test lifecycle baseline_records_failing_and_passing_checks`. Observes: The baseline lists `bad` with passed=false and exit_code=Some(1), and `good` with passed=true and exit_code=Some(0).
  Fails without the behavior: Without the recording, baseline.json is absent and the test's read of it panics. If failing checks were recorded as passed, the assert passed=false and exit_code=Some(1) fails.
- Criterion 2: Lifecycle test with a check that fails on the base and passes after the producer's README edit (`grep -q Hello README.md`). It asserts baseline.base_commit == the job's base commit (the candidate's base_commit and `fx.head()` before the run). It asserts baseline.before_invocation == the seq of the producer's invocation record. It asserts the baseline entry has passed=false while the same check in verification.json has passed=true. Run: `cargo test --locked -p athena-cli --test lifecycle baseline_is_taken_from_the_base_before_the_producer`. Observes: The baseline carries the base SHA and the seq of the first producer invocation. The baseline value differs from the candidate's value for the same check.
  Fails without the behavior: If the baseline were taken from the candidate, or after the producer, the base-commit assert or the before_invocation assert fails. The check `grep -q Hello README.md` would also show passed=true instead of false.
- Criterion 3: The lifecycle test has one check that fails on the base but passes after the producer's edit, and one whose command does not exist (`/nonexistent/athena-baseline-check`). It asserts that the producer was invoked, that the baseline notes `could not start` for the second check, and that the job's verdict is decided by the candidate checks as before. The unit tests in checks.rs cover `baseline_runs` for a timeout using a short Duration and `sleep`, a spawn failure, and an Err from `run_check` turned into a failed run with a note. Run: `cargo test --locked -p athena-cli --test lifecycle baseline_failure_does_not_stop_the_job && cargo test --locked -p athena-core checks::tests::baseline`. Observes: The job continues. The baseline notes the failure or the timeout. The final outcome follows the existing verification.
  Fails without the behavior: If a baseline error propagated, the job would end before the producer and the producer-invoked assert fails. If a check that fails on the base changed the verdict, the job would not land.
- Criterion 4: Run the full test, clippy and fmt commands. Inspect `git diff --stat` and `git diff` for crates/athena-core/src/verify.rs and routing.rs. Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check && git diff --stat`. Observes: All existing tests pass unedited. The only diffs are the listed deliverables, with none in verify.rs or routing.rs.
  Fails without the behavior: If the baseline changed what verification or routing sees, an existing lifecycle or verify test would fail. A diff touching verify.rs or routing.rs would be visible.
- Criterion 5: Unit tests in records.rs: a `Baseline` round-trips through `AnyRecord::parse` and `check_record_file` at `<job>/baseline.json`. A misplaced path is refused, and an invalid record (a duplicate check name or an empty base_commit) is refused. A lifecycle test loads `RecordSet::load` on the closing commit and asserts no record errors. The same test uses a failing check that prints a secret-looking canary and more than 4 KiB, and asserts the stored tail is redacted and no longer than TAIL_BYTES plus the ellipsis. Run: `cargo test --locked -p athena-data baseline && cargo test --locked -p athena-cli --test lifecycle baseline`. Observes: baseline.json is accepted by the records check. Each configured check appears by name with passed, exit_code and a redacted, capped tail.
  Fails without the behavior: If the kind were not registered in `AnyRecord`, `canonical_path` and `schema`, `check_record_file` returns a Parse error for baseline.json and the records check reports it as invalid. If the tail were not redacted and capped, the canary test fails.
- Criterion 6: Read the diff of the CHANGELOG entry and the design and record docs. The check is the wording, not a test. Run: `git diff CHANGELOG.md docs/design/athena-v0.md docs/records.md`. Observes: One or two sentences state that the baseline is recorded only and that part 2 will use it at verification. docs/records.md lists baseline.json with its schema.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)
- docs/records.md (modify)

## The change

Base 8d4120b06754 → candidate af220880147f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |   8 ++
 crates/athena-cli/tests/common/mod.rs |  19 +++++
 crates/athena-cli/tests/lifecycle.rs  | 150 ++++++++++++++++++++++++++++++++++
 crates/athena-core/src/checks.rs      |  94 +++++++++++++++++++++
 crates/athena-core/src/run.rs         | 107 +++++++++++++++++++++++-
 crates/athena-core/src/store.rs       |   6 +-
 crates/athena-data/src/records.rs     |  97 ++++++++++++++++++++++
 docs/design/athena-v0.md              |   1 +
 docs/records.md                       |   1 +
 9 files changed, 479 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (941 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
