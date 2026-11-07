# Job 20261107-031925-mrpk: verify "Make the throughput review measure wasted work, not only landed time"

## Request (verbatim)

> Make the throughput review measure wasted work, not only the time of work that landed.
> 
> The throughput review measures minutes per landed job. On 4 to 6 Nov, 58% of attempts did not land and it could not see them; four of its seven runs did nothing because an earlier design request was still open.
> 
> Change the review so that:
> - it reports attempts per landed change, money spent on attempts that did not land, and the share of changes that landed on the first attempt, over the newest 20 landings and the 20 before them
> - it triggers on a rise in any of these, as well as on the median time
> - it does not skip a run because an earlier design request is open; it notes the open request instead
> - it only queues a request when the change is larger than the noise between recent windows, and says so
> 
> Acceptance:
> - a test: a window where half of attempts did not land reports attempts per landing of 2 and a first-attempt share of 0.5 from sample records
> - a test: a rise in wasted spend triggers a review with no change in median time
> - a test: an open earlier design request no longer stops the review
> - athena status shows the three measures
> - all tests pass; CHANGELOG states only what the diff does
> 
> Files changed: the throughput review in Hestia, status display, tests, CHANGELOG.md.

## Reading (accepted by the user)

Add attempts per landed change, spend on attempts that did not land, and first-attempt share to the throughput section (newest 20 landings vs the 20 before). Hestia's throughput review triggers on a rise in any of them, notes rather than skips on an open design request, and queues a request only when the change exceeds recent-window noise. Audience: the maintainer reading `athena status` and the review.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs: `throughput_due(landed, median, latest)` (~line 1341) is the trigger, called from the throughput review in `Athena::review_passes` (~line 1711); right after it, `open_throughput_request` (~line 1728) currently stops the review when a design request is open, so the order is: trigger check, then the open-request skip, then the model proposal, then slowdown/stage growth. Read crates/athena-core/src/status.rs: `Throughput::compute(&Records)` (~line 984) builds the section over `THROUGHPUT_WINDOW` = 20 newest landed jobs, already tracks `unlanded_jobs`/`unlanded_minutes`, and `Throughput::text()` is what `athena status` prints via crates/athena-cli/src/display.rs. The new measures are computed directly from job/attempt records and outcome totals (cost) in `Throughput::compute`, not through the minutes stand-in. Read crates/athena-data/src/records.rs: `ThroughputReview` (~line 2188) and `ThroughputTrigger` (Cadence, Worsened) are accepted by `AnyRecord::parse` / `check_record_file` (line 3111) via `AnyRecord::ThroughputReview(r) => r.validate()`; the change adds fields with serde defaults (so old records still parse) and new trigger variants, and is covered by `ThroughputReview::validate` plus a round-trip test through `AnyRecord::parse` in records.rs tests. No new record file type or allow-list entry is added.
- Prior-window comparison reuses the previous review's stored measures (new defaulted fields on ThroughputReview) or recomputes the 20-before window from records; the producer picks the simpler one.
- Noise is estimated as the spread (e.g. max-min or median absolute deviation) of the measure across recent 20-landing windows; a request is queued only when the change exceeds it, and the review note says whether it did.
- Wasted spend = cost of attempts on jobs/attempts that did not land, from outcome totals; first-attempt share = landed changes with exactly one attempt / landed changes.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new status.rs test builds sample records where half of attempts in the window did not land and asserts attempts per landing == 2.0 and first-attempt share == 0.5.
2. [behavior] Run `cargo test -p athena-core`: a new hestia.rs test shows `throughput_due` (or its successor) returns a trigger when wasted spend rises beyond the previous review and noise while median minutes is unchanged; the same inputs without the rise return None.
3. [behavior] Run `cargo test -p athena-core`: a new hestia.rs test shows an open earlier design request no longer stops the review: the review runs, and its record note mentions the open request.
4. [behavior] Run `cargo test -p athena-core`: a test shows a change within the noise between recent windows queues no request and the note says it was within noise, while a larger change queues one.
5. [behavior] Run `cargo test -p athena-cli` and `cargo run -- status` (or the status_snapshot test): the throughput output shows attempts per landed change, money spent on attempts that did not land, and first-attempt share.
6. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass; an old ThroughputReview JSON without the new fields still parses via `AnyRecord::parse`, and a new one round-trips.
7. [content] CHANGELOG.md has entries naming only what the diff does (the three measures, new triggers, open-request note, noise rule); no unrelated claims.
8. [content] The doc comment of hestia.rs and the Throughput review section of docs/design/hestia.md no longer say an open design request skips the review (Grep for the old wording finds none).

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_snapshot.rs (modify)
- CHANGELOG.md (modify)
- docs/design/hestia.md (modify)
- crates/athena-core/src/snapshot.rs (modify)

