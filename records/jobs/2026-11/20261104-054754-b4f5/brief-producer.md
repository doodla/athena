# Job 20261104-054754-b4f5: Make the decisions page read approve/reject decisions from main

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

## Findings you must address

The verifier of job 20261103-201526-t4qv raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (xtask/src/decisions.rs read_snapshot()/render(); CHANGELOG.md): The root cause of approved and rejected proposals showing on the published page is neither found, fixed nor named; for decided proposals the change only moves an existing filter. In the old code, read_snapshot already ran `proposals.retain(|p| !decided.contains(&p.item.id));` (the line removed in the diff), and the old test the_page_links_every_record_to_github_at_the_tip_and_lists_only_open_proposals already checked it. Moving the check into render() changes nothing for decided proposals. So whatever made qyd9 (approved) and jtd2 (rejected) show on 3 Nov is untouched. Reading the code suggests a stale page: crates/athena-core/src/work_decisions.rs approve_work/reject_work commit the decision to the records branch at once, but .github/workflows/pages.yml rebuilds decisions.html only on `push: branches: [main]`. The other possibility is that the decision records are missing from the branch. Neither can be checked in this snapshot, which has no records/. The workflow is out of scope, so the root-cause rule required naming the cause and proposing a fix. The CHANGELOG line names no cause, while nearby entries ('Root cause: ...') do. The reading's assumptions also asked the producer for this.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 7 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Make "fails without the fix" a check that Athena runs itself. For a bug-fix job, Athena would run the new or changed tests against the base tree's production code, with the job's test files overlaid, and record the failing output as a named check. It would then run them on the final tree. The verifi...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

26 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in xtask/src/decisions.rs builds a temp git repo where a proposal work item is on the athena/records branch and its Approved (and a second, Rejected) WorkDecision is under records/work-decisions/ on main only, runs locate() and render(), and asserts the proposal's id is absent and no '<article' appears. Run: cargo test -p xtask decisions.
A companion test with an undecided proposal in the same layout asserts it is still listed as one card, so the fix does not hide everything. Run: cargo test -p xtask decisions.
The new decided-on-main test fails on the base code. The verifier checks this by reading read_snapshot in the base (git show of the base file), where decisions come only from the records tip, and by inspecting that the test's decision is absent from that tip. Run: cargo test -p xtask decisions on the candidate.
xtask/src/decisions.rs read_snapshot() reads work decisions from main's tree as well as from the records branch, and the module doc says decisions are read from both places and why.
CHANGELOG.md has one entry that names the root cause (decisions are recorded on main while the page read only athena/records) and says only what the diff does, with no claim about the workflow or other behavior.
cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
