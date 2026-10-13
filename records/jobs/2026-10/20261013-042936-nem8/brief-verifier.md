# Job 20261013-042936-nem8: verify "Design the defaults-learning loop in docs/design/defaults-loop.md"

## Request (verbatim)

> Close the defaults loop: use Jev to match interventions and corrections against the defaults model, so repeats become new, versioned defaults.

## Reading (accepted by the user)

Docs-only job: write docs/design/defaults-loop.md describing how Jev matches interventions and corrections to defaults and how repeats become proposed, versioned defaults for review. No code, no defaults.toml edits; proposals are review-only.

Assumptions:
- Records are not visible to me; the producer must find the cited repeats in what it can read, and label any it cannot verify.
- Design doc follows the style of existing docs/design/*.md, including alternatives.
- Review of the design happens in a later job.

## Acceptance criteria

1. [content] docs/design/defaults-loop.md exists and is grounded in real intervention/correction records, citing them, including the four repeats: CHANGELOG excluded by scope criteria (x4), reader token budget blocking (x4), design/review/build bundled into one job, config field and its use in one commit.
2. [content] It states what Jev decides (matching a record to a default id with a probability, clustering/paraphrase, drafting statement text) versus what deterministic code decides (threshold, id uniqueness, version bump, validation via DefaultsModel::parse, writing the proposal).
3. [content] It specifies the proposal format: fields id, statement, applies_to, version bump, and sources citing records, with an example.
4. [content] It describes how a proposal reaches the user for review, and that Athena never edits defaults.toml itself; proposals are review-only.
5. [content] It describes Jev calibration before trust: shadow mode, labeled set from past records, scoring rule, and promotion criteria; and addresses the D31 exclusion of Jev.
6. [content] It lists alternatives considered, and numbers cited are measured from records or labeled as estimates. No secrets or personal data appear.
7. [content] No Rust, defaults.toml, or governance files change.

## Expected deliverables

- docs/design/defaults-loop.md (create)
- docs/decisions.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Any Rust code or tests
- Editing defaults.toml or bumping its version
- Implementing the Jev gateway client
- Landing new defaults automatically

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design written down first with alternatives, before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite records; numbers measured or labeled estimates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One document, no code.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the doc.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.

## The change

Base cf5529134965 → candidate d8923950d671. In your working directory, `git diff HEAD~1` shows it.

```text
 docs/decisions.md            |  25 ++++
 docs/design/defaults-loop.md | 328 +++++++++++++++++++++++++++++++++++++++++++
 docs/design/roadmap.md       |   7 +-
 3 files changed, 358 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (382 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
