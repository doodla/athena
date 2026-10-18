# Job 20261017-184923-myb9: verify "Fix six open newcomer findings on the firstmate explainer page"

## Request (verbatim)

> Fix the open newcomer findings on the firstmate explainer page (https://doodla.github.io/athena/), which xtask/src/site.rs generates from studies/firstmate-work-handling/. Already fixed on main in bca96fc, not to be redone: the definitions of captain and yolo, the glossary entry for home, the ra1 to ra6 card titles, and the lede's study id. Still open, from requests 20261016-181716-9pab and 20261015-173219-y7tm and audience rating 20261017-095516-gw15:
> 1. Use two different terms for firstmate's deciding agent and for its worker agents, and say which one "decided by agent" means. t4's trigger mixes deciders: it says "the merge lands" and also "(decided by agent)".
> 2. Explain why the yolo merge t10 is drawn ready to ready while t4 takes a merge to done. Change the model only if firstmate's source at the pinned SHA shows the drawing is wrong; otherwise explain it on the card.
> 3. Relabel or explain the "terminal-success" tag on reopened.
> 4. Remove the f2 model-limit note, or rewrite it for a newcomer.
> 5. Define "in scope" (what makes work in scope for an automatic merge) where it first appears, with a glossary entry backed by a sourced claim.
> 6. Point to the next step near the top of the page, not only at the bottom.
> Plan: change content in the study data (intro.json, model.json, claims) and rendering in xtask/src/site.rs only where the change is about rendering. Every new statement about firstmate cites a claim whose source is at the pinned SHA. Add or extend site tests for items 1, 3, 5 and 6.
> Acceptance:
> - cargo test: site tests show distinct decider wording (no transition marked "decided by agent" whose trigger names another decider's event), the reopened tag relabelled or explained, a glossary entry for "in scope", and a link to the next step before the first transition card.
> - The t10 card explains ready to ready with a claim sourced in firstmate at the pinned SHA, or the model changes with that citation.
> - The f2 note is gone or rewritten; no newcomer-facing text mentions a model limit without explaining it.
> - `cargo run -p xtask -- site` builds and validates the study; cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean; CHANGELOG.md has an entry.
> Out of scope: restructuring the front page around Athena (a site-structure design comes with the second study), and new findings about firstmate.

## Reading (accepted by the user)

Fixes the six open findings on the generated firstmate page for newcomers. It adds distinct decider and worker terms, a t10 ready-to-ready explanation, a reopened tag fix, a rewritten f2 note, an "in scope" glossary entry, and an early next-step link. Content changes go in the study JSON; xtask/src/site.rs changes only for rendering, with tests and a CHANGELOG entry.

Assumptions:
- Audience: newcomers to the page at https://doodla.github.io/athena/, and the rater who reads it cold.
- Item 1: 'agent' stays the deciding agent and the worker agents get 'worker'. t4's trigger is reworded so the merge-landing event is decided by code or by a named event, not by the agent.
- Item 3 is read as relabelling reopened's kind or adding an explanation. Which of the two depends on how site.rs renders kind.
- t10 is assumed to be explained, not remodelled, unless the firstmate source at the pinned SHA says otherwise. I could not read that source here.
- The set of governance files (athena.toml, defaults.toml, workflows) does not need to change.

## Acceptance criteria

1. [behavior] Run `cargo test`. Site tests pass and show three things. First, no transition marked 'decided by agent' has a trigger that names another decider's event, so t4's trigger is split or re-attributed. Second, the deciding agent and the worker agents use two different terms, and the page says which one 'decided by agent' means. Third, the page contains a glossary entry for 'in scope'.
2. [behavior] Run `cargo test`. A site test shows the reopened state no longer carries the bare 'terminal-success' tag, or the tag is explained on the page. A second site test shows a link to the next step appears in the HTML before the first transition card.
3. [content] The t10 card explains why the yolo merge is drawn ready to ready while t4 takes a merge to done, citing a claim whose source is firstmate at the pinned SHA. If that source shows the drawing is wrong, model.json changes instead, with the citation.
4. [content] The f2 model-limit note is removed, or rewritten in plain words a newcomer can follow. No newcomer-facing text mentions a model limit without explaining it.
5. [content] 'In scope' is defined where it first appears in intro.json (green terms, the 'what' text and the t10 summary), and a glossary entry for it cites a claim sourced at the pinned SHA. Every new statement about firstmate cites an existing or new claim.
6. [behavior] Run `cargo run -p xtask -- site`. It builds the page and validates the study without error. Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. Both are clean.
7. [content] CHANGELOG.md has an entry for this change. The items already fixed in bca96fc (captain, yolo, home, ra1 to ra6, lede study id) are not reworked.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- xtask/src/site.rs (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/validation.json (modify)

## Out of scope

- Restructuring the front page around Athena
- New findings about firstmate
- Rework of the captain, yolo, home, ra1 to ra6 and lede study id fixes from bca96fc

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the page's wording and links, using site tests and the site build.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Site tests are added for items 1, 3, 5 and 6; cargo test, clippy and fmt are required.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every new firstmate statement cites a claim sourced at the pinned SHA.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered HTML, not just that the data files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The change stays within the study JSON, site.rs and CHANGELOG.md.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria are checkable by cargo test and `cargo run -p xtask -- site`.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The new wording is written for a reader with no prior context.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base d832b1f858ec → candidate 004b935ff5c1. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                  |  12 +++
 studies/firstmate-work-handling/findings.json |  11 +--
 studies/firstmate-work-handling/intro.json    |  34 ++++----
 studies/firstmate-work-handling/model.json    |  12 +--
 xtask/src/site.rs                             | 118 +++++++++++++++++++++++++-
 5 files changed, 151 insertions(+), 36 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (913 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
