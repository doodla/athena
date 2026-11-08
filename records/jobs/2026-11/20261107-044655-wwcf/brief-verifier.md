# Job 20261107-044655-wwcf: verify "Find and remove the cause of the longer producer stage"

## Request (verbatim)

> Priority: high
> 
> Throughput slowdown: producer grew from 12.7 to 23.5 min per landed job
> 
> Athena's throughput review found that the median minutes per landed job rose from 918.1 to 1604.8, more than 25%, since the previous throughput review. Find why the stage that grew most now takes longer and remove the cause.
> 
> Stage that grew most: producer, a median of 23.5 minutes per landed job now, 12.7 before.
> Stages whose median minutes grew, in minutes per landed job before to now: producer 12.7 to 23.5, planning 1.7 to 9.0, checks 0.3 to 0.7, verifier 3.4 to 3.6.
> 
> The medians are over the newest landed jobs that Athena measured. The status section of athena status prints the same stage medians.
> 
> Acceptance:
> - The cause of the longer producer stage is found and stated in the change or in a note under docs/design/
> - The change shortens that stage, and the next throughput review shows the median minutes per landed job falling
> 
> Files: crates/, docs/design/

## Reading (accepted by the user)

Investigate why the median producer stage per landed job grew from 12.7 to 23.5 minutes, using the stage-timing code in athena-core and the producer path. Fix the cause and write it down in a note under docs/design/. Audience: Athena maintainers reading the next throughput review.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (stage medians), crates/athena-core/src/store.rs and snapshot.rs (stage timing records), crates/athena-cli/src/main.rs (entry point and order of run stages), and crates/athena-data/src/records.rs (records validation allow-list). The change adds no new record or data file under crates/. If it does, it must be covered by check_record_file, AnyRecord and the round-trip test. Stage minutes are measured directly from recorded stage durations, not through a stand-in.
- Assumption: the cause is in how the producer stage is run (for example cold builds, repeated checks inside the producer, oversized prompt or context, or retries). The producer must confirm it from the code and the recorded timings available to it before choosing a fix.
- The rise in planning (1.7 to 9.0) may share the cause. A fix for it is out of scope unless it is the same root cause.
- Producer wall time cannot be re-measured inside a verifier run. The verifier checks the mechanism by tests and the note's cited evidence.
- Environment: none beyond the repository snapshot. No CI workflow, push or PR is needed.

## Acceptance criteria

1. [content] A note under docs/design/ states the cause of the longer producer stage. It cites the code paths and the measured numbers (12.7 to 23.5 minutes), and labels any estimate.
2. [behavior] Run cargo test. A new or changed test fails on the old behavior and passes on the new, showing the identified cause no longer adds time to the producer stage.
3. [behavior] Run cargo clippy and cargo fmt --check. Both are clean.
4. [content] The diff touches only the code on the cause path and its tests. It does not change stage-median reporting merely to make the numbers look better.
5. [content] The fix removes the cause itself, not a symptom such as a raised timeout or hidden timing. The note says what the next throughput review should show: a lower producer median.

## Expected deliverables

