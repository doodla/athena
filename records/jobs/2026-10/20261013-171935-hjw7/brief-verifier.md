# Job 20261013-171935-hjw7: verify "Compact reader rendering of defaults; add 'root-cause' default"

## Request (verbatim)

> Retry 20261013-170708-3p3y with the verifier's major finding fixed: leave the existing defaults' statements exactly as they are (no rewording into extra sentences); only the new 'root-cause' entry and the compact rendering change. Mention the compact rendering in the CHANGELOG entry.

## Reading (accepted by the user)

Change DefaultsModel::render so the reader sees only the first sentence of each statement, keeping full statements for producer/verifier briefs. Add a multi-sentence 'root-cause' entry to defaults.toml and leave every existing statement byte-for-byte unchanged; note the compact rendering in the CHANGELOG.

Assumptions:
- 'Compact' means first sentence of each statement, split on '. ' or terminal period, so single-sentence statements render unchanged.
- The edited design doc is docs/design/athena-v0.md (the verifier's nit), if a design edit is made at all.
- The earlier candidate reached 1723 of 1800 tokens; leaving existing statements whole adds about 40 tokens (estimate), still under budget.
- Sentence splitting must not break on abbreviations like 'e.g.'; the root-cause statement is worded to avoid them.

## Acceptance criteria

1. [content] Every existing default in defaults.toml (rust, strangers, evidence, buy-first, tested, public, small, verified, designed, lazy-requests, mechanism) has its statement, applies_to and source byte-identical to the base commit (git diff shows only added lines for the new entry).
2. [content] defaults.toml gains a 'root-cause' entry: fix where the fault originates, not the symptom or the one instance that surfaced; name an out-of-scope cause and propose the fix rather than working around it; the change description names the root cause and why the change addresses it; the root-cause fix is the one focused change ('small'). Source cites BRIEF.md addendum '2026-10-13: Root causes'. Its first sentence alone conveys the core rule.
3. [content] No new defaults.toml field is introduced (the installed gate still parses the file), and BRIEF.md and the token budgets in athena.toml are unchanged.
4. [content] DefaultsModel::render (reader form) emits only the first sentence of each statement; full statements remain available to the producer/verifier briefs via get(). Statements containing ';' or ', and' remain whole in the reader form because they are one sentence.
5. [content] CHANGELOG.md has an entry that adds the 'root-cause' default and mentions the compact reader rendering.
6. [behavior] Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass, including a new unit test that render() shows only the first sentence of a multi-sentence statement, keeps a semicolon statement whole, and that the reader's defaults component is within its token budget in crates/athena-cli/tests/budgets.rs.

## Expected deliverables

- crates/athena-data/src/defaults.rs (modify)
- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)

## Out of scope

- Rewording any existing default's statement
- Raising token budgets or editing athena.toml
- Editing BRIEF.md
- Adding a new defaults.toml field
- Changing other defaults' meaning

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Compact rendering is implemented in Rust in athena-data.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests for render; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: compact render plus the single new entry.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Token budget test enforces the reader's size limit.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier compares defaults.toml diff and runs cargo checks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and doc comments written for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the change.

## The change

Base c9f14b766a2a → candidate eb91452ce775. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                       |  3 +++
 crates/athena-data/src/defaults.rs | 54 ++++++++++++++++++++++++++++++++++++--
 defaults.toml                      |  6 +++++
 3 files changed, 61 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (366 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
