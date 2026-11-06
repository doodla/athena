# Job 20261105-232005-y3wv: verify "Replay new refusal rules over past readings inside the producer loop"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Nothing requires a new refusal or classification rule to be measured against real past requests befo
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: new-refusal-rule-not-replayed-against-real-requests (20261028-034817-rhs5)
> Cause: Nothing requires a new refusal or classification rule to be measured against real past requests before it lands. Acceptance criteria and the verifier test that the rule exists and fires on its intended cases. Neither measures how often it w...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261028-034806-d8g0 (job 20261027-054211-a2n5); prevention it proposed: Add a check that runs for any change to an admission or refusal rule. It replays the new rule over the recorded past requests (accepted ones included) and reports the share it would newly refuse. The job fails if that share exceeds a set th...
> Introspection 20261105-231947-72g2 (job 20261105-093556-zyt3); prevention it proposed: Run the replay of past accepted readings as a check inside the producer's loop. It would give the producer the list of newly refused readings and the measured share. Cap the share with a numeric limit set in the request or config. Make the ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the replay of past accepted readings (`replay_stage` in run/mod.rs) runs only after the producer finishes, as a hold at verification. The producer never sees the newly refused list or the measured share, so a rule such as job 20261105-093556-zyt3's bug-fix check lands at 18% newly refused and is rejected after the work is done. Fix: run the replay as a check in the producer's loop and feed the list and share back to it. Audience: Athena's operator and the producer. Job 20261027-054211-a2n5 landed and is kept; job 20261105-093556-zyt3's candidate is dropped.

Assumptions:
- Code survey: I read crates/athena-core/src/run/mod.rs (`produce_and_verify`, `replay_stage`, `replay_run`, `replay_checkers`), crates/athena-core/src/reading.rs (`replay_cases`, `replay`, `REPLAY_HOLD_PERCENT`, `replay_declaration_of`, `check_governance_coverage`, `refusing_check`) and crates/athena-core/src/verify.rs (the replay blocker). Today the replay runs once after the checks pass and before verification, and it never fails the job on its own. No record or data file is added under crates/, so `check_record_file`, `AnyRecord` and the round-trip test are not affected. The share is measured directly: base and candidate `check-reading` builds are run over the newest accepted readings, and the newly refused ones are counted.
- Job 20261027-054211-a2n5 (landed, verdict pass): its governance-coverage work in reading.rs, admission.rs, CHANGELOG.md and docs/architecture.md is kept as it is. Its findings are minor: broad keywords, and a keyword table test that has its own phrasings. They are not the cause here, so this job does not address them.
- Job 20261105-093556-zyt3 (rejected): its candidate is dropped, including `check_root_cause_diagnosis`, the changes to briefs.rs, refusal_corpus.rs and run/mod.rs, and its CHANGELOG and architecture edits. Its findings are addressed by the generic mechanism: the producer now sees the newly refused list and the measured share during its own loop, so false positives such as 'the fix' in 'fails without the fix' show up before verification. That bug-fix check is not re-landed here.
- The numeric limit is the existing one: `REPLAY_HOLD_PERCENT` (10) or the share the criteria declare plus 5 points. No new config key, so athena.toml stays unchanged.
- The in-loop replay reuses `replay_cases`, `replay` and `replay_checkers`, and runs after the producer's checks pass. When it exceeds the limit, the list and share go back to the producer as a send-back, within the existing retry budget. When the replay cannot run, it says `no replay: <reason>` and does not count as a pass.
- This job adds no new refusal rule, so the 'expected N% of newly refused readings' criterion does not apply.

## Acceptance criteria

1. [content] The root cause is named in the change description and CHANGELOG.md: nothing makes the producer measure a new or tightened refusal rule against past accepted readings while it works. The replay (`replay_stage` in crates/athena-core/src/run/mod.rs) runs once after the producer finishes, as a verification hold. Checked by reading the diff.
2. [behavior] The test(s) added in crates/athena-core/src/run/tests.rs (the producer-loop replay tests) fail on the base tree and pass with the change. They use a fake candidate checker that newly refuses more than the limit of a corpus of recorded accepted readings. They assert that the producer is sent back, that its brief names the newly refused job ids and the measured percent, and that the loop ends once the share is under the limit. Run `cargo test -p athena-core`.
3. [behavior] A test in crates/athena-core/src/run/tests.rs or crates/athena-core/src/reading.rs shows that the in-loop replay respects the declared share and the 10% default (`REPLAY_HOLD_PERCENT`): exactly the limit passes, one over is sent back. It also shows that an unrunnable replay reports `no replay: <reason>` and is not treated as a pass. Run `cargo test -p athena-core`.
4. [behavior] A test shows that a job whose reading adds no refusal rule, or whose replay has no history, is not sent back, and that an empty corpus is reported as `no replay` and not as success.
5. [content] docs/architecture.md and CHANGELOG.md describe the replay as a check inside the producer loop, with its limit, and say the post-hoc verification hold stays as the backstop.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass from the repository root.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)
- crates/athena-core/src/run/send_back.rs (modify)

