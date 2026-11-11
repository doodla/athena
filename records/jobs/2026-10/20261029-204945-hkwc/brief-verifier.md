# Job 20261029-204945-hkwc: verify "Add bounded baseline-vs-candidate experiment check for OLY-9-05"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-9-05 a mechanism
> 
> Requirement OLY-9-05 in docs/spec.toml is active and has no mechanism.
> Requirement: Phase 3 introduces bounded baseline-versus-candidate experiments for changes to engineering methods.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-9-05
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-9-05 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a new athena-core module that defines a bounded baseline-versus-candidate experiment for a change to an engineering method, refuses unbounded or incomplete ones, and decides a verdict from the two arms. Record it as OLY-9-05's mechanism in docs/spec.toml with a measure ending `status: not shown` and the test name. The audience is Athena's spec sweep and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/exploration.rs (Experiment, validate, conclude, record_result: refusals for empty hypothesis, empty approach, zero bound), crates/athena-core/src/lib.rs (pub mod list), crates/athena-core/src/hestia.rs (spec sweep treats mechanism 'none' or a blank/unknown-label measure as a gap; measure must end `status: <label>` or `status: not shown`) and crates/athena-core/src/status.rs (labels athena status prints). The new module is a pure library function with no entry point, so no resolution or probing order applies and it is measured directly by unit tests of its own functions. It adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected.
- Existing code: extends the bounded-experiment pattern of crates/athena-core/src/exploration.rs and follows the spec-field conventions checked in crates/athena-core/src/hestia.rs; docs/spec.toml already holds OLY-9-05 with mechanism 'none'.
- Design: a new module crates/athena-core/src/method_experiment.rs with a MethodExperiment (method name, baseline arm, candidate arm, bound in runs, a metric where lower or higher is better), a validate that refuses an empty method, a missing arm, or a zero bound, and a compare that returns Refused, Stopped (runs exceeded bound), or a verdict of Better, Worse, or NoDifference for the candidate. It is not wired into the run pipeline; wiring is out of scope.
- Measure is 'cargo test -p athena-core method_experiment; status: not shown' because no `athena status` line reports it, matching OLY-9-04's style.
- This adds a refusal rule only for a new type with no past readings, so no past reading is newly refused: expected 0% of newly refused readings.
- Environment: none; no CI workflow, push or history read is involved.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-9-05 has mechanism naming crates/athena-core/src/method_experiment.rs (not 'none'), owner 'Athena', and a measure ending `status: not shown` that names `cargo test -p athena-core method_experiment`.
2. [content] crates/athena-core/src/method_experiment.rs exists and is declared with `pub mod method_experiment;` in crates/athena-core/src/lib.rs (Grep shows both).
3. [behavior] Run `cargo test -p athena-core method_experiment`: tests pass showing a zero-bound, empty-method or missing-arm experiment is refused, runs beyond the bound yield Stopped, and a candidate that beats, trails, or ties the baseline yields Better, Worse, NoDifference respectively. Each test calls the module's functions, so removing the module makes them fail to compile.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no warnings.

## Expected deliverables

- crates/athena-core/src/method_experiment.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Wiring experiments into the run pipeline, drain or athena status output
- Changes to athena.toml, defaults.toml, workflows or prompts
- Other spec requirements

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has unit tests; fmt, clippy and test criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests and check refusal and verdict behavior.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused new module plus spec entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9401f5e22764 → candidate 97e987dfec66. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   3 +
 crates/athena-core/src/lib.rs               |   1 +
 crates/athena-core/src/method_experiment.rs | 160 ++++++++++++++++++++++++++++
 docs/spec.toml                              |   4 +-
 4 files changed, 166 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-204945-hkwc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-204945-hkwc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42953310276 for 97e987dfec66cb54698d655a5af32ecd0ccdaa59)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42953310276 for 97e987dfec66cb54698d655a5af32ecd0ccdaa59)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42953310276 for 97e987dfec66cb54698d655a5af32ecd0ccdaa59)
- ci-result: `not a check` passed (Check results for 97e987dfec66cb54698d655a5af32ecd0ccdaa59 come from the CI run https://github.com/doodla/athena/actions/runs/42953310276 (687s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no warnings.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
