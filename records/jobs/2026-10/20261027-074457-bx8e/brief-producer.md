# Job 20261027-074457-bx8e: Fix leftover firstmate page findings: t3 trigger, spacing, decider test

## Request (verbatim)

> Audience rating 4/5: the Athena site is mostly about firstmate, uses 'in scope' without defining it, and puts the call to action only at the bottom
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261018-000648-ne91 (key audience:20261017-095516-gw15), queued at 2026-10-18T00:06:48.839926844Z by the steward role.
> Rung of the requirements ladder: implementation.
> 
> ## Source
> 
> path records/audience-ratings/2026-10/20261017-095516-gw15.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261017-095516-gw15",
>   "at": "2026-10-17T09:54:55.553975836Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 4,
>   "findings": [
>     {
>       "text": "The opening three paragraphs tell a newcomer what Athena is, what a study is, and that this page is a study of firstmate. A first-time visitor can say what they are looking at within a few lines.",
>       "source": "<p class=\"lede\">Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository."
>     },
>     {
>       "text": "Someone who lands on the Athena site expects to learn about Athena. Most of the page is about a different project, firstmate. Athena gets two sentences and is then only a demonstration, so a reader who came for Athena learns little about using it.",
>       "source": "Title and h1: \"Athena explains how firstmate handles work\""
>     },
>     {
>       "text": "There is a clear next step, but it sits near the bottom, after the long transition cards, glossary and limits. Nothing near the top points to it, so a newcomer who stops reading early leaves without a call to action.",
>       "source": "Next steps: \"Learn about Athena, and how to install and try it: https://github.com/doodla/athena\""
>     },
>     {
>       "text": "The \"Start here\" section is a good on-ramp. It defines terms before using them and follows one task from start to finish in plain steps.",
>       "source": "Start here: \"The three parts below say what firstmate is, follow one piece of work, and say what happens when something fails or has no limit.\""
>     },
>     {
>       "text": "The page says plainly what firstmate is and why someone would use it: parallel agents that don't overwrite each other's files and don't need a person to start each one.",
>       "source": "What firstmate is: \"Firstmate is a tool for people who direct AI coding agents.\""
>     },
>     {
>       "text": "\"In scope\" is used many times but never defined, not even in the Glossary. A newcomer can't tell what makes work in scope for an automatic merge.",
>       "source": "yolo: \"on, firstmate merges green, in-scope work itself\""
>     },
>     {
>       "text": "A merge is shown as a transition that stays in the Ready state, while t4 handles reaching Done. A newcomer may find it confusing that merging doesn't change the state.",
>       "source": "t10 ready → ready: \"With yolo on, firstmate merges green, in-scope work itself\""
>     },
>     {
>       "text": "This sentence is awkward and hard to follow. Some trigger lines also mix deciders, e.g. t4's trigger includes \"the merge lands\" yet says \"(decided by agent)\".",
>       "source": "Transitions lead-in: \"The two deciders are explained below the cards' heading.\""
>     },
>     {
>       "text": "The limits section is honest that nothing was run. However, f2 and f3 explain the study's internal data model, which means little to a newcomer.",
>       "source": "Limits of this study, f2: \"The study's data format asks every guard to name who checks it, and it has no way to say nobody does\""
>     },
>     {
>       "text": "Each statement links to its claim, and each claim to source lines at the pinned commit, which backs up the page's promise of traceability. On some list items the citation runs are long enough to make reading harder.",
>       "source": "Each statement ends with bracketed ids, e.g. \"[c1] [c11] [c21] [c22]\""
>     }
>   ]
> }
> ```
> 
> ## Evidence
> 
> ### 1. path records/audience-ratings/2026-10/20261017-095516-gw15.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261017-095516-gw15",
>   "at": "2026-10-17T09:54:55.553975836Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 4,
>   "findings": [
>     {
>       "text": "The opening three paragraphs tell a newcomer what Athena is, what a study is, and that this page is a study of firstmate. A first-time visitor can say what they are looking at within a few lines.",
>       "source": "<p class=\"lede\">Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository."
>     },
>     {
>       "text": "Someone who lands on the Athena site expects to learn about Athena. Most of the page is about a different project, firstmate. Athena gets two sentences and is then only a demonstration, so a reader who came for Athena learns little about using it.",
>       "source": "Title and h1: \"Athena explains how firstmate handles work\""
>     },
>     {
>       "text": "There is a clear next step, but it sits near the bottom, after the long transition cards, glossary and limits. Nothing near the top points to it, so a newcomer who stops reading early leaves without a call to action.",
>       "source": "Next steps: \"Learn about Athena, and how to install and try it: https://github.com/doodla/athena\""
>     },
>     {
>       "text": "The \"Start here\" section is a good on-ramp. It defines terms before using them and follows one task from start to finish in plain steps.",
>       "source": "Start here: \"The three parts below say what firstmate is, follow one piece of work, and say what happens when something fails or has no limit.\""
>     },
>     {
>       "text": "The page says plainly what firstmate is and why someone would use it: parallel agents that don't overwrite each other's files and don't need a person to start each one.",
>       "source": "What firstmate is: \"Firstmate is a tool for people who direct AI coding agents.\""
>     },
>     {
>       "text": "\"In scope\" is used many times but never defined, not even in the Glossary. A newcomer can't tell what makes work in scope for an automatic merge.",
>       "source": "yolo: \"on, firstmate merges green, in-scope work itself\""
>     },
>     {
>       "text": "A merge is shown as a transition that stays in the Ready state, while t4 handles reaching Done. A newcomer may find it confusing that merging doesn't change the state.",
>       "source": "t10 ready → ready: \"With yolo on, firstmate merges green, in-scope work itself\""
>     },
>     {
>       "text": "This sentence is awkward and hard to follow. Some trigger lines also mix deciders, e.g. t4's trigger includes \"the merge lands\" yet says \"(decided by agent)\".",
>       "source": "Transitions lead-in: \"The two deciders are explained below the cards' heading.\""
>     },
>     {
>       "text": "The limits section is honest that nothing was run. However, f2 and f3 explain the study's internal data model, which means little to a newcomer.",
>       "source": "Limits of this study, f2: \"The study's data format asks every guard to name who checks it, and it has no way to say nobody does\""
>     },
>     {
>       "text": "Each statement links to its claim, and each claim to source lines at the pinned commit, which backs up the page's promise of traceability. On some list items the citation runs are long enough to make reading harder.",
>       "source": "Each statement ends with bracketed ids, e.g. \"[c1] [c11] [c21] [c22]\""
>     }
>   ]
> }
> ```
> 
> ### 2. job 20261018-000747-w4v7
> Request:
> ```
> On the published study page: (1) define 'in scope' (for an automatic merge) in the text and in the Glossary; (2) add a link near the top to the Athena repository as the next step; (3) rewrite the transitions lead-in sentence, and make each trigger name one decider only (for example t4); (4) explain why a merge (t10) leaves the state at Ready; (5) rewrite limits f2 and f3 in plain words a newcomer can follow. How much of the page should be about Athena itself rather than the studied system is a design question, so it stays a proposal and is not part of this request.
> ```
> Outcome: dropped. closed: Done: its four items (define 'in scope' with a glossary entry, a next-step link near the top, one decider per trigger, and the reopened tag) landed as job 20261017-184923-myb9 (9884df5); the rating it came from predates that landing.

