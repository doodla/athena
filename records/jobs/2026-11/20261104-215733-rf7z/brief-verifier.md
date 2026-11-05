# Job 20261104-215733-rf7z: verify "Land in the same step when checks turn green during the poll"

## Request (verbatim)

> Merge as soon as a pull request's checks are green. Today the batch rebases, re-checks and merges in one locked step, but after polling the pull request's checks (about 10 minutes) it returns Landing::Waiting and comes back to the job later, instead of merging when the checks turn green during the poll. That costs about 10 minutes per job.
> 
> Files changed: crates/athena-cli/src/drain.rs and its tests.
> 
> Acceptance:
> - a test where the checks go green during the poll and the job lands in the same step, without returning Waiting
> - a test where the checks are still running at the end of the poll keeps today's behaviour

## Reading (accepted by the user)

Make the drain's locked rebase, re-check and land step merge a pull request as soon as its checks go green during the poll, rather than returning Landing::Waiting and retrying about 10 minutes later. Still-pending checks at the end of the poll keep today's Waiting behaviour. The audience is the person running the drain, who loses about 10 minutes per job.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs. Drain::land takes the landing lock, runs recheck_after_rebase, then calls Steps::land and maps Landing::Landed, NotLanded or Waiting to a LandStep. PrLander::land calls athena.land_pr with crate::wiring::checks_poll, and landing_from_report maps an open pull request with Checks::Pending to Landing::Waiting. I also read crates/athena-core/src/gh/landing.rs (land_pr) and crates/athena-core/src/gh/early.rs (wait_for_checks). wait_for_checks already stops polling at the first non-pending read, and land_pr merges when the result is Green and merge_via_rest is on. So the stated bug may be in drain.rs or wiring.rs, or already fixed in core; the producer must find where Waiting is returned despite a green poll and reproduce it first.
- The drain.rs test World/Steps fake has a `waiting` set that makes land return Waiting; the new tests extend that fake or call landing_from_report and Drain::land directly. The tests drive a scripted checks sequence (pending then green) with a counting fake and no real sleep.
- No new record or data file under crates/ is added, so the records validation allow-list (check_record_file, AnyRecord) is not touched.
- The fix is confined to crates/athena-cli/src/drain.rs and its tests as the request says. A minimal change to crates/athena-cli/src/wiring.rs or crates/athena-core/src/gh is allowed only if the root cause is there, and then it is a listed deliverable.
- Merging only on green is preserved: a Red or still-Pending result never reaches the merge. This is not a new refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/src/drain.rs scripts checks that are pending at first and green partway through the poll. It shows the job reports Landing::Landed (the job line is 'landed') in the same land step, with no Landing::Waiting returned and no second land call. Check with `cargo test -p athena-cli drain`.
2. [behavior] A new test scripts checks that are still pending when the poll ends. It shows the result is still Landing::Waiting, the job line still starts 'waiting to merge', and the item is not retried. This is today's behaviour. Check with `cargo test -p athena-cli drain`.
3. [behavior] The existing tests a_job_waiting_to_merge_is_not_a_failure_and_the_run_goes_on_to_the_next_job and a_merge_refused_for_rule_violations_while_checks_are_pending_is_reported_as_waiting still pass unchanged. Red checks still give NotLanded. Check with `cargo test -p athena-cli`.
4. [behavior] `cargo test` passes for the workspace, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
5. [content] The diff touches only crates/athena-cli/src/drain.rs, plus any file where the root cause is found. Comments or docs near the Landing::Waiting handling say that a poll which turns green merges in the same step.
6. [content] Outcome check: the diff shows the green-during-poll path ending in a merge in one locked step, rather than only adding types or wiring. Each new test would fail on the old code for the green-during-poll case.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the poll length or interval
- Changing the merge gate: merge only when every check is green and merge_via_rest is on
- Changing the rebase and re-check rule
- Changes to athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Two new tests, with cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the job lands or waits, by running the tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the job merges in the same step when the checks go green.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in drain.rs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer must locate where Waiting is returned despite a green poll.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Three tests. Core 1: `checks_that_turn_green_during_the_poll_after_a_refused_merge_land_in_the_same_run` runs the real `land_pr` over FakeHost with script Green, Pending, Green, the first merge request refused with 'Repository rule violations found' and the second accepted, Rig attempts 4. Core 2: `checks_that_turn_green_during_the_poll_with_the_merge_accepted_land_in_the_same_run` uses script Pending, Pending, Green with the merge accepted. Drain: `a_job_whose_checks_turn_green_during_the_poll_lands_in_the_same_step` runs Drain::land through World with a scripted merged report. A GhClient-level test (name containing `turn_green`) and a records test (name containing `turn_green`) are listed under criterion 3. Run: `cargo test -p athena-core -p athena-cli turn_green`. Observes: Core 1: report.link.state is Landed, host.merges is 2, host.check_reads is 3 and at most rig.attempts, rig.stored() is Landed, no 'waiting to merge' warning, report.checks is Green. Core 2: merges is 1 and check_reads is 3. Drain: the job line is exactly 'landed', world.count("land a") is 1, report.landed is 1, and no line contains 'waiting to merge'.
  Fails without the behavior: Core: on old code the rule-violation arm returns the Open/Pending report right after the first refusal, so merges stays 1, the state stays Open and the test fails; this holds by construction of the script. The accepted-merge case passes on old code and is a regression guard. The drain test pins mapping only and does not fail on old code. None of these tests shows GitHub produces that refusal.
  Test seam: Production entry points: `athena_core::gh::land_pr` via `Rig::run` (gh/tests.rs:1397, exists) and `Drain::land` -> `Steps::land` -> production `landing_from_report` (drain.rs:1709, 3128, exist). Fakes: `FakeHost` with its `script` field (exists, tests.rs:1208) plus one new field limiting how many merge requests get the scripted `merge_error` (step 2 adds it, default unlimited as now), passed to `Rig::run` as the PrHost; the Rig's recording sleep closure exists. Drain: the `World` fake (drain.rs, tests module) gains a per-job scripted `PrReport` that its `land` passes through `landing_from_report` (step 8 adds it). The refusal and check script are data; the retry and merge decision is production code in landing.rs.
