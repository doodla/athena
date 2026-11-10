# Job 20261110-151108-70nx: verify "Resolve every cited past job id to its recorded reading in the reader"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The reading cited past jobs by id but nothing resolved those ids to their recorded deliverables befo
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: new-size-rule-fixtures-invented-not-taken-from-recorded-failed-jobs (20261109-090758-mrgx)
> Cause: The reading cited past jobs by id but nothing resolved those ids to their recorded deliverables before the reading was accepted. The thresholds (3 groups, 30 deliverables) and the test fixture were then built from an assumed shape. Nothing ...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261109-090744-yxah (job 20261109-075120-7zqw); prevention it proposed: When a request or reading adds a refusal rule that cites past jobs, Athena should resolve each cited job id to its recorded reading and deliverables at reading time. The criteria should then require the rule's test to load those recorded sh...
> Introspection 20261110-124233-20rr (job 20261029-111121-fzx0); prevention it proposed: Add a code check that runs any new reading-refusal or matching rule over the recorded past requests and accepted readings, with the old and new builds, before the producer finishes. It reports newly refused readings that cover all bullets a...
> Introspection 20261110-144807-kchg (job 20261110-124307-bn3c); prevention it proposed: Make a replay over the recorded real requests a required step of any job that adds or changes a refusal or matching rule. The step runs the old and new builds over all recorded requests and writes a record with the measured newly-refused co...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader brief resolves job ids (to recorded deliverables, criteria and changed files) only for ids written in the request text. Ids cited only by the retry's introspection (earlier cases, cause and prevention text) are never resolved, so readings invent fixtures and thresholds from an assumed shape. The change resolves those ids too, so the reader sees their recorded shape; it adds no refusal rule. It also addresses retried jobs 20261109-075120-7zqw, 20261029-111121-fzx0 and 20261110-124307-bn3c by dropping their candidates.

Assumptions:
- Goal and audience: the reader stage of Athena, and whoever judges its readings, must be given the recorded deliverables of every past job a request or its retry cause cites, instead of inventing a shape. The fix is information given to the reader, not a refusal rule, so no refusal rate applies.
- Code survey: crates/athena-core/src/reading/topics.rs (referenced_jobs resolves only job_ids_in(request) and followed_up_ids(request), capped by MAX_REFERENCED_JOBS, via carry()); crates/athena-core/src/reading.rs read_once (builds ReaderInputs, passes attempt.introspection through earlier_attempt); crates/athena-context/src/briefs/reader.rs (EarlierAttempt already holds reading_deliverables, reading_criteria and changed_files; RetryCause.earlier_cases carry job ids; retry_cause_section prints only text for them; prior_result_section/referenced_jobs_section render the carried jobs). No record or data file is added under crates/, so check_record_file/AnyRecord are not affected.
- Previous job 20261109-075120-7zqw (size check, rejected): its candidate's check_job_size, invented ZAJ2_SHAPE fixture and wrong doc claims are dropped, not kept; the finding that fixtures were invented is exactly this root cause, now fixed by resolving cited ids to recorded readings. The size-rule finding (bound too coarse) is out of scope as a separate rule.
- Previous job 20261029-111121-fzx0 (acceptance-bullet coverage check, rejected) and 20261110-124307-bn3c (its retry, rejected): the coverage.rs module, the fake.rs and drain.rs edits and the checks.rs wiring are dropped. Their findings (word-stem scoring, code-fence parsing, false positives) concern a refusal rule not requested here, and no longer apply because that rule is not in this change. The fixtures they needed (recorded readings of cited jobs) are what this change supplies to the reader.
- Referenced job ids are carried through the existing carry() path, redacted by the existing Redactor and bounded by MAX_REFERENCED_JOBS; jobs cited only by the introspection are listed after those named in the request so the cap never drops a request-named job. They are shown to the reader; continued_jobs still decides which must be accounted for, so the accounting rule check_prior_result_and_scope is unchanged.
- Existing code: crates/athena-core/src/reading/topics.rs, crates/athena-context/src/briefs/reader.rs and crates/athena-core/src/reading.rs; the change extends referenced_jobs and ReaderInputs.referenced_jobs rather than adding a new resolver.
- Consumers: none found (no config deliverable).

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/reading/topics.rs builds Records with a job A cited only in a retry's introspection earlier_cases (job field) or in its cause/prevention text, not in the request text, and asserts referenced_jobs (or its replacement) returns A with its recorded reading_deliverables. Run `cargo test -p athena-core referenced` to check; the test fails on the base code, which ignores such ids, and passes with the change.
2. [behavior] A test in crates/athena-context/src/briefs.rs builds a reader brief for a retry whose introspection cites a past job only by id, and asserts the brief contains that job's recorded deliverables, criteria and changed files under a heading for cited jobs. Run `cargo test -p athena-context brief` to check; it fails without the change.
3. [behavior] A test shows the cause is generic: with two different cited jobs (different ids, different deliverable lists) both appear in the brief with their own recorded deliverables, ids from the request text come first, and the MAX_REFERENCED_JOBS cap never drops a request-named job. Run `cargo test -p athena-core -p athena-context`.
4. [content] The doc comment on referenced_jobs in crates/athena-core/src/reading/topics.rs names the root cause (ids cited by introspection were never resolved to recorded deliverables, so fixtures and thresholds were invented), and the CHANGELOG.md Unreleased entry states it and lists jobs 20261109-075120-7zqw, 20261029-111121-fzx0 and 20261110-124307-bn3c as the instances. Check with Grep on both files.
5. [content] The change adds no new refusal or matching rule: `git diff` shows no new step! in refusing_check in crates/athena-core/src/reading/checks.rs and no new check function, so no newly refused share applies. check_prior_result_and_scope and continued_jobs behave as before; their existing tests still pass.
6. [content] The 'Referenced jobs' and 'Previous result and change scope' brief sections for cited-only jobs print the recorded deliverables, with no line starting with `Job ` added inside carried_work, and the reader prompt files are unchanged (git diff shows no prompt file edit; the reader prompt set has 2 tokens of headroom).
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.

