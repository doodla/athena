# Job 20261109-161505-bp5e: verify "Refuse a change at admission when it omits the changelog deliverable"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The changelog default is a rule the producer must remember, and no code compares the diff's file lis
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: changelog-default-deliverable-not-produced-caught-only-at-final-verdict (20261108-185500-x0jg)
> Cause: The changelog default is a rule the producer must remember, and no code compares the diff's file list with the expected deliverables before the verdict. For doc-only design jobs the changelog feels optional, so it is easy to skip. The readi...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261108-185451-t573 (job 20261108-183842-fsaz); prevention it proposed: Add a code check that runs before the verifier. When the changelog default applies to a job, it fails fast if the diff does not touch CHANGELOG.md. It should report the missing file to the producer, which gets a repair round. The reading sh...
> Introspection 20261109-161451-cn7n (job 20261028-000831-rkv5); prevention it proposed: Add a code check before handover and verification. When the changelog default applies, the check fails the producer's run if the diff has no CHANGELOG.md change, and it names the missing file. Readings should list CHANGELOG.md as a delivera...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the `changelog` default is added to every change reading's deliverables by `ensure_changelog` (crates/athena-core/src/reading.rs), but nothing in code compares the candidate's changed files with that deliverable before the verifier, so jobs 20261108-183842-fsaz and 20261028-000831-rkv5 were failed by the final verdict only. Fix: add an admission rule in crates/athena-core/src/admission.rs that raises a violation naming CHANGELOG.md when the reading lists it, the changelog default applies, and the candidate's changes do not touch it.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (ensure_changelog adds CHANGELOG.md modify p=0.9 and the `changelog` default for capability change), crates/athena-core/src/admission.rs (admit / admit_without_secrets take AdmissionInput with reading and changes, and return PolicyViolation entries), and crates/athena-core/src/run/mod.rs (~line 773-814: admission violations are summarised and fail the candidate before check_and_verify). No record or data file is added under crates/, so check_record_file/AnyRecord are not affected. The property is measured directly by comparing input.changes paths with the reading's deliverables.
- Job 20261108-183842-fsaz: its docs/design files (per-job-landing-branches.md, continuous-landing.md, squash-chain-landing.md) and its major finding (CHANGELOG.md not modified) are addressed by the class fix, not by redoing that design; that candidate's three docs files are excluded in out_of_scope as not kept here. Its minor findings (status.rs path, attribution rule) concern the design note and are dropped with it.
- Job 20261028-000831-rkv5: its candidate's changes to studies/firstmate-work-handling/intro.json and xtask/src/site.rs are not kept or redone here (out_of_scope); its major finding (no CHANGELOG.md entry) is the class this change prevents; its minor/nit findings in xtask tests are dropped as unrelated to the cause.
- Assumption: the rule applies only to change-capability readings whose deliverables include CHANGELOG.md with action modify and whose defaults_applied has id `changelog`; a violation is reported to the producer through the existing admission-violation path, which stops the candidate before the verifier. If a repair round for the producer does not already exist for admission violations, only the existing behavior is used.
- Governance: no change to athena.toml, defaults.toml or workflows is needed; the rule is code.
- The rule refuses more candidates: expected 5% of newly refused readings (candidates that skipped the changelog today fail later at the verifier, so the higher refusal rate is intended).

## Acceptance criteria

1. [content] crates/athena-core/src/admission.rs contains a rule that emits a PolicyViolation naming CHANGELOG.md when the reading's deliverables list it, defaults_applied has id `changelog`, and no change in input.changes has path CHANGELOG.md; its doc comment names the root cause (the default was a rule the producer must remember, with no code comparing the diff to the deliverables).
2. [behavior] Run `cargo test -p athena-core admission`: new tests show a candidate that changes only docs/ files with a change reading carrying the changelog default yields a violation naming CHANGELOG.md; the same candidate plus a CHANGELOG.md change yields none; a non-change reading or a reading without the default yields none. The first test fails when the new rule is removed.
3. [behavior] Run `cargo test -p athena-core` : a test with a docs-only design reading built through ensure_changelog (the path both failed jobs took) is refused by admit before verification, showing the check is for the class and not one job.
4. [behavior] Existing tests that run admit or the run pipeline with a change reading still pass (cargo test) after fixtures are updated where they omitted CHANGELOG.md.
5. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass.
6. [content] CHANGELOG.md has an entry for this change naming the cause and the new admission rule.

## Expected deliverables

- crates/athena-core/src/admission.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Redoing the design docs of job 20261108-183842-fsaz (docs/design/per-job-landing-branches.md, continuous-landing.md, squash-chain-landing.md): dropped, not kept
- Redoing the glossary work of job 20261028-000831-rkv5 (studies/firstmate-work-handling/intro.json, xtask/src/site.rs): dropped, not kept
- Changes to athena.toml, defaults.toml, workflows or prompts
- Editing records/ or owner files
- Changing how ensure_changelog adds the deliverable

