# Job 20261013-141110-exjw: Add athena land --pr with opt-in REST merge, no stray emails

## Request (verbatim)

> Retry 20261013-135309-9e8t, keeping its accepted correction: merge via REST only when an opt-in setting (off by default) is on; otherwise stop at an open, green PR and land on observing the merge. No email addresses outside the allow-list.

## Reading (accepted by the user)

Retry of the `athena land --pr` slice: push the job's branch, open the PR, wait for green checks, using only `gh api` (REST). The merge call is made only when a new opt-in setting (default off) is on. Otherwise Athena stops at an open, green PR and later lands the job by observing the merge with the existing merged-content check. No email address outside the allow-list appears in any file.

Assumptions:
- The accepted correction is the one in the request: the merge is gated on an opt-in setting that is off by default. I could not read the earlier job's record, so I take this wording as the whole correction.
- The earlier failure was an email-like literal in gh.rs, probably in a test fixture or a git author string. The retry avoids any email-shaped text.
- The setting lives in the user-facing config parsed by crates/athena-data/src/config.rs, with a serde default of false, so athena.toml (a governance file) need not change.
- Tests use fakes behind the existing Host and Content traits, so the network is never touched. The `gh` binary is invoked only through a thin command-runner trait.
- The default path ends at an open, green PR. Landing is recorded on a later observation, which reuses `check_merged` and `apply_outcome` from pr.rs.

## Acceptance criteria

1. [content] `athena land --pr <job>` exists in the CLI. It pushes only the job's work branch (through `check_push`, never forced), opens a PR, reads check status, and stores a `PrLink` in the job's state directory.
2. [content] The host client in crates/athena-core/src/gh.rs runs only `gh api` REST calls (paths like repos/{owner}/{repo}/pulls). It has no `gh pr` subcommands, no `graphql` endpoint, and no GraphQL query text.
3. [content] A new boolean setting enables merging via REST. It defaults to false when absent from the config. athena.toml is not changed, or if it is, the criterion naming it says why.
4. [behavior] Run `cargo test`. A test with a fake host shows that with the setting off (the default) no merge request is issued and the command ends with an open, green PR. A second test shows that with the setting on the REST merge call is issued.
5. [behavior] Run `cargo test`. A test shows that when the PR is later seen merged, `check_merged` runs and the job is landed on a matching fingerprint, or `merged-unverified` otherwise. The merged commit is found by PR number, never by reading trailers.
6. [behavior] Run `cargo test`. A test shows that red or pending checks never trigger a merge, even with the setting on.
7. [content] No email address outside the `email_allow` list in athena.toml appears in any changed file, including tests and fixtures. Use `noreply@`, `@example.com` or similar, or avoid emails entirely, e.g. no `user@host` strings in gh.rs.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. All pass.

## Expected deliverables

- crates/athena-core/src/gh.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-data/src/config.rs (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Stealth mode and PRs in repositories Athena does not own.
- Any GraphQL use, including `gh pr merge --auto` and other `gh pr` subcommands.
- Auto-merge settings on the host.
- Changes to the email allow-list or other governance files.
- Squash-chain or multi-job PR handling.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust. gh is called as an external program and no scripts are added.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-host unit tests cover the setting off and on, red checks, push refusal and the merged-content outcomes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused slice: the command, the REST client and the setting. Stealth mode is left out.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No email addresses outside the allow-list, and no tokens or credentials in fixtures.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The merge gate is a code-level setting check with tests, not prose.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the installed `gh` CLI rather than adding a GitHub HTTP crate.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs fmt, clippy and tests.

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
