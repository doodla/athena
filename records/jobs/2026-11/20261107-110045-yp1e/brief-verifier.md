# Job 20261107-110045-yp1e: verify "Enforce realistic-branch fixtures for landing/history readings by check"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The planning stage derives a commit-shape rule from the intended happy path instead of from the full
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-tested-only-against-job-commits-not-realistic-branch-history (20261024-140616-afw6)
> Cause: The planning stage derives a commit-shape rule from the intended happy path instead of from the full set of shapes the system produces. Here it missed the retry-after-block landing. The plan's demonstrations also use a fixture that does not...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261024-154443-51wt (job 20261024-153908-1rve); prevention it proposed: Change the plan template and critic checklist for any landing or history-check rule. The plan must include an enumerated table of every commit shape the real branch contains. The table must cover job commits, records-only commits, owner-pat...
> Introspection 20261024-161503-q0pp (job 20261024-154552-e65v); prevention it proposed: Before handover, run a check that runs every test in each crate that calls the changed history or classification functions. For this change that means `athena-core` reconcile and `athena-cli --test reconcile`, not only `land::`. The check s...
> Introspection 20261025-012736-2q6w (job 20261025-012657-hac3); prevention it proposed: Add a default to the reading stage. Any criterion on a landing, history or attribution check must name a test fixture that uses a realistic branch. That branch has at least one intervention commit, one records-only commit and the job landin...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (jobs 20261024-153908-1rve, 20261024-154552-e65v, 20261025-012657-hac3; introspections 51wt, q0pp, 2q6w): landing, history and attribution rules are derived from the happy-path commit shape, and the guard against it exists only as planner/critic prose. Fix at the reading stage with a mechanical check in reading.rs, added to refusing_check, that refuses such readings unless a criterion names a realistic-branch fixture with an intervention commit, a records-only commit and the job landing. Earlier jobs' source changes are not retried: that work is dropped.

Assumptions:
- Code Survey placeholder replaced below.
- Code survey: read crates/athena-core/src/reading.rs: refusing_check (about line 3227) runs check_prior_result_and_scope, check_criteria_evidence, check_code_survey (line 3912) and the later checks in order and returns the first failure by name; check_code_survey is the model (applies only to code/tests readings with a behavior criterion, reads assumptions and deliverables, never the repo). Also read crates/athena-context/prompts/planner.md line 16 and critic.md line 14, which already hold the commit-shape table and realistic-fixture guidance as prose only, and crates/athena-context/src/lib.rs test landing_rules_need_a_commit_shape_table_and_a_realistic_fixture. No record or data file is added, so check_record_file and AnyRecord are not involved; the property is measured directly by calling the new check on Reading values.
- Goal and audience: the Athena operator and the planner/verifier stages; the goal is that landing, history and attribution rules are no longer tested only against job commits. The check triggers when deliverables include a landing/history/attribution source (land.rs, history.rs, attribution.rs, or gh.rs check functions) or a criterion text mentions landing, history check or attribution check, and the reading is code/tests with a behavior criterion.
- Environment: none; the check is pure over the Reading, with no model call, no network, no repository read and no write.
- Prior jobs: 20261024-153908-1rve and 20261024-154552-e65v changed (or tried to change) land.rs, gh.rs, invariant.rs, records-history.md and CHANGELOG.md; 20261025-012657-hac3 changed commits.rs, manage.rs, athena-git, status.rs, auditor.rs, athena-gate files, decisions.md. Their specific pr_commits and trailer-cutover candidates are dropped, not retried: their findings (reconcile fixtures not updated, operator/ fixture, missing post-C0 tests) are the instances, and this job fixes the class they share rather than re-landing them.
- The reader prompt has 3 tokens of headroom, so no reader prompt growth; the planner (175 left) and critic prompts already carry the guidance and are left unchanged. The existing mechanical gap is the only thing added.

## Acceptance criteria