- docs/design/producer-slowdown.md (create)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/store.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the throughput review thresholds or the stage-median calculation
- Fixing the planning, checks or verifier stage growth unless it shares the producer's root cause
- Edits to athena.toml, defaults.toml, workflows or prompt files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The fix and tests are in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The criteria require fixing the cause, not the symptom.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The design note cites sources and labels estimates.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A test shows the behavior change.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria tie to a lower producer median in the next review.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the cause path.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the note with Read and check each cited path and symbol with Grep. The command shows the file exists. Run: `ls docs/design/producer-slowdown.md`. Observes: The note exists. It quotes the measured figures from the request (producer 12.7 to 23.5, planning 1.7 to 9.0 marked out of scope, checks 0.3 to 0.7, verifier 3.4 to 3.6, median total 918.1 to 1604.8). It states the cause with citations: briefs.rs:640, 856 and 863; athena-cli/Cargo.toml:16-32; athena.toml:80 and 111; producer.rs:76 and 361-365; D158; and 69057b9 (2026-11-06) against the 2026-11-04 review (d0ff1e9). It says the wrapper landed 2026-11-07 (9a97e24), so slot waits and the replay are not the cause of the window's rise, and that replay time is not producer stage time (`role_minutes`, status.rs:933-941). It lists candidates (a) to (d) with the record that settles each: `commands_run`, the producer invocation count per landed job, and the `self-check failed:` detail. It labels the cost of the build and the share of jobs affected as estimates, says no records were in the snapshot, and states what the next review should show, with the caveat about slot waits in its window.
- Criterion 2: Two tests. (1) In briefs.rs, `producer_briefs_do_not_tell_the_producer_to_build_for_the_mirror_check` builds a producer brief with `producer_brief` for a docs-only reading. It asserts the brief has the `## Files kept by hand` section, between the banned words and the checks. It asserts the section names xtask/shard-timings.txt and docs/decisions.md, and that no line of it contains `cargo test` or `--test mirrors`. (2) In run/tests.rs, `mirrors_the_producer_is_not_asked_to_build_the_mirror_target_and_the_loop_still_sends_the_drift_back` runs `mirror_job(&athena_source_repo(), vec![edit("crates/demo/tests/b.rs", "// b\n")], Vec::new())`. It asserts prompts[0] contains the mirror facts but not `--test mirrors` or `cargo test -p athena-cli`, and that the drift is still sent back: two producer runs, prompts[1] containing MIRROR_SENTENCE and `add the line `demo --test b 5.00``. The existing `mirrors_*` tests run unchanged. Run: `cargo test -p athena-context producer_briefs && cargo test -p athena-core --lib mirrors_`. Observes: Both new tests pass, with the existing `mirrors_a_new_test_file_without_a_timings_line_is_sent_back_with_the_file_and_the_fix` and `mirrors_a_clean_producer_is_not_sent_back`. The producer is given no workspace-wide build to run, and the in-process loop check still catches the drift it replaced.
  Fails without the behavior: On the old brief text, `producer_brief` renders `run `cargo test -p athena-cli --test mirrors` before you finish` in the 'Files kept by hand' section. The unit test's 'no cargo test or --test mirrors in the section' assertion fails, and so does the run test's assertion on prompts[0]. A variant that dropped the loop's mirror check would fail the run test's send-back assertions.
  Test seam: Production entry points: `athena_context::briefs::producer_brief` (called directly with the `ProducerInputs` fixture the neighbouring tests use, briefs.rs ~3243-3254, exists) and `Athena::go` through `mirror_job` (run/tests.rs:2501), which uses `athena_source_repo()` and the capturing fake agent `fixing_on_send_back` (exists). Guards: the work tree must be Athena's own source (`is_athena_source`) and the change must touch a file the timings entry watches, which `athena_source_repo()` and `edit("crates/demo/tests/b.rs", ...)` provide, as in the existing test at tests.rs:2519. Step 4 adds both tests. The brief builder and the loop's mirror check stay production code; the fake only records the prompt and makes the edit.
- Criterion 3: Run the two workspace checks on the finished tree, as Athena runs them: first `cargo fmt --all --check`, then the clippy command. They are two separate commands. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Both exit 0 with no diagnostics over production and test code. The renamed constant has no remaining references to the old name, and no test imports an unused item.
  Fails without the behavior: An unformatted line, a leftover reference to `MIRROR_CHECK_INSTRUCTION`, or an unused import in a replaced test makes one of the two commands exit non-zero.
  Test seam: The real commands over the whole workspace, no double. Both exist today.
- Criterion 4: Read the diff statistic and confirm that only the cause path and its tests changed. Run: `git diff --stat`. Observes: The stat lists exactly crates/athena-context/src/briefs.rs, crates/athena-core/src/run/tests.rs, docs/design/producer-slowdown.md and CHANGELOG.md. status.rs, store.rs, main.rs, slots.rs, slot_exec.rs, producer.rs, config.rs and athena.toml are absent, so no stage-median reporting, threshold, slot setting or governance file changed. `cargo test -p athena-core --lib throughput` passes unchanged.
- Criterion 5: Read the code diff and the note's closing section. Run: `git diff -- crates/athena-context/src/briefs.rs docs/design/producer-slowdown.md`. Observes: The code diff removes work the producer is told to do (a cold workspace-wide test build on every job). It changes no timeout (`build_wait_s`, the 300 s wrapper cap), no slot count, no send-back limit and no stage reporting. The note says the next throughput review should show a lower producer median, with the size an estimate. It names what is not expected to move: planning, checks, the verifier, and the send-back and slot-wait shares. It says that slot waits and CPU contention from `max_concurrency = 2` are in the next window and may hide part of the fall. It gives the way to check: `athena status` stage medians, the stored throughput comparison, and `commands_run` of recent producer invocations no longer holding `cargo test -p athena-cli --test mirrors`.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/producer-slowdown.md (create)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5887ebab25be → candidate 222c267fff5b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   9 +++
 crates/athena-context/src/briefs.rs |  46 ++++++-----
 crates/athena-core/src/run/tests.rs |  23 ++++++
 docs/design/producer-slowdown.md    | 151 ++++++++++++++++++++++++++++++++++++
 4 files changed, 208 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-044655-wwcf/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-044655-wwcf/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42428624085 for 222c267fff5b0291a8300c76f9ea054b59c39473)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42428624085 for 222c267fff5b0291a8300c76f9ea054b59c39473)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42428624085 for 222c267fff5b0291a8300c76f9ea054b59c39473)
- ci-result: `not a check` passed (Check results for 222c267fff5b0291a8300c76f9ea054b59c39473 come from the CI run https://github.com/doodla/athena/actions/runs/42428624085 (632s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo clippy and cargo fmt --check. Both are clean.)
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-context producer_briefs && cargo test -p athena-core --lib mirrors_`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
