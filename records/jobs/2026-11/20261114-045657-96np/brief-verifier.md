# Job 20261114-045657-96np: verify "Run real-records status criteria before the verifier and record evidence"

## Request (verbatim)

> Priority: high
> 
> Five status- or usage-measure jobs failed between 2026-11-11 and 2026-11-12, all with failure class 20261111-105030-a2bc. The reading gate (check_records_measure) already makes these readings include a criterion that runs the measure on the real records against an independent range. But nothing runs that criterion before the final verdict. Producers ship measures that were checked only on fixtures, or evidence that cites no command that ran, and the verifier rejects the whole job. After the producer finishes and before the verifier starts, have Athena itself run each behavior criterion that names real records (the status --snapshot command the reading gives) against the records snapshot. Record the command and its output as evidence that the producer and verifier can see. If the measure gives the same answer for every item, or falls outside the stated range, send it back to the producer. If the snapshot cannot be reached from the producer's workspace, end the job early with that reason rather than at the verdict. Add a test using the fake producer, and a CHANGELOG.md entry.
> 
> Introspections that record the repeated cause: 20261111-105017-5k6m, 20261111-122618-q1ha, 20261111-145341-v9p4, 20261111-212939-722z, 20261112-074421-cpkp, 20261112-180919-sydm, 20261112-190037-pkhv, 20261113-000027-1saa
> 
> Acceptance:
> - The change is made as described: New status-measure jobs keep failing at the final verdict because nothing runs the real-records criterion before the verifier does
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: CHANGELOG.md

## Reading (accepted by the user)

After the producer's report and before the verifier, Athena runs each behavior criterion that gives a `status --snapshot` command against the evidence snapshot and records the command and its output as evidence for the producer and verifier. A constant measure or an out-of-range value is sent back to the producer; an unreachable snapshot ends the job early with that reason. Audience: Athena operators and the verifier, who stop seeing these jobs fail at the final verdict.

Assumptions:
- Code survey: crates/athena-core/src/run/producer.rs (producer_loop: report, format_stage, change_list, loop_replay, loop_check_failures, then break to the verifier; send-backs via PRODUCER_SEND_BACKS and problems/send_back_text), crates/athena-core/src/run/mod.rs (the evidence snapshot is written at ~line 1153 via crate::snapshot::evidence_dir and substituted into the producer brief), crates/athena-core/src/snapshot.rs (`athena status --snapshot <dir>` reads it with no repository), crates/athena-core/src/reading/surveys.rs and reading.rs (the real-records criterion text), crates/athena-core/src/run/tests.rs and crates/athena-agent/src/fake.rs (fake producer). Run order: producer report, format, self-checks, then the new records-criteria run, then verifier. The measure is read directly from the command's printed output on the snapshot. No record or data file is added under crates/, so check_record_file and AnyRecord are untouched unless a new record kind is chosen, which is not planned; the output is kept as check evidence in the existing form.
- Existing code: crates/athena-core/src/snapshot.rs (evidence_dir, write, status over a snapshot dir), crates/athena-core/src/run/producer.rs (loop_check_failures, append_check_failures, send-back text) and run/checks.rs (check evidence the verifier sees); the change extends these rather than adding a parallel path.
- Selection: a behavior criterion whose text contains `status --snapshot` with an evidence directory; Athena runs it from its own process on the evidence snapshot directory, not inside the producer sandbox.
- Same answer for every item means the measure's printed per-item values are all identical when there is more than one item; the stated range is parsed from the criterion text. If the range cannot be parsed, the output is recorded and shown but does not send back.
- Environment: reads the local evidence snapshot directory Athena writes per job (read-only copy of records/, landings/ and main first-parent history); no network, push, PR or CI. `Snapshot unreachable from the producer's workspace` means the path is missing or not readable by the producer's grant; then the job ends early with that reason via self.failure.
- Send-backs reuse the existing PRODUCER_SEND_BACKS budget; after it is spent the failure reaches the verifier as a flagged gap, like check failures.
- Field meaning: no existing field, CLI argument or config key is reused; new behavior uses existing status --snapshot CLI as documented in crates/athena-core/src/lib.rs:28.