1. [content] crates/athena-core/src/reading.rs gains a pub fn check_history_fixture(reading: &Reading) -> Result<()> whose doc comment names the root cause (rules derived from the happy-path commit shape, jobs 20261024-153908-1rve, 20261024-154552-e65v, 20261025-012657-hac3) and states 'expected N% of newly refused readings' with a whole-percent N.
2. [content] refusing_check in reading.rs calls the new check with step!(check_history_fixture(reading)), so the first-failure name and order stay derived from stringify!.
3. [behavior] Run `cargo test -p athena-core reading`: a new unit test shows a code reading with a behavior criterion for a landing/history/attribution rule and no criterion naming a realistic-branch fixture with an intervention commit, a records-only commit and the job landing is refused with an error that says what to add; the same test shows the reading is accepted once such a criterion exists. The refusal test fails if the check is removed from refusing_check.
4. [behavior] Run `cargo test -p athena-core reading`: a test shows readings unrelated to landing/history/attribution (docs, prompts, a code reading touching none of those) are not refused by the new check, and the existing replay/refusal-rate tests still pass.
5. [content] Any reading fixtures in crates/ tests that legitimately touch landing or history rules are updated with the realistic-fixture criterion so existing tests stay valid, and the CHANGELOG.md entry names the root cause.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass (the tested default).
7. [content] The change description names the root cause and explains why it was fixed at the reading-stage check, where readings are accepted, not at one pr_commits or trailer instance.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Re-landing the pr_commits content classification (land.rs, gh.rs, invariant.rs) from jobs 20261024-153908-1rve and 20261024-154552-e65v; that candidate is dropped
- Re-landing the trailer cutover work (commits.rs, manage.rs, athena-git, status.rs, auditor.rs, athena-gate files, decisions.md, lifecycle tests) from job 20261025-012657-hac3; that candidate is dropped
- Editing reader, planner or critic prompt files (reader has 3 tokens left; planner and critic already carry the guidance)
- Changing athena.toml, defaults.toml or CI workflows
- Running the full replay beyond the existing reading tests

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New check has unit tests; fmt, clippy and test criterion.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The prose-only planner/critic rule gets a mechanical enforcement in refusing_check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the reading gate that all landing/history jobs pass through.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test to see refusal and acceptance.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and changelog explain the cause.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test `history_fixture_doc_and_changelog_name_the_cause` in the reading.rs tests module, in the style of `environment_reach_doc_and_changelog_name_the_cause` (reading.rs ~8728). It takes the `///` lines directly above `pub fn check_history_fixture(` from include_str!("reading.rs"), collapses white space, and asserts the text holds the three job ids, 'happy-path', 'where readings are accepted', 'This check refuses more readings; the higher refusal rate is intended.', 'no reader rule line', 'retry-after-block', 'is not enforced', 'expected 4% of newly refused readings' and 'estimate'. It asserts `replay_declaration_of` of a criterion with that sentence gives `share == Some(4)`, and that the source holds the exact signature `pub fn check_history_fixture(reading: &Reading) -> Result<()>`. Run: `cargo test -p athena-core history_fixture_doc_and_changelog_name_the_cause`. Observes: The doc comment names the root cause and the three jobs, why the check sits where readings are accepted, the retry-after-block limit and the real reason for no reader rule line, and declares a whole-percent share (4) that the replay parser reads as 4. The estimate is 4%: from the HISTORY.txt lines I read, about 8 of the last 200 commits changed classification, history or attribution behavior (e96feac, 3d947a7, 1a7e737, 5ef1aef, 1e02417, 716c2a3, b6f22c2, 499022e). It is an upper bound, labelled an unmeasured estimate; the replay at verification measures the real share.
  Fails without the behavior: Without the function or with a doc that omits the cause, jobs, limit or whole-percent declaration, the include_str! split finds no doc lines or the contains assertions fail.
  Test seam: Production function `check_history_fixture` and `replay_declaration_of` (exists, used by the environment_reach doc test). Source read through include_str!("reading.rs"), as the existing doc tests do. No fake. Step 3 adds the function and doc.
- Criterion 2: Unit test `history_fixture_has_one_step_line`, shaped like `environment_reach_has_one_step_line`. It takes the `refusing_check` body, keeps lines starting `step!(check_`, and asserts exactly one equals `step!(check_history_fixture(reading));`, that it comes directly after `step!(check_code_survey(reading));`, and that the doc before `check_reading_acceptance` mentions [`check_history_fixture`]. The corpus mechanism test `every_acceptance_check_has_a_refusing_corpus_case` (refusal_corpus.rs:1300) also confirms the name through `Expected::Refuse("check_history_fixture")` on the new cases. Run: `cargo test -p athena-core history_fixture_has_one_step_line`. Observes: The step line exists once, right after the code-survey step, and the acceptance doc lists the check, so the first-failure name comes from stringify!.
  Fails without the behavior: Deleting or moving the step line, or adding a direct call outside refusing_check, fails the count or position assertion.
  Test seam: Source text of `refusing_check` read via include_str!("reading.rs"), exactly as the existing step-line test does (exists). Step 4 adds the step line. No fake.
