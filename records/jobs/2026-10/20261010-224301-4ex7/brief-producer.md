# Job 20261010-224301-4ex7: Make history check pass on 64ecdc9; verify gate changes with their own g

## Request (verbatim)

> athena check --history now rejects commit 64ecdc9 ('the Athena-Job trailer names 20261010-132713-ezm8, but the commit adds the records of 20261010-131714-adcd'); make it pass on our real history, and make a gate change unable to land as verified while the candidate's own gate fails the history check. A first try was rejected because: it ran the old compiled-in gate, not the candidate's; it never found why that commit carries another job's records; its acceptance command didn't run; it exempted 64ecdc9 by a 7-character prefix.

## Reading (accepted by the user)

Find out why commit 64ecdc9 holds job adcd's records under job ezm8's trailer, and fix the cause or exempt that one commit by its full 40-character id. Also make verification run the candidate's own freshly built gate on `check --history`, so a gate change whose own gate fails that check cannot land as verified. Provide an acceptance command that really runs.

Assumptions:
- The mismatch is real history, not a false positive. Its cause is found by inspecting git show for 64ecdc9 and the code that stages job records at landing (crates/athena-core/src/run.rs, land.rs, commits.rs).
- 'Candidate's own gate' means the athena-gate code in the candidate tree. Verification must build it and run `check --history` through it, for example via cargo run in the candidate worktree, instead of calling the compiled-in athena_gate::history::check_history.
- The first try's four faults are requirements: run the candidate's gate, find the root cause, give an acceptance command that runs, and use no prefix exemption.
- The invariant.rs landing() check stays as it is. Config validation already refuses exemptions shorter than a full id.

## Acceptance criteria

1. [content] The change records, in docs/decisions.md or the commit message, why commit 64ecdc9 carries the records of job 20261010-131714-adcd while its Athena-Job trailer names 20261010-132713-ezm8, citing the evidence (git show output, code path). It says whether a bug in the landing or records-staging code caused it.
2. [content] If the cause is a code bug, it is fixed in the code with a regression test that fails without the fix.
3. [behavior] Run `cargo run -- check --history` from the repository root. It exits 0 and reports no violation for the 64ecdc9 commit. Check that the command actually builds and runs, and say so in the output.
4. [content] If the real commit needs an exemption, athena.toml [history] exempt adds the full 40-character id of 64ecdc9 (found with git log or git show), never a prefix. athena.toml is named as a governance deliverable. If the cause is a bug that can be fixed without an exemption, no exemption is added.
5. [content] Verification of a candidate that changes the gate (crates/athena-gate, or the check, land or verify paths that call it) runs the history check with the candidate's own freshly built gate, not the compiled-in one. A candidate whose gate fails the history check cannot land as verified.
6. [behavior] Run `cargo test` and `cargo clippy --all-targets -- -D warnings`. Both pass. New tests cover (a) a gate-changing candidate whose gate fails the history check being refused and (b) a passing one being accepted.
7. [behavior] Run `cargo fmt --check`. It passes.
8. [content] docs/enforcement.md gets a row for the new mechanism, linking its blocking and passing tests. CHANGELOG.md notes the change.

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/commits.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/enforcement.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Rewriting git history.
- Exempting any other commit.
- Changing the secrets, personal-data or records rules.
- Changing CI workflows, unless the root cause requires it.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests, plus the existing TOML and Markdown.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the candidate-gate history check, and cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule that a gate change cannot land while its own gate fails the history check is enforced in code and listed in docs/enforcement.md.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The root-cause claim cites git show output and code paths.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by running the history check command.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the cause fix, the candidate-gate check, and at most one exemption.

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
