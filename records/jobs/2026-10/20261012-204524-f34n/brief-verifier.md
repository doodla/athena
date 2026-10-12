# Job 20261012-204524-f34n: verify "Form squash landing chains from landed jobs only"

## Request (verbatim)

> a simulated squash of this branch fails: 'jobs 20261012-201349-pf6d and 20261012-201841-1r97 both follow 20261012-195832-rrf8' (pf6d was rejected, 1r97 is its retry), then 1r97 and 20261012-203446-bkk7 count as invalid landings; only landed jobs should form the chain

## Reading (accepted by the user)

The gate's squash check treats every job whose closing records a commit adds as part of the chain, including rejected jobs such as pf6d. Their retry 1r97 then looks like a fork. Only jobs whose outcome is landed should be chained and judged.

Assumptions:
- `closing_jobs` in crates/athena-gate/src/invariant.rs collects rejected jobs as well as landed ones. That is the cause of both errors. I read only that function and `chain_problem`.
- A rejected or otherwise non-landed job in a squash is ignored for the chain. A job's previous_job may point at a rejected job. The chain head then has a previous job that is not in the commit and did not land on the parent. The check must allow that case without opening a hole for unlanded work.
- The records of a rejected job still ship in the squash. The gate must not require them to chain.
- Whether a job is landed is read from its outcome.json or outcome-landed.json in the commit tree, as `landed_at` does.

## Acceptance criteria

1. [behavior] A new test in crates/athena-gate/src/landing_tests.rs builds a squash with a rejected job A, its landed retry B (previous_job = A) and a landed job C that follows B. The gate accepts it. Run `cargo test -p athena-gate`.
2. [behavior] A squash of landed jobs that really fork is still refused with 'both follow'. The existing fork tests pass. Run `cargo test -p athena-gate`.
3. [content] The code that selects the jobs a commit lands excludes jobs whose recorded outcome is not landed. A doc comment on it says why.
4. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.
5. [content] docs/design/squash-chain-landing.md or docs/decisions.md records the rule that only landed jobs form the chain.

## Expected deliverables

- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- docs/design/squash-chain-landing.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how athena writes retries or previous_job.
- Changing the rule that a squash cannot span a rules file.
- Re-scoring or intervention handling (the earlier rrf8 fix).
- Changing the policy test on docs wording.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add gate tests for a squash holding a rejected job, its landed retry and a follow-up job. Run cargo test and clippy.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Change the gate's job selection and its tests. Touch docs only where the design states the rule.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the cause from the error text in invariant.rs and landing_tests.rs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Add a short note in the design doc or decisions.md, written for an outside reader.

## The change

Base 7e409779529a → candidate a078c637c6da. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |  4 +++
 crates/athena-gate/src/invariant.rs     | 29 ++++++++++++++---
 crates/athena-gate/src/landing_tests.rs | 58 ++++++++++++++++++++++++++++-----
 docs/decisions.md                       | 20 ++++++++++++
 docs/design/squash-chain-landing.md     |  7 ++++
 5 files changed, 104 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (295 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
