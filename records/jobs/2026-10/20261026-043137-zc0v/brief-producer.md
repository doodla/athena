# Job 20261026-043137-zc0v: Design Jev code-health checks as a shadow-mode design doc

## Request (verbatim)

> Priority: urgent
> Design (no code yet) Jev checks for code health (BRIEF.md addendum of 2026-10-26, 'Get Jev checks designed for code health'). Jev answers typed choice questions with a probability per option (POST /v1/evaluate, about $0.00001 a call; docs/design/defaults-loop.md section 6 describes the calling convention), so it can judge every changed file and function at every landing where line counts cannot. Design the questions: for example whether a function does one thing or several, whether a change adds coupling between modules, which module a new item belongs in, whether a file is drifting toward a god object, and whether a change duplicates logic that exists elsewhere. For each: the state Jev is given (a bounded excerpt and its context), the options and criteria, the confidence threshold, and what a confident answer does (a finding in status, a structural-pass request via the code-health check of request 20261026-042558-5p65, or nothing). Start in shadow mode, recording answers without acting, and score them by evidence, as Sri asked for Jev's class answers (BRIEF.md addendum of 2026-10-25): a file Jev flags that later breaks or churns, a split it recommends that later lands and reduces churn. Say how the checks fit with the deterministic code-health measures, Hestia (20261025-033743-npr5) and the scope guard, and what they cost per landing.
> Acceptance:
> - docs/design/jev-code-health.md lists each question with its state, options, criteria, threshold and effect, the shadow period and the evidence that scores it, and the cost per landing.
> - A proposed entry in docs/decisions.md and a build plan in jobs each sized for one producer run.
> Files: docs/design/jev-code-health.md, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Write docs/design/jev-code-health.md (no code): typed Jev questions for judging changed files and functions, each with state, options, criteria, threshold and effect, run in shadow mode and scored by later evidence, plus fit with deterministic code-health, Hestia and the scope guard, and cost per landing. Add a proposed decision entry and a build plan of one-producer-run jobs. Audience: Sri, who will judge whether the checks are specified well enough to build.

Assumptions:
- Jev's calling convention is read from docs/design/defaults-loop.md section 6 and cited, not restated from memory.
- The deterministic code-health check of request 20261026-042558-5p65 is not yet built; the doc refers to it as planned, with its measures and thresholds as that request states them.
- docs/design/hestia.md exists (from 20261025-033743-npr5) and docs/design/scope-guard.md exists; the doc cites both rather than changing them.
- Costs are computed from the stated $0.00001 per call times a stated calls-per-landing count, labeled as estimates.
- Questions proposed: function-does-one-thing, adds-coupling, which-module-for-new-item, god-object-drift, duplicates-existing-logic.
- The decisions.md entry is marked proposed, not accepted; build plan items are listed in the design doc and in the decisions entry.
- No code, athena.toml or workflow changes; shadow mode needs only a records format that the build plan describes.

## Acceptance criteria

1. [content] docs/design/jev-code-health.md has, for each of at least five questions (one-thing-or-several, added coupling, module placement, god-object drift, duplicated logic), the state given to Jev (bounded excerpt and context), the options, the criteria, a numeric confidence threshold, and the effect of a confident answer (status finding, structural-pass request via 20261026-042558-5p65, or nothing).
2. [content] The doc defines the shadow period (answers recorded, no action), its exit rule, and the evidence that scores each question: a flagged file that later breaks or churns, and a recommended split that lands and reduces churn, with how each is measured from landing records.
3. [content] The doc explains how the checks relate to the deterministic code-health measures, Hestia and the scope guard, citing those docs, and gives cost per landing as calls times $0.00001 with the call-count assumption labeled as an estimate.
4. [content] docs/decisions.md gains one entry marked proposed (not accepted) with alternatives, and the doc's build plan lists jobs each stated as sized for one producer run with files and a test or demonstration per job.
5. [content] CHANGELOG.md has an entry for the design; no files outside docs/design/jev-code-health.md, docs/decisions.md and CHANGELOG.md change, and cargo test still passes (any doc/decision link test is satisfied).
6. [behavior] Run cargo test from the repository root: all tests pass, including any test that checks decisions.md format or links.

## Expected deliverables

- docs/design/jev-code-health.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Implementing any Jev call, question runner or status display
- Building the deterministic code-health check of 20261026-042558-5p65
- Changing Hestia, scope guard, athena.toml, defaults.toml or workflows
- Marking the decision accepted

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Doc gives alternatives considered (e.g. line-count only, model per file vs per function) and a proposed decision for review before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Costs and call counts are measured from cited docs or labeled as estimates; scoring evidence is defined from landing records.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Scoring criteria name observable later events (flagged file breaks or churns; recommended split lands and churn drops), not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Build plan split into jobs each sized for one producer run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc written for an outside reader, defining Jev and the shadow period.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Each question's effect and the shadow-period exit rule are tied to measured outcomes.
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

docs/design/jev-code-health.md has, for each of at least five questions (one-thing-or-several, added coupling, module placement, god-object drift, duplicated logic), the state given to Jev (bounded excerpt and context), the options, the criteria, a numeric confidence threshold, and the effect of a confident answer (status finding, structural-pass request via 20261026-042558-5p65, or nothing).
The doc defines the shadow period (answers recorded, no action), its exit rule, and the evidence that scores each question: a flagged file that later breaks or churns, and a recommended split that lands and reduces churn, with how each is measured from landing records.
The doc explains how the checks relate to the deterministic code-health measures, Hestia and the scope guard, citing those docs, and gives cost per landing as calls times $0.00001 with the call-count assumption labeled as an estimate.
docs/decisions.md gains one entry marked proposed (not accepted) with alternatives, and the doc's build plan lists jobs each stated as sized for one producer run with files and a test or demonstration per job.
CHANGELOG.md has an entry for the design; no files outside docs/design/jev-code-health.md, docs/decisions.md and CHANGELOG.md change, and cargo test still passes (any doc/decision link test is satisfied).
Run cargo test from the repository root: all tests pass, including any test that checks decisions.md format or links.

## Report

End with the producer report as your structured output.
