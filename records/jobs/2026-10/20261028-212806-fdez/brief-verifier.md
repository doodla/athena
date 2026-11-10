# Job 20261028-212806-fdez: verify "Split deterministic_turn in fake.rs into per-role helper functions"

## Request (verbatim)

> Structural pass: crates/athena-agent/src/fake.rs
> 
> Code health found crates/athena-agent/src/fake.rs over a limit: function deterministic_turn is 204 lines, over the limit of 200.
> 
> Measured: 399 production lines (test modules excluded); longest function deterministic_turn at 204 lines; 3 landings in the last 7 days; 10 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: function deterministic_turn is 204 lines, over the limit of 200
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-agent/src/fake.rs

## Reading (accepted by the user)

Break the 204-line deterministic_turn in crates/athena-agent/src/fake.rs into small per-role functions (reader, producer, verifier, planner) that the match dispatches to. Behavior stays identical. The audience is the code-health gate and reviewers.

Assumptions:
- Code survey: read crates/athena-agent/src/fake.rs; deterministic_turn (lines 198-429) is a match on spec.role, with Reader (~100 lines) and Producer (~58 lines) arms being the largest. Helpers like section and first_enum already exist. No records or data files are added, and no entry point or probing order changes.
- Existing code: crates/athena-agent/src/fake.rs only; helpers stay private in the same file.
- Goal: get under the 200-line function limit via a behavior-preserving extraction; the file is only 399 production lines so no module split is needed.
- Consumers: none found (no config deliverable).

## Acceptance criteria

1. [content] In crates/athena-agent/src/fake.rs, deterministic_turn is a short match that delegates the Reader, Producer, Verifier and Planner arms to separate functions; no function in the file exceeds 200 lines (check by reading the file).
2. [content] The diff to fake.rs moves code only: JSON field names, literals, prompt section headings and constants (FAKE_DELIVERABLE, FAKE_CHANGELOG, SPEED_WORDS) are unchanged, and the public signature of deterministic_turn is unchanged (git diff).
3. [behavior] Run `cargo test -p athena-agent`: all existing tests in fake.rs and the crate pass without any test edits.
4. [behavior] Run `cargo clippy --workspace --all-targets` (no warnings) and `cargo fmt --check` (clean) and `cargo test --workspace` (passes).
5. [content] No file other than crates/athena-agent/src/fake.rs changes (git diff --stat); no governance files are touched.

## Expected deliverables

- crates/athena-agent/src/fake.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing fake agent behavior or output JSON
- Changing limits in athena.toml or code-health configuration
- Editing tests or other files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criterion requires cargo test, clippy and fmt to pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single-file focused refactor.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criterion runs existing tests to show unchanged behavior.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5d4f45d66661 → candidate b58241ad533c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                    |   2 +
 crates/athena-agent/src/fake.rs | 429 +++++++++++++++++++++-------------------
 2 files changed, 223 insertions(+), 208 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261028-212806-fdez/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-212806-fdez/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42758722649 for b58241ad533cab8de838848cae02a2f5a40c7cb6)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42758722649 for b58241ad533cab8de838848cae02a2f5a40c7cb6)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42758722649 for b58241ad533cab8de838848cae02a2f5a40c7cb6)
- ci-result: `not a check` passed (Check results for b58241ad533cab8de838848cae02a2f5a40c7cb6 come from the CI run https://github.com/doodla/athena/actions/runs/42758722649 (751s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --workspace --all-targets` (no warnings) and `cargo fmt --check` (clean) and `cargo test --workspace` (passes).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
