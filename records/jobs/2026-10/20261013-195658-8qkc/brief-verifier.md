# Job 20261013-195658-8qkc: verify "Turn on merge_via_rest and fix the test that pinned it off"

## Request (verbatim)

> Retry 20261013-195040-vy1x: set [landing] merge_via_rest = true in athena.toml with the reason comment, as before. It failed because the test rest_merge_is_off_unless_the_config_turns_it_on (crates/athena-data/src/config.rs) asserts that the repository's own athena.toml has no [landing] table. Root cause: that test pins this repository's live setting instead of the default. Change the test to check the default on a config with the [landing] table removed (or absent), and assert that this repository's config now turns the merge on. Only athena.toml and that test change.

## Reading (accepted by the user)

Add a [landing] table with merge_via_rest = true and a reason comment to athena.toml. Rewrite rest_merge_is_off_unless_the_config_turns_it_on to check the default on a config with the [landing] table removed, and to assert this repository's config turns the merge on.

Assumptions:
- Other tests that append to REPO_CONFIG do not add a [landing] table (the grep shows only the one test does), so they are unaffected.
- The [landing] table can be placed anywhere valid in athena.toml, and removing it for the test can be done by text manipulation of the table (a helper that drops the [landing] header and its key lines).

## Acceptance criteria

1. [content] athena.toml contains a [landing] table with merge_via_rest = true, preceded by a comment giving the reason (owner set the repo squash-only with a ruleset on main and approved merging verified work once required checks pass). No other setting changes.
2. [content] The test no longer asserts REPO_CONFIG lacks [landing]; it builds a config with the [landing] table removed (not by appending a second table, which would be a duplicate) and asserts the default is off.
3. [content] The test asserts Config::parse(REPO_CONFIG) has landing.merge_via_rest true, and still checks that an explicit true and an empty [landing] table behave as before, using a stripped base.
4. [behavior] Run `cargo test --locked`; all tests pass, including config::tests::rest_merge_is_off_unless_the_config_turns_it_on.
5. [behavior] Run `cargo clippy --locked` and `cargo fmt --check`; both are clean.
6. [content] Only athena.toml and crates/athena-data/src/config.rs change.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any other athena.toml setting
- Other tests or code in config.rs
- Repository settings, rulesets, or CI workflows

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The test is changed to pin the default, not the live setting.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only athena.toml and the one test change.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The comment states the reason for an outside reader.

## The change

Base 0f1707901ede → candidate 37c60d149bfd. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |  4 ++++
 athena.toml                      |  6 ++++++
 crates/athena-data/src/config.rs | 23 ++++++++++++++++++-----
 3 files changed, 28 insertions(+), 5 deletions(-)
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