## Acceptance criteria

1. [behavior] A new fake-producer test in crates/athena-core/src/run/tests.rs: a reading with a `status --snapshot` behavior criterion whose measure is constant across items is sent back to the producer before the verifier runs, and the recorded evidence holds the command and its output. Run `cargo test -p athena-core` and name the test.
2. [behavior] A fake-producer test shows a measure inside the stated range passes: the command and output are recorded as evidence in the verifier's brief and the verifier then runs. Run `cargo test -p athena-core`.
3. [behavior] A fake-producer test shows a measure outside the stated range is sent back, and an unreachable snapshot ends the job early with a reason that names the snapshot, before any verifier invocation. Run `cargo test -p athena-core`.
4. [content] crates/athena-core/src/run/producer.rs (or a new run module it calls) runs the criterion after loop_check_failures and before `break Some(report)`, in Athena's own process, and the diff shows the command and output added to the evidence the producer's send-back and the verifier's brief carry.
5. [content] CHANGELOG.md has a new entry describing the pre-verifier real-records run, its send-back conditions and the early end when the snapshot is unreachable.
6. [content] If a new appended decision is added to docs/decisions.md it has a Provenance bullet; no existing entry or anything under records/ is edited.
7. [behavior] No unrelated behavior changes: criteria that name no real records run nothing new, and the existing tests pass. Run `cargo test`.
8. [behavior] Run `cargo fmt --check`.
9. [behavior] Run `cargo clippy --all-targets`; no warnings.

## Expected deliverables

- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/records_criteria.rs (create)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the check_records_measure reading gate or its wording in reading/surveys.rs
- Editing existing entries in docs/decisions.md or anything under records/
- Changing athena.toml, defaults.toml or workflows
- Running criteria that do not name real records (fixture-only criteria)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion checks that a status-measure job is stopped or sent back before the verifier, not just that code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-producer test plus fmt, clippy and test criteria.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG.md entry and doc comments written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Fake-producer run through `Athena::go`, in the shape of `a_failing_check_is_sent_back_before_the_verifier_and_the_fix_reaches_it` (tests.rs:4543). The test commits `crates/athena-cli/Cargo.toml` (name `athena-cli`) with `commit_file`, so the package gate passes for real. The scripted reader adds a behavior criterion: "Run `cargo run -p athena-cli -- status --snapshot evidence`: on the real records the landing time it prints is within the expected range 2 to 10 minutes taken from the commit times in history.json". The producer and verifier report it. The producer script adds FIXED.txt only when its prompt holds `## Sent back` (as `fixing_file_on_send_back`, tests.rs:4493). The `argv_seam` maps the inner `cargo run` argv to an `sh -c` that prints a real-shaped status block: the header (the `snapshot    <evidence path>` line first, history and job counts, hashes, dates), the throughput lines, a `median per landed job over the last 3 measured: 4.0 min (reading 0.0 · planning 0.0 · producer 1.0 · checks 0.0 · verifier 1.0 · landing and rebuild 0.0)` line, and a per-job line `landing time 4.0 min (20261101-101010-aaaa 3.0 · 20261101-111111-bbbb 3.0 · 20261101-121212-cccc 3.0)` before FIXED.txt exists and `(... 3.0 · ... 6.5 · ... 4.2)` after it, plus a cargo-style `--> src/x.rs:120:9` line on stderr. The per-job line stands for what a change to a per-job measure prints; today's status prints none (stated limit). Run: `cargo test -p athena-core records_loop_constant_measure_is_sent_back_and_the_fix_reaches_the_verifier`. Observes: Pass 1 prints the constant job-id list, so the producer is sent back and the UI shows `producer   sent back: records criterion 1 ... same value for every item`. The second producer prompt holds `## Sent back`, the command with the job's real evidence path, the judged line and the output. Pass 2 prints varied in-range figures and the verifier runs once, so the counts (producer, verifier) are (2, 1). The one verifier prompt holds `records-criterion-1`, the command and pass 2's output. The header numbers, the stage-median line (equal 0.0 stages, not job ids) and the stderr line do not change the outcome.
  Fails without the behavior: With no pre-verifier run the constant output is never seen: the producer count is 1, no `producer   sent back: records criterion 1` line exists, the second prompt holds no command or output, and the verifier prompt holds no `records-criterion-1`.
  Test seam: Production entry `Athena::go` -> `producer_loop` -> `loop_records_criteria`. Guards: not nested, not pinned, a non-empty change, a selected criterion, package gate passes, reach passes. Fixture: a `FakeAgentDriver` script (exists: `editing`, `scoped`, `fixing_file_on_send_back`, `recording_verifier_prompts`, tests.rs:3158, 3945, 4493, 4509) plus a new helper that adds the behavior criterion (plan step 8). `commit_file` exists (tests.rs:358). Injected by `athena_with` (exists, tests.rs:191) and by `records_criteria::argv_seam::install` (thread-local, added by plan step 2, mirrors `demo_seam` at checks.rs:585-615). The mapper replaces only the inner `cargo run` argv, so the redirect, reading, package gate, judging and send-back are production code.
