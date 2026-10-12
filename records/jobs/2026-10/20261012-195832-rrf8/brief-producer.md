# Job 20261012-195832-rrf8: Accept intervention re-scores in squashed commits in the history check

## Request (verbatim)

> in a squash, a score.json that an intervention re-scored fails 'does not follow from the job's records' (seen for 20261012-095109-f1x7 and 20261012-180308-knqc in a simulated squash of PR #9); fix it so squashes with interventions pass

## Reading (accepted by the user)

The history check's score validation (jobs.rs check → expected_score) recomputes a re-score from the squashed tree and wrongly rejects re-scores that `athena intervene --job` wrote legitimately. The fix makes squashes carrying interventions pass and keeps forged re-scores refused, with a regression test.

Assumptions:
- The false failure comes from expected_score() in jobs.rs recomputing the closing score from the final squashed tree (all interventions, all records), whereas the recorded re-score was derived by rescore() from the earlier closing score at intervention time. I did not run a simulated squash, so the exact differing field is unconfirmed; the producer must reproduce it first.
- The fix stays in the checker (athena-gate), keeping the forged-re-score refusal intact, rather than changing how scores are written.
- No governance files change.

## Acceptance criteria

1. [content] The root cause of the false 'does not follow from job X's records' for an intervention re-score in a squash is identified in code comments or the commit message, citing the differing field(s).
2. [content] crates/athena-gate/src/jobs.rs no longer reports a re-score as not following from the records when the squash tree holds the closing score, the intervention and its re-score for the same job, as `athena intervene --job` writes them.
3. [behavior] A new test builds a squash (chain or single job) containing a landed job plus an intervention and its re-score, and asserts check() returns no violations. Run `cargo test -p athena-gate`.
4. [behavior] The existing test hand_written_scores_and_outcomes_are_refused still passes: a forged perfect re-score (all Brier values zeroed) with a real intervention is still refused with 'does not follow'. Run `cargo test -p athena-cli --test lifecycle hand_written`.
5. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Expected deliverables

- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-score/src/lib.rs (modify)

## Out of scope

- Exempting commits from the history check (D66/D67).
- Changing how athena intervene writes scores.
- Rewriting history or editing records/.
- Other history-check rules for squashes.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A regression test for squash plus intervention re-score accompanies the fix; the whole suite and clippy stay clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the re-score validation in athena-gate.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments and any decision-log entry explain the squash case for an outside reader.

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
