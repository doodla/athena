# Job 20261103-201526-t4qv: Show only open, above-limit proposals on the decisions page

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

A new test in xtask/src/decisions.rs builds a Snapshot with three proposals (one with an Approved WorkDecision, one with a Rejected WorkDecision, one raised by a role whose limit covers its rung) and asserts render() contains 'Nothing waiting for you', contains no '<article', and has no 'N waiting' heading. Run: cargo test -p xtask decisions.
A new test builds a records branch (or Snapshot) with one open proposal above the raising role's limit and asserts render() shows exactly one '<article' card and the heading '1 waiting'. Run: cargo test -p xtask decisions.
A test through the records branch path (locate() on a temp git repo with RecordsWriter) covers an approved decision whose job is set, confirming the approved proposal is not listed. Run: cargo test -p xtask decisions.
The empty state is a single line reading 'Nothing waiting for you' in xtask/src/decisions.rs render(), replacing the old 'No proposals waiting' section; the missing-records-branch message stays distinct.
Existing tests in xtask/src/decisions.rs that conflicted with the new rule are updated, and the module doc comment states that only open, above-limit proposals are listed.
cargo test and cargo clippy --all-targets pass and cargo fmt --check is clean.
CHANGELOG.md has one line describing the decisions page fix.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
