# Job 20261015-085558-xebp: verify "Make the explainer site introduce Athena and firstmate to newcomers"

## Request (verbatim)

> A newcomer rated https://doodla.github.io/athena/ 2/5; address its findings
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261015-064136-gy12 (key audience:20261015-064135-6k80), queued at 2026-10-15T06:41:19.177222836Z by the audience role.
> Rung of the requirements ladder: implementation.
> 
> ## Source
> 
> path records/audience-ratings/2026-10/20261015-064135-6k80.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261015-064135-6k80",
>   "at": "2026-10-15T06:41:19.177222836Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 2,
>   "findings": [
>     {
>       "text": "Someone opening the Athena site expects to learn what Athena is, but the page is entirely about a different project, firstmate. Athena is never named or explained, so the newcomer cannot say what the site is.",
>       "source": "<title>How firstmate handles work</title>; footer: \"Generated from studies/firstmate-work-handling\""
>     },
>     {
>       "text": "The opening line assumes the reader already knows what firstmate is and what a \"study\" is. Neither is introduced, and nothing says who made the study or why.",
>       "source": "\"A reading of firstmate at one pinned commit, from the study firstmate-work-handling.\""
>     },
>     {
>       "text": "This is the closest the page comes to a definition. It doesn't say the workers are AI coding agents or who would use firstmate, so the reader still can't tell what problem it solves or why it matters.",
>       "source": "What firstmate is: \"Firstmate takes an item from a backlog and gives it to a worker, which does the work in its own isolated worktree\""
>     },
>     {
>       "text": "There is no clear next step: the repository URL and every source are plain text, not links, and nothing says how to install, try, or learn more about firstmate or Athena.",
>       "source": "Pinned firstmate commit: \"https://github.com/kunchenguid/firstmate.git\"; claim sources such as \"AGENTS.md:206-206\""
>     },
>     {
>       "text": "The 'Start here' section does well: it explains the page's own layout, walks one task from start to finish in order, and lists the failure cases. A newcomer who already knows the context could follow the task lifecycle.",
>       "source": "Start here"
>     },
>     {
>       "text": "The glossary is near the top and is long and detailed. That makes the page feel like reference material, and some terms are still left vague, such as \"The text says no more about who the captain is.\"",
>       "source": "Glossary (about 30 terms, including \"exit 75\", \"places-held\", \"stale wake\")"
>     },
>     {
>       "text": "This note is about the tool that generated the page and uses its internal field name (guard_evaluated_by). A newcomer cannot make sense of it.",
>       "source": "Limits of this study, f2: \"A guard of none has no value in guard_evaluated_by for nobody. These transitions carry agent there as a placeholder\""
>     },
>     {
>       "text": "Every statement is backed by a claim with file-and-line sources, which is trustworthy, but the citations break up the text. The sources point to lines in the firstmate repository that the reader cannot open from the page.",
>       "source": "Each statement ends with bracketed claim links such as \"[c1] [c22]\""
>     }
>   ]
> }
> ```
> 
> ## Evidence
> 
> ### 1. path records/audience-ratings/2026-10/20261015-064135-6k80.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261015-064135-6k80",
>   "at": "2026-10-15T06:41:19.177222836Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 2,
>   "findings": [
>     {
>       "text": "Someone opening the Athena site expects to learn what Athena is, but the page is entirely about a different project, firstmate. Athena is never named or explained, so the newcomer cannot say what the site is.",
>       "source": "<title>How firstmate handles work</title>; footer: \"Generated from studies/firstmate-work-handling\""
>     },
>     {
>       "text": "The opening line assumes the reader already knows what firstmate is and what a \"study\" is. Neither is introduced, and nothing says who made the study or why.",
>       "source": "\"A reading of firstmate at one pinned commit, from the study firstmate-work-handling.\""
>     },
>     {
>       "text": "This is the closest the page comes to a definition. It doesn't say the workers are AI coding agents or who would use firstmate, so the reader still can't tell what problem it solves or why it matters.",
>       "source": "What firstmate is: \"Firstmate takes an item from a backlog and gives it to a worker, which does the work in its own isolated worktree\""
>     },
>     {
>       "text": "There is no clear next step: the repository URL and every source are plain text, not links, and nothing says how to install, try, or learn more about firstmate or Athena.",
>       "source": "Pinned firstmate commit: \"https://github.com/kunchenguid/firstmate.git\"; claim sources such as \"AGENTS.md:206-206\""
>     },
>     {
>       "text": "The 'Start here' section does well: it explains the page's own layout, walks one task from start to finish in order, and lists the failure cases. A newcomer who already knows the context could follow the task lifecycle.",
>       "source": "Start here"
>     },
>     {
>       "text": "The glossary is near the top and is long and detailed. That makes the page feel like reference material, and some terms are still left vague, such as \"The text says no more about who the captain is.\"",
>       "source": "Glossary (about 30 terms, including \"exit 75\", \"places-held\", \"stale wake\")"
>     },
>     {
>       "text": "This note is about the tool that generated the page and uses its internal field name (guard_evaluated_by). A newcomer cannot make sense of it.",
>       "source": "Limits of this study, f2: \"A guard of none has no value in guard_evaluated_by for nobody. These transitions carry agent there as a placeholder\""
>     },
>     {
>       "text": "Every statement is backed by a claim with file-and-line sources, which is trustworthy, but the citations break up the text. The sources point to lines in the firstmate repository that the reader cannot open from the page.",
>       "source": "Each statement ends with bracketed claim links such as \"[c1] [c22]\""
>     }
>   ]
> }
> ```

