# Job 20261029-073603-7kqv: verify "Add throughput status section and throughput review to Athena"

## Request (verbatim)

> Priority: urgent
> Make Athena find its own throughput improvements (Sri, 2026-10-29: 'How can we have these things generated within Athena'). The velocity analysis and the asynchronous-drain idea came from people, not Athena: its reviews cover failures (error review), file and function size (code health) and structure (the 10-landing design review), but nothing measures throughput or asks what change to the pipeline's shape would remove the biggest wait. Add: (1) a throughput section in athena status computed from records: landed jobs per day, median minutes per landed job with a per-stage breakdown (reading, planning, producer, checks, verifier, landing and rebuild), idle time between jobs, and time spent on jobs that did not land; (2) a throughput review run every 10 landings and whenever median minutes per landed job worsens by more than 20% against the previous review: a strong model is given that section and docs/design and names the biggest bottleneck and the architectural change that removes it, with expected minutes saved per landed job; (3) the review queues its top proposal as a design request through the shared steward request builder, at priority set by expected minutes saved (speed first, OP-23), and records the prediction; (4) after a change cited by a review lands, the next review compares the measured minutes with the prediction and records the result.
> Acceptance:
> - A test with fixture job records shows status reports landed per day, median minutes per landed job by stage, idle time and time on failed jobs.
> - A test with a fake model shows the review queues one design request carrying the bottleneck, the proposed change and the predicted saving, and does not queue a duplicate while it is open.
> - A test shows a later review records predicted against measured minutes for a landed change it cited.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry; docs/design/hestia.md or the review design names the cadence.
> Files: crates/athena-core/src/status.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/code_health.rs, docs/design/hestia.md, CHANGELOG.md

## Reading (accepted by the user)

