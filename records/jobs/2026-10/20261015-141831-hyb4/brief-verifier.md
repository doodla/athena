# Job 20261015-141831-hyb4: verify "Add behavior-evidence default with protection-test rule and examples"

## Request (verbatim)

> try again on 20261015-095214-y8kr

## Reading (accepted by the user)

Redo rejected job 20261015-095214-y8kr from the base (its edits were discarded): add a new defaults.toml entry requiring acceptance criteria to check the requested behavior, with protection/denial tests shown unprotected-as-root then refused and failing rather than skipping. Add worked examples and a test. Keep the diff minimal: no reader.md rewording and every CHANGELOG claim true. Audience: the reader, producer and verifier receiving rendered defaults.

Assumptions:
- The earlier changes were discarded, so the retry rebuilds the whole feature, not just the two repairs.
- The defaults.toml entry is the only prompt-facing change; reader.md is left byte-identical unless the reader token budget in athena-context forces a change.
- If reader.md must change, only the intended lines change, each instruction stays its own bullet, and the CHANGELOG says why.
- The 'applies_to' of the new entry is ["*"] and its first sentence carries the core rule, since the rendered brief shows only the first sentence.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry (e.g. id "behavior-evidence") with applies_to ["*"], a source citing this request, and a statement whose FIRST sentence says criteria must check the requested behavior happening, not that types, files or wiring exist.
2. [content] The statement's later sentences require that a protection or denial test first shows the action succeeds without the protection, as the same user and environment the real component runs as (root), then shows it is refused, and that it fails, not skips or passes silently, if the environment cannot enforce the protection.
3. [behavior] A new unit test in crates/athena-data/src/defaults.rs parses the repository defaults.toml and asserts the new entry exists and its render() line contains the core rule. Run `cargo test -p athena-data`: it passes along with root_cause_default_leads_with_the_core_rule.
4. [content] docs/design/defaults-loop.md has a worked example for a 'make X read-only' request whose criteria are: a write to X as root succeeds before protection; the same root write is refused after; the test fails, not skips, if the environment cannot enforce it (a root-bypassed file mode is not proof).
5. [content] The same doc has a worked example for 'role calls a model' whose criteria run the role end to end and check the model request is observed (e.g. through the fake client) and its output is used, not merely that a client or config is wired in.
6. [content] Outcome check for the reader: the rendered reader brief for the defaults set includes the new default line with the core rule, so a reader writing criteria for a read-only request sees it. Check by running the athena-context tests, which must still pass within the reader token budget.
7. [content] `git diff` shows no collateral edits: crates/athena-context/prompts/reader.md is unchanged, or, if changed, differs only by the intended lines with every instruction still its own bullet. Each CHANGELOG.md claim under Unreleased / Added names only things present in the diff, with no claim about the reader prompt unless reader.md contains it.
8. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass from the repository root.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Building a pre-verification diff or changelog checker (the introspection's proposed code-check).
- Changing how prompts are built beyond the defaults rendering.
- Rewording existing reader prompt guidance.
- Editing records/ or the owner's files.
- Changing existing defaults.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A unit test in defaults.rs asserts the new entry and its rendered first-sentence line; cargo test, clippy and fmt must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule goes into the shared defaults every role receives, not into one prompt, and the earlier failure's cause (collateral edits and untrue changelog claims) is avoided by keeping the diff minimal.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: defaults entry, test, example doc and one changelog line.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria require running cargo test and checking the diff for collateral changes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Entry and examples are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base ed1bca45daf8 → candidate 47233dcfe4c8. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                       | 11 +++++++++++
 crates/athena-context/src/lib.rs   |  2 +-
 crates/athena-data/src/defaults.rs | 10 ++++++++++
 defaults.toml                      |  6 ++++++
 docs/decisions.md                  |  4 ++--
 docs/design/defaults-loop.md       | 26 ++++++++++++++++++++++++++
 6 files changed, 56 insertions(+), 3 deletions(-)
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
