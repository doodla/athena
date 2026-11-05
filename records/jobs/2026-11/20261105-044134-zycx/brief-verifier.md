# Job 20261105-044134-zycx: verify "Wait for main's in-progress CI run before running the baseline locally"

## Request (verbatim)

> Wait for main's GitHub run instead of re-testing locally. When a job starts right after a landing, main's GitHub run for the same code is usually still in progress, so the baseline falls back to a 20-minute local run ('main: no usable CI result ... on main'). This happened in 3 of the last 10 jobs. When main's run for the base's content is in progress, wait for it (reading it every 20 seconds, up to the existing ci_wait_s limit) and reuse it if it passes; run locally only if it fails, is missing, or the wait runs out.
> 
> Files changed: crates/athena-core/src/run/checks.rs and its tests.
> 
> Acceptance:
> - a test where main's run for the base's content is in progress and then passes: the baseline is reused, with no local run
> - a test where main's run fails: the baseline runs locally
> - the existing baseline-reuse tests still pass

## Reading (accepted by the user)

When the baseline lookup finds main's push run for the base's content still in progress, wait for it (a read every 20 s, up to the existing ci_wait_s) and reuse it if it passes. If it fails, is missing, or the wait runs out, the baseline runs locally as today. The audience is Athena operators, who now avoid the 20-minute local baseline right after a landing.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs. reused_baseline resolves the base's content commit, then tries (1) the landed job's candidate run on its CI branch, then (2) the origin/<default> tip run when its fingerprint equals the base's. Both go through green_run, which calls with_ci_source_on(branch, 0, ..), so it makes exactly one read with no wait. A run that is unfinished falls to 'no usable CI result for {sha} on {branch}' and the baseline runs locally in baseline_checks.
- Code survey: crates/athena-core/src/run/mod.rs with_ci_source_on turns wait_s into polls via routing::ci_polls (a read every CI_POLL = 20 s, polls = wait_s/20 + 1) and paces reads with the test sleeper seam ci_seam. The candidate path at mod.rs ~1596 already passes jc.config.budget.ci_wait_s. The change threads ci_wait_s (from ctx.jc.config.budget) into green_run for the main-tip lookup (step 2), where main's run is the one likely still in progress; this reuses the existing wait machinery and adds no new limit.
- Assumption: waiting applies to main's run for the base's content (lookup 2). The landed-candidate lookup (1) is for a finished run on the job's CI branch and may keep one read. Failed, missing, or timed-out runs still fall back to a local run with the existing 'baseline runs locally: ...' line.
- Code survey: the existing baseline-reuse tests are in crates/athena-core/src/run/tests.rs (baseline_reused_from_*, baseline_runs_locally_when_*), using CiReplies/CiReply (Green, Red, Unfinished, Missing) and ci_reuse_run_with. The new tests go there. CiReplies needs a way to answer Unfinished for the first N reads and then Green; run/tests.rs is therefore a likely extra file beyond checks.rs. No record or data file is added under crates/, so the records allow-list is untouched. No refusal rule is added.
- The test for the green case must count reads (baseline_reads) to show the wait polled more than once, and the sleeper seam must show 20 s pauses, so the wait is measured directly.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/tests.rs has main's run for the base's content return in-progress on the first reads and then green. It asserts reused_from is Some(CI_RUN_ADDRESS), baseline_ran is false, more than one read of main happened, and the recorded sleeps are 20 s each. Run: cargo test -p athena-core baseline.
2. [behavior] A new test has main's run for the base's content in progress and then failing (red). It asserts baseline_ran is true, reused_from is None, and the 'ci         baseline runs locally' line is printed. Run: cargo test -p athena-core baseline.
3. [behavior] A new test has main's run still in progress when the ci_wait_s bound is exhausted. It asserts the number of reads equals ci_polls(ci_wait_s) and the baseline runs locally. Run: cargo test -p athena-core baseline.
4. [behavior] All existing baseline tests still pass unchanged: baseline_reused_from_the_landed_candidates_run_runs_nothing_locally, baseline_reused_from_the_default_branch_run_runs_nothing_locally, baseline_reused_from_a_squash_on_main_with_the_bases_fingerprint, baseline_runs_locally_when_main_tip_has_other_governed_content and baseline_runs_locally_when_no_green_ci_result_exists. Run: cargo test -p athena-core.
5. [behavior] cargo clippy --all-targets and cargo fmt --check are clean.
6. [content] The doc comments on reused_baseline and green_run in crates/athena-core/src/run/checks.rs no longer say 'no wait'. They state that main's in-progress run is waited for, reading every 20 s up to ci_wait_s.
7. [content] The diff touches no governance file (athena.toml, defaults.toml, workflows), and reuses ci_wait_s and routing::CI_POLL without adding a new limit.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the ci_wait_s default or adding a new config key
- Changing how the candidate's own checks wait for CI (run/mod.rs ~1596)
- Changing routing.rs, ci_result.rs, or the gh client
- Changing the fingerprint rules for deciding whether main's tip matches the base

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and its tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for wait-then-pass, fail, and timeout; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert reuse, no local run, and read counts, not wiring.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing Wait, Paced, ci_polls and ci_wait_s machinery.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the baseline lookup.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run through cargo test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Waits for main's in-progress run instead of falling back on every race after a landing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments are updated for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test baseline_waits_for_mains_in_progress_run_then_reuses_it in run/tests.rs. Setup: origin is GITHUB. main_tip_beside_head(git, None) makes a tip commit on origin/main with the base's fingerprint. The fake is CiReplies::new(Green).only_for(&tip, "main").unfinished_first(2), so the first 2 reads of that run say in_progress and later reads say success. Run: `cargo test -p athena-core baseline_waits_for_mains_in_progress_run_then_reuses_it`. Observes: baseline.reused_from == Some(CI_RUN_ADDRESS); baseline_ran false (MARKS_BASELINE_OUTSIDE did not leave baseline-ran); baseline_reads filtered to head_sha=<tip> and branch=main& is exactly 3 (more than one); the recorded sleeps hold 20 s each, with 2 for the baseline plus the 75 the candidate's own missing run adds, so all 77 equal 20 s; every baseline check is reused and passed; the 'ci         baseline reused from' line is printed.
  Fails without the behavior: Without the wait, green_run makes one read, sees in_progress, and falls back: baseline_ran is true, reused_from is None, and reads is 1, so every assertion fails.
  Test seam: Production entry point: Athena::go (through ci_reuse_run_with), which calls take_baseline, then reused_baseline, then green_run. Guards: a GitHub origin, a route that covers fmt, clippy and test, a content commit, a fingerprint equal to main's tip. Fakes: CiReplies (a gh::Runner) and the recording sleeper, both injected through ci_seam::install inside ci_reuse_run_with, which exists today. The N-then-final-reply mode of CiReplies is added by plan step 4. The fake only answers by reply and counter; the wait logic stays in production code.
