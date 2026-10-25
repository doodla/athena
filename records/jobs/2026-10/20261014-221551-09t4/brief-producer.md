# Job 20261014-221551-09t4: Make the firstmate explainer page followable by newcomers

## Request (verbatim)

> Someone who's never seen firstmate can't follow the explainer page at https://doodla.github.io/athena/. Make it useful to them.

## Reading (accepted by the user)

The page already has a "Start here" intro and glossary (intro.json, xtask/src/site.rs), but the sections after it still show raw ids (t1, c12), "decided by agent/code" fields and terse state tags that a newcomer can't read. I'll make those sections self-explanatory with cited plain-language lead-ins and card summaries, so a reader who has never seen firstmate can follow the page top to bottom.

Assumptions:
- The audience is a reader who has never seen firstmate and opens the published GitHub Pages page; the page is built by `cargo run -p xtask -- site` from studies/firstmate-work-handling.
- The deployed page may lag the repo's existing intro; I assume the remaining gap is in the sections after 'Start here', which show ids and jargon unexplained.
- Every factual sentence keeps citing claim ids from claims.json; no new claims about firstmate are invented and the study is not re-run.
- The deploy workflow (.github/workflows/pages.yml) is not changed.

## Acceptance criteria

1. [behavior] Run `cargo run -p xtask -- site`; the rendered page has a short lead-in under each of the States, Counters, Transitions, and Why sections that says in plain words what the section shows and how to read it, and every firstmate-specific term it uses appears in the glossary.
2. [content] In the rendered Transitions section, each card shows a plain-language one-line summary of what happens before the trigger/guard fields, and 'agent' and 'code' as deciders are explained. A newcomer can read the cards without knowing the ids.
3. [behavior] Run `cargo test -p xtask`; all tests pass, including new ones asserting the lead-ins appear, that every new sentence still cites an existing claim, and that an uncited new element is rejected.
4. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`; both are clean.
5. [content] CHANGELOG.md records the change, and the explainer design doc matches the new page structure.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)
- studies/firstmate-work-handling/model.json (modify)

## Out of scope

- Re-running or extending the firstmate study and adding new claims
- Changing the pages workflow or hosting
- Visual redesign beyond what readability needs
- Changing the study's pinned commit

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The page is written for an outside reader who knows nothing of firstmate.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every new sentence cites claim ids; the renderer rejects uncited ones.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New renderer behavior gets xtask tests; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the renderer and intro text.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the page source and existing intro from the repository instead of asking.
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