## Expected deliverables

- crates/athena-core/src/reading/topics.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any new refusal rule: the size check (check_job_size), the acceptance-bullet coverage check (coverage.rs) and a replay harness; the dropped candidates of jobs 20261109-075120-7zqw, 20261029-111121-fzx0 and 20261110-124307-bn3c are discarded, including crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/coverage.rs, crates/athena-agent/src/fake.rs and crates/athena-cli/tests/drain.rs.
- Changing reader prompts, athena.toml, defaults.toml or .github/workflows.
- Redoing the work of the earlier jobs (size bounds, bullet coverage); these stay separate requests.
- Auto-splitting jobs or changing the small-job default.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed where the ids are resolved (the reader inputs), not for one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail on the base and pass with the change.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, no refusal rule.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in crates/athena-core/src/reading/topics.rs `mod tests`, names starting `referenced_jobs_`. Records hold job A (id not in the request, deliverables `modify crates/x/a.rs` and `create crates/x/b.rs`) and the retried job B. The attempt is `EarlierAttempt{job: B, introspection: Some(RetryCause{earlier_cases:[EarlierCase{job: Some(A),..}],..}), ..}`. Variants: (a) A only in `earlier_cases[].job`; (b) A only in `RetryCause.cause` text, `earlier_cases` empty; (c) A only in `prevention` text; (d) B is never returned, an id in no record is ignored, and `attempt: None` gives request-named jobs only. Each calls `referenced_jobs(&records, request, Some(&attempt), &redactor)` and asserts A comes back with `reading_deliverables == ["modify crates/x/a.rs", "create crates/x/b.rs"]`. Run: `cargo test -p athena-core referenced`. Observes: The tests pass: A, cited only by the introspection, comes back with its recorded deliverables. The end-to-end proof that fails by assertion on the base code is the retry.rs test of criterion 2.
  Fails without the behavior: The base `referenced_jobs` reads only `followed_up_ids(request)` and `job_ids_in(request)` (topics.rs:540-545), so it never returns A. These tests do not compile against the base signature, so the verifier's revert run shows a compile error. The assertion-level failure on the base is shown by the retry.rs and briefs.rs tests (criterion 2).
  Test seam: Production entry point `referenced_jobs` (exists, pub, topics.rs:533), called directly. Fixtures: `Records::default()` with `JobRecords` inserted into `records.jobs` by the local helper `job_with_reading` (added by step 2, built on `scoped_reading`, made pub(super) by step 1), and `Redactor::new(&athena_gate::builtin_config())` (as in reading.rs:1864). The introspection is a plain `RetryCause` value, so the id resolution under test is not in the double.