## Out of scope

- Changing the slowdown request's 25% rule or the stage-growth comparison beyond what the noise rule needs.
- Changing athena.toml, defaults.toml, workflows or prompt files.
- Rewriting existing records under records/.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for each behavior; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests asserting values 2.0 and 0.5, triggers and no-skip.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Status output exercised via test or cargo run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the review, status and record.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks that wasted work is seen and triggers the review.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A new status.rs test builds in-memory records with the existing helpers `timed_job` (status.rs:3261) and `loaded` (3343): 20 landed jobs (index i starts at hour 2i, 20 minutes long) and 20 failed jobs (`OutcomeState::Failed`). Failed job k starts 30 minutes after landing k starts and runs 5 minutes, so it closes after landing k ends and before landing k+1 ends, inside landing k's window. Each failed job's producer invocation costs 0.50 USD via `invocation(&job, 1, Role::Producer, 0.5)` (status.rs:2443). Landings 1-10 get request text 'try again on <id of failed job k-1>' in the form `reading::retry_reference` accepts, so 10 landings are retries and failed jobs 10-19 are never retried. The test first asserts the window counts (20 landings, 20 unlanded, 40 attempts), then calls `waste_windows(&records, THROUGHPUT_WINDOW)` and `Throughput::compute(&records)`. Further cases: only 20 landed jobs; 7 landed jobs (partial window); and a closed unlanded job with no invocations added to the half-failed fixture, which must not change any count (finding 4). Run: `cargo test -p athena-core half_of_the_attempts_not_landing`. Observes: Window 0 holds 40 attempts; attempts per landing is exactly 2.0; first-attempt share is exactly 0.5; wasted spend is 10.00 USD; `Throughput::compute(..).waste` shows the same figures. The no-failure case gives 1.0, 1.0, 0.0. The 7-landing case reports window 0 with landings == 7 and no previous window. Adding a closed unlanded job with no invocations leaves attempts at 40 and `unlanded_jobs` unchanged.
  Fails without the behavior: Without `waste_windows` the test does not compile. With attempts counted as landings only, attempts per landing is 1.0; with no retry link the share is 1.0, so the 2.0 and 0.5 assertions fail. Without the guard inside `did_not_land` the no-invocation job makes attempts 41 and the guard assertion fails.
  Test seam: Production entry points `status::waste_windows` (added by step 2) and `Throughput::compute` (exists, status.rs ~984). Fixtures `timed_job`, `loaded`, `invocation` exist (status.rs:3261, 3343, 2443); the test edits request text and invocation cost on the loaded records. No double stands in front of the logic under test.
