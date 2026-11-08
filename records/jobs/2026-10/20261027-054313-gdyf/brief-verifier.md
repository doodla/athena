# Job 20261027-054313-gdyf: verify "Refuse scope limits that omit default-written and dependent files"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The reading wrote a scope limit from the files the request named, without searching the repo for eve
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: scope-criterion-too-narrow-misses-dependent-tests (20261018-045551-4h0h)
> Cause: The reading wrote a scope limit from the files the request named, without searching the repo for every consumer of the changed config. Nothing in the reading step finds all the tests that load the real athena.toml. The strict criterion woul...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261018-045542-tem7 (job 20261018-045437-7z81); prevention it proposed: When a reading restricts edits for a config-changing job, a check should first list every file in the repo that loads the changed config file (for example by searching for athena.toml reads in tests and code). The reading should then allow ...
> Introspection 20261023-052812-187k (job 20261023-052543-9bw7); prevention it proposed: Add a check at reading acceptance. For any criterion that limits which files may change, the check builds the set of files that Athena's defaults always write, such as CHANGELOG.md, and compares it with the allowed list. If a default file i...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (class scope-criterion-too-narrow-misses-dependent-tests, jobs 20261018-045437-7z81 and 20261023-052543-9bw7): reading acceptance never checks a "only these files may change" limit against files Athena always writes (CHANGELOG.md) or the consumers of a changed config. Fix it in reading acceptance with one new step in `refusing_check`, so the reader is refused and repairs the reading before a producer runs.

