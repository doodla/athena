# Job 20261010-163553-tpkz: verify "Make history check honor [history] exempt entries added later"

## Request (verbatim)

> Let a [history] exempt entry added later take effect: it never exempts any commit today, since each commit is judged by its parent's configuration, so an adopter cannot excuse history from before Athena (docs/design/reviews/athena-v0-acceptance-3.md).

## Reading (accepted by the user)

Today each commit is judged under its parent's config, so an exempt entry added later never covers the commit it names. I read the request as: in `athena check --history`, take the exemption list from the tip's config, as the `[secrets]` and `[personal_data]` allow-lists already are. The validity checks stay: full ids only, and only commits already in history.

Assumptions:
- Only the full-history check (`athena check --history`, CI) uses the tip's exempt list. Per-commit checks (pre-commit and similar) keep using the parent's config, since there is no tip there.
- The mechanism is a new exempt-list field on `HistoryState` in `crates/athena-gate/src/invariant.rs`. `check_history` fills it from `tip_config`, and `invariant::check` consults it when history state is present and falls back to `judged.config.history.exempt` otherwise.
- When the tip's config does not parse, the built-in config's list applies, as `check_history` already does for the other tip-level settings.
- The existing validation (full commit ids in `config.rs`; new entries must name a commit already in history in `content.rs`) is unchanged. This is what stops an exemption covering a later commit.
- Probe 4 in `docs/design/reviews/athena-v0-acceptance-3.md` (an entry added after the fact has no effect) becomes an effective exemption. The review file is a historical record, so it is not edited.

## Acceptance criteria

1. [behavior] A new `athena-gate` test builds a history with a hand-change commit (no Athena trailers). A later commit adds that commit's full id to `[history] exempt` through a declared intervention. `check_history` at the tip then reports no invariant violation for the hand-change commit. Run `cargo test -p athena-gate`.
2. [behavior] The existing tests still pass: `a_short_history_exemption_does_not_hide_a_hand_change`, `an_exemption_can_only_name_a_commit_already_in_history`, `exempt_commits_are_skipped`, and the config test that rejects short exempt ids. Run `cargo test --workspace`.
3. [behavior] A test shows an exemption still does not cover a commit made after the entry, or a commit not in history. Run `cargo test -p athena-gate`.
4. [behavior] `cargo clippy --workspace --all-targets` and `cargo fmt --check` are clean.
5. [content] The comment above the exemption check in `invariant.rs` and the doc comment on `HistoryState` state that the history check reads the exempt list from the tip's config, and say why.
6. [content] No governance files are changed. `athena.toml` is not edited, and no records are touched.

## Expected deliverables

- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/content.rs (modify)

## Out of scope

- Changing exempt-entry validation (full ids, must be an ancestor)
- Changing per-commit (pre-commit) judging by the parent's config
- Editing the acceptance review documents or records/
- Editing athena.toml or other governance files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests for the new behavior; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the gate crate's history and invariant modules.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The intended mechanism was recovered from the review file and the code, not asked.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checked with cargo test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments and docs explain why the tip's config is used.

## The change

Base 2052cf612275 → candidate 5817b35cefa5. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |   5 ++
 crates/athena-gate/src/history.rs   | 101 +++++++++++++++++++++++++++++++++++-
 crates/athena-gate/src/invariant.rs |  18 +++++--
 docs/decisions.md                   |  16 ++++++
 docs/enforcement.md                 |   2 +-
 5 files changed, 136 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (218 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
