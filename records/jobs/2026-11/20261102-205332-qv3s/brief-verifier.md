# Job 20261102-205332-qv3s: verify "Hand verifier failing tests and log tail from a red reused CI run"

## Request (verbatim)

> When the checks stage reuses a red GitHub CI run, hand the verifier the names of the failing tests and the end of the failing job's log, so it does not rerun the whole suite to find them.
> Today the build-inputs job's GitHub run failed one test; the verifier spent its whole 2400s budget running every test locally to find which one, and ended with no verdict.
> - When a reused CI run is red, Athena reads the failing shard job's log through the existing gh client and records, on the failed check, the failing test names (cargo's 'failures:' list) and the last lines of output, as a passing local check already records its output tail.
> - The verifier brief shows those names and that tail, and tells the verifier to run only the failing tests (cargo test -p <crate> <name>) unless it has a reason to run more.
> - A test with a fake gh runner returning a red run and a shard log containing a 'failures:' list shows the failed check record carries the names and the verifier input shows them.
> - Changes stay in crates/athena-core (ci_result, gh, run/checks, verification brief) and crates/athena-context briefs; CHANGELOG.md gets an entry.
> - Before planning, list what else reads the check record and the verifier brief, with a test or a reason for each.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion.
> Priority: high

## Reading (accepted by the user)

When the checks stage reuses a red GitHub CI run, Athena reads the failing shard job's log via the gh client and records the cargo 'failures:' test names and the log tail on the failed CheckRun (failed_tests, output_tail). The verifier brief then shows them and tells the verifier to run only those tests (cargo test -p <crate> <name>) unless it has reason to run more. The audience is the verifier, which today runs out of budget.

Assumptions:
- Code survey: read crates/athena-core/src/ci_result.rs (CiRecord/Step/CiRun carry only fixed fields, no log), crates/athena-core/src/routing.rs (run_routed_with_record: resolve() then runs_from_record for a Source::Ci record, so reuse happens before any local fallback; red CI gives Outcome::Failed), crates/athena-core/src/checks.rs (runs_from_record at ~306 builds CheckRun with empty failed_tests and output_tail; failed_tests/output_tail extraction for local runs at ~480-512), crates/athena-core/src/gh/client.rs (failing_test_name(log) at ~154, already used at ~559; CiBranchSource/ci_source ~750-800; parse_ci_run), crates/athena-core/src/run/checks.rs (SnapExec), crates/athena-core/src/run/mod.rs (CheckRun construction ~1537), crates/athena-core/src/verify.rs (failed_tests used for baseline comparison at ~138-160 and ~335), and crates/athena-context/src/briefs.rs (verifier brief, uses output_tail/check_runs).
- Existing: CheckRun in crates/athena-data/src/records.rs already has failed_tests: Vec<String> and output_tail: String, so no new record field or data file is expected; the records validation allow-list and AnyRecord round trip should be unaffected. If a field is added, the producer must cover it with the existing round-trip test.
- Setting failed_tests on a reused red check also feeds verify.rs baseline logic (failing tests that also failed on base), status.rs, reading.rs carried-check output, introspect.rs and run/send_back.rs; each needs a test or a stated reason it is unaffected. This list goes in the CHANGELOG or the producer's plan before planning.
- The shard log is fetched through the existing gh Runner/GhClient (a fake Runner in tests); a failure to fetch or parse the log degrades to today's behavior (failed record without names), never a crash or a pass.
- Log text is data: names and tail are redacted and clipped like local output tails before being recorded.

## Acceptance criteria