## Out of scope

- Re-landing `check_root_cause_diagnosis` or any other new refusal rule from job 20261105-093556-zyt3: its candidate files (crates/athena-core/tests/refusal_corpus.rs and its other edits) are dropped
- Reworking job 20261027-054211-a2n5's governance keyword table or its crates/athena-core/src/admission.rs changes: that work is kept as it is
- Editing athena.toml, defaults.toml, workflows or other governance files
- Editing prompt files (the reader and verifier prompts have no token headroom)
- Changing the replay hold thresholds or the verifier's blocker in crates/athena-core/src/verify.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests, plus fmt, clippy and test passing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is where the producer's loop lacks the replay, not in one job's rule.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the producer is sent back with the list and share.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty corpus or an unrunnable replay reports `no replay`, never a pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The limit is enforced in the producer loop by code.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs cargo build, test, clippy and fmt --check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff for the cause statement. Run: `git diff -- CHANGELOG.md crates/athena-core/src/run/mod.rs crates/athena-core/src/run/producer.rs`. Observes: The CHANGELOG [Unreleased] entry and the doc comments on `loop_replay` and `producer_loop` both state the root cause: nothing made the producer measure a new or tightened refusal rule against past accepted readings while it worked; `replay_stage` ran once after the producer finished, as a verification hold. The change is in `producer_loop`, not in one rule.
- Criterion 2: Behavior test `producer_is_sent_back_with_the_newly_refused_list_and_share_then_the_loop_ends_under_the_limit` in crates/athena-core/src/run/tests.rs, driven through `Athena::go`. A second test, `the_post_hoc_hold_reuses_the_in_loop_measurement_for_the_same_tree`, counts candidate-checker calls. Run: `cargo test -p athena-core producer_is_sent_back_with_the_newly_refused_list_and_share`. Observes: The producer is invoked twice. Prompt 1 has no `## Sent back`. Prompt 2 contains `## Sent back`, the three seeded newly refused ids, `30%` and `limit 10%`. The UI shows `replay     in the loop: 3 of 10 accepted readings newly refused (30%, limit 10%)` and, after the second pass, `replay     in the loop: 0 of 10 ...`, both before the `checks` line. The job ends `OutcomeState::Landed` and `replay.json` has 0 newly refused. In the reuse test one passing pass makes the candidate closure run exactly 10 times (the in-loop pass only; the hold reuses it), and the recorded Replay equals the in-loop run.
  Fails without the behavior: On the base tree the producer runs once and its prompt has no `## Sent back`, and no `replay     in the loop:` line exists. The post-hoc hold then rejects the job with 3 of 10 newly refused. The prompt-count, brief-content, UI-line and Landed assertions all fail.
  Test seam: Production entry point `Athena::go` (via the existing `finished_job` helper), reaching `produce_and_verify`, `producer_loop`, the new `loop_replay` and, for the reuse test, `replay_run`. Guards: the change list must touch a path under `crates/`, and the history must be non-empty. Fixture `seed_accepted_jobs(&athena, 10, 3)` (exists, tests.rs:1417) seeds 3 marked readings of 10. Fake: `crate::replay_seam::install(real_rules(), candidate)` (exists, lib.rs:174), where the candidate closure refuses the marker title only while a shared `Arc<AtomicUsize>` producer-run counter is below 2 and bumps a second call counter. `athena_with` (exists, tests.rs:187) takes a script wrapping `base_script` that bumps the counter on `Role::Producer`, edits `crates/demo/src/lib.rs` and README.md, and records each `user_prompt`. The closures hold only counters; the replay, limit, slot and send-back text are production code. The seam is read after the slot is taken, so the production slot path runs. The tests are added by plan step 8.