- Criterion 2: New test baseline_runs_locally_when_mains_in_progress_run_fails: same setup, with CiReplies::new(Red).only_for(&tip, "main").unfinished_first(1). Run: `cargo test -p athena-core baseline_runs_locally_when_mains_in_progress_run_fails`. Observes: baseline_ran true; baseline.reused_from None; no baseline check is reused; exactly one baseline line, starting 'ci         baseline runs locally: ' and containing CI_RUN_ADDRESS (the 'did not pass' reason); reads of main equal 2 (the in-progress read, then the red read), so the wait stopped at the red result and did not run on to the bound.
  Fails without the behavior: Without the wait, the first read is in_progress and the reason is 'no usable CI result', which lacks CI_RUN_ADDRESS and has 1 read. Without the red branch (a wait that ignores failures), reads would be 76, not 2.
  Test seam: Same as criterion 1: Athena::go through ci_reuse_run_with, CiReplies and the sleeper installed via ci_seam, existing. The Red-after-N reply uses the CiReplies mode added in step 4.
- Criterion 3: New test baseline_runs_locally_when_mains_run_stays_in_progress_to_the_bound: the fake is CiReplies::new(Unfinished).only_for(&tip, "main"). A second case, baseline_does_not_wait_for_a_missing_main_run, uses CiReplies::new(Missing) with the same setup. Run: `cargo test -p athena-core baseline_runs_locally_when_mains_run_stays_in_progress_to_the_bound baseline_does_not_wait_for_a_missing_main_run`. Observes: Unfinished case: reads of main equal crate::routing::ci_polls(1500), which is 76 (the ci_wait_s of the test config, as the existing test at run/tests.rs:3587 relies on); baseline_ran true; reused_from None; the line is 'ci         baseline runs locally: ... no usable CI result for <tip> on main'. Missing case: reads of main equal 1 and baseline_ran is true, so a missing run is not waited for.
  Fails without the behavior: Without the bound, the loop would stop short of or run past 76 reads. Without the missing-run gate, the Missing case would make 76 reads, not 1.
  Test seam: Same as criterion 1: Athena::go through ci_reuse_run_with, CiReplies and the sleeper via ci_seam, existing. ci_polls is public in routing.rs:124.
