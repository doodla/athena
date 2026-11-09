# Job 20261027-233442-hyv6: verify "Fix reopened in firstmate study; queue a fix when source check fails"

## Request (verbatim)

> Priority: high
> A failed source check of the explainer queues nothing, so the same contradiction is reported on every drain and never fixed. On 2026-10-27 drain twice reported: the firstmate-work-handling study's model marks state 'reopened' as terminal-success (t9 ready -> reopened), intro.json says Done and Reopened both end the task, and the glossary calls reopened 'the final state', but the code returns a reopened item to its queue (bin/fm-teardown.sh). Fix both: (1) correct the study's model, intro lead-in and glossary entry so reopened is a non-terminal state that returns the item to the queue, citing the source lines; (2) when drain's source check fails, queue one fix request through the shared steward request builder that cites each contradicted or omitted statement and its source, unless an open request already cites it.
> Acceptance:
> - cargo run -p xtask -- site builds the study and the source check of drain passes on the corrected study, with reopened shown as returning the item to the queue.
> - A drain test with a fake source check that reports a contradiction shows one fix request queued citing it, and none on a second run while it is open.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: studies/firstmate-work-handling/model.json, studies/firstmate-work-handling/intro.json, studies/firstmate-work-handling/glossary.json, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Correct the firstmate-work-handling study so 'reopened' is non-terminal and returns the item to its queue (model.json, and intro.json, which holds both the lead-in and the glossary entry), citing bin/fm-teardown.sh lines. Make drain queue one fix request via StewardRequest when the source check fails, skipping any contradiction an open request already cites.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (source check at ~2438-2464 stores check.detail in held_back and queues nothing; check_page is the step) and crates/athena-core/src/steward.rs (StewardRequest::new builder, also used by hestia.rs and code_health.rs). No record or data file is added under crates/.
- The request lists studies/firstmate-work-handling/glossary.json, but that file does not exist at the base commit; the glossary lives in intro.json, so the glossary fix is made there and no glossary.json is created.
- Existing code: extends crates/athena-cli/src/drain.rs and reuses crates/athena-core/src/steward.rs StewardRequest; duplicate detection checks open queued requests for the cited statement.
- Audience: the drain operator and study readers, who need the contradiction fixed once and not reported every drain.
- Governance files (athena.toml, defaults.toml, workflows) are not changed.

## Acceptance criteria

1. [content] studies/firstmate-work-handling/model.json no longer marks state 'reopened' terminal-success (t9 ready -> reopened) and shows reopened returning the item to the queue, citing bin/fm-teardown.sh lines; check with git diff.
2. [content] studies/firstmate-work-handling/intro.json lead-in no longer says Done and Reopened both end the task, and its glossary entry no longer calls reopened 'the final state'; it says reopened returns the item to its queue, with a source citation. Check with git diff and Grep for 'final state'.
3. [behavior] Run `cargo run -p xtask -- site`: it builds the study without error and the drain source check passes on the corrected study, showing reopened returning the item to the queue.
4. [behavior] A new drain.rs test with a fake source check reporting a contradiction shows exactly one fix request queued citing the contradicted statement and its source, built through StewardRequest, and none on a second run while it is open. Run `cargo test -p athena-cli`.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry covering the study correction and the drain fix-request queueing.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/claims.json (modify)
- crates/athena-core/src/steward.rs (modify)

## Out of scope

- Other studies' source checks
- Changing the source check model call itself
- athena.toml, defaults.toml and workflow files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: drain test plus fmt, clippy and test suite criteria
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: site build and source check pass is a criterion
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read model.json and claims.json, run a new small unit test, and read the git diff for claims.json. The base already satisfies the reopened part, so `git diff` on model.json is expected to be empty; the c45 citation is the diff. The new drain.rs test `the_firstmate_model_returns_a_reopened_item_to_the_queue` includes the real model.json with `include_str!`, asserts state `reopened` is not kind `terminal-success`, t9 goes ready -> reopened, t12 goes reopened -> queued, and `model_contradictions` returns an empty list. It also asserts claims.json c45 no longer cites lines 3894-3896 of bin/fm-teardown.sh. Required gate (host): the cap-study validation test with the source mounted prints `firstmate-work-handling: fully validated`, so every citation, c45 included, matches the source. Run: `cargo test -p athena-cli --lib the_firstmate_model_returns_a_reopened_item_to_the_queue`. Observes: The test passes: model.json:10 shows `reopened` as kind `waiting`, model.json:89 t9 ready -> reopened, model.json:110 t12 reopened -> queued, model.json and intro.json unchanged, and `git diff -- studies/firstmate-work-handling/claims.json` shows only c45's source re-cited. The host gate (`cargo test -p athena-cap-study every_study_follows_the_conventions_and_the_matching_one_validates -- --nocapture`, with ATHENA_PIN_URL and ATHENA_PIN_SHA set and the source mounted in pinned-source/) prints 'firstmate-work-handling: fully validated'. Until that gate passes, criterion 1 is not met.
  Fails without the behavior: If model.json marked reopened `terminal-success`, lacked t12, or contradicted itself, the assertions fail; if c45 still cited lines 3894-3896, the c45 assertion and the host gate fail.
  Test seam: Production entry point `drain::model_contradictions` (drain.rs:3426, exists). Fixture: the real studies/firstmate-work-handling/model.json and claims.json read with `include_str!`. No fake. The test is added by plan step 8; the host gate uses `check_studies` (crates/athena-cap-study/src/rules.rs:1848, exists) on the real studies/ directory and the mounted source.