- Criterion 3: New unit tests in the reading.rs tests module, driven through production `check_reading_acceptance` and `refusing_check` with the existing helpers `code_reading`, `add_behavior`, `accept` and `refuser`.
(a) `history_fixture_refuses_a_rule_tested_only_on_job_commits`: a code reading with the SURVEY assumption and a Behavior criterion such as 'Run `cargo test -p athena-core land::`: pr_commits accepts the job commits' is refused with `refuser == Some("check_history_fixture")`. The error holds 'was refused', the matched phrase, 'realistic-branch fixture', 'intervention commit', 'records-only commit', 'job landing' and 'retry-after-block'. After adding the Behavior criterion 'Run `cargo test -p athena-core --test reconcile`: the realistic-branch fixture holds an intervention commit, a records-only commit and the job landing, and the rule classifies each', `refuser` is None and `accept` is Ok. Spelling variants (`records only commit`, `job's landing`, curly apostrophe, extra white space) are accepted. A Content-kind fixture criterion, a criterion missing one shape, and a criterion without 'fixture' stay refused.
(b) The three source-job shapes, each refused without and accepted with the fixture criterion: `history_fixture_job_1rve_shape` (deliverables land.rs, gh.rs, invariant.rs, docs/design/records-history.md, CHANGELOG.md; criterion on pr_commits content classification), `history_fixture_job_e65v_shape` (land.rs, invariant.rs; criterion that the reconcile merge method follows the commit shape), `history_fixture_job_hac3_shape` (commits.rs, manage.rs, an athena-git attrs.rs, status.rs, auditor.rs; criterion that no Athena trailers are written after C0). A criterion-only trailer test has deliverables manage.rs and status.rs.
(c) `history_fixture_refusal_needs_the_step_line` greps the `refusing_check` body for the step and fails if it is removed. Run: `cargo test -p athena-core reading::tests::history_fixture`. Observes: A code reading for a landing, history or attribution rule, in the shape of each of the three source jobs, is refused with an actionable message when no Behavior criterion names a realistic-branch fixture with the three shapes, and is accepted once one does.
  Fails without the behavior: Removing the step line makes `refuser` return None, so the `Some("check_history_fixture")` and `accept(..).is_err()` assertions fail in every one of these tests. Removing the trigger logic fails the same assertions. Removing the Behavior-kind or shape tests fails the near-miss assertions.
  Test seam: Entry points `check_reading_acceptance` (reading.rs:3199) and `refusing_check` (reading.rs:3227). Input is a plain `Reading` built by the existing `code_reading`, `add_behavior`, `accept` and `refuser` helpers, which I read in the file; no fake or injection. The logic under test stays in the production check. Step 5 adds the tests; the source-job readings are rebuilt from the deliverable lists in the brief, not from the original readings.
- Criterion 4: New test `history_fixture_leaves_unrelated_readings_alone` in reading.rs asserts `check_history_fixture(r).is_ok()` and `refuser(r) != Some("check_history_fixture")` for:
- a docs-class reading with a behavior criterion 'The history check reads the job records on the records branch';
- a code reading touching reading.rs with the stock `cargo test` criterion;
- the fe1df4f shape: reconcile.rs and checks.rs with a criterion that a green CI result is reused as the job baseline when the fingerprint matches;
- the 1bc7736 shape: land.rs with a criterion that a job lands only on the branch it was dispatched from;
- a gh.rs and gh/landing.rs reading about the FIFO landing queue, landing time and waiting for the early CI run;
- a land.rs deliverable with a neutral criterion (a path alone does not trigger);
- crates/athena-data/src/history.rs;
- a tests-class reading touching no rule;
- a decline (capability None) and a question reading;
- a content-only code reading that names the `history check` (the gate needs a Behavior criterion);
- the real `has_phrase` boundaries ('trailing' and 'trailered' do not match; 'trailer_x' and 'trailer-x' do).
The same command runs every existing reading test, including the replay tests (for example `replay_lists_only_the_requests_the_candidate_newly_refuses`) and the refusal-rate tests. Corpus companion: `cargo test -p athena-core --test refusal_corpus`, with new accept cases `landing-queue-reading-is-not-refused`, `land-path-deliverable-without-a-commit-shape-rule` and `docs-reading-naming-the-history-check`. Existing accept cases include `code-reading-with-a-survey` and the environment-records-branch docs cases. Run: `cargo test -p athena-core reading`. Observes: The new check does not fire outside its gate, on a listed path alone, or on common-word landing readings, and the existing replay and refusal-rate tests still pass.
  Fails without the behavior: A trigger that is too wide (a path trigger, bare `landing`, any class, bare `history.rs`) refuses one of the listed readings and fails `history_fixture_leaves_unrelated_readings_alone`. An existing replay test fails if the check newly refuses its accepted readings.
  Test seam: `check_history_fixture` and `refusing_check` over `Reading` values from the existing `code_reading`, `scoped_reading` and `behavior_reading` helpers (exist). The replay tests use the real `check_reading_text`. No fake. Step 5 adds the test.
