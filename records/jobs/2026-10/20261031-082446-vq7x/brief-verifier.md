# Job 20261031-082446-vq7x: verify "Add a steward triage pass that sets priorities at drain start"

## Request (verbatim)

> Priority: high
> Depends on 20261031-082442-92qg
> A steward triage pass at the start of each drain that sets priorities from the current focus
> 
> After 20261031-082442-92qg, Athena has a recorded focus and caps on urgent and high. Add a triage duty for the steward role that runs at the start of every drain, after the Hestia sweep. One model call reads the current focus, the open items (title, rung, current priority and who set it, age, dependencies) and the failure classes seen in the last two days, and returns a priority for each item it would change, with a one-line reason tied to the focus or to a repeated failure.
> 
> Athena applies them as priority records attributed to the steward, within the caps. It never changes an item whose latest priority record was set by a person, and never closes anything; it may propose closing an item as a steward work item. The drain prints one line per change.
> 
> Acceptance:
> - A test with a fake triage answer shows priority records written with the steward as author and the reason, the caps respected, and a person-set item left unchanged.
> - A test shows the drain prints the changes and goes on when the triage call fails.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

Builds on job 20261031-082442-92qg (focus record and urgent/high caps; dispatched, not verified): after the Hestia sweep, each drain makes one model call with the focus, open items and recent failure classes. It applies the returned priorities as steward-authored priority records within the caps, and prints one line per change. A failed call prints a note and the drain goes on.

