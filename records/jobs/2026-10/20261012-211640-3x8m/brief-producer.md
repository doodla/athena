# Job 20261012-211640-3x8m: Make study selectable and let it read firstmate at a pinned SHA

## Request (verbatim)

> the reader still offers only 'change' (study isn't in the catalog), and the study can't fetch firstmate yet; make study selectable and able to read firstmate at a pinned SHA

## Reading (accepted by the user)

Add a [capabilities.study] section to athena.toml so the reader's catalog and schema offer `study`. Add Rust code that checks out https://github.com/kunchenguid/firstmate read-only at a full 40-character SHA, which is firstmate's main head resolved at study time and recorded. Running firstmate stays out of scope.

Assumptions:
- The SHA is resolved from firstmate's current main by a one-time lookup recorded in config. Athena code then enforces the full 40-character SHA at read time. If the producer cannot reach the network, the SHA is left to the user and the code still enforces the pin.
- The existing athena-cap-study crate already registers Study in wiring.rs, so the work is config plus a small materialisation module, not a new capability.
- Materialisation uses the git CLI or an existing crate, per buy-first, and the choice is recorded.
- Tests use a local fixture repository, never the network.

## Acceptance criteria

1. [content] athena.toml has a [capabilities.study] section (producer_commands, verifier_commands, env, checks) and is named as a governance edit; [capabilities.change] is unchanged. The verifier grant has no write commands.
2. [behavior] Run `cargo test --locked`: a new test loads the repository's athena.toml and shows the reader's catalog offers both `change` and `study` and the reader schema's capability enum is ["change","study",null].
3. [content] Config records the study source as https://github.com/kunchenguid/firstmate plus a full 40-character SHA. The SHA is firstmate's main head as resolved at study time and recorded in the config and CHANGELOG. A short, abbreviated, or missing SHA is rejected.
4. [behavior] Run `cargo test --locked`: tests using a local fixture git repository (no network) show the source is materialised at exactly the pinned SHA, and a missing, short, or non-matching SHA is an error.
5. [content] The fetch and checkout happen in Athena's Rust code, not through an agent command. The studied tree is read-only to agents, and the study grants add no network command.
6. [content] The study card, CHANGELOG.md, and docs/decisions.md (a new D-entry listing alternatives) record the call and say running firstmate remains out of scope.
7. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`: both are clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cap-study/src/source.rs (create)
- crates/athena-cap-study/Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-cli/src/wiring.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/architecture.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)

## Out of scope

- Running or building firstmate
- Any change to [capabilities.change]
- Network access from agent commands
- Changes to the study model or records format beyond the source pin

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The materialiser and tests are Rust; the only non-Rust edits are TOML and Markdown.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the catalog and the SHA pin.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The full-SHA pin is enforced in code, not prose.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Prefer the git CLI or an existing crate for fetching over a custom implementation.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Only the public repository URL and a SHA are committed.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: enable study and add the pinned read.

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash in code, docs, or design-revision text (BRIEF.md included); describe the content instead.

## Report

End with the producer report as your structured output.
