# Job 20261015-001450-ethx: Make the firstmate explainer page followable by newcomers

## Request (verbatim)

> try again on 20261014-221551-09t4

## Reading (accepted by the user)

Add cited plain-language lead-ins under the States, Counters, Transitions and Why sections, and one-line summaries and explained deciders on transition cards, so a reader new to firstmate can follow the page and glossary. The retry avoids the earlier failure by not adding fields to model.json, whose strict schema rejects unknown fields like `summary`; new prose goes in intro.json or site.rs, and cargo test is run before finishing.

Assumptions:
- Audience: a newcomer who has never seen firstmate, reading https://doodla.github.io/athena/.
- The failed attempt broke because crates/athena-cap-study deserializes model.json with deny_unknown_fields, so a `summary` field was rejected; this attempt must not add unknown fields to model.json or must change the schema and its tests together.
- Preferred approach: put new plain-language text in intro.json (or generated in xtask/src/site.rs from existing model data), each sentence citing an existing claim.
- The 'basic example walked through start to finish' from the proposed request is satisfied by a short worked example in the intro that follows one piece of work through states and transitions, citing claims.
- The producer runs cargo test --locked, clippy and fmt --check before finishing.

## Acceptance criteria

1. [behavior] Run `cargo test --locked` (whole workspace); all tests pass, including athena-cap-study's rules tests that load studies/firstmate-work-handling/model.json, and new xtask tests asserting the lead-ins and example render and that an uncited new element is rejected.
2. [behavior] Run `cargo run -p xtask -- site`; the rendered page has a plain-language lead-in under each of States, Counters, Transitions and Why saying what it shows and how to read it.
3. [content] A newcomer can read the rendered Transitions cards without knowing ids: each card opens with a one-line plain summary, and the deciders 'agent' and 'code' are explained.
4. [content] Every firstmate-specific term is defined in the glossary or at first use, and the page includes a short worked example from start to finish; every new sentence cites an existing claim.
5. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`; both are clean.
6. [content] CHANGELOG.md records the change and docs/design/explainer-site.md matches the new page structure.

## Expected deliverables

- xtask/src/site.rs (modify)
- studies/firstmate-work-handling/intro.json (modify)
- CHANGELOG.md (modify)
- docs/design/explainer-site.md (modify)
- studies/firstmate-work-handling/model.json (modify)
- crates/athena-cap-study/src/model.rs (modify)

## Out of scope

- Changing the firstmate study's claims, pin, or findings
- Changing the deployment workflow or hosting of the site
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests assert lead-ins and the worked example render and cite existing claims; cargo test, clippy and fmt must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every new sentence cites an existing claim id.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The earlier failure came from the model.json schema, so the plan keeps new prose out of strict-schema files.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Wording is plain and defines each term on first use.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the page generator, intro data, and docs.
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
