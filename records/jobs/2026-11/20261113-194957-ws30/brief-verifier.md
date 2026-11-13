# Job 20261113-194957-ws30: verify "Design note: land each job on fetched main via three-way merge"

## Request (verbatim)

> try again on 20261113-190327-f794

## Reading (accepted by the user)

Retry of job 20261113-190327-f794 (rejected on a failing repo-wide test check unrelated to a docs-only diff, a missing run of cargo test, and three minor note errors). Write the design note for throughput review 20261113-190159-mapn: retire the long-lived local landing branch and build each job's single landing commit on fetched host main by a three-way merge from the job's recorded base. The audience is Athena's owner and the next throughput review.

Assumptions:
- Existing code: crates/athena-git/src/lib.rs (Git::merge_tree and related helpers), crates/athena-core/src/land.rs (plan, dispatched_from, refuse_detached, Verified::base) and docs/design/direct-main-merge-tree-landing.md, which already proposes the same mechanism for an earlier review.
- Job 20261113-190327-f794 is retried from base. Its three changed files (docs/design/direct-main-single-commit-landing.md, CHANGELOG.md, docs/decisions.md) are kept as deliverables and rewritten. Its test failure (drain parallel test, shard 3) is judged as unrelated to a docs-only diff; the criterion cites Athena's recorded check on the same tree, as D115 allows.
- Fixes for the three minor findings: the base commit comes from Verified::base(), and dispatched_from and refuse_detached are branch checks only. The 5.0-minute prediction is scored in one way only, as the THROUGHPUT.md median compared with 2342.1 minus 5.0. The note states plan's preconditions: the dispatch branch is checked out, and the base is an ancestor of head.
- Goal: a reviewable design that the next throughput review can score. Audience: Athena's owner and the next throughput review. No code in crates/ changes.

## Acceptance criteria

1. [content] A new file docs/design/direct-main-single-commit-landing.md describes retiring the long-lived local landing branch and building each job's single landing commit on fetched host main by a three-way merge from the job's recorded base. It explains how this removes the landing-and-rebuild wait (2310.0 of 2342.1 median minutes per landed job).
2. [content] The design has an Existing code section that reuses or extends crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs and docs/design/direct-main-merge-tree-landing.md. It names the functions it builds on and what changes. It names Verified::base() as the source of the base, and says dispatched_from and refuse_detached are only branch checks.
3. [content] The note states the preconditions of plan: the dispatch branch is checked out, and the base is an ancestor of the head. It says how migration meets them.
4. [content] The note states a predicted saving of 5.0 minutes per landed job, labeled as an estimate against the 2342.1 median. It says one way the next review measures it: the THROUGHPUT.md median per landed job for jobs landed after the change, compared with 2337.1. No other basis is given for scoring.
5. [content] The note cites review 20261113-190159-mapn and states that a change implementing it must cite that id in its request text.
6. [content] The note lists at least two alternatives with why each was rejected. It also lists the risks, including merge conflicts at landing and the failure path.
7. [content] CHANGELOG.md has a new entry for the job. docs/decisions.md changes only by a new appended decision with a Provenance bullet. Nothing under records/ and no existing decision is edited.
8. [behavior] The diff touches only docs, so Athena's recorded fmt, clippy and test checks on the same final tree count as evidence under D115. If the recorded test check failed, the verifier runs cargo test --locked and confirms any failing test is unrelated to the diff, such as the drain test. The checks are cargo fmt --check, cargo clippy and cargo test.

## Expected deliverables

- docs/design/direct-main-single-commit-landing.md (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Implementing the landing change in crates/ code
- Editing athena.toml, defaults.toml or workflows
- Editing existing entries of docs/decisions.md or anything under records/
- Fixing the drain parallel test or changing how the test check handles docs-only jobs (the introspection's proposed follow-up request)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that the note states the saving and how the next review measures it.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The note labels 5.0 min as an estimate and cites THROUGHPUT.md figures.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note lists alternatives and risks.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1b1a6576e8f4 → candidate 1ef91544439e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                     |   1 +
 docs/decisions.md                                |  19 ++++
 docs/design/direct-main-single-commit-landing.md | 135 +++++++++++++++++++++++
 3 files changed, 155 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261113-194957-ws30/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-194957-ws30/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43499893292 for 1ef91544439efee8dff392625fa3a1d84171ef52)
- ci-result: `not a check` passed (Check results for 1ef91544439efee8dff392625fa3a1d84171ef52 come from the CI run https://github.com/doodla/athena/actions/runs/43499893292 (936s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: The diff touches only docs, so Athena's recorded fmt, clippy and test checks on the same final tree count as evidence under D115. If the recorded test check failed, the verifier runs cargo test --locked and confirms any failing test is unrelated to the diff, such as the drain test. The checks are cargo fmt --check, cargo clippy and cargo test.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
