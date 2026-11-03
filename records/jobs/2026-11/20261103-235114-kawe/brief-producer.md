# Job 20261103-235114-kawe: Rewrite decision cards with per-proposal question and outcomes

## Request (verbatim)

> try again on 20261103-201548-qt18

## Reading (accepted by the user)

Retry of rejected job 20261103-201548-qt18: rework each card on decisions.html (xtask/src/decisions.rs) so the owner sees a plain yes/no question, an "If yes" and "If no" line that differ per proposal, a short cost/risk line, and a recommendation only when a record states one; evidence moves into a collapsed details block. Over-limit proposals say "you must decide" rather than "Recommend no".

Assumptions:
- Code survey: read crates/athena-data/src/records.rs (WorkItem: role, kind, key, title, rung, source, evidence; validate()), crates/athena-core/src/steward.rs (ProposedChange, title/request text and tests near line 1504) and xtask/src/decisions.rs (Brief::new, Brief::check, MAX_WORDS, render, locate). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. Behavior is measured by rendering Brief::html directly in xtask tests.
- Job 20261103-201548-qt18 addressed: its candidate changed CHANGELOG.md, docs/conventions/owner-style.md, xtask/src/decisions.rs, xtask/src/site.rs. All four are kept as deliverables and reworked from the base (the candidate was rejected and discarded). The collapsed details layout, word cap and recommendation-line refusal idea are reused; each of its findings is fixed: prefix-only question, identical yes/no lines, invented 'Recommend no', unchecked jargon in real titles, the 'Recommend yes' wording, and the reading misread gap.
- WorkItem has no recorded change, cost, risk or recommendation field, and adding fields is out of scope. So per-card text is built from recorded fields that do vary: title, rung, raising role's recorded limit, source kind and evidence count and kinds. The question is rewritten around the proposal's title content by rung-specific wording, not a fixed prefix on the title. If a title cannot be phrased as a decision, the card says so plainly rather than faking it.
- No field records a recommendation, so no card shows one. An over-limit or role-unknown proposal states 'You must decide'; a proposal within the role's limit says Athena may change it itself. Brief::check no longer demands a recommendation line but requires a decision-owner line.
- Brief::check also enforces the jargon ban (rung names, role names, 'job', 'work item', 'proposal', 'blind spot', 'commit', 'path') on visible text, so real titles that contain jargon are refused or reworded, not only fixtures.
- Audience: the owner, who said on 3 Nov the cards were 'garbo'. Filtering which proposals show stays with job 20261103-201526-t4qv.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test builds cards from at least two realistic Steward-shaped WorkItems (different title, rung, source, evidence count) and asserts the question lines differ, the 'If yes' lines differ, the 'If no' lines differ, each contains a fact from its own proposal, and no question equals the title with a fixed prefix.
2. [behavior] Run `cargo test -p xtask`: a test asserts a proposal above the raising role's limit renders 'you must decide' and the text 'Recommend no' appears on no card; a proposal with no recorded recommendation renders none.
3. [behavior] Run `cargo test -p xtask`: a test asserts the byte offsets of the question (ending '?'), 'If yes', 'If no' and cost/risk lines are all smaller than the first evidence path, record id or record link, and that these sit inside a `<details>` element without `open`.
4. [behavior] Run `cargo test -p xtask`: a test renders a sample page and checks each card's visible words are within MAX_WORDS, it has exactly one question, and Brief::check refuses a card whose visible text contains banned jargon, including from a real-looking title.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`: all pass, including existing xtask site tests that read decisions.html.
6. [content] Grep xtask/src/decisions.rs: 'Rung:', 'Raised by:', 'Source:' appear only inside the details section, and the module doc describes the new layout; CHANGELOG.md has an entry and docs/conventions/owner-style.md matches the behavior.
7. [content] The diff touches no file under records/, crates/, athena.toml, defaults.toml or .github/workflows.

## Expected deliverables

