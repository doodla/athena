# Job 20261029-061956-namb: verify "Split planning_round in planning.rs under the 200-line limit"

## Request (verbatim)

> Structural pass: crates/athena-core/src/run/planning.rs
> 
> Code health found crates/athena-core/src/run/planning.rs over a limit: function planning_round is 291 lines, over the limit of 200.
> 
> Measured: 515 production lines (test modules excluded); longest function planning_round at 291 lines; 0 landings in the last 7 days; 0 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: function planning_round is 291 lines, over the limit of 200
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/run/planning.rs

## Reading (accepted by the user)

Break the 291-line planning_round (crates/athena-core/src/run/planning.rs, line 80) into smaller private helper methods so every function is at most 200 lines. Behavior stays identical. The audience is code health and the reviewer.

Assumptions:
- Code survey: read crates/athena-core/src/run/planning.rs; planning_round spans lines 80-387, followed by end_round (388), findings_line, redacted, plan_record, planning_grant, and a tests module at line 596.
- Goal: reduce the function length below 200 lines by extracting helpers within the same file; no module split is needed since the file is 515 production lines, well under 2000.
- Helpers stay private; no public API, record format or prompt changes.
- Consumers: none found (no config changes).

## Acceptance criteria

1. [content] In crates/athena-core/src/run/planning.rs, planning_round is at most 200 lines and no other function exceeds 200 lines (check by reading the file).
2. [behavior] Run `cargo test -p athena-core`: all existing tests, including those in planning.rs, pass unchanged.
3. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean.
4. [content] The diff touches only crates/athena-core/src/run/planning.rs, contains no change to existing test code, and the extracted code moves without changed logic.

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Behavior changes to planning
- Changes to other run/ files
- Changes to athena.toml or defaults.toml

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check function length and passing tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single-file refactor.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c95914a4f610 → candidate b64167db6a1b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   5 ++
 crates/athena-core/src/run/planning.rs | 105 ++++++++++++++++++++++++---------
 2 files changed, 81 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-061956-namb/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-061956-namb/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42799318063 for b64167db6a1ba3dcc4613551ba3710e7c5ccf69b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42799318063 for b64167db6a1ba3dcc4613551ba3710e7c5ccf69b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42799318063 for b64167db6a1ba3dcc4613551ba3710e7c5ccf69b)
- ci-result: `not a check` passed (Check results for b64167db6a1ba3dcc4613551ba3710e7c5ccf69b come from the CI run https://github.com/doodla/athena/actions/runs/42799318063 (785s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
