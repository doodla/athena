# Job 20261027-054253-ary8: verify "Check the diff against a job's allowed paths before verification"

## Request (verbatim)

> Priority: normal
> 
> When an accepted reading limits which files may change (a criterion such as 'no file outside X changed' or 'test-only, production unchanged'), store that limit with the job as a list of allowed paths. Before the producer hands off, and again before verification, a code check compares the diff's changed files with that list. If any file is outside the list, the check fails, names the files, and sends the job back to the producer before the verifier runs. Acceptance: a test shows a diff touching a path outside the allowed list is sent back with the path named; a test shows a diff inside the list passes; a job whose reading has no such limit is unaffected; cargo fmt --check, cargo clippy --all-targets and cargo test pass.
> 
> Introspections that record the repeated cause: 20261018-045410-kwcn, 20261025-202407-sgmw
> 
> Acceptance:
> - The change is made as described: Producers edit files outside the job's allowed scope, and only the final verifier catches it
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261018-045426-cdqv.json

## Reading (accepted by the user)

When an accepted reading limits which files may change (e.g. "no file outside X changed", "test-only"), the limit is stored with the job as a list of allowed paths. A code check compares the diff's changed files to that list before producer hand-off and again before verification, and sends the job back to the producer naming any outside file. Audience: Athena operators, who today only see this caught by the final verifier.

