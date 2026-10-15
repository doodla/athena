# Job 20261015-060245-yqev: Retry: correct and complete the firstmate explainer page

## Request (verbatim)

> try again on 20261015-033631-x3a1

## Reading (accepted by the user)

Retry of rejected job 20261015-033631-x3a1: fix what the firstmate explainer page (built by `xtask site` from studies/firstmate-work-handling) gets wrong or omits, for newcomers who read only the page, and also fix the two earlier nits. The earlier rejection was a verifier snapshot mutation, not a repository fault. Changing the verifier is not part of this request.

Assumptions:
- The typed request, 'try again', takes precedence over the introspection's proposed verifier-hardening request, so verification infrastructure is not changed here.
- The audience is newcomers who read only the page; the goal is a rating of 4 or more out of 5 and no wrong or missing firstmate facts.
- The earlier reading and its criteria remain valid, as the user accepted them.
- Fixes are limited to facts already backed by claims, or new claims whose quotes appear in the pinned source.
- The page text 'only by a session start' must be reworded to what c14 and c15 say, or backed by a claim that says it directly.
- The CHANGELOG must say the source-quote test runs only when pinned-source/ is mounted, not that it always checks the pinned commit.

## Acceptance criteria

1. [content] Every changed firstmate fact in intro.json, claims.json and model.json is backed by a claim whose quote appears in the pinned firstmate source. The CHANGELOG entry lists each correction and addition with the firstmate source path and lines.
2. [content] The Start here section and transition cards state the failure and limit behaviour found in the claims. Lock contention exits 1 without waiting. An unreadable capacity exits 1. A relaunch failure restores the record. No retry or relaunch bound was found. A missing session start leaves a close unreplayed. Merge authority is worded as the configured setting, and the 'ship' glossary entry is corrected.
3. [content] The glossary entry 'exit 1' says only what claims c4 and c7 support (the lock and unreadable-capacity cases). 'captain' is not described with words no claim states. Placeholder values (guard 'none' with evaluator 'agent' on t5, t7, t8) appear on the page as 'no guard applies'.
4. [content] The wording 'only by a session start' in intro.json (what[6] and summary t5) matches what c14 and c15 state, or a claim says it directly. The CHANGELOG states that the source-quote test runs only when pinned-source/ is mounted.
5. [behavior] Run `cargo run -p xtask -- site --out <fresh dir inside the working directory>`. It exits 0 and writes index.html. Every cited claim id resolves and no intro element, glossary term or transition summary is uncited.
6. [behavior] Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `cargo fmt --check`. All pass.
7. [content] No governance file (athena.toml, defaults.toml, workflows) changes. If the intro structure changes, docs/design/explainer-site.md describes it.

## Expected deliverables

- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/rationales.json (modify)
- studies/firstmate-work-handling/findings.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- xtask/src/site.rs (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how the verifier is run (read-only snapshot, fingerprinting, rerun on mutation)
- Changes to athena.toml, defaults.toml or workflows
- Changing the pinned firstmate commit
- Studying parts of firstmate beyond work handling

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The page and CHANGELOG are written for newcomers who read only the page.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each corrected fact cites a claim whose quote is in the pinned source, and the CHANGELOG gives source paths and lines.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The site build is run to check that every citation resolves.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The wording is fixed in the claims and model, not only the page text.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The goal and criteria were recovered from the earlier job instead of asking.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused docs change in the firstmate study; no infrastructure work.
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
