# Job 20261026-144234-bz4e: verify "Split process-lifecycle code out of drain.rs to get under limits"

## Request (verbatim)

> Structural pass: crates/athena-cli/src/drain.rs
> 
> Code health found crates/athena-cli/src/drain.rs over a limit: 1798 production lines with 5 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week.
> 
> Measured: 1798 production lines (test modules excluded); longest function run_job at 131 lines; 5 landings in the last 7 days; 16 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: 1798 production lines with 5 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-cli/src/drain.rs

## Reading (accepted by the user)

Restructure crates/athena-cli/src/drain.rs, which is at 1798 production lines with 5 landings in a week against a limit of 1500. Move the pid-file, PidGuard, stop/restart and RealLifecycle section (around lines 3412 to the end) into a new child module, drain/lifecycle.rs. Re-export every public item from drain so callers in main.rs and display.rs compile unchanged. The audience is the code-health gate and the maintainers. There is no behavior change.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (item outline), crates/athena-cli/src/main.rs and crates/athena-cli/src/display.rs (callers through the drain:: path). Callers use drain::guarded, RealProcs, StopWait, PidRecord, DrainState, StopOutcome, RestartOutcome and reinstall_note. The drain.rs outline shows about 17 lines of imports plus doc comments at the top (up to line 260), then types, the Drain engine, drain_with, and from about line 3412 a self-contained process-lifecycle section: pid_file, log_file, PidRecord, Procs/RealProcs, DrainState, PidGuard, guarded, StopWait, stop_and_wait, restart, RealLifecycle. No records or data files are added, so the record validation allow-list is not involved.
- Code survey: the move target is crates/athena-cli/src/drain/lifecycle.rs, a child module of drain.rs. It is declared with `mod lifecycle; pub use lifecycle::*;` (or explicit re-exports), so existing paths such as crate::drain::DrainState keep working. The producer keeps the tests with the code they test, or leaves them in drain.rs if they need private items, and may widen visibility to pub(super) where required.
- The lifecycle section is chosen because it has the least coupling to the Drain engine. If the section turns out to be under 300 production lines, the producer also moves another cohesive group, for example the study_*/model_contradictions helpers or the PrLander code. The target is below 1500 production lines in drain.rs, with no function over 200 lines.
- No refusal rule is added or tightened, so no newly-refused-reading percentage applies.

## Acceptance criteria

1. [content] crates/athena-cli/src/drain.rs has fewer than 1500 production lines (test modules excluded), and a new crates/athena-cli/src/drain/lifecycle.rs holds the moved code. Check with `ls` and by reading both files.
2. [content] The diff only moves code, re-exports it and adjusts visibility or imports. No function bodies change in logic, and public paths such as crate::drain::guarded, DrainState, StopOutcome and RestartOutcome are still exported from drain. Check with `git diff`.
3. [behavior] `cargo test -p athena-cli` passes, with the same tests present as before the change and none deleted or ignored. This is the outcome check: the drain behavior is unchanged.
4. [behavior] `cargo clippy --all-targets -- -D warnings` is clean and `cargo fmt --check` passes.
5. [behavior] `cargo build` succeeds for the whole workspace, so main.rs and display.rs callers compile without edits.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/drain/lifecycle.rs (create)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any behavior change or refactor of the drain engine logic
- Changes to athena.toml code-health limits or other governance files
- Splitting other files such as display.rs or main.rs

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module split, with no behavior edits.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Existing tests must pass, and clippy and fmt must be clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the file is under the limit and behavior is unchanged, not that code was moved.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The file is split by responsibility rather than trimmed to just under the limit.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 74277e4fc4df → candidate 611c2254b627. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   3 +
 crates/athena-cli/src/drain.rs           | 581 +-----------------------------
 crates/athena-cli/src/drain/lifecycle.rs | 583 +++++++++++++++++++++++++++++++
 3 files changed, 589 insertions(+), 578 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-144234-bz4e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144234-bz4e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42454246094 for 611c2254b62757950e23ebf251316f345d1c4fc1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42454246094 for 611c2254b62757950e23ebf251316f345d1c4fc1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42454246094 for 611c2254b62757950e23ebf251316f345d1c4fc1)
- ci-result: `not a check` passed (Check results for 611c2254b62757950e23ebf251316f345d1c4fc1 come from the CI run https://github.com/doodla/athena/actions/runs/42454246094 (757s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo clippy --all-targets -- -D warnings` is clean and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
