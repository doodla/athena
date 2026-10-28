# Job 20261028-201400-ykmf: verify "Carry earlier job's files, approach and results into the reading"

## Request (verbatim)

> try again on 20261027-054209-z7sj

## Reading (accepted by the user)

Retry of job 20261027-054209-z7sj (dispatched, never verified, no findings): fix the root cause that the reader's carry-over of an earlier job omits its changed files, approach and measured results, by extending `EarlierAttempt`, the brief section and the carry check, with tests covering the class.

Assumptions:
- Goal and audience: retries of a job should carry what the earlier job did, so the reader (and the user judging the reading) sees settled work instead of redoing or dropping it.
- Job 20261027-054209-z7sj: end state dispatched, never verified, 0 findings recorded, so there are no findings to fix; its changes were discarded and its root-cause reading and deliverables are reused, since the user accepted that reading.
- Code survey: read crates/athena-core/src/reading.rs (`carry` at ~1563 builds `EarlierAttempt` from `JobRecords`; `check_prior_result_and_scope` at ~2229 only checks job-id naming plus deliverables and out_of_scope) and crates/athena-context/src/briefs.rs (`EarlierAttempt` at ~1316, `prior_result_section` at ~1395). `JobRecords` in crates/athena-core/src/store.rs has `candidate: Option<Candidate>`, and run/mod.rs sets `changed_paths` when closing. The change adds fields to `EarlierAttempt`, filled in `carry` from the candidate and measured checks, printed in the brief, and enforced in the check. It adds no record or data file, so the records allow-list (`check_record_file`, `AnyRecord`) is untouched. Properties are measured by calling `carry`, `reader_brief` and the check directly.
- The existing `EarlierAttempt` constructors (fake reader, tests) are updated for the new fields; Default keeps most call sites unchanged.

## Acceptance criteria

