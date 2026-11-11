# Job 20261029-204934-s2vj: verify "Give OLY-R8-01 a per-class specialization line in athena status"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-R8-01 a mechanism
> 
> Requirement OLY-R8-01 in docs/spec.toml is active and has no mechanism.
> Requirement: The system identifies tasks where additional specialization improves results.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-R8-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-R8-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a measured `specialization` section to `athena status`. For each class of work it compares producer model/effort settings by first-pass verification and landed share, and flags classes where a non-default setting beat the default. Record it as the mechanism for OLY-R8-01 in docs/spec.toml with a measure ending `status: specialization`.

Assumptions:
- Goal and audience: the Athena owner and the spec pass (Hestia) judge whether OLY-R8-01 'identifies tasks where additional specialization improves results' is shown by a mechanism and measure in docs/spec.toml.
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at ~line 305, StatusView.producer_settings at ~401, assembled at ~1246 by producer_settings(&records)), crates/athena-core/src/status/planning.rs (producer_settings at ~394 groups request_jobs by the first Producer invocation's model and effort and counts jobs, verified, first_pass, landed, cost), crates/athena-cli/src/display.rs (producer_settings_lines at ~158; printed at ~1258 only when non-empty; label at line start). Entry point is the status view builder in status.rs; the new function runs after records load, reading job.invocations, job.verification and job.state() with no probing or resolution order beyond that. Records validation: no new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are untouched; the class comes from the existing request job class_of_work field in crates/athena-data/src/records.rs (~line 607, Option<String>), which must be read, not added. The property is measured directly: per class_of_work, the first-pass rate and landed share of each model/effort bucket compared with the most-used bucket in that class, from records, not a stand-in such as a config list.
- Design: a class is flagged 'specialization helps' only when a non-baseline setting has at least a minimum job count (a named constant, e.g. 3) and a strictly higher landed share or first-pass rate than the baseline setting of that class; with fewer jobs it prints 'not enough jobs', never a flag. Jobs with no class_of_work are grouped under 'unclassified'. Empty records print nothing and are not counted as success.
- Existing code: crates/athena-core/src/status/planning.rs producer_settings and ProducerSettingStats, crates/athena-core/src/status.rs STATUS_MEASURE_LABELS, crates/athena-cli/src/display.rs producer_settings_lines are reused and extended, none created.
- Consumers: none found for a config change; docs/spec.toml is the only non-code deliverable. xtask spec test requires the `status:` marker and the Hestia spec pass checks the label against STATUS_MEASURE_LABELS, so the new label must be added there too.
- This adds no refusal rule, so no refused-reading percentage applies.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-R8-01 has mechanism naming the new function in crates/athena-core/src/status/planning.rs (not 'none'), owner Athena, and a measure ending `status: specialization` (Grep for the entry).
2. [content] crates/athena-core/src/status.rs STATUS_MEASURE_LABELS contains a ('specialization', <StatusView field>) entry, and that field exists on StatusView and is filled in the view builder.
3. [behavior] Run `cargo test -p athena-core`: a unit test builds records with two classes where a non-default model/effort lands more often than the default with enough jobs, and asserts that class is flagged and the other (too few jobs, or no better setting) is not; the test fails when the new function is stubbed to return nothing and passes with it.
4. [behavior] Run `cargo test -p athena-cli`: a display test shows the `specialization` section starts a line, lists each class with its settings and the flag or 'not enough jobs' text, and is absent for empty records.
5. [behavior] Run `cargo test` for the workspace including xtask: the spec test and the Hestia label check accept the new measure label.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/status/planning.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Automatically routing jobs to specialized producers or changing model selection.
- Changes to athena.toml, defaults.toml, CI workflows or prompt files.
- Other requirements in docs/spec.toml.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and display tests plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail without the mechanism.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Empty records print nothing and are never counted as shown.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One status section built on existing producer_settings code.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e128cad6a03e → candidate 6a768e7fa89e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |  5 ++
 crates/athena-cli/src/display.rs          | 93 +++++++++++++++++++++++++++++-
 crates/athena-core/src/status.rs          | 95 +++++++++++++++++++++++++++++++
 crates/athena-core/src/status/planning.rs | 90 ++++++++++++++++++++++++++++-
 docs/spec.toml                            |  4 +-
 5 files changed, 282 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-204934-s2vj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-204934-s2vj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42937447934 for 6a768e7fa89efb0b9e5ee555f09331805a09cb9a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42937447934 for 6a768e7fa89efb0b9e5ee555f09331805a09cb9a)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42937447934 for 6a768e7fa89efb0b9e5ee555f09331805a09cb9a)
- ci-result: `not a check` passed (Check results for 6a768e7fa89efb0b9e5ee555f09331805a09cb9a come from the CI run https://github.com/doodla/athena/actions/runs/42937447934 (706s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
