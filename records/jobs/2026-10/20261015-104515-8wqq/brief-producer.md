# Job 20261015-104515-8wqq: Make the explainer site introduce Athena and fix newcomer findings

## Request (verbatim)

> A newcomer rated https://doodla.github.io/athena/ 2/5; address its findings
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261015-085449-1n1k (key audience:20261015-085448-tj9k), queued at 2026-10-15T08:54:34.756168929Z by the audience role.
> Rung of the requirements ladder: implementation.
> 
> ## Source
> 
> path records/audience-ratings/2026-10/20261015-085448-tj9k.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261015-085448-tj9k",
>   "at": "2026-10-15T08:54:34.756168929Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 2,
>   "findings": [
>     {
>       "text": "The site's address is doodla.github.io/athena/, but I read the whole page and it never names Athena. A newcomer who came to learn what Athena is leaves without an answer, and nothing explains why a page about a different project is here.",
>       "source": "<title>How firstmate handles work</title>; the first heading is \"How firstmate handles work\""
>     },
>     {
>       "text": "The page assumes the reader already knows what \"the study\" is and who made it. It never says that the study is the product being shown, or that this page is an example of what that product makes.",
>       "source": "\"A reading of firstmate at one pinned commit, from the study firstmate-work-handling.\""
>     },
>     {
>       "text": "This explains how firstmate works but not what it is for. It doesn't say who uses it or that the workers are AI coding agents. A reader has to piece that together from terms like \"harness\" and \"captain\".",
>       "source": "Start here > What firstmate is: \"Firstmate takes an item from a backlog and gives it to a worker, which does the work in its own isolated worktree\""
>     },
>     {
>       "text": "The person who approves merges is never explained. The page admits this gap instead of filling it, so a newcomer can't picture who is actually involved.",
>       "source": "Glossary > captain: \"The text says no more about who the captain is.\""
>     },
>     {
>       "text": "The numbered walk-through of one task is clear and concrete. It is the strongest part of the page for a reader who already cares about firstmate.",
>       "source": "Start here > One piece of work, start to finish"
>     },
>     {
>       "text": "Internal schema terms leak into the page here, and the passage is hard for an outsider to read.",
>       "source": "Limits of this study > f2: \"A guard of none has no value in guard_evaluated_by for nobody.\""
>     },
>     {
>       "text": "There is no next step anywhere on the page. It gives no link to Athena, no way to install or run anything, and no link to firstmate except a bare git URL in the header. The claim sources are file:line strings, not links to those files.",
>       "source": "<footer>Generated from studies/firstmate-work-handling: 22 claims at commit ...</footer>"
>     }
>   ]
> }
> ```
> 
> ## Evidence
> 
> ### 1. path records/audience-ratings/2026-10/20261015-085448-tj9k.json
> ```
> {
>   "schema": "athena.audience-rating/1",
>   "id": "20261015-085448-tj9k",
>   "at": "2026-10-15T08:54:34.756168929Z",
>   "target": "https://doodla.github.io/athena/",
>   "audience": "newcomer",
>   "rating": 2,
>   "findings": [
>     {
>       "text": "The site's address is doodla.github.io/athena/, but I read the whole page and it never names Athena. A newcomer who came to learn what Athena is leaves without an answer, and nothing explains why a page about a different project is here.",
>       "source": "<title>How firstmate handles work</title>; the first heading is \"How firstmate handles work\""
>     },
>     {
>       "text": "The page assumes the reader already knows what \"the study\" is and who made it. It never says that the study is the product being shown, or that this page is an example of what that product makes.",
>       "source": "\"A reading of firstmate at one pinned commit, from the study firstmate-work-handling.\""
>     },
>     {
>       "text": "This explains how firstmate works but not what it is for. It doesn't say who uses it or that the workers are AI coding agents. A reader has to piece that together from terms like \"harness\" and \"captain\".",
>       "source": "Start here > What firstmate is: \"Firstmate takes an item from a backlog and gives it to a worker, which does the work in its own isolated worktree\""
>     },
>     {
>       "text": "The person who approves merges is never explained. The page admits this gap instead of filling it, so a newcomer can't picture who is actually involved.",
>       "source": "Glossary > captain: \"The text says no more about who the captain is.\""
>     },
>     {
>       "text": "The numbered walk-through of one task is clear and concrete. It is the strongest part of the page for a reader who already cares about firstmate.",
>       "source": "Start here > One piece of work, start to finish"
>     },
>     {
>       "text": "Internal schema terms leak into the page here, and the passage is hard for an outsider to read.",
>       "source": "Limits of this study > f2: \"A guard of none has no value in guard_evaluated_by for nobody.\""
>     },
>     {
>       "text": "There is no next step anywhere on the page. It gives no link to Athena, no way to install or run anything, and no link to firstmate except a bare git URL in the header. The claim sources are file:line strings, not links to those files.",
>       "source": "<footer>Generated from studies/firstmate-work-handling: 22 claims at commit ...</footer>"
>     }
>   ]
> }
> ```

## Reading (accepted by the user)

Reworks the generated explainer page (xtask/src/site.rs plus the study's intro and findings text) so a newcomer first learns what Athena is and that this page is an example study of firstmate. It also says who firstmate is for, explains the captain, rewrites the f2 finding in plain words, and adds links to Athena, firstmate and the cited source files. The audience is a newcomer arriving at doodla.github.io/athena.

Assumptions:
- The Athena repository URL is https://github.com/doodla/athena, inferred from the Pages address doodla.github.io/athena. The producer should confirm it from Cargo.toml or the README.
- Every new sentence about firstmate must keep citing a claim, because the generator rejects uncited elements. Sentences about Athena itself are page-level text with an 'about this page' tag, as the glossary already does.
- Claim sources link to the pinned commit in the firstmate repository, using repository_url and sha from pin.json. The links are built only when repository_url is a recognisable https GitHub URL, and otherwise the plain text is kept.
- The finding that the page does not say who the captain is stays an honest limit. If the study has no claim about the captain, the page says that plainly in newcomer terms instead of inventing facts.
- The 2/5 findings are addressed by editing the generator and the study text it renders. The walk-through, which the rater praised, stays as it is.

## Acceptance criteria

1. [content] The rendered page's <title> and first heading name Athena, and an opening paragraph says what Athena is, that this page is an example of what its study capability produces, and that firstmate is the system studied.
2. [content] The 'What firstmate is' section says who uses firstmate and that its workers are AI coding agents. The glossary entry for captain says plainly that it is the human who approves merges, or states what the study does and does not establish about that person. It no longer consists only of 'The text says no more about who the captain is.'
3. [content] The page has a visible next-step section with clickable links to the Athena repository and to firstmate's repository. Each claim source is a link to its file at the pinned commit rather than a bare file:line string.
4. [content] The 'Limits of this study' text for finding f2 no longer shows the schema phrase 'guard_evaluated_by for nobody' to the reader. It is reworded in plain language, or the schema field names are explained.
5. [behavior] Run `cargo test -p xtask`. The site tests pass, including new tests for the Athena title, the next-step links, the source links and the escaping of link text. Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
6. [behavior] Run `cargo run -p xtask -- site`. It writes index.html under target/site, and that file contains the Athena heading and the repository links. It does not write inside the work tree.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- studies/firstmate-work-handling/validation.json (modify)

## Out of scope

- Changing the study's claims, model or pinned firstmate commit
- Redesigning the page layout or the CSS beyond what the new sections need
- Changing .github/workflows/pages.yml or the deployment setup
- Re-running or re-validating the study against firstmate

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New rendering behaviour in site.rs gets unit tests, and cargo test and clippy must be clean.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The page is rewritten for an outside reader who has never heard of Athena or firstmate.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Statements about firstmate keep citing claim ids, and nothing new is asserted without a claim.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is made in the generator and the study text, not by hand-editing the output.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the site generator and its source text.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data are added, and the user's email is not written anywhere.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier builds the site and checks the output for the Athena heading and the links.
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
