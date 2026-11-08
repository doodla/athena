# Job 20261026-144236-qvvv: verify "Split reading.rs into submodules to get under 1500 lines"

## Request (verbatim)

> Structural pass: crates/athena-core/src/reading.rs
> 
> Code health found crates/athena-core/src/reading.rs over a limit: 1737 production lines with 8 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week.
> 
> Measured: 1737 production lines (test modules excluded); longest function build_reading at 147 lines; 8 landings in the last 7 days; 10 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: 1737 production lines with 8 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/reading.rs

## Reading (accepted by the user)

Restructure crates/athena-core/src/reading.rs (about 1737 production lines, 8 landings in 7 days) by moving cohesive groups into child modules under crates/athena-core/src/reading/. Candidates are the retry and carry-over helpers, the delivery-verb and outcome-state check, the defined-terms check, and the reference and boundary checks. Public paths stay the same through re-exports. Behavior does not change. The audience is the code-health gate and maintainers.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (top-level item outline only) and listed crates/athena-core/src/. The file is declared in lib.rs, and its items are used through crate::reading:: by run/, steward.rs, plan.rs, status.rs, hestia.rs, tests/refusal_corpus.rs and the athena-cli crate. No record or data file is added, so the records allow-list is untouched.
- The file has many sections beyond impl Athena: the retry and carry code, the brief-topic code, the check_* functions, the delivery-verb and outcome-state tables, and the defined-terms and quoted-span helpers. Each section is self-contained and can move to its own child module with `pub use` re-exports, so callers do not change.
- Converting reading.rs to a module with child files keeps reading.rs as the parent, using Rust's non-mod-rs layout (reading.rs plus reading/*.rs). Only reading.rs is in the request's Files line, so the new child files are the extra deliverables.
- Tests that sit in the file's test module may need to stay in the parent or move with their code. Their assertions are not changed.
- Code health counts production lines per file. The target is the parent file under 1500 lines, and the children are well under the limits too. No function over 200 lines is introduced.
- Environment: none. This is a local refactor with no CI, push or network.

## Acceptance criteria

1. [content] crates/athena-core/src/reading.rs has fewer than 1500 production lines (test modules excluded). Every new child file under crates/athena-core/src/reading/ is also under 1500 lines, and no function exceeds 200 lines. Check by reading the files.
2. [content] Every previously public item of crate::reading (for example check_defined_terms, check_outcome_state, plan_entry, earlier_attempt, retry_findings, same_scope) is still reachable at the same path. The diff shows `pub use` re-exports and no edits to callers outside reading.rs, apart from mod declarations.
3. [content] The diff is a pure move. Function bodies, constants and tests are unchanged apart from visibility (pub(super) or pub(crate)) and import paths. No logic edits.
4. [behavior] Run `cargo test -p athena-core`, including tests/refusal_corpus.rs and the reading tests. All pass, with the same tests and no removed or ignored ones.
5. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean, and `cargo build` succeeds for the whole workspace.
6. [content] athena.toml, defaults.toml and the workflows are not changed. The change is one focused structural split.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/retry.rs (create)
- crates/athena-core/src/reading/checks.rs (create)
- crates/athena-core/src/reading/terms.rs (create)
- crates/athena-core/src/reading/outcome.rs (create)
- crates/athena-core/src/reading/topics.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Any behavior change, new check or refusal rule
- Changes to callers outside reading.rs beyond what re-exports make unnecessary
- Splitting other large files
- Edits to athena.toml, defaults.toml or CI workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust code only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One structural split with no logic changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A verifier runs cargo test, clippy and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test suite is the check for unchanged behavior, plus a diff check that confirms the change is a pure move.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Several cohesive responsibilities move out, so the file stays under the limit and does not creep back over it.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 38d5cd1b4732 → candidate 4af71c867269. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |    4 +
 crates/athena-core/src/reading.rs          | 4171 +---------------------------
 crates/athena-core/src/reading/checks.rs   |  657 +++++
 crates/athena-core/src/reading/outcome.rs  |  352 +++
 crates/athena-core/src/reading/replay.rs   |  410 +++
 crates/athena-core/src/reading/retry.rs    |  839 ++++++
 crates/athena-core/src/reading/surveys.rs  |  996 +++++++
 crates/athena-core/src/reading/terms.rs    |  300 ++
 crates/athena-core/src/reading/topics.rs   |  560 ++++
 crates/athena-core/tests/refusal_corpus.rs |   45 +-
 10 files changed, 4210 insertions(+), 4124 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-144236-qvvv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144236-qvvv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42461965381 for 4af71c867269797c16cec2462c9f50857b5751e1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42461965381 for 4af71c867269797c16cec2462c9f50857b5751e1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42461965381 for 4af71c867269797c16cec2462c9f50857b5751e1)
- ci-result: `not a check` passed (Check results for 4af71c867269797c16cec2462c9f50857b5751e1 come from the CI run https://github.com/doodla/athena/actions/runs/42461965381 (760s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean, and `cargo build` succeeds for the whole workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
