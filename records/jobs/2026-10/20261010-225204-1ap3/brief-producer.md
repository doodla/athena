# Job 20261010-225204-1ap3: Make history check handle stray second root and run via athena-cli

## Request (verbatim)

> Third try at 20261010-224301-4ex7. Cause found: 64ecdc9aa4fd24cbe4d4884c864d7e9041585c3d has no parent (git log --max-parents=0 shows two roots on main); Athena's commit path wrote it during job ezm8, and ff2aafa built on it. Make the history check handle it, make the acceptance command actually run, and keep the rule that a gate change is judged by its own gate.

## Reading (accepted by the user)

The full-history check must not fail on the parentless commit 64ecdc9a (and ff2aafa built on it), by exempting it by full id in athena.toml and testing a second-root history. A gate change must pass the history check under its own candidate gate, and acceptance uses `cargo run -p athena-cli -- check --history`, since bare `cargo run` cannot pick a binary.

Assumptions:
- Exempting the stray root by full id in athena.toml [history] exempt is the simplest handling; the exemption rule (D60) requires the exempt commit to already be in the adding commit's history, which holds for the tip.
- ff2aafa needs its own exemption only if the check flags it independently of its root; the producer should run the history check and add only the ids actually flagged.
- The history check must judge a commit that changes the gate (athena-gate or athena.toml) with that commit's own candidate gate, and the producer should find how Judged::new picks its configuration and make the history path agree with the landing path.
- The runnable acceptance command is `cargo run -p athena-cli -- check --history`; the binary is named athena in crates/athena-cli, and the workspace has several binaries (xtask, athena), so bare `cargo run` fails.

## Acceptance criteria

1. [content] athena.toml `[history] exempt` lists the full id 64ecdc9aa4fd24cbe4d4884c864d7e9041585c3d (and ff2aafa's full id only if the check flags it on its own), each with a comment saying it is a stray second root written during job ezm8. athena.toml is a governance file named in this criterion.
2. [content] A test in crates/athena-gate/src/history.rs builds a history with a second parentless commit and a child built on it. It shows check_history handles them: an exempt root is not flagged, and a non-exempt second root is still flagged.
3. [content] A test shows a commit that changes the gate is judged by its own candidate gate in the history check, and that a hand change on a non-exempt commit is still flagged. No rule is weakened for ordinary commits.
4. [behavior] Run `cargo test -p athena-gate`: the new tests and all existing history, exemption and landing tests pass.
5. [behavior] Run `cargo run -p athena-cli -- check --history` from the repo root: it starts (no 'could not determine which binary' error) and does not report 64ecdc9aa4fd or ff2aafa as violations.
6. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`; both are clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/testutil.rs (modify)
- CHANGELOG.md (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Rewriting or removing the stray commits from history
- Changes to records/ or the owner's files
- Changing how Athena's commit path writes commits, unless the producer finds the root cause is in this repository's code and the fix is small
- Other CI workflow changes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in history.rs cover a second root and a child built on it; cargo test, clippy and fmt must be clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name commands the verifier can run, using -p athena-cli.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The exemption lives in athena.toml, which the gate enforces, rather than in prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the exemption, the candidate-gate judgment, and tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The athena.toml comment says what the stray root is and why it is exempt.

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