- xtask/src/decisions.rs (modify)
- xtask/src/site.rs (modify)
- docs/conventions/owner-style.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing which proposals are listed (job 20261103-201526-t4qv).
- Adding fields to WorkItem or any record schema, or any file under crates/.
- athena.toml, defaults.toml, .github/workflows, records/ and prompt files.
- Recording or applying decisions from the page; it stays display-only.

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert that text differs per proposal and carries its facts, not just that lines exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: xtask tests plus fmt, clippy and workspace tests.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Jargon ban and decision-owner line enforced in Brief::check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the invented recommendation and prefix-only question causes, not the one instance.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog updated for outside readers.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks target the owner reading the card, not the code wiring.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261103-201548-qt18 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (xtask/src/decisions.rs:138): The 'question' is the old title with 'Go ahead with this:' in front, so the card still leads with the same text the owner called unclear `question: format!("Go ahead with this: {}?", phrase(&item.title))`. Before this change the card's h3 was item.title, which is the problem statement the owner complained about. A title such as 'Verifier misses broken links' becomes 'Go ahead with this: Verifier misses broken links?', which is not a yes/no decision in plain words. Better: make the proposal state its decision (a question field, or a check that refuses a title that is not a decision), or at least check the title and say plainly when it cannot be phrased as a question.
> major (xtask/src/decisions.rs:139-140): The 'If yes' and 'If no' lines are the same fixed text on every card, so they say nothing about the decision yes_line is always 'If yes: Athena makes this change and works with it from then on.' and no_line is always 'If no: nothing changes. Athena keeps working as it does today.' They use nothing from the proposal: not its title, rung or evidence. The request asks for one line on what happens if yes and one on what happens if no for each decision. Identical lines leave the owner where they started. The reading's assumption says the lines are derived from the title and rung; these are not.
> major (xtask/src/decisions.rs:86-96): 'Recommend no' is given for every proposal above the raising role's limit, which turns 'you must decide this' into invented advice against it `Some(_) => "Recommend no: it is bigger than Athena may change on its own, so it needs your care."`, and None (role not recorded) is also 'Recommend no'. The old text said 'Decide yourself'. That a change is above the role's limit says who must decide, not whether the change is good. The reason given ('needs your care') does not support 'no'. Once the filter in job 20261103-201526-t4qv lands, the page shows only above-limit proposals, so every card would say 'Recommend no' with the same reason. This breaks the owner-style rule 'nothing invented'. Better: recommend from something in the record, or state plainly that Athena makes no recommendation on the merits.
> major: The reading does not match the request, so passing criteria do not show the request was met The reading says the yes/no, cost and risk text is 'derived by code from the title, rung and the raising role's recorded limit, with plain-word templates'. The candidate meets the letter of that but not the request. The 'If yes' and 'If no' lines are fixed text that is the same on every card. The question is the old title with a prefix. The request's goal, that the owner can tell what they are deciding in five seconds, depends on per-decision content that the records do not hold. The reading should have raised that gap, for example as a needed question or outcome field, or as a check on titles, instead of assuming templates would be enough.

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

Run `cargo test -p xtask`: a test builds cards from at least two realistic Steward-shaped WorkItems (different title, rung, source, evidence count) and asserts the question lines differ, the 'If yes' lines differ, the 'If no' lines differ, each contains a fact from its own proposal, and no question equals the title with a fixed prefix.
Run `cargo test -p xtask`: a test asserts a proposal above the raising role's limit renders 'you must decide' and the text 'Recommend no' appears on no card; a proposal with no recorded recommendation renders none.
Run `cargo test -p xtask`: a test asserts the byte offsets of the question (ending '?'), 'If yes', 'If no' and cost/risk lines are all smaller than the first evidence path, record id or record link, and that these sit inside a `<details>` element without `open`.
Run `cargo test -p xtask`: a test renders a sample page and checks each card's visible words are within MAX_WORDS, it has exactly one question, and Brief::check refuses a card whose visible text contains banned jargon, including from a real-looking title.
Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`: all pass, including existing xtask site tests that read decisions.html.
Grep xtask/src/decisions.rs: 'Rung:', 'Raised by:', 'Source:' appear only inside the details section, and the module doc describes the new layout; CHANGELOG.md has an entry and docs/conventions/owner-style.md matches the behavior.
The diff touches no file under records/, crates/, athena.toml, defaults.toml or .github/workflows.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