## Reading (accepted by the user)

The site is a page about firstmate rendered by xtask/src/site.rs from the study's JSON. I read the rating as asking for: an Athena and study introduction up front, a clearer firstmate definition, links for the repo and sources, the glossary moved lower, and generator jargon removed from the limits section. Readers are newcomers to the site.

Assumptions:
- The audience is a newcomer who opens the site root and knows neither Athena nor firstmate.
- Athena's repository link is derivable from the repo, README or Cargo metadata; the producer will choose a stable link.
- Claim citations stay, since the site's design requires them, but they are made less intrusive and sources become links.
- No workflow change is needed because pages.yml already builds the site from xtask.

## Acceptance criteria

1. [content] The rendered page's title, heading and opening paragraph name Athena, say what Athena is, and say who made the study and why, before firstmate is discussed.
2. [content] The page says what firstmate is in plain words, including that its workers are AI coding agents, who would use it, and what problem it solves. The page also explains what a study is.
3. [content] The pinned repository URL is an HTML link, and each claim source is a link to the pinned commit's file and lines when the repository is on GitHub. The link text is HTML-escaped.
4. [content] The glossary appears after the walk-through and states, not before. The vague captain entry is clarified or removed.
5. [content] The Limits section does not show internal field names such as guard_evaluated_by, and the guard-none note reads in plain language.
6. [content] The page includes a next-steps section that links to Athena and to firstmate.
7. [behavior] Run cargo test --workspace and cargo clippy --workspace --all-targets. Both pass, including updated xtask site tests that check the new order and links.
8. [behavior] Run cargo run -p xtask -- site. It still succeeds, and every element still cites an existing claim.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Changing the firstmate study's claims or re-running the study
- Changing the audience rating mechanism or records/
- Changing .github/workflows/pages.yml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The xtask site tests are updated and cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The generator and intro data are fixed, not the published HTML.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the site generator and its intro data.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The page copy is written for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No personal data goes into the page or links.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base f6f2732cab5a → candidate 4510f0ad5d70. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                  |   8 +
 README.md                                     |   2 +
 docs/design/explainer-site.md                 |  15 +-
 studies/firstmate-work-handling/findings.json |   2 +-
 studies/firstmate-work-handling/intro.json    |   7 +-
 xtask/src/site.rs                             | 220 ++++++++++++++++++++------
 6 files changed, 203 insertions(+), 51 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (680 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
