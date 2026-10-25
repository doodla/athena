# Job 20261017-091546-05yx: Fix explainer glossary, card titles, lede, and claim source issues

## Request (verbatim)

> Newcomer rated the explainer 4/5: green/red and 'home' are never defined, the glossary is unordered, ra1–ra6 card titles are opaque, and some claims list the same source twice
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261017-083619-nqc2 (key audience:20261017-041207-se1q), queued at 2026-10-17T08:36:19.976284874Z by the steward role.
> Rung of the requirements ladder: implementation.
> 
> ## Source
> 
> path records/audience-ratings/2026-10/20261017-041207-se1q.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261017-041207-se1q",
>   "at": "2026-10-17T04:11:42.916739704Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 4,
>   "findings": [
>     {
>       "text": "The opening paragraphs tell a newcomer what Athena is, what a study is, and why this one exists: to show that every statement traces back to source lines. That quickly answers both \"what is this\" and \"why does it matter\".",
>       "source": "Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository. ... A study is Athena's written reading of another software project"
>     },
>     {
>       "text": "\"Start here\" uses captain, yolo, green/red, and local-only landing before explaining any of them. Captain and yolo are only defined in the Glossary further down, and green/red are never defined anywhere, so a newcomer reading top to bottom hits terms they don't know.",
>       "source": "With the yolo setting off, the captain approves every PR merge and every local-only landing. With yolo on, firstmate merges green, in-scope work itself, and never a red one."
>     },
>     {
>       "text": "\"Home\" is used here and in other places (\"this home has no item\", \"a home's backlog\") but has no Glossary entry. That makes it hard to understand what a secondmate is.",
>       "source": "A secondmate is a persistent home, not a backlog item."
>     },
>     {
>       "text": "The study's internal id appears in the lede with no explanation. To a newcomer it is noise.",
>       "source": "This page is that study of firstmate, from <code>firstmate-work-handling</code>."
>     },
>     {
>       "text": "The design-choice cards are titled only with opaque ids (ra1–ra6). A short descriptive title would let a reader skim for the choices they care about. The transition cards (t1–t11) at least show from-state → to-state.",
>       "source": "Heading \"Why it is built this way\": cards titled ra1 … ra6"
>     },
>     {
>       "text": "The Glossary is not in alphabetical order, so it is slow to look up a term met earlier on the page.",
>       "source": "Heading \"Glossary\": first entries are \"code\", \"queued\", \"in-flight\" … last entries \"stale wake\", \"relaunch\""
>     },
>     {
>       "text": "The page ends with clear next actions for both projects and explains how to check a claim. This answers \"what do I do next\".",
>       "source": "Heading \"Next steps\": \"Learn about Athena, and how to install and try it\" / \"Learn about firstmate ... and see how to use it\""
>     },
>     {
>       "text": "The page says plainly that nothing was run and every claim comes from reading the code. That honesty helps a newcomer judge how far to trust it.",
>       "source": "Heading \"Limits of this study\": \"f1 could-not-run This study read the pinned tree only ... no failure or retry edge was observed\""
>     },
>     {
>       "text": "Several claims list the same source link twice, and c34 appears before c33. These are small flaws but they weaken the page's own message about careful tracing.",
>       "source": "Heading \"Claims and sources\": claim c2 lists docs/configuration.md:357-357 twice; c14 and c42 likewise repeat a source"
>     }
>   ]
> }
> ```
> 
> ## Evidence
> 
> ### 1. path records/audience-ratings/2026-10/20261017-041207-se1q.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261017-041207-se1q",
>   "at": "2026-10-17T04:11:42.916739704Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 4,
>   "findings": [
>     {
>       "text": "The opening paragraphs tell a newcomer what Athena is, what a study is, and why this one exists: to show that every statement traces back to source lines. That quickly answers both \"what is this\" and \"why does it matter\".",
>       "source": "Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository. ... A study is Athena's written reading of another software project"
>     },
>     {
>       "text": "\"Start here\" uses captain, yolo, green/red, and local-only landing before explaining any of them. Captain and yolo are only defined in the Glossary further down, and green/red are never defined anywhere, so a newcomer reading top to bottom hits terms they don't know.",
>       "source": "With the yolo setting off, the captain approves every PR merge and every local-only landing. With yolo on, firstmate merges green, in-scope work itself, and never a red one."
>     },
>     {
>       "text": "\"Home\" is used here and in other places (\"this home has no item\", \"a home's backlog\") but has no Glossary entry. That makes it hard to understand what a secondmate is.",
>       "source": "A secondmate is a persistent home, not a backlog item."
>     },
>     {
>       "text": "The study's internal id appears in the lede with no explanation. To a newcomer it is noise.",
>       "source": "This page is that study of firstmate, from <code>firstmate-work-handling</code>."
>     },
>     {
>       "text": "The design-choice cards are titled only with opaque ids (ra1–ra6). A short descriptive title would let a reader skim for the choices they care about. The transition cards (t1–t11) at least show from-state → to-state.",
>       "source": "Heading \"Why it is built this way\": cards titled ra1 … ra6"
>     },
>     {
>       "text": "The Glossary is not in alphabetical order, so it is slow to look up a term met earlier on the page.",
>       "source": "Heading \"Glossary\": first entries are \"code\", \"queued\", \"in-flight\" … last entries \"stale wake\", \"relaunch\""
>     },
>     {
>       "text": "The page ends with clear next actions for both projects and explains how to check a claim. This answers \"what do I do next\".",
>       "source": "Heading \"Next steps\": \"Learn about Athena, and how to install and try it\" / \"Learn about firstmate ... and see how to use it\""
>     },
>     {
>       "text": "The page says plainly that nothing was run and every claim comes from reading the code. That honesty helps a newcomer judge how far to trust it.",
>       "source": "Heading \"Limits of this study\": \"f1 could-not-run This study read the pinned tree only ... no failure or retry edge was observed\""
>     },
>     {
>       "text": "Several claims list the same source link twice, and c34 appears before c33. These are small flaws but they weaken the page's own message about careful tracing.",
>       "source": "Heading \"Claims and sources\": claim c2 lists docs/configuration.md:357-357 twice; c14 and c42 likewise repeat a source"
>     }
>   ]
> }
> ```
> 
> ### 2. job 20261017-083729-y4p6
> Request:
> ```
> Fix the explainer page generator and data for these issues. Define captain, yolo, green/red and local-only landing where 'Start here' first uses them, or link each to the Glossary. Add Glossary entries for green, red and home. Sort the Glossary alphabetically. Give each design-choice card (ra1–ra6) a short descriptive title. Explain or drop the study's internal id in the lede. Remove duplicate source links within a claim and list claims in numeric order. Add generator tests that check the Glossary is sorted, that no claim repeats a source, and that claims are ordered.
> ```
> Outcome: none recorded yet.