1. [behavior] A test in crates/athena-core using a fake gh Runner that returns a red run for the SHA and a failing shard job log containing a cargo 'failures:' list shows the failed CheckRun (via run_routed_with_record) has passed=false, failed_tests equal to the listed names, and a non-empty output_tail holding the last lines of the log. Run: cargo test -p athena-core.
2. [behavior] A test builds the verifier input/brief from that failed check record and asserts the brief text contains the failing test names and the log tail, and the instruction to run only the failing tests with cargo test -p <crate> <name> unless there is a reason to run more. Run: cargo test -p athena-context -p athena-core.
3. [behavior] A test shows that when the shard log cannot be read or has no 'failures:' list, the reused red check is still recorded as failed with empty failed_tests, and a green reused run is unchanged (no log fetch, no names).
4. [content] Changes are confined to crates/athena-core (ci_result, gh, run/checks, routing/checks, verification brief) and crates/athena-context briefs, plus CHANGELOG.md; git diff --stat shows no other paths.
5. [content] CHANGELOG.md has an entry describing that reused red CI runs now give the verifier failing test names and the log tail.
6. [content] The producer's change lists every other reader of the check record and the verifier brief (verify.rs baseline comparison, status.rs, reading.rs carried output, introspect.rs, run/send_back.rs, athena-context outputs.rs/lib.rs, athena-cli tests) each with a test that covers it or a one-line reason it is unaffected; Grep for each of those readers confirms the claimed handling.
7. [behavior] Existing tests for local check output tails and for failed_tests baseline comparison in verify.rs still pass, showing the new data does not change verdict logic for local runs. Run: cargo test -p athena-core verify.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Fetching logs for green runs or for non-test failures such as fmt and clippy beyond recording their log tail where trivial
- Changing the verifier's time budget or the verdict rules
- Changing the CI workflow files or athena.toml
- Adding a new record type or data file

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the red-run record and the brief; existing tests stay green.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check names and tail appearing in the record and brief, not that wiring exists.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change inside the named crates.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Log text is redacted before recording.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The verifier gets the data at the source (the reused red run) instead of a longer budget.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier exercises the new tests with cargo test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Behavior is enforced by tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test `red_reused_run_records_failing_tests_and_log_tail` in gh/tests.rs. It calls the production `crate::routing::run_routed_with_record`, with the production `GhClient::new(&runner, ..).ci_source(CI_BRANCH)` as the `ci` argument. The route covers fmt, clippy and test, and a stub `Exec` records any call and implements `redact` by calling a real Redactor. The fake `gh::Runner` (the existing `CiRunner`, extended in step 9) answers the runs query with a red run for the SHA and answers the jobs query with failed `test (shard 1)`, whose id is 77, plus a green fmt and clippy. It answers `actions/jobs/77/logs` with timestamped cargo output: a `== shard 1/6: athena-core --lib` line, a `Running unittests ... (target/debug/deps/athena_core-abc)` line, an indented `failures:` list of two tests, and `test result: FAILED. 1 passed; 2 failed`. Guards and preconditions: the record is not all-passed and the `test` check failed. The test asserts that the `test` CheckRun has passed=false and reused. It asserts failed_tests == ["athena_core tests::a", "athena_core tests::b"] and that output_tail starts with `failing units: athena-core --lib` and ends with the last log line with no timestamps. It asserts fmt and clippy have empty names, the stub Exec ran nothing, and the returned record is Some. Run: `cargo test -p athena-core red_reused_run_records`. Observes: The failed `test` check carries exactly the listed names and the log tail with the units header; no local check ran; fmt and clippy are untouched.
  Fails without the behavior: Without the `failure_detail` call and attach, failed_tests is empty and output_tail is "", so the equality and non-empty assertions fail. Without the log fetch, the fake is never asked for `actions/jobs/77/logs`.
  Test seam: Entry point `routing::run_routed_with_record` via `GhClient::ci_source`. The fake `gh::Runner` is injected through `GhClient::new(&runner, ..)`; `CiRunner` exists in gh/tests.rs and step 9 extends it with job ids, a logs reply and an Err mode. The stub `Exec` is a new test struct, and the production `Exec::redact` is added in step 5. The parsing, tail and marker logic is production code; the fake only returns canned text.