## Reading (accepted by the user)

Most items in the audience rating already landed (9884df5 and earlier); the f2/f3 model-limit notes are gone from intro.json. This change closes the verifier's leftovers from job 20261017-184923-myb9 so the newcomer-facing page makes only cited statements: the unsupported t3 trigger wording, JSON spacing, and a weak decider test.

Assumptions:
- Goal: a newcomer reading the published firstmate explainer sees only claim-backed statements, with one decider per trigger.
- Items 1-5 of the dropped request are already on main; only the three verifier findings remain.
- The model.json t3 trigger currently reads 'the agent accepts the worker's report...' with no supporting claim among c10, c11, c32, c37.
- The page should not be restructured around Athena; that stays a separate design proposal.

## Acceptance criteria

1. [content] model.json t3 trigger states only what a cited claim supports, and does not say the agent accepts the worker's report unless a cited claim supports that.
2. [content] Edited model.json transitions (t3, t8, t10, t11) have a space after the comma before "trigger_evaluated_by", matching the rest of the file.
3. [behavior] Run cargo test: the site test for decider wording checks structurally, rather than by a keyword ban, that no transition with trigger_evaluated_by 'agent' has a trigger describing a non-agent event, and it passes.
4. [behavior] Run cargo run -p xtask -- site: the study builds and validates, and the generated page still has the in-scope glossary entry, a next-step link before the first transition card, and no f2/f3 model-limit note.
5. [behavior] cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
6. [content] CHANGELOG.md has an entry for the change.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/claims.json (modify)

## Out of scope

- Restructuring the front page around Athena
- New findings about firstmate
- Re-doing items already landed in 9884df5 and bca96fc

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: criteria check the generated page and tests
- small: Prefer one focused change; size is a design choice, not a wish list. Here: only the three leftover findings
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: new firstmate statements must cite a claim
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: test made structural rather than keyword-based
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 30 cases in 30 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the verifier is dispatched, a code check confirms that the producer's report exists, is non-empty, and is included in the verifier's input. If any acceptance criterion refers to the report's content, the job is held at handover when the report is absent, and is not sent to the verifier to be ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

16 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

model.json t3 trigger states only what a cited claim supports, and does not say the agent accepts the worker's report unless a cited claim supports that.
Edited model.json transitions (t3, t8, t10, t11) have a space after the comma before "trigger_evaluated_by", matching the rest of the file.
Run cargo test: the site test for decider wording checks structurally, rather than by a keyword ban, that no transition with trigger_evaluated_by 'agent' has a trigger describing a non-agent event, and it passes.
Run cargo run -p xtask -- site: the study builds and validates, and the generated page still has the in-scope glossary entry, a next-step link before the first transition card, and no f2/f3 model-limit note.
cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
CHANGELOG.md has an entry for the change.

## Report

End with the producer report as your structured output.
