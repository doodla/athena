# Job 20261104-141009-hsg4: verify "Throughput review: flag median slowdowns and queue a request"

## Request (verbatim)

> Make the throughput review catch slowdowns and keep moving. The median time per landed job rose from 25.4 minutes (review of 29 Oct) to about 60 minutes (1 to 4 Nov) and nothing flagged it: the review proposes one design request and then waits until that request closes.
> 
> Files changed: the throughput review code and its tests.
> 
> Acceptance:
> - each review run compares the latest median minutes per landed job with the previous review's and names the stages whose minutes grew, even while a design request is open
> - when the median grew by more than a quarter, the review queues a request at high priority naming the stage that grew most and its measured minutes
> - a test where the median rose from 25 to 60 minutes with a design request open: the grown stages are named and a request is queued
> - a test where the median is flat: nothing is queued

## Reading (accepted by the user)

Today the throughput review (crates/athena-core/src/hestia.rs) queues one model-proposed design request and does nothing while it is open, so the 25.4 to 60 minute rise went unflagged. I read it as: every review run compares the latest median with the previous review's and names the stages that grew, even with a design request open. When the median grew by more than 25%, it also queues a high-priority request naming the biggest-growing stage and its measured minutes.

Assumptions:
- Goal and audience: the operator reading throughput reviews and the queue wants a slowdown flagged and acted on, not hidden behind an open design request.
- Code survey: read crates/athena-core/src/hestia.rs (throughput_due ~1236, throughput_review ~1438, open_throughput_request, throughput_request, throughput_followups, tests from ~3893), crates/athena-core/src/status.rs (Throughput::compute, landed_jobs, StageMinutes::named, median, THROUGHPUT_WINDOW) and crates/athena-data/src/records.rs (ThroughputReview, ThroughputTrigger, ThroughputProposal; AnyRecord::ThroughputReview validate and schema). The entry point is Athena::throughput_review, called from review_passes. It loads records, calls Throughput::compute, then throughput_due, then either notes that a request is open or asks the model. The median is measured directly from landed jobs' stage minutes, not through a stand-in.
- The previous review stores only window_median_minutes and landings, not per-stage medians. Plan: recompute the previous per-stage medians from landed_jobs(records) cut to the first `previous.landings` jobs, using the newest THROUGHPUT_WINDOW of them, and compare with the current Throughput::median_stages. This needs no record schema change, so records.rs, its schema and the AnyRecord round-trip test stay unchanged. If a producer finds this unworkable and adds a record field, it must update the schema and the round-trip test (check_record_file/AnyRecord).
- A new pure function (for example `stage_growth`) returns the stages whose median minutes grew, largest first. The deterministic slowdown request is queued by Athena code through the existing StewardRequest/self.queue path at Priority::High, without needing the model, and is queued regardless of an open Throughput design request. The review note and the review's CLI output line name the grown stages.
- The 25% threshold is a new constant (for example SLOWDOWN_RATIO = 1.25). The existing 20% WORSEN_RATIO trigger stays, so a review still runs between 20% and 25%, names the stages, and queues nothing.
- To avoid queuing a duplicate slowdown request on every run, a still-open request with a distinct title marker for slowdowns suppresses another one. Each comparison is against the previous review, so a persistent slowdown is flagged once per review.

## Acceptance criteria

