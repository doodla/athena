# Job 20261029-142902-5x4h: verify "Add read-ahead and stale-reading re-check to the drain (A5)"

## Request (verbatim)

> Priority: urgent
> Async drain build job A5 (docs/design/async-drain.md, decision D145). Read-ahead (job 5): read, and plan hard jobs, up to read_ahead items ahead of the running one, with the stale-reading re-check of section 4. Depends on 20261029-142849-04d9. With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/reading.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A5 of the async drain (D145): while job N runs, the drain reads and plans hard jobs up to read_ahead items ahead, re-checking a reading for staleness when its job starts (design section 4). Follows job 20261029-142849-04d9, whose [drain.pipeline] config is kept as is. With max_in_flight = 1 the drain behaves as today. The audience is the maintainer who runs the drain and the verifier who checks it.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (entry points drain and drain_concurrent, Steps trait, Item, Limits) and docs/design/async-drain.md sections 4, 11 and 12 (read_ahead, job 5). No record or data file is added under crates/, so check_record_file and AnyRecord are unaffected. The property is measured directly: tests drive drain/drain_concurrent with a fake Steps and count read and plan calls made before the prior job finishes.
- Prior job 20261029-142849-04d9 (landed, verdict pass): its config.rs, CHANGELOG.md and design-doc work is kept and not edited here. Its findings are addressed as follows. The stale design-doc line 21 and the terse DrainPipeline error message are out of scope for this job. The MAX_CHECK_SLOTS_CEILING nit is not a defect and needs no fix. read_ahead is read from the existing Config.drain.pipeline, which needs no config change.
- No new refusal rule is added, so no newly-refused-readings percentage applies.
- Reading changes in crates/athena-core/src/reading.rs are limited to what the stale-reading check needs, such as exposing the touched paths of a reading; if none is needed the file stays unchanged.
- Governance files (athena.toml, defaults.toml, workflows) do not change; athena.toml keeps max_in_flight = 1.

## Acceptance criteria

