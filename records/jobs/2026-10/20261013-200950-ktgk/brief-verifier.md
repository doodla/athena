# Job 20261013-200950-ktgk: verify "Turn on landing.merge_via_rest in athena.toml; fix its test; log it"

## Request (verbatim)

> Retry 20261013-195658-8qkc: same change (merge_via_rest = true with the reason comment, and the test checks the default on a stripped config and that this repository turns the merge on). My earlier 'only athena.toml and that test' was too strict: a CHANGELOG.md entry under 'Changed' is wanted too. Nothing else changes.

## Reading (accepted by the user)

Add [landing] merge_via_rest = true to athena.toml with a reason comment. Rewrite the config.rs test to check the default on a config with the [landing] table stripped, and to assert this repository's config turns the merge on. Add one CHANGELOG.md entry under Unreleased 'Changed'.

Assumptions:
- The Unreleased section has no '### Changed' heading yet (the only ones are at lines 58 and 248, in released sections), so the producer adds one below 'Added'.
- The test may strip the table by removing the [landing] section from REPO_CONFIG text, or use a config without it.
- The other existing assertions (explicit true turns it on, empty table is off) stay.

## Acceptance criteria

1. [content] athena.toml has a [landing] table with merge_via_rest = true, preceded by a comment giving the reason.
2. [content] The test in crates/athena-data/src/config.rs derives a stripped config with no [landing] table from REPO_CONFIG and asserts merge_via_rest is false on it.
3. [content] The test asserts that parsing this repository's athena.toml gives merge_via_rest == true, and no longer asserts that REPO_CONFIG lacks [landing].
4. [content] CHANGELOG.md gains an entry under '## [Unreleased]' in a '### Changed' subsection (not 'Added') describing the setting change; no other CHANGELOG lines change.
5. [behavior] Run `cargo test -p athena-data`: rest_merge_is_off_unless_the_config_turns_it_on and the other tests pass.
6. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean.
7. [content] No files other than athena.toml, crates/athena-data/src/config.rs and CHANGELOG.md change.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any other file, including docs, README and workflows
- Changing the default of merge_via_rest in code
- Other settings in athena.toml

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The test pinned the live repo setting instead of the default; it is changed to test the default on a stripped config.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change over three files.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces both the default and this repo's setting.

## The change

Base 77601bb20c5a → candidate 193ff138f2c8. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |  3 +++
 athena.toml                      |  5 +++++
 crates/athena-data/src/config.rs | 15 ++++++++++-----
 3 files changed, 18 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (376 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
