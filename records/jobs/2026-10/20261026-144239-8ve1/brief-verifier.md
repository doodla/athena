# Job 20261026-144239-8ve1: verify "Split CLI run function into per-command handlers in main.rs"

## Request (verbatim)

> Structural pass: crates/athena-cli/src/main.rs
> 
> Code health found crates/athena-cli/src/main.rs over a limit: function run is 549 lines, over the limit of 200.
> 
> Measured: 1290 production lines (test modules excluded); longest function run at 549 lines; 9 landings in the last 7 days; 24 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: function run is 549 lines, over the limit of 200
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-cli/src/main.rs

## Reading (accepted by the user)

Refactor crates/athena-cli/src/main.rs so no function exceeds 200 lines (run is 549) and the file stays under its production-line limit, by moving each large Command arm of run into its own handler function (or a new sibling module). Behavior is unchanged. The audience is code health and the maintainers.

Assumptions:
- Code survey: read crates/athena-cli/src/main.rs; run (line 1016 to about 1697) is one match over Command with inline arms (Ask, Retry, Propose, ScoreJev, Review, AcceptProposal, RejectProposal, ApproveWork, and more), each calling open(..), an Athena method, then print_json or display::*. Helpers sit above it (lines 572-1014). Sibling modules exist: drain.rs, display.rs, wiring.rs, hooks.rs, decisions.rs in crates/athena-cli/src/.
- Approach: extract each long arm into a private fn taking the destructured fields and returning Result<u8>; keep run as a thin dispatcher. Group arms into a sibling module only if that is needed to get the file under the 1500-line landing-frequency limit. Do not change CLI output, exit codes, or clap definitions.
- No records or data files are added, so the records allow-list is unaffected. No refusal rule is added or changed.
- Governance files (athena.toml, defaults.toml, clippy config) are not touched.
- The 1290 production lines are already under 1500 and 2000; the breach is the per-function limit, so run must drop to 200 lines or fewer, and so must every new function.

## Acceptance criteria

1. [content] In crates/athena-cli/src/main.rs (and any new module), no function exceeds 200 lines; run is a dispatcher of roughly 200 lines or fewer. Check by reading the file.
2. [content] The diff is a pure restructuring: the Command enum, clap definitions, message strings and exit codes are unchanged, and the existing tests in main.rs and crates/athena-cli/tests are not modified or removed.
3. [behavior] Run `cargo test -p athena-cli`. All existing tests pass, including tests/binary.rs and tests/proposals.rs, which exercise CLI output.
4. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`. Both are clean.
5. [behavior] Run `cargo run -p athena-cli -- --help`. The output matches the pre-change output, so the subcommand list is unchanged.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/commands.rs (create)
- crates/athena-cli/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any behavior, CLI surface, or output change
- Changes to other files over limits
- Changes to athena.toml, defaults.toml or code-health limits

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused refactor of a single file
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The long function is split by command, not trimmed by cosmetic edits
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the test suite and --help rather than checking that functions exist
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Refactor stays in Rust
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3760067a6e69 → candidate bfaba9e8275a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   3 +
 crates/athena-cli/src/commands.rs | 656 +++++++++++++++++++++++++++++++++++++
 crates/athena-cli/src/main.rs     | 670 ++++----------------------------------
 3 files changed, 722 insertions(+), 607 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-144239-8ve1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144239-8ve1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42466071351 for bfaba9e8275a81c54aeab7979ac5a9ff55cd6561)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42466071351 for bfaba9e8275a81c54aeab7979ac5a9ff55cd6561)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42466071351 for bfaba9e8275a81c54aeab7979ac5a9ff55cd6561)
- ci-result: `not a check` passed (Check results for bfaba9e8275a81c54aeab7979ac5a9ff55cd6561 come from the CI run https://github.com/doodla/athena/actions/runs/42466071351 (761s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`. Both are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
