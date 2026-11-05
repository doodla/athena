# Job 20261105-170035-1704: verify "Retry: design note on many agents, few concurrent builds"

## Request (verbatim)

> try again on 20261105-103606-svwx

## Reading (accepted by the user)

Retry of job 20261105-103606-svwx: write one design note in docs/design/ for running dozens of agents at once on this 4-CPU, 15 GB, ~27 GB-disk machine, with agent slots separate from queued build slots and shared compiled output. The audience is the owner, who decides whether to fund it. The earlier candidate, docs/design/many-agents-few-builds.md, is dropped and rewritten, fixing each of its findings.

Assumptions:
- Existing code: crates/athena-data/src/config.rs (MAX_CONCURRENCY_CEILING), crates/athena-core/src/disk.rs (disk checks), crates/athena-core/src/run/mod.rs (run pipeline and cargo invocation), crates/athena-core/src/state.rs (remove_produce at about line 272) and docs/design/warm-baseline-build.md (shared baseline build) exist and are not created by this job; the design builds on them.
- Job 20261105-103606-svwx was rejected (verdict fail). Its one-file candidate, docs/design/many-agents-few-builds.md, is dropped; the retry starts from base and writes a fresh note. The same filename may be reused; it is a create, since the candidate is not in the repository.
- Finding fixes for 20261105-103606-svwx: (a) lanes vs per-run slots: the note settles that a job holds its lane for the whole job (preferred lane) and only a CPU build slot per cargo run, so trimming happens per job and lane size is sized for one job's output; (b) remove_produce is located in state.rs, not disk.rs; (c) the note says in one line what the roughly 77 excluded job directories are (no outcome.json, or still in flight); (d) the CHANGELOG finding does not apply: the request says one new file only, so CHANGELOG.md is excluded.
- The failed criterion 8 (cargo test and clippy evidence) was an evidence-visibility problem in Athena's flow, not a defect in the note. The introspection's proposed follow-up (recorded command runs for criteria) is out of scope here. The doc-only change is checked by the recorded check run of cargo test and cargo clippy, which the verifier may also run itself.
- Measurements come from job records of the last 3 days (2026-11-03 to 2026-11-05); the producer reads them from the files it has and cites job ids. Any figure it cannot measure is labeled an estimate.
- Machine arithmetic: 4 CPUs, 15 GB memory, about 27 GB free disk; at 10 to 12 GB per target directory, disk allows about 2 builds today.

## Acceptance criteria

1. [content] Exactly one new file under docs/design/ is added and no other path changes (check with git status and git diff); CHANGELOG.md, athena.toml, defaults.toml and workflows are untouched.
2. [content] The design has an Existing code section that reuses or extends crates/athena-data/src/config.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/state.rs and docs/design/warm-baseline-build.md, saying what each already does and what the design builds on.
3. [content] A measurements section splits job wall time over the last 3 days into model time, local build and test time, and waiting, with numbers. Each figure cites the job record ids or files it came from, unmeasured numbers are labeled estimates, and one line says what the jobs left out of the sample are.
4. [content] The note defines agent slots and build slots separately, including a queue for build slots, and states how a job acquires and releases a build slot around cargo runs. It states whether a lane or target directory is held per job or per cargo run, and that choice is consistent across all sections.
5. [content] The note proposes a way for jobs to share compiled output, compares at least two alternatives, and gives an expected per-job disk figure against the current 10 to 12 GB.
6. [content] The note states how many agents and how many builds this machine can run at once, with arithmetic from 4 CPUs, 15 GB memory and about 27 GB disk, and says whether MAX_CONCURRENCY_CEILING must be raised.
7. [content] A build order lists small changes, each landable alone. Each names the files or functions it touches (for example config.rs, run/mod.rs, state.rs for remove_produce, disk.rs) and states how its effect will be measured.
8. [behavior] The doc-only change leaves the build green: the recorded check run of cargo test and cargo clippy --all-targets --locked -- -D warnings passes, and the verifier can run cargo test and cargo clippy itself.

## Expected deliverables

- docs/design/many-agents-few-builds.md (create)
- docs/design/agents-and-builds.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Implementing any proposed change in code
- Editing athena.toml, defaults.toml or workflows
- Changing MAX_CONCURRENCY_CEILING now
- Editing existing design docs
- Modifying CHANGELOG.md (the request allows one new file only)
- Making criteria that name commands verifiable from recorded evidence (the introspection's follow-up request)

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The note is written for the owner as an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Measurements cite job ids and files; estimates are labeled.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note compares alternatives and records the recommended build order.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new file; each proposed step is landable alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the note's content, measurements and arithmetic, not just that the file exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must still pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier checks the note against the criteria.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base dd4794e03bda → candidate 337c02d4402b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 docs/design/many-agents-few-builds.md | 180 ++++++++++++++++++++++++++++++++++
 1 file changed, 180 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-170035-1704/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-170035-1704/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42071534057 for 337c02d4402bdc9beddbbfc28cc2d819fbc2325b)
- ci-result: `not a check` passed (Check results for 337c02d4402bdc9beddbbfc28cc2d819fbc2325b come from the CI run https://github.com/doodla/athena/actions/runs/42071534057 (453s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: The doc-only change leaves the build green: the recorded check run of cargo test and cargo clippy --all-targets --locked -- -D warnings passes, and the verifier can run cargo test and cargo clippy itself.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
