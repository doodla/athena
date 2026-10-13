# Job 20261013-224445-96rh: verify "Add ensure_changelog and verifier scope rule; leave reader prompt alone"

## Request (verbatim)

> Retry 20261013-223248-jk8v without the reader prompt line, which put the reader set over its 1800-token budget: keep ensure_changelog in reading.rs (CHANGELOG.md added to change readings, with the 'changelog' default citing BRIEF.md), the verifier brief's scope section (a changed file outside the deliverables list is the scope finding, judged against the list, not criterion wording), the tests, and the CHANGELOG.md and docs/decisions.md entries. Do not change reader.md or defaults.toml.

## Reading (accepted by the user)

Retry of jk8v without the reader.md line and without touching defaults.toml. Code in reading.rs adds CHANGELOG.md to change readings' deliverables (with a 'changelog' default citing BRIEF.md), and the verifier brief states that a changed file outside the deliverables list is the scope finding.

Assumptions:
- The verifier brief's scope section is the 'Expected deliverables' part generated in crates/athena-context/src/briefs.rs; the producer may instead put the sentence in prompts/verifier.md if that is where verifier prose lives.
- Since defaults.toml cannot change, the 'changelog' default is a synthesized defaults_applied entry (id 'changelog') written by code, not a new entry in defaults.toml.
- The reader's 1800-token budget is untouched because reader.md is unchanged; the added brief text goes to the verifier, not the reader.
- Edits to the existing 'Expected deliverables' heading text must not break existing briefs.rs tests.

## Acceptance criteria

1. [behavior] cargo test passes, including new tests: for a change reading with no CHANGELOG.md deliverable, build_reading adds a modify CHANGELOG.md deliverable and a 'changelog' defaults_applied entry whose text cites BRIEF.md's 'tagged releases and a changelog'.
2. [behavior] cargo test covers that ensure_changelog adds nothing when CHANGELOG.md is already listed (no duplicate default or deliverable) and does nothing for non-change (study or declined) readings.
3. [behavior] cargo test covers that the verifier brief contains a scope section saying a changed file outside the deliverables list is the scope finding, judged against the list and not against criterion wording.
4. [content] ensure_changelog is a function in crates/athena-core/src/reading.rs that does no text matching of acceptance criteria.
5. [content] CHANGELOG.md and docs/decisions.md each gain an entry describing the change and why (single statement of scope, no heuristics).
6. [content] crates/athena-context/prompts/reader.md and defaults.toml are unchanged.
7. [behavior] cargo clippy and cargo fmt --check are clean.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- crates/athena-context/prompts/reader.md
- defaults.toml
- any text matching over acceptance criteria
- changes to admission or governance-file rules

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code plus docs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests for ensure_changelog and the verifier brief scope text; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the code rule, the brief text, tests, and two doc entries.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Scope is made a single statement (the deliverables list) instead of heuristics over criteria.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc entries state the why for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The changelog rule is enforced in code, not prose.

## The change

Base fa0e140c0c18 → candidate 328cf5704229. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |  9 +++
 crates/athena-context/prompts/verifier.md |  1 +
 crates/athena-context/src/briefs.rs       | 30 ++++++++++
 crates/athena-core/src/reading.rs         | 96 +++++++++++++++++++++++++++++++
 docs/decisions.md                         | 21 +++++++
 5 files changed, 157 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (382 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