1. [behavior] Run cargo test: a new drain test with max_in_flight = 2 and read_ahead = 1, driven through drain_concurrent, shows the next item's read (and plan, if the job is hard) is requested while the current job is still running, and never more than read_ahead items ahead.
2. [behavior] Run cargo test: a test shows that when the running job changes paths the read-ahead reading touches, that reading is marked stale and the touched paths are re-checked when the job starts; a reading whose paths are untouched is used as is.
3. [behavior] Run cargo test: with max_in_flight = 1 no read-ahead happens, and the existing drain tests in crates/athena-cli/src/drain.rs pass unchanged (git diff shows no edits to existing test bodies).
4. [behavior] Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test; all pass.
5. [content] CHANGELOG.md has a new entry describing read-ahead and the stale-reading re-check, saying that max_in_flight = 1 is unchanged.
6. [content] The change touches only crates/athena-cli/src/drain.rs, crates/athena-core/src/reading.rs (if needed) and CHANGELOG.md; git diff shows no edits to athena.toml, defaults.toml or .github/workflows.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing athena.toml or raising max_in_flight to 3 (build job 10)
- Changes to crates/athena-data/src/config.rs from job 20261029-142849-04d9 (kept as landed)
- Fixing the stale line 21 in docs/design/async-drain.md and the DrainPipeline error wording from job 20261029-142849-04d9
- Disk-guard, check-slot and conflict-return behavior (other build jobs)
- Changes to defaults.toml or .github/workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy -D warnings and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain through its entry point and observe read-ahead happening.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is read-ahead observed with max_in_flight above 1 and no change at 1.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to the three named files.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog and doc comments written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in `mod tests` of crates/athena-cli/src/drain.rs call the production entry point `drain_concurrent(&fake, limits, &Quiet, 2)`. Guards: a bound of 2 gives workers above 1, and `read_ahead()` above 0 starts the pump. The fake `ReadAheadWorld` is `Sync` and implements `Steps` (the seam, which exists today; step 1 adds the five default methods it overrides). It keeps a `Mutex<Vec<String>>` log and counts reads made and runs started. `caps` is read 0.5, run 1.0. The budget is large. `queue()` returns a→b→c with `depends_on` a→b→c and complete plan entries built like `planned()` in tests/drain.rs (goal, rung and acceptance set). Exact touches: a `[src/a.rs, CHANGELOG.md]`, b `[src/b.rs, CHANGELOG.md]`, c `[src/c.rs, CHANGELOG.md]`, d `[src/d.rs, CHANGELOG.md]`. Each entry has disjoint, non-empty paths and shares only CHANGELOG.md, which the rule ignores. The fake overrides `read_ahead()` to 1. It logs every `plan_ahead` call (`plan_ahead <id>`) and returns Some(Planned{cost:0.2}) only for b. Its `read` costs 0.1. `run(a)` polls up to 5 s until `read b` and `plan b` are logged, holds 200 ms, then logs `end a`. The fake only records calls and supplies data. The pump, window rule, claim and ordering are production code. A second test uses a→b→c→d with `read_ahead` 2. A third has `landing` of every job changing only its own `src/x.rs` plus CHANGELOG.md, which proves the shared-file rule: read-ahead is not skipped. Run: `cargo test --locked -p athena-cli --lib read_ahead`. Observes: Test `read_ahead_reads_and_plans_the_next_hard_job_while_the_current_one_runs`: the log has `start a` < `read b` < `plan_ahead b` < `plan b` < `end a`. `read c` comes after `start b`. `plan_ahead c` is logged (the drain asked), no `plan c` is logged, and `Report.spent_usd` equals the sum of the fake's read, run and b-plan costs only, so the drain charged a plan for b and none for c. The max of (reads made minus runs started) is at most 1. Test `read_ahead_never_runs_further_ahead_than_the_window` (read_ahead 2): while a runs, b and c are read and d is not, and the same difference never exceeds 2. Test `read_ahead_is_not_skipped_for_jobs_that_share_only_changelog`: `read b` precedes `end a` although a and b both touch CHANGELOG.md. Test `read_ahead_skips_an_item_that_shares_paths_with_a_job_in_flight`: a touches `[src/shared]`, b touches `[src/shared/x.rs]`, and a second item e has empty touches. `read b` and `read e` are logged only after `end a`, and the other cases still run in order. Test `read_ahead_paths_rule_matches_the_plan_grouping`: a table (`src/x` vs `src/x/y.rs` clash, `src/x` vs `src/xy.rs` do not, only-CHANGELOG touches, empty touches) gives the same answer from `paths_disjoint` as `Plan::build` gives for 'same group'. Test `read_ahead_ends_with_the_drain_when_a_stop_is_requested` (chain a→b→c→d, read_ahead 2, fake `stop_requested` true once `end a` is logged): `drain_concurrent` returns, `Report.stop` is `Stop::Requested`, elapsed is under 2 s, and no `read` is logged after the stop.
  Fails without the behavior: With no pump, `read b` is logged only after `end a`, so the order assertion fails and `run(a)` times out at 5 s. With no window bound, `read c` precedes `start b` and the difference exceeds 1. With a naive prefix rule that does not ignore CHANGELOG.md, the shared-file test sees `read b` only after `end a`. With a copy that drifts from plan.rs, the parity test fails. Without the drop guard, the stop test hangs until the pump notices the stop and the elapsed assertion fails.
- Criterion 2: Unit tests through `drain_concurrent(&fake, limits, &Quiet, 2)` with `read_ahead` 1 and a→b where b depends on a. Seam: `Steps::plan_entry` (exists today; used for the touches), `Steps::recheck` and `Steps::reread` (step 1 adds them), `Landing::Landed { changed }` (exists). The fixtures state touches exactly. a's planned touches are `[src/a.rs, CHANGELOG.md]`, disjoint from b's `[src/b.rs, CHANGELOG.md]`, so the pump does read b ahead. Stale test: the fake's `land(a)` returns `Landed { changed: ["src/b.rs", "CHANGELOG.md"] }`, which is the case where the plan underestimated what a changes. Its `recheck` logs `recheck b [src/b.rs]` and returns false, and its `reread` logs `reread b` and returns a runnable `Read`. Untouched test: `land(a)` returns `["src/a.rs", "CHANGELOG.md"]`. Shared-only test: `land(a)` returns `["CHANGELOG.md"]` (both the landing and the reading touch it). A fourth case has `recheck` return true. The fake only supplies data and logs. The overlap test, the stale note, the `recheck` call, the fresh-read decision and which `Read` runs are production code in drain.rs. Run: `cargo test --locked -p athena-cli --lib stale_reading`. Observes: Test `stale_reading_is_rechecked_when_its_job_starts`: the log has `read b` (ahead) < `land a` < `recheck b [src/b.rs]` < `reread b` < `start b`. `recheck` appears exactly once, `Report.notes` has one line naming b and src/b.rs as stale, and b runs on the reread job. Test `stale_reading_kept_when_recheck_says_it_holds`: `recheck` appears once, there is no `reread`, and b runs on the ahead reading. Test `stale_reading_not_marked_when_paths_are_untouched`: no `recheck`, no `reread`, no stale note, `read b` exactly once, and b runs on that reading. Test `stale_reading_not_marked_when_only_shared_files_landed`: the same observation when only CHANGELOG.md landed. Test `stale_reading_runs_the_job_the_reading_named`: `read b` returns `job: Some("b-2")`, and `run` is called with `b-2` and the job line names `b-2` (finding 4).
  Fails without the behavior: Without landing-path tracking and the overlap check there is no `recheck` call and no stale note, so the stale test fails. If every landing forced a re-check, or CHANGELOG.md counted, the untouched and shared-only tests see a `recheck` and fail. If the drain ran the item's own id and not `read.job`, the `b-2` test fails.
