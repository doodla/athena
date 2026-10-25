# Job 20261017-004000-824m: Clarify firstmate study page jargon, diagram, tags and card titles

## Request (verbatim)

> Firstmate study page (rated 4/5): undefined jargon early, ambiguous 'agent', confusing yolo-merge loop and reopened tag, cards headed only by bare ids
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261016-181624-7h2b (key audience:20261016-033753-mhm0), queued at 2026-10-16T18:16:24.223917066Z by the steward role.
> Rung of the requirements ladder: implementation.
> 
> ## Source
> 
> path records/audience-ratings/2026-10/20261016-033753-mhm0.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261016-033753-mhm0",
>   "at": "2026-10-16T03:37:34.272896607Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 4,
>   "findings": [
>     {
>       "text": "The opening paragraphs tell a newcomer what Athena is, what a study is, and why this page exists. They can tell what they are looking at within three sentences.",
>       "source": "Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository."
>     },
>     {
>       "text": "The page ends with clear next actions: links to Athena and to firstmate, and a way to check any statement through its claim id. A newcomer knows what to do next.",
>       "source": "Next steps"
>     },
>     {
>       "text": "'Start here' uses 'captain' and 'yolo' before saying what they are. Both are defined only in the glossary near the bottom, so a newcomer reading top to bottom hits undefined jargon in the first section.",
>       "source": "With the yolo setting off, the captain approves every PR merge and every local-only landing."
>     },
>     {
>       "text": "'Home' (also 'firstmate home', 'this home') is used throughout the page but has no glossary entry. The definitions of secondmate and backlog item rely on it, so they stay unclear.",
>       "source": "A secondmate is a persistent home, not a backlog item."
>     },
>     {
>       "text": "The glossary uses 'agent' both for firstmate's deciding part and for the workers. A newcomer cannot easily tell which agent each card's 'decided by agent' means.",
>       "source": "agent — The part of firstmate that judges what to do ... worker — An agent that carries out one task in its own worktree."
>     },
>     {
>       "text": "The yolo merge is drawn as a loop from ready back to ready, but t4 says a merge leads to done. A newcomer may not see why merging leaves the work's state unchanged.",
>       "source": "t10 ready → ready"
>     },
>     {
>       "text": "The design-choice cards have only bare ids (ra1–ra6) as headings, so a reader cannot skim them to find a topic. A short title on each card would help.",
>       "source": "<h3>ra1</h3>"
>     },
>     {
>       "text": "Reopened is tagged 'terminal-success' even though the item stays open, waiting for the captain. A newcomer may read 'success' as meaning the work is finished.",
>       "source": "<li><strong>reopened</strong> <span class=\"tag\">terminal-success</span>"
>     },
>     {
>       "text": "The lede names an internal study identifier without explaining it. To a newcomer it is noise.",
>       "source": "This page is that study of firstmate, from <code>firstmate-work-handling</code>."
>     }
>   ]
> }
> ```
> 
> ## Evidence
> 
> ### 1. path records/audience-ratings/2026-10/20261016-033753-mhm0.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261016-033753-mhm0",
>   "at": "2026-10-16T03:37:34.272896607Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 4,
>   "findings": [
>     {
>       "text": "The opening paragraphs tell a newcomer what Athena is, what a study is, and why this page exists. They can tell what they are looking at within three sentences.",
>       "source": "Athena is an open-source system, written in Rust, that turns a short request into a change to a code repository."
>     },
>     {
>       "text": "The page ends with clear next actions: links to Athena and to firstmate, and a way to check any statement through its claim id. A newcomer knows what to do next.",
>       "source": "Next steps"
>     },
>     {
>       "text": "'Start here' uses 'captain' and 'yolo' before saying what they are. Both are defined only in the glossary near the bottom, so a newcomer reading top to bottom hits undefined jargon in the first section.",
>       "source": "With the yolo setting off, the captain approves every PR merge and every local-only landing."
>     },
>     {
>       "text": "'Home' (also 'firstmate home', 'this home') is used throughout the page but has no glossary entry. The definitions of secondmate and backlog item rely on it, so they stay unclear.",
>       "source": "A secondmate is a persistent home, not a backlog item."
>     },
>     {
>       "text": "The glossary uses 'agent' both for firstmate's deciding part and for the workers. A newcomer cannot easily tell which agent each card's 'decided by agent' means.",
>       "source": "agent — The part of firstmate that judges what to do ... worker — An agent that carries out one task in its own worktree."
>     },
>     {
>       "text": "The yolo merge is drawn as a loop from ready back to ready, but t4 says a merge leads to done. A newcomer may not see why merging leaves the work's state unchanged.",
>       "source": "t10 ready → ready"
>     },
>     {
>       "text": "The design-choice cards have only bare ids (ra1–ra6) as headings, so a reader cannot skim them to find a topic. A short title on each card would help.",
>       "source": "<h3>ra1</h3>"
>     },
>     {
>       "text": "Reopened is tagged 'terminal-success' even though the item stays open, waiting for the captain. A newcomer may read 'success' as meaning the work is finished.",
>       "source": "<li><strong>reopened</strong> <span class=\"tag\">terminal-success</span>"
>     },
>     {
>       "text": "The lede names an internal study identifier without explaining it. To a newcomer it is noise.",
>       "source": "This page is that study of firstmate, from <code>firstmate-work-handling</code>."
>     }
>   ]
> }
> ```
> 
> ### 2. job 20261016-181716-9pab
> Request:
> ```
> Revise the firstmate study page at https://doodla.github.io/athena/: (1) define 'captain' and 'yolo' where they first appear in 'Start here'; (2) add a glossary entry for 'home' (firstmate home); (3) use two different terms for firstmate's deciding agent and for the worker agents, and say which one 'decided by agent' means; (4) explain why the yolo merge t10 is drawn ready → ready when t4 says a merge leads to done, or fix the diagram; (5) give each design-choice card (ra1–ra6) a short topic title; (6) relabel or explain the 'terminal-success' tag on reopened; (7) explain the study identifier in the lede, or remove it. Keep the opening and the next steps as they are; the rating praised both.
> ```
> Outcome: none recorded yet.