Assumptions:
- Code Survey placeholder replaced below.
- Code survey: read crates/athena-cli/src/drain.rs (drain entry point; queue assembled from steward::approved_work, queued_work and interrupted_work around line 3325), crates/athena-core/src/steward.rs (queued_work, repeated_failures at ~712, run at ~1168) and a grep of crates/athena-core/src/hestia.rs, crates/athena-core/src/plan.rs, crates/athena-data/src/records.rs (priority records, AnyRecord validation) and crates/athena-agent/src/fake.rs (fake model answers used by crates/athena-cli/tests/drain.rs and steward.rs). The new triage runs in drain.rs right after the Hestia sweep and before work selection. Priority records already exist, so no new record kind is added; the existing records round-trip/check_record_file coverage applies, and the test must confirm the steward-authored priority record passes it. Behavior is measured by reading the written priority records and the drain output in the new test, not through a stand-in.
- Job 20261031-082442-92qg is a dependency: its focus record, `athena focus`, and the [priority_caps] config are assumed present. Its 12 changed files are not part of this change; they are kept as that job's work and excluded here (see out_of_scope). Its result is not re-verified here.
- Goal: the person running Athena gets a meaningful queue ordering without hand re-tiering; the audience is the drain operator reading its output and the verifier reading tests.
- Triage input uses open items (title, rung, current priority and its setter, age, dependencies), the current focus, and failure classes from the last two days via steward::repeated_failures or equivalent. The model answer is JSON of {item, priority, reason}; invalid entries are ignored. A proposal to close an item is recorded as a steward work item, never a close.
- Caps are respected by counting items already urgent/high (including person-set) and skipping steward changes that would exceed them.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test triage`: with a fake triage answer, priority records are written whose author is the steward and which carry the one-line reason; the urgent and high caps from athena.toml are not exceeded; and an item whose latest priority record was set by a person is unchanged.
2. [behavior] Run `cargo test -p athena-cli --test triage`: when the fake triage call fails, the drain prints a triage-failed note, still prints the changes from other steps, and goes on to work the queue; the drain output has one line per applied change on success.
3. [behavior] Run `cargo test -p athena-cli --test triage`: a fake answer proposing to close an item results in a steward work item and no close record for the item.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.
5. [content] CHANGELOG.md has an entry describing the steward triage pass; Grep for 'triage' in CHANGELOG.md finds it.
6. [content] Grep of crates/athena-cli/src/drain.rs shows the triage call placed after the Hestia sweep and before queue selection, and the triage code in crates/athena-core/src/triage.rs never writes a close record.
7. [content] No governance file (athena.toml, defaults.toml, workflows) is changed; git diff shows only the listed deliverables.

## Expected deliverables

- crates/athena-core/src/triage.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/triage.rs (create)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)

## Out of scope

- Work of job 20261031-082442-92qg is kept as is and not changed here: CHANGELOG.md focus entry, athena.toml, crates/athena-cli/src/display.rs, crates/athena-cli/src/main.rs, crates/athena-cli/tests/focus.rs, crates/athena-core/src/plan.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, crates/athena-core/src/store.rs, crates/athena-data/src/config.rs (those files other than where listed as deliverables above)
- Closing any item from triage
- Changing the caps, the focus command or the plan demotion logic
- A new triage prompt file under the prompt sets (token headroom is nearly exhausted)
- A standalone `athena triage` command; a follow-up request
- Rewriting priority records set by a person
- Editing anything under records/ by hand

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy -D warnings criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run the triage tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check written records and drain output.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; a standalone command and prompt files are left out.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The no-close and person-set rules are enforced in code and tested.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog entry are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test `triage_writes_steward_priority_records_within_caps_and_leaves_person_set_items` in `crates/athena-cli/tests/triage.rs`. It runs the real drain over real `Records`. Setup: a `Fixture`, `athena.set_focus("speed first")`, and 6 queued complete requests (path plus bullet, so nothing is read). One request gets `athena.set_priority(id, "low")`, a person's record. The `Steps` wrapper delegates `triage`, `queue`, `plan_entry`, `job_history`, `priority_caps` and `person_set` to a real `RealSteps` and stubs `run`, `land`, `rate` and `check_page`, as the `Replay` wrapper in `tests/drain.rs` does. The fake answer proposes urgent for 4 normal requests (the default cap is 3) and also urgent for the person-set one. Run: `cargo test -p athena-cli --test triage`. Observes: After `drain::drain`, `Records::load` shows: three steward priority records at urgent, each with `set_by == Some("steward")` and the reason from the answer, each passing `check_record_file`. The fourth urgent proposal, over the cap, has no new record. The person-set request still has exactly its original record, so its latest priority is unchanged. The count of urgent items after the drain is at most 3.
  Fails without the behavior: Without the triage pass no steward priority record is written. Without the person-set guard the person's low item turns urgent. Without the cap check a fourth urgent record appears.
  Test seam: Production entry point: `athena_cli::drain::drain`, which calls `Steps::triage`, which for the real steps calls `Athena::triage`. The fake is a `Script` closure injected through `Fixture::athena(script)` that returns the scripted answer for a steward-role invocation whose prompt has the triage heading, and `deterministic_turn` otherwise. The script helper is added to `tests/common/mod.rs` in the steps. The `Steps::triage` hook and `Athena::triage` do not exist yet and are added by the plan steps. `Fixture`, `RealSteps` (public fields), `Lander` stubs and `Lines` exist today (read).
- Criterion 2: Two tests in `crates/athena-cli/tests/triage.rs`. (a) `a_failed_triage_call_prints_a_note_and_the_drain_goes_on`: the script returns a failed `FakeTurn` (non-success outcome) for the triage prompt. The drain runs with two queued requests, and the `Steps` wrapper records which jobs were run. (b) `the_drain_prints_one_line_per_applied_change`: with a good answer that changes 2 items, the lines collected by the drain's `Lines` `Ui` are counted. Run: `cargo test -p athena-cli --test triage`. Observes: (a) The output has a line starting `triage failed`. Both queued jobs are still run by the wrapper's `run`. No priority record is written and the drain report stops with `QueueEmpty`. (b) Exactly one `triage`-prefixed change line per applied change, each naming the job, the old and new priority, and the reason. The lines appear before any line from running a job.
  Fails without the behavior: Without the catch in `RealSteps::triage`, the error stops the drain or no note is printed, so (a) fails. Without the line printing in `drain_with`, (b) finds no change lines.
  Test seam: Entry point: `athena_cli::drain::drain` with the same wrapper, `Lines` `Ui` and script injection as criterion 1. The failure is injected as a `FakeTurn` with a non-success `InvocationOutcome`, so `invoke_and_record` and `consult` run for real and the error path of `Athena::triage` is the one under test. This extends the criterion 1 seam, and the steps add it.
- Criterion 3: Test `a_close_proposal_becomes_a_steward_work_item_and_never_a_close` in `crates/athena-cli/tests/triage.rs`. The answer has `close: [{item: <job>, reason}]` and also a priority change for another item. A pure unit test in `triage.rs` checks the same through `plan_changes`. Run: `cargo test -p athena-cli --test triage`. Observes: `Records::load` shows a new `WorkItem` with role `steward`, kind Proposal, key `triage-close:<job>`, and the job as evidence. The job has no outcome or close record, and its state is still queued, so it is still in the drain's queue. A second drain on the same records adds no second work item (key already known). A grep-style assertion in the test reads the `triage.rs` source via `include_str!` and finds no use of `Outcome` or a close constructor.
  Fails without the behavior: A version that closes the job leaves it with an outcome or out of the queue. A version that ignores `close` writes no work item.
  Test seam: Entry point: `athena_cli::drain::drain` as in criterion 1. The injected fake is the scripted triage answer. `WorkItem` writing is a new function in `triage.rs`, added by a plan step. `Records` loading and `WorkItem::validate` exist.
- Criterion 4: Run the three checks Athena itself runs, on the finished tree. This includes all existing tests, since the `PriorityRecord` fields touch struct literals in `reading.rs` and `tests/drain.rs`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The existing priority, plan, drain and records round-trip tests still pass, and a person-set record with no author field still parses.
  Fails without the behavior: A literal missing the new fields fails to compile. A malformed `Steps` default or an unused-import warning fails clippy `-D warnings`.
  Test seam: Not a seam test: whole-workspace gates. Nothing is injected.
- Criterion 5: Read `CHANGELOG.md` under `[Unreleased]` for a bullet describing the steward triage pass at drain start. It names the focus input, the caps, person-set items left alone, close proposals as work items, and that a failed call only prints a note. Run: `grep -n -i triage CHANGELOG.md`. Observes: At least one hit inside the Unreleased section (before the first release heading), describing the feature in plain words.
  Test seam: Content check. No seam.
- Criterion 6: Grep `crates/athena-cli/src/drain.rs` for `review_passes()` and `triage()` in `drain_with`, and confirm line order: the triage loop is after the review-pass loop and before `steps.queue()`. A unit test in `drain.rs` (a `Steps` stub that logs calls) asserts the call order `review_passes`, `triage`, `queue`. Grep `crates/athena-core/src/triage.rs` for close-writing code. Run: `grep -n -E "review_passes\(\)|steps\.triage\(\)|steps\.queue\(\)" crates/athena-cli/src/drain.rs && grep -n -i -E "AnyRecord::(Outcome|Close)|OutcomeState|fn close" crates/athena-core/src/triage.rs`. Observes: The first command lists `review_passes()`, then `triage()`, then the first `queue()` in `drain_with`, in line order. The second finds no match, so the module writes no close or outcome record.
  Fails without the behavior: If triage ran after `queue()` or before the review passes, the call-order unit test and the line-order grep fail.
  Test seam: Content check, plus the call-order unit test in `drain.rs`. That test uses a logging `Steps` stub like the existing `Housekeeping` stub (exists, read) with the `triage` hook added in the steps.
- Criterion 7: Check that the diff touches only the listed deliverables, plus the minimal literal fixes in `reading.rs` and `tests/drain.rs`, and no governance file. Run: `git diff --stat`. Observes: The changed paths are among: `triage.rs`, `lib.rs`, `drain.rs`, `tests/triage.rs`, `CHANGELOG.md`, `fake.rs`, `steward.rs`, `records.rs`, `tests/common/mod.rs`, and the literal fixes. `athena.toml`, `defaults.toml`, workflows and the lint and tool configuration do not appear.
  Test seam: Content check. No seam.

Planned files:

- crates/athena-core/src/triage.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/triage.rs (create)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base 5cee7dbb3f21 → candidate 27f5997a1969. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  14 +
 crates/athena-agent/src/fake.rs       |  26 +-
 crates/athena-cli/src/drain.rs        |  39 +-
 crates/athena-cli/tests/common/mod.rs |  25 ++
 crates/athena-cli/tests/drain.rs      |   4 +
 crates/athena-cli/tests/triage.rs     | 495 +++++++++++++++++++++
 crates/athena-core/src/lib.rs         |   5 +
 crates/athena-core/src/reading.rs     |   4 +
 crates/athena-core/src/steward.rs     |  98 ++++-
 crates/athena-core/src/triage.rs      | 806 ++++++++++++++++++++++++++++++++++
 crates/athena-data/src/records.rs     |  84 ++++
 11 files changed, 1588 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41581175070 for 27f5997a1969383cae347c91fbd6c2eccc6b471a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41581175070 for 27f5997a1969383cae347c91fbd6c2eccc6b471a)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41581175070 for 27f5997a1969383cae347c91fbd6c2eccc6b471a)
- ci-result: `not a check` passed (Check results for 27f5997a1969383cae347c91fbd6c2eccc6b471a come from the CI run https://github.com/doodla/athena/actions/runs/41581175070 (603s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