- Criterion 3: New test `read_ahead_is_off_with_a_bound_of_one` runs the same `ReadAheadWorld` fake with `read_ahead()` at 1 through `drain_concurrent(&fake, limits, &Quiet, 1)` and through `drain(&fake, limits, &Quiet)` (both entry points exist today). It also runs it with `read_ahead()` 0 at a bound of 2. Seam: the `Steps` default methods (step 1). Existing tests (the `mod tests` bodies in drain.rs and all of crates/athena-cli/tests/drain.rs) run unchanged. The verifier then checks that the diff only adds test functions. Run: `cargo test --locked -p athena-cli`. Observes: In the bound-1 and `read_ahead` 0 cases `read b` is logged only after `end a`, the fake counts zero `plan_ahead` calls, and no stale note appears. Every existing drain test passes. `git diff -- crates/athena-cli` shows only added lines in drain.rs (new items and new test functions) and no change to crates/athena-cli/tests/drain.rs. Each existing test body is untouched.
  Fails without the behavior: If the pump started at bound 1 or with `read_ahead` 0, `read b` would precede `end a` and the new test fails. If the default methods or the moved batch loop changed bound-1 behavior, the unedited existing tests in drain.rs and tests/drain.rs would fail.
- Criterion 4: Run the three Athena checks in order on the finished change. The producer runs `cargo fmt --all` first. The `&&` chain stops at the first failure, so exit 0 means all three passed. It covers the pump, `Steps` defaults, `Planned`, the State fields, the path helpers and every test, old and new. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0, and the test run lists the read_ahead and stale_reading tests and the unedited existing drain tests as passing.
  Fails without the behavior: Unformatted new code makes `cargo fmt --check` exit non-zero. A clippy warning from the new code (an unused `Planned` field or `plan_cap` default, a lock held across a call) fails under -D warnings. If read-ahead were missing or broke bound-1 behavior, the criterion 1, 2 or 3 tests would fail and `cargo test` would exit non-zero.
- Criterion 5: Read the CHANGELOG.md diff for a new bullet under `## [Unreleased]` / `### Added`, in the existing style. Run: `git diff -- CHANGELOG.md`. Observes: The bullet describes read-ahead and the stale-reading re-check of section 4 of the async-drain design. It says reading and planning of hard jobs run up to `read_ahead` items ahead only when more than one job runs at once. It says items whose paths clash with a job in flight are not read ahead, a reading whose paths a landing changed is re-checked at job start, `max_in_flight = 1` is unchanged, and the real steps do not turn it on yet. It adds no email-like text and names no machine or operator.
- Criterion 6: Inspect the list of changed paths. Run: `git status`. Observes: Only crates/athena-cli/src/drain.rs and CHANGELOG.md are modified. reading.rs and plan.rs are untouched. Nothing under athena.toml, defaults.toml, .github/workflows or crates/athena-cli/tests appears.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9484ed113622 → candidate 1c7a5ce2f7e4. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                   |   14 +
 crates/athena-cli/src/drain.rs | 1053 +++++++++++++++++++++++++++++++++++++++-
 2 files changed, 1052 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1689 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test; all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
