# Job 20261026-232429-r147: verify "Move Planning and Pin/reread rows into run/planning.rs and run/pin.rs"

## Request (verbatim)

> Priority: high
> Structure pass job R3 (docs/design/structure-pass.md, section 4). Move the Planning, pin and reread rows. No behavior change. Depends on 20261026-232423-hvyb and 20261026-232426-91cn.
> Acceptance:
> - The proof of section 5 holds: existing tests pass with only import lines edited, the test list is unchanged, moved text is the same text, public paths are unchanged through re-exports, and the report lists the caller check for each moved name.
> - cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/pin.rs, CHANGELOG.md

## Reading (accepted by the user)

Structure pass R3: move the Planning row (stop_planning, plan_round, planning_round, end_round, RoundEnd, findings_line, plan_record, redacted, planning_grant and planning_grant_tests) to run/planning.rs and the Pin and reread row (pin_repo, study_pin, reread_after_configuration_change) to run/pin.rs, with no behavior change. Follows landed R2 (20261026-232426-91cn, its work is kept) and R1 (20261026-232423-hvyb, abandoned but superseded by a retry that landed, so nothing from it is redone). Audience: the maintainer judging the structure-pass proof.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs: it has `mod send_back;` plus `pub(crate) use send_back::{...}` from R2; Planning items (RoundEnd ~1479, findings_line, plan_record, redacted, planning_grant ~3481, planning_grant_tests ~3490) sit in mod.rs. The row line ranges come from docs/design/structure-pass.md rows 63-64 and 112-113, which may be stale after R1/R2.
- Job 20261026-232426-91cn (R2) landed with a minor finding: the design doc contradicts itself on whether test modules move. This job follows R2's precedent: planning_grant_tests moves with planning_grant (as the design's Planning row lists it), so its test path becomes run::planning::planning_grant_tests::* with the same short names. The doc fix is out of scope. R2's files (send_back.rs, mod.rs, CHANGELOG.md) are kept as landed; mod.rs and CHANGELOG.md are modified again.
- Job 20261026-232423-hvyb (R1) was abandoned with no candidate and no verdict; its outcome was superseded by a retry that landed run/mod.rs, so no finding remains to address.
- Moved names keep their crate::run paths via `mod planning; mod pin;` and `pub(crate) use` re-exports in mod.rs; items used only inside the submodules are visible as pub(super) as needed. Only import lines in existing tests are edited.
- The caller check is done by Grep for each moved name over crates/, and the producer lists the results in its report.
- Governance files (athena.toml, defaults.toml, workflows) are not needed.

## Acceptance criteria

1. [content] crates/athena-core/src/run/planning.rs exists and contains stop_planning, plan_round, planning_round, end_round, RoundEnd, findings_line, plan_record, redacted, planning_grant and mod planning_grant_tests; Grep over run/mod.rs finds no definitions of them.
2. [content] crates/athena-core/src/run/pin.rs exists and contains pin_repo, study_pin and reread_after_configuration_change; Grep over run/mod.rs finds no definitions of them.
3. [content] The git diff shows the moved function and test bodies identical to the removed text (only visibility and import lines differ), and existing tests change only in their import lines.
4. [content] run/mod.rs declares the new modules and re-exports every moved name that was reachable by a path outside the module, so no caller file outside run/ is edited (git diff --stat lists only mod.rs, planning.rs, pin.rs and CHANGELOG.md).
5. [content] CHANGELOG.md has an Unreleased entry for the move that says there is no behavior change.
6. [behavior] Run `cargo test --locked`: it passes and the test list is the same as before apart from the module prefix of the moved planning_grant_tests (names unchanged). Also run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both are clean.

## Expected deliverables

- crates/athena-core/src/run/planning.rs (create)
- crates/athena-core/src/run/pin.rs (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any other row of run/mod.rs (producer, verifier, etc.; later R jobs)
- Editing tests beyond import lines
- Changing athena.toml, defaults.toml or workflows
- Redoing R1 or R2, already landed
- Correcting the docs/design/structure-pass.md contradiction about test modules
- Any behavior change or deduplicating test helpers

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the two rows move; one focused change.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs cargo test to show unchanged behavior.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Passing existing tests with only import edits is the check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog entry and clear module layout.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5106de0f28c8 → candidate bc0dbb1cfc28. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   5 +
 crates/athena-core/src/run/mod.rs      | 726 +--------------------------------
 crates/athena-core/src/run/pin.rs      | 193 +++++++++
 crates/athena-core/src/run/planning.rs | 541 ++++++++++++++++++++++++
 4 files changed, 741 insertions(+), 724 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1617 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked`: it passes and the test list is the same as before apart from the module prefix of the moved planning_grant_tests (names unchanged). Also run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
