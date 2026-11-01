# Job 20261101-185259-36ae: verify "Wait for pending checks before merging; treat rule-violation refusal as"

## Request (verbatim)

> The drain asked GitHub to merge the rebase-before-landing pull request while its history check was still running. GitHub refused with 'Repository rule violations found', the drain counted the landed job as failed and stopped the run. A later 'athena land --pr' on the same job merged it cleanly once every check had finished.
> - Before requesting a merge, wait while any check run on the pull request head is still queued or in progress; only a failed required check is a failure, and a failed optional check stays a note.
> - A refused merge with 'Repository rule violations' while checks are pending is a wait, not a failure: the drain reports the job as waiting to merge and does not stop the run.
> - A gh test: a pull request with the required check green and another check in progress is reported as pending, and no merge call is made.
> - A drain test: a merge refused for rule violations is reported as waiting, and the run goes on to the next job.
> - CHANGELOG.md gets an entry.
> Priority: urgent

## Reading (accepted by the user)

The drain asked GitHub to merge while a non-required check run was still in progress, was refused with 'Repository rule violations', and stopped the run. The change makes check classification report pending while any check run on the head is queued or in progress, and makes a rule-violation merge refusal a wait: the drain reports the job as waiting to merge and moves to the next job. The audience is the person running the drain.

Assumptions:
- Code survey: crates/athena-core/src/gh/client.rs (classify_checks decides only on the required CI_AGGREGATE_JOB run, so an in-progress optional run reads Green); crates/athena-core/src/gh/early.rs (wait_for_checks polls check_report until not Pending); crates/athena-core/src/gh/landing.rs (land_pr calls host.merge only when merge_via_rest && checks == Green, and a merge error propagates as Err); crates/athena-cli/src/drain.rs (PrLander::land maps Err to Landing::NotLanded, which the drain counts as a failed landing). Property is measured directly via classify_checks output and via the Landing value PrLander returns.
- No record or data file under crates/ is added, so records validation (check_record_file, AnyRecord) is not touched.
- Rule-violation detection matches the error text 'Repository rule violations' from the host merge call; it is converted into a pending report (merge not confirmed) in land_pr, and PrLander/drain reports a waiting-to-merge outcome instead of a failure. The exact enum shape (a new Landing variant or a flag on PrReport) is the producer's choice, consistent with how the drain handles other not-landed outcomes.
- Check runs with status queued or in_progress count as pending; completed runs with failure conclusions stay: required failure is Red, optional failure is a note only.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [behavior] A gh test in crates/athena-core/src/gh/tests.rs: check runs with the required CI_AGGREGATE_JOB completed green plus another check run in_progress classify as Checks::Pending, and a land_pr run against the fake host makes no merge call (merge_requested false). Run `cargo test -p athena-core gh::`.
2. [behavior] A classify test shows a failed optional check run (completed, failure) with the required check green is still Green with a note, and a failed required check is Red. Run `cargo test -p athena-core gh::`.
3. [behavior] A gh test: when the fake host's merge returns an error containing 'Repository rule violations' while checks are not all complete, land_pr returns a pending/waiting result rather than Err. Run `cargo test -p athena-core gh::`.
4. [behavior] A drain test in crates/athena-cli/src/drain.rs: a merge refused for rule violations is reported as waiting to merge (not a failed landing), the run does not stop, and the next job is worked. Run `cargo test -p athena-cli drain`.
5. [content] CHANGELOG.md has a new entry describing waiting on pending check runs and treating a rule-violation merge refusal as waiting.
6. [behavior] `cargo test` passes and `cargo clippy --all-targets` and `cargo fmt --check` are clean.
7. [content] Doc comments on classify_checks and wait_for_checks are updated to state that any queued or in-progress check run makes the verdict pending.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Reading required checks from branch protection instead of the fixed CI_AGGREGATE_JOB name.
- Changing the re-run logic for red required checks.
- Changes to athena.toml, defaults.toml or workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New gh and drain tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing pending classification, no merge call, and the drain continuing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes both the premature merge request (pending runs ignored) and the drain's handling of the refusal.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across gh and drain.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier runs the named tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Test A: call classify_checks with check_runs [required run completed/success, a second run in_progress] and assert checks == Pending. Test B: drive land_pr through the existing Rig::run helper against a FakeHost that carries those check-run JSON replies. A new FakeHost field, set by a new constructor, makes check_report return classify_checks(runs, status). Assert host.merges == 0, report.merge_requested is false, report.checks == Some(Pending) and the link stays Open, with merge_via_rest on. Run: `cargo test -p athena-core gh::`. Observes: classify_checks returns Pending for a green required check plus an in-progress run. land_pr with merge_via_rest on makes no merge call.
  Fails without the behavior: Without the not-completed rule in classify_checks the in-progress optional run reads Green, FakeHost::merge is called (merges == 1, merge_requested true) and both assertions fail.
  Test seam: Production entry points are gh::classify_checks and gh::land_pr, via Rig::run (exists, tests.rs ~1030). The fake is FakeHost (exists, tests.rs 836), which implements PrHost. Plan step 5 adds a `runs: RefCell<Option<Value>>` field and a `check_report` override on FakeHost. The override only calls the production classify_checks, so no classification logic lives in the double.
- Criterion 2: Extend the existing test only_the_required_check_decides_and_optional_failures_become_notes (tests.rs 709), or add a sibling. Assert classify_checks([required success, head_run failure]) is Green with one note naming 'pull request head' and 'failure'. Assert [required failure, an optional completed run] is Red. Assert [required failure, an optional in_progress run] is still Red. Run: `cargo test -p athena-core gh::`. Observes: Verdicts and notes returned by classify_checks.
  Fails without the behavior: If optional failures stopped being notes the Green assertion fails. If a required failure stopped being Red the Red assertion fails. The 'not completed' rule must not break these.
  Test seam: Production gh::classify_checks called directly. Existing helpers required_run and head_run (tests.rs 699-706) are the fixtures. The seam exists.
