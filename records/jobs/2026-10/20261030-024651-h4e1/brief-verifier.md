# Job 20261030-024651-h4e1: verify "Record a mechanism and measure for OLY-B-A1-02 in docs/spec.toml"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A1-02 a mechanism
> 
> Requirement OLY-B-A1-02 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena keeps a living strategy view of goals, bets, risks, open questions and next steps, current enough for Sri to read at any time.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A1-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A1-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

The living strategy view already exists (`athena strategy`, crates/athena-cli/src/strategy.rs, docs/design/strategy-view.md). Point OLY-B-A1-02 at it with owner Athena and a "status: not shown" measure naming the test. Add a test that fails if the view is stale or the spec entry loses its mechanism.

Assumptions:
- Goal and audience: Sri reads the strategy view at any time; the verifier checks the spec entry and test.
- Code survey: read crates/athena-cli/src/strategy.rs (run, render, parse_map, parse_goals, drift, existing test strategy_view_table_matches_the_map), crates/athena-cli/src/commands.rs strategy(), main.rs Command::Strategy; xtask/tests/spec.rs validates spec fields. No new record or data file under crates/ is added, so check_record_file and AnyRecord are not affected.
- Existing code: crates/athena-cli/src/strategy.rs, docs/design/strategy-view.md and docs/design/requirements-map.md are reused and extended, not created.
- The measure uses `status: not shown` because I did not confirm that STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs has a strategy label; the producer should check and use a label if one fits.
- No athena.toml, defaults.toml or workflow change is needed; no refusal rule is added.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-A1-02 has a mechanism other than "none" that names `athena strategy` and crates/athena-cli/src/strategy.rs, owner "Athena", and a measure ending in `status: not shown` that names the test which shows it.
2. [behavior] Run `cargo test -p athena-cli strategy`: a new test reads docs/spec.toml and asserts OLY-B-A1-02's mechanism names an existing path (strategy.rs) and that `athena strategy` over the real map and view reports no errors; it fails with mechanism "none" and passes with the change.
3. [behavior] Run `cargo test -p xtask --test spec`: every active requirement still has mechanism, owner and measure, and it passes.
4. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
5. [content] Outcome check: Sri can run `athena strategy` and read docs/design/strategy-view.md to see goals, coverage and not-covered rows; the test is derived from that real output, not a stand-in.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-cli/src/strategy.rs (modify)
- docs/design/strategy-view.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the strategy view's content or format beyond what the test needs
- Other requirements' mechanisms
- Adding new status labels to athena status

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the strategy test against the real files.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing strategy module; builds nothing new.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 6a3b049fe7cc → candidate 0c153f4e8624. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  1 +
 crates/athena-cli/src/strategy.rs | 29 +++++++++++++++++++++++++++++
 docs/design/strategy-view.md      |  4 +++-
 docs/spec.toml                    |  4 ++--
 4 files changed, 35 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-024651-h4e1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-024651-h4e1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43286677917 for 0c153f4e862468393e29bf2d2f1f9cc98d59c6d4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43286677917 for 0c153f4e862468393e29bf2d2f1f9cc98d59c6d4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43286677917 for 0c153f4e862468393e29bf2d2f1f9cc98d59c6d4)
- ci-result: `not a check` passed (Check results for 0c153f4e862468393e29bf2d2f1f9cc98d59c6d4 come from the CI run https://github.com/doodla/athena/actions/runs/43286677917 (894s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