## Reading (accepted by the user)

Address the newcomer's rating findings on the firstmate study page (audience: newcomers reading the published page). Edit the page text in studies/firstmate-work-handling/intro.json and the renderer in xtask/src/site.rs: define terms where first used, add a 'home' entry, split 'agent' into two terms, explain the t10 ready→ready merge, title ra1–ra6, relabel the reopened tag, and explain or remove the study id. Leave the opening and Next steps unchanged.

Assumptions:
- The published page is generated by xtask/src/site.rs from studies/firstmate-work-handling/*.json; the site is rebuilt on deploy, so no generated HTML is edited.
- Rationale records reject unknown fields (deny_unknown_fields), so ra1–ra6 titles are added as a new list in intro.json read by site.rs rather than in rationales.json or the schema.
- model.json state kinds (terminal-success) are validated study data and stay unchanged; the page shows a clearer display label and/or a short explanation for reopened.
- The deciding part of firstmate keeps one name and workers get another (for example 'judgment' vs 'worker'); 'decided by agent' on cards is rendered with the deciding-part term and explained.
- t10's ready→ready arrow is explained in the page text from the cited claims, not by changing model.json, unless the cited claims show the diagram is wrong.
- Claims cited in new or changed sentences must come from existing claim ids; no new claims are invented.
- The study id in the lede is removed or replaced with a plain-language explanation; the pin line stays.

## Acceptance criteria

1. [content] In 'Start here', 'captain' and 'yolo' are each defined in plain words at or before their first use, so a newcomer reading top to bottom meets no undefined term in that section.
2. [content] The glossary has an entry for 'home' (firstmate home) that makes the secondmate and backlog item definitions understandable.
3. [content] The glossary and the cards use two distinct terms for firstmate's deciding part and for the workers, and the page states which one 'decided by' refers to; the word 'agent' is no longer used for both.
4. [content] The t10 card (or the page text near it) explains why a yolo merge is shown as ready → ready while t4 leads to done, or the diagram is corrected consistently with the cited claims.
5. [content] Each of the six design-choice cards (ra1–ra6) has a short topic title in its heading alongside the id.
6. [content] The reopened state no longer displays a bare 'terminal-success' tag that implies finished work; the label or an adjacent note says the item stays open waiting for the captain.
7. [content] The lede no longer shows the raw study identifier without explanation; it is explained or removed. The opening paragraphs and the Next steps section keep their current wording.
8. [behavior] Run `cargo test` (including xtask site tests, updated for any heading or text changes) and `cargo clippy --all-targets` and `cargo fmt --check`: all pass, and the real-study render test still passes with every cited claim id resolving.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/validation.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the studied claims, sources, or the pinned firstmate commit
- Changing the study record schema or validation rules in crates/athena-cap-study
- Rewriting the opening paragraphs or the Next steps section
- Editing records/ including the audience rating
- Changes to site styling beyond what the new titles need

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Page generator changes are in Rust (xtask/src/site.rs); content stays in the existing JSON data.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Update xtask site tests and require cargo test, clippy and fmt --check to pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: New explanatory sentences cite existing claim ids so the page's claim-citation rule still holds.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Wording is written for a newcomer with no knowledge of firstmate.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check what the reader sees on the rendered page, not that fields exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused edit of the page text and renderer; no schema change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Terms are defined at first use and the ambiguous term is split, rather than patching single sentences.
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