- Criterion 2: Two tests. (i) Real path, new in crates/athena-cli/tests/retry.rs, `a_retry_resolves_the_cited_earlier_case_job_to_its_recorded_deliverables`: reuse `setup`, `two_cases` and `Model` to make two rejected jobs of one class. Call `athena.retry(&second, ..)`, take `model.last_brief()`, and slice the `# Referenced jobs` section up to the next `\n# ` heading. Assert the section exists, contains `Job {first}.`, contains `Deliverables it listed`, and contains the quoted `modify {path}` line for each deliverable of `Records::load(..).jobs[first].latest_reading()`, with that list non-empty. Also assert the `Previous result and change scope` section has one `Job ` line for `second` and one for `first`, and that `first`'s changed files are quoted there. (ii) Rendering, new in crates/athena-context/src/briefs.rs beside `a_referenced_job_brief_prints_files_approach_and_checks` (briefs.rs:3974): `reader_inputs(&reading())` with `earlier_attempt` carrying `introspection: Some(RetryCause{earlier_cases:[EarlierCase{job:Some("20260901-000000-dddd"),..}],..})` and `referenced_jobs` holding that job with criteria, deliverables and changed files; assert criteria and deliverables are quoted under the Referenced jobs entry and the changed files under the prior-result entry. Run: `cargo test -p athena-cli --test retry && cargo test -p athena-context brief`. Observes: The reader brief for a retry whose only link to a past job is an id in the introspection contains that job's recorded criteria, deliverables and changed files under the cited-jobs heading, produced by the real read path.
  Fails without the behavior: On the base code the retry request `try again on {second}` names only the skipped retried job, so `referenced_jobs` is empty, the `# Referenced jobs` heading is absent and the retry.rs assertion fails. Both tests compile on the base, so this is an assertion failure. The briefs.rs test also fails on the base because `referenced_jobs_section` prints no criteria or deliverables (reader.rs:442-458).
  Test seam: Test (i): production `Athena::retry` → `read_once` → `referenced_jobs` → `carry` → `reader_brief`; the fake agent script `Model::script` captures the reader brief (exists, retry.rs:43-69); fixtures `setup`, `two_cases`, `reject` exist (retry.rs:103-158). The expected deliverables are read from the loaded records, not hard-coded. Test (ii): production `reader_brief` (exists, reader.rs:318) with `reader_inputs` and `reading()` fixtures (exist, briefs.rs:3430, 3898); the new test is added by step 6. The rendering code is under test, and the data in the double is input.
- Criterion 3: (i) topics.rs test `referenced_jobs_lists_request_named_jobs_first_and_never_drops_one`: the request names 3 jobs R1..R3 (`MAX_REFERENCED_JOBS`), each with its own deliverables. The introspection cites D1, D2 with different deliverable lists, then more distinct jobs beyond `MAX_CITED_JOBS`, plus R1 again. Assert the result order is R1,R2,R3,D1,D2,.. with R1 once, all three request-named jobs present, D1's deliverables differing from D2's and each equal to its own record, and cited jobs past `MAX_CITED_JOBS` dropped. A second case: 4 request-named jobs where the 4th is also cited by the introspection; assert it is carried after the three request-named ones (finding 5). (ii) briefs.rs test with two cited `EarlierAttempt`s of different ids and deliverable lists: both appear in the brief, each with its own list, in the given order, with request-named ids first. Run: `cargo test -p athena-core -p athena-context`. Observes: All tests in both crates pass, including the cap, order and dedupe test and the two-cited-jobs brief test.
  Fails without the behavior: The base code returns only request-named jobs, so D1 and D2 are absent. A shared cap of 3 fails the assertion that D1 is kept after three request-named jobs. Putting cited ids first fails the order assertion. Deduping against considered ids fails the 4th-job case.
  Test seam: `referenced_jobs` and `reader_brief`, called directly with the same fixtures as criteria 1 and 2 (exist or added by steps 1 and 2); the signature change and `MAX_CITED_JOBS` are added by step 2.
