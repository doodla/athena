# Job 20261030-165704-qfqs: verify "Make drain follow athena plan's order and log passed-over items"

## Request (verbatim)

> Priority: urgent
> Re-ask of 20261030-033434-3v30, which stopped twice in planning at the old $8 job cap (no code written; the cap is now $12).
> athena drain does not work items in the order athena plan prints. Evidence from 2026-10-30:
> - After 20261029-142858-5c0p was set to low priority with athena priority, drain-23 dispatched it next, ahead of urgent items.
> - athena plan lists 20261029-233021-e5sg first (group 1, no unmet dependency), yet drain-24 dispatched 20261029-142902-5x4h, then 20261029-180911-7q3e, then 20261029-210827-18fc, and never read or mentioned e5sg; its log has no line naming it.
> Find the cause in the drain's ordering (crates/athena-cli/src/drain.rs, plan building in crates/athena-core/src/plan.rs) and fix it so drain works items in the same order athena plan prints, and when drain passes over a planned item it logs one line naming the item and the reason.
> Acceptance:
> - A test through the drain entry point with a priority record that lowers an urgent-text request shows the drain works it after a higher-priority item.
> - A test shows the drain's order equals athena plan's order for the same queue, and that a passed-over item gets a log line with its reason.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/plan.rs, CHANGELOG.md

## Reading (accepted by the user)

Re-ask of 3v30: find why drain's dispatch order differs from athena plan (priority records and e5sg's omission), fix it so drain works the planned order, and log one line per passed-over planned item with the reason. Audience: the person running athena drain, who compares its dispatch with athena plan.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs (entry point drain() at ~1966, plan() at ~2175, Steps::queue trait at ~450, RealSteps::entry_of ~2434 and RealSteps::queue ~2514) and the test fakes in crates/athena-cli/tests/drain.rs (fake queue() impls). Plan building lives in crates/athena-core/src/plan.rs. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. The order is measured directly: the sequence of jobs the drain starts, compared with the entries athena plan returns for the same queue.
- Job 20261030-033434-3v30 (previous attempt) stopped in planning with no candidate and no files changed. Its round-2 critique findings are addressed here: (a) do not assume a timeline; weigh three causes, a plan that goes stale mid-run, a drain built before commits 9f3946d and d72e755 (dependency rules), and a run that ended before reaching a waiting item; (b) log every planned item left unworked, with its reason; (c) re-plan only when records, priority records or job ends change; (d) test a priority record present before drain() starts as well as one added mid-run. Its allowed files (drain.rs, plan.rs, CHANGELOG.md) are kept as deliverables; reading.rs is added only as a low-probability deliverable.
- Job 20261029-142858-5c0p (A4, now low priority): not retried here. Its candidate changed nothing, so no work is kept or dropped. It is only the evidence that a lowered job was dispatched ahead of urgent ones.
- Job 20261029-233021-e5sg: not retried here. Its rejected candidate changes to crates/athena-core/src/status.rs, crates/athena-core/src/store.rs, crates/athena-git/src/lib.rs and its CHANGELOG entry are not touched or reapplied; its findings are left to its own follow-up. It is only evidence of an item that plan listed first but drain never logged.
- Job 20261029-142902-5x4h, 20261029-180911-7q3e and 20261029-210827-18fc are cited only as the observed dispatch order; they are not changed.
- Assumption: priority semantics (urgent, high, normal, low) stay as they are. The fix makes drain use the same plan-building function and the same ordering as athena plan, and re-plans after each item only when the queue inputs changed.
- CHANGELOG.md is the only file that gets an entry; athena.toml, defaults.toml and workflows need no change.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-cli: a test calling drain() with a priority record that lowers an urgent-text request shows that item is started after a higher-priority item. It covers both a record present before drain() starts and one added while an earlier item runs.
2. [behavior] Run cargo test: a test builds one queue and shows the sequence of jobs drain() starts equals the entries athena plan's plan() returns for the same queue, including a group-1 item that has a cited-but-not-'Depends on' job id.
3. [behavior] Run cargo test -p athena-cli: a test shows each planned item that is not worked (held, refused, cyclic, not ready, clash with a running batch, over budget, job limit, run ended first) gets exactly one UI log line with its job id and a reason, once per run.
4. [behavior] Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test: all pass, and the existing drain tests pass unchanged.
5. [content] Grep of crates/athena-cli/src/drain.rs and crates/athena-core/src/plan.rs shows drain takes its order from the same plan-building code as plan(), with no separate ordering in drain.
6. [content] CHANGELOG.md has a new entry that names the cause found, the order fix and the passed-over log line.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Retrying or fixing jobs 20261029-142858-5c0p, 20261029-233021-e5sg (status.rs, store.rs, athena-git changes) or 20261029-142902-5x4h; their candidates are not touched
- Changes to athena.toml, defaults.toml, .github/workflows or records/
- Wiring read-ahead or raising max_in_flight
- Changing the priority rules themselves (urgent/high/normal/low semantics) beyond making drain and plan agree

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new drain tests plus cargo test and clippy -D warnings.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check dispatch order and log lines by running tests, not by wiring existing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is weighed among three explanations and the fix is applied to ordering in general.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands the verifier runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to drain ordering and logging.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that drain order equals plan order, not that code was edited.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two tests in crates/athena-cli/tests/drain.rs on the real path, calling `drain::drain(&steps, limits(..), &ui)` with `RealSteps`. The fixture is `Fixture`, `fx.athena(script)` and `athena.queue(text)`, which exist today. U1 and U2 have 'Priority: urgent' in their text and N is normal, each with an acceptance bullet and no read needed. Guard: all three items are complete, so `prepare` reads nothing. (a) `commit_records_to(&fx, &[AnyRecord::Priority{job: U2, priority: "low"}], msg)` (exists, tests/drain.rs ~3575) before drain; expect landings U1, N, U2. (b) The same record is committed while U1 runs. Preferred seam: a one-shot hook on `RecordingLander` (exists ~1336; step 7 adds the hook field) that commits the record when U1 lands. If committing from the lander is not possible, step 7 adds a small `Steps` wrapper that delegates every method to `real_steps(..)` (exists ~2448) and commits the record inside `run` of U1. The order is read from `RecordingLander`. Run: `cargo test -p athena-cli --test drain a_priority_record_that_lowers_an_urgent_request`. Observes: The order of landed jobs is U1, N, U2 in both cases: the lowered urgent-text request comes after the higher-priority item.
  Fails without the behavior: Mid-run test: drain freezes its plan at the start, so the lowered request U2 still runs second, ahead of the normal item N, and the assertion `landed == [U1, N, U2]` fails. The before-start test already passes today (the plan reads the record at the start), so it guards that behavior and does not prove the fix.