1. [behavior] Test (run `cargo test -p athena-core hestia`): the previous review recorded a median of 25 minutes, the current median is 60, and an open request whose text has a line starting 'Throughput design: ' exists. The review result or note names every stage whose minutes grew, and a new request is queued at Priority::High. The test asserts the stage names and the queued request text.
2. [behavior] Test (run `cargo test -p athena-core hestia`): the median is flat against the previous review, so no request is queued and no stage is reported as grown. The test asserts that the queue holds no new request.
3. [behavior] Test (run `cargo test -p athena-core hestia`): a rise above 20% but not above 25% names the grown stages and queues no slowdown request. A rise just over 25% queues one.
4. [content] The queued slowdown request text, as built by the new code, names the stage that grew most and its measured median minutes (and the previous minutes) and carries priority high. Grep of the request builder in crates/athena-core/src/hestia.rs shows both values interpolated, and the test asserts them.
5. [behavior] Test: a review with an open design request still compares stage medians against the previous review and keeps the existing behavior of asking the model for no second proposal. Existing throughput tests still pass (`cargo test`).
6. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass. Changes are limited to the throughput review code and its tests, and no governance file changes.
7. [content] The module doc comment in hestia.rs, and docs/design/hestia.md if it describes the trigger, state the new comparison, the 25% rule and the stage naming. This matches outcome-check: the check is the slowdown being flagged, not the review merely running.

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/design/hestia.md (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the model prompt or its proposal schema, or the code-health and error reviews.
- Changing the throughput measurement itself (stage definitions, THROUGHPUT_WINDOW).
- Tuning the review cadence (REVIEW_EVERY) or the 20% trigger.
- Governance files (athena.toml, defaults.toml, workflows).

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the slowdown and flat cases, and cargo test and clippy stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check stage naming and queueing in tests with an open request, not only that the functions exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is a slowdown flagged and a request queued, not the review merely running.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in hestia.rs, with the previous stage medians derived from existing records and no schema change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The 25% rule is enforced in code and tested, not left as prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the rule for an outside reader.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two new tests drive `Athena::throughput_review` over committed records, with an open `Throughput design: ` request and a model that must not be called. Test A (`a_median_rise_from_25_to_60_with_a_design_request_open_names_the_grown_stages_and_queues_a_high_request`): ten 25-minute jobs (producer 25) plus `earlier_review(10, 25.0)` plus the open design request job, then 11 jobs of 60 minutes (producer 45, checks 10, verifier 5). Test B (`unmeasured_landed_jobs_before_the_earlier_review_do_not_hide_the_growth`): twelve landed jobs with no invocation (`timed_job` with `StageMinutes::default()`) landing first, then ten 25-minute jobs, `earlier_review(22, 25.0)`, the open design request, then 11 jobs of 60 minutes. A naive cut of `landed_jobs` by 22 would take all 21 measured jobs and report nothing. Also a status.rs test of `landed_jobs_among_first`, and a pure test of `review_subject`. Run: `cargo test -p athena-core hestia`. Observes: Both tests: `review.window_median_minutes == 60.0`; the model call log is empty; `note` contains 'still open' and the names producer, checks and verifier (producer 25.0 to 45.0 first); exactly one open request starts `Throughput slowdown: `, its `priority()` is `Priority::High`, and the queued text begins `Priority: high` and has the stage names and minutes; `note` contains `slowdown request <job> queued`; the `Lines` UI got a line with the stage names. Test B also asserts `previous_stage_medians(...).producer == 25.0` for the rebuilt window. `review_subject` with a slowdown job and no design job contains the job id and not 'no request queued'. A second review with 10 more, slower jobs while the slowdown request is open names the grown stage and queues no second one.
  Fails without the behavior: Without the comparison ahead of the open-request branch, the note is only the 'still open' text and no `Throughput slowdown: ` request exists. With the round-1 cut, Test B finds no grown stage and queues nothing. With the old subject code, the `review_subject` assertion fails.
  Test seam: Entry point `Athena::throughput_review` (exists, hestia.rs 1438). Fixtures `put`, `seed_landed`, `athena_with`, `earlier_review`, `Lines`, `throughput_model` and `status::planning_cost_tests::timed_job` (all exist; `timed_job` drops zero-minute stages, so `StageMinutes::default()` gives a landed job with no invocation). Step 8 adds `seed_stages`, `seed_unmeasured`, and `open_design_request_job`. The pure functions `review_subject` and `landed_jobs_among_first` are added by steps 1 and 5. The model fake is injected through `athena_with`, as in the existing throughput tests. No logic sits inside a double.
- Criterion 2: New test `a_flat_median_queues_nothing`: ten 25-minute jobs, `earlier_review(10, 25.0)`, ten more 25-minute jobs (twenty landings so the cadence makes the review due), plus an open design request job. Call `throughput_review`, then reload records. A pure `stage_growth` test over equal medians. Run: `cargo test -p athena-core hestia::tests::a_flat_median_queues_nothing`. Observes: The review runs; the open request count is still the one design request; no job text has a line starting `Throughput slowdown: `; the model was not called; `note` does not name any stage as grown; `stage_growth` of equal medians is empty.
  Fails without the behavior: If every review were treated as a slowdown or unchanged stages were listed as grown, the growth list would be non-empty or a slowdown request would appear.
  Test seam: Entry point `Athena::throughput_review` (exists) and pure `stage_growth` (added in step 2). Fixtures exist: `seed_landed`, `earlier_review`, `put`, `athena_with`, `throughput_model`; the open-request fixture is added in step 8.
- Criterion 3: New test `a_rise_of_twenty_to_twenty_five_percent_names_stages_but_queues_no_slowdown_request`, looping over two fixtures: ten 20-minute jobs, `earlier_review(10, 20.0)`, 11 producer-only jobs of 24.5 and then of 25.5 minutes, and an open design request. Plus a pure `is_slowdown` test. Run: `cargo test -p athena-core hestia::tests::a_rise_of_twenty_to_twenty_five_percent_names_stages_but_queues_no_slowdown_request`. Observes: At 24.5 (trigger `Worsened`): `note` names `producer` and no `Throughput slowdown: ` request exists. At 25.5: exactly one high-priority slowdown request exists. `is_slowdown(20.0, 25.0)` is false, `(20.0, 25.1)` true, `(0.0, 10.0)` false.
  Fails without the behavior: A 20% threshold or `>=` would queue a request at 24.5; a threshold above 25% would queue none at 25.5.
  Test seam: Entry point `Athena::throughput_review` and pure `is_slowdown` (step 2). Fixtures `seed_landed` (now calls `seed_stages`, step 8), `earlier_review`, `put`, `athena_with` exist.
- Criterion 4: Pure test `the_slowdown_request_names_the_biggest_stage_and_its_minutes_at_high_priority` calls `slowdown_request(25.0, 60.0, &growth)` with growth producer 25.0 to 45.0, checks 0.0 to 10.0, verifier 0.0 to 5.0. A grep of `slowdown_request` in hestia.rs shows the stage, the measured minutes and the previous minutes interpolated in the title and body. Run: `cargo test -p athena-core hestia::tests::the_slowdown_request_names_the_biggest_stage_and_its_minutes_at_high_priority`. Observes: The text starts `Priority: high`; the title is `Throughput slowdown: producer ...`; it contains `45.0`, `25.0`, `60.0` and the other grown stages on one line; `request_text_acceptance(text)` is non-empty; no body line starts like a bullet; no review id appears; `slowdown_request` with an empty growth returns Err.
  Fails without the behavior: A wrong stage, missing minutes or a priority other than high fails the string assertions.
  Test seam: Production function `slowdown_request` (step 2), called directly; no double.
- Criterion 5: Test A from criterion 1 asserts the model call log is empty and the open design request count stays at one while the stage medians are compared. The existing throughput tests run unchanged, and a pure `throughput_outcome` test shows a review with a queued design request and a slowdown remark prints both. Run: `cargo test -p athena-core hestia`. Observes: All pass. In the existing open-request test the second review's note still contains 'still open' and the model is called once. `throughput_outcome` for a design-request review with note 'slowdown request X queued at high priority' contains both 'design request' and 'slowdown request X'; with an empty note it equals the old text.
  Fails without the behavior: If the open-request branch were removed or reordered, the existing 'queues one design request and none while it is open' test would show a second model call. If the comparison were skipped, Test A fails. If `throughput_outcome` dropped the note, its assertion fails.
  Test seam: Entry point `Athena::throughput_review` (exists); `throughput_model` is the `FakeAgentDriver` injected by `athena_with` (exists, hestia.rs 3893). `throughput_outcome` is added in step 5 and called directly with hand-built `ThroughputReview` values.
- Criterion 6: Run the three Athena checks, then `git diff --stat`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. `git diff --stat` lists only hestia.rs, status.rs, records.rs (doc comments only), docs/design/hestia.md and CHANGELOG.md; no athena.toml, defaults.toml or workflow file.
  Fails without the behavior: A formatting, clippy or test failure fails the command; a governance file change shows in the diff.
  Test seam: None: cargo and git run on the working tree.
- Criterion 7: Read the diff of the hestia.rs module comment (lines 43-52) and docs/design/hestia.md section 12, and the CHANGELOG entry. Run: `git diff -- crates/athena-core/src/hestia.rs docs/design/hestia.md CHANGELOG.md`. Observes: The module comment, the doc comments on `SLOWDOWN_RATIO`, `stage_growth`, `previous_stage_medians`, `slowdown_request`, and section 12 state: every review compares the latest median with the previous review's, even with a design request open; the previous stage medians are rebuilt from the same set of landed jobs the stored count came from; grown stages are named; more than 25% queues a high-priority request naming the biggest-growing stage and its measured minutes; one open slowdown request holds back another. The CHANGELOG Unreleased entry names the behavior.
  Fails without the behavior: Without the doc changes the module comment and section 12 would still describe only the 10-landing and 20% triggers.
  Test seam: None: a content check of the diff.

Planned files:

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-data/src/records.rs (modify)
- docs/design/hestia.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9aa2e377fe94 → candidate 8d6d8fd49841. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  11 +
 crates/athena-core/src/hestia.rs  | 721 ++++++++++++++++++++++++++++++++++++--
 crates/athena-core/src/status.rs  |  99 +++++-
 crates/athena-data/src/records.rs |   7 +-
 docs/design/hestia.md             |  19 +
 5 files changed, 810 insertions(+), 47 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41911278310 for 8d6d8fd49841c9b7adea5e967e9a227828c9a4f3)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41911278310 for 8d6d8fd49841c9b7adea5e967e9a227828c9a4f3)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41911278310 for 8d6d8fd49841c9b7adea5e967e9a227828c9a4f3)
