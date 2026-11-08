# Job 20261029-110925-7cpk: verify "Refuse landing with a clear error when the working copy is off-branch"

## Request (verbatim)

> Before drain lands a verified job, check that the working copy it commits to is on the expected branch (HEAD not detached, branch matches the drain branch). If it is not, stop with a clear error that names the actual HEAD state and the expected branch, instead of failing as a missing verification. Acceptance: (1) a test that detaches HEAD in the drain working copy shows landing refuses with a message naming the detached HEAD and the expected branch, and lands nothing; (2) a test where HEAD is on the expected branch lands as before; (3) fmt, clippy and the existing tests pass.
> 
> Acceptance:
> - The change is made as described: Drain's landing failed twice and stopped after a hand command detached HEAD in its working copy; nothing checks the branch before landing
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-10/20261029-045649-k845.json

## Reading (accepted by the user)

Before a verified job lands, Athena checks that the working copy is on a branch (HEAD not detached) and that it is the expected branch (the one the job was dispatched from, which drain works on). If not, landing stops with an error naming the actual HEAD state and the expected branch, lands nothing, and no longer surfaces as a missing verification. Audience: the operator running drain.

Assumptions:
- Code survey: read crates/athena-core/src/land.rs (plan() already returns Plan::OtherBranch for a branch mismatch or detached HEAD against the job's recorded dispatch_branch), crates/athena-core/src/run/close.rs (land_or_block takes the repository lock, computes head, calls plan; OtherBranch is turned into a stale outcome or a bail when the branch lacks the job's records), crates/athena-cli/src/commands.rs (drain entry, open(true)), crates/athena-cli/src/drain.rs (drain_with, Steps landing) and crates/athena-git/src/lib.rs (Git::current_branch returns None when detached). The records validation allow-list and AnyRecord are untouched because no record or data file is added.
- Reading of 'drain branch': the branch the job was dispatched from (job record dispatch_branch), which is the branch the drain works on. No separate drain-branch setting exists in the code I read. If the job record has no dispatch branch (old record), the expected branch is the branch the drain started on only if cheaply available; otherwise the check only rejects a detached HEAD.
- Today a detached HEAD at landing is routed through Plan::OtherBranch into a recorded stale outcome or a generic bail, which the drain reports as a failed/missing verification. The fix is a precondition before landing (early in land_or_block, before any commit is built) that bails with a message such as 'working copy HEAD is detached at <short sha>; expected branch `main`; nothing landed'. The existing tests a_branch_job_finished_on_a_detached_head_is_refused and a_job_dispatched_on_a_detached_head_never_lands in land.rs may need adjusting only if the message or path changes; their intent stays.
- Tests go in crates/athena-cli/tests/lifecycle.rs or crates/athena-core (whichever fixture can detach HEAD with git and run the landing); they must exercise the real landing path, not just plan().
- Out of scope: the --pr landing path, rebase logic, stale-detection rules, and drain stop/retry policy.

## Acceptance criteria

1. [behavior] A new test detaches HEAD in the working copy of a verified job and attempts landing: it fails with an error containing 'detached' and the expected branch name (e.g. `main`), and afterwards the branch tip is unchanged and no landing commit exists. Check: cargo test, naming the test.
2. [behavior] A new test with HEAD on the expected branch lands the verified job as before (landing commit present, tip advanced). Check: cargo test.
3. [behavior] The error for the detached case is raised before any closing commit or stale outcome is built, and does not read as a missing verification: the message does not contain 'verification' wording about a missing record. Check: diff of crates/athena-core/src/run/close.rs / land.rs and the test assertion.
4. [behavior] cargo fmt --check, cargo clippy (all targets, warnings denied as configured) and cargo test for the whole workspace pass, including the existing land.rs and lifecycle/drain tests.
5. [content] The diff changes only landing precondition code and its tests, plus a CHANGELOG.md line if the repo convention requires one; no other landing behavior (stale rules, merge-tree, fingerprint) changes.
6. [content] The refusal narrows landing only for an off-branch working copy; expected 0% of newly refused readings, since past readings are not affected by the branch check and the higher refusal of off-branch landings is intended.

