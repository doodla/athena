# Job 20261108-110309-qbwe: verify "Move status rendering out of display.rs and repoint source-reading test"

## Request (verbatim)

> try again on 20261026-193430-avnq

## Reading (accepted by the user)

Retry of job 20261026-193430-avnq: move the status-rendering group from crates/athena-cli/src/display.rs into display/status_view.rs (re-exported, bodies verbatim) to get under the 1500-line/5-landing limit, and also repoint the athena-core test that reads display.rs's source text, which broke last time. Audience: the code-health gate and maintainers; no behavior change.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (test mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line at ~1402 uses include_str!("../../athena-cli/src/display.rs") and searches for printed label strings like "code health ") and crates/athena-cli/src/display.rs (imports at top, tests module at ~1944). That include_str! is the only source-text reader of display.rs found in crates/ by Grep.
- Job 20261026-193430-avnq: its candidate (CHANGELOG.md entry, display.rs trimmed to ~1030 lines, new display/status_view.rs) is kept, redone from base. Its blocker (athena-core test reading display.rs text broke) is fixed by adding include_str!("../../athena-cli/src/display/status_view.rs") to the searched sources in status.rs. Its major finding (reading narrowed tests to athena-cli) is fixed by requiring the full workspace test. The nit (misplaced repeated-causes doc comment) is carried verbatim and left alone as a pure move.
- display.rs becomes a directory-module root with a child file display/status_view.rs declared by `mod status_view;` and `pub use status_view::{status, snapshot_status};`; lib.rs probably needs no change (listed with low probability).
- The test keeps checking the same labels; only its source set widens, so it is not weakened.
- Moved bodies are verbatim; helpers shared with other display functions stay in display.rs, with visibility widened to pub(super) only where needed.

## Acceptance criteria

1. [content] crates/athena-cli/src/display.rs has fewer than 1500 production lines (test module excluded) and the status-rendering functions (plan_suffix, status, snapshot_status, code_health_section, introspection and their section helpers) are defined in the new crates/athena-cli/src/display/status_view.rs, checkable with Grep/Read.
2. [content] display.rs re-exports status and snapshot_status so no caller outside display changes; moved function bodies are verbatim (git diff shows only moves plus visibility/import edits).
3. [content] crates/athena-core/src/status.rs test every_measure_label_names_a_status_view_field_and_a_printed_line additionally searches include_str!("../../athena-cli/src/display/status_view.rs"); its label list and assertions are unchanged, and no test is deleted or weakened.
4. [behavior] Run `cargo test --workspace`: all tests pass, including athena-core status::mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line and the athena-cli display tests.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --workspace --all-targets`: both clean.
6. [content] CHANGELOG.md has a Changed entry for the split; no governance files (athena.toml, defaults.toml, Cargo configs) change.

## Expected deliverables

- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/display/status_view.rs (create)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/lib.rs (modify)

## Out of scope

- Behavior or output format changes
- Splitting other files or raising limits
- Refactoring the status function itself
- Fixing the misplaced doc comment on trend_section/repeated_causes_section
- Changes to athena.toml, defaults.toml or other governance files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust module split.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Full workspace test and clippy required.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Finds and repoints the source-text-reading test, not just display.rs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run workspace tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome is gate-passing size with all tests green.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 78838ebc83a0 → candidate aa952b2adf54. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |   4 +
 crates/athena-cli/src/display.rs             | 492 +--------------------------
 crates/athena-cli/src/display/status_view.rs | 490 ++++++++++++++++++++++++++
 crates/athena-core/src/status.rs             |   3 +-
 4 files changed, 504 insertions(+), 485 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-110309-qbwe/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-110309-qbwe/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42474932176 for aa952b2adf54c03fcee0a958b03edbae819250c7)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42474932176 for aa952b2adf54c03fcee0a958b03edbae819250c7)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42474932176 for aa952b2adf54c03fcee0a958b03edbae819250c7)
- ci-result: `not a check` passed (Check results for aa952b2adf54c03fcee0a958b03edbae819250c7 come from the CI run https://github.com/doodla/athena/actions/runs/42474932176 (765s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --workspace`: all tests pass, including athena-core status::mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line and the athena-cli display tests.; Run `cargo fmt --check` and `cargo clippy --workspace --all-targets`: both clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