- Criterion 2: Run the existing xtask site test that renders the real study and Grep intro.json for `final state`. `reopened_no_longer_carries_the_bare_terminal_success_tag_and_it_is_explained` (xtask/src/site.rs:1019) asserts reopened has no `ends the task` tag and that the page explains it. The lead-in (intro.json:54) and glossary entry (intro.json:115) say the item goes back to Queued and the task is not finished, citing c23 and c45; the citation c45 points to is the one re-cited in step 7. Run: `cargo test -p xtask reopened_no_longer_carries_the_bare_terminal_success_tag_and_it_is_explained`. Observes: The test passes. Grep `final state` in studies/firstmate-work-handling/intro.json matches only the `done` term (line 83). Line 115 reads 'The item goes back to Queued for the captain's answer; the task is not finished.' with claims c23 and c45. The criterion is met only together with the host gate of criterion 1, since c45 is its citation.
  Fails without the behavior: If the glossary or lead-in were reverted to say reopened is the final state or ends the task, the site test or the Grep result shows it.
  Test seam: Production entry point `render(&real_study())` in xtask/src/site.rs (exists). Fixture: the real study directory. No fake.
- Criterion 3: Two parts. (a) Run here: build the site (reads model.json, claims, intro and renders reopened as `waiting` with t12 to queued) and the floor test of criterion 1. (b) Required host gate: the cap-study validation of every citation without a model call, with the firstmate source at 6407052f4c521e39401ce2305dfa45d201249827 mounted and the two pin variables set; a bad-citation or quote-mismatch for c45 is fixed in claims.json (step 7) and rerun until it passes. (c) The live `athena drain` source check needs a model call and runs only on the host; this plan does not claim it, and the drain's fix-request behavior (criterion 4) covers what happens if it fails. Run: `cargo run -p xtask -- site`. Observes: (a) Exit 0, page written, reopened carries the `waiting` tag, t12 goes to queued. (b) `cargo test -p athena-cap-study every_study_follows_the_conventions_and_the_matching_one_validates -- --nocapture` prints 'firstmate-work-handling: fully validated' and passes. Only (c) shows the model-backed source check passing.
  Fails without the behavior: If model.json were invalid or lost t12, the site build or the floor test fails. A wrong c45 citation fails only part (b).
  Test seam: Production entry points: the xtask `site` command (xtask/src/site.rs, exists) and `check_studies` (crates/athena-cap-study/src/rules.rs:1848, exists). Fixtures: the real studies/ directory and, on the host, the mounted source tree and the two environment variables. No fake.