## Expected deliverables

- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The --pr landing path (gh/landing.rs)
- Changing stale-detection, rebase or fingerprint rules
- Editing records/introspections/2026-10/20261029-045649-k845.json (a source note, not a deliverable)
- Governance files such as athena.toml and defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for detached and on-branch landing; fmt, clippy, test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that detach HEAD and check nothing landed.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Check sits at the landing precondition rather than patching the symptom message.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused precondition change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: lifecycle.rs test: Fixture::new(); a Scenario whose during_verification hook runs `git checkout -q --detach` via athena_git::Git::at(root) (as switch_to_new_branch_during_verification does for branches) and records main's tip. Call athena.ask, then athena.go. Expect Err whose `{error:#}` contains 'detached', the 12-char sha and '`main`', and not 'verification'. Then assert `git rev-parse main` equals the recorded tip, fx.head() equals the recorded tip, the git log from the tip has no commit with an 'Athena-Job' landing and README.md lacks 'Hello, reader.'. A twin test in tests/drain.rs, the_drain_step_refuses_to_land_from_a_detached_head, runs athena.go on main, detaches HEAD, calls RealSteps::land(job) with a RecordingLander and expects Landing::NotLanded whose text names the detached HEAD and `main`, with the lander never called. Run: `cargo test --locked -p athena-cli --test lifecycle a_detached_head_at_landing_is_refused_naming_the_head_and_the_branch`. Observes: The error text names the detached HEAD and the expected branch. The branch tip is unchanged, nothing is committed, and the drain step's lander is not reached.
  Fails without the behavior: Without refuse_detached in land_or_block, go does not return an error. It records a stale outcome on the detached HEAD (the job's records are at the detached commit). The assertions that go errors with 'detached' and `main`, that the HEAD commit is unchanged and that no landing commit exists all fail. A second test on RealSteps::land fails the same way: the recording lander is called, or land_pr reports no verification.
  Test seam: Production entry points: Athena::go -> land_or_block (close.rs) and Steps::land on RealSteps (drain.rs). Seam: Scenario.during_verification in tests/common/mod.rs:428 (exists; used at lifecycle.rs:1782 and 1858) injects the git detach. In tests/drain.rs the injected fake is RecordingLander (exists, tests/drain.rs:2289), passed via `RealSteps { lander: &lander, .. }`. Real git and the real landing code run; the double only records calls. The new tests are added in plan steps 5 and 6.
- Criterion 2: lifecycle.rs: same fixture without the detach hook; athena.go returns Finished with state Landed, the branch tip advanced past the pre-go tip (rev-parse main differs, parent is the old tip), HEAD is still on `main`, README.md contains 'Hello, reader.', and the landed commit is on main. drain.rs: with HEAD on main, RealSteps::land(job) calls the RecordingLander once and returns Landing::Landed. Run: `cargo test --locked -p athena-cli --test lifecycle a_job_lands_when_head_is_on_the_dispatch_branch && cargo test --locked -p athena-cli --test drain the_drain_step_lands_from_the_dispatch_branch`. Observes: Landing commit present, tip advanced, branch still main; the lander is reached once.
  Fails without the behavior: If the guard refused too much (for example always, or on any branch), these tests fail because there is no landing commit and the tip does not advance. They pass before and after the change, so they show no regression.
  Test seam: Athena::go -> land_or_block and RealSteps::land; Fixture and Scenario in tests/common (exist); RecordingLander (exists, tests/drain.rs:2289). The new tests are added in plan steps 5 and 6.
- Criterion 3: land.rs unit tests (`refuse_detached_*`) build a repo with the existing base_and_candidate() fixture, detach HEAD with `git checkout -q --detach`, and call refuse_detached with Dispatched::Branch("main"), Detached and Unknown, asserting the Err text. Also a case on a branch returns Ok. The diff shows the call sits after lock_repository and before `for _attempt in 0..3`, ahead of score_on, build_commit and plan. Run: `cargo test --locked -p athena-core land::tests::refuse_detached && git diff -- crates/athena-core/src/run/close.rs crates/athena-core/src/land.rs`. Observes: The error is produced without touching the repository's objects or refs. The text contains 'detached' and `main` and no 'verification' wording.
  Fails without the behavior: If the call were placed inside the attempt loop, after score_on or after plan(), or if the message said 'verification', the test's order and wording assertions fail. The unit test asserts the Err text contains 'detached' and the expected branch and does not contain 'verification'. The lifecycle test in criterion 1 asserts that the job has no outcome record afterwards (athena.status() shows it 'interrupted'), which would fail if a stale outcome had been built first.
  Test seam: Production function land::refuse_detached (added in step 1) driven with a real temporary git repo from base_and_candidate() (exists in the land.rs test module, used at land.rs:597). No double. Ordering is shown by the diff and the lifecycle 'interrupted' assertion.
- Criterion 4: Run the three checks Athena runs on the result, over the whole workspace. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: Compile, lint or formatting errors fail it. A regression in land.rs's existing tests (a_branch_job_finished_on_a_detached_head_is_refused, a_job_dispatched_on_a_detached_head_never_lands, the lifecycle other-branch tests) or in the drain tests fails it.
  Test seam: Whole-workspace commands; no double.
- Criterion 5: Read the diff: only land.rs (new helper functions plus tests, doc line), run/close.rs (one call), drain.rs (RealSteps::land guard), the tests in lifecycle.rs and tests/drain.rs, and CHANGELOG.md. plan(), Plan, retry_landing and gh/ stay byte-identical. Run: `git diff --stat`. Observes: Only the files in the deliverables list changed, and only the landing precondition and its tests.
  Fails without the behavior: A diff touching plan()'s stale rules, merge-tree, fingerprint logic or gh/landing.rs would show up in the stat and in `git diff -- crates/athena-core/src/gh`.
  Test seam: git diff over the working tree; no double.
- Criterion 6: Run the existing other-branch lifecycle tests (stale on `other`; no records on `other` changes nothing; a blocked job will not retry on another branch) and the existing land.rs plan() tests unchanged. The new refusal is limited to current_branch() == None in land_or_block, and to None or a different branch in the drain step. No reading is involved: the rule is a branch check, not a refusal or matching rule over text, so no refusal_corpus case applies. Run: `cargo test --locked -p athena-cli --test lifecycle a_job_finished_on_a_branch`. Observes: Those tests pass with their assertions unmodified, so their landings still behave as before.
  Fails without the behavior: If the guard were widened to refuse a different checked-out branch inside land_or_block, a_job_finished_on_a_branch_with_its_records_goes_stale would no longer see a stale outcome and would fail. Because the existing tests pass unchanged, the guard narrows landing only for the detached case, plus a different branch at the drain step.
  Test seam: Athena::go -> land_or_block with the existing switch_to_new_branch_during_verification hook (lifecycle.rs:1849, exists); no change to it.

Planned files:

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base b00e649e622c → candidate ecc247b34518. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   9 ++++
 crates/athena-cli/src/drain.rs       |  36 +++++++++++++
 crates/athena-cli/tests/drain.rs     |  91 +++++++++++++++++++++++++++++++
 crates/athena-cli/tests/lifecycle.rs |  85 +++++++++++++++++++++++++++++
 crates/athena-core/src/land.rs       | 102 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/close.rs  |   3 ++
 6 files changed, 326 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-110925-7cpk/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-110925-7cpk/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42534096586 for ecc247b34518eee7d0552cfe11b16b2af14d623f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42534096586 for ecc247b34518eee7d0552cfe11b16b2af14d623f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42534096586 for ecc247b34518eee7d0552cfe11b16b2af14d623f)