## Reading (accepted by the user)

Respond to the newcomer's 4/5 rating by changing the explainer generator (xtask/src/site.rs) and the firstmate-work-handling study data. Define or link the Start-here terms, add and sort Glossary entries, title the ra1–ra6 cards, explain the lede id, dedupe and order claim sources, and add generator tests for each.

Assumptions:
- Audience is a newcomer reading the published page top to bottom; the goal is that terms are defined before use.
- Page is generated by xtask/src/site.rs from studies/firstmate-work-handling/*.json; fixes go in the generator where they are systematic (sort, dedupe, order) and in the data where content is needed (glossary entries, card titles, intro).
- If study JSON is hashed or validated by the study capability, edits must keep validation.json consistent.
- Card titles may be added as a new field in rationales.json or a generator-side map; the producer picks whichever the schema allows.

## Acceptance criteria

1. [behavior] Run cargo test: the new generator tests pass, covering Glossary sorted alphabetically, no claim listing the same source twice, and claims in numeric order (c33 before c34).
2. [behavior] Run cargo clippy and cargo fmt --check: both clean.
3. [content] In the generated page, captain, yolo, green, red and local-only landing are defined at first use in 'Start here' or link to Glossary entries; the Glossary has entries for green, red and home.
4. [content] Glossary entries render in alphabetical order, and claims render in numeric order with no claim showing a repeated source (c2, c14, c42 fixed).
5. [content] Each design-choice card ra1–ra6 shows a short descriptive title alongside its id.
6. [content] The lede either explains the internal id firstmate-work-handling or no longer shows it unexplained.
7. [behavior] Run cargo run for the site generator xtask (as documented) and confirm the output page contains the new titles and definitions; the generator must still validate the study.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/rationales.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the study's claims or conclusions
- Changes to other studies or the audience-rating records
- Redesigning the page layout beyond the listed fixes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add generator tests for sorted glossary, no repeated claim source, ordered claims; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered page output, not just that data exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Sorting, deduping and ordering are done in the generator so they hold for any study, not only the instances found.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Definitions are written plainly for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the generator and study data.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

## Report

End with the producer report as your structured output.
