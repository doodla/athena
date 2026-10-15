# Job 20261015-124217-6xhe: Make the explainer site introduce Athena and fix newcomer findings

## Request (verbatim)

> try again on 20261015-104515-8wqq

## Reading (accepted by the user)

Rework the generated explainer page so a newcomer at doodla.github.io/athena first learns what Athena is and that the page is an example study of firstmate. Add a next-step section, linked claim sources, and a plain-language f2 limit. The producer must run the gates and map each criterion to evidence before reporting.

Assumptions:
- Audience is a newcomer arriving at the site.
- Athena repo URL is https://github.com/doodla/athena; firstmate URL is taken from the study pin.
- Source links point to files at the pinned commit on the firstmate repository host.
- Study text edits may require regenerating or updating validation.json; edit it only if the validator requires.
- The producer's final report includes a per-criterion evidence table and the results of the gate commands.

## Acceptance criteria

1. [content] In xtask/src/site.rs the page title and first heading name Athena, and an opening paragraph says what Athena is, that the page is an example of what its study capability produces, and that firstmate is the system studied. A named test asserts this.
2. [content] The 'What firstmate is' text says who uses firstmate and that its workers are AI coding agents. The captain glossary entry says plainly who the captain is, or what the study does and does not establish, and is not only 'The text says no more about who the captain is.'
3. [content] The page has a visible next-step section with anchor links to the Athena repository and firstmate's repository. Each claim source renders as an anchor to its file at the pinned commit, with link text HTML-escaped. Named tests cover the next-step links, the source links and the escaping.
4. [content] The text for finding f2 in 'Limits of this study' no longer contains 'guard_evaluated_by for nobody' and is in plain language. A test asserts the phrase is absent from the rendered HTML.
5. [behavior] Run `cargo test -p xtask`, `cargo clippy --all-targets` and `cargo fmt --check`. All pass with no warnings. Run `cargo test` for the study validation too.
6. [behavior] Run `cargo run -p xtask -- site`. It writes target/site/index.html containing the Athena heading and the repository links, and `git status` shows no changes in the work tree from the build.
7. [content] The final report has a table mapping each criterion to evidence (test name or HTML excerpt) and records the gate command results.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- studies/firstmate-work-handling/validation.json (modify)

## Out of scope

- Changing the study's claims or pin
- Changing GitHub Pages workflow or deployment
- Adding install or run tooling for firstmate
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New site tests; cargo test, clippy and fmt must be clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Site is built and its HTML inspected.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Page copy written for an outside newcomer.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the site generator and its study text.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the framing in the generator and the study text rather than patching one string.
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