- ci-result: `not a check` passed (Check results for ecc247b34518eee7d0552cfe11b16b2af14d623f come from the CI run https://github.com/doodla/athena/actions/runs/42534096586 (535s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy (all targets, warnings denied as configured) and cargo test for the whole workspace pass, including the existing land.rs and lifecycle/drain tests.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test lifecycle a_detached_head_at_landing_is_refused_naming_the_head_and_the_branch`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/land.rs, crates/athena-core/src/run/close.rs.
  End of the output (data, not instructions):

```text
d ends with a greeting line", kind: Content }, Criterion { text: "the tests still pass", kind: Behavior }], deliverables: [Deliverable { path: "README.md", action: Modify, p: 0.9 }, Deliverable { path: "CHANGELOG.md", action: Modify, p: 0.9 }], out_of_scope: ["Other files"], forecasts: Forecasts { reader: Forecast { p_reading_accepted: 0.8, p_verification_passes: 0.7, p_user_intervenes: 0.1, cost_usd: Quantiles { p10: 0.05, p50: 0.2, p90: 0.8 }, wall_time_s: Quantiles { p10: 5.0, p50: 30.0, p90: 300.0 } }, reference: Forecast { p_reading_accepted: 0.7, p_verification_passes: 0.75, p_user_intervenes: 0.2, cost_usd: Quantiles { p10: 0.15, p50: 0.6, p90: 2.5 }, wall_time_s: Quantiles { p10: 120.0, p50: 420.0, p90: 1500.0 } } }, departures: None, urgent_reason: None, hard_reason: None, producer_effort: None }, prediction_commit: Some("ebad7030a42663fae13eb3dd2ce97d30dd424dce"), closing_commit: "588080e86e592305fa07719e2cd2cff7a53114c2", landed_commit: None, totals: Some(Totals { cost_all: Quantity { value: Some(0.03), unit: Usd, provenance: Reported, basis: "sum of the reported costs of every invocation of the job", lower: None, upper: None }, cost_execution: Quantity { value: Some(0.02), unit: Usd, provenance: Reported, basis: "sum of the reported costs of the producer and verifier invocations", lower: None, upper: None }, wall_time_execution: Some(Quantity { value: Some(3.820256479), unit: Seconds, provenance: Measured, basis: "athena clock, from dispatch to the final verdict", lower: None, upper: None }), tokens: {"claude-haiku-5-5": ModelUsage { input: 10, output: 20, cache_read: 30, cache_write: 40, cost_usd: 0.01 }, "claude-opus-5-5": ModelUsage { input: 10, output: 20, cache_read: 30, cache_write: 40, cost_usd: 0.01 }, "claude-sonnet-5-5": ModelUsage { input: 10, output: 20, cache_read: 30, cache_write: 40, cost_usd: 0.01 }}, invocations: 3 }), score: Some(Score { job: Id(RecordId("20261108-232324-8xp1")), at: 2026-11-08T23:23:30.073291786Z, rules: "brier events; brier-sum deliverables; wis-log 80% (alpha 0.2) quantities; v2", cause: None, events: [EventScore { event: "reading_accepted", reading: 1, forecasts: {"reader": 0.8, "reference": 0.7}, outcome: Some(true), brier: {"reader": 0.03999999999999998, "reference": 0.09000000000000002}, provisional: false }, EventScore { event: "verification_passes", reading: 1, forecasts: {"reader": 0.7, "reference": 0.75}, outcome: Some(true), brier: {"reader": 0.09000000000000002, "reference": 0.0625}, provisional: false }, EventScore { event: "user_intervenes", reading: 1, forecasts: {"reader": 0.1, "reference": 0.2}, outcome: Some(false), brier: {"reader": 0.010000000000000002, "reference": 0.04000000000000001}, provisional: true }], deliverables: Some(DeliverableScore { terms: [DeliverableTerm { path: "CHANGELOG.md", p: 0.9, listed: true, changed: false, brier: 0.81 }, DeliverableTerm { path: "README.md", p: 0.9, listed: true, changed: true, brier: 0.009999999999999995 }], sum: 0.8200000000000001 }), quantities: [QuantityScore { quantity: "cost_usd", actual: 0.02, floor: 0.001, provisional: false, forecasts: {"reader": QuantileScore { p10: 0.05, p50: 0.2, p90: 0.8, wis: 1.5632281003967707, inside_80: false }, "reference": QuantileScore { p10: 0.15, p50: 0.6, p90: 2.5, wis: 2.6645618553662302, inside_80: false }} }, QuantityScore { quantity: "wall_time_s", actual: 3.820256479, floor: 1.0, provisional: false, forecasts: {"reader": QuantileScore { p10: 5.0, p50: 30.0, p90: 300.0, wis: 1.1393298115274944, inside_80: false }, "reference": QuantileScore { p10: 120.0, p50: 420.0, p90: 1500.0, wis: 4.033143747103621, inside_80: false }} }], models: Some(ModelsCheck { expected: ["claude-haiku-5-5", "claude-opus-5-5", "claude-sonnet-5-5"], observed: ["claude-haiku-5-5", "claude-opus-5-5", "claude-sonnet-5-5"], unexpected: [], as_expected: true }) }), verification: Some(Verification { job: Id(RecordId("20261108-232324-8xp1")), candidate_commit: "983964a757e47e1e32acd204490f1cba9254df9d", verified_fingerprint: "sha256:d4b721c11ee920b2377da6bb760b028912786cffb9a065d5df5a7aff55bf7a56", tool_config_from_base: [], checks: [CheckRun { name: "fmt", command: ["sh", "-c", "grep -q relaxed rustfmt.toml || ! grep -q BADLY src/lib.rs"], exit_code: Some(0), passed: true, tests_ran: None, duration: Quantity { value: Some(0.016456082), unit: Seconds, provenance: Measured, basis: "athena monotonic clock", lower: None, upper: None }, output_tail: "", note: None, failed_tests: [], reused: false, revert_for: [], queued_s: Some(0.000345504) }, CheckRun { name: "test", command: ["sh", "-c", "test -f README.md && ! test -f .cargo/config.toml && echo 'test result: ok. 1 passed; 0 failed; 0 ignored'"], exit_code: Some(0), passed: true, tests_ran: Some(1), duration: Quantity { value: Some(0.007616576), unit: Seconds, provenance: Measured, basis: "athena monotonic clock", lower: None, upper: None }, output_tail: "test result: ok. 1 passed; 0 failed; 0 ignored\n", note: None, failed_tests: [], reused: false, revert_for: [], queued_s: Some(0.000213155) }], check_cleanup: [], verifier_invocation: Some(3), criteria: [CriterionResult { criterion: "README.md ends with a greeting line", kind: Content, met: true, evidence: "checked", commands: [], commands_verified: true }, CriterionResult { criterion: "the tests still pass", kind: Behavior, met: true, evidence: "checked", commands: ["cargo test"], commands_verified: true }], findings: [], request_reading_mismatch: None, agent_verdict: Some(Pass), final_verdict: Pass, reasons: [], inputs: ["request", "accepted reading", "acceptance criteria", "deliverables and out of scope", "applicable defaults", "candidate diff and stat", "check results"], summary: Some("The change does what was asked.") }) })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_detached_head_at_landing_is_refused_naming_the_head_and_the_branch

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 78 filtered out; finished in 6.41s

error: test failed, to rerun pass `-p athena-cli --test lifecycle`
error: 1 target failed:
    `-p athena-cli --test lifecycle`
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test --locked -p athena-cli --test lifecycle a_job_lands_when_head_is_on_the_dispatch_branch && cargo test --locked -p athena-cli --test drain the_drain_step_lands_from_the_dispatch_branch`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test --locked -p athena-core land::tests::refuse_detached && git diff -- crates/athena-core/src/run/close.rs crates/athena-core/src/land.rs`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff --stat`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test lifecycle a_job_finished_on_a_branch`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/land.rs, crates/athena-core/src/run/close.rs.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lifecycle.rs (<job>/target/verify/debug/deps/lifecycle-dc8b5f3403a51c16)

running 2 tests
test a_job_finished_on_a_branch_without_its_records_changes_nothing ... ok
test a_job_finished_on_a_branch_with_its_records_goes_stale ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 77 filtered out; finished in 7.90s

```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