Athena status gains a throughput section computed from job records (landed per day, median minutes per landed job by stage, idle time, time on failed jobs). A throughput review runs every 10 landings or on a >20% worsening, asks a strong model for the biggest bottleneck and architectural fix, queues one design request via the shared steward request builder, and later records predicted vs measured minutes. Audience: Sri and the maintainers who want Athena to find its own speedups.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs (error_review, review_passes, root_cause_request, repeated_causes; review_passes is the entry point running reviews after a landing, and the new throughput review is added there beside the error review and code-health passes), crates/athena-core/src/status.rs (status rendering), crates/athena-core/src/code_health.rs (design-review cadence of 10 landings, reused for the counter), and crates/athena-data/src/records.rs (AnyRecord at line ~2386 and check_record_file at ~2686 are the records validation that accepts a new record kind by path and schema).
- Throughput is computed directly from existing job records (timestamps per stage, outcomes), not a stand-in; stage durations come from the same records status already reads.
- The prediction record (bottleneck, change, expected minutes saved, cited request, later measured minutes) reuses an existing record type if one fits; if a new record kind is added to crates/athena-data/src/records.rs, it gets a path, schema tag, an AnyRecord variant, check_record_file acceptance and a round-trip test in that file modelled on a_hestia_sweep_has_a_path_a_schema_and_round_trips. This is why records.rs is listed at moderate probability.
- The design request goes through the existing shared steward request builder used by root_cause_request; priority maps from expected minutes saved (more minutes, higher priority). Duplicate suppression looks for an open request from a prior throughput review.
- Model calls in tests use the existing fake-model facility; no real model is called.
- Governance files (athena.toml, defaults.toml) do not change; the 'strong model' uses an existing configured model setting, and no cadence constant needs a config entry.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core status`: a test with fixture job records asserts status output reports landed jobs per day, median minutes per landed job with per-stage values (reading, planning, producer, checks, verifier, landing and rebuild), idle time between jobs, and time spent on non-landed jobs, with exact expected numbers.
2. [behavior] Run `cargo test -p athena-core hestia`: a fake-model test shows a throughput review queues exactly one design request containing the bottleneck, proposed change and predicted minutes saved, and a second review run while it is open queues none.
3. [behavior] Run `cargo test -p athena-core hestia`: a test shows the review triggers at 10 landings since the last review and when median minutes per landed job worsens by more than 20%, and does not trigger otherwise.
4. [behavior] Run `cargo test -p athena-core hestia`: a test shows that after a cited change lands, the next review records predicted against measured minutes per landed job for it.
5. [behavior] Run `cargo test -p athena-core hestia`: a test shows request priority rises with expected minutes saved.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has an entry for the throughput section and review, and docs/design/hestia.md states the cadence (every 10 landings or >20% worsening) and the prediction-vs-measured follow-up.
8. [content] Any new record kind is accepted by check_record_file / AnyRecord in crates/athena-data/src/records.rs and has a round-trip test there; if no new kind is added, records.rs is unchanged.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- docs/design/hestia.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Implementing the asynchronous-drain change itself
- Changing athena.toml or defaults.toml
- Changing the error review or code-health review behavior

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests, fmt and clippy -D warnings are criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run named cargo test commands.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Design doc and changelog written for outside readers.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert measured numbers and queued requests, not wiring.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Review cadence is enforced by code triggers, tested.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in status.rs over an in-memory `Records` (JobRecords fields are pub): 3 landed jobs on two UTC days with Reader/Planner/Critic/Producer/Verifier invocations of known started_at/ended_at, verification checks of known durations, outcome.at at known offsets; one Rejected job and one plan-stopped job with known spans; gaps between job intervals and one overlap. Calls `Throughput::compute(&records)` (pure, no seam needed; added by step 3) and asserts the struct fields and the `text()` string that display.rs prints. Run: `cargo test -p athena-core status`. Observes: Exact numbers: landed per day (e.g. 2 and 1), median total minutes, medians for reading, planning, producer, checks, verifier and landing/rebuild, idle minutes (overlap gives 0), minutes on non-landed jobs.
  Fails without the behavior: Without Throughput::compute the test does not compile; a mis-attributed stage (planner counted as producer) or wrong idle merging fails the exact per-stage assertions.
- Criterion 2: Test in hestia.rs using the existing temp-repo helper `athena_in`, changed to take a driver. Entry point `Athena::throughput_review(&TestUi)`; guards: not nested and `throughput_due` true. Seam: `FakeAgentDriver::new(script)` (exists today; used in run/mod.rs and test_support.rs; hestia.rs athena_in uses `FakeAgentDriver::deterministic()`), script returns `FakeTurn::output(json!({bottleneck, stage, change, expected_minutes_saved, rationale}))` and counts calls. Seed 10 landed jobs with `put`. Run the review, then seed 10 more landings and run again. The double only returns canned JSON; request building, validation and suppression are production code. Run: `cargo test -p athena-core hestia`. Observes: After the first run exactly one open request job whose text has the marker line, bottleneck, proposed change and predicted minutes; one throughput-review record with the proposal and request job id. After the second run still one request, a second review record with request_job None and a note; model called once per review.
  Fails without the behavior: Without duplicate suppression the second run queues a second request and the open-request count fails; without the StewardRequest call no request exists.
- Criterion 3: Table test of pure `throughput_due(landed_count, window_median, latest_review)` with constructed review records, plus a test through `Athena::throughput_review` with a counting fake driver (same seam as criterion 2). Run: `cargo test -p athena-core hestia`. Observes: No prior review: 9 landed gives None, 10 gives Cadence. Prior review at 10 landings: 19 landed and equal median None, 20 landed Cadence. Prior median 10.0: current 12.0 None (20% is not more than 20%), 12.1 with a new landing Worsened, 12.1 with no new landing None, improved median None. Zero model calls and nothing queued when not due; one call when due.
  Fails without the behavior: A cadence of 9 or 11, a >= comparison at 20%, or firing on every drain fails a boundary assertion or the call-count assertion.
- Criterion 4: Test in hestia.rs. Commit with `put` a first review record R (median 20.0, proposal predicted saving 6.0, request_job J), a landed request job C whose text contains R and lands later than R, and 3 later landed jobs with known shorter spans. Run `Athena::throughput_review` with the fake driver past the 10-landing mark; also call pure `throughput_followups`. Seam as criterion 2. Run: `cargo test -p athena-core hestia`. Observes: The new review record's followups holds one entry for R: predicted 6.0, baseline 20.0, measured window median (e.g. 15.0) and measured saving 5.0. A third review adds no second follow-up for R. A cited change not yet landed records none. J itself, once landed, is not treated as the cited change.
  Fails without the behavior: Without the follow-up step, followups is empty and the predicted-against-measured assertion fails.
- Criterion 5: Test pure `priority_for_minutes` at 0.5, 2.0, 5.0, 10.0 and 25.0, and run `Athena::throughput_review` in two temp repos with the fake model predicting 12.0 and 1.0 (seam as criterion 2), reading the queued request text and `Records::priority()`. Run: `cargo test -p athena-core hestia`. Observes: Mapping low, normal, high, urgent, urgent; the 12.0 request carries `Priority: urgent` and the 1.0 request `Priority: low`; priority is non-decreasing over an increasing series.
  Fails without the behavior: A constant priority gives both requests the same level and the assertion fails.
- Criterion 6: Run the three Athena checks as separate simple commands: the one listed here is the test run; fmt and clippy are run the same way (`cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`). The tests that carry the behavior are those of criteria 1 to 5 and 8, so the full run also proves that the new StatusView field, AnyRecord variant, Records field and review_passes call break no existing status, display, records, drain, sweep, code_health or steward test. Run: `cargo test --locked`. Observes: All three exit 0. Existing drain and sweep tests still pass because throughput_due needs 10 landed jobs and fixtures that reach 10 get a prior review record.
  Fails without the behavior: A new AnyRecord variant left unhandled in a match, a missing StatusView field in a struct literal, a clippy warning, unformatted code, or a drain/sweep test hitting the model call through review_passes makes the corresponding command exit non-zero.
- Criterion 7: Read the diff: CHANGELOG.md Unreleased entry naming the status throughput section and the throughput review; hestia.md 'Throughput review' section stating every 10 landings (`code_health::REVIEW_EVERY`) or a median worse by more than 20% than the previous review, the priority mapping, the prediction record (path and schema), and the follow-up of predicted against measured minutes. Run: `git diff -- CHANGELOG.md docs/design/hestia.md`. Observes: The changelog entry exists; the design text contains 'every 10 landings', '20%', 'measured' and 'athena.throughput-review/1'.
- Criterion 8: New records.rs test modelled on `a_hestia_sweep_has_a_path_a_schema_and_round_trips`: build a ThroughputReview with a proposal and a follow-up, wrap in `AnyRecord::ThroughputReview`, assert schema, canonical_path `records/throughput-reviews/<month>/<id>.json`, and a JSON round trip through `check_record_file(path, bytes)`; a misplaced path gives Misplaced; NaN median or empty bottleneck is refused by validate. Run: `cargo test -p athena-data records`. Observes: The kind parses, validates, sits at its canonical path and round-trips.
  Fails without the behavior: Without the AnyRecord variant the schema tag does not parse and check_record_file refuses the file.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- docs/design/hestia.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2f0b7c2ba1b8 → candidate feea678c9481. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  10 +
 crates/athena-cli/src/display.rs      |   3 +
 crates/athena-cli/tests/job_plan.rs   |   5 +-
 crates/athena-core/src/code_health.rs |   4 +-
 crates/athena-core/src/hestia.rs      | 892 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/status.rs      | 508 ++++++++++++++++++-
 crates/athena-core/src/store.rs       |   7 +
 crates/athena-data/src/records.rs     | 217 +++++++++
 docs/design/hestia.md                 |  55 +++
 9 files changed, 1689 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1645 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