- Criterion 3: A unit test on the limit and boundary, plus three run tests through `Athena::go`. Run: `cargo test -p athena-core replay_limit_is_declared_share_plus_five && cargo test -p athena-core in_loop_replay_exactly_at_the_default_limit && cargo test -p athena-core an_unrunnable_replay_says_no_replay_in_the_loop && cargo test -p athena-core an_in_loop_replay_waits_for_a_build_slot`. Observes: `replay_limit_percent(None)` is 10 and `Some(20)` is 25. With total 50, 5 newly refused and no declared share is not held and 6 is; with declared 20 and total 20, 5 is not held and 6 is. `replay()` over closure checkers shows the same boundary. Run test: 1 of 10 marked (10%) gives one prompt, no `## Sent back`, and the line `(10%, limit 10%)`; 2 of 10 (20%) gives a second prompt naming `20%` and the limit. Unrunnable test: it calls `seed_accepted_jobs` first, so the empty-history reason is not reached. With no seam and the fake driver, a UI line `replay     in the loop: no replay: the driver is a fake, so nothing is built` appears before the `checks` line. No `in the loop: 0 of` line appears, there is one producer prompt, and the post-hoc `replay.no_replay` is Some. Slot test: config `[drain.pipeline] build_wait_s = 1`, a test-held slot via `SlotQueue::try_acquire`, the seam installed and history seeded. The line is `replay     in the loop: no replay: no build slot was free within build_wait_s`, one producer prompt, and `go` returns a `SlotWait` error from the checks, as in `a_check_that_waits_past_build_wait_s_holds_the_job` (tests.rs:2759).
  Fails without the behavior: The unit test does not compile without `replay_limit_percent`. In the 20% run test the base tree sees one producer prompt, so the second-prompt assertion fails. The unrunnable test asserts a line starting `replay     in the loop: no replay:`, which the base tree never prints (it prints only the unprefixed post-hoc line). The slot test asserts that line carries `no build slot` while the only slot is held; the base tree prints none, and an in-loop replay that skipped the slot would measure instead.
  Test seam: Entry points `replay_is_held`, `replay_limit_percent`, `replay` (reading.rs) and `Athena::go`. Fakes: closure `ReadingChecker`s in the unit test; `seed_accepted_jobs`; `replay_seam::install` with `real_rules()`; `repo_with_config`, `slot_queue` and `take_slot_soon` (exist, tests.rs:2346-2380); `Lines` UI (tests.rs:167). The seam is deliberately not installed in the unrunnable test; the fake driver's own skip in `replay_checkers` (mod.rs:1818) makes it unrunnable. `replay_limit_percent` is added by step 1; `loop_replay` and its slot by step 5; the tests by steps 1 and 8.
- Criterion 4: Two run tests through `Athena::go` plus a pure unit test of the path gate. Run: `cargo test -p athena-core a_job_whose_change_touches_no_reading_rule && cargo test -p athena-core an_empty_history_in_the_loop && cargo test -p athena-core touches_reading_rules`. Observes: (a) With the seam installed over 3 of 10 marked and the default README-only `base_script`, there is one producer prompt and no line containing `in the loop`; the post-hoc hold still rejects the job. (b) With no seeded jobs and a producer editing a `crates/` file, the line `replay     in the loop: no replay: no accepted readings in the history` (`NO_HISTORY_REASON`) appears before the `checks` line. There is one producer prompt and no `in the loop: 0 of 0` line. The post-hoc `replay.no_replay` equals `no replay: no accepted readings in the history`, so it is not success. Unit test: `README.md` false, `docs/x.md` false, `crates/athena-core/src/reading.rs` true, `Cargo.lock` true, empty list false.
  Fails without the behavior: (a) If the loop replayed every change, the README-only run would print an in-loop line and take a second prompt. (b) The base tree prints no `in the loop` line, so the line assertion fails; if an empty corpus counted as a pass, a `0 of 0` in-loop line or a missing `no replay` would fail it. The unit test fails if the gate is missing or wrong.
  Test seam: Entry point `Athena::go`; fixtures `seed_accepted_jobs`, `replay_seam::install`, `Lines` and `athena_with` (all exist). `touches_reading_rules` is a pure function added by step 1; the run tests are added by step 8.
- Criterion 5: Read the docs diff and run the briefs test that the producer brief names the replay and its limit. Run: `git diff -- docs/architecture.md CHANGELOG.md crates/athena-context/src/briefs.rs && cargo test -p athena-context producer_brief_tells_the_producer_the_replay_is_checked_while_it_works`. Observes: Both docs describe the replay as a check inside the producer loop. They give the limit (`REPLAY_HOLD_PERCENT`, or the declared share plus 5), the `crates/`/Cargo gate, the build slot, the single shared send-back, and `no replay: <reason>` never being a pass. They state that the same-tree measurement is reused by the hold, and that the post-hoc verification hold stays as the backstop. The CHANGELOG entry is under [Unreleased]. The briefs test asserts the producer brief mentions the in-loop replay and its limit.
  Fails without the behavior: The briefs.rs test fails on the base tree because the producer brief has no replay section.
  Test seam: `producer_brief` (exists, briefs.rs) is called directly by the test; no fake. The section and test are added by step 2.