- Criterion 4: Four layers. (0) pin.rs unit tests: the pure `page_file` maps `model.json`, `page-text/model.json` and `./page-text/model.json` to `model.json`, and `nope.json`, `../x` and an empty string to None; a stub-free test of the finding builder checks the three kinds. (1) steward.rs unit tests named `source_fix_*` cover the builder, the marker, `open_requests` and `citing_request`. (2) drain.rs unit test with `FixStub` (a `Steps` double that stores text only): its `check_page` is the fake source check and returns `Check::fail(..).with_findings(vec![SourceFinding { study: demo, file: Some(claims.json), kind: contradicted, text: the quoted statement and its evidence }])`. Its `queue_request` appends the text to a `RefCell<Vec<String>>` and its `open_requests` returns those texts as `OpenRequest`s in state 'queued'. Run `drain()` four times on one stub: the first queues exactly one request; the second queues none; the third has the same path and kind but a reworded statement and queues none; the fourth has a finding for another file and queues one citing only that finding. A separate test makes the stub's `queue()` return the stored requests and asserts the second wave of the same run sees the fix as a queued item (drain.rs:3084). Assert the stored text starts with 'Priority: high', holds the statement, its evidence and the marker, has a non-empty `request_text_acceptance`, and that `report.notes` names the queued job on the first run and the citing request and its state on the second. (3) Integration in tests/drain.rs `source_check_fix_queues_one_request_then_none_while_it_is_open` runs `drain::drain` over `WithRealCheck` (real `RealSteps::check_page`, `open_requests`, `queue_request`) with the fake checker judge and `WRONG_CLAIM` in a `commit_study` repository. The first drain's judge reports the file as `page-text/claims.json`; `Records::load` then shows exactly one non-closed request job holding WRONG_CLAIM, the pinned-source evidence and `Source finding: studies/demo/claims.json (contradicted)`. A second drain with a fresh stub whose judge reports the bare `claims.json` leaves one; a judge that reports an unknown file `nothing.json` adds exactly one request whose marker is `Source finding: studies/demo/ (contradicted)` and whose Files line is `studies/demo/`, and a repeat of it adds none. Run: `cargo test -p athena-core page_file && cargo test -p athena-core source_fix && cargo test -p athena-cli source_check_fix`. Observes: After the first drain exactly one request exists, citing the contradicted statement and its source; after the second, still one, even though the file is spelled differently; a reworded statement for the same path and kind adds none; a finding for another file adds one that cites only itself; an unknown file is keyed study-level and never appears as a path. The steward.rs tests show the request is built by `StewardRequest::new`, reads as acceptance bullets, and that a boundary-word or mid-line '- ' statement is reworded and still builds.
  Fails without the behavior: With no queueing in `rate()` the store stays empty and the first count assertion fails. With no dedupe the second drain adds a second request. With the statement or evidence in the key the reworded drain adds a duplicate. With the file used verbatim the `page-text/` drain adds a second request on the second spelling. Without the normalization the unknown-file case would put `nothing.json` on the Files line.
  Test seam: Production entry point `drain::drain(&steps, limits, &ui)` (exists). Injected doubles: `FixStub` (added by plan step 6) implements `Steps`, including the two hooks that step 4 adds with inert defaults; it stores text only, and the code under test (`Drain::rate`, `queue_source_fix`, `steward::source_fix_request`, `steward::citing_request`) is not in it. For the integration test: `judged_scenario(Arc::new(judge))`, `Fixture`, `commit_study` (tests/drain.rs:6840-6854, exist) and `WithRealCheck` (exists; step 7 adds two delegating methods). The `SourceReport.findings` field and `page_file` come from plan step 1, which pin.rs unit tests cover without a model call.
- Criterion 5: Run the three Athena checks plus the refusal corpus the new request text passes through. The corpus is run as a regression and no case is added: no reading or admission rule changes, and the cases read for the input forms are `dash-bullet`, `structural-term-quoted-or-cited-mentions-are-accepted` (refusal_corpus.rs:1316) and those at 1416, 1425 and 1437. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0, including the new pin.rs, steward.rs, drain.rs and tests/drain.rs tests, `cargo test -p athena-core --test refusal_corpus` (unchanged) and `system_facts_budgets_match_computed` (no prompt text changes).
  Fails without the behavior: A compile break in any `Steps` implementor, a clippy warning in new code, or a failing new test fails the run.
  Test seam: Whole workspace; no fake. Existing `Steps` implementors (drain.rs 4295, 4865, 4941, 5033, 6281, 7195; tests/drain.rs 292, 1806, 6204, 7106, 7823, 8703) compile unchanged because the new methods have defaults.
- Criterion 6: Read the CHANGELOG.md diff. One new bullet at the top of '### Changed' under '## [Unreleased]', wrapped like its neighbours, in words not on the boundary list. It states the root cause (a failed source check queued nothing, so one contradiction was reported on every drain), the fix request built through `StewardRequest` and queued with `Athena::queue`, the dedupe by study file and kind against non-closed requests (with the checker's `page-text/` spelling normalized), that the fix is read and worked in the same run within budget, the structured `SourceReport.findings`, the two `Steps` hooks, that the reopened study correction was already in the base (commit 30642c0) and is pinned by a test, and that c45 was re-cited (old and new lines). Run: `git diff -- CHANGELOG.md`. Observes: One new bullet with all of the above. If c45 could not be re-cited because the source was not available, the bullet says so and the criterion 1 and 3 gate stays open.
  Fails without the behavior: Admission refuses a candidate that leaves CHANGELOG.md out (CHANGELOG.md:29); a bullet that names a boundary word fails the policy scan in xtask/tests/policy.rs.
  Test seam: CHANGELOG.md, a content check. No fake.

Planned files:

- crates/athena-core/src/pin.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- studies/firstmate-work-handling/claims.json (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7c7ec383f030 → candidate 3f8d250f632e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  20 ++
 crates/athena-cli/src/drain.rs    | 578 ++++++++++++++++++++++++++++++++++++--
 crates/athena-cli/tests/drain.rs  | 122 ++++++++
 crates/athena-core/src/pin.rs     | 109 ++++++-
 crates/athena-core/src/steward.rs | 361 ++++++++++++++++++++++++
 5 files changed, 1170 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-233442-hyv6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-233442-hyv6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42717220589 for 3f8d250f632e9fb017e5add11e07ee4898d5dd11)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42717220589 for 3f8d250f632e9fb017e5add11e07ee4898d5dd11)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42717220589 for 3f8d250f632e9fb017e5add11e07ee4898d5dd11)