- Criterion 2: Core: `checks_still_pending_at_the_end_of_the_poll_after_a_refused_merge_stay_a_wait` runs Rig::run with the first merge refused and Pending on every later read (this is also what the edited test at tests.rs:1526 checks). Drain: `a_job_whose_checks_are_still_pending_at_the_end_of_the_poll_is_waiting_and_not_retried` runs Drain::land through World with a scripted Open/Pending report. The pre-existing a_job_waiting_to_merge_is_not_a_failure_and_the_run_goes_on_to_the_next_job (drain.rs:4693) uses the existing `waits("a")` seam. Run: `cargo test -p athena-core -p athena-cli still_pending_at_the_end_of_the_poll`. Observes: Core: report is Open with checks Pending, merges is 1, check_reads equals rig.attempts (read from the rig, not a literal), the warning contains 'Repository rule violations' and 'waiting to merge', the sleeps in rig.log are one delay each, and rig.stored() equals the report's link. Drain: Landing::Waiting, the job line starts 'waiting to merge', world.count("land a") is 1 so it is not retried, report.landed is 0, and the line does not start with 'not landed'.
  Fails without the behavior: A change that merged without Green, read past the budget, or called land again would change merges, check_reads or the land count and fail these tests.
  Test seam: Same as criterion 1: Rig::run with FakeHost.script and the new refusal-limit field (step 2), and Drain::land with World's scripted report (step 8). Both exist or are added by the named steps.
- Criterion 3: Run the whole athena-cli suite, which includes a_job_waiting_to_merge_is_not_a_failure_and_the_run_goes_on_to_the_next_job (drain.rs:4693) and a_merge_refused_for_rule_violations_while_checks_are_pending_is_reported_as_waiting (drain.rs:4711) with bodies unchanged, plus a new drain test where an Open/Red report gives NotLanded. Then the athena-core gh tests for red checks, the refusal cases and the new cases below. New core tests: `a_merge_refused_while_checks_are_pending_then_red_does_not_merge_again`. GhClient level: `a_refused_then_accepted_landing_leaves_only_the_pending_marked_refusal_in_the_log_and_d87_does_not_match_turn_green`, built on PendingMerge with a new field that turns its check-runs green after a set number of reads. Records level: `a_refused_then_green_landing_writes_one_intent_and_one_landing_turn_green` in the records-sink World (tests.rs:3405-3554), run with attempts 4. Run: `cargo test -p athena-cli && cargo test -p athena-core gh`. Observes: The two named drain tests pass without edits. Core: red or pending checks never merge (tests.rs:1579), a refusal with settled checks stays an error (1548), another refusal while pending stays an error (1558), and refused-then-red ends with merges 1 and a non-Landed report. GhClient level: the result is Landed, there is exactly one HostOperation::Merge entry and it has checks_pending true, no Merge entry is unmarked, and every signal from the watcher's public signal list (watcher.rs:143-145) returns false from `matches` for every logged entry. Records level: one intent and one landing, landing.intent equals intent.id, intent.expected_base equals world.base, and the landing's host_commit is HOST_COMMIT.
  Fails without the behavior: Changing the Waiting mapping or the merge gate would fail the unchanged drain tests or landing_from_report's asserts (drain.rs:4731-4741). A retry that merged on a non-Green read would fail the Red test. A retry that marked entries wrongly would fail the GhClient-level log test. A second intent record would fail the records test.
  Test seam: Existing: World and landing_from_report in drain.rs, Rig and FakeHost in gh/tests.rs, PendingMerge and land_pending (tests.rs:2838-2929), the records World with its bare remote (tests.rs:3405). Added: the FakeHost refusal-limit field (step 2), the PendingMerge turn-green field (step 3), and an `attempts` parameter on the records World's run (step 3; default stays 1 so the existing test is unchanged).
