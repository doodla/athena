# Job 20261015-095214-y8kr: verify "Add a defaults entry: criteria check behavior, protections tested as roo"

## Request (verbatim)

> Add a reading default: every acceptance criterion must check the requested behavior happening, not just that types, files or wiring exist. Any test of a protection or denial must first show the action succeeds without the protection, under the same user and environment the real component runs as (here, root), and then show it is refused. If the environment cannot enforce the protection, the test must fail, not skip or pass silently. Show that a reading for a 'make X read-only' or 'role calls a model' request produces criteria of this form.

## Reading (accepted by the user)

Add a new entry to defaults.toml (applies to all work) saying every acceptance criterion must check the requested behavior happening, and that protection or denial tests must first show the action succeeds unprotected as the real user (root), then is refused, and fail rather than skip if the environment cannot enforce it. Add worked examples for 'make X read-only' and 'role calls a model' requests. The audience is the reader, producer and verifier, who receive the rendered defaults.

Assumptions:
- 'Reading default' means a new entry in defaults.toml that the reader cites in defaults_applied; it is not a code change to the reader's logic.
- The new entry applies_to ["*"] because the rule concerns acceptance criteria for any class of work.
- 'Show that a reading ... produces criteria of this form' is satisfied by worked examples in docs plus tests that the rendered default carries the rule; Athena cannot run the reader live from tests.
- The existing render() only emits the first sentence, so the core rule must lead the statement; the root-user and fail-not-skip details stay in the full statement text.
- The verifier's allowed commands run as the real component's user, so examples cite cargo test as the way to check.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry (for example id "behavior-evidence") with applies_to ["*"], a source citing this request, and a statement whose FIRST sentence states the core rule (criteria check the requested behavior happening, not that types, files or wiring exist), because the rendered brief shows only the first sentence.
2. [content] The statement's later sentences require that any protection/denial test first shows the action succeeds without the protection, under the same user and environment the real component runs as (root), then shows it is refused, and that if the environment cannot enforce the protection the test fails rather than skips or passes silently.
3. [behavior] A new unit test in crates/athena-data/src/defaults.rs parses the repository defaults.toml and asserts the new entry exists and its render() line appears with the core rule; run `cargo test -p athena-data` and see it pass, while existing tests (including root_cause_default_leads_with_the_core_rule) still pass.
4. [content] A worked example (docs/design/defaults-loop.md or a doc test fixture) shows a reading for 'make X read-only' whose criteria are: (1) a write to X as root succeeds before protection, (2) after protection the same write as root is refused, (3) the test fails, not skips, if the environment cannot enforce it (e.g. a root-bypassed file mode is not accepted as proof).
5. [content] A second worked example for 'role calls a model' has criteria that run the role end to end and check the model call is made and its output is used (for example through the fake client, with the request observed), not merely that a client type or config is wired in.
6. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` all pass from the repository root.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Changing how the reader, producer or verifier prompts are built beyond the existing defaults rendering
- Implementing an actual read-only protection or model-calling role
- Editing BRIEF.md or records/

## User defaults that apply

- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a unit test that the entry exists and renders its core sentence, not left as prose.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add a defaults.rs test; cargo test and clippy stay clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checked by running cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new default plus tests and examples, with no reader logic changes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The statement and examples are written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The docs example cites the source request and the defaults-loop design.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 1e533d24e01c → candidate 8d61152bc243. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |  9 +++++++++
 crates/athena-context/prompts/reader.md | 11 +++++------
 crates/athena-data/src/defaults.rs      | 21 +++++++++++++++++++++
 defaults.toml                           |  6 ++++++
 docs/design/defaults-loop.md            | 29 +++++++++++++++++++++++++++++
 5 files changed, 70 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (685 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
