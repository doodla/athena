# Job 20261029-180911-7q3e: verify "Add class-of-work trend to athena status for OLY-9-10"

## Request (verbatim)

> Priority: urgent
> 
> Give requirement OLY-9-10 a mechanism
> 
> Requirement OLY-9-10 in docs/spec.toml is active and has no mechanism.
> Requirement: Phase 5 is accepted when comparable historical evidence shows repeated classes of work improving and identifies earlier changes that made things worse.
> Owner: Hestia
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-9-10
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-9-10 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Give OLY-9-10 a mechanism: a new class-trend measure in athena-core that compares older and newer landed jobs per class of work (minutes), reports classes improving, and names the earlier landed job after which the class's cost rose. It is printed by `athena status` as a `trend` line and recorded in docs/spec.toml with measure ending `status: trend`. Audience: Hestia/the owner judging Phase 5.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (Throughput::compute, landed_jobs, StageMinutes, median, STATUS_MEASURE_LABELS, StatusView, test every_measure_label_names_a_status_view_field_and_a_printed_line), crates/athena-cli/src/display.rs (status text assembly near the throughput line), crates/athena-core/src/store.rs (JobRecords.job.class_of_work, Records::request_jobs) and crates/athena-core/src/hestia.rs (measure label check). No new record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are not affected; the measure is computed directly from existing job records, not via a stand-in.
- Design: new module crates/athena-core/src/class_trend.rs computes, per class_of_work (from the job record) with at least 4 landed jobs, the median total minutes of the older half against the newer half of landed_jobs(); a drop is 'improving', a rise is 'worse'. For a worsening class it names the landed job that begins the newer half's rise (the earliest job after which the class's minutes rose above the older median and stayed). StatusView gets a `class_trend` field, and STATUS_MEASURE_LABELS gets ("trend", "class_trend").
- The spec.toml measure is rewritten to end with `status: trend`, and mechanism points at class_trend.rs, status.rs and display.rs.
- Cost is measured as the existing minutes-per-landed-job (invocation, check and landing time), the comparable evidence available; token cost is out of scope.
- No refusal rule is added or tightened, so no replay refusal-rate criterion applies.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-9-10 has mechanism naming crates/athena-core/src/class_trend.rs (a file that exists), keeps owner Hestia, and a measure ending with `status: trend`; STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs contains the `trend` label.
2. [behavior] Run `cargo test -p athena-core class_trend`: a test builds landed jobs of one class whose minutes fall from older to newer and asserts the class is reported improving; another builds a class whose minutes rise after a named earlier job and asserts that job id is reported as making it worse; a class with too few jobs is not reported. These tests fail if compute returns nothing.
3. [behavior] Run `cargo test -p athena-core every_measure_label_names_a_status_view_field_and_a_printed_line`: passes with the new `trend` label, proving StatusView has class_trend and `athena status` prints a line starting `trend `.
4. [behavior] Run `cargo test -p athena-cli`: a status rendering test shows the `trend` line text with improving and worse classes; and the xtask/spec test over docs/spec.toml passes (`cargo test --workspace`).
5. [behavior] `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace` all pass.
6. [content] The trend section is absent (no line) when no class has enough landed jobs, so an empty history is not shown as success.

## Expected deliverables

- crates/athena-core/src/class_trend.rs (create)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Token-cost trends or forecast-score trends
- Automatically reverting or proposing fixes for changes that made things worse
- Changing athena.toml, defaults.toml or workflows
- Other requirements without mechanisms

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the module; fmt, clippy, test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that check improving/worse output.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Measure is enforced by status label test.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Absent output when history is too thin.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One module plus wiring.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5ea553195fc0 → candidate a8b4dc7d8f52. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |   4 +
 crates/athena-cli/src/display.rs      |  39 ++++++
 crates/athena-core/src/class_trend.rs | 220 ++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs         |   1 +
 crates/athena-core/src/status.rs      |   6 +
 docs/spec.toml                        |   4 +-
 6 files changed, 272 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1694 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
