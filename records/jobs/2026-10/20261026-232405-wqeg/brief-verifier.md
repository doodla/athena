# Job 20261026-232405-wqeg: verify "Move Records-branch writes row into gh/records.rs (G4)"

## Request (verbatim)

> Priority: high
> Structure pass job G4 (docs/design/structure-pass.md, section 4). Move the Records-branch writes row. No behavior change. Depends on 20261026-232356-ds51 and 20261026-232402-6qme.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-core/src/gh/records.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass job G4, following G1 (20261026-232356-ds51) and G3 (20261026-232402-6qme): move the Records-branch writes row (LandingRecords, LandingFacts, verification_id, history_method, GitLandingRecords and impls, CutoverRun, cutover_job_for, run_cutover, cutover_catch_up, the ReconcileRecords impl) from gh/mod.rs into new gh/records.rs, verbatim, with re-exports keeping gh:: paths. Audience: the maintainer and verifier checking the section 5 proof.

Assumptions:
- Prior jobs G1 (20261026-232356-ds51) and G3 (20261026-232402-6qme) are landed and kept as is: their changed files (CHANGELOG.md, gh/mod.rs, introspect.rs, external_calls_guard.rs, docs/decisions.md, gh/remote.rs, gh/pages.rs, deleted gh.rs) are not reverted; only gh/mod.rs and CHANGELOG.md are touched again here, the others are out of scope.
- G1/G3 findings addressed: stale gh.rs paths in docs (system-facts.md, architecture.md) are left to a later docs job (G8) per section 5 rule 7; the broken intra-doc link in remote.rs is not touched here; D138 wording is not touched. For this job the caller check per moved name is requested explicitly in the report, and the verifier can rerun it by Grep.
- Code survey: read docs/design/structure-pass.md (row at line 45, module table line 87, job table lines 167 and 268); the moved code lives in crates/athena-core/src/gh/mod.rs and will go to crates/athena-core/src/gh/records.rs; callers are crates/athena-core/src/reconcile.rs and cutover tests in crates/athena-cli/tests. No record or data file is added, so check_record_file and AnyRecord are not involved.
- Tests use super::* and rely on re-exports; the new module imports what it needs (host for CommitHost, ReconcileRecords) as the design table says.
- Moved text is verbatim; only a module doc line, import lines, mod/pub use lines change. The line range 1427-1850 in the design doc may have shifted after G1 and G3, so the producer locates the row by name.

## Acceptance criteria

1. [content] crates/athena-core/src/gh/records.rs exists and contains LandingRecords, LandingFacts, verification_id, history_method, GitLandingRecords and its impls, CutoverRun, cutover_job_for, run_cutover, cutover_catch_up and the ReconcileRecords impl; none of these definitions remain in gh/mod.rs (Grep fn/struct/trait definitions in both files).
2. [content] gh/mod.rs declares the records module and re-exports every public moved name with pub use, so Grep of `gh::(LandingRecords|GitLandingRecords|LandingFacts|cutover)` across crates/ is unchanged and no caller outside gh/ uses gh::records:: paths.
3. [content] git diff shows the moved text is identical to the removed text (apart from added module doc, imports, mod and pub use lines), and no test function is added, removed, renamed, or edited beyond import lines.
4. [content] CHANGELOG.md has a G4 entry under Unreleased stating the Records-branch writes row moved and behavior is unchanged.
5. [content] The final report lists the caller check (a Grep result) for each moved name; the verifier can rerun those Greps over crates/.
6. [behavior] Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass, and the number of tests run equals the count after G3 (1397).

## Expected deliverables

- crates/athena-core/src/gh/records.rs (create)
- crates/athena-core/src/gh/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/reconcile.rs (modify)

## Out of scope

- Moving other gh rows (other G jobs)
- Any run.rs change (R jobs)
- Updating docs that cite gh.rs (system-facts.md, architecture.md)
- Editing introspect.rs, external_calls_guard.rs, docs/decisions.md, athena.toml, defaults.toml or workflows
- Fixing the remote.rs intra-doc link
- Any behavior change or test edits beyond import lines

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust module move.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One row moved.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier reruns Greps and cargo checks.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks target unchanged callers and test count.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Module doc line and changelog entry written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d95a7381d1d3 → candidate 221eadaa75de. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   7 +
 crates/athena-core/src/gh/mod.rs     | 441 +----------------------------------
 crates/athena-core/src/gh/records.rs | 434 ++++++++++++++++++++++++++++++++++
 3 files changed, 453 insertions(+), 429 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass, and the number of tests run equals the count after G3 (1397).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
