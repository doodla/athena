# Job 20261109-004033-y1jj: verify "Split briefs.rs into submodules to get under code-health limits"

## Request (verbatim)

> try again on 20261027-114907-99s2

## Reading (accepted by the user)

Retry of job 20261027-114907-99s2: measure briefs.rs in this snapshot (about 2544 production lines, reader_brief about 219 lines, not the quoted 1524), then split it into several submodules (boundary, reader, planner/critic, plan rendering) and break up reader_brief so briefs.rs is under 1500 lines and no function exceeds 200. Audience: code health (`athena status`). No behavior change.

Assumptions:
- Retry of 20261027-114907-99s2, which was rejected because briefs.rs stayed at 2287 production lines with a 217-line function. Findings addressed: (1) blocker, file still over limit: this retry moves about 1100+ lines across several submodules, not one 257-line slice. (2) major, plan sized from the quoted 1524 rather than measurement: the producer must measure first (production lines before the first #[cfg(test)], longest function, e.g. via `cargo run -q -p athena-cli -- status`) and size the split to the measurement. (3) major, reading mismatch: the earlier boundary-only reading is replaced. (4) nit, tab indentation in moved BOUNDARY_SCAN_INSTRUCTION: moves must be byte-for-byte, no tabs introduced. The candidate's work (boundary.rs, briefs.rs edits, CHANGELOG.md) is kept as the starting design (boundary move) but redone from base and extended.
- Code survey: read crates/athena-context/src/briefs.rs (about 2544 production lines; #[cfg(test)] mod tests at line 2546). Groups by Grep of top-level items: boundary scan and plain rewrite lines 815-1100 (boundary_words, boundary_fragments, banned_words_section, BOUNDARY_SCAN_INSTRUCTION, MIRROR_CHECK_INSTRUCTION, BoundaryHit, boundary_scan, plain_boundary_text); reader types and brief lines 2007-2540 (EarlierCase, RetryCause, EarlierAttempt, prior_result_section, carried_work, ReaderInputs, READING_RULES, reader_brief at 2315-2533, about 219 lines); planner and critic lines 1820-2006; plan and findings rendering lines 1589-1750; verifier lines 1105-1590. crates/athena-context/src/lib.rs and outputs.rs are the only other files in the crate. Property measured directly with athena_core::code_health through `athena status`.
- Measured at this snapshot by Grep, an estimate pending the producer's own measurement: about 2544 production lines; reader_brief about 219 lines. A target of under 1500 needs about 1050 or more lines moved; moving the reader group (about 530), boundary group (about 285) and planner/critic (about 190) gives about 1000, plus plan rendering (about 160) reaches about 1370.
- Public paths stay unchanged: each new submodule is declared in briefs.rs and its public items are re-exported with `pub use`; private helpers used by briefs.rs or its tests become pub(super). The existing tests module stays in briefs.rs, with import adjustments only. Tests or xtask policy scans that read briefs.rs as text (for example the xtask policy test) must be checked and fixed if they break; a text-reading test for the moved code may be repointed to the new file, not weakened.
- reader_brief is broken into private helper functions in the reader submodule (for example per-section helpers), keeping output byte-identical.
- CHANGELOG.md gets one entry because the repo convention records changes there. No athena.toml or limits change.

## Acceptance criteria

1. [behavior] Run `cargo run -q -p athena-cli -- status` on the candidate: crates/athena-context/src/briefs.rs shows under 1500 production lines and is not listed over any limit; no function in briefs.rs or the new submodules exceeds 200 lines (reader_brief included).
2. [content] New files under crates/athena-context/src/briefs/ (at least boundary.rs and a reader submodule) exist; briefs.rs declares each with `mod` and `pub use`s the public items, so `athena_context::briefs::` paths including boundary_words, boundary_fragments, boundary_scan, plain_boundary_text, BoundaryHit, reader_brief, ReaderInputs, planner_brief and critic_brief still resolve for other crates.
3. [content] The diff only moves code, apart from visibility changes and splitting reader_brief into helpers: Grep for a tab character in the new files finds none (string continuation lines keep the base's indentation), and no string literal or brief text changes.
4. [behavior] Run `cargo test --workspace`: all existing tests pass, including the briefs tests and any test that reads source files as text; none is deleted or weakened in the diff (test module changes limited to imports).
5. [behavior] Run `cargo clippy --workspace --all-targets` with no warnings, and `cargo fmt --check` passes.
6. [content] CHANGELOG.md has an entry describing the briefs.rs split.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/briefs/boundary.rs (create)
- crates/athena-context/src/briefs/reader.rs (create)
- crates/athena-context/src/briefs/planner.rs (create)
- crates/athena-context/src/briefs/plan.rs (create)
- CHANGELOG.md (modify)
- xtask/tests/policy.rs (modify)
- crates/athena-context/src/lib.rs (modify)

## Out of scope

- Any behavior change to brief text or boundary scanning
- Splitting other files or changing athena.toml limits
- Changes to lib.rs unless a module declaration proves necessary
- Adding the proposed Athena code check for size-reduction jobs (a separate change)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code stays Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused structural split with no behavior change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Sized to the measured file, not the quoted figure.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion checks the code-health measurement itself.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: New modules get clear doc comments.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 0af8b7f0979c → candidate 87f060c0d0eb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |    9 +
 crates/athena-context/src/briefs.rs          | 1183 +-------------------------
 crates/athena-context/src/briefs/boundary.rs |  280 ++++++
 crates/athena-context/src/briefs/plan.rs     |  188 ++++
 crates/athena-context/src/briefs/planner.rs  |  176 ++++
 crates/athena-context/src/briefs/reader.rs   |  569 +++++++++++++
 6 files changed, 1247 insertions(+), 1158 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-004033-y1jj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-004033-y1jj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42544880094 for 87f060c0d0ebb383f501117225b54e1cb1dccfcd)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42544880094 for 87f060c0d0ebb383f501117225b54e1cb1dccfcd)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42544880094 for 87f060c0d0ebb383f501117225b54e1cb1dccfcd)
- ci-result: `not a check` passed (Check results for 87f060c0d0ebb383f501117225b54e1cb1dccfcd come from the CI run https://github.com/doodla/athena/actions/runs/42544880094 (795s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