Assumptions:
- Code survey: crates/athena-core/src/run/mod.rs holds the run loop (produce_and_verify, planned_paths, unchanged_planned_files). unchanged_planned_files is the existing path-only check that sends a candidate back to the producer before the verifier, and the new check is added beside it and called at the same two points. Its tests live in crates/athena-core/src/run_pin_tests.rs, which the new tests follow. The send-back text is built in crates/athena-core/src/run/send_back.rs. The job's accepted reading is read in crates/athena-core/src/reading.rs and crates/athena-context/src/briefs.rs (AcceptedReading, whose criteria and deliverables are the input).
- Code survey: crates/athena-data/src/records.rs defines the Job struct (athena.job/1, #[serde(deny_unknown_fields)]). The allowed-paths list is added there as an optional field with #[serde(default, skip_serializing_if = "Option::is_none")], as pin and dispatch_branch are, so older records still parse. Records are accepted by check_record_file (path and bytes, returns Option<AnyRecord>) and the AnyRecord enum; the existing Job round-trip test in records.rs is extended to cover the new field, and a record without it must still pass check_record_file.
- Code survey: the limit is derived in code when the reading is accepted. A criterion that says 'no file outside', 'only ... may change', 'test-only' or 'production unchanged' makes the allowed list the reading's deliverable paths. A reading with no such criterion stores no list, and the check does nothing for it. The reader prompt has 3 tokens of headroom, so prompts are not edited. The property is measured directly: the diff's changed-file list (the snapshot change list, including deleted files) is compared with the stored list, with paths normalised as normal_path does.
- The 'Files:' line in the request names an unrelated failure-class record; it is context only and is not changed. records/ never changes.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run_pin_tests.rs (run: cargo test -p athena-core) passes a diff touching a path outside the allowed list to the new check. It returns that path by name and the job is sent back to the producer before the verifier runs.
2. [behavior] A new test (cargo test -p athena-core) passes a diff whose changed files are all in the allowed list, including a ./-prefixed path and a deleted file. The check passes and the verifier runs.
3. [behavior] A new test (cargo test -p athena-core) shows a job whose reading has no path limit stores no allowed list and is not checked, whatever files change. It also shows that a limiting criterion makes the stored list equal the deliverable paths.
4. [behavior] The check runs at both points, before producer hand-off and again before verification. A test (cargo test -p athena-core) shows an out-of-list change made after the first send-back is sent back again before the verifier.
5. [behavior] crates/athena-data/src/records.rs: a round-trip test passes a Job with the new allowed-paths field through check_record_file and AnyRecord, and a Job record without the field still passes check_record_file (cargo test -p athena-data).
6. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass at the repository root.
7. [content] No unrelated behavior changes: existing tests pass unmodified apart from the new field in Job constructors, and no governance file, prompt file or file under records/ is changed.
8. [content] The send-back text for an out-of-scope diff names each offending path and says to revert it. CHANGELOG.md records the new check.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing reader or producer prompts
- Editing any file under records/, including the failure-class record named in the request
- Changing the verifier's own checks
- Adding a new reader output field

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover refusal, pass and unaffected cases; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests and check the named path is reported.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that out-of-scope diffs are caught before the verifier.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The limit is enforced by a code check, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check beside the existing unchanged_planned_files check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog entry are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Pure test plus end-to-end tests. The pure test calls outside_allowed_paths with a stored list and a changed list holding one outside path and asserts that exact path comes back; a second pure test builds the text with send_back_text_scoped from it. The end-to-end tests run whole jobs whose reader output carries a limiting criterion and whose producer also edits NOTES.md (an added file) and, in a second test, deletes a file outside the list with a (path, None) fake edit (finding 8). Run: `cargo test -p athena-core outside_allowed`. Observes: The helper returns the outside path by name. In the end-to-end test where attempt 1 is clean, the second producer prompt carries '## Sent back' and names NOTES.md, and the verifier runs once after the send-back. In the deletion test the deleted outside file is named in the second prompt. In the test where the diff stays outside, the verifier count is 0.
  Fails without the behavior: With no check, outside_allowed_paths does not exist and the job reaches the verifier with one producer prompt. A check returning nothing fails the named-path assertion, and a check that ignored deletions fails the deletion test.
  Test seam: Entry points: crate::run::outside_allowed_paths (added by step 3, beside unchanged_planned_files) and Athena::go through producer_loop. Fixture: the existing counting/editing fake script (run/tests.rs:2152, 2234), governed_job_with (2263), athena_with (187), Counts and Lines. A wrapper script added by step 9 appends a limiting criterion to the fake reader's acceptance_criteria only and picks per-attempt edits, including (path, None) deletions, from the prompt vector, as reconciling_on_send_back does. It supplies input only; the logic under test is production code. The seam for the pure test is added by step 3; the wrapper by step 9.
- Criterion 2: Pure test plus end-to-end tests. The pure test passes outside_allowed_paths changed paths that are a './'-prefixed allowed path, a deleted allowed path, a plain allowed path and a path under an allowed 'dir/' entry, and asserts an empty result. End-to-end: a scoped job whose producer edits only README.md, and a scoped job whose producer deletes an allowed file with a (path, None) edit (finding 8). Run: `cargo test -p athena-core inside_allowed`. Observes: The helper returns an empty Vec. Both jobs reach the verifier with one producer prompt, verifier count 1 and no scope Failed outcome; the deletion job's change list carries the deleted path, so a real deletion goes through the check.
  Fails without the behavior: A raw-string comparison flags './README.md'; a check that compared only added files would flag the deleted path; no prefix rule flags the 'dir/' case; an always-rejecting check leaves the verifier count at 0.
  Test seam: Entry points: outside_allowed_paths (step 3) and Athena::go through producer_loop. The end-to-end jobs use the scoped wrapper (step 9); change_list includes deletions (finding 8 cites athena-git snapshot.rs:200 via diff_tree), so the deletion is a real working-tree deletion made by the fake producer's (path, None) edit. The wrapper supplies input only.
- Criterion 3: Pure tests build readings with the accepted() builder (run_pin_tests.rs:1046), set deliverables on its public field and check allowed_paths(). Positive: a limiting criterion with deliverables ./a.rs and b.rs gives Some(["a.rs","b.rs"]) equal to the deliverable paths; 'No file outside X changed', 'test-only, production unchanged', 'test-only change in X' and 'only the docs may change' each give a list. Negative, None: each of this job's 8 criteria; the four repository sentences (records-history.md:134, athena-v0.md:984, reading.rs:6598, context-engineer.md:153); 'reached through a test-only seam', 'the helper is test-only' and 'test-only edits in the helper' (finding 4); a reading with no limit; a limiting criterion with no deliverables. End-to-end: an unscoped job whose producer edits README.md and NOTES.md, which also loads the committed records and asserts the job record's allowed_paths is None and that its job.json text has no 'allowed_paths' key; the scoped job's record holds the deliverable paths (finding 7). Run: `cargo test -p athena-core allowed_paths`. Observes: The stored list equals the deliverable paths for limiting readings and is None for the rest. The unscoped job has one producer prompt, one verifier run, no scope reason, no 'scope' UI line and no allowed_paths key in its record.
  Fails without the behavior: A matcher that ignored criteria returns None for limiting readings and fails the equality. One that matched the repository sentences or the bare 'test-only' adjective fails the negative cases. One that matched everything flags NOTES.md in the unscoped job. A bug that stored a list but skipped the check fails the record assertion.
  Test seam: Entry points: AcceptedReading::allowed_paths and limiting_criterion (step 1) and Athena::go. The pure tests use the existing accepted() builder. The end-to-end test uses the existing governed_job helper (run/tests.rs:2252), which takes extra producer edits, and reads the record through Records::load(&athena.git, &athena.head().unwrap()) as the test at run/tests.rs ~3131 does; the producer confirms the field path to the Job (records.jobs[id].job). No double holds logic under test.
- Criterion 4: End-to-end tests with the scoped wrapper. Test A: the producer edits NOTES.md on attempt 0, a different file EXTRA.md on attempt 1 (after the first send-back), and reverts both on attempt 2. Test B: every attempt edits NOTES.md. Test C (resume, finding 6): a job is held at the Checks stage by low disk with an out-of-list candidate and no stored list (the reader adds the limiting criterion only from its second call on); the retry's reading then carries the limit, the retry resumes the kept candidate, and the head of check_and_verify closes it. Test D: a nested run is not checked, if the existing nested test harness can drive one cheaply; otherwise this is a code-reading claim only and the producer says so. Run: `cargo test -p athena-core scope_send_back`. Observes: Test A: three producer prompts, prompts[2] contains '## Sent back' and names EXTRA.md, one verifier run. Test B: three producer runs, verifier count 0, Failed outcome whose reason starts with SCOPE_REASON_PREFIX and names NOTES.md (point 2); the UI lines include 'files still outside the allowed paths after the send-back'. Test C: no producer or planner run on the retry, verifier count 0, Failed with SCOPE_REASON_PREFIX, no 'checks' lines. The producer should be ready to report failure if the reader gate refuses the extra criterion, rather than change the gate.
  Fails without the behavior: Without the extra scope allowance, attempt 1 hits the limit at producer.rs:336, so Test A has two producer prompts and the job closes at point 2; the three-prompt assertion fails. With no check in check_and_verify, Test B reaches the verifier (count 1) with no scope reason, and Test C proceeds to the checks.
  Test seam: Entry points: Athena::go -> producer_loop (point 1) and check_and_verify (point 2), reached from produce_and_verify (mod.rs ~1536) and resume_candidate (mod.rs:523). Fakes: the existing editing/counting scripts, plus a wrapper that picks the edit by prompt index and the reader output by call count. Test C reuses the low_while_checking disk hook and the low-disk-at-checks retry flow (run/tests.rs:3076-3149). Counts, athena_with and Lines exist; the wrapper is added by step 9.
- Criterion 5: Two tests in records.rs. One builds a Job with allowed_paths: Some(vec!["crates/a.rs","README.md"]) through the job_on helper, serialises with AnyRecord::to_json, asserts the text contains "allowed_paths", re-parses it and passes the bytes and canonical_path() to check_record_file. The other uses a Job JSON with no allowed_paths key and asserts check_record_file returns the record with allowed_paths == None and that re-serialising omits the key. Run: `cargo test -p athena-data allowed_paths`. Observes: The round-tripped record equals the original; the old-shape record passes the records check and re-serialises without the key.
  Fails without the behavior: Without the field, the new record fails deny_unknown_fields on parse. Without #[serde(default)] the old-shape record fails; without skip_serializing_if the old-shape text gains the key and the absence assertion fails.
  Test seam: Entry points: check_record_file(path, bytes) and AnyRecord::parse/to_json. Fixture: the existing job_on helper (records.rs ~5118) next to a_job_keeps_its_dispatch_branch_through_a_round_trip (~5137). The seam exists today; step 2 adds the field.
- Criterion 6: Run the three checks Athena runs at the repository root, plus the refusal corpus to show no reading-gate behavior changed. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test -p athena-core --test refusal_corpus`. Observes: All exit 0. The corpus run gives each case its expected outcome (case ids I read, such as dependents-complete-ci-reuse and user-decision, were not run by me, so I report no result).
  Fails without the behavior: A compile error in a missed Job literal (store.rs, reading.rs, records.rs, run/mod.rs), a clippy warning, or an existing send-back or retry test broken by a changed output would fail.
  Test seam: Cargo's own harness; no doubles; the commands exist today.
- Criterion 7: Inspect the working-tree diff: only the listed Rust files, store.rs, run/producer.rs, docs/records.md and CHANGELOG.md changed; existing tests are unchanged apart from allowed_paths: None in Job literals; nothing in athena.toml, defaults.toml, workflows, prompt files or records/. Run: `git status --short && git diff --stat`. Observes: The path list matches the plan's files, with no governance, prompt or records/ file. cargo test --locked (criterion 6) passes, including the existing send-back and retry-finding tests.
  Fails without the behavior: An edited prompt, config or records file shows in git status; a changed existing test expectation shows in git diff.
  Test seam: git status and diff on the working tree; they exist today.
- Criterion 8: A pure test calls send_back_text_scoped with two outside paths and a criterion and asserts both appear as '- path' lines under the outside-paths heading, that the text quotes the criterion, says the limit is the reading's deliverables and contains the revert instruction, and that with a None scope the output equals send_back_text_full's. The CHANGELOG entry is read from the diff. Run: `cargo test -p athena-core scoped_send_back && git diff -- CHANGELOG.md`. Observes: Each offending path is named and the text says to revert it. CHANGELOG.md shows one new bullet under '## [Unreleased]' / '### Added' that names the stored allowed paths, the two checks, and that the limit is the reading's deliverables (so a directory entry ending in '/' covers a tree).
  Fails without the behavior: A text that did not list the paths or did not say to revert fails the assertions; no changelog bullet means an empty diff for CHANGELOG.md.
  Test seam: Entry point: send_back_text_scoped (added by step 4), called directly. The changelog check is a diff read. No doubles.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2e119a839248 → candidate 35ced02f6ef8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  10 +
 crates/athena-context/src/briefs.rs     | 206 +++++++++++++++++
 crates/athena-core/src/reading.rs       |  47 +++-
 crates/athena-core/src/run/mod.rs       |  86 +++++++-
 crates/athena-core/src/run/producer.rs  |  86 ++++++--
 crates/athena-core/src/run/send_back.rs |  39 ++++
 crates/athena-core/src/run/tests.rs     | 376 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/run_pin_tests.rs | 146 +++++++++++++
 crates/athena-core/src/store.rs         |   1 +
 crates/athena-data/src/records.rs       |  60 +++++
 docs/records.md                         |   2 +-
 11 files changed, 1037 insertions(+), 22 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054253-ary8/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054253-ary8/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42201719329 for 35ced02f6ef8dd0df2a6f3a66a84279624b7d82b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42201719329 for 35ced02f6ef8dd0df2a6f3a66a84279624b7d82b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42201719329 for 35ced02f6ef8dd0df2a6f3a66a84279624b7d82b)
- ci-result: `not a check` passed (Check results for 35ced02f6ef8dd0df2a6f3a66a84279624b7d82b come from the CI run https://github.com/doodla/athena/actions/runs/42201719329 (570s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets and cargo test pass at the repository root.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core outside_allowed`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, crates/athena-data/src/records.rs, docs/records.md.
  End of the output (data, not instructions):

```text
     |                                        ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `kind`, `title`, `class_of_work`, `capability` ... and 19 others

error[E0609]: no field `allowed_paths` on type `athena_data::records::Job`
    --> crates/athena-core/src/run/tests.rs:2814:47
     |
2814 |         assert_eq!(stored_job(&athena, &done).allowed_paths, None);
     |                                               ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `kind`, `title`, `class_of_work`, `capability` ... and 19 others

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1362:21
     |
1362 |             reading.allowed_paths(),
     |                     ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `limiting_criterion` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1366:28
     |
1366 |         assert_eq!(reading.limiting_criterion(), Some(criterion));
     |                            ^^^^^^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1378:28
     |
1378 |         assert_eq!(reading.allowed_paths(), None, "{criterion}");
     |                            ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `limiting_criterion` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1379:28
     |
1379 |         assert_eq!(reading.limiting_criterion(), None, "{criterion}");
     |                            ^^^^^^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1384:57
     |
1384 |         reading_with(&["cargo test passes"], &["a.rs"]).allowed_paths(),
     |                                                         ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1388:59
     |
1388 |         reading_with(&["No file outside X changed"], &[]).allowed_paths(),
     |                                                           ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0560]: struct `athena_data::records::Job` has no field named `allowed_paths`
   --> crates/athena-core/src/store.rs:680:13
    |
680 |             allowed_paths: None,
    |             ^^^^^^^^^^^^^ `athena_data::records::Job` does not have this field
    |
    = note: all struct fields are already assigned

Some errors have detailed explanations: E0425, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 21 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core inside_allowed`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, crates/athena-data/src/records.rs, docs/records.md.
  End of the output (data, not instructions):

```text
2651 |             stored_job(&athena, &done).allowed_paths,
     |                                        ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `kind`, `title`, `class_of_work`, `capability` ... and 19 others

error[E0609]: no field `allowed_paths` on type `athena_data::records::Job`
    --> crates/athena-core/src/run/tests.rs:2814:47
     |
2814 |         assert_eq!(stored_job(&athena, &done).allowed_paths, None);
     |                                               ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `kind`, `title`, `class_of_work`, `capability` ... and 19 others

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1362:21
     |
1362 |             reading.allowed_paths(),
     |                     ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `limiting_criterion` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1366:28
     |
1366 |         assert_eq!(reading.limiting_criterion(), Some(criterion));
     |                            ^^^^^^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1378:28
     |
1378 |         assert_eq!(reading.allowed_paths(), None, "{criterion}");
     |                            ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `limiting_criterion` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1379:28
     |
1379 |         assert_eq!(reading.limiting_criterion(), None, "{criterion}");
     |                            ^^^^^^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1384:57
     |
1384 |         reading_with(&["cargo test passes"], &["a.rs"]).allowed_paths(),
     |                                                         ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1388:59
     |
1388 |         reading_with(&["No file outside X changed"], &[]).allowed_paths(),
     |                                                           ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0560]: struct `athena_data::records::Job` has no field named `allowed_paths`
   --> crates/athena-core/src/store.rs:680:13
    |
680 |             allowed_paths: None,
    |             ^^^^^^^^^^^^^ `athena_data::records::Job` does not have this field
    |
    = note: all struct fields are already assigned

Some errors have detailed explanations: E0425, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 21 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core allowed_paths`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, crates/athena-data/src/records.rs, docs/records.md.
  End of the output (data, not instructions):

```text
2651 |             stored_job(&athena, &done).allowed_paths,
     |                                        ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `kind`, `title`, `class_of_work`, `capability` ... and 19 others

error[E0609]: no field `allowed_paths` on type `athena_data::records::Job`
    --> crates/athena-core/src/run/tests.rs:2814:47
     |
2814 |         assert_eq!(stored_job(&athena, &done).allowed_paths, None);
     |                                               ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `kind`, `title`, `class_of_work`, `capability` ... and 19 others

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1362:21
     |
1362 |             reading.allowed_paths(),
     |                     ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `limiting_criterion` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1366:28
     |
1366 |         assert_eq!(reading.limiting_criterion(), Some(criterion));
     |                            ^^^^^^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1378:28
     |
1378 |         assert_eq!(reading.allowed_paths(), None, "{criterion}");
     |                            ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `limiting_criterion` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1379:28
     |
1379 |         assert_eq!(reading.limiting_criterion(), None, "{criterion}");
     |                            ^^^^^^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1384:57
     |
1384 |         reading_with(&["cargo test passes"], &["a.rs"]).allowed_paths(),
     |                                                         ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0599]: no method named `allowed_paths` found for struct `athena_context::briefs::AcceptedReading` in the current scope
    --> crates/athena-core/src/run_pin_tests.rs:1388:59
     |
1388 |         reading_with(&["No file outside X changed"], &[]).allowed_paths(),
     |                                                           ^^^^^^^^^^^^^ method not found in `athena_context::briefs::AcceptedReading`

error[E0560]: struct `athena_data::records::Job` has no field named `allowed_paths`
   --> crates/athena-core/src/store.rs:680:13
    |
680 |             allowed_paths: None,
    |             ^^^^^^^^^^^^^ `athena_data::records::Job` does not have this field
    |
    = note: all struct fields are already assigned

Some errors have detailed explanations: E0425, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 21 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core scope_send_back`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-data allowed_paths`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test -p athena-core --test refusal_corpus`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git status --short && git diff --stat`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test -p athena-core scoped_send_back && git diff -- CHANGELOG.md`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
