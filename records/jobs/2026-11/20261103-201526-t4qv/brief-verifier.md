# Job 20261103-201526-t4qv: verify "Show only open, above-limit proposals on the decisions page"

## Request (verbatim)

> The published decisions page (https://doodla.github.io/athena/decisions.html) lists steward proposals that were already decided as 'waiting for your decision'. On 3 Nov it showed 3 waiting: work item 20261016-055247-qyd9 was approved on 30 Oct (job 20261030-081751-edsg), 20261017-204324-jtd2 was rejected, and 20261018-000648-ne91 is within the steward's own limit, so it is not the owner's decision at all. Show only proposals that are still open and above the steward's limit; when there are none, say 'Nothing waiting for you' in one line. Acceptance: a test where an approved, a rejected, and a within-limit proposal produce 'Nothing waiting for you'; a test where one open above-limit proposal produces one card.

## Reading (accepted by the user)

The owner decisions page (xtask/src/decisions.rs) must list a proposal only if no approve or reject decision names it and its rung is above the raising role's limit; with none, it says "Nothing waiting for you" in one line. Audience: the owner reading decisions.html. Job 20261030-081751-edsg (a different docs task, dropped, no candidate files) is not continued; its outputs are not kept.

Assumptions:
- Code survey: xtask/src/decisions.rs is the entry point: locate() resolves the records branch tip, read_snapshot() reads records/work-items, records/work-decisions and records/roles via athena_data::records::check_record_file, drops proposals named by any WorkDecision, and render() builds the page. Brief::new/recommendation() already computes within-limit via role_limit() and athena_data::roles::may_change (crates/athena-data/src/roles.rs), but within-limit proposals are still rendered as cards. WorkDecision and WorkDecisionState (Approved{job}/Rejected{reason}) are in crates/athena-data/src/records.rs and are already accepted by check_record_file, so no new record or data file is added and AnyRecord is unchanged. The property is measured directly by calling render(&Records::Read(Snapshot{..})) and inspecting the HTML.
- Existing code: the open-proposal filter in read_snapshot (xtask/src/decisions.rs) already removes proposals named by a decision; the producer should find why an approved item (20261016-055247-qyd9, approved by job 20261030-081751-edsg) and a rejected one (20261017-204324-jtd2) still showed on 3 Nov (e.g. decision records unreadable, missing from the fetched records branch, or the published page built from a stale branch) and fix the root cause, while also making render() itself drop decided and within-limit proposals so the rule holds for any Snapshot.
- A role with no recorded definition is treated as above the limit (owner's decision), as the existing recommendation() does; within-limit means may_change(limit, rung) is true for the newest recorded role version.
- Job 20261030-081751-edsg's planned files (docs/design/outcome-check-overrates.md, which does not exist in the snapshot, docs/decisions.md, docs/design/audience-role.md) are dropped, not kept; this change does not touch them.
- Existing tests in xtask/src/decisions.rs that assert '2 waiting' or the old empty-state wording 'No proposal is waiting for a decision' are updated to the new rule and wording.
- Standing defaults applied: tested (cargo test and clippy pass), verified, behavior-evidence, root-cause, small, rust.

## Acceptance criteria

1. [behavior] A new test in xtask/src/decisions.rs builds a Snapshot with three proposals (one with an Approved WorkDecision, one with a Rejected WorkDecision, one raised by a role whose limit covers its rung) and asserts render() contains 'Nothing waiting for you', contains no '<article', and has no 'N waiting' heading. Run: cargo test -p xtask decisions.
2. [behavior] A new test builds a records branch (or Snapshot) with one open proposal above the raising role's limit and asserts render() shows exactly one '<article' card and the heading '1 waiting'. Run: cargo test -p xtask decisions.
3. [behavior] A test through the records branch path (locate() on a temp git repo with RecordsWriter) covers an approved decision whose job is set, confirming the approved proposal is not listed. Run: cargo test -p xtask decisions.
4. [content] The empty state is a single line reading 'Nothing waiting for you' in xtask/src/decisions.rs render(), replacing the old 'No proposals waiting' section; the missing-records-branch message stays distinct.
5. [content] Existing tests in xtask/src/decisions.rs that conflicted with the new rule are updated, and the module doc comment states that only open, above-limit proposals are listed.
6. [behavior] cargo test and cargo clippy --all-targets pass and cargo fmt --check is clean.
7. [content] CHANGELOG.md has one line describing the decisions page fix.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- CHANGELOG.md (modify)
- xtask/src/site.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing records/ files or recording decisions for the three named work items
- athena.toml, defaults.toml, rust-toolchain.toml, .gitattributes, setup.sh and .github/ workflows
- The docs and design work from job 20261030-081751-edsg (docs/design/outcome-check-overrates.md, docs/decisions.md, docs/design/audience-role.md)
- Changing approve-work or reject-work behavior or the record schemas in crates/athena-data
- Redesigning the card layout or the 160-word brief limit

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the empty and one-card cases; cargo test and clippy criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered HTML from render(), not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Producer must find why decided items still showed, not only add a filter.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust in xtask.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file of code plus changelog.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test for the named tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bdb41b9f1cef → candidate 0bf201557d0c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md           |   1 +
 xtask/src/decisions.rs | 128 ++++++++++++++++++++++++++++++++++++++++++++++---
 2 files changed, 123 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41779936603 for 0bf201557d0caaeeafc2e79a7527f1364bc15520)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41779936603 for 0bf201557d0caaeeafc2e79a7527f1364bc15520)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41779936603 for 0bf201557d0caaeeafc2e79a7527f1364bc15520)
- ci-result: `not a check` passed (Check results for 0bf201557d0caaeeafc2e79a7527f1364bc15520 come from the CI run https://github.com/doodla/athena/actions/runs/41779936603 (617s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test and cargo clippy --all-targets pass and cargo fmt --check is clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