## User defaults that apply

- changelog: (not in the defaults model) Here: The change records itself in CHANGELOG.md.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New admission tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix placed at admission, where candidate files are first compared with deliverables, not per job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by an admission rule in code, not prose.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of the rule and its predicate. Run: `git diff -- crates/athena-core/src/admission.rs crates/athena-core/src/reading.rs`. Observes: `admit_without_secrets` pushes `violation("changelog", Some("CHANGELOG.md"), ..)` when `changelog_required` holds. `changelog_required` calls `reading::changelog_unproduced`, which is true only for capability `change`, a CHANGELOG.md Modify deliverable, a `changelog` default, and no changed path equal to CHANGELOG.md or ./CHANGELOG.md. The doc comments of the rule, `changelog_required` and the predicate name the root cause: the default was a rule the producer must remember, and no code compared the diff with the deliverables.
- Criterion 2: New admission tests drive `admit_without_secrets` through a helper `admit_files_with(reading, base_extra, files)`. The existing `admit_files` (admission.rs:1550) calls it with `&reading()` and no extra base files. Tests: (a) change reading with the `changelog` default and a CHANGELOG.md Modify deliverable, candidate only `docs/a.md`: exactly one violation, rule `changelog`, path CHANGELOG.md, detail containing CHANGELOG.md. (b) The same plus a CHANGELOG.md change: none. (c) A study reading with the default, and a change reading without it: none. (d) Base .gitattributes with `/CHANGELOG.md athena-readonly`: no `changelog` violation. (e) A reading with out_of_scope `the changelog` and the default still gets the violation (pins current behavior). (f) The base has no CHANGELOG.md and the candidate adds it (status added): none. Predicate unit tests in reading.rs beside the `ensure_changelog` tests (~2174-2210): true for docs-only; false for CHANGELOG.md, ./CHANGELOG.md and an added CHANGELOG.md; true for docs/CHANGELOG.md; false for a study reading. Run: `cargo test -p athena-core admission`. Observes: (a) passes with one `changelog` violation; (b), (c), (d) and (f) find no `changelog` violation; (e) finds it. With the rule's call removed from `admit_without_secrets`, test (a) fails on its one-violation assertion.
  Fails without the behavior: With the `changelog` push removed from `admit_without_secrets`, test (a) fails on its one-violation assertion; with `changelog_unproduced` returning false, the reading.rs predicate tests fail.
  Test seam: Production entry point `admit_without_secrets(&AdmissionInput)` (admission.rs:687), the function `admit` calls first; `admit` is not called because its secret scan needs gitleaks, and existing tests use the same seam (admission.rs:1550-1587, read). Fixture: a temp git repo and a `Reading` built in the test, injected through `AdmissionInput.reading`, `.changes`, `.candidate` and `.base`. `admit_without_secrets` and `admit_files` exist; the `admit_files_with` helper is added by plan step 3. The double builds only the repository and reading; the rule runs in production code.
