# Job 20261027-074457-bx8e: verify "Fix leftover firstmate page findings: t3 trigger, spacing, decider test"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 956daf4866e9 → candidate 58921f535b51. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  7 +++++++
 studies/firstmate-work-handling/model.json |  8 ++++----
 xtask/src/site.rs                          | 14 +++++++-------
 3 files changed, 18 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1426 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
