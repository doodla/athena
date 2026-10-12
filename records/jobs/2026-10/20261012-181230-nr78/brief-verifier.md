# Job 20261012-181230-nr78: verify "Make a history-exempt commit excuse the job records it adds"

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

## The change

Base fb0a14431f50 → candidate 04c624e7082c. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |   3 +
 crates/athena-gate/src/history.rs   | 126 ++++++++++++++++++++++++++++++++++++
 crates/athena-gate/src/invariant.rs |  33 ++++++----
 crates/athena-gate/src/jobs.rs      |   5 ++
 docs/decisions.md                   |  24 +++++++
 5 files changed, 179 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (267 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