- Criterion 2: Same harness as criterion 1, with the seam printing a block whose per-job line is `landing time 4.6 min (20261101-101010-aaaa 3.0 · 20261101-111111-bbbb 6.5 · 20261101-121212-cccc 4.2)` against the range 2 to 10 minutes, and the verifier prompts kept with `recording_verifier_prompts`. Three tests: (a) the in-range test asserts counts, prompt and lines; (b) a verifier script takes the evidence path from its prompt (`evidence_path_in`, tests.rs:851), sets `commands_run` and the criterion's `commands` to the exact command (`cargo run -p athena-cli -- status --snapshot <real path>`), and the job reaches Landed; (c) a verifier script cites the same command with an empty `commands_run`, and the job does not land, with the D43 finding `cited but not seen running`. A briefs unit test asserts the rendered entry's sentence: Athena's own run, it does not count as the verifier's command, run it in your snapshot and cite it. Run: `cargo test -p athena-core records_loop_measure_in_range_is_recorded_for_the_verifier && cargo test -p athena-core records_loop_job_lands_when_the_verifier_runs_and_cites_the_command && cargo test -p athena-core records_loop_a_verifier_citing_athenas_run_without_running_it_is_unmet && cargo test -p athena-context records_run`. Observes: (a) The counts are (1, 1), no UI line starts with `producer   sent back`, and the single verifier prompt holds `records-criterion-1`, the command with the real evidence path, the judged `landing time` line and a result sentence saying the figure 4.6 min is inside the range 2 to 10 minutes, with the sentence that this is Athena's run and must be run and cited again. The mapper received the evidence path in place of the `evidence` token. (b) `finished.state` is `Landed`: the verdict counts the command the verifier ran. (c) The state is not `Landed` and the finding evidence says `cited but not seen running`. The briefs test passes on the new sentence.
  Fails without the behavior: Without the run and its hand-off through `Reported`, `Built` and `verification_stage`, the verifier prompt holds no `records-criterion-1`, command or output, and the (a) assertions fail. A wrong in-range judgement would send the producer back and make the producer count 2. Without the sentence the briefs test fails. (b) and (c) pin the verdict behavior (choice b) so a later change to verify.rs that began counting Athena's run would be seen.
  Test seam: `Athena::go` with `athena_with` (exists, tests.rs:191), the `editing` and `recording_verifier_prompts` scripts (exist), `commit_file` (exists), a new verifier-script wrapper in tests.rs (plan step 8) that sets `commands_run` and `commands`, `FakeTurn.commands_run` (exists, fake.rs:21), and `argv_seam` (plan step 2). The briefs test calls `verifier_brief_with_evidence` on a `CheckResultView` with `records_run` (plan step 6).
