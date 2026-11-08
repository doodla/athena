# Job 20261026-144231-a31y: verify "Split records.rs into submodules under the code-health limits"

## Request (verbatim)

> Structural pass: crates/athena-data/src/records.rs
> 
> Code health found crates/athena-data/src/records.rs over a limit: 2543 production lines, over the limit of 2000; 2543 production lines with 7 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week.
> 
> Measured: 2543 production lines (test modules excluded); longest function canonical_path at 85 lines; 7 landings in the last 7 days; 17 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: 2543 production lines, over the limit of 2000
> - The file is under the limit it breached, as measured by code health: 2543 production lines with 7 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-data/src/records.rs

## Reading (accepted by the user)

Restructure crates/athena-data/src/records.rs (2543 production lines) by moving cohesive groups of record types into submodules under crates/athena-data/src/records/, re-exported from records so every public path is unchanged. The goal is for code health to see records.rs under 1500 production lines, with no behavior change; the audience is the code-health gate and maintainers.

Assumptions:
- Code Survey: Code survey: read crates/athena-data/src/records.rs and crates/athena-data/src/lib.rs (pub mod records). records.rs holds ~3200 lines: core job/invocation/verification types (top), introspection, failure-class, proposal, work-item, audience-rating, audit, throughput, build-measurement, Hestia sweep, job-plan/critique, baseline and replay types, then the AnyRecord enum, RecordError, AnyRecord impl, check_record_file and is_job_dir, then #[cfg(test)] mod tests at line 3216.
- Code survey: no record or data file is added, so the check_record_file/AnyRecord allow-list and round-trip test are unchanged; AnyRecord and check_record_file stay in records.rs and keep working through re-exports.
- Plan: move self-contained groups (e.g. introspection+class labels+proposals; work items+audience+audit; throughput+build measurement+Hestia; plan/critique/baseline/replay) into records/*.rs as private modules with `pub use` in records.rs, preserving every public path such as athena_data::records::Foo. Items used across modules use pub(crate)/use super::*. Leave the tests module in place so tests still compile.
- Limit reading: code health counts production lines excluding test modules; the 1500-line limit is the tighter target, so records.rs should end clearly below 1500 production lines and each new file also below it.
- No behavior change: no signatures, serde attributes, schema constants or validation logic are edited, only moved.

## Acceptance criteria

1. [behavior] cargo test -p athena-data passes with the existing tests unchanged in substance (run cargo test).
2. [behavior] cargo clippy --all-targets and cargo fmt --check are clean.
3. [content] crates/athena-data/src/records.rs has well under 1500 production lines (excluding the cfg(test) module), and every new file under crates/athena-data/src/records/ is also under 1500; verifiable with Read/Grep on line counts.
4. [content] All previously public items remain reachable at the same athena_data::records::* paths via pub use re-exports; no other crate needs import edits (cargo check --workspace passes).
5. [content] The diff only moves code: no changes to serde attributes, schema constants, field types, or function bodies, checkable with git diff.
6. [content] AnyRecord, check_record_file and RecordError remain in records.rs and still cover every moved record type.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/records/introspection.rs (create)
- crates/athena-data/src/records/work.rs (create)
- crates/athena-data/src/records/throughput.rs (create)
- crates/athena-data/src/records/plan.rs (create)
- crates/athena-data/src/records/core.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Behavior changes, renamed public items, or edits to other crates
- Splitting the tests module
- Changes to athena.toml or code-health limits

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust refactor.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move-only refactor.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test/clippy/fmt.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Split by responsibility, not trimming lines.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier runs cargo checks.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c37726ef8ac8 → candidate 38e7cb23b2c2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                    |    3 +
 crates/athena-data/src/records.rs               | 1821 +----------------------
 crates/athena-data/src/records/core.rs          |  265 ++++
 crates/athena-data/src/records/introspection.rs |  549 +++++++
 crates/athena-data/src/records/plan.rs          |  336 +++++
 crates/athena-data/src/records/throughput.rs    |  374 +++++
 crates/athena-data/src/records/work.rs          |  302 ++++
 7 files changed, 1840 insertions(+), 1810 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-144231-a31y/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144231-a31y/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42434346585 for 38e7cb23b2c25237aa779b9616cb12cfd0a71431)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42434346585 for 38e7cb23b2c25237aa779b9616cb12cfd0a71431)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42434346585 for 38e7cb23b2c25237aa779b9616cb12cfd0a71431)
- ci-result: `not a check` passed (Check results for 38e7cb23b2c25237aa779b9616cb12cfd0a71431 come from the CI run https://github.com/doodla/athena/actions/runs/42434346585 (531s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo clippy --all-targets and cargo fmt --check are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
