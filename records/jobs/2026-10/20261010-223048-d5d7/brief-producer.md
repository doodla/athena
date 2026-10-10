# Job 20261010-223048-d5d7: Fix the history check failure and gate it in the verified-landing path

## Request (verbatim)

> The last landing breaks athena check --history on our own history: 'the Athena-Job trailer names 20261010-132713-ezm8, but the commit adds the records of 20261010-131714-adcd (commit 64ecdc9)'. Fix it, and make sure a gate change can't land as verified while the full-history check fails.

## Reading (accepted by the user)

Commit 64ecdc9 carries an Athena-Job trailer for one job but adds another job's closing records, so the invariant's landing check rejects it in `athena check --history`. The reading is: find why the landing path built that commit, fix it, and make the full-history check part of what a landing must pass, so a gate change cannot land as verified while it fails.

Assumptions:
- History cannot be rewritten (the repository's rules say published history is never rewritten), so commit 64ecdc9 stays. The fix must make the history check pass on the current tip some other way.
- The in-process gate on the landing commit (run.rs land_or_block / build_commit) did not catch the mismatch that invariant.rs:255-265 flags in the history check. The producer must find out why, for example by trailers and records drawn from different jobs, or by a gate path that skips the landing rule.
- The fix for 'a gate change can't land as verified while the full-history check fails' is a mechanism, not prose. Candidates are an in-process history check on the candidate's tree at landing, or a check entry run before landing. The producer chooses and records the call.
- The tests live beside the code: the crates/athena-gate unit tests, and the CLI integration tests in crates/athena-cli/tests if the behavior reaches the CLI.

## Acceptance criteria

1. [content] The root cause of the mismatch between the Athena-Job trailer and the closing records in commit 64ecdc9 is identified in the change description, and the code path that produced it is fixed.
2. [behavior] Run `cargo test --locked`. A new regression test reproduces a landing whose Athena-Job trailer names a different job from the closing records it adds, and shows the in-process gate refuses it or the landing path can no longer produce it.
3. [behavior] Run `cargo run --locked -- check --history` from the repository root. It reports no violation for 64ecdc9 and exits 0. Any exemption or handling of that commit is explained in a code or doc comment.
4. [behavior] Run `cargo test --locked`. A new test shows that a change to the gate (crates/athena-gate) which makes the full-history check fail cannot reach a verified landing, because the landing path runs the history check on the resulting tree or tip and refuses.
5. [content] docs/enforcement.md (and CHANGELOG.md if it records behavior changes) name the new mechanism with its refusal and allowing tests. Any change to athena.toml is listed as a deliverable and named in a criterion.
6. [behavior] Run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Both are clean, and `cargo test --locked` passes.

## Expected deliverables

- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)

## Out of scope

- Rewriting or force-changing published history.
- Changing CI workflows, setup.sh, or toolchain, lint, and rustfmt configuration.
- Editing anything under records/.
- Redesigning the trailer format or the job-identification rule beyond what the fix needs.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests, plus Markdown docs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Regression tests for the mismatch and for the landing-time history gate; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The 'cannot land as verified while the history check fails' rule is enforced in code, not prose.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The root cause is cited from the code path and the failing message; enforcement.md lists refusal and allowing tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused fix: the root cause plus the landing-time history gate.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier can run, including `cargo run -- check --history`.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: I recovered the failing rule from invariant.rs:255-265 and the CI history job instead of asking.

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
