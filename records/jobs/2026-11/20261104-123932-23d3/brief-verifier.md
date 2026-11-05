# Job 20261104-123932-23d3: verify "Revise warm-baseline design note to cover the CI-result path"

## Request (verbatim)

> try again on 20261029-160024-80qj

## Reading (accepted by the user)

Retry of rejected job 20261029-160024-80qj: write docs/design/warm-baseline-build.md (with a CHANGELOG entry) so it first describes how checks get results today (CI-result reuse vs local fallback), splits the 14.1 minutes by path, and scales the 8.0-minute prediction to the local-fallback share. Audience: the next throughput review and maintainers judging the design.

Assumptions:
- Existing code: crates/athena-core/src/routing.rs (CI result asked first, polled up to [budget] ci_wait_s, local checks as fallback) and docs/design/ci-routing.md (section 4.1 CI target cache keyed by Cargo.lock and rust-toolchain.toml; section 10 item 3) are read and cited, not changed.
- Job 20261029-160024-80qj findings addressed: major (CI path omitted) fixed by a current-state section, per-path durations and a share of jobs; minor (mtime freshness overstated) fixed by saying stable Cargo uses mtimes, so all workspace crates rebuild in a fresh snapshot and only third-party deps are reused, and by lowering the estimate; nit (CHANGELOG vs 'outside docs/design/') settled because CHANGELOG.md is a deliverable. Its note and CHANGELOG entry are kept as the starting point and revised.
- Per-path durations come from recorded ci_result records (source CI or local, and duration) where readable, plus THROUGHPUT.md; records/ is absent from the reader's copy, so any number not measurable is labeled an estimate with its source.
- If the warm baseline would not shorten the stage for most jobs, the note says so and recommends extending CI-result reuse or a warmer CI cache; the 8.0-minute prediction is restated scaled by the local-fallback share and the throughput review 20261029-155931-wfkr is still cited.
- No code, config, rust-toolchain.toml, athena.toml, defaults.toml or workflows change.

## Acceptance criteria

1. [content] The design has an Existing code section that reuses or extends crates/athena-core/src/routing.rs and docs/design/ci-routing.md, describing how checks get results today: CI result requested first, polled up to ci_wait_s, local checks only as fallback.
2. [content] The Problem section gives measured durations for the CI-reuse path and the local-run path from recorded jobs (ci_result source and duration) or THROUGHPUT.md, with the source of each number and any unmeasured number labeled an estimate, and states what share of jobs run checks locally.
3. [content] The 'Why this removes the bottleneck' section is derived from those per-path numbers, scales the 8.0-minute predicted saving to the local-fallback share, and says plainly if most jobs would not benefit, with a recommendation.
4. [content] Alternatives considered include extending or warming the CI-result path and CI target cache, with a reasoned comparison against the local warm baseline, alongside shared target dir, sccache and no change.
5. [content] The note corrects the freshness claim: stable Cargo uses file mtimes, a fresh snapshot rebuilds all workspace crates, only third-party deps are reused, and the estimate allows for it.
6. [content] The note still cites review 20261029-155931-wfkr and states how the next throughput review measures the saving (median minutes per landed job over the last 20, checks stage vs 14.1 and 25.4, split by CI vs local).
7. [content] git diff shows changes only to docs/design/warm-baseline-build.md and CHANGELOG.md; rust-toolchain.toml, athena.toml, defaults.toml, Cargo.lock, crates/ and .github/workflows/ are unchanged.

## Expected deliverables

- docs/design/warm-baseline-build.md (create)
- CHANGELOG.md (modify)

## Out of scope

- rust-toolchain.toml (read as a cache key input only; must not change)
- Implementing the warm baseline build or CI-reuse changes in crates/ or xtask/
- Changes to athena.toml, defaults.toml, Cargo.lock or .github/workflows/
- Changing docs/design/ci-routing.md or routing.rs (cited only)

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every number in the note cites its source or is labeled an estimate.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note lists alternatives and risks and reviews them.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the note's content against routing.rs and ci-routing.md, not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the design note and its changelog entry change.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is a note whose claim holds for the real checks flow.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Note is written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 849e0d330374 → candidate eb11c914373d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   6 ++
 docs/design/warm-baseline-build.md | 111 +++++++++++++++++++++++++++++++++++++
 2 files changed, 117 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261104-123932-23d3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261104-123932-23d3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41977170672 for eb11c914373d0dc832546027327ba4b9ba66ae43)
- ci-result: `not a check` passed (Check results for eb11c914373d0dc832546027327ba4b9ba66ae43 come from the CI run https://github.com/doodla/athena/actions/runs/41977170672 (541s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
