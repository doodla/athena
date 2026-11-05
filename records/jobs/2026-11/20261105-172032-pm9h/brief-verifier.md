# Job 20261105-172032-pm9h: verify "Design note: many agents at once, few builds at once"

## Request (verbatim)

> Design: many agents at once, few builds at once. Write a design note in docs/design/ for running dozens of agents in parallel on this machine (4 CPUs, 15 GB memory, about 27 GB free disk), where each job's Rust build takes about 10 to 12 GB.
> 
> The note:
> - measures from job records how a job's wall time splits into model time, local build and test time, and waiting, citing the records; when it leaves jobs out of the sample, it says exactly which (for example, job directories that hold only a request and never ran)
> - separates agent slots (model work) from build slots (cargo runs, limited by disk and CPU), with a queue for build slots
> - proposes how jobs share compiled output so each build takes far less disk
> - covers the local landing-and-rebuild step, which status --snapshot shows as the longest stage
> - says how many agents and builds at once this machine can run, with the numbers
> - lists the build order as small changes, each landable on its own
> 
> Files changed: one new file under docs/design/.
> 
> Acceptance:
> - every measurement cites the records it comes from, and every exclusion from the sample is stated correctly
> - each proposed change names the code it touches and how its effect will be measured

## Reading (accepted by the user)

Write one new design note under docs/design/ for running dozens of agents on a 4-CPU, 15 GB, ~27 GB-free machine. It separates agent slots from queued build slots, shares compiled output across jobs, and covers the landing-and-rebuild stage. Its audience is the maintainer, who will judge the measurements, sizing numbers, and landable build order.

Assumptions:
- Existing code: docs/design/parallel-jobs.md, docs/design/warm-baseline-build.md, docs/design/continuous-landing.md, docs/design/async-drain.md, crates/athena-core/src/status.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-git/src/snapshot.rs, crates/athena-data/src/paths.rs are read and reused or extended by the design, not created.
- Goal: a design note the maintainer can use to decide a concurrency model; audience is the maintainer and the verifier checking citations.
- The note is docs only; no Rust code changes, so no Code survey is needed. Exact filename is the producer's choice.
- Job records are read from the records available to the producer; if some are absent the note says so rather than inventing numbers.
- The default designed and evidence apply: alternatives are written down and numbers are measured or labeled estimates.

## Acceptance criteria

1. [content] Exactly one new file is added under docs/design/ and no other path changes (git status / git diff shows a single added .md file; athena.toml, defaults.toml and workflows are untouched).
2. [content] The note has an Existing code section that reuses or extends docs/design/parallel-jobs.md, docs/design/warm-baseline-build.md, docs/design/continuous-landing.md, crates/athena-core/src/status.rs, crates/athena-core/src/run/checks.rs and crates/athena-git/src/snapshot.rs, saying what each already does and what the new design adds.
3. [content] A measurement section splits job wall time into model time, local build and test time, and waiting, with each figure citing the specific job record files it was computed from; a reader can open the cited records and reproduce each number.
4. [content] The note states exactly which job directories were left out of the sample (e.g. directories holding only a request that never ran) and each stated exclusion is true of the directories named or counted, checked against the records listing.
5. [content] The note separates agent slots (model work) from build slots (cargo runs limited by disk and CPU), with a queue for build slots and its ordering and timeout behaviour described.
6. [content] The note proposes a shared compiled-output scheme (for example shared CARGO_TARGET_DIR, sccache or a warm baseline copy) with a disk-per-build estimate that is labeled as measured or estimate, and records the buy-first call on existing tools.
7. [content] The note covers the local landing-and-rebuild step, citing the status --snapshot output or code (crates/athena-core/src/status.rs) that shows it as the longest stage, and proposes how to shorten it.
8. [content] The note gives numbers for how many agents and how many builds this machine can run at once, derived from 4 CPUs, 15 GB memory, ~27 GB free disk and 10-12 GB per build, with the arithmetic shown.
9. [content] The note lists the build order as small numbered changes, each landable on its own; each names the crates/ files it touches and how its effect will be measured (a named metric and where it is read).

## Expected deliverables

- docs/design/agent-and-build-slots.md (create)
- docs/design/build-slots.md (create)
- docs/design/parallel-agents-build-slots.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Implementing any of the proposed changes in Rust
- Changing athena.toml, defaults.toml or workflows
- Editing existing design notes

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every measurement cites its job records; unmeasured numbers are labeled estimates.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note lists alternatives for sharing compiled output and slot control.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Considers sccache and shared target dirs before building custom caching.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new file; build order is small independently landable changes.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check numbers and citations against records, not file existence.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Each proposed change names how its effect is measured.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 49454c58782c → candidate d68463748c7a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 docs/design/agent-and-build-slots.md | 440 +++++++++++++++++++++++++++++++++++
 1 file changed, 440 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-172032-pm9h/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-172032-pm9h/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42078440142 for d68463748c7a72acd702f1f27b28fd9cd8ff12ae)
- ci-result: `not a check` passed (Check results for d68463748c7a72acd702f1f27b28fd9cd8ff12ae come from the CI run https://github.com/doodla/athena/actions/runs/42078440142 (459s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