- Criterion 4: topics.rs test `referenced_jobs_doc_and_changelog_name_the_cause`. It finds the marker with `concat!("pub fn ", "referenced_jobs(")` in `include_str!("topics.rs")`, walks back over the contiguous `///` lines above it, and asserts that block contains the phrases `concat!("never ", "resolved")` and `concat!("in", "vented")`. It then slices `include_str!("../../../../CHANGELOG.md")` from `## [Unreleased]` (CHANGELOG.md:8) to the next `## [` heading and asserts the slice names 20261109-075120-7zqw, 20261029-111121-fzx0 and 20261110-124307-bn3c and the same two phrases. The verifier also runs Grep on the CHANGELOG ids and on the `///` block above `pub fn referenced_jobs(` only. Run: `cargo test -p athena-core referenced_jobs_doc`. Observes: The doc block and the Unreleased entry name the cause and the three instances. The test's own literals are split, so they cannot satisfy the search.
  Fails without the behavior: Without the doc comment or the CHANGELOG entry the extracted block or slice lacks the phrases and ids, and the test fails.
  Test seam: Plain `include_str!` of two repository files; no double. The text is written by steps 2 and 8.
- Criterion 5: `git diff --stat` must not list crates/athena-core/src/reading/checks.rs, so there is no new `step!`/`check_` function. The existing prior-result tests stay green (reading.rs 3764-3837 and 4288-4387, which pass cited-only `referenced` jobs and expect them not accounted for). refusal_corpus.rs gets one case, `introspection-cited-job-carried-but-not-relied-on` (Expected::Accept): the retried job has an introspection citing OTHER_JOB, `referenced` holds OTHER_JOB, and the reading names only the retried job. Existing cases I read there: root-cause-cites-two-jobs-without-relying (Accept), follow-up-on-a-job-the-reading-ignores (Refuse check_prior_result_and_scope), retry-complaint-reading-names-the-job (Accept), retry-complaint-reading-omits-the-job (Refuse check_prior_result_and_scope). Together they show cited jobs are not accounted for and followed-up jobs still are. Refused share: 0% of past accepted readings, because no rule is added. I could not run the corpus test and cite only case ids I read. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: All corpus cases keep their outcomes and the new case is accepted; `git diff --stat` shows no checks.rs.
  Fails without the behavior: A change that turned carried `referenced` jobs into jobs to account for would make the new case refuse and fail. A new `step!` would make `every_acceptance_check_has_a_refusing_corpus_case` demand a refusing case.
  Test seam: `refusing_check` driven by the corpus `Case` and `run` (exist, refusal_corpus.rs:72, 2595). Step 7 adds the case; `RetryCause` and `EarlierCase` are already exported from `athena_context::briefs`.
- Criterion 6: A briefs.rs test with a cited-only job whose `reading_deliverables` is set. It asserts the deliverables are quoted under both `# Referenced jobs` and the `PRIOR_RESULT_HEADING` entry. It asserts the `PRIOR_RESULT_HEADING` section has exactly one line starting `Job ` per job (as at briefs.rs:3969-3970), and that the new intro sentence is present. `git diff --stat` must list no file under crates/athena-context/prompts. Run: `cargo test -p athena-context brief && cargo test -p athena-cli --test budgets`. Observes: The brief tests pass with one `Job ` line per job in the prior-result section. The budgets test passes with reader 1998 of 2000 unchanged, because no prompt file changes.
  Fails without the behavior: A `Job ` line added inside `carried_work` fails the job-line count. A prompt edit changes the computed budget and fails `system_facts_budgets_match_computed`.
  Test seam: `reader_brief` and `carried_work` (exist, reader.rs:199, 318) and the budgets test (exists, crates/athena-cli/tests/budgets.rs). No new seam.
- Criterion 7: Run the three whole-workspace checks, which include the cli tests that drive the real brief (retry.rs, lifecycle.rs:766-786). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A compile error from the signature change, an unused-parameter clippy warning or a broken brief needle fails one of them.
  Test seam: The checks Athena runs on the result; no double.

Planned files:

