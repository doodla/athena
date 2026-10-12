# Job 20261012-204524-f34n: Form squash landing chains from landed jobs only

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