- Criterion 2: Two tests. (a) In crates/athena-core/src/run/tests.rs, add `CiReply::RedTest` to the `CiReplies` fake: a failed `test (shard 0)` job with an id and a stamped log. The fake answers the `actions/jobs/<id>/logs` path before its `path.contains("/jobs")` branch. Then `ci_reuse_run(Some(GITHUB), CiReply::RedTest)` drives `Athena::go` through `with_ci_source`, `ci_seam` and `check_seam`. It asserts that `run.verifier_prompts.last()` contains the failing names, a tail line, the failing-units line, `cargo test -p <crate> <name>`, the unless-there-is-a-reason-to-run-more sentence, and the explanation of the `<binary> <test>` form. It asserts no local check ran and that `run.checks()` carries the names. (b) In crates/athena-context/src/briefs.rs, a test calls `verifier_brief` with a CheckResultView that has failed_tests (one marker entry among them), failing_units and output_tail, and asserts the section, instruction, mapping text and incomplete-list warning. A second test with empty new fields asserts the brief is byte-identical to the old rendering and has no instruction. A verify.rs test calls `check_result_view` on a reused failed CheckRun and on a local failed CheckRun, and asserts the fields are filled only for the first. Run: `cargo test -p athena-context -p athena-core verifier`. Observes: The verifier prompt shows the failing names, the units, the log tail and the run-only-the-failing-tests instruction for a reused red run, and nothing new for local runs.
  Fails without the behavior: Without the new view fields and the briefs.rs rendering, the prompt lacks the names and tail and the `contains` assertions fail. Without `check_result_view` filling the fields for reused failed checks, the end-to-end prompt lacks them.
  Test seam: Entry points `Athena::go` (via `ci_reuse_run`), `verify::check_result_view` (added in step 8) and `athena_context::briefs::verifier_brief`. Fakes: `ci_seam::install`, `check_seam::install` and the verifier script capturing `spec.user_prompt` all exist in run/tests.rs. The `RedTest` reply and the logs-path ordering are added in step 10. The verifier does run after a red check (run/verification.rs:64-93 does not return early), and the test asserts `verifier_prompts` is non-empty.
- Criterion 3: Tests named `reused_red_degrades_*` in gh/tests.rs using the same fake Runner and `run_routed_with_record`: (a) the logs call returns Err for the only failed shard: `test` is failed and reused, failed_tests is empty, output_tail is empty, and there is no panic. (b) The log has no `failures:` list: failed with empty failed_tests (a tail may be recorded). (c) Two failed shards, shard 1 with names and shard 2's log returning Err: failed_tests holds shard 1's names plus `test (shard 2) (log not read)`. A timed_out shard adds `(did not finish)`. More than 50 names yields 50 plus `(list cut at 50)`. A verify.rs test, `reused_red_incomplete_list_is_not_pre_existing`, uses a Baseline that lists shard 1's names and asserts `is_pre_existing` is false for the check with the marker and true for the same names without it. (d) A green reused run: the fake records every request path and none contains `/logs`. A routing.rs fake CiSource whose `failure_detail` counts calls shows zero calls. (e) An Exec whose `redact` errs: the detail is dropped and the check stays plain failed. Run: `cargo test -p athena-core reused_red`. Observes: An unreadable or list-less log leaves today's plain failed record. A partly read set of shards is flagged with a marker so it is never judged pre-existing. A green run makes no log fetch.
  Fails without the behavior: Without the degrade path, an Err from the log read would propagate or panic. Without the markers, case (c) would hold only shard 1's names, and the verify.rs assertion that the check is not pre-existing would fail. Without the `all_passed` guard, the counter in (d) would be above zero.
  Test seam: Entry point `run_routed_with_record`; `verify::is_pre_existing` for the baseline assertion. Fakes: the `gh::Runner` fake with an Err mode and multi-shard jobs (CiRunner extended in step 9), a counting CiSource in the routing.rs test module (extends the existing `Ci`; added in step 5), and the erring Exec test struct. They are injected through `GhClient::new` and the `ci` and `exec` arguments. The marker logic is production code in `detail_from_job_logs`; the fakes return only canned replies.