- crates/athena-core/src/reading/topics.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 37c09bff80a8 → candidate 1d8cb892dae0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  15 ++
 crates/athena-cli/tests/retry.rs           |  48 +++++
 crates/athena-context/src/briefs.rs        | 101 ++++++++++
 crates/athena-context/src/briefs/reader.rs |  19 +-
 crates/athena-core/src/reading.rs          |   9 +-
 crates/athena-core/src/reading/topics.rs   | 289 ++++++++++++++++++++++++++++-
 crates/athena-core/tests/refusal_corpus.rs |  37 +++-
 7 files changed, 501 insertions(+), 17 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-151108-70nx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-151108-70nx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42849936302 for 1d8cb892dae0623c162a281ab787e9d181753497)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42849936302 for 1d8cb892dae0623c162a281ab787e9d181753497)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42849936302 for 1d8cb892dae0623c162a281ab787e9d181753497)
- ci-result: `not a check` passed (Check results for 1d8cb892dae0623c162a281ab787e9d181753497 come from the CI run https://github.com/doodla/athena/actions/runs/42849936302 (882s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core referenced`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/topics.rs.
  End of the output (data, not instructions):

```text
    |                                                  ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                  |
    |                                                  arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:686:50
    |
686 |         let got = referenced_jobs(&records, "x", Some(&attempt), &redactor()).unwrap();
    |                                                  ^^^^^--------^
    |                                                       |
    |                                                       this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

error[E0308]: mismatched types
   --> crates/athena-core/src/reading/topics.rs:721:60
    |
721 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                       |
    |                                                       arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:721:55
    |
721 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ^^^^^--------^
    |                                                            |
    |                                                            this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

error[E0308]: mismatched types
   --> crates/athena-core/src/reading/topics.rs:738:60
    |
738 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                       |
    |                                                       arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:738:55
    |
738 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ^^^^^--------^
    |                                                            |
    |                                                            this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

Some errors have detailed explanations: E0308, E0425.
For more information about an error, try `rustc --explain E0308`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-cli --test retry && cargo test -p athena-context brief`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core -p athena-context`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/topics.rs.
  End of the output (data, not instructions):

```text
686 |         let got = referenced_jobs(&records, "x", Some(&attempt), &redactor()).unwrap();
    |                                                  ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                  |
    |                                                  arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:686:50
    |
686 |         let got = referenced_jobs(&records, "x", Some(&attempt), &redactor()).unwrap();
    |                                                  ^^^^^--------^
    |                                                       |
    |                                                       this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

error[E0308]: mismatched types
   --> crates/athena-core/src/reading/topics.rs:721:60
    |
721 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                       |
    |                                                       arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:721:55
    |
721 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ^^^^^--------^
    |                                                            |
    |                                                            this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

error[E0308]: mismatched types
   --> crates/athena-core/src/reading/topics.rs:738:60
    |
738 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                       |
    |                                                       arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:738:55
    |
738 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ^^^^^--------^
    |                                                            |
    |                                                            this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

Some errors have detailed explanations: E0308, E0425.
For more information about an error, try `rustc --explain E0308`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core referenced_jobs_doc`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/topics.rs.
  End of the output (data, not instructions):

```text
686 |         let got = referenced_jobs(&records, "x", Some(&attempt), &redactor()).unwrap();
    |                                                  ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                  |
    |                                                  arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:686:50
    |
686 |         let got = referenced_jobs(&records, "x", Some(&attempt), &redactor()).unwrap();
    |                                                  ^^^^^--------^
    |                                                       |
    |                                                       this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

error[E0308]: mismatched types
   --> crates/athena-core/src/reading/topics.rs:721:60
    |
721 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                       |
    |                                                       arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:721:55
    |
721 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ^^^^^--------^
    |                                                            |
    |                                                            this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

error[E0308]: mismatched types
   --> crates/athena-core/src/reading/topics.rs:738:60
    |
738 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ---- ^^^^^^^^ expected `&str`, found `&EarlierAttempt`
    |                                                       |
    |                                                       arguments to this enum variant are incorrect
    |
    = note: expected reference `&str`
               found reference `&athena_context::briefs::EarlierAttempt`
help: the type constructed contains `&athena_context::briefs::EarlierAttempt` due to the type of the argument passed
   --> crates/athena-core/src/reading/topics.rs:738:55
    |
738 |         let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
    |                                                       ^^^^^--------^
    |                                                            |
    |                                                            this argument influences the type of `Some`
note: tuple variant defined here
   --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/option.rs:605:4

Some errors have detailed explanations: E0308, E0425.
For more information about an error, try `rustc --explain E0308`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core --test refusal_corpus`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-context brief && cargo test -p athena-cli --test budgets`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
