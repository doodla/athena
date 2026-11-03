# Job 20261103-201548-qt18: Rewrite decision cards to lead with a yes/no question and outcomes

## Request (verbatim)

> Rewrite each card on the decisions page so the owner can tell what they are deciding in five seconds. The owner's words on 3 Nov: 'It's garbo. I don't have a sense of what I'm deciding.' Each card today leads with a problem statement and file paths. Instead each card leads with: the decision as one yes/no question in plain words (for example 'Add a design step before every job?'); one line on what happens if yes and one on what happens if no; cost and risk in a few words; Athena's recommendation and its one-line reason. Evidence paths and record ids go in a collapsed section below. No jargon (no rung names, role names or record kinds in the visible text). Acceptance: a test that renders a card from a sample proposal and checks the question, yes, no, cost and recommendation lines appear before any path; the audience role rates a sample page against the owner-style rules.
> Depends on 20261103-201526-t4qv

## Reading (accepted by the user)

Rewrite each card on decisions.html (xtask/src/decisions.rs) for the owner, who must tell in five seconds what they are deciding: a plain yes/no question, one line for yes, one for no, cost and risk in a few words, and Athena's recommendation with a one-line reason. Evidence paths, record ids and the record link move into a collapsed details section. Prior job 20261103-201526-t4qv (open-and-above-limit filter) has no candidate and no findings; this change builds on the current page and does not redo that filter.

Assumptions:
- Code survey: I read xtask/src/decisions.rs (entry: write -> render(&locate(root)); locate resolves the records branch tip via athena_git::records_branch::resolve_tip, then read_snapshot lists records/work-items, records/work-decisions and records/roles and parses each blob with athena_data::records::check_record_file; proposals decided by a work decision are dropped; Brief::new builds a card, Brief::check refuses over MAX_WORDS=120 or no 'Recommendation:' line; Brief::html emits the card). I read crates/athena-data/src/records.rs (WorkItem has only title, rung, role, source, evidence, key; validate() requires evidence for proposals) and xtask/src/site.rs (DECISIONS_PAGE, CSS, escape, source_url). The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected and the records validation allow-list needs no change. The property (order of lines in a card) is measured directly on the rendered HTML string of Brief::html/render by finding byte offsets of the question, yes, no, cost and recommendation lines versus the first path/record id, not through a stand-in.
- Job 20261103-201526-t4qv: it was only proposed, not verified, with no candidate files and no findings. Nothing from it is kept or dropped; this job edits the same file as it exists in the snapshot, and the filter behaviour (open, above-limit only; 'Nothing waiting for you') is left to that job and not changed here.
- Audience is the owner reading the published decisions page; the goal is a five-second grasp of the decision. Style rules are docs/conventions/owner-style.md.
- Proposal records carry no yes/no, cost or risk fields, so the card text is derived by code from the title, rung and the raising role's recorded limit, with plain-word templates (no schema change). Cost and risk are short plain phrases chosen from the rung, labelled as Athena's estimate, never invented numbers.
- The 'audience role rates a sample page' criterion is met by a test that renders a sample page and checks the owner-style rules mechanically (word limit, no jargon words), plus the existing owner-style check if one exists; no new role is created.
- Jargon banned in visible text (outside the collapsed details): rung names, role names, record kinds such as work item, proposal record, blind spot, job, commit, path.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a new test renders a card (Brief::html) from a sample WorkItem proposal and asserts the byte offsets of the question (ending in '?'), the 'If yes' line, the 'If no' line, the cost/risk line and the recommendation line are all smaller than the offset of the first evidence path, record id or record link.
2. [behavior] Run `cargo test -p xtask`: a test asserts the evidence list, record id and GitHub record link sit inside a collapsed `<details>` element (no `open` attribute) in the card HTML, and nothing visible outside it contains a path or record id.
3. [behavior] Run `cargo test -p xtask`: a test renders a sample page of several cards and checks owner-style rules mechanically: each card's visible words are within MAX_WORDS, the visible text contains none of the banned jargon (rung names, role names, 'work item', 'proposal', 'blind spot', 'job', 'commit', 'path'), and each card has exactly one question.
4. [behavior] Run `cargo test -p xtask`: the recommendation line states Recommend yes or Recommend no with a one-line reason, and a card without a recommendation is still refused by Brief::check.
5. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`: both are clean; and `cargo test --workspace` passes including the existing xtask site tests that read decisions.html.
6. [content] Grep xtask/src/decisions.rs: the old visible lines 'Rung:', 'Raised by:', 'Source:' no longer appear outside the details section, and the module doc comment describes the new card layout.
7. [content] The diff touches no file under records/, athena.toml, defaults.toml or .github/workflows, and adds no record or data file under crates/.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- xtask/src/site.rs (modify)
- docs/conventions/owner-style.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing which proposals are listed (open, above-limit filter, 'Nothing waiting for you'): that is job 20261103-201526-t4qv.
- Adding fields to WorkItem or any record schema, or any file under crates/.
- athena.toml, defaults.toml, .github/workflows, records/ and prompt files.
- Recording or applying decisions from the page; it stays display-only.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust in the xtask crate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered card order and text, not wiring.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments updated for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file's rendering changed, no schema change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test and clippy and inspects rendered output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the owner can tell the decision from the card top.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Order and jargon rules enforced by tests and Brief::check, not prose.
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

Run `cargo test -p xtask`: a new test renders a card (Brief::html) from a sample WorkItem proposal and asserts the byte offsets of the question (ending in '?'), the 'If yes' line, the 'If no' line, the cost/risk line and the recommendation line are all smaller than the offset of the first evidence path, record id or record link.
Run `cargo test -p xtask`: a test asserts the evidence list, record id and GitHub record link sit inside a collapsed `<details>` element (no `open` attribute) in the card HTML, and nothing visible outside it contains a path or record id.
Run `cargo test -p xtask`: a test renders a sample page of several cards and checks owner-style rules mechanically: each card's visible words are within MAX_WORDS, the visible text contains none of the banned jargon (rung names, role names, 'work item', 'proposal', 'blind spot', 'job', 'commit', 'path'), and each card has exactly one question.
Run `cargo test -p xtask`: the recommendation line states Recommend yes or Recommend no with a one-line reason, and a card without a recommendation is still refused by Brief::check.
Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`: both are clean; and `cargo test --workspace` passes including the existing xtask site tests that read decisions.html.
Grep xtask/src/decisions.rs: the old visible lines 'Rung:', 'Raised by:', 'Source:' no longer appear outside the details section, and the module doc comment describes the new card layout.
The diff touches no file under records/, athena.toml, defaults.toml or .github/workflows, and adds no record or data file under crates/.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