- Criterion 6: Run the three repository checks from the repository root. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0, including every new test from criteria 2 to 4, the existing replay tests, the run_pin_tests send-back tests, the briefs.rs tests and `cargo test -p athena-core --test refusal_corpus`. The refusal corpus is unaffected because no `step!(check_` line is added (the extractor reads only those lines); I read the test names `every_acceptance_check_has_a_refusing_corpus_case` and `no_check_bypasses_refusing_check` and cite no result.
  Fails without the behavior: `cargo test --locked` fails on the base tree plus the new tests alone, because they call functions that do not exist. It also fails if the change breaks an existing test, for example a run_pin_tests send-back test, a prompt-count test or the replay tests at tests.rs:1488, 1605, 1667. Clippy fails on an unused import or dead wrapper; fmt fails on unformatted code.
  Test seam: The cargo commands themselves, on the producer's tree at the repository root; no fake. The tests they run use existing seams (`Athena::go`, `athena_with`, `seed_accepted_jobs`, `replay_seam::install`, `Lines`, `slot_queue`).

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f31fac8ed951 → candidate 6d2dfac3987a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  22 ++
 crates/athena-context/src/briefs.rs     | 103 ++++++++
 crates/athena-core/src/reading.rs       |  64 ++++-
 crates/athena-core/src/run/checks.rs    |   2 +-
 crates/athena-core/src/run/mod.rs       | 236 +++++++++++++++++-
 crates/athena-core/src/run/producer.rs  |  56 ++++-
 crates/athena-core/src/run/send_back.rs |  73 +++++-
 crates/athena-core/src/run/tests.rs     | 419 ++++++++++++++++++++++++++++++++
 docs/architecture.md                    |  19 ++
 9 files changed, 974 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-232005-y3wv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-232005-y3wv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42147358578 for 6d2dfac3987a32c578bcb211ab8af0e749e768f9)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42147358578 for 6d2dfac3987a32c578bcb211ab8af0e749e768f9)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42147358578 for 6d2dfac3987a32c578bcb211ab8af0e749e768f9)
- ci-result: `not a check` passed (Check results for 6d2dfac3987a32c578bcb211ab8af0e749e768f9 come from the CI run https://github.com/doodla/athena/actions/runs/42147358578 (676s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass from the repository root.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core producer_is_sent_back_with_the_newly_refused_list_and_share`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, docs/architecture.md.
  End of the output (data, not instructions):

```text
     |                    ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `touches_reading_rules` in this scope
    --> crates/athena-core/src/reading.rs:7768:18
     |
7768 |         assert!(!touches_reading_rules(&[]));
     |                  ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `touches_reading_rules` in this scope
    --> crates/athena-core/src/reading.rs:7769:18
     |
7769 |         assert!(!touches_reading_rules(&paths(&["README.md"])));
     |                  ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `touches_reading_rules` in this scope
    --> crates/athena-core/src/reading.rs:7770:18
     |
7770 |         assert!(!touches_reading_rules(&paths(&[
     |                  ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `touches_reading_rules` in this scope
    --> crates/athena-core/src/reading.rs:7774:18
     |
7774 |         assert!(!touches_reading_rules(&paths(&["docs/crates/x.md"])));
     |                  ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `touches_reading_rules` in this scope
    --> crates/athena-core/src/reading.rs:7775:17
     |
7775 |         assert!(touches_reading_rules(&paths(&[
     |                 ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `touches_reading_rules` in this scope
    --> crates/athena-core/src/reading.rs:7778:17
     |
7778 |         assert!(touches_reading_rules(&paths(&["README.md", "Cargo.lock"])));
     |                 ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `touches_reading_rules` in this scope
    --> crates/athena-core/src/reading.rs:7779:17
     |
7779 |         assert!(touches_reading_rules(&paths(&["Cargo.toml"])));
     |                 ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `send_back_text_with_replay` in this scope
   --> crates/athena-core/src/run/send_back.rs:308:13
    |
308 |             send_back_text_with_replay("BRIEF", &unmet, &[], None)
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `send_back_text_with_replay` in this scope
   --> crates/athena-core/src/run/send_back.rs:314:13
    |
314 |             send_back_text_with_replay("BRIEF", &unmet, &["src/a.rs".into()], Some((&v, 10)));
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 16 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-core replay_limit_is_declared_share_plus_five && cargo test -p athena-core in_loop_replay_exactly_at_the_default_limit && cargo test -p athena-core an_unrunnable_replay_says_no_replay_in_the_loop && cargo test -p athena-core an_in_loop_replay_waits_for_a_build_slot`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core a_job_whose_change_touches_no_reading_rule && cargo test -p athena-core an_empty_history_in_the_loop && cargo test -p athena-core touches_reading_rules`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff -- docs/architecture.md CHANGELOG.md crates/athena-context/src/briefs.rs && cargo test -p athena-context producer_brief_tells_the_producer_the_replay_is_checked_while_it_works`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