- Criterion 3: Admission level (admission.rs): a docs-only design reading (class_of_work `docs`, capability `change`, deliverable docs/design/x.md modify) is passed through `crate::reading::ensure_changelog`, the path both failed jobs took. The test asserts the reading now lists CHANGELOG.md and the `changelog` default, then runs `admit_files_with` with candidate docs/design/x.md alone and asserts a `changelog` violation. Run level (run/tests.rs), three tests with the `Docs` capability given an `id` field set to "change" and a scripted reader returning capability `change` with deliverable README.md: (1) `a_change_job_sent_back_for_the_changelog_adds_it_and_reaches_the_verifier`: pass one edits README.md only and reports CHANGELOG.md in unchanged_planned_files with a reason; the second producer brief contains a `## Sent back` section naming CHANGELOG.md and saying a reason does not excuse it; pass two also edits CHANGELOG.md; the invocations are Reader, Producer, Producer, Verifier. This test would also fail if the producer-loop check used the snapshot's base commit in the user's repository, so it pins finding 8. (2) `a_change_job_that_never_adds_the_changelog_is_refused_before_the_verifier`: the producer omits it on every pass; there are three producer invocations (shared slot plus the extra slot); no Verifier invocation; the outcome is Failed with a reason containing `admission refused the change` and `CHANGELOG.md`; the candidate holds a `changelog` violation; `qualifies_for_tree_retry` is false for it (pins the restart-from-base consequence). (3) `a_change_job_that_edits_the_changelog_at_once_reaches_the_verifier`: one producer invocation, then a Verifier invocation. Unit tests for `append_missing_changelog` in run_pin_tests.rs (flag false leaves the text unchanged; the section names CHANGELOG.md). Run: `cargo test -p athena-core`. Observes: Admission test: violation naming CHANGELOG.md for a reading built by `ensure_changelog`. Run level: (1) records a second producer brief naming CHANGELOG.md and then a Verifier; (2) ends Failed at admission with no Verifier; (3) reaches the verifier with one producer run.
  Fails without the behavior: Without the admission rule, (2) reaches the verifier and the admission test finds no violation. Without the producer-loop check, (1) has no second producer pass and (2) has one producer invocation instead of three. Without the `id` field on the test capability, the scripted `change` reading finds no matching capability.
  Test seam: Entry points `Athena::ask` and `Athena::go` (as `finished_job`, run/tests.rs:1829, uses them), reaching `producer_loop` (run/producer.rs:265) and `candidate_stage` and `admit` (candidate.rs:154). Fixture: `athena_with(..)` (run/tests.rs:187) with a scripted `FakeAgentDriver` and `athena.capabilities = vec![Box::new(Docs { id: "change", .. })]`, the replacement pattern at run/tests.rs:5846. The script wraps `base_script` (tests.rs:219) and changes only the reader capability and the producer's edits; it adds no rule logic. The `Docs` struct (tests.rs:90, `id()` returns "study" at line 103) gets an `id: &'static str` field in step 7; the literals at tests.rs:201 and 5846 pass "study". The rest of the seam exists; the admission-level test uses the `admit_files_with` seam added in step 3.
- Criterion 4: Run the whole workspace suite after the double updates, in order first `cargo test -p athena-agent`, then `cargo test -p athena-core` (including `--test refusal_corpus` unchanged as a regression check), then `cargo test -p athena-cli` (including `--test binary`, the nested fake run at binary.rs:138-238), then `cargo test --locked`. The fake change producer (fake.rs) edits CHANGELOG.md when its `## Expected deliverables` lists it. The CLI `Scenario` (common/mod.rs) does the same, built from the full edit list. Tests that pin exact candidate paths for a job that reaches admission get new expected lists (lifecycle.rs near 2395 and 3060, retry.rs:787 and 832; drain.rs:5847 only if it fails). Tests whose job ends before admission need no change. The rule refuses candidates by path and adds no reading-text input form, so no refusal_corpus case is added. Run: `cargo test --locked`. Observes: Every existing test passes, including the athena-cli lifecycle, retry, drain, job_plan and introspection tests, the binary.rs nested fake run, the run_pin_tests `unchanged_planned_files` tests (unchanged) and the refusal corpus.
  Fails without the behavior: If the doubles were not updated, every change-reading test whose producer edits only README.md or FAKE-CHANGE.md would fail with `admission refused the change: CHANGELOG.md ...` or be sent back by the producer loop.
  Test seam: Entry points `Athena::ask` and `Athena::go`, driven by the CLI `Scenario::script` (common/mod.rs:562-620, exists; step 9 changes it) and the `FakeAgentDriver` producer (fake.rs:298-336, exists; step 8 changes it). The doubles only produce the edit; the rule under test stays in admission.rs and producer.rs.
- Criterion 5: Run the three Athena checks in order over the whole workspace: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. They exercise the changed production path (`admit_without_secrets`, `producer_loop`) through the run and CLI tests. Run: `cargo fmt --all --check`. Observes: fmt exits 0; clippy exits 0 with no warnings; test exits 0 with every test passing, including the new admission, reading, run and send_back tests.
  Fails without the behavior: Without the double updates, `cargo test --locked` fails with `admission refused the change` on change-reading tests; an unused helper or unformatted code fails clippy or fmt.
  Test seam: The cargo checks themselves, which call every production path above through `Athena::ask` and `Athena::go` (run/tests.rs and CLI tests). Fixtures: the temp git repo and `Reading` in admission tests, `athena_with(..)` with a scripted `FakeAgentDriver` (run/tests.rs:187), the `Scenario` script (common/mod.rs:562) and the fake producer (fake.rs:298). All exist; the changes are plan steps 3, 7, 8 and 9.
