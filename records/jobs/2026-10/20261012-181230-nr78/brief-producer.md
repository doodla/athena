# Job 20261012-181230-nr78: Make a history-exempt commit excuse the job records it adds

## Request (verbatim)

> the ee4b0dd exemption (20261012-180308-knqc) didn't make the history check pass: 9 per-record violations remain for the six jobs whose records arrived in ee4b0dd ('outcome says landed, but this commit is not a valid landing' and 'score.json does not follow'); make an exempt commit excuse its records too

## Reading (accepted by the user)

The exemption is honored only in invariant::check; jobs::check (crates/athena-gate/src/jobs.rs) still flags the 'landed' outcomes and score.json records an exempt commit adds. I read the request as making jobs::check skip exempt commits, using the same exempt list as the invariant check, with tests and a decision note.

Assumptions:
- Records-level violations come from jobs::check, which lacks the exemption early-return that invariant::check has.
- The exemption excuses only the exempt commit's own added records, not records of later commits.
- Other checks (content, secrets) are unchanged; only the per-record job-records check is changed.
- docs/decisions.md has a numbered decision log where the next entry follows D67.

## Acceptance criteria

1. [content] jobs::check in crates/athena-gate/src/jobs.rs returns no violations for a commit on the [history] exempt list (the history walk's tip list; the parent's list for a per-commit check), using the same rule as invariant::check.
2. [content] The exempt-commit test is shared by invariant::check and jobs::check (one helper), not duplicated.
3. [behavior] A new test in athena-gate builds a history where an exempt commit adds an outcome saying landed and a score.json that would otherwise be flagged; run `cargo test -p athena-gate` to see the history check report no JobRecords violation for it.
4. [behavior] A test shows a non-exempt commit adding the same records is still flagged, and a commit after the exempt one is not excused; run `cargo test`.
5. [behavior] `cargo clippy` is clean, `cargo fmt --check` passes, and `cargo test` passes.
6. [content] A decision entry in docs/decisions.md (and a CHANGELOG line) records that an exemption also excuses the records the commit adds, citing D66/D67.

## Expected deliverables

- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Verifying that ee4b0dd is in main's history
- Changing the exempt list in athena.toml
- Altering other history checks (secrets, content, configuration)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests for exempt and non-exempt commits adding job records.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the gate crate plus a decision note.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and decision entry explain why records follow the exemption.

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