- Criterion 2: Two tests. (a) Pure `throughput_due(landed, median, waste, latest)` with `earlier_review_with_waste(10, 10.0, Some(attempts 1.5, wasted 2.00, share 0.8))`. Case A: 15 landed, same median 10.0, wasted 2.50, other figures equal returns Some(Wasted). Case C: all figures equal returns None. Case B: wasted 2.50 with no landing since (10 landed) returns None; exactly 2.40 (20%) with 15 landed returns None. Case D: attempts 1.9, then share 0.6, each return Some(Wasted). Case E: previous with `waste: None` returns None. Case F: previous wasted 0.0 and current 0.5 with attempts equal returns None; with attempts up 25% returns Some(Wasted). (b) Review level: 80 landings in four windows (`seed_landed(.., 0, 80, 10.0)`, median flat), failed attempts from `seed_failed` at 1.00 USD each: windows 3, 2, 1, 0 hold 2, 5, 2, 10. The previous review (75 landings, median 10.0, stored wasted 2.00) is stored with `put_earlier_review_with_waste`; 80 - 75 is 5 landings, so cadence does not fire and only the spend rise can. `athena.throughput_review(&Quiet)` runs with `throughput_model(6.0)` injected through `athena_with`. The test asserts `waste_windows` attempt counts per window (22, 25, 22, 30 attempts) and that the median trigger would be None before the review. Run: `cargo test -p athena-core a_rise_in_wasted_spend_triggers_a_review_with_the_median_unchanged`. Observes: (a) The Some/None results above, with the median identical in A and C. (b) The record has trigger `Wasted` and `window_median_minutes` 10.0, and the model log has one entry. Spend rose 10 vs 2 (change 8) against noise 3 and floor 1.00, so the gate queues one design request and the note names wasted spend as beyond noise.
  Fails without the behavior: Without the waste argument and the Wasted branch, Case A returns None because the median is unchanged, and the review-level case returns Ok(None) because nothing is due.
  Test seam: Production `hestia::throughput_due` (signature changed in step 4) and `Athena::throughput_review` (exists, hestia.rs ~1701). Existing fixtures: `athena_with`, `seed_landed`, `put_earlier_review`, `throughput_model` (hestia.rs:4233-4346), `earlier_review` literal (4348). Step 6 adds `earlier_review_with_waste`, `put_earlier_review_with_waste` and `seed_failed(dir, athena, slot, usd)`: a Failed job built with `timed_job` starting 30 minutes after the slot's landing start, producer cost set via `invocation(..).cost`, committed with `put`. `seed_landed(.., 10.0)` landings last 10 minutes and the next starts 2 hours later (4252-4281), so each failed job closes inside its slot's window. Noise comes from the real `Throughput::compute`.