- Criterion 3: Two end-to-end tests and the `reach` unit tests. (a) `records_loop_outside_the_range_is_sent_back`: the per-job line is `landing time 273.8 min (...varied job-id list...)` before FIXED.txt and `4.0 min` after; the send-back names the figure 273.8 and the range 2 to 10 minutes; counts (2, 1). (b) `records_loop_unreachable_snapshot_ends_the_job_before_the_verifier`: the producer script takes the evidence path from its prompt (`evidence_path_in`, tests.rs:851), calls `crate::snapshot::unlock` and removes the directory, as a snapshot cleaned away mid-job would be (the unlock-and-remove pattern is in tests.rs:859-948). The test repository holds the `athena-cli` package file, so the package gate passes and reach is what fails. (c) Unit tests `reach_*` in records_criteria.rs call `reach(evidence, &Policy)` with `Policy` values built as at policy.rs:992-1016 and 1380-1411: evidence `None`, evidence at another path, evidence correct. Run: `cargo test -p athena-core -- records_loop_outside_the_range_is_sent_back records_loop_unreachable_snapshot_ends_the_job_before_the_verifier records_criteria::tests::reach_`. Observes: (a) The send-back text holds the out-of-range figure and the range, then the corrected pass goes on to the verifier. (b) `finished.state` is `Failed`; the reason names `<state>/jobs/<id>/evidence` in the placeholder form (no real state path) and says it cannot be reached from the producer's workspace; the verifier count is 0 and the producer count is 1. (c) With evidence `None` or at another path `reach` returns an error naming the placeholder path and that the producer's policy gave no read access; with the right evidence it passes.
  Fails without the behavior: Without the out-of-range judgement the 273.8 figure passes and no send-back line appears. Without the reach check the job reaches the verifier (verifier count 1) or the cargo run is attempted on a missing directory, so the verifier-count-0 and named-snapshot assertions fail. Without the policy half the `reach_` tests for evidence `None` fail.
  Test seam: `Athena::go` -> `producer_loop` -> the reach check inside `loop_records_criteria` (production code: `is_dir`, `read_history`, the policy-file parse, `evaluate`). The producer script of (b) edits the real evidence directory through std::fs (the `Script` closure receives its prompt: exists, fake.rs:43, 97). (a) uses `argv_seam` (plan step 2). (c) calls the production function `reach` directly with `Policy` values (no double).
