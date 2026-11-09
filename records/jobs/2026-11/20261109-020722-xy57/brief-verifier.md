# Job 20261109-020722-xy57: verify "Redo per-job main landing design note with premise check"

## Request (verbatim)

> try again on 20261109-014709-kxrd

## Reading (accepted by the user)

Retry of job 20261109-014709-kxrd: write docs/design/per-job-main-landing.md for throughput review 20261109-014617-5cg0, first checking that the 1842.1-minute landing stage is mostly a measurement effect (repeated landing listings per agent-and-build-slots.md §6.1), and claiming only real savings. Audience: Athena maintainers and the next throughput review that scores the prediction.

Assumptions:
- Retry of job 20261109-014709-kxrd (rejected, verdict fail). Its candidate added docs/design/per-job-main-landing.md; that file is kept as the base for the rewrite (a deliverable, modified in place). Each of its 7 findings is addressed as a checklist item: (1) the 1842.1-minute stage is mostly repeated listings, not real waiting, and the saving is restated in a real quantity; (2) per-job-landing-branches.md gets a pointer to the new note so the citation duty is not conflicting; (3) the CHANGELOG/decisions inconsistency is resolved by making both deliverables with criteria; (4) the misread premise is fixed by citing §6.1.
- Existing code: docs/design/agent-and-build-slots.md, docs/design/per-job-landing-branches.md, docs/design/squash-chain-landing.md, docs/design/native-pr-landing.md, docs/design/continuous-landing.md, crates/athena-core/src/land.rs, crates/athena-core/src/status/throughput.rs
- Premise check: the reader did not open throughput.rs or §6.1 here; the producer must, per the verifier's cited facts: landing_end (throughput.rs ~lines 405-413) uses the later of the outcome and landing_record_time, which takes the newest landing record listing the job; job 20261104-054754-b4f5 appears in 60 of 215 landing files, first 4.3 min after verifier, newest about 6,900 min later (verifier's computation, to be re-measured). So the 1842.1 figure is largely a measurement effect.
- Goal: a note that proposes how to cut landing wait and claims only real savings, measured in the same quantity as the problem (minutes from verifier pass to first landing record). Real wait is labeled with source or estimate; the 1830.0 figure is not used unless records show it is real.
- Environment: none needed; the note describes a PR-based flow but the job only edits markdown in the repository and reaches no remote.
- Docs-only change; CHANGELOG.md and docs/decisions.md are changed deliberately, listed as deliverables and named in criteria, which resolves the prior contradiction with the scope-limit criterion.

## Acceptance criteria

1. [content] docs/design/per-job-main-landing.md exists (ls docs/design) and cites throughput review 20261109-014617-5cg0 and the figures 1842.1 and 1903.5.
2. [content] The design has an Existing code section that reuses or extends docs/design/per-job-landing-branches.md, docs/design/squash-chain-landing.md, docs/design/native-pr-landing.md and crates/athena-core/src/land.rs, saying what is kept and what changes, with file and function references describing how landing works today before the proposal.
3. [content] A premise section cites docs/design/agent-and-build-slots.md §6.1 and crates/athena-core/src/status/throughput.rs landing_end, states what the 1842.1 figure measures, and separates real elapsed wait per job from the measurement effect of repeated landing listings, each number sourced or labeled as an estimate (Grep for '6.1' and 'landing_end').
4. [content] The claimed saving is stated in the same real quantity (minutes from verifier pass to first landing record) and does not claim 1830.0 minutes as saved unless the note shows records that it is real wait; the problem-size, mechanism and ranking sections are consistent with the corrected size.
5. [content] The note lists alternatives, including measuring from the first landing record, with rejection reasons, plus failure cases (merge conflict, main moved, verified content differing), and says the next review reports both the unchanged landing stage and the first-landing stage and how it measures the saving.
6. [content] The note states an implementing change must cite review 20261109-014617-5cg0 in its request text.
7. [content] docs/design/per-job-landing-branches.md has a pointer to the new note that resolves which review an implementing change must cite (git diff).
8. [content] CHANGELOG.md has a new entry for the new note and docs/decisions.md has a recorded decision (git diff).
9. [content] Only docs/design/per-job-main-landing.md, docs/design/per-job-landing-branches.md, docs/decisions.md and CHANGELOG.md change (git diff --stat); no crates/, athena.toml, defaults.toml, workflows, THROUGHPUT.md or records/ change.
10. [behavior] Run cargo test: the policy tests that scan docs pass with the new note, and cargo fmt --check and cargo clippy are clean.

## Expected deliverables

- docs/design/per-job-main-landing.md (create)
- docs/design/per-job-landing-branches.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/
- Editing athena.toml, defaults.toml or workflows
- Editing THROUGHPUT.md or records/
- Changing how throughput.rs measures landing_end (only proposed in the note)

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every number in the note carries a source file or an estimate label.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Note written for an outside reader with explicit terms.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note includes existing code, alternatives and failure cases.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Addresses the measurement effect, not just the headline stage.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Saving is stated in the measured quantity and checked by the next review.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check note content via grep and diff.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 32791de31e8f → candidate 43db5d91ead3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   6 +
 docs/decisions.md                       |  22 ++++
 docs/design/per-job-landing-branches.md |  10 +-
 docs/design/per-job-main-landing.md     | 192 ++++++++++++++++++++++++++++++++
 4 files changed, 229 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-020722-xy57/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-020722-xy57/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42549583974 for 43db5d91ead385f6ab8fce9a29aba9f9e136da1f)
- ci-result: `not a check` passed (Check results for 43db5d91ead385f6ab8fce9a29aba9f9e136da1f come from the CI run https://github.com/doodla/athena/actions/runs/42549583974 (649s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
