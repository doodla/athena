# Job 20261026-144242-sjp3: verify "Break up the 205-line status function in status.rs"

## Request (verbatim)

> Structural pass: crates/athena-core/src/status.rs
> 
> Code health found crates/athena-core/src/status.rs over a limit: function status is 205 lines, over the limit of 200.
> 
> Measured: 916 production lines (test modules excluded); longest function status at 205 lines; 9 landings in the last 7 days; 14 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: function status is 205 lines, over the limit of 200
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/status.rs

## Reading (accepted by the user)

Refactor `Athena::status` (crates/athena-core/src/status.rs, line 2079, ~205 lines) into private helper methods or functions so every function is at most 200 lines. Behavior and output stay identical. The audience is code health, which measures the limit, and the maintainers who read the file.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs. `status()` (lines 2079-2346) loads Records and the RecordSet, then builds job lines (jobs_by_state, recent, open, first_round), intervention counts, per-class forecast headlines, the pending quantities, the bootstrap note and the `next` action lines. It ends by assembling StatusView. The `trace` method follows it and is untouched.
- Natural seams in `status()`: (1) the job loop that produces jobs_by_state, recent, open and first_round; (2) the per-class forecast headlines; (3) the pending-quantity resolution join; (4) the bootstrap note; (5) the `next` line construction. Extract these into private free functions or methods that take `&Records` or `&[JobLine]`. Keep the order of `next` pushes exactly as it is.
- The file is 916 production lines, well under the 2000 line limit. Only the 200 line per-function limit is breached. Whole-file splitting into modules is not needed. The big test module stays where it is.
- No record or data files are added, so the records allow-list and round-trip tests are not affected. No rule is added or tightened, so no refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows, lint config) are not needed. Do not add lint allows to silence the limit.

## Acceptance criteria

1. [content] In crates/athena-core/src/status.rs, no function, including the `status` method, is longer than 200 lines. A reader can check this by reading the file. `status` is now clearly shorter than 205 lines and delegates to named helpers.
2. [content] The diff touches only crates/athena-core/src/status.rs. The StatusView fields, public signatures and `next` line texts and order are unchanged, and no existing test is edited or deleted.
3. [behavior] `cargo test -p athena-core` passes with the existing status tests unchanged.
4. [behavior] `cargo clippy --all-targets` reports no warnings, and `cargo fmt --check` passes.
5. [content] The extracted helpers have doc comments written for an outside reader, and no lint suppression is added to hide the length.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Splitting status.rs into multiple modules
- Changing the `trace` method or any other function
- Changing the status output, next-step text or StatusView shape
- Changing code-health limits or athena.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file, one focused extraction.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Break up the long function along real responsibilities instead of trimming lines to land just under 200.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Existing tests are run to show behavior is unchanged.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: New helpers get doc comments.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2375f96178ed → candidate 9d3048bcfce2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   6 ++
 crates/athena-core/src/status.rs | 201 ++++++++++++++++++++++-----------------
 2 files changed, 120 insertions(+), 87 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-144242-sjp3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144242-sjp3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42468822942 for 9d3048bcfce2bcdfa8bf2ac85522bd05cca2e2c6)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42468822942 for 9d3048bcfce2bcdfa8bf2ac85522bd05cca2e2c6)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42468822942 for 9d3048bcfce2bcdfa8bf2ac85522bd05cca2e2c6)
- ci-result: `not a check` passed (Check results for 9d3048bcfce2bcdfa8bf2ac85522bd05cca2e2c6 come from the CI run https://github.com/doodla/athena/actions/runs/42468822942 (552s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo clippy --all-targets` reports no warnings, and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