- Criterion 2: Two tests in tests/drain.rs on a real Fixture with no approvable proposals, so drain's approval step adds nothing. Entry points: `drain::plan(&steps, 5.0, &Quiet)` and `drain::drain(&steps, limits(50.0), &ui)`, both exist today, on one Athena. Static guard test: an older rejected request R, the newest request E that cites R's id on a line not starting 'Depends on' (group 1, first in the plan), a request with a 'Depends on' line and a different priority, and one request with no acceptance bullets. It asserts the landing sequence from `RecordingLander` equals `report.plan.order`, that E is first, and that the refused item has one 'passed over' line on the UI (`common::Lines`, tests/common/mod.rs ~304). Mid-run test: while the first item runs, the same hook or wrapper as criterion 1 queues a new urgent request with `athena.queue(text)` and calls `drain::plan` at once. The queued request has an acceptance bullet, so plan() reads nothing and costs nothing. The drain's dispatches after the first item must equal the captured `order` without the running item. Run: `cargo test -p athena-cli --test drain drain_order`. Observes: The started sequence equals the plan order for the static queue, and equals the plan printed mid-run for the changed queue.
  Fails without the behavior: `drain_order_follows_the_plan_as_the_queue_changes` fails on base code: the request queued mid-run is never dispatched by the frozen plan, so the dispatches after the first item differ from `plan()`'s order. `drain_order_equals_plan_order` fails if drain used another order, held the group-1 item that only cites a job id, or left the refused item without a passed-over line.
