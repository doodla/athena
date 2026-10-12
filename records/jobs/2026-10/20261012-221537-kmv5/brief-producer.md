# Job 20261012-221537-kmv5: Pin the study instance's repo to a SHA and mount it read-only

## Request (verbatim)

> now the firstmate pinning: when a study starts, Athena's own code resolves firstmate's main (https://github.com/kunchenguid/firstmate) to a SHA, records it, and puts the pinned tree read-only inside the agents' working directory; test that the core calls it

## Reading (accepted by the user)

When a study job starts, Athena's Rust core resolves the study instance's configured repo URL and ref (firstmate's main is only the first instance's config, per BRIEF S1) to a commit SHA, records it, and exports that tree read-only into the agents' working directory. A core test with a fake pinner shows the study dispatch calls it with the configured values.

Assumptions:
- athena.toml has no study-instance setting yet, so the change adds an optional instance table (repo URL, ref) under the study capability config, with firstmate/main as its first value.
- athena-core currently does not mention study in its sources, so the change adds a small pinning seam (trait plus real git implementation) called from the core's job start for the study capability only.
- The real implementation uses git through the existing athena-git crate (resolve with ls-remote, export with a read-only checkout), not a new crate; tests use a fake pinner and a local temporary git repo, never the network.
- The pin record is stored with the job's other records using the existing record types, extended with url, ref and sha.
- The tree is placed at a fixed subdirectory of the agents' working directory, with write bits cleared on every file and directory.

## Acceptance criteria

1. [content] The study capability's config in athena.toml (and the Config type in athena-data/src/config.rs) carries the instance's repo URL and ref; no Rust source hard-codes the firstmate URL or main outside tests/fixtures.
2. [content] A pinning seam exists (trait with a real git-backed implementation) that resolves a URL and ref to a full 40-hex SHA and exports the tree at that SHA into a directory.
3. [content] The core's job start calls the pinner only when the capability is study, before the producer runs, and records URL, ref and SHA in the job's records (record type updated as needed).
4. [content] The pinned tree is placed inside the agents' working directory with all files read-only, and the study producer/verifier grants give no write access to it.
5. [behavior] Run `cargo test -p athena-core`: a test with a fake pinner asserts the study dispatch calls it once with the URL and ref from the study config (not hard-coded) and records the returned SHA.
6. [behavior] Run `cargo test`: a test shows the change capability never calls the pinner.
7. [behavior] Run `cargo test`: an export test on a local temporary git repo checks the tree matches the pinned commit and no file has write permission bits.
8. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test` over the workspace: all pass cleanly.
9. [content] CHANGELOG.md has an entry, and a design/decisions doc records the git-versus-crate choice (buy-first).

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/pin.rs (create)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- CHANGELOG.md (modify)
- docs/design.md (modify)

## Out of scope

- Pinning Hermes Agent or Athena itself beyond what the config makes possible
- Running the actual study or any network call in tests
- Changing change-capability behavior
- Tag-based pinning of Athena's own release

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pinning, config and tests are all Rust.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use git via athena-git rather than a new crate, recorded in the decisions doc.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-pinner dispatch test, change-negative test and export test; clippy and fmt clean.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Read-only is enforced by file mode bits and the grants, not prose.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The pin is recorded as URL, ref and SHA in job records.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: config, pinner, dispatch hook, record.

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