- Criterion 4: After the producer finishes, inspect the diff paths. Run: `git diff --stat`. Observes: Only crates/athena-core (ci_result.rs, gh/client.rs, gh/mod.rs, gh/tests.rs, checks.rs, routing.rs, run/checks.rs, run/tests.rs, verify.rs, and run/verification.rs, the one wiring file outside the deliverable list, justified above), crates/athena-context/src/briefs.rs and CHANGELOG.md appear. No data file, workflow, athena.toml or athena-data path.
- Criterion 5: Read the CHANGELOG.md [Unreleased] diff. Run: `git diff CHANGELOG.md`. Observes: An entry under [Unreleased] states that a reused red CI run now gives the verifier the failing test names and the log tail. It names the root cause (the red record carried no log data), explains the marker rule and the stricter test-by-test comparison, and includes the reader audit of criterion 6.
- Criterion 6: The CHANGELOG entry and the producer's summary list each reader with a test or a one-line reason. The verifier checks each with Grep over crates/ for `output_tail|failed_tests|CheckResultView`. Readers. verify.rs `is_pre_existing`, `failure_split` and the judge: new tests (names all on the base stay pre-existing; one new name or a marker makes it the job's own failure; empty names keep the check-name rule), and the doc comment is updated. checks.rs `log_entry` and `failing_names`: a new unit test shows the ui line now names the failing tests of a reused red check. status.rs: only constructs CheckRun, never reads the fields (unaffected). reading.rs carried-check output: already clipped by `tail(.., CARRIED_CHECK_OUTPUT_BYTES)`, so a longer tail cannot grow it. introspect.rs: already `clip_tail(.., TAIL_CAP)`. run/send_back.rs: constructs only. athena-context outputs.rs and lib.rs: Grep finds no `output_tail` or `failed_tests` outside briefs.rs. athena-cli tests: lifecycle.rs asserts the local-run tail bound (local runs unchanged) and status_durations.rs only constructs. The producer re-confirms each claim by reading the line before writing it, and the briefs tests cover the verifier brief reader. Run: `cargo test -p athena-core is_pre_existing`. Observes: The baseline-comparison tests for a reused red check with names, with a marker and with empty names pass. Grep matches the claimed read-only or construct-only behavior for each listed reader.
  Fails without the behavior: Without the new verify.rs tests, nothing pins that a reused red check with names is compared test by test. Before this change an empty-name reused red check on a base-red check was pre-existing wholesale.
  Test seam: Entry points `verify::is_pre_existing` and `verify::failure_split`. Fixtures are CheckRun and Baseline literals; the verify.rs test helpers `reused_from_ci` and the baseline builders exist (read at verify.rs tests). No fake is needed.
- Criterion 7: Run the existing verify and local-tail tests unchanged. No assertion is edited; the only edits are the two or three new fields on literals where the compiler needs them, and Default on CheckResultView. Run: `cargo test -p athena-core verify`. Observes: Existing verify.rs baseline-comparison and verdict tests pass, and the checks.rs tests for local failed and passing tails pass. Local runs are untouched: `run_check` is not edited, and the view fields are filled only for reused failed checks.
  Fails without the behavior: If the change altered failed_tests handling for local runs, for example by always filling the view fields or by changing `is_pre_existing` for non-reused checks, these existing tests would fail.
  Test seam: Existing tests; no new seam.

Planned files:

- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 55530e9517d7 → candidate dff7d9aaa7fb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  26 +++
 crates/athena-context/src/briefs.rs        | 153 ++++++++++++-
 crates/athena-core/src/checks.rs           |  92 ++++++++
 crates/athena-core/src/ci_result.rs        |  22 +-
 crates/athena-core/src/gh/client.rs        | 221 +++++++++++++++++-
 crates/athena-core/src/gh/mod.rs           |   4 +-
 crates/athena-core/src/gh/tests.rs         | 357 +++++++++++++++++++++++++++++
 crates/athena-core/src/routing.rs          | 137 ++++++++++-
 crates/athena-core/src/run/checks.rs       |   4 +
 crates/athena-core/src/run/tests.rs        |  74 +++++-
 crates/athena-core/src/run/verification.rs |  14 +-
 crates/athena-core/src/verify.rs           | 129 ++++++++++-
 12 files changed, 1204 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41591821894 for dff7d9aaa7fb3e80a876c448a4bf200fccc65d85)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41591821894 for dff7d9aaa7fb3e80a876c448a4bf200fccc65d85)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41591821894 for dff7d9aaa7fb3e80a876c448a4bf200fccc65d85)
- ci-result: `not a check` passed (Check results for dff7d9aaa7fb3e80a876c448a4bf200fccc65d85 come from the CI run https://github.com/doodla/athena/actions/runs/41591821894 (605s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