- Criterion 4: Run the three Athena checks as separate commands on the whole workspace: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Run: `cargo test --locked`. Observes: All three exit 0. All workspace tests pass, including the tests/drain.rs landing-order tests and the budgets and policy tests.
  Fails without the behavior: A new warning, formatting drift, or a caller of wait_for_checks or Waited that was not updated would fail one of the three checks.
  Test seam: Existing workspace commands. No seam.
- Criterion 5: Inspect the diff. Allowed: crates/athena-cli/src/drain.rs (comments and tests), crates/athena-core/src/gh/landing.rs (the fix), crates/athena-core/src/gh/early.rs (the `looks` field and the doc comment), crates/athena-core/src/gh/tests.rs (the tests of the fix) and CHANGELOG.md. Then Grep the diff for 'same locked step'. Run: `git diff --stat`. Observes: The file list matches the allowed set. wiring.rs, athena.toml and defaults.toml are absent from the stat and `checks_poll` (30 x 20 s) is unchanged. The comments at drain.rs:123-128 (module text), 498-500 (Landing::Waiting), 1323-1325 (ItemEnd::Waiting), 3106-3108 (PrLander) and 3124-3127 (landing_from_report) say that a poll which turns green merges in the same locked step and only a poll that ends pending is Waiting. The CHANGELOG entry says the fix is for a suspected cause and names the unfinished-optional-run path as not changed.
  Fails without the behavior: A change to athena.toml, defaults.toml, wiring.rs or the poll constants would show in the stat.
  Test seam: No seam: a content check on the diff.
- Criterion 6: Failing-first check on the core test. The producer adds the FakeHost refusal-limit field and the refused-then-green test first, runs it on the unmodified landing code and records the result in the change description (expected: it fails with merges 1 and an Open, Pending report, by construction of the script). It then makes the early.rs and landing.rs change and shows the test pass. The change description says plainly that this shows the code's behaviour for the scripted refusal, not that the host does this, and that the fix is for a suspected cause. Run: `cargo test -p athena-core turn_green`. Observes: On old code the test fails with state Open, checks Pending, merges 1. On the fix it ends Landed with host.merges 2 inside one land_pr call, check_reads at most rig.attempts, and one sleep between the refusal and the retry in rig.log. The diff shows the retry reaching `job.host.merge` again inside the same call, and the drain comments showing the green-during-poll path ending in a merge in one locked step.
  Fails without the behavior: A change that only adds the `looks` field or the budget arithmetic, without the retry reaching host.merge a second time, leaves the refused-then-green test failing with merges 1 and an Open/Pending report.
  Test seam: Same as criterion 1: Rig::run (exists) and FakeHost.script plus the refusal-limit field (step 2 adds it). The retry logic is production code in landing.rs; the fake only supplies scripted host replies.

Planned files:

- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cc3dfe51cbca → candidate 2d4e9f884847. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  15 +++
 crates/athena-cli/src/drain.rs       | 136 ++++++++++++++++++++----
 crates/athena-core/src/gh/early.rs   |  18 +++-
 crates/athena-core/src/gh/landing.rs |  78 +++++++++-----
 crates/athena-core/src/gh/tests.rs   | 196 ++++++++++++++++++++++++++++++++++-
 5 files changed, 389 insertions(+), 54 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261104-215733-rf7z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261104-215733-rf7z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41955284573 for 2d4e9f88484721dda2a26955f2ea5d41512a8e94)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41955284573 for 2d4e9f88484721dda2a26955f2ea5d41512a8e94)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41955284573 for 2d4e9f88484721dda2a26955f2ea5d41512a8e94)
