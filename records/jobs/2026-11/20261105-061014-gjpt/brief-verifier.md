# Job 20261105-061014-gjpt: verify "Revise warm-baseline-build design note on measured CI and local paths"

## Request (verbatim)

> try again on 20261104-123932-23d3

## Reading (accepted by the user)

Retry of job 20261104-123932-23d3: rewrite docs/design/warm-baseline-build.md (plus CHANGELOG entry) with an Existing code section reusing routing.rs, ci-routing.md and ci.yml, measured per-path durations and local share (October window versus November), the corrected 'mostly CI wait' claim, and only new recommendations tagged against what exists. Audience: the next throughput review and maintainers.

Assumptions:
- Retry of rejected job 20261104-123932-23d3: its findings are addressed as follows: the false 'mostly CI wait' claim is corrected from the October records; the guessed CI duration is replaced by the median of the 52 recorded CI run durations and local durations from records; the already-built recommendations (PR cache restore, test shards) are marked 'already present' or dropped; job counts are re-taken from verification.json records (50 of 58 November jobs reused CI); the note splits the review's own window, not only November. Its two changed files, docs/design/warm-baseline-build.md and CHANGELOG.md, are kept as deliverables and revised.
- Existing code: crates/athena-core/src/routing.rs, docs/design/ci-routing.md and .github/workflows/ci.yml are existing and not created by this job; the design reuses or extends them.
- Goal: a correct, evidence-based design note; audience is the next throughput review and maintainers judging the design. The request text must cite review 20261029-155931-wfkr.
- Records under records/ may be absent from the producer's copy; numbers it cannot measure are labeled estimates.

## Acceptance criteria

1. [content] The design has an Existing code section that reuses or extends crates/athena-core/src/routing.rs, docs/design/ci-routing.md and .github/workflows/ci.yml, listing each as already present (target-cache restore on pull-request runs, sharded test job, CI-result polling up to ci_wait_s, local fallback) with file and step or function names.
2. [content] The Problem section gives measured durations for the CI-reuse path (median of the recorded 'CI run <id> (<n>s)' durations in brief-verifier records) and the local-run path (from verification.json records and THROUGHPUT.md), each number with its source or an 'estimate' label, and states the local-run share both for the review window (279 landings up to 2026-10-29, mostly local) and for November (about 50 of 58 reused CI).
3. [content] The note checks the claim that the 14.1-minute median is mostly CI wait against the records, and corrects it: the October review window ran checks mostly locally, and the later bounded CI polling (job 20261101-061448-sdya) already moved the stage away from 14.1; the note says the next review's comparison to 14.1 must be split by CI versus local.
4. [content] Every recommendation is tagged 'new' or 'already present'; none re-proposes restoring the target cache on pull-request runs or splitting test across jobs, which ci.yml and ci-routing.md already do. The remaining recommendation follows the measured cause, scales the 8.0-minute prediction to the local-fallback share, and says plainly if most jobs would not benefit.
5. [content] Alternatives considered include a warm local baseline, warming or extending the CI-result path and CI target cache, shared target dir, sccache and no change, compared with reasons; the note corrects the freshness claim (stable Cargo uses file mtimes, workspace crates rebuild, only third-party deps reuse).
6. [content] The note cites review 20261029-155931-wfkr and states how the next throughput review measures the saving (median minutes per landed job over the last 20, checks stage versus 14.1 and 25.4, split by CI versus local).
7. [content] git diff shows changes only to docs/design/warm-baseline-build.md and CHANGELOG.md; rust-toolchain.toml, athena.toml, defaults.toml, Cargo.lock, crates/, xtask/ and .github/workflows/ are unchanged.

## Expected deliverables

- docs/design/warm-baseline-build.md (create)
- CHANGELOG.md (modify)

## Out of scope

- rust-toolchain.toml (read as a cache key input only)
- Implementing the warm baseline build or CI-reuse changes in crates/ or xtask/
- Changes to athena.toml, defaults.toml, Cargo.lock or .github/workflows/
- Changing docs/design/ci-routing.md or routing.rs (cited only)

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every Problem-section number carries a source or an estimate label.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Note written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One doc plus a changelog entry.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the corrected analysis and per-path numbers, not file existence.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Recommendation rests on the measured cause, the local-fallback share.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c8f2bbad2de7 → candidate 628297cb816f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   9 +++
 docs/design/warm-baseline-build.md | 137 +++++++++++++++++++++++++++++++++++++
 2 files changed, 146 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-061014-gjpt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-061014-gjpt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41979198338 for 628297cb816f1349bebd5d07cf96180034b2e79f)
- ci-result: `not a check` passed (Check results for 628297cb816f1349bebd5d07cf96180034b2e79f come from the CI run https://github.com/doodla/athena/actions/runs/41979198338 (454s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
