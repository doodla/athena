# Job 20261108-063602-h7je: verify "Split drain.rs into child modules to get under 1500 production lines"

## Request (verbatim)

> try again on 20261026-144234-bz4e

## Reading (accepted by the user)

Retry of job 20261026-144234-bz4e: the earlier request's 1798-line figure was stale; drain.rs now has about 4585 production lines (first test module at line 4586). Split it into several child modules (lifecycle, real_steps, lander, engine) so drain.rs ends under 1500 production lines, with no behavior change. Audience: the code-health gate and maintainers.

Assumptions:
- Job 20261026-144234-bz4e is addressed: its blocker (drain.rs still ~4013 production lines) is fixed by moving four groups, not one; its major finding (stale figure, no re-measure) is fixed by requiring the producer to measure production lines of drain.rs before and after and report both numbers; its minor CHANGELOG finding is fixed by wording the entry only after the measured result is true. Its candidate files are all kept as deliverables: CHANGELOG.md, drain.rs, drain/lifecycle.rs (the lifecycle move itself was sound and is redone). main.rs was in its scope but is not expected to change, since drain re-exports keep paths stable.
- Code survey: read crates/athena-cli/src/drain.rs top-level items. Production code runs to line 4585; test modules start at 4586 (batches_tests, tests, in_flight_tests, read_ahead_tests, landing_queue_tests). Groups by line: lifecycle (PidRecord..RealLifecycle, 3457-4027, ~570 lines); RealSteps and its Steps impl (4028-4585, ~560); Lander/PrLander/PlanReport and study helpers (3199-3456, ~260); the Drain engine (State, Carry, LandingQueue, Drain, ItemEnd, RunStep and impls, 1181-3198, ~2000); data types and the Steps trait (288-1180, ~900 incl. InFlight and Report). Callers in main.rs and display.rs use crate::drain paths, so glob or explicit re-exports must keep those paths. Tests use super::*, so private items moved out need pub(super) visibility and re-import in drain.rs.
- Target math: 4585 minus lifecycle, real_steps, lander and engine (about 3390 lines) leaves roughly 1200 production lines in drain.rs, under 1500. If the engine move proves not cleanly separable, the producer moves the next largest cohesive group (for example InFlight/Report types) instead, and reports the measured number.
- Measuring production lines: test modules (#[cfg(test)] mod blocks) are excluded, as code_health::measure_source does. The producer measures with that tool or by counting lines before the first #[cfg(test)] module, in each new file as well, and every new file must stay under 1500 lines too, since the limit is per file with 5 landings a week.
- Child modules live under crates/athena-cli/src/drain/ and are declared in drain.rs with mod and pub use re-exports. Moves only; no function body logic changes.
- Governance: athena.toml code-health limits are not changed; no governance file is a deliverable.

## Acceptance criteria

1. [content] Production lines of crates/athena-cli/src/drain.rs (everything before the first #[cfg(test)] module) are fewer than 1500. Check by Grep for '^#\[cfg\(test\)\]' with line numbers in drain.rs and reading the first match; the line before it is the production line count.
2. [content] New files under crates/athena-cli/src/drain/ (at least lifecycle.rs, real_steps.rs, lander.rs, engine.rs) each have fewer than 1500 production lines, checked by Grep for line numbers of the first #[cfg(test)] or the file length via Read.
3. [content] The diff (git diff) only moves code, adds mod and re-export lines, and adjusts imports or visibility such as pub(super). No function body logic changes, and crate::drain public paths used by main.rs and display.rs (guarded, DrainState, StopOutcome, RestartOutcome and others) remain exported from drain.
4. [content] No test is deleted, ignored or edited beyond import paths: the #[cfg(test)] modules stay in drain.rs (or move with their code) with the same test function names. Check with git diff and Grep for '#\[test\]' counts before and after.
5. [content] CHANGELOG.md has an entry under Changed that describes the split and states only what is true after the measurement (drain.rs under 1500 production lines).
6. [behavior] Run `cargo test -p athena-cli`: it passes with the same number of tests as before the change (outcome check: drain behavior is unchanged).
7. [behavior] Run `cargo clippy --all-targets -- -D warnings` (clean) and `cargo fmt --check` (passes).
8. [behavior] Run `cargo build`: the whole workspace compiles, so main.rs and display.rs callers need no edits.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/drain/lifecycle.rs (create)
- crates/athena-cli/src/drain/real_steps.rs (create)
- crates/athena-cli/src/drain/lander.rs (create)
- crates/athena-cli/src/drain/engine.rs (create)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Any behavior change or logic refactor of the drain engine
- Changes to athena.toml code-health limits or other governance files
- Splitting other files such as display.rs or main.rs
- Adding the proposed reading-time size-measurement check to Athena itself

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust module moves.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the measured line count and that tests still pass, not only that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused split with no logic change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Re-measure the file instead of trusting the stale figure.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Modules are named by responsibility.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 39d3492379f7 → candidate fa15fc4e073f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |    6 +
 crates/athena-cli/src/drain.rs            | 3417 +----------------------------
 crates/athena-cli/src/drain/engine.rs     |  620 ++++++
 crates/athena-cli/src/drain/item_work.rs  | 1042 +++++++++
 crates/athena-cli/src/drain/lander.rs     |  200 ++
 crates/athena-cli/src/drain/lifecycle.rs  |  582 +++++
 crates/athena-cli/src/drain/real_steps.rs |  574 +++++
 crates/athena-cli/src/drain/rounds.rs     |  422 ++++
 8 files changed, 3458 insertions(+), 3405 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-063602-h7je/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-063602-h7je/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42456710726 for fa15fc4e073fbe970d9d4e59a270b4cd511319c4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42456710726 for fa15fc4e073fbe970d9d4e59a270b4cd511319c4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42456710726 for fa15fc4e073fbe970d9d4e59a270b4cd511319c4)
- ci-result: `not a check` passed (Check results for fa15fc4e073fbe970d9d4e59a270b4cd511319c4 come from the CI run https://github.com/doodla/athena/actions/runs/42456710726 (627s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