- Criterion 4: Run the existing tests with the new code in place; none of their code is edited. Step 1 (landed candidate) keeps one read. In the Green cases the wrapper passes the first read through, so the read counts those tests assert are unchanged. The Missing, Unreachable and no-origin cases have no origin/HEAD, or no run, so the wrapper never waits. Run: `cargo test -p athena-core`. Observes: baseline_reused_from_the_landed_candidates_run_runs_nothing_locally, baseline_reused_from_the_default_branch_run_runs_nothing_locally, baseline_reused_from_a_squash_on_main_with_the_bases_fingerprint, baseline_runs_locally_when_main_tip_has_other_governed_content and baseline_runs_locally_when_no_green_ci_result_exists pass with their assertions untouched, as do ci_reuse_falls_back_to_local_checks_when_the_run_gives_no_result and the other ci_reuse_* tests.
  Fails without the behavior: A wrapper that waited on a missing run would break baseline_runs_locally_when_no_green_ci_result_exists and ci_reuse_*, which assert read and pause counts.
  Test seam: Existing tests; no new seam.
- Criterion 5: Run the format and lint checks Athena runs. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Exit status 0 with no warnings; then cargo fmt --all --check also exits 0.
  Fails without the behavior: Not a behavior criterion by itself; a new unused field, a needless clone or unformatted code would fail it.
  Test seam: No test seam; static checks on the whole workspace.
- Criterion 6: Read the diff of the doc comments in checks.rs. Run: `git diff -- crates/athena-core/src/run/checks.rs`. Observes: The comments on take_baseline, reused_baseline and green_run no longer say 'no wait' or 'One read'. They say main's in-progress run is waited for, reading every 20 s (routing::CI_POLL) up to ci_wait_s, that a missing, red or timed-out run is not reused, and that the landed candidate's lookup is one read. grep for 'no wait' in checks.rs finds none.
  Fails without the behavior: Not a behavior criterion.
  Test seam: None; content check.
- Criterion 7: Read the file list of the diff and search the new code. Run: `git diff --stat`. Observes: Only checks.rs, tests.rs and CHANGELOG.md are listed; athena.toml, defaults.toml and .github/workflows are not. The new code names budget.ci_wait_s and routing::ci_polls/CI_POLL through with_ci_source_on, and adds no constant, config key or Duration.
  Fails without the behavior: Not a behavior criterion.
  Test seam: None; content check.

Planned files:

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ddfa8aab9b2a → candidate cae416f38b87. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   9 ++
 crates/athena-core/src/run/checks.rs |  66 ++++++++++---
 crates/athena-core/src/run/tests.rs  | 175 ++++++++++++++++++++++++++++++++++-
 3 files changed, 233 insertions(+), 17 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-044134-zycx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-044134-zycx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41982176088 for cae416f38b872a1e7dc692b97092bf2036c9d7f3)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41982176088 for cae416f38b872a1e7dc692b97092bf2036c9d7f3)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41982176088 for cae416f38b872a1e7dc692b97092bf2036c9d7f3)