- Criterion 4: Content check by reading the diff: the call site, the own-process run, and the evidence in both hand-offs. Run: `git diff -- crates/athena-core/src/run/producer.rs crates/athena-core/src/run/records_criteria.rs crates/athena-core/src/run/mod.rs crates/athena-core/src/run/verification.rs crates/athena-context/src/briefs.rs`. Observes: In producer.rs the call `self.loop_records_criteria(...)` sits after `let check_failures = self.loop_check_failures(...)` and before the `if unmet.is_empty() ... { break Some(report); }` test, which also tests the records problems. The call runs in Athena's own process through `SnapExec` and not in the producer's sandbox. `append_records_problems` adds the command, judged lines and output to `attempt_brief`. verification.rs pushes one `records-criterion-<n>` `CheckResultView` per run. briefs.rs renders `records_run` with the command, the output and the sentence that it is Athena's run. Every stored text goes through the state-root replacement before it is kept.
- Criterion 5: Content check by reading the diff. Run: `git diff -- CHANGELOG.md`. Observes: One added bullet at the top of `### Added` under `## [Unreleased]`, no existing line changed. It says Athena runs each behavior criterion that names real records and gives a `status --snapshot` command, after the producer's report and the in-loop checks and before the verifier, only where the repository has the `athena-cli` package. The command and output go to the producer's send-back and the verifier's brief. A figure outside the stated range, or a list of three or more job ids that all show the same value, is sent back through the shared send-back. It states that today's status output prints no per-job list, so the constant check can fire only when the change prints one. An unreachable snapshot (missing, or no read grant for the producer) ends the job early naming the snapshot. What is not judged is only recorded; a non-zero exit is recorded and not sent back. The verifier must run and cite the command itself. A candidate taken through the resume shortcut gets no records run. The tests are named and D183 is cited.
- Criterion 6: Content check by reading the diff; `git diff --stat` shows no path under records/. Run: `git diff -- docs/decisions.md`. Observes: Added lines only, after D182: a `## D183` heading and a `- **Provenance:**` bullet; no existing entry edited. The decision-log structure check in the full `cargo test` (`mirrors`, crates/athena-core/src/mirrors.rs) passes on the appended entry.
- Criterion 7: Run the whole workspace, including the existing producer-loop tests (the in-loop check tests at tests.rs:4543-4940 and the scope, mirror and changelog send-backs) and the refusal corpus (`cargo test -p athena-core --test refusal_corpus`, which holds `records-measure-change-with-real-records-range-history-json`, `records-measure-change-with-real-records-range-git-log` and `refusal-rate-declaration-wrapped-line`; the plan changes no gate). New tests that show no unrelated change: `records_loop_fixture_only_criteria_run_nothing` (behavior criteria that are `cargo test` fixtures, one with `status --snapshot` and no `real records`, one with `real records` and no `status --snapshot`; the seam counts calls); `records_loop_without_an_athena_cli_package_records_not_run` (a selected criterion in a repository without the package: a `not run` entry, seam count 0, no send-back, no reach check); unit tests for `athena_cli_package` (with the package, without it, another package name); and the judge unit tests on literal lines copied from throughput.rs:794-805, 810, 818, 832, 837: (i) `spent on attempts that did not land: 1.23 USD over the last 5 landing(s); the 5 landings before: 0.40 USD` with range 1 to 2 USD judges only 1.23 and gives no problem, with range 0.1 to 0.5 USD the problem names 1.23; (ii) `landed on the first attempt: 60% over the last 5 landing(s); the 5 landings before: 20%` with range 50 to 70 percent judges only 60%; (iii) `build wait 3.0 s over 4 check(s), longest 1.0 s` with range 2 to 5 seconds judges only 3.0 s, and `check(s)` is not a unit; (iv) `median per landed job over the last 3 measured: 4.0 min (reading 0.0 · planning 0.0 · producer 0.0 · checks 0.0 · verifier 0.0 · landing and rebuild 0.0)`, with and without a criterion that says per job, gives no constant and judges only 4.0 min; (v) `throughput 6 landed (2026-11-10 3 · 2026-11-11 3 · 2026-11-12 3)` is never a constant list; (vi) a job-id list of three equal items is constant, two equal items are not, three with one different are not, a list with one non-job-id label is not a per-item list, and a constant is caught with no stated range; (vii) the unit alias table, one test per alias against a range of the family's long name, and a figure in another family is not judged; (viii) header counts, dates, hashes, job counts and cargo `--> file:line:col` lines are never judged. `records_loop_real_throughput_text_is_judged_on_the_named_line` runs a job to Landed, writes the evidence snapshot of the repository head (`crate::snapshot::write`, unlock at the end), reads the real `Throughput::text()` through `crate::snapshot::status`, prepends the header in the shape of display.rs:1196-1249 and judges it. Run: `cargo test`. Observes: All tests pass. The seam's call count is 0 for the non-matching criteria and for the no-package repository. On the real text, a criterion naming `idle between jobs` with range 0 to 100000 minutes is Passed with exactly the `idle between jobs` line judged; the same criterion with range 1000 to 2000 minutes is a problem; a criterion naming `landing time` (no line holds it) is not judged and sends nothing back although the block holds many numbers; a criterion naming `throughput` judges the `throughput N landed (...)` line for figures with a unit only (none) and never calls the daily list constant. If `Throughput::compute` returns `None` for the core harness job, the producer moves the one real-text test to crates/athena-cli/tests with the judge reached through a documented `pub` re-export and reports which.
  Fails without the behavior: If selection were too broad the fixture-only criteria would call the seam and the counter assertion would fail. If the package gate were missing the no-package test would run the seam. If judging took every figure on a line, tests (i) to (iii) would send back or judge 0.40 USD, 20% or the longest wait. If judging took lists of non-job-id labels, tests (iv) and (v) would send back. The existing producer-loop tests would fail if send-back behavior had changed.
  Test seam: Production entry `Athena::go` for the loop tests and the pure functions `select`, `judge`, `athena_cli_package` and `reach` for the unit tests; `argv_seam` (plan step 2) and its call counter owned by the test; `athena_with`, `run_job`, `Counts` and `commit_file` exist (tests.rs:191, 4001, 3043, 358). The real-text test uses the production `crate::snapshot::write`, `status` and `Throughput::text()`; only the header lines are written by the test.