Assumptions:
- Goal and audience: stop readings from writing a file-scope limit too narrow; the audience is the reader role, whose reading is judged by the reading gate, and then verifiers who fail a job that edits a default or dependent file.
- Previous jobs: 20261018-045437-7z81 landed and verified (pass); its only findings were a nit on the triplicated [job_plan]-stripping test helper (not touched, still allowed by its criterion 4) and a minor note that the producer output was not visible; neither needs a fix here. Its changed files (athena.toml, the three test files, docs/decisions.md, docs/design/job-planning.md, CHANGELOG.md) are kept as landed and are all out_of_scope for this job. Job 20261023-052543-9bw7 was abandoned with no candidate (its content could not land because the base held a reverted spec); its design work is not redone here, and docs/design/records-history.md is out_of_scope. Its scope limit omitted CHANGELOG.md, which Athena always writes; that is the second instance this job fixes.
- Code survey: read crates/athena-core/src/reading/checks.rs (check_reading_acceptance and refusing_check run checks as `step!` lines in order; check_governance_coverage at line 607 and GOVERNANCE_BY_DELIVERABLE at 559 map a deliverable to a governance file), and crates/athena-core/src/reading.rs and crates/athena-core/tests/refusal_corpus.rs (each step! needs a corpus case in which that check refuses). The change adds no record or data file under crates/, so check_record_file and AnyRecord are not touched. The property is measured directly by calling refusing_check on a Reading and asserting the returned check name.
- Existing code: crates/athena-core/src/reading/checks.rs (refusing_check, check_governance_coverage, governance_is_covered, GOVERNANCE_BY_DELIVERABLE) and crates/athena-core/src/admission.rs (is_governance); the new check extends these step and table conventions and reuses the Reading fields, rather than adding a second mechanism.
- Design of the new check, named check_scope_limit: a criterion or out_of_scope item is a scope limit when it says only/no other/nothing else files may change. When a reading has one, it must list CHANGELOG.md as a deliverable (Athena's default write) or name it in the limit; and when athena.toml or defaults.toml is a modify deliverable, it must list a crates/ test deliverable or carry an assumption starting `Consumers:` naming the search of the repo for files that load that config. The error names the missing file and why.
- The reader prompt has 3 tokens left, so the prompt text is not changed; the check list in crates/athena-context/src/briefs.rs may need one short line, and the producer must stay inside the prompt token budget (or place it in a set with room).
- Term ladder: not used; no structural term from the request needs defining.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/checks.rs defines `check_scope_limit` with a doc comment that names the root cause (a scope limit written from the named files without listing default-written files and config consumers) and cites jobs 20261018-045437-7z81 and 20261023-052543-9bw7, and `refusing_check` calls it as a single `step!(check_scope_limit(..))` line.
2. [behavior] Run `cargo test -p athena-core`: a unit test in crates/athena-core/src/reading.rs builds a reading with a scope-limit criterion and no CHANGELOG.md deliverable and asserts refusing_check returns "check_scope_limit"; the same reading with CHANGELOG.md listed is accepted. The test fails when the step! line is removed.
3. [behavior] Run `cargo test -p athena-core`: a test builds a reading with a modify deliverable athena.toml, a scope limit, and neither a crates/ test deliverable nor a `Consumers:` assumption, and asserts it is refused by check_scope_limit; adding either makes it pass. A reading without a scope limit and a reading with no config deliverable are unaffected.
4. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a corpus case in crates/athena-core/tests/refusal_corpus.rs expects Refuse("check_scope_limit") using the shape of job 20261023-052543-9bw7's scope, and the corpus's rule that every step! has a refusing case passes.
5. [behavior] Run `cargo test`: the whole workspace passes, so existing accepted readings in the corpus are not newly refused beyond the intended cases: expected 3% of newly refused readings, and the higher refusal rate is intended because a scope limit that omits files Athena always writes would fail verification.
6. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass.
7. [content] CHANGELOG.md has an entry for the new check, and docs/decisions.md records the call (rule enforced by mechanism, not prose) naming the failure class scope-criterion-too-narrow-misses-dependent-tests.
8. [content] git diff shows no change to athena.toml, defaults.toml, workflows, records/, or the files kept from job 20261018-045437-7z81 beyond what is listed as deliverables.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Files changed by job 20261018-045437-7z81 (athena.toml, crates/athena-cli/tests/common/mod.rs, crates/athena-data/src/config.rs, crates/athena-core/src/run_pin_tests.rs, docs/design/job-planning.md): kept as landed, not touched
- Merging the three duplicated [job_plan]-stripping test helpers (a nit in job 20261018-045437-7z81)
- docs/design/records-history.md and the records-history design of job 20261023-052543-9bw7
- Changes to athena.toml, defaults.toml, workflows, [history] exempt entries, or records/
- Changing prompt text of the reader role (3 tokens left)
- Scanning file contents of the repo for consumers of a config file

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The check and its tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and a corpus case; cargo test, clippy and fmt required.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is a refusing step in reading acceptance, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in reading acceptance, covering all scope limits, not the two jobs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail without the step.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that a too-narrow scope limit is refused before a producer runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new check and its tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in the reading.rs tests (pattern of `history_fixture_doc_and_changelog_name_the_cause`, reading.rs:7945) reads checks.rs and CHANGELOG.md with `include_str!`, takes the contiguous `///` block above `pub fn check_scope_limit(`, and asserts it holds the root-cause sentence (a scope limit written from the named files without listing default-written files and config consumers) and both job ids 20261018-045437-7z81 and 20261023-052543-9bw7. It also asserts the body of `refusing_check` holds exactly one line `step!(check_scope_limit(reading));`. Run: `cargo test -p athena-core scope_limit_doc_and_changelog_name_the_cause`. Observes: The test passes: the doc comment, the single step line and the needles exist and name both jobs.
  Fails without the behavior: Without the function, its doc comment or the step line, the split/count assertions fail; removing a job id from the doc comment fails the needle checks.
  Test seam: Production entry points: the source text of checks.rs and CHANGELOG.md read through `include_str!`, as the existing doc tests do. No double. The seam exists today (reading.rs:7945 pattern); plan step 8 adds the test.
- Criterion 2: Unit test `scope_limit_changelog` calls `refusing_check` through the existing `gate_refuser` helper (reading.rs:6977) on readings built by `governed_reading` (reading.rs:4329; capability change, no question), always with a CHANGELOG.md deliverable (production shape after `ensure_changelog`) except the first case. Refused with Some("check_scope_limit"): 'Only docs/x.md may change' with no CHANGELOG.md deliverable; the same with the deliverable; 'test-only change'; 'only the docs may change'; 'Only the deliverable src/a.rs may change'; 'Only docs/x.md may change; CHANGELOG.md is not changed'; 'Only docs/x.md may change and CHANGELOG.md stays as it is'; 'No file outside docs/x.md changed'. Accepted (None): 'Only docs/x.md and CHANGELOG.md may change'; '... may change, keeping one entry'; 'Only docs/x.md and CHANGELOG.md may change; src not changed'; 'No file outside docs/x.md and CHANGELOG.md changed'; 'Only the deliverables may change'; 'Test-only change that also edits CHANGELOG.md'; bare 'No other file is changed'; 'No other file is changed beyond formatting' (finding 4); 'Only maybe tidy things; no file outside docs/x.md and CHANGELOG.md changed' (an earlier unmatched 'only', finding 4); 'production unchanged'; the same limits as an out_of_scope item, or on a reading with a question or a capability other than change. A second test asserts `check_reading_acceptance` returns Err whose text names CHANGELOG.md, its reason and the repair. A third assertion, for the queue effect, calls `check_reading_text(&reading, request)` (replay.rs:54, the call `queued_work` makes at steward.rs:421) on the limit reading and expects `ReadingCheck::Refuse`, and Accept after the name is added. Criterion 2's 'with CHANGELOG.md listed' is shown as CHANGELOG.md named in the limit's allowed list. A briefs.rs test `change_limit_reports_the_allowed_words` pins the `allowed` and `bare` fields for each form and that `limits_changed_files` agrees with `change_limit(..).is_some()` on every phrase the existing matcher tests use. Run: `cargo test -p athena-core scope_limit_changelog && cargo test -p athena-context change_limit_reports_the_allowed_words`. Observes: The first reading is refused under the name check_scope_limit; the repaired, in-list and bare readings return None; check_reading_text refuses then accepts; the briefs test passes.
  Fails without the behavior: Without the step line `refusing_check` returns None for the first reading and the Some("check_scope_limit") assertion fails; a rule that accepted a CHANGELOG.md deliverable alone fails the production-shaped assertion; one that fired only on named files fails 'test-only change'; one that accepted any 'deliverable' word fails 'the deliverable src/a.rs'; one that accepted a name outside the allowed list fails 'is not changed' and 'stays as it is'; one that refused a name inside the list fails 'keeping one entry'; a second parser that opened the span at an unmatched or non-matcher trigger fails the 'beyond formatting' and earlier-'only' assertions.
  Test seam: Production entry points: `refusing_check`, `check_reading_acceptance` (checks.rs:515, :487), `check_reading_text` (replay.rs:54) and `change_limit` (briefs.rs, added in step 1). Fixture: an in-memory `Reading` from `governed_reading`/`scoped_reading` (reading.rs:4329, 3473) passed as an argument; no double. The seam exists today; steps 1-3 add the function and check, step 8 the tests.
- Criterion 3: Unit test `scope_limit_config_consumers` builds `governed_reading(&["athena.toml", "docs/x.md"], &["Only athena.toml, docs/x.md and CHANGELOG.md may change"], &[])` (CHANGELOG.md is in the allowed list so only the config half fires; athena.toml is named so `check_governance_coverage` passes). Mapping of criterion 3's 'neither a crates/ test deliverable nor a Consumers: assumption ... adding either makes it pass': adding the test deliverable counts when that consumer is also a deliverable named in a Consumers line; adding the assumption counts when the line says unchanged / none found or names only deliverables. Refused (check_scope_limit; error names athena.toml and `Consumers:` and the repair): the base reading; the base plus an unrelated crates/athena-core/tests/refusal_corpus.rs deliverable and no Consumers line; `Consumers: crates/athena-cli/tests/common/mod.rs loads athena.toml` when that path is not a deliverable; the same path plus 'must be updated; others unchanged' (finding 5: still refused); `Consumers: crates/a.rs unchanged` when crates/a.rs is not a deliverable (refused); `Consumers: crates/a.rs (deliverable), xtask/tests/policy.rs` with the xtask path not a deliverable; a Consumers line with no path and no phrase; a line naming only athena.toml. Accepted (None): the same Consumers line with crates/athena-cli/tests/common/mod.rs added as a deliverable; `Consumers: xtask/tests/policy.rs` with that deliverable; a path under a deliverable directory ending in '/'; `Consumers: tests that load athena.toml unchanged`; `Consumers: none found, searched the repo for athena.toml reads`. Unaffected (None): the same deliverables with a criterion that is not a limit; a limit with docs/x.md and no config deliverable; defaults.toml covered by the same cases. A second test `scope_limit_proposal_apply_shape` uses the request text of `accept_request` (the line 'Deliverable: defaults.toml (modify). Make one change to it and touch no other file.' plus bullets), a defaults.toml modify deliverable and the criterion 'No other file is changed': refused with no Consumers line, the error text containing 'Consumers:' and 'none found'; accepted with 'Consumers: none found' and with 'Consumers: tests that load defaults.toml unchanged'. A drift test asserts `include_str!("proposals.rs")` still holds that request line. Run: `cargo test -p athena-core scope_limit_config_consumers && cargo test -p athena-core scope_limit_proposal_apply_shape`. Observes: The config reading is refused until every named consumer path is a deliverable or the line names no path and says unchanged or none found; readings without a limit or without a config deliverable return None; the proposal-apply shape is refused once and accepted with the repair line.
  Fails without the behavior: Without the config half the first assertion returns None and the test fails; a rule that accepted any test deliverable fails the 'unrelated test deliverable' assertion; a rule that accepted 'unchanged' beside a non-deliverable path fails the finding-5 assertions; a crates/-only parser fails the xtask assertions; a rule that fired without a limit fails the 'not a limit' assertion; a rule that exempted bare limits fails the proposal-apply refusal.
  Test seam: Production entry points: `refusing_check` through `gate_refuser`, `check_reading_acceptance` with the request text argument, and `check_scope_limit` directly. Fixture: in-memory `Reading`; no double. The seam exists today; step 2 adds the check and step 8 the tests.
- Criterion 4: Step 4 adds `scope_limit_cases()` to `cases()` in crates/athena-core/tests/refusal_corpus.rs, built with the existing `case(..)` helper from `base_reading` with one field changed each (harness `run(&Case)` at refusal_corpus.rs:2189 and the wrapper test at 2220). Refuse("check_scope_limit"), 12 cases: 'only X may change' with a dash bullet, shape of 9bw7's scope (`scope-limit-only-may-change-omits-changelog`); 'no file outside X changed' with a star bullet (`scope-limit-no-file-outside-omits-changelog`); a `1.` numbered request (`scope-limit-numbered-part-omits-changelog`); 'test-only change' naming no file (`scope-limit-test-only-change-omits-changelog`); 'only the docs may change' (`scope-limit-only-the-docs-omits-changelog`); 'Only the deliverable src/a.rs may change' (`scope-limit-singular-deliverable-omits-changelog`); a limit that puts CHANGELOG.md outside the allowed list and forbids it (`scope-limit-names-changelog-but-forbids-it`); a config deliverable with a limit and no Consumers line, shape of 7z81 (`scope-limit-config-deliverable-without-consumers`); a Consumers line naming a non-deliverable crates/ path (`scope-limit-consumer-not-a-deliverable`); the same path with 'unchanged' on the line (`scope-limit-consumer-not-a-deliverable-says-unchanged`); a non-deliverable xtask/ path (`scope-limit-xtask-consumer-not-a-deliverable`); the proposal-apply shape: request line from `accept_request`, defaults.toml modify deliverable, criterion 'No other file is changed', no Consumers line (`scope-limit-proposal-apply-without-consumers`). Accept, 13 cases: bare 'No other file is changed' (`scope-limit-bare-no-other-file`); 'No other file is changed beyond formatting' (`scope-limit-bare-beyond-formatting`); an earlier unmatched 'only' before a 'no file outside X and CHANGELOG.md changed' limit (`scope-limit-earlier-only-then-outside-list`); 'production unchanged' (`scope-limit-bare-production-unchanged`); a limit with CHANGELOG.md in the allowed list (`scope-limit-names-changelog`); 'Only X and CHANGELOG.md may change, keeping one entry' (`scope-limit-names-changelog-with-keeping`); a limit pointing to 'the deliverables' (`scope-limit-points-to-deliverables`); config deliverable with Consumers paths that are deliverables (`scope-limit-config-consumers-are-deliverables`); with an xtask/ deliverable consumer (`scope-limit-xtask-consumer-is-a-deliverable`); with 'unchanged' and no path (`scope-limit-config-consumers-unchanged`); with 'none found' (`scope-limit-config-consumers-none-found`); a config deliverable with no limit (`config-deliverable-without-a-limit`); the proposal-apply shape with 'Consumers: none found' (`scope-limit-proposal-apply-with-consumers`). New `INPUT_FORMS` tags: `scope-limit-omits-default-file`, `scope-limit-forbids-default-file`, `scope-limit-config-without-consumers`, `scope-limit-names-default-file`, `scope-limit-consumer-not-a-deliverable`, `scope-limit-proposal-apply`, `scope-limit-bare-form`; the array length is updated. athena.toml, defaults.toml and the consumer paths go in the cases' `repo_files` so the later boundary check passes. I read the corpus layout (existing cases such as `code-reading-without-a-survey`, `criterion-rests-on-the-producer-report`) but saw no test result; the producer and verifier run the command. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: All cases give their expected outcome (`corpus_cases_get_the_expected_outcome`, which also checks bullet counts); `every_acceptance_check_has_a_refusing_corpus_case` finds a case in which check_scope_limit refuses; `corpus_covers_the_listed_input_forms` and `the_wrapper_agrees_with_refusing_check_on_every_case` pass.
  Fails without the behavior: Without the step the twelve Refuse cases are accepted and `corpus_cases_get_the_expected_outcome` fails; without the cases `every_acceptance_check_has_a_refusing_corpus_case` fails with 'no corpus case in which ["check_scope_limit"] refuses'.
  Test seam: Production entry points: `refusing_check` and `check_reading_acceptance`, called by `run(&Case)` and the wrapper test in refusal_corpus.rs. Fixture: `Case` values built in memory by `case`/`base_reading`; no double. The harness exists today; step 4 adds the cases and tags.
- Criterion 5: Run the whole workspace suite. Newly refused readings are limited to those whose first limiting criterion is non-bare and omits CHANGELOG.md and 'the deliverables' from the matcher's allowed words, or whose config deliverable lacks a valid Consumers line. The one fixture found in crates/ that reaches the gate with such a limit is `LIMIT` in run/tests.rs:3213 (changed in step 5). Other limit phrases call `limits_changed_files`/`allowed_paths` directly (run_pin_tests.rs:1355-1396, briefs.rs tests) and do not reach the gate. cli tests with athena.toml as a deliverable (lifecycle.rs:872, 1009; common/mod.rs:245; job_plan.rs:700) have no limiting criterion, so the check returns Ok. drain.rs:4063-4095 seeds readings whose criteria are 'the producer's report lists the caller-check Grep', 'the demo study has a pin, claims and an intro' and 'the tests still pass': none limits files, so their Refuse and Accept outcomes do not change. The cli proposals test (crates/athena-cli/tests/proposals.rs:206) runs `accept_proposal` through the real gate with a fake reader; I did not read the criteria that fake supplies, so the suite run is the evidence. The 3% share is an estimate (not measured) and is measured by the verification replay (decision 9), not by this run. Run: `cargo test --locked`. Observes: All tests pass, including the scope tests in run/tests.rs, the cli lifecycle, drain and proposals tests, steward tests, briefs tests and the budgets test.
  Fails without the behavior: If LIMIT in run/tests.rs were not updated, the new step refuses the fake reader's reading and the scope tests (run/tests.rs near 3340-3420) fail; if the check wrongly fired on the base reading, the corpus Accept cases fail; if `change_limit` changed what `limits_changed_files` matches, the existing briefs and run_pin_tests matcher tests fail.
  Test seam: The whole existing suite with its in-memory fakes (the athena_agent::fake script built by `scoped` in run/tests.rs:3228; the Fixture in crates/athena-cli/tests/proposals.rs and drain.rs). The seam exists today; step 5 edits the fixture constant.
- Criterion 6: Run the two gates Athena runs over the whole workspace after all edits. The new helpers are private and used only by `check_scope_limit`; `change_limit` and `ChangeLimit` are pub and used; imports are used. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings`. Observes: Both commands exit 0: fmt reports no diff and clippy reports no warning.
  Fails without the behavior: A badly formatted line makes `cargo fmt --all --check` exit nonzero; an unused import, an unused helper (dead_code) or a needless clone makes clippy exit nonzero under `-D warnings`.
  Test seam: Production entry points: the same fmt and clippy commands Athena runs on the result. Fixture: the working tree with the new code, tests and corpus cases; no double. The seam exists today; step 10 runs both.
- Criterion 7: The doc test of criterion 1 also asserts the Unreleased CHANGELOG item naming `check_scope_limit` exists with the failure class name 'scope-criterion-too-narrow-misses-dependent-tests', both job ids, 'higher refusal rate is intended' with 'expected 3% of newly refused readings' and 'estimate', the sentence mapping criterion 2's 'CHANGELOG.md listed' to 'named in the limit' with reading.rs:859, the proposal-apply Consumers sentence, the sentence that Proposed jobs with such a stored limit leave the queue until read again, and the sentence that run/tests.rs `LIMIT` changed because it goes through the gate. `git diff -- docs/decisions.md` shows an appended D166 whose Case, Decision, Alternatives, Tradeoff, Reversal cost and Provenance bullets name the failure class, say the rule is enforced by mechanism (a `step!` in `refusing_check`) and not prose, and record the one matcher in briefs.rs, the revised sense of criterion 3, the finding-5 Consumers rule, the proposal-apply decision, the known misses, the unverified search and the queue effect. The log is append-only (mirrors.rs:23, 241-265), so D166 follows D165. Run: `cargo test -p athena-core scope_limit_doc_and_changelog_name_the_cause && cargo test -p athena-core mirrors`. Observes: Both pass; the docs/decisions.md diff contains only added lines and CHANGELOG.md gains one item.
  Fails without the behavior: Without the CHANGELOG item the needle assertions fail; a D166 that breaks the entry shape or edits earlier lines fails the mirrors tests.
  Test seam: Production entry points: CHANGELOG.md read by `include_str!` in the unit test; the decision-log structure check in mirrors.rs. No double. The seam exists today; step 9 adds the entries and step 8 the test.
- Criterion 8: Read the diff stat: only the seven files in `files` appear (the six named deliverables plus the one-constant run/tests.rs edit, under crates/ and raised as a User question). athena.toml, defaults.toml, .github/workflows, records/, proposals.rs, steward.rs, crates/athena-cli/tests/common/mod.rs, crates/athena-data/src/config.rs, crates/athena-core/src/run_pin_tests.rs and docs/design/*.md are absent. Run: `git diff --stat`. Observes: No path outside the seven listed files is changed.
  Fails without the behavior: If the producer touched athena.toml, defaults.toml, a workflow, records/, proposals.rs, steward.rs or a kept file, the stat would list it and the criterion would fail.
  Test seam: Production entry point: git's own diff of the working tree against the base; no fixture or double. The seam exists today.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 1 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 3% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): Run `cargo test`: the whole workspace passes, so existing accepted readings in the corpus are not newly refused beyond the intended cases: expected 3% of newly refused readings, and the higher refusal rate is intended because a scope limit that omits files Athena always writes would fail verificatio

- 20261108-142640-egnb: the reading was refused because a criterion that limits which files may change leaves out files the job must change: defaults.toml is a deliverable and a criterion limits which files may change, so th

## The change

Base 06857fa0d1b8 → candidate c5c4849d6d41. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  25 ++
 crates/athena-context/src/briefs.rs        | 120 ++++++++-
 crates/athena-core/src/reading.rs          | 339 ++++++++++++++++++++++++
 crates/athena-core/src/reading/checks.rs   | 178 ++++++++++++-
 crates/athena-core/src/run/tests.rs        |   5 +-
 crates/athena-core/tests/refusal_corpus.rs | 408 ++++++++++++++++++++++++++++-
 docs/decisions.md                          |  55 ++++
 7 files changed, 1114 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054313-gdyf/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054313-gdyf/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42520856560 for c5c4849d6d4143b68e242e901b629f206be27d08)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42520856560 for c5c4849d6d4143b68e242e901b629f206be27d08)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42520856560 for c5c4849d6d4143b68e242e901b629f206be27d08)
- ci-result: `not a check` passed (Check results for c5c4849d6d4143b68e242e901b629f206be27d08 come from the CI run https://github.com/doodla/athena/actions/runs/42520856560 (747s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test`: the whole workspace passes, so existing accepted readings in the corpus are not newly refused beyond the intended cases: expected 3% of newly refused readings, and the higher refusal rate is intended because a scope limit that omits files Athena always writes would fail verification.; Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core scope_limit_doc_and_changelog_name_the_cause`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading/checks.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling serde_derive v1.0.229
   Compiling itoa v1.0.18
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling rustix v1.1.5
   Compiling bitflags v2.13.2
   Compiling ref-cast v1.0.27
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `check_scope_limit` in this scope
    --> crates/athena-core/src/reading.rs:8084:17
     |
8084 |         assert!(check_scope_limit(&reading).is_ok());
     |                 ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_scope_limit` in this scope
    --> crates/athena-core/src/reading.rs:8087:17
     |
8087 |         assert!(check_scope_limit(&reading).is_ok());
     |                 ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_scope_limit` in this scope
    --> crates/athena-core/src/reading.rs:8208:21
     |
8208 |         let error = check_scope_limit(&reading).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core scope_limit_changelog && cargo test -p athena-context change_limit_reports_the_allowed_words`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-core scope_limit_config_consumers && cargo test -p athena-core scope_limit_proposal_apply_shape`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading/checks.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.62s
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 23 tests
test ambiguous_name_changelog_and_decision ... ok
test approve_request_has_the_generators_shapes ... ok
test defined_terms_decision_and_changelog ... ok
test defined_terms_replay_over_the_cited_jobs ... ok
test delivery_verb_replay_over_the_cited_jobs ... ok
test corpus_covers_the_listed_input_forms ... ok
test design_existing_code_refusal_is_declared_intended ... ok
test an_extra_check_with_no_case_is_reported ... ok
test environment_cache_scope_refusal_is_declared_intended ... ok
test generated_request_has_the_generators_shapes ... ok
test environment_reach_refusal_is_declared_intended ... ok
test no_check_bypasses_refusing_check ... ok
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::an_impact_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::impact_cases_get_the_expected_outcome ... ok
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined ... ok
test corpus_cases_get_the_expected_outcome ... FAILED
test history_fixture_refusal_is_declared_intended ... ok
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok

failures:

---- corpus_cases_get_the_expected_outcome stdout ----

thread 'corpus_cases_get_the_expected_outcome' (30224) panicked at crates/athena-core/tests/refusal_corpus.rs:2622:5:
scope-limit-only-may-change-omits-changelog (job 20261023-052543-9bw7, a limit that omits CHANGELOG.md; rebuilt from the job brief): expected Refuse("check_scope_limit"), got accepted
scope-limit-no-file-outside-omits-changelog (job 20261023-052543-9bw7, a limit that omits CHANGELOG.md; rebuilt from the job brief): expected Refuse("check_scope_limit"), got accepted
scope-limit-numbered-part-omits-changelog (job 20261023-052543-9bw7, a limit that omits CHANGELOG.md; rebuilt from the job brief): expected Refuse("check_scope_limit"), got accepted
scope-limit-test-only-change-omits-changelog (job 20261023-052543-9bw7, a limit that omits CHANGELOG.md; rebuilt from the job brief): expected Refuse("check_scope_limit"), got accepted
scope-limit-only-the-docs-omits-changelog (job 20261023-052543-9bw7, a limit that omits CHANGELOG.md; rebuilt from the job brief): expected Refuse("check_scope_limit"), got accepted
scope-limit-singular-deliverable-omits-changelog (job 20261023-052543-9bw7, a limit that omits CHANGELOG.md; rebuilt from the job brief): expected Refuse("check_scope_limit"), got accepted
scope-limit-names-changelog-but-forbids-it (job 20261023-052543-9bw7, a limit that omits CHANGELOG.md; rebuilt from the job brief): expected Refuse("check_scope_limit"), got accepted
scope-limit-config-deliverable-without-consumers (job 20261018-045437-7z81, a limit that omits the tests that load athena.toml; rebuilt from the job brief): expected Refuse("check_scope_limit"), got accepted
scope-limit-consumer-not-a-deliverable (job 20261018-045437-7z81, a limit that omits the tests that load athena.toml; rebuilt from the job brief): expected Refuse("check_scope_limit"), got refused by check_reading_references: the reading was refused because its text relies on repo paths that are not in the snapshot at the base commit: assumptions[0]: crates/athena-cli/tests/common/mod.rs; an earlier job's file, role or prompt is not there until that job lands. Repair: list each missing artifact as a create deliverable if this change adds it, or say in the text that it does not exist, or name the path that does exist
scope-limit-consumer-not-a-deliverable-says-unchanged (job 20261018-045437-7z81, a limit that omits the tests that load athena.toml; rebuilt from the job brief): expected Refuse("check_scope_limit"), got refused by check_reading_references: the reading was refused because its text relies on repo paths that are not in the snapshot at the base commit: assumptions[0]: crates/athena-cli/tests/common/mod.rs; an earlier job's file, role or prompt is not there until that job lands. Repair: list each missing artifact as a create deliverable if this change adds it, or say in the text that it does not exist, or name the path that does exist
scope-limit-xtask-consumer-not-a-deliverable (job 20261018-045437-7z81, a limit that omits the tests that load athena.toml; rebuilt from the job brief): expected Refuse("check_scope_limit"), got refused by check_reading_references: the reading was refused because its text relies on repo paths that are not in the snapshot at the base commit: assumptions[0]: xtask/tests/policy.rs; an earlier job's file, role or prompt is not there until that job lands. Repair: list each missing artifact as a create deliverable if this change adds it, or say in the text that it does not exist, or name the path that does exist
scope-limit-proposal-apply-without-consumers (the request of a defaults-proposal job (`accept_request` in proposals.rs) with the reading its reader writes): expected Refuse("check_scope_limit"), got accepted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    corpus_cases_get_the_expected_outcome

test result: FAILED. 22 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading/checks.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling utf8parse v0.2.2
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-query v1.1.5
   Compiling anstyle v1.0.14
   Compiling anstyle-parse v1.0.0
   Compiling colorchoice v1.0.5
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.1
   Compiling anstream v1.0.0
   Compiling heck v0.5.0
   Compiling clap_builder v4.6.7
   Compiling crc32fast v1.5.2
   Compiling simd-adler32 v0.3.10
   Compiling adler2 v2.0.1
   Compiling miniz_oxide v0.9.1
   Compiling xattr v1.6.1
   Compiling filetime v0.2.29
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling clap_derive v4.6.7
   Compiling serde v1.0.229
   Compiling clap v4.6.7
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
error[E0425]: cannot find function `change_limit` in this scope
    --> crates/athena-context/src/briefs.rs:5231:34
     |
5231 |         let named = |text: &str| change_limit(text).unwrap_or_else(|| panic!("no limit: {text}"));
     |                                  ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `change_limit` in this scope
    --> crates/athena-context/src/briefs.rs:5262:20
     |
5262 |         assert_eq!(change_limit("The helper is test-only"), None);
     |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `change_limit` in this scope
    --> crates/athena-context/src/briefs.rs:5277:17
     |
5277 |                 change_limit(text).is_some(),
     |                 ^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-core scope_limit_doc_and_changelog_name_the_cause && cargo test -p athena-core mirrors`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
