# Job 20261013-034832-767t: Study firstmate's work handling (M1) pinned at 6407052f

## Request (verbatim)

> Retry 20261013-033155-kmh0.

## Reading (accepted by the user)

Retry the M1 study of firstmate's work handling, pinned to the full commit SHA 6407052f4c521e39401ce2305dfa45d201249827 that the user gave (pinned-source/ itself has no SHA). The study must carry that SHA in pin.json and every citation, and its validation must be reproducible by a kept test. It must also fix the earlier minor flaws.

Assumptions:
- The pin SHA is 6407052f4c521e39401ce2305dfa45d201249827, as the user stated; pinned-source/ contents are read as that commit.
- The studies/ tree from the failed attempt is absent, so the study is created from scratch.
- The study follows the typed records in crates/athena-cap-study (pin, claims, rationales, model, validation).
- The earlier tree's docs/non-rust-files.md studies/** row is still needed for the new JSON files.

## Acceptance criteria

1. [content] studies/firstmate-work-handling/pin.json holds the full SHA 6407052f4c521e39401ce2305dfa45d201249827; no placeholder such as UNRESOLVED-PIN-SHA appears anywhere in the study.
2. [content] Every citation in claims.json and rationales.json carries that SHA, an existing path and line range, and a verbatim quote from the pinned tree.
3. [behavior] Run cargo test: a kept test in the repository validates studies/firstmate-work-handling, including the wrong-SHA and quote-match rules, and passes.
4. [content] Every claim is tagged mechanism or prose-only, never 'not examined'. Model transitions with guard 'none' do not say code evaluates a guard.
5. [content] docs/non-rust-files.md lists the studies/** JSON files with their reason, and CHANGELOG.md names the change.
6. [behavior] Run cargo fmt --check and cargo clippy; both report no problems.

## Expected deliverables

- studies/firstmate-work-handling/pin.json (create)
- studies/firstmate-work-handling/claims.json (create)
- studies/firstmate-work-handling/rationales.json (create)
- studies/firstmate-work-handling/model.json (create)
- studies/firstmate-work-handling/validation.json (create)
- crates/athena-cap-study/src/rules.rs (modify)
- docs/non-rust-files.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the study capability's design or rules beyond what a kept validation test needs.
- Studying systems other than firstmate's work handling.
- Editing records/, athena.toml, defaults.toml, or CI configuration.

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim cites the pinned SHA, path, line range and a verbatim quote.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A kept test validates the study against real pinned-source text; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Validation must be reproducible by the verifier running cargo test.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The JSON study files are non-Rust, so each is listed with its reason in docs/non-rust-files.md.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Every claim's enforcement is tagged mechanism or prose-only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One study plus the minimum test and doc changes.

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