- Criterion 8: Run the formatter check on the result. Athena also formats the work tree at the end of each producer pass (producer.rs:448). Run: `cargo fmt --check`. Observes: Exit 0 and no diff output.
  Fails without the behavior: Any unformatted line in records_criteria.rs, tests.rs, briefs.rs or producer.rs makes the check exit non-zero.
  Test seam: The command itself; no seam.
- Criterion 9: Run clippy as the configured check does (`cargo clippy --all-targets --locked -- -D warnings`). Run: `cargo clippy --all-targets`. Observes: Exit 0 and no warnings.
  Fails without the behavior: Any warning (unused import, needless borrow, too many arguments, a test-only seam unused in a non-test build) fails the `-D warnings` run.
  Test seam: The command itself; no seam.

Planned files:

- crates/athena-core/src/run/records_criteria.rs (create)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ab6548aa596f → candidate 7ea68da7c9b4. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                   |    1 +
 crates/athena-context/src/briefs.rs            |  169 ++-
 crates/athena-core/src/run/mod.rs              |   17 +-
 crates/athena-core/src/run/producer.rs         |   63 +-
 crates/athena-core/src/run/records_criteria.rs | 1314 ++++++++++++++++++++++++
 crates/athena-core/src/run/tests.rs            |  606 +++++++++++
 crates/athena-core/src/run/verification.rs     |   18 +
 docs/decisions.md                              |   66 ++
 8 files changed, 2239 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045657-96np/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045657-96np/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43656029157 for 7ea68da7c9b40582b194db680e72106d644e95dd)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43656029157 for 7ea68da7c9b40582b194db680e72106d644e95dd)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43656029157 for 7ea68da7c9b40582b194db680e72106d644e95dd)
