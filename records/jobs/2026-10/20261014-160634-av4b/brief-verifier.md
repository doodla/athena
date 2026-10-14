# Job 20261014-160634-av4b: verify "Update the Claude Code pin to 2.1.299"

## Request (verbatim)

> Update the Claude Code pin to 2.1.299.

## Reading (accepted by the user)

Change the pinned Claude Code version from 2.1.296 to 2.1.299 in athena.toml (claude_version) and in the current-state docs that quote the pin. Historical review records and test fixtures that merely use a version string are left alone.

Assumptions:
- 2.1.299 is a real published npm release of @anthropic-ai/claude-code; the verifier cannot check this offline.
- Test fixtures using "2.1.296" as an arbitrary version string need not change, as they are self-contained.
- Historical review documents stay at 2.1.296 as dated evidence.
- No checksum or other pin sits beside claude_version that needs updating.

## Acceptance criteria

1. [content] athena.toml sets claude_version = "2.1.299" and no other key in athena.toml changes.
2. [content] README.md and docs/design/athena-v0.md (setup and pin table lines) and docs/decisions.md line about the pin name 2.1.299 as the current pin.
3. [content] Review/acceptance documents under docs/design/reviews/ are unchanged, since they record what was tested at 2.1.296.
4. [behavior] Run `cargo test` and `cargo clippy` and `cargo fmt --check`: all pass with the new pin.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- README.md (modify)
- docs/design/athena-v0.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)

## Out of scope

- Rewriting historical review documents
- Changing Claude Code flags or behavior for the new version
- Running live model tests

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Found the pin in athena.toml and the docs quoting it by searching the repository.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused version bump; history and fixtures untouched.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: athena.toml is a governance file, so it is a named deliverable and criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Update every live reference to the pin, not just the config.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 65effd97cead → candidate 3c32c1850b86. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            | 4 ++++
 README.md                               | 2 +-
 athena.toml                             | 2 +-
 crates/athena-agent/src/claude.rs       | 4 ++--
 crates/athena-gate/src/landing_tests.rs | 2 +-
 crates/athena-gate/src/reconcile.rs     | 2 +-
 docs/decisions.md                       | 2 +-
 docs/design/athena-v0.md                | 8 ++++----
 docs/records.md                         | 2 +-
 9 files changed, 16 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (545 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
