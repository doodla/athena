# Job 20261014-154909-f4bt: Add a newcomer introduction and glossary to the explainer page

## Request (verbatim)

> Someone who's never seen firstmate can't follow the explainer page at https://doodla.github.io/athena/. Make it useful to them.

## Reading (accepted by the user)

The explainer page (rendered by xtask/src/site.rs from studies/firstmate-work-handling) jumps straight into states, counters and transitions without saying what firstmate is. I read this as adding a plain-language "start here" section: what firstmate is, how a piece of work moves through it, and a glossary of terms. Every sentence in it must cite claims, as the page's existing rule requires.

Assumptions:
- 'Never seen firstmate' means the page lacks context and definitions, not that its facts are wrong.
- The page stays generated from the study JSON by xtask; no hand-edited HTML.
- The page stays a single self-contained HTML file with no scripts or external links, as the existing tests require.
- Existing claims are enough to ground the intro; no new study claims are needed.
- Neither the workflow file nor any governance file changes.

## Acceptance criteria

1. [content] The rendered page opens with a section before 'States of a piece of work' that says in plain words what firstmate is (a supervisor that ships with a crew) and gives a short walk-through of one piece of work from start to finish.
2. [content] A glossary defines each term the page uses without explanation (for example state, counter, transition, trigger, guard, claim, pin), in wording a reader new to firstmate can follow.
3. [content] Each factual sentence in the new section cites a claim id from claims.json. Any fixed explanatory text describes only the page's own structure and makes no claim about firstmate.
4. [behavior] Run `cargo test -p xtask`: all tests pass, including a new test that the page contains the intro and glossary, and that the existing citation checks still reject an uncited element.
5. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean.
6. [behavior] Run `cargo run -p xtask -- site` from the repository root: it succeeds and writes the page without errors.

## Expected deliverables

- xtask/src/site.rs (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/claims.json (modify)

## Out of scope

- Re-studying firstmate or changing the study's claims, model or findings
- Restyling or redesigning the page layout
- Changing the Pages workflow or deployment
- Studying Hermes or Athena itself

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The page is for outside readers, which is the point of the request.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every new sentence cites a claim, or is plainly about the page's own structure.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New rendering code gets tests; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the page renderer.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: I took the missing context from the xtask renderer and the study files, not from a question.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the site build and the tests.
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