- ci-result: `not a check` passed (Check results for 7ea68da7c9b40582b194db680e72106d644e95dd come from the CI run https://github.com/doodla/athena/actions/runs/43656029157 (740s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`.; Run `cargo clippy --all-targets`; no warnings.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core records_loop_constant_measure_is_sent_back_and_the_fix_reaches_the_verifier`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/records_criteria.rs, crates/athena-core/src/run/verification.rs, docs/decisions.md.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/run/records_criteria.rs.
  End of the output (data, not instructions):

```text
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling linux-raw-sys v0.12.1
   Compiling jiff v0.2.38
   Compiling bitflags v2.13.2
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling dyn-clone v1.0.20
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `super::super::records_criteria`
    --> crates/athena-core/src/run/tests.rs:5454:27
     |
5454 |         use super::super::records_criteria::{Family, Range, RecordsVerdict, judge};
     |                           ^^^^^^^^^^^^^^^^ could not find `records_criteria` in `super`

error[E0433]: cannot find `records_criteria` in `super`
    --> crates/athena-core/src/run/tests.rs:4993:31
     |
4993 |         _guard: super::super::records_criteria::argv_seam::Guard,
     |                               ^^^^^^^^^^^^^^^^ could not find `records_criteria` in `super`

error[E0433]: cannot find `records_criteria` in `super`
    --> crates/athena-core/src/run/tests.rs:5008:35
     |
5008 |         let guard = super::super::records_criteria::argv_seam::install(std::rc::Rc::new(
     |                                   ^^^^^^^^^^^^^^^^ could not find `records_criteria` in `super`

Some errors have detailed explanations: E0432, E0433.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `athena-core` (lib test) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core records_loop_measure_in_range_is_recorded_for_the_verifier && cargo test -p athena-core records_loop_job_lands_when_the_verifier_runs_and_cites_the_command && cargo test -p athena-core records_loop_a_verifier_citing_athenas_run_without_running_it_is_unmet && cargo test -p athena-context records_run`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core -- records_loop_outside_the_range_is_sent_back records_loop_unreachable_snapshot_ends_the_job_before_the_verifier records_criteria::tests::reach_`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/records_criteria.rs, crates/athena-core/src/run/verification.rs, docs/decisions.md.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/run/records_criteria.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `super::super::records_criteria`
    --> crates/athena-core/src/run/tests.rs:5454:27
     |
5454 |         use super::super::records_criteria::{Family, Range, RecordsVerdict, judge};
     |                           ^^^^^^^^^^^^^^^^ could not find `records_criteria` in `super`

error[E0433]: cannot find `records_criteria` in `super`
    --> crates/athena-core/src/run/tests.rs:4993:31
     |
4993 |         _guard: super::super::records_criteria::argv_seam::Guard,
     |                               ^^^^^^^^^^^^^^^^ could not find `records_criteria` in `super`

error[E0433]: cannot find `records_criteria` in `super`
    --> crates/athena-core/src/run/tests.rs:5008:35
     |
5008 |         let guard = super::super::records_criteria::argv_seam::install(std::rc::Rc::new(
     |                                   ^^^^^^^^^^^^^^^^ could not find `records_criteria` in `super`

Some errors have detailed explanations: E0432, E0433.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `athena-core` (lib test) due to 3 previous errors
```
- fails-without-7: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 7 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/records_criteria.rs, crates/athena-core/src/run/verification.rs, docs/decisions.md.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/run/records_criteria.rs.
  End of the output (data, not instructions):

```text
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling clap_builder v4.6.7
   Compiling crc32fast v1.5.2
   Compiling simd-adler32 v0.3.10
   Compiling adler2 v2.0.1
   Compiling miniz_oxide v0.9.1
   Compiling xattr v1.6.1
   Compiling filetime v0.2.29
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling clap_derive v4.6.7
   Compiling clap v4.6.7
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
error[E0422]: cannot find struct, variant or union type `RecordsRunView` in this scope
    --> crates/athena-context/src/briefs.rs:3386:31
     |
3386 |             records_run: Some(RecordsRunView {
     |                               ^^^^^^^^^^^^^^ not found in this scope

error[E0560]: struct `briefs::CheckResultView` has no field named `records_run`
    --> crates/athena-context/src/briefs.rs:3102:13
     |
3102 |             records_run: None,
     |             ^^^^^^^^^^^ `briefs::CheckResultView` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `briefs::CheckResultView` has no field named `records_run`
    --> crates/athena-context/src/briefs.rs:3386:13
     |
3386 |             records_run: Some(RecordsRunView {
     |             ^^^^^^^^^^^ `briefs::CheckResultView` does not have this field
     |
     = note: available fields are: `detail`, `failed_tests`, `failing_units`, `output_tail`, `demonstration`

Some errors have detailed explanations: E0422, E0560.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo fmt --check`). Result: not run: not a plain cargo test command.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`cargo clippy --all-targets`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