1. [content] The change description names the root cause: the reader's carry-over of an earlier job (`carry` in crates/athena-core/src/reading.rs, `prior_result_section` in crates/athena-context/src/briefs.rs) leaves out the files its candidate changed, the producer's approach and the measured check results, and `check_prior_result_and_scope` only tests that the job id is named. Job 20261027-054209-z7sj was dispatched and never verified, with no findings; this retry starts from the base.
2. [behavior] A unit test in crates/athena-core/src/reading.rs builds a job record with a candidate (changed paths, producer approach summary, a measured check) and shows `carry` returns those files, the approach and the measured result, redacted and bounded. Run `cargo test --locked -p athena-core reading`; the test fails on the base commit and passes with the change.
3. [behavior] A unit test in crates/athena-context/src/briefs.rs shows `reader_brief` prints the files touched and the approach for both a retried job and a referenced job, and prints a 'none recorded' line when no candidate exists. Run `cargo test --locked -p athena-context briefs`.
4. [behavior] A unit test shows `check_prior_result_and_scope` refuses a reading that names the job id but neither lists nor excludes a prior job's changed files, and accepts one that does. It uses prior jobs of at least two failure shapes (dispatched/abandoned unverified, rejected with findings), not one job id. Run `cargo test --locked -p athena-core check_prior_result`.
5. [behavior] The change refuses more readings (retries whose prior job changed files but whose reading ignores them); the higher refusal rate is intended: expected 5% of newly refused readings, because only retries with a recorded candidate are affected.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked` all pass, including the fake reader in crates/athena-agent/src/fake.rs and the tests in crates/athena-cli/tests/retry.rs and lifecycle.rs.
7. [content] CHANGELOG.md records the fix. No prompt file, athena.toml, defaults.toml or workflow changes: the reader prompt has 3 tokens of headroom, so new wording lives in the generated brief section.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)

## Out of scope

- Editing prompt files (the reader set has 3 tokens left)
- Changing athena.toml, defaults.toml, workflows or tool pins
- Reopening the test-suite speedup of jobs 20261025-202447-xgk9 and 20261025-214509-y179 or the scope-guard work of 20261026-024220-94p9
- Adopting cargo-nextest or sccache; deleting or weakening tests
- Changing T1-T5 thresholds, killing running jobs or lifting pauses
- Changing how retry chains are followed or MAX_REFERENCED_JOBS

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the carry-over at its origin (carry, brief, check) rather than one job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run named cargo test commands.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests cover the class using two failure shapes.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The reading check enforces the carry-over, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check of the CHANGELOG diff, plus an automated check that the CHANGELOG states the declaration. The entry under [Unreleased] / Fixed names the root cause: `carry` in crates/athena-core/src/reading.rs and `prior_result_section` in crates/athena-context/src/briefs.rs leave out the files, approach and measured check results, and `check_prior_result_and_scope` only tests that the job id is named. It says job 20261027-054209-z7sj was dispatched and never verified with no findings, and that this retry starts from the base. It also gives the 5% as a declared expectation, not a measurement, and says the replay supplies no earlier attempts. Run: `git diff -- CHANGELOG.md`. Observes: The added lines contain `carry`, `prior_result_section`, `check_prior_result_and_scope`, the three omitted things, the 20261027-054209-z7sj sentence, and the phrases 'refuses more readings' and 'higher refusal rate is intended'. The reading.rs test `prior_result_refusal_is_declared_intended` (run in criterion 5's command) reads the CHANGELOG with `include_str!` and asserts the same phrases.
- Criterion 2: Unit test `carry_includes_changed_files_approach_and_measured_checks_redacted_and_bounded` in reading.rs. Entry point: `carry(&JobRecords, &Redactor)`, private and called from the in-file tests module as the existing carry tests do. Fixture: the existing `job_with_checks` (~3358) and `candidate_of` (~4422), with `job.candidate` set to a real `Candidate` of more than 30 `ChangeEntry` values (statuses Added/Modified/Deleted). One path holds a token-shaped secret and the `producer_report.summary` is long with the same token. Two real `CheckRun`s: one passed with tests_ran, one failed with an exit code and output text. Redactor is `Redactor::new(&athena_gate::builtin_config())` as in existing carry tests. Nothing is a double; the seam exists today. A second job has no candidate. A third has a path altered by redaction. Run: `cargo test --locked -p athena-core reading`. Observes: changed_files has exactly 30 'status path' entries. The redacted-path entry is absent and counted in `changed_files_omitted` with the cap overflow. The token appears in neither files nor approach. The approach is bounded and keeps its start. `checks` has both lines with pass/fail, exit code and test count, and none of the output text. The no-candidate job gives empty files, `approach == None`, and keeps `checks` from verification. On the base commit the fields do not exist, so the test fails to compile (counts as failing); the existing `carry_*` tests still pass.
  Fails without the behavior: If `carry` did not copy candidate.changes, producer_report.summary or the checks, the files/approach/checks assertions fail; without redaction the token assertion fails; without the cap the length assertions fail; without dropping the redacted path the omitted-count assertion fails.
- Criterion 3: Unit tests in briefs.rs using the existing `reader_inputs(&reading())` helper and the production `reader_brief(&ReaderInputs)`, cut at PRIOR_RESULT_HEADING as the existing tests do. Inputs are plain data structs, so there is no double. (a) `a_retry_brief_prints_files_approach_and_checks_of_the_earlier_job` sets `inputs.earlier_attempt` with changed_files ['modified src/a.rs','deleted README.md'], omitted 2, an approach and a check line. (b) `a_referenced_job_brief_prints_files_approach_and_checks` sets `inputs.referenced_jobs` the same way. (c) `a_prior_without_a_candidate_says_none_recorded` uses a Default `EarlierAttempt` in both slots. Run: `cargo test --locked -p athena-context briefs`. Observes: For both the retried and the referenced job the section holds '> modified src/a.rs', '> deleted README.md', the 'and 2 more not listed' line, the approach text and the check line. For the empty prior it holds 'Files its candidate changed: none recorded for it', 'Producer approach: none recorded for it' and 'Measured checks: none recorded for it'. No new line starts with 'Job '. `the_reader_brief_names_each_reading_rule_in_one_short_line` still passes (rule line at most 220 chars, rules section under 2400 bytes).
  Fails without the behavior: If `prior_result_section` printed nothing for the new fields, the contains-assertions fail for both prior kinds; if it printed nothing for empty priors, the 'none recorded' assertions fail.
- Criterion 4: Unit tests in reading.rs named `check_prior_result_*`. Entry point: `check_prior_result_and_scope(&Reading, &[&EarlierAttempt])` with the existing `scoped_reading` (~4811). The priors are made by the real `carry` over `JobRecords`: (a) a dispatched/abandoned unverified job (candidate changing `src/a.rs` and `crates/x/src/a.rs`, no verification, no outcome, no findings); (b) a rejected job (verdict fail, findings, failed check, same shape of candidate) from `job_with_checks`. The reading names each job id, lists README.md as the only deliverable and has out_of_scope ['the changelog']. Run: `cargo test --locked -p athena-core check_prior_result`. Observes: For both shapes: refused with a message containing "the earlier job's files" and the file names, and containing neither 'the previous result' nor 'the change scope'. Accepted when the file is a deliverable, when a directory deliverable covers it, when an out_of_scope item names the exact path, and when out_of_scope names an ancestor directory such as `crates/x/` as its own token. Accepted for a prior with no recorded files. With two priors, only the one with unaccounted files is named. Regression cases for finding 1: a reading whose assumptions and summary mention 'crates/athena-core/src/lib.rs' and 'crates/' is still refused for the unlisted 'crates/athena-core/src/a.rs'. A reading naming 'crates/x/src/a.rs' in out_of_scope does not account for 'src/a.rs'. Prose 'crates/' in the summary does not account for any file. `src/a` does not match 'src/a.rs'. A path followed by '.' at a sentence end matches. A file whose redacted name was dropped by `carry` never causes a refusal. The existing id and scope tests still pass.
  Fails without the behavior: If the check ignored the files, the refusal assertions fail for both shapes; if it ignored deliverables or out_of_scope, the acceptance assertions fail; if it matched substrings or free text, the finding-1 regression cases fail.
- Criterion 5: Table test `check_prior_result_refuses_only_priors_with_recorded_files_the_reading_ignores` in reading.rs. One fixed reading, which names the job, lists one deliverable and one out_of_scope item and mentions no earlier file, is run through the real check against priors built by `carry`: no candidate; a candidate with no changes; files the reading lists; files it excludes; files it ignores in shape (a) and in shape (b). Also `prior_result_refusal_is_declared_intended`: `replay_declaration` on a criterion with 'expected 5% of newly refused readings' returns share Some(5); the doc comment of `check_prior_result_and_scope` contains 'This check refuses more readings; the higher refusal rate is intended.'; the CHANGELOG (via `include_str!`, whitespace made single spaces) contains 'refuses more readings' and 'higher refusal rate is intended'. The 5% is a declared expectation. `check_reading_text` (~2831) gives the replay no priors, so the replay cannot observe this check; the test shows which class is refused. Run: `cargo test --locked -p athena-core check_prior_result`. Observes: Exactly the two 'ignores' rows are refused and every other row is accepted, so the new refusals are bounded to retries with a recorded candidate whose files the reading ignores. The declaration parses as 5, below the 10% limit, with no reason needed. The doc and CHANGELOG sentences are present.
  Fails without the behavior: If the check also refused priors with no recorded files, or readings that list or exclude them, the table assertions fail; if it never refused, the 'ignores' rows fail; if the doc or CHANGELOG sentence were missing, the declared-intended test fails.
- Criterion 6: Run the three checks Athena runs. The nested runs go through the fake reader in crates/athena-agent/src/fake.rs (production `deterministic_turn`, Role::Reader arm), which is changed to parse the quoted lines under the new 'Files its candidate changed' header of the previous-result section and name those paths in its out_of_scope item, so nested retries pass the new check. New coverage: retry.rs gets a test that a retry reading naming the job and giving out_of_scope but not the earlier job's file is refused with "the earlier job's files" and accepted once the reader lists the file, reusing the mode-switching reader of the existing result-and-scope test (the seam exists today). lifecycle.rs's retry-with-failed-check test also asserts the brief carries the earlier job's changed file and a measured check line, while PASSING-CANARY stays absent. run/mod.rs gets `a_retry_reader_brief_carries_the_earlier_jobs_files_approach_and_checks`: an Athena built by the existing `athena_with(.., FAILS_ON_EDIT, .., script)`, whose script wraps `base_script()` and records every Reader `user_prompt` into an `Arc<Mutex<Vec<String>>>`. Run a job to rejection with `ask`+`go`, then `athena.retry(job, &Quiet)`; the retry reader's summary names the earlier job as `counting` does. Assert the last reader prompt holds 'modified README.md', the producer summary 'Described the demo.' and a 'probe: failed' line. The wrapper is test-only plumbing; `earlier_attempt`, `carry` and `reader_brief` are production code. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0, including the `retry` and `lifecycle` integration tests, the nested runs, `system_facts_budgets_match_computed`, and the new run/mod.rs test.
  Fails without the behavior: If the fake reader did not name the earlier job's files, a nested retry whose producer changed files other than FAKE-CHANGE.md would be refused and the retry tests would fail; if `carry` or the brief dropped the files, the lifecycle and run/mod.rs assertions fail.
- Criterion 7: Content check that CHANGELOG.md has the entry and that no governance or prompt file changed. Run: `git status`. Observes: Modified files are only reading.rs, briefs.rs, fake.rs, retry.rs, lifecycle.rs, run/mod.rs and CHANGELOG.md (plus a test reader elsewhere only if the full run shows one needs it, which the producer reports). No prompt file, athena.toml, defaults.toml or workflow appears. `git diff -- CHANGELOG.md` shows the Fixed entry. `system_facts_budgets_match_computed` passes in the criterion 6 run, showing the prompt sets are unchanged.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): The change refuses more readings (retries whose prior job changed files but whose reading ignores them); the higher refusal rate is intended: expected 5% of newly refused readings, because only retries with a recorded candidate are affected.

## The change

Base 3a32873b6701 → candidate 1c201bf91e15. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  24 ++
 crates/athena-agent/src/fake.rs      |  79 ++++-
 crates/athena-cli/tests/lifecycle.rs |  16 +
 crates/athena-cli/tests/retry.rs     |  84 +++++
 crates/athena-context/src/briefs.rs  | 182 +++++++++-
 crates/athena-core/src/reading.rs    | 623 ++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/run/mod.rs    |  54 +++
 7 files changed, 1048 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1616 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked` all pass, including the fake reader in crates/athena-agent/src/fake.rs and the tests in crates/athena-cli/tests/retry.rs and lifecycle.rs.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