- Criterion 3: Add a FakeHost field `merge_error: RefCell<Option<String>>`, used by merge() in place of the fixed refusal text. Positive case: checks Green, script [Green, Pending], merge_error 'Repository rule violations found'; call rig.run(&host, true, ...) and assert Ok, report.checks == Some(Pending), host.merges == 1, report.link.state == Open, and a warning mentioning the refusal. Negative case: script [Green, Green] with the same error text must still return Err. Run: `cargo test -p athena-core gh::`. Observes: land_pr returns Ok with a Pending report in the first case and an Err in the second.
  Fails without the behavior: Without the rule-violation branch land_pr returns Err from host.merge, so the Ok/unwrap in the test fails. Without the re-read, the Green-then-Green negative case would be treated as a wait.
  Test seam: Production gh::land_pr via Rig::run (exists). FakeHost.script (exists, tests.rs 851) supplies the second read. Plan step 5 adds merge_error.
- Criterion 4: Test 1, in drain.rs `mod tests` (the World fixture, line 3744): World gets `waiting: BTreeSet<String>`, set by a new `.waits(job)` builder, and World::land returns Landing::Waiting for those jobs. Use independent items a and b with disjoint paths and no dependency. Run `concurrent(&world, 2)` (which calls drain_concurrent). Assert report.stop == Stop::QueueEmpty, not RepeatedFailure. Assert world.count("land a") == 1 (no retry). Assert world.logged("land b") and report.landed == 1. Assert the job line for a has landing starting 'waiting to merge'. Test 2: landing_from_report on a PrReport (built like display.rs's report() helper) with an Open link, checks Some(Pending) and merge_requested true gives Landing::Waiting. The same report with checks Red gives NotLanded, and Err gives NotLanded. Run: `cargo test -p athena-cli drain`. Observes: Report.stop, report.jobs[..].landing text, the log order and landed counts.
  Fails without the behavior: Without Landing::Waiting and ItemEnd::Waiting, a refused landing is counted as a failed attempt. The item is retried, the run ends with Stop::RepeatedFailure and b never lands, so every assertion fails.
  Test seam: Production entry points are drain::drain_concurrent (via the `concurrent` helper, exists) and landing_from_report (new, plan step 7). The fake is World (exists, drain.rs 3744), injected as `&dyn Steps`. Step 8 adds the `waiting` set and the .waits builder. The fake only returns the scripted Landing; the drain's reaction to it is production code.
- Criterion 5: Add an entry under ## [Unreleased] ### Fixed (CHANGELOG.md line 10). It says the drain now waits while any check run on the pull request head is queued or in progress, and treats a 'Repository rule violations' merge refusal while checks are pending as 'waiting to merge'. It names the root cause: classify_checks looked only at the required run. Read the entry; a Grep with the Grep tool for 'waiting to merge' and 'Repository rule violations' in CHANGELOG.md is the check. Run: `grep -n "Repository rule violations" CHANGELOG.md`. Observes: The entry is present under Unreleased and describes both behaviors.
  Fails without the behavior: Without the entry the grep finds nothing.
  Test seam: Content check, no test seam. If the verifier cannot run grep, use the Grep tool on CHANGELOG.md.
- Criterion 6: Run the three commands separately: cargo fmt --all --check; cargo clippy --all-targets --locked -- -D warnings; cargo test --locked. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0. This also covers the existing tests that expect Pending-with-no-merge, and the display.rs tests.
  Fails without the behavior: Any compile error, lint warning (for example a non-exhaustive match on Landing or ItemEnd) or failing existing test makes one of the three commands exit non-zero.
  Test seam: Whole workspace. Existing and new tests.
- Criterion 7: Edit the doc comment on classify_checks (client.rs ~785) and the doc of wait_for_checks (early.rs ~117) to say that any check run on the head still queued or in progress makes the verdict pending, and that only a failed required check is Red. Check by reading both comments and by grep (the Grep tool also works). Run: `grep -n "queued or in progress\|in progress" crates/athena-core/src/gh/client.rs crates/athena-core/src/gh/early.rs`. Observes: Both doc comments state the any-queued-or-in-progress-run rule.
  Fails without the behavior: Without the doc updates the grep finds only the old wording.
  Test seam: Content check, no test seam.

Planned files:

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 31f2111f13ac → candidate f7a2d5aa3e4c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  12 +++
 crates/athena-cli/src/display.rs     |  17 ++++
 crates/athena-cli/src/drain.rs       | 164 ++++++++++++++++++++++++++++++-----
 crates/athena-core/src/gh/client.rs  |  23 +++--
 crates/athena-core/src/gh/early.rs   |   4 +-
 crates/athena-core/src/gh/landing.rs |  48 ++++++++--
 crates/athena-core/src/gh/tests.rs   | 121 ++++++++++++++++++++++++++
 7 files changed, 354 insertions(+), 35 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41343379060 for f7a2d5aa3e4ceeef439f40ea71f71ab94fa55a81)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41343379060 for f7a2d5aa3e4ceeef439f40ea71f71ab94fa55a81)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41343379060 for f7a2d5aa3e4ceeef439f40ea71f71ab94fa55a81)
- ci-result: `not a check` passed (Check results for f7a2d5aa3e4ceeef439f40ea71f71ab94fa55a81 come from the CI run https://github.com/doodla/athena/actions/runs/41343379060 (588s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes and `cargo clippy --all-targets` and `cargo fmt --check` are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
