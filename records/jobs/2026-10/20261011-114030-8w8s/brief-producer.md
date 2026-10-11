# Job 20261011-114030-8w8s: Exempt the ezm8 second root commit in the history check

## Request (verbatim)

> Get athena check --history passing on our real history so the squash-support change can land. The only failure is 64ecdc9aa4fd24cbe4d4884c864d7e9041585c3d, a parentless second root on main that Athena wrote during job ezm8. Exempting it needs the queued fix that lets a [history] exempt entry added later take effect (20261010-163553-tpkz). Don't change CI. Gate changes must still pass the history check with their own gate.

## Reading (accepted by the user)

Add the full id of the parentless second root 64ecdc9aa4fd24cbe4d4884c864d7e9041585c3d to [history] exempt in athena.toml, so athena check --history passes. I assume the queued fix tpkz lands first, and I leave CI and gate code alone.

Assumptions:
- The queued fix 20261010-163553-tpkz lands before this job is dispatched, so an exempt entry added later takes effect; this job does not reimplement it.
- The whole change is adding one full commit id to [history] exempt in athena.toml.
- The exempt entry must name a commit already in history when added; 64ecdc9 is on main, so it qualifies.
- If tpkz has not landed, the check will still fail and the verifier should report that rather than the producer altering gate code.

## Acceptance criteria

1. [content] athena.toml's [history] exempt list keeps 49c6ac1524677b8091081115fb5d410ed6d2b2ea and adds the full 40-character id 64ecdc9aa4fd24cbe4d4884c864d7e9041585c3d, with a comment saying it is a parentless second root Athena wrote during job ezm8.
2. [content] Only athena.toml (and optionally CHANGELOG.md) change; no CI workflow, .gitattributes, or other governance file is touched.
3. [behavior] Run `cargo run -- check --history` (or the repository's built athena binary via cargo run) from the root: it reports no violations, and 64ecdc9 is no longer flagged.
4. [behavior] `cargo test`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo fmt --all --check` pass, including the config test that exempt entries are full commit ids.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the later-added-exemption fix (20261010-163553-tpkz)
- Changing CI workflows
- Rewriting or squashing history
- The squash-support change itself

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One-line config change; the gate fix stays in its own queued job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The exemption is enforced by the history check, and the config parser requires a full commit id.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verification runs athena check --history on the real history.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The commit id and the file to edit came from the request and athena.toml.

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
