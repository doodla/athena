# Job 20261108-185513-gc6d: verify "Design per-job landing branches with a CHANGELOG entry"

## Request (verbatim)

> try again on 20261108-183842-fsaz

## Reading (accepted by the user)

Retry job 20261108-183842-fsaz: write the per-job landing branches design note (one job, one commit, one PR cut from main) for review 20261108-183648-5eya, now with the CHANGELOG.md entry, corrected stage path, and a stronger measurement rule.

Assumptions:
- Retry of job 20261108-183842-fsaz: its major finding (CHANGELOG.md not modified) is fixed by listing CHANGELOG.md as a deliverable and requiring an entry; its minor finding on status.rs vs status/throughput.rs is fixed by a criterion; its minor finding on weak attribution is fixed by requiring the per-phase split measure. Its three candidate files are kept as deliverables.
- Existing code: docs/design/continuous-landing.md, docs/design/squash-chain-landing.md and crates/athena-git/src/lib.rs are existing and are reused or extended by the design.
- Goal: a design note for throughput review 20261108-183648-5eya; audience is Athena maintainers and the next throughput review.
- The proposed pre-verifier changelog check from the introspection is a separate job and is out of scope here; the new request as typed asks only for the design with a CHANGELOG entry.
- Environment: the design discusses pull requests, but nothing is pushed or opened by this job; it writes only repository files locally, with no CI or history reads.

## Acceptance criteria

1. [content] A new Markdown file docs/design/per-job-landing-branches.md exists in the diff and describes landing each job as one commit in one pull request cut from current main, instead of stacking on a long-lived local branch.
2. [content] The note cites throughput review 20261108-183648-5eya and the figures 1537.1 of 1582.1 median minutes (THROUGHPUT.md); Grep for the review id in the new file finds it.
3. [content] The note states the predicted saving (1530.0 minutes per landed job) labeled as an estimate, and says the next throughput review measures it using the per-phase split from build job 1, so that a concurrent continuous-landing change is separated from this one.
4. [content] The design has an Existing code section that reuses or extends docs/design/continuous-landing.md, docs/design/squash-chain-landing.md and crates/athena-git/src/lib.rs, naming what is kept and what changes.
5. [content] The note lists alternatives considered (keep stacking with squash, rebase the stack) with reasons for rejection, and risks such as conflicts and landing order when cutting from main.
6. [content] Any reference to StageMinutes or the stage definition names crates/athena-core/src/status/throughput.rs, not status.rs (Grep 'StageMinutes' in the verifier's snapshot confirms the path).
7. [content] CHANGELOG.md is modified in the diff with an entry for the per-job landing design note, in the style of the existing continuous-landing.md entry (git diff shows a CHANGELOG.md hunk).
8. [content] docs/design/continuous-landing.md and docs/design/squash-chain-landing.md each gain only a short pointer to the new note (git diff). The diff touches only docs/design/ and CHANGELOG.md, and contains no secrets or personal data.

## Expected deliverables

- docs/design/per-job-landing-branches.md (create)
- docs/design/continuous-landing.md (modify)
- docs/design/squash-chain-landing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the landing change in crates/ or xtask/
- Implementing the pre-verifier changelog check
- Changes to athena.toml, defaults.toml or workflows
- Editing THROUGHPUT.md or records

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design note with alternatives and risks.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Cites THROUGHPUT.md figures and labels the saving an estimate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for outside readers.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Names how the next throughput review measures the saving.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One note plus two pointers and a changelog entry.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bcae35b98c4a → candidate 0eee697caa74. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   8 ++
 docs/design/continuous-landing.md       |   4 +
 docs/design/per-job-landing-branches.md | 197 ++++++++++++++++++++++++++++++++
 docs/design/squash-chain-landing.md     |   4 +
 4 files changed, 213 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-185513-gc6d/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-185513-gc6d/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42512523462 for 0eee697caa742d73aab08571a814a7b6bbbdc120)
- ci-result: `not a check` passed (Check results for 0eee697caa742d73aab08571a814a7b6bbbdc120 come from the CI run https://github.com/doodla/athena/actions/runs/42512523462 (505s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