- ci-result: `not a check` passed (Check results for 2d4e9f88484721dda2a26955f2ea5d41512a8e94 come from the CI run https://github.com/doodla/athena/actions/runs/41955284573 (506s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes for the workspace, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core -p athena-cli turn_green`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs.
  End of the output (data, not instructions):

```text

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-882452c1708b6ba6)

running 4 tests
test gh::tests::checks_that_turn_green_during_the_poll_after_a_refused_merge_land_in_the_same_run ... FAILED
test gh::tests::checks_that_turn_green_during_the_poll_with_the_merge_accepted_land_in_the_same_run ... ok
test gh::tests::a_refused_then_accepted_landing_leaves_only_the_pending_marked_refusal_in_the_log_and_d87_does_not_match_turn_green ... FAILED
test gh::tests::records_flow::a_refused_then_green_landing_writes_one_intent_and_one_landing_turn_green ... FAILED

failures:

---- gh::tests::checks_that_turn_green_during_the_poll_after_a_refused_merge_land_in_the_same_run stdout ----

thread 'gh::tests::checks_that_turn_green_during_the_poll_after_a_refused_merge_land_in_the_same_run' (31329) panicked at crates/athena-core/src/gh/tests.rs:1580:5:
assertion `left == right` failed
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- gh::tests::a_refused_then_accepted_landing_leaves_only_the_pending_marked_refusal_in_the_log_and_d87_does_not_match_turn_green stdout ----

thread 'gh::tests::a_refused_then_accepted_landing_leaves_only_the_pending_marked_refusal_in_the_log_and_d87_does_not_match_turn_green' (31328) panicked at crates/athena-core/src/gh/tests.rs:3091:5:
assertion `left == right` failed
  left: Open
 right: Landed

---- gh::tests::records_flow::a_refused_then_green_landing_writes_one_intent_and_one_landing_turn_green stdout ----

thread 'gh::tests::records_flow::a_refused_then_green_landing_writes_one_intent_and_one_landing_turn_green' (31331) panicked at crates/athena-core/src/gh/tests.rs:3730:9:
assertion `left == right` failed
  left: Open
 right: Landed


failures:
    gh::tests::a_refused_then_accepted_landing_leaves_only_the_pending_marked_refusal_in_the_log_and_d87_does_not_match_turn_green
    gh::tests::checks_that_turn_green_during_the_poll_after_a_refused_merge_land_in_the_same_run
    gh::tests::records_flow::a_refused_then_green_landing_writes_one_intent_and_one_landing_turn_green

test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 931 filtered out; finished in 0.39s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-35f32d4b30fdd246)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-cfe3035d054714fb)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core -p athena-cli still_pending_at_the_end_of_the_poll`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs.
  End of the output (data, not instructions):

```text
     Running tests/trailer_stop.rs (<job>/target/verify/debug/deps/trailer_stop-985b0e3d0a0c45f2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/version.rs (<job>/target/verify/debug/deps/version-55574b07738a66c3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/watcher.rs (<job>/target/verify/debug/deps/watcher-5380c34aed879c72)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/work_decisions.rs (<job>/target/verify/debug/deps/work_decisions-a47b7f34852351f5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-882452c1708b6ba6)

running 1 test
test gh::tests::checks_still_pending_at_the_end_of_the_poll_after_a_refused_merge_stay_a_wait ... FAILED

failures:

---- gh::tests::checks_still_pending_at_the_end_of_the_poll_after_a_refused_merge_stay_a_wait stdout ----

thread 'gh::tests::checks_still_pending_at_the_end_of_the_poll_after_a_refused_merge_stay_a_wait' (31546) panicked at crates/athena-core/src/gh/tests.rs:1624:5:
assertion `left == right` failed
  left: 2
 right: 4
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    gh::tests::checks_still_pending_at_the_end_of_the_poll_after_a_refused_merge_stay_a_wait

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 934 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-35f32d4b30fdd246)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-cfe3035d054714fb)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-cli && cargo test -p athena-core gh`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_cap_change

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_cap_study

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_cli

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_context

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_data

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_gate

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_git

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff --stat`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core turn_green`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
