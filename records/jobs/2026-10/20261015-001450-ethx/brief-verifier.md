# Job 20261015-001450-ethx: verify "Make the firstmate explainer page followable by newcomers"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base eaf3bdb4f09d → candidate b1f82495fa31. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                               |   6 ++
 docs/design/explainer-site.md              |   9 +-
 studies/firstmate-work-handling/intro.json |  30 +++++++
 xtask/src/site.rs                          | 137 ++++++++++++++++++++++++++---
 4 files changed, 168 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (596 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
