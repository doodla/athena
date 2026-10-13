# Job 20261013-033155-kmh0: Study firstmate's work handling as a validated study

## Request (verbatim)

> Study firstmate's work handling (M1).

## Reading (accepted by the user)

Read as a study job: model how firstmate (pinned from the configured repo, kunchenguid/firstmate) handles work, as cited typed records plus a work-handling model that the athena-cap-study validator accepts. It is the "study firstmate's work handling" first use of the capability named in BRIEF.md; no Athena behavior changes.

Assumptions:
- The study is stored under studies/firstmate-work-handling/ as JSON records; the exact layout follows the athena-cap-study schemas and may differ.
- 'M1' means the first milestone in BRIEF.md (one mechanism, work handling, on firstmate), not a code milestone.
- Scope is firstmate only; Hermes and Athena itself are separate studies.
- I did not read pinned-source/ or the record schemas in crates/athena-cap-study, so the file names are guesses.

## Acceptance criteria

1. [content] A study directory (assumed studies/firstmate-work-handling/) holds typed records: pin (resolved SHA), claims, rationale cards, and a work-handling model, matching the study.*/1 schemas in docs/design/study-capability-m1.md.
2. [content] Every claim about firstmate cites a file and line range (or run) in pinned-source/ with a quote; numbers are measured or labeled as estimates.
3. [content] The model covers how work enters, is assigned to the crew, progresses, is reviewed, and finishes or fails, with guards and each element evidenced.
4. [content] Any divergence between what firstmate's docs say and what it does is recorded as a contradicts-linked finding, and enforcement of each rule is marked as mechanism or prose-only.
5. [behavior] Run cargo test --locked and cargo clippy --all-targets --locked -- -D warnings; both pass, and the study validation (every section 5.4 rule) reports pass with no offending ids.
6. [behavior] Run cargo fmt --all --check; it passes.

## Expected deliverables

- studies/firstmate-work-handling/pin.json (create)
- studies/firstmate-work-handling/claims.json (create)
- studies/firstmate-work-handling/model.json (create)
- studies/firstmate-work-handling/validation.json (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Studying Hermes Agent or Athena itself
- Rendering the explainer page (Pages)
- Changing the study capability crate or schemas
- Changing governance files (athena.toml, CI, lints)
- Running firstmate

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim cites the pinned source; numbers are measured or labeled as estimates.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered 'M1' and the scope from BRIEF.md and docs/design/study-capability-m1.md.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Quotes from firstmate are public repo text only; no secrets or personal data.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier reruns validation and checks citations against the pin.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One study on one mechanism.

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash in code, docs, or design-revision text (BRIEF.md included); describe the content instead.

## Report

End with the producer report as your structured output.