- Criterion 3: A new hestia.rs test: `athena_with`, `seed_landed(.., 0, 10, 25.0)`, `put_earlier_review_citing` (previous review whose proposal's `request_job` is the open request), `open_design_request_job` (id 20260905-000000-d000), then `seed_stages(.., 10, 11, slow_stages())` so the median rises 25 to 60 over 21 landings. The fake `throughput_model(6.0)` logs each THROUGHPUT.md it receives. It calls `athena.throughput_review(&Quiet)`. Run: `cargo test -p athena-core an_open_design_request_no_longer_stops_the_review`. Observes: The review returns a record whose note contains the open request's id, 'still open', 'design request: not enough history: 21 of 60 landings, none queued', and, for the slowdown, 'noise could not be estimated' with 'slowdown request ... queued at high priority'. The note therefore agrees with what happened: no design request was queued (open_design_requests stays 1, proposal is Some with `request_job == None`), the slowdown request was queued under the unchanged 25% rule because 21 landings give no noise estimate. The model log has exactly one entry and that THROUGHPUT.md names d000.
  Fails without the behavior: With the old branch (hestia.rs:1728-1733) the model log is empty and `proposal` is None, so the log-length, proposal and THROUGHPUT.md assertions fail.
  Test seam: Production `Athena::throughput_review`. `athena_with`, `seed_landed`, `seed_stages`, `put_earlier_review`, `open_design_request_job`, `throughput_model` exist (hestia.rs:4233-4346). `put_earlier_review_citing` is added in step 6. The model is the fake driver injected by `athena_with`; the skip removal in step 5 is the production change.
- Criterion 4: Four tests. (a) Pure `request_gate` over in-memory window figures: change 0.1 attempts against noise 0.3 gives queue false and a note containing 'within noise'; change 0.5 against 0.3 gives queue true, naming measure, change, noise, floor and 'differences'; change equal to the bar is within; a share that fell counts as worse; an improved or unmoved measure gives 'no measure worsened'; flat-history cases: older windows flat (noise 0) and window 0 worse by exactly one job (attempts per landing +0.05, share -0.05, or spend +one attempt's median cost) gives queue false with 'within noise'; two jobs' worth gives queue true; a median up by 0.5 minute over flat windows is within, up by 2 minutes beyond; under 60 landings gives queue false with 'not enough history: N of 60'; the same with no earlier review and no open request gives queue true, and with an open request false. (b) Review level on the criterion 2(b) fixture (80 landings, flat median, 1.00 USD failed attempts, windows 3/2/1 hold 2/5/2, so spend noise 3 USD, attempts noise 0.15, share and median noise 0, floors 0.05 and 1 minute) with the previous review at 75 landings citing the open d000 and a stored baseline of 2.00. Within case: window 0 holds 4 failed (spend +2, attempts +0.1) and the stored baseline 2.00 makes `Wasted` fire (asserted before the note). Beyond case: window 0 holds 10 failed (spend +8, attempts +0.4); also asserts two design requests are open and `improvement_of` lists both. (c) Clean-history case (finding 2): windows 1-3 hold 0 failed, window 0 exactly one failed run, stored baseline 0.0, previous review placed at 70 landings so 80 - 70 = 10 makes the Cadence trigger fire; the test asserts the record's trigger is `Cadence` before asserting the note. (d) Slowdown within noise: 80 landings whose window medians are 10, 22, 10, 22 minutes (noise 12), the previous review stored at median 10 with 75 landings and a window-0 median of 14 (rise 4, over 25% and a stage grew via `seed_stages`), asserting no slowdown request is queued and the note says 'slowdown request: ... within noise'; a second case with flat older windows and a 14 minute median queues the slowdown request. Run: `cargo test -p athena-core a_change_within_the_noise_queues_no_request`. Observes: Within (b): trigger Wasted, model log one entry, proposal `request_job None`, only d000 open, note contains 'within noise', 'still open' and the difference count (2). Beyond (b): one model call, a second design request queued, `improvement_of` lists d000 and the new one, note names spend and attempts as beyond noise. (c): trigger Cadence, nothing queued, note 'within noise'. (d): within-noise slowdown queues none; flat-history slowdown queues one. Pure cases give the results above.
  Fails without the behavior: Without the gate, the within-noise case still queues a request, so 'only d000 open' and the 'within noise' note fail. Without the floor, the flat-history cases (noise 0, change 0.05) queue and fail. Without the slowdown noise condition, case (d) queues a slowdown request.
  Test seam: Production `hestia::request_gate`, `hestia::slowdown_beyond_noise` (pure, step 4) and `Athena::throughput_review` (step 5). Fixtures: in-memory values for (a); for (b)-(d) the seed helpers, `seed_failed`, `seed_stages`, `put_earlier_review_with_waste`, `put_earlier_review_citing` (step 6) and the fake driver injected by `athena_with`. Window figures, noise and floors come from the real `Throughput::compute`, not from a double.
- Criterion 5: Extend crates/athena-cli/tests/status_snapshot.rs. After reading the output of `athena status --snapshot` (via `athena_in`, status_snapshot.rs:40), assert it contains 'attempts per landed change: 1.00', 'spent on attempts that did not land: 0.00 USD' and 'landed on the first attempt: 100%' for the one-landed-job fixture. In status.rs, extend `every_printed_figure_states_its_basis...` (3796) so each new figure line is followed by a basis line, and add a 40-landing test that checks the previous-20 figures and the 'over the last N landings' text print. Run: `cargo test -p athena-cli --test status_snapshot`. Observes: The binary's real output carries the three measures with figures and bases, even with one landing (partial window). `cargo run -- status --snapshot <dir>` prints the same text.
  Fails without the behavior: On today's output the three strings are absent, so the assertions fail.
  Test seam: The real binary through `athena_in` and `Throughput::text` (exist). The text lines are added by step 3. No double.
- Criterion 6: New records.rs tests: an old ThroughputReview JSON (a serialized record with the `waste` key removed) parses through `AnyRecord::parse` and `check_record_file` with `waste == None`; a review with figures round-trips equal; NaN, negative, share above 1 and attempts below 1 are refused by `validate`; the `Wasted` trigger serializes as 'wasted'. Then the whole suite, clippy and fmt. Run: `cargo test --locked`. Observes: All tests pass; `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` print nothing.
  Fails without the behavior: Without `#[serde(default)]` on `waste`, the old-JSON parse fails with a missing field; without `Wasted` in the trigger enum a new record is refused.
  Test seam: `AnyRecord::parse` (records.rs ~2951) and `check_record_file` (~3111) with in-memory JSON strings; both exist.
- Criterion 7: Read the CHANGELOG.md diff. One Unreleased entry lists only: the three measures in the throughput section of `athena status`; the `Wasted` trigger; the open-request note and the review no longer skipping; the noise rule for design requests including the one-job floor, nothing queued below 60 landings except the first review, and the blind spot for steady trends; the slowdown request following the noise rule at 60+ landings and the 25% rule alone below; the optional `waste` field on the throughput-review record. Each sentence maps to a hunk in hestia.rs, status.rs or records.rs. No entry for display.rs or snapshot.rs, which are not changed. Run: `git diff -- CHANGELOG.md`. Observes: Nothing is claimed that the diff does not do.
- Criterion 8: Inspect the diff of the docs and Grep for the old wording. Updated: the hestia.rs module doc (~43-62), the `THROUGHPUT_MARKER` doc (1296-1298), the `open_throughput_request` doc (~1467), the `throughput_review` doc (1690-1700, including '(none while an earlier one is open)'), the `compare_with_previous_review` doc (1829-1838) and the note text at 1730; docs/design/hestia.md lines ~288 and 300-301 and section 12; the docs of `ThroughputProposal.request_job` and `ThroughputReview.proposal` in records.rs (~2159, ~2196). Run: `git diff -- crates/athena-core/src/hestia.rs docs/design/hestia.md crates/athena-data/src/records.rs`. Observes: Grep for 'queues no second one and does not ask the model', 'none while an earlier one is open', 'the model was not asked for another' and 'the review queues no second one' finds none; hestia.md section 12 describes the three measures, the Wasted trigger, the window gate with its noise basis, floors and the steady-trend blind spot, the slowdown noise condition, and the note instead of a skip.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-cli/tests/status_snapshot.rs (modify)
- CHANGELOG.md (modify)
- docs/design/hestia.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2ad155a5bbe6 → candidate ff1397eb61fd. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   20 +
 crates/athena-cli/tests/status_snapshot.rs |    8 +
 crates/athena-core/src/hestia.rs           | 1101 +++++++++++++++++++++++++---
 crates/athena-core/src/status.rs           |  610 ++++++++++++++-
 crates/athena-data/src/records.rs          |  117 ++-
 docs/design/hestia.md                      |   71 +-
 6 files changed, 1794 insertions(+), 133 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-031925-mrpk/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-031925-mrpk/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42347964014 for ff1397eb61fdf78936034dfd7fdd8a2bc47add7e)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42347964014 for ff1397eb61fdf78936034dfd7fdd8a2bc47add7e)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42347964014 for ff1397eb61fdf78936034dfd7fdd8a2bc47add7e)
