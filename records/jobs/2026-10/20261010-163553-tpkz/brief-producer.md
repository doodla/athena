# Job 20261010-163553-tpkz: Make history check honor [history] exempt entries added later

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

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Report

End with the producer report as your structured output.