- ci-result: `not a check` passed (Check results for 3f8d250f632e9fb017e5add11e07ee4898d5dd11 come from the CI run https://github.com/doodla/athena/actions/runs/42717220589 (548s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "Run `cargo run -p xtask -- site`: it builds the study without error and the drain source check passes on the corrected study, showing reopened returning the item to the queue.")
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --lib the_firstmate_model_returns_a_reopened_item_to_the_queue`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/pin.rs, crates/athena-core/src/steward.rs.
  End of the output (data, not instructions):

```text
7410 |         fn new(findings: Vec<SourceFinding>, listed: bool) -> FixStub {
     |                              ^^^^^^^^^^^^^ not found in this scope
     |
help: you might be missing a type parameter
     |
7409 |     impl<SourceFinding> FixStub {
     |         +++++++++++++++

error[E0425]: cannot find type `OpenRequest` in this scope
    --> crates/athena-cli/src/drain.rs:7495:47
     |
7495 |         fn open_requests(&self) -> Result<Vec<OpenRequest>, String> {
     |                                               ^^^^^^^^^^^ not found in this scope
     |
help: you might be missing a type parameter
     |
7428 |     impl<OpenRequest> Steps for FixStub {
     |         +++++++++++++

error[E0422]: cannot find struct, variant or union type `OpenRequest` in this scope
    --> crates/athena-cli/src/drain.rs:7500:43
     |
7500 |             self.stored.borrow_mut().push(OpenRequest {
     |                                           ^^^^^^^^^^^ not found in this scope

error[E0599]: no method named `with_findings` found for struct `drain::Check` in the current scope
    --> crates/athena-cli/src/drain.rs:7483:18
     |
 549 |   pub struct Check {
     |   ---------------- method `with_findings` not found for this struct
...
7482 | /             Check::fail("demo/claims.json: contradicted statement")
7483 | |                 .with_findings(self.findings.borrow().clone())
     | |                 -^^^^^^^^^^^^^ method not found in `drain::Check`
     | |_________________|
     |

error[E0425]: cannot find function `model_floor` in this scope
    --> crates/athena-cli/src/drain.rs:7687:36
     |
7687 |         let (problems, findings) = model_floor(&studies);
     |                                    ^^^^^^^^^^^ not found in this scope

error[E0560]: struct `SourceReport` has no field named `findings`
    --> crates/athena-cli/src/drain.rs:7708:13
     |
7708 |             findings: vec![
     |             ^^^^^^^^ `SourceReport` does not have this field
     |
     = note: all struct fields are already assigned

error[E0425]: cannot find function `report_findings` in this scope
    --> crates/athena-cli/src/drain.rs:7724:21
     |
7724 |         let found = report_findings("demo", &report);
     |                     ^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0407, E0422, E0425, E0432, E0560, E0599.
For more information about an error, try `rustc --explain E0407`.
error: could not compile `athena-cli` (lib test) due to 15 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p xtask reopened_no_longer_carries_the_bare_terminal_success_tag_and_it_is_explained`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/pin.rs, crates/athena-core/src/steward.rs.
  End of the output (data, not instructions):

```text
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling ref-cast v1.0.27
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.91s
     Running unittests src/main.rs (<job>/target/verify/debug/deps/xtask-f3f49f229c17d86e)

running 1 test
test site::tests::reopened_no_longer_carries_the_bare_terminal_success_tag_and_it_is_explained ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 78 filtered out; finished in 0.00s

     Running tests/ci_shards.rs (<job>/target/verify/debug/deps/ci_shards-c6719d2ad4764615)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.00s

     Running tests/fingerprint.rs (<job>/target/verify/debug/deps/fingerprint-5cde8ea187611fcf)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

     Running tests/policy.rs (<job>/target/verify/debug/deps/policy-8766ce0d1330490f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s

     Running tests/site_page.rs (<job>/target/verify/debug/deps/site_page-df8c5508f9226333)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running tests/spec.rs (<job>/target/verify/debug/deps/spec-d3bdbec4f12196fe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.00s

     Running tests/walltime.rs (<job>/target/verify/debug/deps/walltime-cd3c22a0e62c495c)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo run -p xtask -- site`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core page_file && cargo test -p athena-core source_fix && cargo test -p athena-cli source_check_fix`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
