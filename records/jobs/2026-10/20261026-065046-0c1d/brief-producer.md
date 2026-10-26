# Job 20261026-065046-0c1d: Revise Jev code-health design to reuse existing Jev client and scorer

## Request (verbatim)

> try again on 20261026-043137-zc0v

## Reading (accepted by the user)

Retry of the rejected design job: write docs/design/jev-code-health.md (no code) specifying typed Jev code-health questions in shadow mode, built on the existing Jev client, FakeJev, live test and labeled.rs/score-jev in crates/athena-core. Audience: Sri, judging whether the checks are specified well enough to build.

Assumptions:
- Deterministic code-health measures from 20261026-042558-5p65 are not in the repo, so the doc marks them as assumed.
- Prior attempt's changes were discarded; the doc is written from the base.
- Goal: a design Sri can judge as buildable; audience Sri.

## Acceptance criteria

1. [content] docs/design/jev-code-health.md has an 'Existing code' section listing with file paths and purpose: the POST /v1/evaluate client and FakeJev in crates/athena-core/src/jev.rs, the opt-in live test live_jev_answers_both_question_kinds, and labeled.rs with the `athena score-jev` command; each path exists in the repo.
2. [content] Every build-plan item (including J3 and J4) is marked reuse, extend or new against that survey; the plan adds only code-health questions, state builders, thresholds, shadow records and evidence scoring, and rebuilds no client, fake or scorer.
3. [content] Batching (several questions per call, citing jev.rs request_body) and cost per landing use batched calls, with the per-call price given as a range ($0.000012 to about $0.00002, citing defaults-loop.md) and call counts labeled estimates.
4. [content] For each of at least five questions (one-thing-or-several, added coupling, module placement, god-object drift, duplicated logic) the doc gives state (bounded excerpt and context), options, criteria, numeric confidence threshold, and effect (status finding, structural-pass request via 20261026-042558-5p65, or nothing).
5. [content] The doc defines the shadow period, its exit rule, and evidence scoring (flagged file later breaks or churns; recommended split lands and reduces churn), how each is measured from landing records, and how it feeds the existing labeled.rs scorer.
6. [content] The doc explains fit with deterministic code-health measures (those from 20261026-042558-5p65 marked as assumed since the request is not in the repo), Hestia (20261025-033743-npr5) and the scope guard.
7. [content] docs/decisions.md gains one entry marked proposed with alternatives; build-plan jobs are each stated as sized for one producer run with files and a test or demonstration; CHANGELOG.md has an entry; no other files change except docs/decision-links.toml if required.
8. [behavior] Run cargo test from the repository root: all tests pass, including any test checking decisions.md format or decision links.

## Expected deliverables

- docs/design/jev-code-health.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Writing any Rust code
- Changing jev.rs, labeled.rs or the score-jev command
- Implementing the deterministic code-health check

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria require scoring by later evidence (churn, breakage) and cargo test passing.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Cost and call counts labeled estimates; existing code cited by path.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Proposed decision entry with alternatives.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Build plan split into one-producer-run jobs.
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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/design/jev-code-health.md has an 'Existing code' section listing with file paths and purpose: the POST /v1/evaluate client and FakeJev in crates/athena-core/src/jev.rs, the opt-in live test live_jev_answers_both_question_kinds, and labeled.rs with the `athena score-jev` command; each path exists in the repo.
Every build-plan item (including J3 and J4) is marked reuse, extend or new against that survey; the plan adds only code-health questions, state builders, thresholds, shadow records and evidence scoring, and rebuilds no client, fake or scorer.
Batching (several questions per call, citing jev.rs request_body) and cost per landing use batched calls, with the per-call price given as a range ($0.000012 to about $0.00002, citing defaults-loop.md) and call counts labeled estimates.
For each of at least five questions (one-thing-or-several, added coupling, module placement, god-object drift, duplicated logic) the doc gives state (bounded excerpt and context), options, criteria, numeric confidence threshold, and effect (status finding, structural-pass request via 20261026-042558-5p65, or nothing).
The doc defines the shadow period, its exit rule, and evidence scoring (flagged file later breaks or churns; recommended split lands and reduces churn), how each is measured from landing records, and how it feeds the existing labeled.rs scorer.
The doc explains fit with deterministic code-health measures (those from 20261026-042558-5p65 marked as assumed since the request is not in the repo), Hestia (20261025-033743-npr5) and the scope guard.
docs/decisions.md gains one entry marked proposed with alternatives; build-plan jobs are each stated as sized for one producer run with files and a test or demonstration; CHANGELOG.md has an entry; no other files change except docs/decision-links.toml if required.
Run cargo test from the repository root: all tests pass, including any test checking decisions.md format or decision links.

## Report

End with the producer report as your structured output.