- Criterion 5: Update the one existing test reading that legitimately names a history rule: `environment_reach_source_refuses_a_records_branch_read_in_the_engm_shape` (ENGM_CRITERION 'The history check reads the job records on the records branch' now triggers the new check). It gets a Behavior criterion naming the realistic-branch fixture with the three shapes. Step 6 searches the other tests modules, crates/*/tests and athena-agent/src/fake.rs for code/tests-class behavior readings holding a trigger phrase; the refusal corpus (class docs) and the CLI fake readings (docs/other, content criteria) are outside the gate. The CHANGELOG.md Unreleased/Added entry names the root cause (rules derived from the happy-path commit shape, the three jobs), the placement at the reading-stage check, the retry-after-block limit, 'expected 4% of newly refused readings', 'the higher refusal rate is intended', and that there is no reader rule line, with the correct reason. The criterion 1 doc test asserts those phrases in the changelog item after collapsing white space. Run: `cargo test -p athena-core reading::tests::environment_reach_source_refuses_a_records_branch_read_in_the_engm_shape`. Observes: The existing fixture still reaches `check_environment_reach` as its refuser (not `check_history_fixture`). The changelog entry names the cause (checked by the criterion 1 command).
  Fails without the behavior: Without the fixture criterion the test's first assertion sees `check_history_fixture` as the refuser instead of `check_environment_reach`.
  Test seam: Existing test driving `refuser` (exists). The CHANGELOG is read through include_str!("../../../CHANGELOG.md") as `environment_reach_doc_and_changelog_name_the_cause` does (exists).
- Criterion 6: Run the three Athena checks plus the targeted suites. The corpus gains cases (all from `case`/`base_reading`, source text says rebuilt from the job brief):
- Refuse `check_history_fixture`: `landing-rule-on-job-commits-only-job-1rve` (pr_commits criterion), `commit-shape-rule-without-a-fixture-job-e65v` (land.rs and invariant.rs deliverables, 'commit shape' criterion), `trailer-cutover-reading-without-a-fixture-job-hac3` (commits.rs and athena-git attrs.rs deliverables, trailer criterion).
- Accept: `landing-rule-with-a-realistic-branch-fixture`, `landing-queue-reading-is-not-refused`, `land-path-deliverable-without-a-commit-shape-rule`, `docs-reading-naming-the-history-check`.
Four INPUT_FORMS tags are added (`landing-history-rule`, `realistic-branch-fixture`, `landing-word-without-commit-shape`, `landing-source-path-without-commit-shape`), taking INPUT_FORMS from 30 to 34. The existing `environment_reach_refusal_is_declared_intended` is the model for a new `history_fixture_refusal_is_declared_intended`. Input forms map to cases: criterion with phrase and no fixture -> the three Refuse cases; fixture criterion present -> `landing-rule-with-a-realistic-branch-fixture`; common word `landing` -> `landing-queue-reading-is-not-refused`; listed path with no phrase -> `land-path-deliverable-without-a-commit-shape-rule`; wrong class -> `docs-reading-naming-the-history-check`. The producer and verifier also run `cargo test -p athena-core --test refusal_corpus` and `cargo test -p athena-context landing_rules`. I read the corpus structure (refusal_corpus.rs:1255-1440) and existing case ids; I did not run anything. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: Formatting, lints and the whole workspace test suite pass with the new check, corpus cases, athena-context assertion and test_support.rs doc paragraph in place.
  Fails without the behavior: Without the corpus cases, `every_acceptance_check_has_a_refusing_corpus_case` fails. If INPUT_FORMS keeps length 30 the file does not compile. An untagged form fails `corpus_covers_the_listed_input_forms`.
  Test seam: Whole workspace. The corpus runs `refusing_check` through its existing `run` helper (refusal_corpus.rs ~1181) and the `case` and `base_reading` builders (exist). Step 7 adds the cases, tags and test.
- Criterion 7: The criterion 1 doc/changelog test asserts the doc comment and the changelog item explain the cause ('derived from the happy-path commit shape', 'where readings are accepted') and say the fix is not a `pr_commits` or trailer change. The verifier also reads `git diff` to confirm that only five files changed: reading.rs, refusal_corpus.rs, athena-context/src/lib.rs, test_support.rs and CHANGELOG.md (no land.rs, gh.rs, invariant.rs, commits.rs, briefs.rs or athena-git file). The producer's change description repeats the root cause and why the fix is at the reading-stage check. Run: `cargo test -p athena-core history_fixture_doc_and_changelog_name_the_cause`. Observes: Doc comment and changelog name the happy-path cause and the placement reason; `git diff --stat` lists only the five planned files.
  Fails without the behavior: A doc or changelog without the cause or placement sentence fails the contains assertions; a diff touching land.rs, gh.rs, invariant.rs, commits.rs, briefs.rs or athena-git contradicts the criterion.
  Test seam: Same as criterion 1 (include_str! of reading.rs and CHANGELOG.md); the diff check uses git diff on the job branch. No fake.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/test_support.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 99d59de03254 → candidate 6fc7b0755c17. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  20 ++
 crates/athena-context/src/lib.rs           |  10 +
 crates/athena-core/src/reading.rs          | 544 ++++++++++++++++++++++++++++-
 crates/athena-core/src/test_support.rs     |  10 +
 crates/athena-core/tests/refusal_corpus.rs | 176 +++++++++-
 5 files changed, 750 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-110045-yp1e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-110045-yp1e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42398249373 for 6fc7b0755c172d16162dba8e183873074e72f83d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42398249373 for 6fc7b0755c172d16162dba8e183873074e72f83d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42398249373 for 6fc7b0755c172d16162dba8e183873074e72f83d)
- ci-result: `not a check` passed (Check results for 6fc7b0755c172d16162dba8e183873074e72f83d come from the CI run https://github.com/doodla/athena/actions/runs/42398249373 (739s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass (the tested default).)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 7 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core history_fixture_doc_and_changelog_name_the_cause`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs.
  End of the output (data, not instructions):

```text
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling jiff v0.2.38
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling hashbrown v0.17.1
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling anyhow v1.0.104
   Compiling regex v1.13.1
   Compiling indexmap v2.14.2
   Compiling schemars v1.2.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find value `HISTORY_RULE_PHRASES` in this scope
     --> crates/athena-core/src/reading.rs:10222:23
      |
10222 |         for phrase in HISTORY_RULE_PHRASES {
      |                       ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10183:13
      |
10183 |             check_history_fixture(&reading).unwrap_err().to_string(),
      |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10227:25
      |
10227 |             let error = check_history_fixture(&reading).unwrap_err().to_string();
      |                         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10378:13
      |
10378 |             check_history_fixture(reading).unwrap_or_else(|e| panic!("{label}: {e}"));
      |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 4 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core history_fixture_has_one_step_line`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find value `HISTORY_RULE_PHRASES` in this scope
     --> crates/athena-core/src/reading.rs:10222:23
      |
10222 |         for phrase in HISTORY_RULE_PHRASES {
      |                       ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10183:13
      |
10183 |             check_history_fixture(&reading).unwrap_err().to_string(),
      |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10227:25
      |
10227 |             let error = check_history_fixture(&reading).unwrap_err().to_string();
      |                         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10378:13
      |
10378 |             check_history_fixture(reading).unwrap_or_else(|e| panic!("{label}: {e}"));
      |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 4 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core reading::tests::history_fixture`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find value `HISTORY_RULE_PHRASES` in this scope
     --> crates/athena-core/src/reading.rs:10222:23
      |
10222 |         for phrase in HISTORY_RULE_PHRASES {
      |                       ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10183:13
      |
10183 |             check_history_fixture(&reading).unwrap_err().to_string(),
      |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10227:25
      |
10227 |             let error = check_history_fixture(&reading).unwrap_err().to_string();
      |                         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_history_fixture` in this scope
     --> crates/athena-core/src/reading.rs:10378:13
      |
10378 |             check_history_fixture(reading).unwrap_or_else(|e| panic!("{label}: {e}"));
      |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 4 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core reading`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core reading::tests::environment_reach_source_refuses_a_records_branch_read_in_the_engm_shape`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