- ci-result: `not a check` passed (Check results for cae416f38b872a1e7dc692b97092bf2036c9d7f3 come from the CI run https://github.com/doodla/athena/actions/runs/41982176088 (545s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo clippy --all-targets and cargo fmt --check are clean.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core baseline_waits_for_mains_in_progress_run_then_reuses_it`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/checks.rs.
  End of the output (data, not instructions):

```text
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 50.95s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-7a0cd9ac4aba6ffa)

running 1 test
test run::tests::produce_and_verify_order_tests::baseline_waits_for_mains_in_progress_run_then_reuses_it ... FAILED

failures:

---- run::tests::produce_and_verify_order_tests::baseline_waits_for_mains_in_progress_run_then_reuses_it stdout ----

thread 'run::tests::produce_and_verify_order_tests::baseline_waits_for_mains_in_progress_run_then_reuses_it' (11017) panicked at crates/athena-core/src/run/tests.rs:3397:9:
assertion `left == right` failed: [
    "dispatched 20261105-065341-fhjr · prediction commit 12e5328a5736",
    "ci         baseline runs locally: the base is not the landing of one job; main: no usable CI result for 1b66835de6794684744b7356603ee3fe6de7909f on main",
    "baseline   fmt ok · clippy ok · test ok",
    "producer   claude-haiku-5-5 · 0s · $0.01 · 0 commands run",
    "candidate  README.md (modified) · fingerprint sha256:a98d7ea8e8b…",
    "checks     fmt ok · clippy ok · test ok (0s)",
    "replay     no replay: no accepted readings in the history",
    "verifier   claude-opus-5-5 · 0s · $0.01 · pass",
    "landed     bed18d359254 \"Describe the demo\" on main",
    "reclaimed 106.5 KB",
    "code health  0 file(s) measured, 0 structural pass request(s) queued",
    "code health  baseline set at 0 landings; the first design review is due 10 landings later",
]
  left: None
 right: Some("https://github.com/owner/repo/actions/runs/4242")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    run::tests::produce_and_verify_order_tests::baseline_waits_for_mains_in_progress_run_then_reuses_it

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 938 filtered out; finished in 5.78s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-deb874dc79482e38)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-658bf9aaa38225b9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core baseline_runs_locally_when_mains_in_progress_run_fails`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/checks.rs.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-7a0cd9ac4aba6ffa)

running 1 test
test run::tests::produce_and_verify_order_tests::baseline_runs_locally_when_mains_in_progress_run_fails ... FAILED

failures:

---- run::tests::produce_and_verify_order_tests::baseline_runs_locally_when_mains_in_progress_run_fails stdout ----

thread 'run::tests::produce_and_verify_order_tests::baseline_runs_locally_when_mains_in_progress_run_fails' (12041) panicked at crates/athena-core/src/run/tests.rs:3435:9:
assertion `left == right` failed: ["repos/owner/repo/actions/runs?head_sha=73d5eff4cc37a7fdacfd2eba97ede5271ee882c1&event=push&branch=main&per_page=20"]
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    run::tests::produce_and_verify_order_tests::baseline_runs_locally_when_mains_in_progress_run_fails

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 938 filtered out; finished in 5.70s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-deb874dc79482e38)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-658bf9aaa38225b9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core baseline_runs_locally_when_mains_run_stays_in_progress_to_the_bound baseline_does_not_wait_for_a_missing_main_run`, exit code 1. Result: the named test command failed without the fix (exit 1).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/checks.rs.
  End of the output (data, not instructions):

```text
error: unexpected argument 'baseline_does_not_wait_for_a_missing_main_run' found

Usage: cargo test [OPTIONS] [TESTNAME] [-- [ARGS]...]

For more information, try '--help'.
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- crates/athena-core/src/run/checks.rs`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
