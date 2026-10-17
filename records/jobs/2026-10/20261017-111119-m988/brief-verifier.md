# Job 20261017-111119-m988: verify "Version the scoring rules so old scores still verify; gate rule changes"

## Request (verbatim)

> PR #129 (job 20261010-154004-60r4) fails CI's 'the full history' check: its new rule marks a bound-only cost score provisional, and 'athena check --history' now finds 8 existing score files (e.g. records/jobs/2026-10/20261013-141005-hrdp/score.json) that do not follow from their records because they predate the rule. Make the rule hold without invalidating history written before it, without hand-editing records, and make a change to scoring rules get checked against the full history before it lands.

## Reading (accepted by the user)

Goal: PR #129's provisional-cost rule must pass `athena check --history` without editing any record, and future scoring-rule changes must be checked against full history before landing. Read as: bump the rules version (v1→v2) so bound-only costs score provisional only under v2, keep recomputing v1 scores under v1 behavior, and add a mechanism that runs the history score check when scoring code changes. Audience: maintainers and CI.

Assumptions:
- PR #129's code is already in the working tree (provisional field and cost_is_only_a_bound exist in athena-score), so this change builds on it.
- The gate already checks `score.rules == RULES` and recomputes the score; the cleanest fix is a known-rules list with legacy behavior for v1, not a [history] exempt entry (which would hide the problem).
- The 'before it lands' mechanism can be a test or xtask ci step that runs the history score recompute, since CI's history job already exists but ran only after the change was written.
- Exact file paths beyond athena-score and athena-gate/jobs.rs are inferred.

## Acceptance criteria

1. [behavior] A new scorer test shows the same job records score with a bound-only cost provisional under the current rules string and not provisional under the previous v1 rules string; run `cargo test -p athena-score`.
2. [behavior] A gate test shows a score file stamped with the v1 rules string, whose cost was a bound, still passes the score-follows-records check, and a v2 score with the wrong provisional flag is still rejected; run `cargo test -p athena-gate`.
3. [behavior] `cargo run -p athena -- check --history` (or the repository's equivalent) over the repository reports no violation for the 8 existing score files, including records/jobs/2026-10/20261013-141005-hrdp/score.json, and no file under records/ is modified in the diff.
4. [content] New scores are written under a bumped rules string (v2); the gate recomputes each score under the rules string it carries and still refuses an unknown rules string.
5. [behavior] A mechanism, not prose, makes a change to scoring behavior get checked against the full history before landing: a test or the xtask ci step recomputes every historical score with the working-tree scorer and fails if the rules string was not bumped; run `cargo run -p xtask -- ci` or `cargo test` to see it pass, and a test shows it fails when a rule changes under an unchanged version.
6. [behavior] `cargo test` passes, `cargo clippy` is clean, and `cargo fmt --check` passes.
7. [content] The call (versioned rules over rewriting history or exempting records) is recorded with alternatives in docs/decisions.md.

## Expected deliverables

- crates/athena-score/src/lib.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- xtask/src/main.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- crates/athena-data/src/records.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Editing or regenerating any file under records/
- Showing provisional status in the status display (athena-cli display.rs)
- Changing the kill-switch outcome logic from job 60r4
- Other rules besides scoring rules

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for both rule versions and the new check; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The history check on rule changes is enforced by a test or xtask step.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix by rule versioning rather than exempting the 8 files.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests and the history check.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Decision recorded with alternatives in docs/decisions.md.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change around scoring rule versioning.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 6aa6b48ddcb9 → candidate 4897df98a90a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   7 ++
 crates/athena-gate/src/jobs.rs          | 161 ++++++++++++++++++++++++++++++--
 crates/athena-gate/src/landing_tests.rs |  38 ++++++++
 crates/athena-score/src/lib.rs          | 109 ++++++++++++++++++++-
 docs/decisions.md                       |  25 +++++
 5 files changed, 326 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (858 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself and run `cargo test --locked` if a test result is missing: no real full `cargo test --locked` result; acceptance criteria not met or not reported: "A new scorer test shows the same job records score with a bound-only cost provisional under the current rules string and not provisional under the previous v1 rules string; run `cargo test -p athena-score`."; "A gate test shows a score file stamped with the v1 rules string, whose cost was a bound, still passes the score-follows-records check, and a v2 score with the wrong provisional flag is still rejected; run `cargo test -p athena-gate`."; "`cargo run -p athena -- check --history` (or the repository's equivalent) over the repository reports no violation for the 8 existing score files, including records/jobs/2026-10/20261013-141005-hrdp/score.json, and no file under records/ is modified in the diff."; "New scores are written under a bumped rules string (v2); the gate recomputes each score under the rules string it carries and still refuses an unknown rules string."; "A mechanism, not prose, makes a change to scoring behavior get checked against the full history before landing: a test or the xtask ci step recomputes every historical score with the working-tree scorer and fails if the rules string was not bumped; run `cargo run -p xtask -- ci` or `cargo test` to see it pass, and a test shows it fails when a rule changes under an unchanged version."; "`cargo test` passes, `cargo clippy` is clean, and `cargo fmt --check` passes."; "The call (versioned rules over rewriting history or exempting records) is recorded with alternatives in docs/decisions.md.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
