# Job 20261013-054813-6nrv: verify "Enable skip_when_only for fmt and clippy checks in athena.toml"

## Request (verbatim)

> Turn on the prose-only check skip in athena.toml now that the build supporting it is adopted.

## Reading (accepted by the user)

Add skip_when_only = ["docs/", "*.md"] to the change capability's fmt and clippy checks in athena.toml. The test check keeps running on every change, because the xtask policy tests read docs/*.md and scan every tracked file, including .md files, for boundary terms. Update the CHANGELOG, which currently says enabling is a follow-up.

Assumptions:
- The user's reply is read as: tests must not be skipped for files a test reads. I checked xtask/tests/policy.rs: it reads docs/non-rust-files.md, docs/architecture.md and docs/enforcement.md, and the boundary scan reads every tracked file outside records/, including all .md files. So no prose pattern is safe for the test check, and I leave it unskipped.
- Clippy and fmt do not read prose, so skipping them on docs/ and *.md is safe. I did not check whether any crate uses include_str! on a .md file.
- The pattern syntax is from path_matches in crates/athena-core/src/checks.rs: 'dir/' matches a directory, '*.ext' matches any depth.

## Acceptance criteria

1. [content] In athena.toml, the [[capabilities.change.checks]] entries named fmt and clippy each have skip_when_only = ["docs/", "*.md"], and no other patterns.
2. [content] The change capability's test check has no skip_when_only key, and athena.toml has a comment explaining why: xtask/tests/policy.rs reads docs/*.md and scans every tracked file, so a prose change can fail a test.
3. [content] The [capabilities.study] checks and every other athena.toml key are unchanged.
4. [content] CHANGELOG.md under Unreleased says the key is now enabled for fmt and clippy only, and the earlier 'does not set the key' follow-up wording no longer reads as pending.
5. [behavior] Run `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`; all pass. The repository's config tests load athena.toml, so this shows it still parses.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/enforcement.md (modify)

## Out of scope

- The study capability's checks
- Changing the skip mechanism in Rust code
- Skipping the test check for any path
- records/ and the owner's files

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused config change plus a changelog note.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The skip is enforced by the runner's skip_note logic, which already has tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The athena.toml comment says what the key does and why the test check is excluded.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs test, clippy and fmt --check.

## The change

Base eadaa4769e24 → candidate 752398288261. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |  7 ++++---
 athena.toml                      |  5 +++++
 crates/athena-data/src/config.rs | 37 ++++++++++++++++++++++++++++++-------
 docs/enforcement.md              |  2 +-
 4 files changed, 40 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (316 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
