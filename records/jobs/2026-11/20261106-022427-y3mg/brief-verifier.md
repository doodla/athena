# Job 20261106-022427-y3mg: verify "Record plan group sizes and unplanned items in the drain report"

## Request (verbatim)

> Record how many items each plan group held in a drain run, for both the one-at-a-time path and the parallel path (build step 6 of docs/design/agent-and-build-slots.md, measurement part).
> 
> A drain run's record lists one entry per plan group with that group's item count, so later runs show how much parallel work was available. An item that has no plan entry is recorded as unplanned, not as a group of 1. athena drain's summary prints the group sizes.
> 
> Acceptance:
> - a test with a plan of groups sized 3, 1 and 2 run with max_concurrency 1 records exactly those three sizes
> - the same plan run with max_concurrency 2 records the same three sizes
> - an item with no plan entry is counted separately as unplanned
> - a test runs the drain summary and shows the group-sizes line
> - records still validate; CHANGELOG states only what the diff does
> 
> Files changed: the drain run record, the drain loop where groups are formed, the summary display, tests, CHANGELOG.md.

## Reading (accepted by the user)

Add per-plan-group item counts and a separate unplanned count to the drain run's Report, filled for both the one-at-a-time and the parallel path, and print a group-sizes line in athena drain's summary. Goal: later runs (the maintainer) can see how much parallel work was available; this is the measurement half of build step 6.

Assumptions:
- Code Survey placeholder replaced below.
- Code survey: read crates/athena-cli/src/drain.rs (Report at ~1101 already holds plan_groups: Vec<Vec<String>>; the shared loop at ~2940-3070 builds the plan, takes first_plan, runs batches_for(...) groups, with one_at_a_time splitting a group to one item; drain_concurrent at ~2803 passes max_concurrency) and crates/athena-cli/src/display.rs (~701 prints 'plan        group N: ...' lines). The group sizes are measured directly from the first plan's groups (plan.groups lengths), not from how many ran at once, so they are identical for max_concurrency 1 and 2. Report is an in-memory struct with Serialize; I found no AnyRecord/check_record_file type for it under crates/athena-data, so no new record file is expected; if the producer adds a persisted record it must be covered by check_record_file/AnyRecord and the round-trip test.
- Existing code: extends Report and the plan-group printing in crates/athena-cli/src/drain.rs and crates/athena-cli/src/display.rs, and reads Plan.groups from crates/athena-core/src/plan.rs; tests go in crates/athena-cli/tests/drain.rs with helpers in crates/athena-cli/tests/common/mod.rs.
- 'Drain run record' is read as the drain Report. Unplanned means a queued item that has no entry in the plan view (view.plan), counted as its own number, never as a group of 1.
- Group sizes are taken from the plan the run started with (first_plan), as plan_groups already is. Items that are refused or cyclic are not counted as unplanned unless they lack a plan entry.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`: a new test drains a plan of three groups sized 3, 1 and 2 with max_concurrency 1 and asserts the report's group sizes are exactly [3, 1, 2].
2. [behavior] A new test drains the same plan with max_concurrency 2 (drain_concurrent) and asserts the same sizes [3, 1, 2]; run with `cargo test -p athena-cli`.
3. [behavior] A new test has a queued item with no plan entry and asserts the report counts it in a separate unplanned field (1) and that no group of size 1 is added for it.
4. [behavior] A new test renders the drain summary (display.rs) and asserts it contains a group-sizes line such as 'group sizes 3, 1, 2' plus the unplanned count; run with `cargo test -p athena-cli`.
5. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass, including the existing record validation tests, so records still validate.
6. [content] CHANGELOG.md gains an entry that states only what the diff does (group sizes and unplanned count in the drain report and summary), with no claim about raising max_concurrency or editing athena.toml.
7. [content] The group sizes are computed in the drain loop from the plan's groups (crates/athena-cli/src/drain.rs) and the field has a doc comment explaining unplanned vs group of 1.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- docs/design/agent-and-build-slots.md (modify)

## Out of scope

- Raising max_concurrency or editing athena.toml / [drain.pipeline] (the config half of step 6)
- Peak in-flight job counts and mean group size computed from many runs
- Changing how the plan forms groups or how the loop dispatches them

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain with real plans of sizes 3,1,2 and read the recorded sizes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to Report, the loop, display, tests, CHANGELOG.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criterion is that later runs can read group sizes from the report and summary.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5bc0b608cbd4 → candidate a83863733990. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  3 ++
 crates/athena-cli/src/display.rs     | 14 +++++++++
 crates/athena-cli/src/drain.rs       | 15 +++++++++
 crates/athena-cli/tests/drain.rs     | 60 ++++++++++++++++++++++++++++++++++++
 docs/design/agent-and-build-slots.md |  4 +++
 5 files changed, 96 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-022427-y3mg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-022427-y3mg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42176226739 for a83863733990449770abbc63e140864663982822)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42176226739 for a83863733990449770abbc63e140864663982822)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42176226739 for a83863733990449770abbc63e140864663982822)
- ci-result: `not a check` passed (Check results for a83863733990449770abbc63e140864663982822 come from the CI run https://github.com/doodla/athena/actions/runs/42176226739 (494s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass, including the existing record validation tests, so records still validate.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
