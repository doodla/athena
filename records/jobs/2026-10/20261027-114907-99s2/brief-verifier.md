# Job 20261027-114907-99s2: verify "Split briefs.rs: move boundary-scan code into a submodule"

## Request (verbatim)

> Structural pass: crates/athena-context/src/briefs.rs
> 
> Code health found crates/athena-context/src/briefs.rs over a limit: 1524 production lines with 7 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week.
> 
> Measured: 1524 production lines (test modules excluded); longest function reader_brief at 158 lines; 7 landings in the last 7 days; 13 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: 1524 production lines with 7 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-context/src/briefs.rs

## Reading (accepted by the user)

Restructure crates/athena-context/src/briefs.rs, the audience being code health, so production lines fall under 1500. Move the boundary-word scan and plain-text rewrite group (boundary_words through plain_boundary_text, about 270 lines) into a new briefs/boundary.rs submodule, re-exported so public paths stay unchanged. No behavior change.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (item outline) and crates/athena-context/src/lib.rs; briefs is declared `pub mod briefs;` in lib.rs, and the file has a `#[cfg(test)] mod tests` at line 2545 that reaches items through `super::*`. No record or data file is added, so check_record_file and AnyRecord are not involved.
- Existing code: boundary_words, boundary_fragments, banned_words_section, BoundaryHit, boundary_scan, plain_alternatives, replace_boundary_words and plain_boundary_text form a cohesive group (lines ~828-1100) that is cut out as a unit; callers elsewhere in the workspace keep using athena_context::briefs::<name> through `pub use`.
- Moved items that tests or sibling code use privately become pub(super) or pub(crate); public API paths stay identical.
- Measured by code health as production lines excluding test modules; the target is under 1500, so move at least about 60 lines, with a margin.
- No refusal rule is added or tightened, so no refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs.rs has fewer than 1500 production lines (test module excluded), checkable by counting lines before `#[cfg(test)]` in the file; a new file crates/athena-context/src/briefs/boundary.rs holds the moved boundary-scan code and briefs.rs declares `mod boundary;` with a `pub use` of its public items.
2. [content] The diff only moves code: no function body in the moved items changes, and the public names (boundary_words, boundary_fragments, boundary_scan, plain_boundary_text, BoundaryHit, and the instruction constants) remain reachable at athena_context::briefs::.
3. [behavior] Run `cargo test --workspace`: all existing tests pass unchanged, including the briefs tests and dependents that import athena_context::briefs items.
4. [behavior] Run `cargo clippy --workspace --all-targets` with no warnings, and `cargo fmt --check` passes.
5. [content] No test is deleted or weakened; the diff to the briefs test module is limited to import adjustments, if any.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/briefs/boundary.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Any behavior change to brief text or boundary scanning
- Splitting other files or changing athena.toml limits
- Changes to lib.rs unless a module declaration proves necessary

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move of a single cohesive group.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt required.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Splits by responsibility rather than trimming lines to scrape under the limit.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 51b3b5d67c83 → candidate 9aefe2682f80. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |   8 +
 crates/athena-context/src/briefs.rs          | 269 +--------------------------
 crates/athena-context/src/briefs/boundary.rs | 266 ++++++++++++++++++++++++++
 3 files changed, 280 insertions(+), 263 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-114907-99s2/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-114907-99s2/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42540152090 for 9aefe2682f8002d8a728e484824a5d93998234ee)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42540152090 for 9aefe2682f8002d8a728e484824a5d93998234ee)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42540152090 for 9aefe2682f8002d8a728e484824a5d93998234ee)
- ci-result: `not a check` passed (Check results for 9aefe2682f8002d8a728e484824a5d93998234ee come from the CI run https://github.com/doodla/athena/actions/runs/42540152090 (639s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --workspace`: all existing tests pass unchanged, including the briefs tests and dependents that import athena_context::briefs items.; Run `cargo clippy --workspace --all-targets` with no warnings, and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