- Criterion 6: Read the diff of CHANGELOG.md. Run: `git diff -- CHANGELOG.md`. Observes: A new first item under `## [Unreleased]` / `### Changed` names the cause (the default was a rule the producer must remember and no code compared the diff with the deliverables; jobs 20261108-183842-fsaz and 20261028-000831-rkv5). It names the `changelog` admission rule, `changelog_unproduced`, the producer-loop send-back, the Governed-only skip, and the updated test doubles. It states that the higher refusal rate is intended (about 5% of change jobs, an estimate), that a repository without CHANGELOG.md must now have every change job create it, that a job refused at admission restarts from the base on retry, and that a resumed pre-upgrade candidate without a changelog entry is refused. The existing tests that `include_str!` CHANGELOG.md still pass.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f7cbf74c7183 → candidate 35923ac70b15. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  18 ++++
 crates/athena-agent/src/fake.rs         |  55 +++++++++-
 crates/athena-cli/tests/common/mod.rs   |  35 ++++++-
 crates/athena-cli/tests/lifecycle.rs    |  37 ++++++-
 crates/athena-cli/tests/retry.rs        |   4 +-
 crates/athena-core/src/admission.rs     | 174 +++++++++++++++++++++++++++++++-
 crates/athena-core/src/reading.rs       |  59 +++++++++++
 crates/athena-core/src/run/mod.rs       |  11 +-
 crates/athena-core/src/run/producer.rs  |  49 ++++++++-
 crates/athena-core/src/run/send_back.rs |  42 ++++++++
 crates/athena-core/src/run/tests.rs     | 154 +++++++++++++++++++++++++++-
 11 files changed, 611 insertions(+), 27 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-161505-bp5e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-161505-bp5e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42669141175 for 35923ac70b155b69efe71104e05aaffeebc46a4d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42669141175 for 35923ac70b155b69efe71104e05aaffeebc46a4d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42669141175 for 35923ac70b155b69efe71104e05aaffeebc46a4d)
- ci-result: `not a check` passed (Check results for 35923ac70b155b69efe71104e05aaffeebc46a4d come from the CI run https://github.com/doodla/athena/actions/runs/42669141175 (525s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets and cargo test pass.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core admission`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
2218 |         assert!(changelog_unproduced(&reading, ["docs/a.md"]));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2219:17
     |
2219 |         assert!(changelog_unproduced(&reading, []));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2221:18
     |
2221 |         assert!(!changelog_unproduced(
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2225:18
     |
2225 |         assert!(!changelog_unproduced(&reading, ["./CHANGELOG.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2227:17
     |
2227 |         assert!(changelog_unproduced(&reading, ["docs/CHANGELOG.md"]));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2231:18
     |
2231 |         assert!(!changelog_unproduced(&study, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2236:18
     |
2236 |         assert!(!changelog_unproduced(&plain, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2245:18
     |
2245 |         assert!(!changelog_unproduced(&created, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `append_missing_changelog` in this scope
   --> crates/athena-core/src/run/send_back.rs:320:20
    |
320 |         assert_eq!(append_missing_changelog(text.clone(), false), text);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `append_missing_changelog` in this scope
   --> crates/athena-core/src/run/send_back.rs:326:19
    |
326 |         let out = append_missing_changelog(text.clone(), true);
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 10 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
     |
2218 |         assert!(changelog_unproduced(&reading, ["docs/a.md"]));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2219:17
     |
2219 |         assert!(changelog_unproduced(&reading, []));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2221:18
     |
2221 |         assert!(!changelog_unproduced(
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2225:18
     |
2225 |         assert!(!changelog_unproduced(&reading, ["./CHANGELOG.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2227:17
     |
2227 |         assert!(changelog_unproduced(&reading, ["docs/CHANGELOG.md"]));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2231:18
     |
2231 |         assert!(!changelog_unproduced(&study, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2236:18
     |
2236 |         assert!(!changelog_unproduced(&plain, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2245:18
     |
2245 |         assert!(!changelog_unproduced(&created, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `append_missing_changelog` in this scope
   --> crates/athena-core/src/run/send_back.rs:320:20
    |
320 |         assert_eq!(append_missing_changelog(text.clone(), false), text);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `append_missing_changelog` in this scope
   --> crates/athena-core/src/run/send_back.rs:326:19
    |
326 |         let out = append_missing_changelog(text.clone(), true);
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 10 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2219:17
     |
2219 |         assert!(changelog_unproduced(&reading, []));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2221:18
     |
2221 |         assert!(!changelog_unproduced(
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2225:18
     |
2225 |         assert!(!changelog_unproduced(&reading, ["./CHANGELOG.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2227:17
     |
2227 |         assert!(changelog_unproduced(&reading, ["docs/CHANGELOG.md"]));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2231:18
     |
2231 |         assert!(!changelog_unproduced(&study, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2236:18
     |
2236 |         assert!(!changelog_unproduced(&plain, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `changelog_unproduced` in this scope
    --> crates/athena-core/src/reading.rs:2245:18
     |
2245 |         assert!(!changelog_unproduced(&created, ["docs/a.md"]));
     |                  ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `append_missing_changelog` in this scope
   --> crates/athena-core/src/run/send_back.rs:320:20
    |
320 |         assert_eq!(append_missing_changelog(text.clone(), false), text);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `append_missing_changelog` in this scope
   --> crates/athena-core/src/run/send_back.rs:326:19
    |
326 |         let out = append_missing_changelog(text.clone(), true);
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 10 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
