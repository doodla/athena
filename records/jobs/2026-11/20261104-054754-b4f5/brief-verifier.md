# Job 20261104-054754-b4f5: verify "Make the decisions page read approve/reject decisions from main"

## Request (verbatim)

> try again on 20261103-201526-t4qv with the root cause named: the published decisions page reads steward work items from the athena/records branch, but the approve-work and reject-work decisions are recorded under records/work-decisions/ on main (for example records/work-decisions/2026-10/20261030-081746-a1he.json, which settles 20261017-204324-jtd2) and that directory is absent from athena/records. So the page never sees any decision and lists every proposal as waiting. Fix the page to read the decisions from where they are recorded (or copy them to the records branch with the work items), so decided proposals drop off. Acceptance: a test that builds the page from a records tree where the work item is on the records branch and its decision is on main, and the proposal is not listed.

## Reading (accepted by the user)

Retry of 20261103-201526-t4qv with the root cause named: decisions.html reads work items from athena/records, but approve-work and reject-work decisions are recorded under records/work-decisions/ on main (for example 2026-10/20261030-081746-a1he.json, which settles 20261017-204324-jtd2). The page therefore never sees a decision and lists every proposal. The fix makes xtask/src/decisions.rs also read decisions from main's tree so decided proposals drop off. The audience is the owner reading the page.

Assumptions:
- Code survey: read xtask/src/decisions.rs. read_snapshot(git, tip) lists records/work-items, records/work-decisions and records/roles only under the athena/records tip, using ls_tree_under and check_record_file. It builds a decided set from WorkDecision records and retains only undecided proposals. locate() resolves the tip with resolve_tip in crates/athena-git, then calls read_snapshot. render() shows the cards. crates/athena-data/src/records.rs (around line 2979) writes decisions to records/work-decisions/<month>/<id>.json. No record schema or allow-list changes, so check_record_file and AnyRecord stay as they are.
- Root cause, to be stated in the CHANGELOG entry and the producer report: the decided filter in read_snapshot already worked, but its input was incomplete. Decisions are committed to main, not to athena/records, so the decided set was empty and no proposal was dropped. .github/workflows/pages.yml checks out main and fetches athena/records, so main's tree (HEAD) is available to the renderer without a workflow change.
- Fix choice: also read WorkDecision records from main's tree (HEAD of the checkout) and union them into the decided set. Copying decisions to the records branch is not chosen because it would change approve-work/reject-work behavior. If HEAD has no records/work-decisions, the result is the same as today.
- The earlier attempt's 'Nothing waiting for you' empty state, the within-limit filter, and the stale-page theory are not carried over. The user's retry names a different cause, and that cause is addressed here. Job 20261103-201526-t4qv findings: the unnamed root cause is fixed by this reading; the in_limit duplication finding no longer applies because no limit logic is added; the test that also passed on old code is fixed by requiring a test that fails on old code. Its two changed files, xtask/src/decisions.rs and CHANGELOG.md, are redone from the base. The attempt's other work is dropped.
- The test builds a temp git repo with RecordsWriter or plain git: work item on athena/records, decision on main, and calls locate() then render(). A real-shape fixture is used, modeled on records/work-decisions/2026-10/20261030-081746-a1he.json.

## Acceptance criteria

1. [behavior] A new test in xtask/src/decisions.rs builds a temp git repo where a proposal work item is on the athena/records branch and its Approved (and a second, Rejected) WorkDecision is under records/work-decisions/ on main only, runs locate() and render(), and asserts the proposal's id is absent and no '<article' appears. Run: cargo test -p xtask decisions.
2. [behavior] A companion test with an undecided proposal in the same layout asserts it is still listed as one card, so the fix does not hide everything. Run: cargo test -p xtask decisions.
3. [behavior] The new decided-on-main test fails on the base code. The verifier checks this by reading read_snapshot in the base (git show of the base file), where decisions come only from the records tip, and by inspecting that the test's decision is absent from that tip. Run: cargo test -p xtask decisions on the candidate.
4. [content] xtask/src/decisions.rs read_snapshot() reads work decisions from main's tree as well as from the records branch, and the module doc says decisions are read from both places and why.
5. [content] CHANGELOG.md has one entry that names the root cause (decisions are recorded on main while the page read only athena/records) and says only what the diff does, with no claim about the workflow or other behavior.
6. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- CHANGELOG.md (modify)
- xtask/src/site.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Editing records/ files or recording new decisions
- athena.toml, defaults.toml, rust-toolchain.toml, .gitattributes, setup.sh and .github/ workflows
- The 'Nothing waiting for you' empty state and the within-limit (steward limit) filter from job 20261103-201526-t4qv; they are dropped
- The docs and design work from job 20261030-081751-edsg
- Changing approve-work or reject-work behavior or the record schemas in crates/athena-data
- Redesigning the card layout or the 160-word brief limit

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the data source the page reads, not by moving a filter; the cause is named in the CHANGELOG.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test runs locate() and render() over a records branch plus a main tree with real-shape decisions.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that decided proposals are absent from the rendered page.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in decisions.rs plus the changelog.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8d6576271d2e → candidate 678e1ea9bfbf. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md           |   4 ++
 xtask/src/decisions.rs | 108 ++++++++++++++++++++++++++++++++++++++++++++++++-
 xtask/src/site.rs      |   3 +-
 3 files changed, 113 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41783495317 for 678e1ea9bfbf202aa3aae17496e6b0bfc4128a19)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41783495317 for 678e1ea9bfbf202aa3aae17496e6b0bfc4128a19)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41783495317 for 678e1ea9bfbf202aa3aae17496e6b0bfc4128a19)
- ci-result: `not a check` passed (Check results for 678e1ea9bfbf202aa3aae17496e6b0bfc4128a19 come from the CI run https://github.com/doodla/athena/actions/runs/41783495317 (456s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy --all-targets and cargo fmt --check all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
