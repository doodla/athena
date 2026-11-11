# Job 20261029-195218-tbnc: verify "Record experiment results as supported, unsupported or inconclusive"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-R6-02 a mechanism
> 
> Requirement OLY-R6-02 in docs/spec.toml is active and has no mechanism.
> Requirement: An experiment's result is recorded as supported, unsupported or inconclusive.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-R6-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-R6-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Extend the Dionysus experiment module in athena-core with a Result enum (supported, unsupported, inconclusive) and a function that records one on a validated, concluded experiment. Then set OLY-R6-02's mechanism and measure in docs/spec.toml, ending the measure with `status: not shown` and the test name.

Assumptions:
- Code survey: read crates/athena-core/src/exploration.rs (Experiment, Outcome, validate, conclude; only users of Experiment in crates/) and crates/athena-core/src/status.rs STATUS_MEASURE_LABELS. No label there shows experiment results, so the measure is `status: not shown` plus the test, as the request allows; adding a status line is out of scope.
- No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test do not apply; the result is an in-memory field on Experiment, with no persistence.
- No refusal rule is tightened for readings; the only new refusal applies to experiments, which nothing outside exploration.rs constructs.
- Goal: Athena's owner and the verifier can see that an experiment result is one of three values; audience is spec reviewers running cargo test.
- Existing code: crates/athena-core/src/exploration.rs and docs/spec.toml are extended, not created.

## Acceptance criteria

1. [content] docs/spec.toml OLY-R6-02 has a mechanism naming exploration.rs and its result-recording function (not "none"), keeps owner "Athena", and has a measure ending `status: not shown` that names the exploration test via `cargo test -p athena-core exploration`.
2. [content] crates/athena-core/src/exploration.rs defines a result type with exactly the variants Supported, Unsupported and Inconclusive and a function that stores one on an Experiment (Grep shows both).
3. [behavior] Run `cargo test -p athena-core exploration`: new tests show a concluded experiment records each of the three results and reads them back, and an invalid (e.g. unbounded) or not-yet-concluded experiment refuses a result with a Refusal. The tests fail to compile or fail if the recording function is removed.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` (including the xtask spec test): all pass, with no other spec.toml requirement changed.

## Expected deliverables

- crates/athena-core/src/exploration.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding a new `athena status` line or STATUS_MEASURE_LABELS entry
- Persisting experiment results to records/
- Changes to other spec requirements, athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests in exploration.rs; fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion runs the tests that record each result.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One module and one spec entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5599715b58b1 → candidate 32644da43c02. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  3 ++
 crates/athena-core/src/exploration.rs | 60 +++++++++++++++++++++++++++++++++++
 docs/spec.toml                        |  4 +--
 3 files changed, 65 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-195218-tbnc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-195218-tbnc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42932233342 for 32644da43c02adf4d39fa59bd52319e212564d3b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42932233342 for 32644da43c02adf4d39fa59bd52319e212564d3b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42932233342 for 32644da43c02adf4d39fa59bd52319e212564d3b)
- ci-result: `not a check` passed (Check results for 32644da43c02adf4d39fa59bd52319e212564d3b come from the CI run https://github.com/doodla/athena/actions/runs/42932233342 (715s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` (including the xtask spec test): all pass, with no other spec.toml requirement changed.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
