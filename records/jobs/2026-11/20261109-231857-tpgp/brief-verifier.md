# Job 20261109-231857-tpgp: verify "Retry 20261027-233442-hyv6: drain fix request and verified reopened"

## Request (verbatim)

> try again on 20261027-233442-hyv6

## Reading (accepted by the user)

Retry of job 20261027-233442-hyv6: keep its drain fix-request work (steward.rs, drain.rs, pin.rs, tests/drain.rs, CHANGELOG.md), fix its findings, and make the reopened-returns-to-Queued citation (c45) checked. The study's model, intro lead-in and glossary already say reopened returns the item to Queued, so those are verified, not rewritten.

Assumptions:
- Prior job 20261027-233442-hyv6 addressed. (a) Major c45 finding: re-cite c45 in claims.json from bin/fm-teardown.sh where the reopen path sets the row to Queued, read from the pinned source (via the study's source-reading tooling), and confirm the study validates with the pinned source, not 'conventions only'. If the pinned source is absent in this copy, the quote must be one that the checker can match, and the CHANGELOG must say what was not checked. (b) Minor duplicate-key finding: tighten the steward.rs finding marker to include the claim id or cited source path and lines, not only file and kind, so a new contradiction in the same file is queued while the same one is skipped. (c) Minor doc-comment finding: move the study_text doc comment back directly above study_text in drain.rs. (d) Minor scope finding: pin.rs and crates/athena-cli/tests/drain.rs are kept and listed as deliverables.
- Part 1 of the original request is already in the base: model.json t9 goes ready -> reopened with t12 reopened -> queued, and the intro.json glossary says the item goes back to Queued. The producer should only change model.json or intro.json if Grep for 'final state' or terminal-success finds a leftover; the lead-in in intro.json line 54 should be checked for 'Done and Reopened both end the task'.
- Code survey: read crates/athena-cli/src/drain.rs (study_text at ~3496, source check and report handling), crates/athena-core/src/steward.rs (StewardRequest builder, source_fix_request, finding_marker, citing_request) and crates/athena-core/src/pin.rs (SourceReport, SourceProblem). The new studies/ files are not records under crates/, so no check_record_file or AnyRecord change is needed.
- Goal: the drain source check stops repeating the same contradiction. Audience: the maintainer who reads drain reports and the verifier who runs cargo.
- Environment: none beyond the local repository; no CI workflow, push or history read.

## Acceptance criteria

1. [content] claims.json claim c45 cites bin/fm-teardown.sh lines where the reopen path returns the row to Queued, with a quote that is present in those lines; git diff shows the lines changed from 3894-3896 or the CHANGELOG states the check result.
2. [content] Grep for 'final state' and 'both end the task' in studies/firstmate-work-handling finds no claim that reopened is terminal; model.json shows reopened -> queued.
3. [behavior] Run `cargo run -p xtask -- site`: it builds the study without error and the drain source check passes on the corrected study.
4. [behavior] Run `cargo test -p athena-cli` and `cargo test -p athena-core`: the drain test with a fake source check reporting a contradiction shows exactly one fix request queued through StewardRequest citing the statement and source, none on a second run while it is open, and a different contradiction in the same file is still queued.
5. [content] drain.rs has the study_text doc comment directly above `pub fn study_text`, checked with git diff.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has an entry covering the study correction, the c45 citation and the drain fix-request queueing.

## Expected deliverables

- studies/firstmate-work-handling/claims.json (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/pin.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)

## Out of scope

- Other studies' source checks
- Changing the source check model call itself
- athena.toml, defaults.toml and workflow files

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the source check passes and the duplicate request is not repeated.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Drain test criterion checks queuing behavior.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check of the diff. If the producer could read the pinned source, c45 cites the lines in bin/fm-teardown.sh where the reopen path sets the row to Queued, each quote copied from them. Otherwise c45 cites the two validated lines, with quotes identical to claims.json:169 and :180. Run: `git diff -- studies/firstmate-work-handling/claims.json CHANGELOG.md`. Observes: The c45 hunk removes start_line 3894 / end_line 3896 and the quote 'Queued'. Each new quote is a verbatim copy of a quote already in claims.json at the cited line, or of the source line read. The CHANGELOG hunk states the route the conventions test printed and, if the source was not read, that the cited lines do not say Queued and the Queued return was not checked.
- Criterion 2: Use Grep for 'final state', 'both end the task' and 'terminal' in studies/firstmate-work-handling, then Read model.json t9, t12 and intro.json line 54. In this copy the hits are intro.json:83 (the `done` glossary entry) and model.json:9 (`done` as terminal-success). `reopened` is `waiting` and t12 goes reopened to queued. Run: `git diff --stat -- studies/firstmate-work-handling/model.json studies/firstmate-work-handling/intro.json`. Observes: No line says reopened is final or that Done and Reopened both end the task. The diff stat is empty for both files, or shows only the removal of a leftover.
- Criterion 3: (a) The site build renders the study. (b) The conventions test with --nocapture prints the route for firstmate-work-handling (crates/athena-cap-study/src/rules.rs:1841-1859). With no pinned-source/ and no ATHENA_PIN_* it prints 'conventions only'; only a printed 'fully validated' shows the c45 quotes matched the pinned tree. Otherwise the source-check half of this criterion is unshown and the CHANGELOG says so. (c) The extended passing-path drain test shows only that a clean checker report passes and queues nothing; it is not evidence about the corrected study. Run: `cargo run -p xtask -- site && cargo test -p athena-cap-study every_study_follows -- --nocapture && cargo test -p athena-cli --test drain a_page_with_correct_text_in_every_file_passes_the_source_check_with_one_verifier_call`. Observes: The site build exits 0. The conventions test passes and prints the route. The passing-path test passes: the check is not failed, the roles are [Verifier], and `Records::load` at head holds no request job whose first line starts 'Source fix: '.
  Fails without the behavior: A malformed c45 (start after end, empty quote, wrong sha) fails the site build and the conventions test. If a clean report queued a request or failed the check, the extended passing-path test fails on its no-request assertion or on `check.passed`.
  Test seam: Existing entry points: `xtask site` (xtask/src/main.rs) and the conventions test (rules.rs:1841). For the drain, the production `RealSteps::check_page` (drain.rs:4666), which calls `Athena::check_study_text`. The fake is the `SourceJudge` closure in `Scenario.source_check` (tests/common/mod.rs), injected through `fx.athena(judged_scenario(judge).script())` (tests/drain.rs:6850), exists today. Plan step 11 adds the helper `check_in(fx, judge)` that `real_check` delegates to, so the test keeps its Fixture and can read `Records::load(&fx.git, &fx.head())`.
- Criterion 4: New drain tests in tests/drain.rs, calling the production `RealSteps { athena, ui: &Quiet, page: &StubPage, lander }.check_page()` with a counting `SourceJudge` fake. Each builds its own claims.json through `commit_study` with its own files: c1 'Teardown closes a task, and a closed task can reopen.' and c2 'A worker slot is released when its pull request is ready.' (the shared Texts claims.json has only c1). Test 1 `source_fix_queues_one_request_per_contradiction_and_none_while_open`: run 1 reports one `contradicted` problem on claims.json whose statement is a fragment of c1's text with no id and evidence 'pinned-source/bin/x.sh lines 30-33'; run 2 reports the same fragment with different case and spacing and different evidence; run 3 reports that problem plus a c2 fragment. Expected: one job whose first line is `Source fix: studies/demo/claims.json contradicted c1` after runs 1 and 2, two after run 3, the second ending ` c2`; run 1's text has the statement, the defanged evidence, an `Acceptance:` section and `Files: studies/demo/claims.json`; `request_text_touches` is only that file; `steward::citing_request` returns the job; the Check is still failed with `demo/claims.json` and the statement in its detail; and the production `steps.queue()` lists the job in items and plan with no open question. Test 2 `source_fix_is_not_queued_again_after_its_job_lands_in_the_same_run`: after run 1, helper `close_request(fx, athena, job, OutcomeState::Landed)` commits an `AnyRecord::Outcome` (as seed_repeated_cause does, tests/drain.rs:4464-4519; Outcome fields as in tests/reconcile.rs:62-79); the fake still reports the contradiction; check_page again leaves exactly one request job, none open, and `steps.queue()` does not list it. Test 3 `source_fix_closed_without_landing_is_queued_by_a_later_run_only`: in a fresh fixture seed a request with `athena.queue(&source_fix_request(&finding))` (hand-built finding, anchor 'c1') and close it as Failed; the first check_page queues one new request (this process has not queued the marker for this directory); closing it and calling check_page again queues none. Test 4 `source_fix_process_failure_queues_nothing`: a 'did not report reading this file' problem queues nothing. For finding 1, Test 5 `source_fix_marker_is_carried_across_a_rebuild`: in a fresh fixture, run `drain::run_pass(root, &Quiet, || { real.check_page()...; drain::drain(&rebuilding stub, limits, &Quiet) }, &|_| { read drain::carry_file(root) with serde_json into drain::Carried; assert source_fix_queued holds (athena.git.dir(), marker); Ok(report of a second plain stub drain) })`; one request exists. Test 6 `source_fix_marker_in_the_carry_is_not_queued_again_after_the_restart`: in a fresh fixture whose directory this process has never queued for, `drain::store_carry(root, &Carried { source_fix_queued: vec![(dir, marker)], ..Carried::default() })`, then `drain::run_pass(root, ...)` whose pass runs check_page with the fake still reporting the contradiction; no request job exists afterwards (the marker can only have come from the carry file, since the static is empty for that directory). Test 7 `source_fix_a_fresh_drain_ignores_a_stale_carry`: `guarded` with a stale carry holding the marker (pid file not naming this process) removes the carry, so check_page queues one request. Test 8 `a_carry_without_the_field_still_reads`: an old carry JSON lacking `source_fix_queued` deserialises via take_carry. Core unit tests: pin.rs `source_finding_*` and steward.rs `source_fix_*`, including pin test `source_finding_a_short_unrelated_definition_is_not_credited` (a 20-character glossary definition contained in a 100-character statement, under half its length, is not an owner; the anchor is the real owner or the hash) and `source_finding_a_fragment_of_one_entry_is_credited` (candidate contains statement). Run: `cargo test -p athena-cli --test drain source_fix && cargo test -p athena-core source_f && cargo test -p athena-cli && cargo test -p athena-core`. Observes: One request after run 1, still one after run 2 although case, spacing and evidence changed and no id was named, two after run 3. No second request after the first job lands with the contradiction still reported. A failed request is re-queued by a later run only. steps.queue() lists the new job. The carry file holds the queued marker before the restart, and a marker restored from the carry is not queued again. A stale carry on a fresh start is dropped. The pin and steward unit tests pass.
  Fails without the behavior: Without dedup, run 2 makes a second request. With a key of file and kind only, run 3's c2 is skipped (count 1). With a key on evidence or the raw statement, run 2 makes a second request. With 'open only' dedup, test 2 finds a second request. With no queueing, the count after run 1 is 0. With a raw marker, the unit test comparing it to the first line of the built request fails. With direct-fields-only matching, the nested-entry anchor test fails. With either-way containment, the short-definition test fails. With the static but no carry field, test 5 fails (carry lacks the marker) and test 6 queues a request.
  Test seam: Production entry points: `RealSteps::check_page` (drain.rs:4666), which calls `Athena::check_study_text` then `Athena::queue` (reading.rs:919); `RealSteps::queue` (drain.rs:4339) for the pickup; `drain::run_pass` (drain.rs:3303) and `drain::guarded` (3842) with the real carry file in a tempdir. Double: the `SourceJudge` closure in `Scenario.source_check`, injected as `judged_scenario` does (tests/drain.rs:6850), and the `Stub`/`rebuilding` steps used by `whole_run` (tests/drain.rs:2075) for the pass; both exist today and return only the checker's JSON report or canned steps. The marker, anchor, dedup, run set, carry fill and restore are production code. `SourceFinding`, `findings_of`, `finding_anchor`, `finding_marker`, `source_fix_request`, `citing_request`, the run set, the `Carried.source_fix_queued` field and the check_page and run_pass changes are added by plan steps 3-9. The tests and the `check_in`, `close_request` helpers are added by steps 10-13.
- Criterion 5: Read the diff. In the base the `/// A study's page text and pin, from its files ...` block (drain.rs:3493-3495) is directly above `pub fn study_text(` (3496). New items (the static set, helpers) go with the statics near line 799 and next to `RealSteps::check_page`. Run: `git diff -- crates/athena-cli/src/drain.rs`. Observes: No hunk adds or removes a line between the doc block and `pub fn study_text`, and no new item sits between them.
- Criterion 6: Run the three workspace checks Athena runs. They cover every Steps impl (the trait is untouched), the existing real_check-based tests that now also queue requests, the carry tests (a_rebuild_between_two_landings_reports_the_whole_run, a_fresh_drain_and_a_stale_carry_start_at_zero, a_failed_exec_leaves_no_carry_behind) and the new tests. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: An unexpected request in an existing drain test's repository, a changed `Check::detail`, a carry that no longer round-trips, an unused item, or unformatted code would fail the drain tests, clippy or fmt.
  Test seam: Existing: the cargo workspace checks.
- Criterion 7: Read the diff. One entry under [Unreleased] covers: the c45 re-citation and the check result (the route printed; whether the pinned source was read, and if not that the cited lines do not say Queued and the Queued return was not checked); the study text verified, not rewritten (model t12, glossary, lead-in); and the drain fix-request queueing (marker form, anchor from the study's own entries including nested ones, one-way containment, hash when none or more than three match, skip while open, a marker queued at most once per drain run even when its job lands and the contradiction remains, the markers carried through the drain.carry file across a rebuild re-exec, second contradiction in the same file queued, same-run pickup, doc comment kept above study_text, the limit that a drain stopped and started by hand is a new run). Run: `git diff -- CHANGELOG.md`. Observes: The entry names claims.json c45, `SourceReport.findings`, `finding_marker`, `source_fix_request`, `citing_request`, `RealSteps::check_page`, `Carried`/`run_pass`, the once-per-run rule across a rebuild and the same-run pickup.

Planned files:

- crates/athena-core/src/pin.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- studies/firstmate-work-handling/claims.json (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e760b980d7d7 → candidate 76ad159c29ea. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  34 +++
 crates/athena-cli/src/drain.rs              | 119 +++++++-
 crates/athena-cli/tests/drain.rs            | 422 +++++++++++++++++++++++++++-
 crates/athena-core/src/pin.rs               | 375 +++++++++++++++++++++++-
 crates/athena-core/src/steward.rs           | 201 +++++++++++++
 studies/firstmate-work-handling/claims.json |   3 +-
 6 files changed, 1147 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-231857-tpgp/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-231857-tpgp/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42726589827 for 76ad159c29ea9ff2fb5e82f0da6fe2a73546fc66)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42726589827 for 76ad159c29ea9ff2fb5e82f0da6fe2a73546fc66)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42726589827 for 76ad159c29ea9ff2fb5e82f0da6fe2a73546fc66)
- ci-result: `not a check` passed (Check results for 76ad159c29ea9ff2fb5e82f0da6fe2a73546fc66 come from the CI run https://github.com/doodla/athena/actions/runs/42726589827 (643s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "Run `cargo run -p xtask -- site`: it builds the study without error and the drain source check passes on the corrected study.")
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo run -p xtask -- site && cargo test -p athena-cap-study every_study_follows -- --nocapture && cargo test -p athena-cli --test drain a_page_with_correct_text_in_every_file_passes_the_source_check_with_one_verifier_call`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-cli --test drain source_fix && cargo test -p athena-core source_f && cargo test -p athena-cli && cargo test -p athena-core`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
