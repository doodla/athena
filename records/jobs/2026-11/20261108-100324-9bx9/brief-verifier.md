# Job 20261108-100324-9bx9: verify "Split status.rs into modules and shorten status() under limits"

## Request (verbatim)

> try again on 20261026-144242-sjp3

## Reading (accepted by the user)

Retry of job 20261026-144242-sjp3 (rejected: the earlier reading trusted a stale 916-line figure; at base status.rs has about 2439 production lines before the test modules at line 2440, so it breaches the 2000 file limit as well as the 200-line `status` limit). Goal: make status.rs pass code health on both limits, for code health and maintainers. Move the throughput/waste code and the planning/keep-rule code into submodules under crates/athena-core/src/status/, re-exported so paths and tests are unchanged, and extract helpers from `status`.

Assumptions:
- Prior job 20261026-144242-sjp3 findings addressed: (1) major, file still over 2000 production lines: fixed by splitting into submodules, moving the old 'splitting is out of scope' item into scope; (2) major, reading mismatch with request: this reading adds a criterion for the file limit and requires measured before/after counts; (3) minor, status only 15 lines under the limit and the job loop not extracted: status must end at 150 lines or fewer, with the job loop moved into a helper; (4) nit, no tests for helpers: existing tests keep covering them via Athena::status, and the producer may add tests but need not. Its candidate changes to status.rs and CHANGELOG.md were discarded and are redone from base; both files stay deliverables.
- Measured with Grep over the base snapshot: status.rs has `mod mistake_tests` at line 2440, so about 2439 production lines (the verifier's 2478 figure counts differently, so re-measure with code health's method). Throughput/waste/LandedJob/Bars/BuildWait code spans roughly lines 575-1416 and planning/keep-rule/later-harm/producer-settings spans roughly lines 120-490. Moving both leaves about 1200-1300 production lines.
- Code survey: read crates/athena-core/src/status.rs (items listed by Grep: Athena::status at line 2079, trace at 2349, tests from 2440 using the file's items) and found no existing status/ directory. lib.rs declares `mod status`; submodules go in crates/athena-core/src/status/ and are re-exported with `pub use` from status.rs so existing `crate::status::X` paths, the mistake_tests and build_measurement_tests, and other crates compile unchanged. No new record or data file is added, so the records validation allow-list, check_record_file and AnyRecord are not touched. The line count is measured directly with code_health::measure_source on the file.
- Dependencies: the new modules use `use super::*` or explicit imports; private items used across modules become pub(super) or pub(crate). Visibility changes must not widen the public API of the crate beyond the existing re-exports.
- Tests in status.rs may need imports adjusted; that is allowed, but no test body, assertion or name is deleted or weakened.

## Acceptance criteria

1. [content] crates/athena-core/src/status.rs has fewer than 1500 production lines (total lines minus #[cfg(test)] module ranges, counted as code_health::measure_source does), down from about 2439 at base. A reader can count it from the file, and the before and after figures appear in the producer's final report and the CHANGELOG entry.
2. [content] No function in status.rs or the new files under crates/athena-core/src/status/ is longer than 200 lines; `Athena::status` is 150 lines or fewer and delegates the job loop and next-line construction to named private helpers.
3. [content] New modules live under crates/athena-core/src/status/, split by responsibility (throughput/waste, planning cost and keep rule), each under 2000 lines and each item re-exported from status.rs so every existing `crate::status::` path still resolves. Public signatures, StatusView fields, and next-line texts and order are unchanged.
4. [content] No existing test is edited in substance or deleted, no #[allow] or other lint suppression is added, and every new module and extracted helper has a doc comment written for an outside reader.
5. [content] CHANGELOG.md gains one Changed entry saying status.rs was split into modules, with the before and after production line counts.
6. [behavior] Run `cargo test` from the repository root: it passes with the existing status tests, including mistake_tests and build_measurement_tests, unchanged.
7. [behavior] Run `cargo clippy --all-targets` (no warnings) and `cargo fmt --check` (clean).

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/status/throughput.rs (create)
- crates/athena-core/src/status/planning.rs (create)
- CHANGELOG.md (modify)
- crates/athena-core/src/lib.rs (modify)

## Out of scope

- Changing the status output, next-step text, StatusView shape or any public signature
- Changing the `trace` method or other behavior
- Changing code-health limits, athena.toml or other governance files
- Adding a reading-acceptance code check (the introspection's prevention proposal, already tried in earlier cases)
- Editing or deleting existing tests

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One refactor with no behavior change.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria measure the file against the limit, not the activity of splitting.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Line counts and the tests are the checks.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the file-size breach, not only the long function.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments on new modules and helpers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8f24e3b8a292 → candidate bc2df795a494. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |    6 +
 crates/athena-core/src/status.rs            | 1554 +++------------------------
 crates/athena-core/src/status/planning.rs   |  449 ++++++++
 crates/athena-core/src/status/throughput.rs |  846 +++++++++++++++
 4 files changed, 1462 insertions(+), 1393 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-100324-9bx9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-100324-9bx9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42470341976 for bc2df795a4949b7d39d674a9f985b8cb84e3eb97)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42470341976 for bc2df795a4949b7d39d674a9f985b8cb84e3eb97)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42470341976 for bc2df795a4949b7d39d674a9f985b8cb84e3eb97)
- ci-result: `not a check` passed (Check results for bc2df795a4949b7d39d674a9f985b8cb84e3eb97 come from the CI run https://github.com/doodla/athena/actions/runs/42470341976 (636s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` from the repository root: it passes with the existing status tests, including mistake_tests and build_measurement_tests, unchanged.; Run `cargo clippy --all-targets` (no warnings) and `cargo fmt --check` (clean).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
