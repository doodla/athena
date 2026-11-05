# Job 20261105-103606-svwx: verify "Design note: many agent slots, few build slots"

## Request (verbatim)

> Design: many agents at once, few builds at once. The owner wants Athena to run dozens of agents in parallel on this one machine (4 CPUs, 15 GB memory, about 27 GB free disk). Today a batch runs at most 8 jobs (MAX_CONCURRENCY_CEILING) and each job builds Rust in its own target directory, about 10 to 12 GB, so disk allows about two at once. Most of a job's time is model time (reader, planner, critic, writer, verifier), not building.
> 
> Write a design note in docs/design/ that:
> - measures, from the job records of the last 3 days, how a job's wall time splits into model time, local build and test time, and waiting
> - separates agent slots (model work, cheap on this machine) from build slots (cargo runs, limited by disk and CPU), with a queue for build slots
> - proposes how jobs share compiled output so each job's build takes far less disk
> - says how many agents and how many builds at once this machine can run, with the numbers behind each
> - lists the build order as small changes, each landable on its own
> 
> Files changed: one new file under docs/design/.
> 
> Acceptance:
> - the note's measurements cite the job records they come from
> - each proposed change names the code it touches and how its effect will be measured

## Reading (accepted by the user)

Write one design note in docs/design/ for running dozens of agents at once on this 4-CPU, 15 GB, ~27 GB-disk machine while capping concurrent cargo builds. The audience is the owner, who will decide whether to fund the work. The note measures time splits from the last 3 days of job records, separates agent slots from build slots, proposes shared compiled output, sizes both limits, and lists landable steps.

Assumptions:
- Existing code: crates/athena-data/src/config.rs (MAX_CONCURRENCY_CEILING, [parallel] max_concurrency), crates/athena-core/src/run/mod.rs (producer CARGO_TARGET_DIR at ~line 1821), crates/athena-core/src/state.rs (produce_dir), crates/athena-core/src/disk.rs (BuildRunner, build with target_dir), crates/athena-agent/src/env.rs (BUILD_DIR_VARS, per-side CARGO_TARGET_DIR), docs/design/parallel-jobs.md, docs/design/warm-baseline-build.md.
- Goal: let the owner choose a concurrency plan; the audience is the owner and an outside reader of the repo. The note is docs only; no code changes.
- Job records live under records/, which the reader cannot see; the producer may have access to them and must cite specific job ids and files, and label any figure it cannot measure as an estimate.
- Shared compiled output is proposed from options such as a shared CARGO_TARGET_DIR per baseline with copy-on-write or hardlinks, sccache, or a warm baseline target; the note compares alternatives and recommends one, building on the warm-baseline-build design.
- Governance files (athena.toml, defaults.toml, workflows) are not changed; the note may propose edits to them as future steps only.

## Acceptance criteria

1. [content] Exactly one new file under docs/design/ is added and no other path changes (check with git status / git diff).
2. [content] The design has an Existing code section that reuses or extends crates/athena-data/src/config.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs and docs/design/warm-baseline-build.md, saying what each already does and what the design builds on.
3. [content] A measurements section splits job wall time into model time, local build and test time, and waiting, over the last 3 days, with numbers and each figure citing the specific job record ids or files it came from; unmeasured numbers are labeled estimates.
4. [content] The note defines agent slots and build slots separately, including a queue for build slots, and states how a job acquires and releases a build slot around cargo runs.
5. [content] The note proposes a way for jobs to share compiled output, with at least two alternatives compared, and gives an expected per-job disk figure against the current 10 to 12 GB.
6. [content] The note states how many agents and how many builds this machine can run at once, with the arithmetic from 4 CPUs, 15 GB memory and about 27 GB disk, and says whether MAX_CONCURRENCY_CEILING must be raised.
7. [content] A build order lists small changes, each landable alone; each names the files or functions it touches (e.g. config.rs, run/mod.rs, state.rs, disk.rs, env.rs) and states how its effect will be measured.
8. [behavior] cargo test and cargo clippy still pass with the doc-only change (run cargo test, cargo clippy).

## Expected deliverables

- docs/design/agents-and-builds.md (create)
- docs/design/many-agents-few-builds.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Implementing any of the proposed changes in code
- Editing athena.toml, defaults.toml or workflows
- Changing MAX_CONCURRENCY_CEILING now
- Editing existing design docs

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Measurements cite job records; unmeasured numbers labeled estimates.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design lists alternatives and a recommendation.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new file; build order as small independent changes.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Each change states how its effect is measured.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the note.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5ffa278deb42 → candidate c42351644365. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 docs/design/many-agents-few-builds.md | 382 ++++++++++++++++++++++++++++++++++
 1 file changed, 382 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-103606-svwx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-103606-svwx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42067850692 for c423516443651bc36ac72a30970780f606938234)
- ci-result: `not a check` passed (Check results for c423516443651bc36ac72a30970780f606938234 come from the CI run https://github.com/doodla/athena/actions/runs/42067850692 (565s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test and cargo clippy still pass with the doc-only change (run cargo test, cargo clippy).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
