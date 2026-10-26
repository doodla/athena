# Job 20261026-065046-0c1d: verify "Revise Jev code-health design to reuse existing Jev client and scorer"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the doc's 'Existing code' section and check each listed path exists. Run: `ls crates/athena-core/src/jev.rs crates/athena-core/src/labeled.rs crates/athena-cli/tests/labeled.rs docs/design/jev-code-health.md`. Observes: All paths listed; the section names request_body, FakeJev, live_jev_answers_both_question_kinds and score-jev with purposes.
- Criterion 2: Read the build-plan table: each item J1..J5 has reuse/extend/new and the only 'new' items are questions, state builders, thresholds, shadow records, evidence scoring. Run: `git diff --stat`. Observes: Only the four deliverable files changed; the doc's table has no new client, fake or scorer row.
- Criterion 3: Read the batching and cost section for the request_body citation, the range, the defaults-loop.md citation and 'estimate' labels. Run: `git diff -- docs/design/jev-code-health.md`. Observes: Text cites jev.rs request_body, $0.000012 to about $0.00002, defaults-loop.md, per-landing batched cost marked estimate.
- Criterion 4: Read the questions section: five questions each with state, options, criteria, numeric threshold, effect. Run: `git diff -- docs/design/jev-code-health.md`. Observes: One-thing-or-several, added coupling, module placement, god-object drift, duplicated logic each have all five fields with a number and an effect from {status finding, structural-pass request via 5p65, nothing}.
- Criterion 5: Read the shadow section for period, exit rule, evidence measures from landing records, and the feed into labeled.rs. Run: `git diff -- docs/design/jev-code-health.md`. Observes: Defines shadow period and exit rule, flagged-file break/churn and split-lands-reduces-churn measures, and how rows become labeled-set input for score_jev.
- Criterion 6: Read the fit section. Run: `git diff -- docs/design/jev-code-health.md`. Observes: Sections on deterministic measures (5p65 marked assumed), Hestia (npr5, hestia.md) and scope guard (scope-guard.md, D127).
- Criterion 7: Inspect the changed files. Run: `git status`. Observes: Only docs/design/jev-code-health.md, docs/decisions.md, CHANGELOG.md (and decision-links.toml only if needed) changed; D131 is marked proposed with alternatives; each build job states one-producer-run size, files and test.
- Criterion 8: Run the full test suite, which includes decisions format, decision-links and boundary/policy tests. Run: `cargo test --locked`. Observes: All tests pass.
  Fails without the behavior: A malformed D131 entry, a shifted decision-link line, or a banned word or email-like text in the new files would fail the decisions-format, links or policy tests.

Planned files:

- docs/design/jev-code-health.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)

## The change

Base 4e422e84e6d9 → candidate 7704277e87c7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                   |   3 +
 docs/decisions.md              |  25 +++++
 docs/design/jev-code-health.md | 235 +++++++++++++++++++++++++++++++++++++++++
 3 files changed, 263 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (1279 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
