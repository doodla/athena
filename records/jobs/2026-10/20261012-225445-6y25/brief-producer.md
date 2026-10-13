# Job 20261012-225445-6y25: Configure firstmate as the first study instance in athena.toml

## Request (verbatim)

> after that: configure firstmate (https://github.com/kunchenguid/firstmate, main) as the first study instance and study its work handling (M1)

## Reading (accepted by the user)

Set athena.toml's [capabilities.study] repo_url to https://github.com/kunchenguid/firstmate and repo_ref to main, replacing the commented example. The M1 work-handling study runs as a separate study job afterward, because it reads this config from the base commit.

Assumptions:
- This job only edits the configuration; the M1 study is a separate study job run afterward.
- The repo_url form is the plain https GitHub URL without .git.
- Editing athena.toml is permitted because the request names it as the deliverable.

## Acceptance criteria

1. [content] athena.toml [capabilities.study] has uncommented repo_url = "https://github.com/kunchenguid/firstmate" and repo_ref = "main"; the example placeholder lines are gone.
2. [content] A short comment notes this is the first study instance and that Athena pins the ref to a commit before dispatch.
3. [content] No other key in athena.toml changes, and no other file changes except optionally a CHANGELOG.md line.
4. [behavior] Run `cargo test --locked` and `cargo clippy --all-targets --locked -- -D warnings`: both pass.
5. [behavior] Run `cargo fmt --all --check`: it passes.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Running the M1 work-handling study itself
- Copying any firstmate content into this repository
- Changing study commands, env, or checks

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the target keys from the commented example in athena.toml.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused config edit.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Only a public repo URL is added.

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
