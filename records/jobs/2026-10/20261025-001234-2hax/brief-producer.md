# Job 20261025-001234-2hax: Stop writing Athena- trailers in commits made after C0

## Request (verbatim)

> Priority: high
> Build job 7, part 1b, of the records-history plan (decision D111): stop writing Athena commit-message trailers in new commits once the cutover is in force. Depends on 20261025-001232-vesh (the cutover). Landings, interventions and records-only commits made after C0 carry no Athena- trailer lines; anything that still reads trailers does so only for commits up to C0.
> Acceptance:
> - A scratch-repository test shows a landing, an intervention and a records-only commit made after C0 with no Athena- trailers, all attributed by the records and passing athena check --history and --attribution.
> - Status and audit still read trailers on commits up to C0.
> Files: crates/athena-core, crates/athena-gate, crates/athena-cli, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Once the records branch shows C0 (the cutover), landings, interventions and records-only commits are written without Athena- trailer lines and attributed by records. Status and audit keep reading trailers, but only for commits up to C0. Audience: the maintainer and the verifier running athena check.

Assumptions:
- The cutover job (20261025-001232-vesh) has landed or its code is present, so C0 can be resolved from the landing record on athena/records.
- The writer decides at commit time whether the records branch shows C0. Before C0 shows, trailers are still written.
- The listed crates are the main sites. Trailer writers appear to be in athena-core land.rs, gh.rs and commits.rs, and athena-git lib.rs, so a few files outside the listed paths may change.

## Acceptance criteria

1. [behavior] Run cargo test: a scratch-repository test makes a landing, an intervention and a records-only commit after C0. It shows none of the three commit messages contains an 'Athena-' trailer line, and that records attribute all three.
2. [behavior] Run cargo test: in the same scratch repository, history check (athena check --history) and attribution check (athena check --attribution) both pass on the post-C0 commits.
3. [behavior] Run cargo test: a test shows status and audit still read trailers on commits up to and including C0. Trailer-bearing commits before C0 give the same results as before.
4. [behavior] Run cargo test: with no cutover_job set, or before its landing record exists, commits are still written with trailers and every existing test passes unchanged.
5. [content] CHANGELOG.md and docs/decisions.md describe the end of trailer writing after C0 (D111, job 7 part 1b) and name which readers still use trailers up to C0.
6. [behavior] cargo clippy and cargo fmt --check are clean, and cargo test passes.
7. [content] No governance files (athena.toml, ci.yml) change.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/commits.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Rewriting or removing trailers on commits up to C0
- Changing the cutover itself or athena.toml cutover_job
- CI workflow changes
- Deleting the trailer-reading code

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: scratch-repository test checks that trailers are absent and checks pass
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: behavior criteria use cargo test
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one focused change
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: all code is Rust
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

Run cargo test: a scratch-repository test makes a landing, an intervention and a records-only commit after C0. It shows none of the three commit messages contains an 'Athena-' trailer line, and that records attribute all three.
Run cargo test: in the same scratch repository, history check (athena check --history) and attribution check (athena check --attribution) both pass on the post-C0 commits.
Run cargo test: a test shows status and audit still read trailers on commits up to and including C0. Trailer-bearing commits before C0 give the same results as before.
Run cargo test: with no cutover_job set, or before its landing record exists, commits are still written with trailers and every existing test passes unchanged.
CHANGELOG.md and docs/decisions.md describe the end of trailer writing after C0 (D111, job 7 part 1b) and name which readers still use trailers up to C0.
cargo clippy and cargo fmt --check are clean, and cargo test passes.
No governance files (athena.toml, ci.yml) change.

## Report

End with the producer report as your structured output.