- ci-result: `not a check` passed (Check results for 8d6d8fd49841c9b7adea5e967e9a227828c9a4f3 come from the CI run https://github.com/doodla/athena/actions/runs/41911278310 (478s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass. Changes are limited to the throughput review code and its tests, and no governance file changes.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core hestia`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
...    |
1251 | |     (landed > before && worse).then_some(ThroughputTrigger::Worsened)
1252 | | }
     | |_- similarly named function `throughput_due` defined here
...
4612 |           assert!(throughput_outcome(&review).starts_with("no proposal: slowdown request"));
     |                   ^^^^^^^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
4612 -         assert!(throughput_outcome(&review).starts_with("no proposal: slowdown request"));
4612 +         assert!(throughput_due(&review).starts_with("no proposal: slowdown request"));
     |

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3524:21
     |
3524 |         let first = landed_jobs_among_first(&records, 5);
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3527:20
     |
3527 |         assert_eq!(landed_jobs_among_first(&records, 7).len(), 4);
     |                    ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3528:17
     |
3528 |         assert!(landed_jobs_among_first(&records, 3).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3529:17
     |
3529 |         assert!(landed_jobs_among_first(&records, 0).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no associated function or constant named `median_of` found for struct `status::StageMinutes` in the current scope
    --> crates/athena-core/src/status.rs:3533:34
     |
 652 | pub struct StageMinutes {
     | ----------------------- associated function or constant `median_of` not found for this struct
...
3533 |         assert_eq!(StageMinutes::median_of(&window), throughput.median_stages);
     |                                  ^^^^^^^^^ associated function or constant not found in `status::StageMinutes`

error[E0599]: no associated function or constant named `median_of` found for struct `status::StageMinutes` in the current scope
    --> crates/athena-core/src/status.rs:3534:34
     |
 652 | pub struct StageMinutes {
     | ----------------------- associated function or constant `median_of` not found for this struct
...
3534 |         assert_eq!(StageMinutes::median_of(&[]), StageMinutes::default());
     |                                  ^^^^^^^^^ associated function or constant not found in `status::StageMinutes`

Some errors have detailed explanations: E0422, E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 43 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core hestia::tests::a_flat_median_queues_nothing`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
1239 | |     latest: Option<&ThroughputReview>,
...    |
1251 | |     (landed > before && worse).then_some(ThroughputTrigger::Worsened)
1252 | | }
     | |_- similarly named function `throughput_due` defined here
...
4612 |           assert!(throughput_outcome(&review).starts_with("no proposal: slowdown request"));
     |                   ^^^^^^^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
4612 -         assert!(throughput_outcome(&review).starts_with("no proposal: slowdown request"));
4612 +         assert!(throughput_due(&review).starts_with("no proposal: slowdown request"));
     |

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3524:21
     |
3524 |         let first = landed_jobs_among_first(&records, 5);
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3527:20
     |
3527 |         assert_eq!(landed_jobs_among_first(&records, 7).len(), 4);
     |                    ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3528:17
     |
3528 |         assert!(landed_jobs_among_first(&records, 3).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3529:17
     |
3529 |         assert!(landed_jobs_among_first(&records, 0).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no associated function or constant named `median_of` found for struct `status::StageMinutes` in the current scope
    --> crates/athena-core/src/status.rs:3533:34
     |
 652 | pub struct StageMinutes {
     | ----------------------- associated function or constant `median_of` not found for this struct
...
3533 |         assert_eq!(StageMinutes::median_of(&window), throughput.median_stages);
     |                                  ^^^^^^^^^ associated function or constant not found in `status::StageMinutes`

error[E0599]: no associated function or constant named `median_of` found for struct `status::StageMinutes` in the current scope
    --> crates/athena-core/src/status.rs:3534:34
     |
 652 | pub struct StageMinutes {
     | ----------------------- associated function or constant `median_of` not found for this struct
...
3534 |         assert_eq!(StageMinutes::median_of(&[]), StageMinutes::default());
     |                                  ^^^^^^^^^ associated function or constant not found in `status::StageMinutes`

Some errors have detailed explanations: E0422, E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 43 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core hestia::tests::a_rise_of_twenty_to_twenty_five_percent_names_stages_but_queues_no_slowdown_request`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
1239 | |     latest: Option<&ThroughputReview>,
...    |
1251 | |     (landed > before && worse).then_some(ThroughputTrigger::Worsened)
1252 | | }
     | |_- similarly named function `throughput_due` defined here
...
4612 |           assert!(throughput_outcome(&review).starts_with("no proposal: slowdown request"));
     |                   ^^^^^^^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
4612 -         assert!(throughput_outcome(&review).starts_with("no proposal: slowdown request"));
4612 +         assert!(throughput_due(&review).starts_with("no proposal: slowdown request"));
     |

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3524:21
     |
3524 |         let first = landed_jobs_among_first(&records, 5);
     |                     ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3527:20
     |
3527 |         assert_eq!(landed_jobs_among_first(&records, 7).len(), 4);
     |                    ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3528:17
     |
3528 |         assert!(landed_jobs_among_first(&records, 3).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `landed_jobs_among_first` in this scope
    --> crates/athena-core/src/status.rs:3529:17
     |
3529 |         assert!(landed_jobs_among_first(&records, 0).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no associated function or constant named `median_of` found for struct `status::StageMinutes` in the current scope
    --> crates/athena-core/src/status.rs:3533:34
     |
 652 | pub struct StageMinutes {
     | ----------------------- associated function or constant `median_of` not found for this struct
...
3533 |         assert_eq!(StageMinutes::median_of(&window), throughput.median_stages);
     |                                  ^^^^^^^^^ associated function or constant not found in `status::StageMinutes`

error[E0599]: no associated function or constant named `median_of` found for struct `status::StageMinutes` in the current scope
    --> crates/athena-core/src/status.rs:3534:34
     |
 652 | pub struct StageMinutes {
     | ----------------------- associated function or constant `median_of` not found for this struct
...
3534 |         assert_eq!(StageMinutes::median_of(&[]), StageMinutes::default());
     |                                  ^^^^^^^^^ associated function or constant not found in `status::StageMinutes`

Some errors have detailed explanations: E0422, E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 43 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core hestia::tests::the_slowdown_request_names_the_biggest_stage_and_its_minutes_at_high_priority`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git diff -- crates/athena-core/src/hestia.rs docs/design/hestia.md CHANGELOG.md`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
