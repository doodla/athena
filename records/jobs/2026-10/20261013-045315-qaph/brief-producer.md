# Job 20261013-045315-qaph: Skip cargo checks the change cannot affect

## Request (verbatim)

> Doc-only jobs spend most of their time on the cargo checks (hzrp: 184s of 311s). Skip checks that can't be affected by the change.

## Reading (accepted by the user)

Add an optional per-check `paths` list (globs/prefixes) to the check config. Athena skips a check when no changed path matches. A doc-only change then skips fmt, clippy and test. Skipped checks are recorded as "not run" and count as passed, like the nested-run case.

Assumptions:
- Mechanism: an optional `paths` filter on each check in athena.toml; with no filter a check always runs.
- The cargo checks run when any changed path is *.rs, Cargo.toml, Cargo.lock, rust-toolchain.toml, rustfmt/clippy config, or lies under crates/ or xtask/. Other changes (e.g. *.md, docs/) skip them.
- Conservative default: if the change list is empty or a path is unknown, run everything.
- Skipped checks appear in the check records as passed with a 'not run: no changed path can affect it' note, reusing the nested-run skip shape.
- The test check (kind=test) zero-test rule is unaffected because skipped checks are not run.
- Both change and study capability check tables get the filter.
- Include tests files (include_str!, CARGO build scripts, .github workflows are not cargo inputs) conservatively: any non-.md path outside docs/ counts as affecting.

## Acceptance criteria

1. [content] CheckConfig in crates/athena-data/src/config.rs has an optional `paths` field (default: always run), and CheckSpec carries it through both capabilities' checks().
2. [content] athena.toml lists `paths` for the fmt, clippy and test checks of the change and study capabilities, so Rust, Cargo and toolchain files trigger them and docs-only changes do not.
3. [content] The check loop in crates/athena-core/src/run.rs skips a check when no changed path matches, records it as passed with a 'not run' note, and runs every check when the change list is empty.
4. [behavior] Run `cargo test --locked`: new tests pass showing a docs-only change list skips cargo checks, a .rs or Cargo.toml change runs them, and an unfiltered check always runs.
5. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.
6. [content] athena.toml still parses under deny_unknown_fields, and a test loads the repo's athena.toml and checks the filters on the shipped checks.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- defaults.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Caching or parallelizing cargo checks
- Skipping the verifier agent for doc-only jobs
- Changing check timeouts or the CI workflow
- Changing which checks exist or their commands

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust in athena-data config and athena-core run; no non-Rust files added.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for path matching and for the skip in the run loop; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The skip rule is configured in athena.toml and enforced in code, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the check-selection step only.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test and clippy; the skip logic is covered by tests.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