- Criterion 3: A scenario table in tests/drain.rs, one `drain::drain` call per scenario on `Stub` (exists, tests/drain.rs 40-99) with `common::Lines` as the UI. Step 7 adds two Stub fields: `history: Vec<JobEnd>`, returned by a new `job_history` override, and `needs_accepting: Vec<&'static str>`, read by the Stub's `run`. The fake only supplies the queue, history and run results; ordering and logging are production code. Scenarios and what each expects: refused (no acceptance), held (rejected dependency in `history`), cyclic (two items that depend on each other), job limit (`max_jobs` 1 with two items unreached), budget short, stop requested (`stop_when_running`), and not ready (`needs_accepting`). Each non-worked planned id has exactly one UI line starting 'plan       ', containing the id, 'passed over' and a reason text, and the count is still 1 after the run. Clash: `drain_concurrent` needs `Steps + Sync`, which `Stub` is not (it uses RefCell), so that scenario uses the Sync stub already in the file (impl at ~4145), or a real Fixture. A wave of two items with `max_jobs` 1 and a later-group item that shares a path with the wave expects one line naming the id, the job limit and the shared path. Added test `a_replan_does_not_read_an_underspecified_item_twice`: a Stub with one underspecified item (incomplete `plan` entry, completed in `after_read`) and at least three items, so there are several waves. It asserts the stub log holds one 'read U' and the UI holds one 'plan       U is underspecified' line. The stub's `read` log already exists. Run: `cargo test -p athena-cli --test drain passed_over_item_is_logged_once`. Observes: One line per non-worked planned id, each with its reason, and no duplicates. One read and one 'reading it' line across all waves.
  Fails without the behavior: On base code refused, held and cyclic items reach only `Report.notes`, and unreached items are not mentioned, so the count of UI lines naming the id is 0 and the test fails. `a_replan_does_not_read_an_underspecified_item_twice` fails if the plan memory is missing: the item would be read once per wave and the 'is underspecified' line would repeat.
- Criterion 4: Run the three checks Athena runs, as three separate whole commands over the workspace: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked` (the command shown). No seam: this is the real workspace. The existing tests in tests/drain.rs and in drain.rs are left unmodified; they are the regression guard, and the existing approval (`job-of-…`), failed-item, retry, read-ahead, budget, limit and concurrent tests are the ones the merge and the per-wave pump reset must keep green. The git diff must show no edit to existing test bodies. Run: `cargo test --locked`. Observes: All three exit 0, including every pre-existing drain test.
  Fails without the behavior: `cargo test --locked` fails if the re-plan loop starts a tried item again, loses an approved `job-of-…` item, re-reads an item, changes dispatch order, adds an unexpected log line, or breaks the read-ahead pump, because an existing drain test then fails. `cargo fmt --all --check` fails on unformatted new code. `cargo clippy --all-targets --locked -- -D warnings` fails on any new warning, such as an unused `shared_path` or `tried`.
- Criterion 5: Two checks. First, the Grep tool on crates/athena-cli/src/drain.rs and crates/athena-core/src/plan.rs for `Plan::build`, `sort_by`, `sort_unstable`, `position` and `fn prepare`, then a read of `batches_for` and the drain_with loop. Second, the unit test added in drain.rs (in-file tests, `batches_for` in the cfg(test) module) builds a `Plan` with `Plan::build_with` from entries of mixed priority and rung and asserts `batches_for(.., &BTreeSet::new())` flattened over planned items equals `plan.order`. It also asserts that adding an id to `tried` removes only that item. Run: `cargo test -p athena-cli batches_for`. Observes: The only code that builds the drain's order is `Plan::build_with` inside `Drain::prepare`, which `plan()` also calls. `batches_for` only filters `plan.groups` and the `place` sort remains only as the age input to `build_with`. The unit test passes.
  Fails without the behavior: A second ordering in drain.rs would show as an extra `Plan::build`, `sort_by` or `sort_unstable` match outside the `place` age sort and `prepare`. A reorder inside `batches_for` would fail the unit test, because the flattened batches would no longer equal `plan.order`.
- Criterion 6: Read the added Unreleased entry in the diff. Run: `git diff CHANGELOG.md`. Observes: A new entry names the cause: drain's plan was built once and ignored later priority records, job ends and queued requests, skips were notes and not log lines, drain approves proposals that `athena plan` does not, and a binary older than d72e755 and 9f3946d may have held a request that only cited a job id (found by reading). It then states the fix: drain re-plans through the same plan builder after each wave. It also names the passed-over log line, 'plan       <id> passed over: <reason>'.
  Fails without the behavior: No Unreleased entry, or one missing the cause, the order fix or the log line, shows in the diff.

Planned files:

- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 507f073d6fb0 → candidate 3710b96b64de. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  23 +++
 crates/athena-cli/src/drain.rs   | 403 +++++++++++++++++++++++++++++++++++----
 crates/athena-cli/tests/drain.rs | 397 +++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/plan.rs   |  25 ++-
 4 files changed, 810 insertions(+), 38 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1761 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
