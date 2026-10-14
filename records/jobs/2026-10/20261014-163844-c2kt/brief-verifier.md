# Job 20261014-163844-c2kt: verify "Gate explainer shipping on a fresh model reader's newcomer rating"

## Request (verbatim)

> Athena's verifier passed the firstmate explainer twice, but an independent reader rates it 2 of 5 for a newcomer both times. Athena should be able to tell when an explainer it ships isn't useful, before it ships.

## Reading (accepted by the user)

Before an explainer ships, a fresh model reader with no part in building it reads the rendered page as a newcomer and rates it 1–5. A rating below 4 blocks shipping. Cheap mechanical checks on the intro and glossary sit alongside the rating.

Assumptions:
- The model reader runs through the existing athena-agent abstraction (with its fake for tests), as a read-only role with only the rendered page, not the study's build history or verifier findings.
- The gate lives in the study capability's ship path (athena-cap-study) and is invoked before the page is published. xtask site keeps only cheap deterministic structure checks, so it needs no model access.
- The reader's output is structured: a score 1-5 plus cited reasons. Below 4 is a rejection carrying those reasons as findings.
- The threshold of 4 is a named constant or config value, not a magic number.
- Mechanical checks are limited to structural ones: non-empty intro, every glossary term defined, no raw ids without plain text, size limits. They do not replace the rating.
- The current firstmate explainer is not rewritten here; the gate is expected to reject it when the reader rates it low.

## Acceptance criteria

1. [content] A newcomer-reader role exists with its own prompt. It receives only the rendered page (or its intro and glossary) and none of the producer's or verifier's notes. Its structured output is a 1-5 score plus reasons.
2. [content] Shipping the explainer is blocked when the score is below 4. The threshold is a named constant or config value. The rejection carries the reader's reasons as findings.
3. [content] Malformed or missing reader output blocks shipping; it never counts as a pass.
4. [content] Deterministic structural checks on the intro and glossary run alongside the rating. Each check has a documented reason, and its error names the failed criterion and the offending text.
5. [behavior] Run `cargo test`. Tests with a fake agent cover scores of 2, 3, 4 and 5 (blocked, blocked, ships, ships) and malformed output (blocked). Tests also check that the reader's input excludes producer and verifier material, and cover each mechanical check with a passing and a failing fixture. All pass.
6. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
7. [content] docs/design/explainer-site.md records the gate, the alternatives considered (purely mechanical, human review, the existing verifier) and why an independent model reader was chosen. It states the limits: the rating is one model's judgment and is noisy, and the reader is fresh but not a human newcomer.
8. [content] CHANGELOG.md notes the new gate.

## Expected deliverables

- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cap-study/src/newcomer.rs (create)
- crates/athena-cap-study/src/rules.rs (modify)
- xtask/src/site.rs (modify)
- docs/design/explainer-site.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-core/src/fake.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Rewriting or improving the firstmate explainer content itself
- Changing the study verifier's existing rules
- Human review workflow
- Changes to athena.toml, defaults.toml, CI workflows or lint configuration
- Calibrating the model reader against human ratings

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is that verification checks the study's rules but never asks a newcomer-reader whether the page is useful, so a usefulness gate is added.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-agent tests cover scores below, at and above 4, malformed reader output, and independence of the reader's inputs. Mechanical checks get passing and failing fixtures.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: docs/design/explainer-site.md records the gate, the alternatives (purely mechanical, human review, reusing the verifier) and the limits.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The block on shipping is enforced in code, not prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior is exercised through tests with the fake agent.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused gate; no rewrite of the firstmate content.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 857bc4208414 → candidate 44c0f408b8d1. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                |   7 +
 Cargo.lock                                  |   1 +
 crates/athena-cap-study/Cargo.toml          |   3 +-
 crates/athena-cap-study/prompts/newcomer.md |  13 +
 crates/athena-cap-study/src/lib.rs          |   4 +-
 crates/athena-cap-study/src/newcomer.rs     | 611 ++++++++++++++++++++++++++++
 docs/architecture.md                        |   2 +-
 docs/design/explainer-site.md               |  40 ++
 8 files changed, 677 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (557 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