- ci-result: `not a check` passed (Check results for ff1397eb61fdf78936034dfd7fdd8a2bc47add7e come from the CI run https://github.com/doodla/athena/actions/runs/42347964014 (591s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass; an old ThroughputReview JSON without the new fields still parses via `AnyRecord::parse`, and a new one round-trips.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core half_of_the_attempts_not_landing`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
    --> crates/athena-core/src/status.rs:4100:31
     |
4100 |         assert_eq!(throughput.bars.median_minutes.bar(), Some(1.0));
     |                               ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `median_previous` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4101:31
     |
4101 |         assert_eq!(throughput.median_previous, Some(20.0));
     |                               ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4122:26
     |
4122 |         assert_eq!(three.bars.wasted_usd.noise, Some(3.0));
     |                          ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4123:26
     |
4123 |         assert_eq!(three.bars.wasted_usd.differences, 1);
     |                          ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4125:24
     |
4125 |         assert_eq!(two.bars.wasted_usd.noise, None);
     |                        ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `waste_previous` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4126:21
     |
4126 |         assert!(two.waste_previous.is_some());
     |                     ^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0599]: no method named `measures` found for struct `status::Throughput` in the current scope
    --> crates/athena-core/src/status.rs:4127:28
     |
 736 | pub struct Throughput {
     | --------------------- method `measures` not found for this struct
...
4127 |         let measures = two.measures();
     |                            ^^^^^^^^ method not found in `status::Throughput`

Some errors have detailed explanations: E0061, E0422, E0425, E0432, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0061`.
error: could not compile `athena-core` (lib test) due to 69 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_rise_in_wasted_spend_triggers_a_review_with_the_median_unchanged`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4100:31
     |
4100 |         assert_eq!(throughput.bars.median_minutes.bar(), Some(1.0));
     |                               ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `median_previous` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4101:31
     |
4101 |         assert_eq!(throughput.median_previous, Some(20.0));
     |                               ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4122:26
     |
4122 |         assert_eq!(three.bars.wasted_usd.noise, Some(3.0));
     |                          ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4123:26
     |
4123 |         assert_eq!(three.bars.wasted_usd.differences, 1);
     |                          ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4125:24
     |
4125 |         assert_eq!(two.bars.wasted_usd.noise, None);
     |                        ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `waste_previous` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4126:21
     |
4126 |         assert!(two.waste_previous.is_some());
     |                     ^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0599]: no method named `measures` found for struct `status::Throughput` in the current scope
    --> crates/athena-core/src/status.rs:4127:28
     |
 736 | pub struct Throughput {
     | --------------------- method `measures` not found for this struct
...
4127 |         let measures = two.measures();
     |                            ^^^^^^^^ method not found in `status::Throughput`

Some errors have detailed explanations: E0061, E0422, E0425, E0432, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0061`.
error: could not compile `athena-core` (lib test) due to 69 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core an_open_design_request_no_longer_stops_the_review`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4100:31
     |
4100 |         assert_eq!(throughput.bars.median_minutes.bar(), Some(1.0));
     |                               ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `median_previous` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4101:31
     |
4101 |         assert_eq!(throughput.median_previous, Some(20.0));
     |                               ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4122:26
     |
4122 |         assert_eq!(three.bars.wasted_usd.noise, Some(3.0));
     |                          ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4123:26
     |
4123 |         assert_eq!(three.bars.wasted_usd.differences, 1);
     |                          ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `bars` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4125:24
     |
4125 |         assert_eq!(two.bars.wasted_usd.noise, None);
     |                        ^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0609]: no field `waste_previous` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:4126:21
     |
4126 |         assert!(two.waste_previous.is_some());
     |                     ^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 5 others

error[E0599]: no method named `measures` found for struct `status::Throughput` in the current scope
    --> crates/athena-core/src/status.rs:4127:28
     |
 736 | pub struct Throughput {
     | --------------------- method `measures` not found for this struct
...
4127 |         let measures = two.measures();
     |                            ^^^^^^^^ method not found in `status::Throughput`

Some errors have detailed explanations: E0061, E0422, E0425, E0432, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0061`.
error: could not compile `athena-core` (lib test) due to 69 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core a_change_within_the_noise_queues_no_request`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-cli --test status_snapshot`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
