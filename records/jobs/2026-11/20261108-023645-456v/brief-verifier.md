# Job 20261108-023645-456v: verify "Merge duplicate failure classes instead of only adding new ones"

## Request (verbatim)

> Lessons become checks, not knowledge. They created 158 separate failure classes that are never merged.

## Reading (accepted by the user)

Sri's note (BRIEF.md, 2026-11-08) says the introspection step only ever adds failure classes (158 so far, none merged). I read it as: make the class registry merge-capable and stop code adding near-duplicates. The audience is Sri and the introspection and planner prompts that read the class list.

Assumptions:
- Code survey: crates/athena-core/src/introspect.rs (entry: the introspection answer handler around lines 1570-1620 where `failure_class == "new"` mints a FailureClass with ClassStatus::Proposed and a fresh RecordId; the registry is read at lines 424-434 from records/failure-classes; Jev class matching runs in shadow via ask_jev, then code narrows by class_of_work and category), crates/athena-data/src/records/introspection.rs (ClassStatus has only `Proposed`; FailureClass is deny_unknown_fields), crates/athena-core/src/labeled.rs and hestia.rs (they read classes, so merged classes must resolve to their target when counting recurrence). The records validation allow-list already accepts athena.failure-class/1 through AnyRecord; any new field or status variant must round-trip in the existing records round-trip test and check_record_file. Class count and recurrence are measured directly on a fixture registry in tests, not through a stand-in.
- Existing code: docs/design/defaults-loop.md section 3.4 already says the user may rename or merge classes, but no code does; this job builds that merge and extends the introspect.rs registry path.
- The real 158 classes live under records/, which this job cannot read or edit. So the job delivers the mechanism (a merge status, resolution of merged ids, duplicate prevention at creation) and does not rewrite existing records.
- Term ladder: not used. Merge means a class record marked merged into a target class id; counts and lookups then treat it as its target.
- Assumed mechanism: add `ClassStatus::Merged` with an optional `merged_into` id, a code path that refuses to create a new class whose normalised name equals an existing live class (reusing it instead), and resolution of merged ids in registry reads, earlier-case selection and recurrence counts. A CLI command for a user-driven merge is optional and only added if small.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [behavior] `cargo test -p athena-core introspect` includes a test where a new-class answer whose normalised name equals an existing live class reuses that class id and adds no registry record.
2. [behavior] A test merges class B into class A. Registry reads and earlier-case selection then count B's introspections under A, and the live class count drops by one. Run with `cargo test -p athena-core`.
3. [behavior] A FailureClass with status merged and merged_into round-trips through the records round-trip test and passes check_record_file. An old `proposed` record still deserializes. Run with `cargo test -p athena-data`.
4. [behavior] `cargo test` passes workspace-wide, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
5. [content] docs/design/defaults-loop.md section 3.4 and CHANGELOG.md describe the merge behavior. They say the existing 158 classes are not rewritten by this change and mark any unbuilt part as design.
6. [content] A decision entry in docs/decisions.md records the call: dedupe at creation plus merge status, with the alternatives of periodic model clustering and a manual-only merge, and why.

## Expected deliverables

- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/labeled.rs (modify)
- crates/athena-data/src/records.rs (modify)
- docs/design/defaults-loop.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/introspection.rs (modify)

## Out of scope

- Rewriting or merging the existing 158 records under records/
- Clustering existing classes with a model
- Changing the lessons-into-defaults or skills work in the other brief lines
- Prompt file edits (token headroom is near zero)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require passing cargo test, clippy and fmt.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at class creation, which is where the duplicates come from, plus a merge path.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show reuse and merge happening.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Dedupe is enforced in code rather than by prompt.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the decision entry are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Scoped to one mechanism and leaves the data migration out.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two tests. (a) Unit test in introspect.rs `a_new_class_answer_naming_a_live_class_reuses_it`: write one live class with `put_class`, read it with `read_records(dir)`, then call `Athena::apply_answer` with `boundary_answer("reviewer")` whose new-class name differs from the class only in case and punctuation ("the REVIEWER is not told!"). Further cases: a name equal to a retired class's name returns the live target; a name empty after normalising does not match and still creates a class; a name that differs in a word creates a new class (control). (b) End-to-end test in crates/athena-cli/tests/introspection.rs `a_new_class_named_like_a_live_class_reuses_it`: two `intervene` events whose scripted model replies `reply("new", ...)` with the same class name in different case. Run: `cargo test -p athena-core introspect && cargo test -p athena-cli --test introspection a_new_class`. Observes: (a) apply_answer returns None (no class to write), the record's failure_class equals the existing id, new_class is false, status is Recorded; the retired-name case returns the live target; the different-wording and empty-name cases return Some(FailureClass) (so the rule is exact equality only). (b) After the second event `Records::load(...).classes.len() == 1`, both introspections carry the same failure_class, only the first has new_class, and the diff between the two heads has no `records/failure-classes/` path.
  Fails without the behavior: Without the dedupe, apply_answer returns Some(FailureClass) with a fresh id and new_class true, and the end-to-end registry holds 2 records, so both tests fail.
  Test seam: Production entry points `Athena::apply_answer` (private; called from in-module tests the way the existing `an_introspection_with_a_boundary_word_is_written_without_it` does) and `Athena::intervene`. Fixtures `athena_with(None)`, `boundary_answer`, `put_class`, `read_records(dir)` in introspect.rs tests, and the scripted `Model` plus `Fixture` in the cli test: all exist today (read at introspect.rs ~1826-2138 and introspection.rs 35-170). The matching is the production `class_named` helper that step 3 adds, called inside apply_answer; no double holds it.
- Criterion 2: Tests with `merge` in the name. (a) introspect.rs `a_merged_class_is_listed_once_and_its_cases_count_under_the_target`: put classes A and B and a case in each; assert 2 live classes; build the merge record with the production `FailureClass::merged_into_class` and `put_record` it; assert `read_records` has 1 live class, every CaseRow resolves to A, and an `apply_answer` that names A and lists B's case in earlier_cases (also in passed_cases) keeps it with no dropped id; the same answer naming B resolves to A; control: before the merge record exists, the same answer drops B's case. (b) hestia.rs: introspections in B and A from different jobs share `cause_key` `class:<A>` and give one `repeated_causes` group of 2 jobs once the merge record is in the store Records. (c) labeled.rs: `read_inputs` on a tempdir with the merge record maps B's label and a Jev choice B to A with probabilities summed; `build(...)` rows target A. (d) cli test `merging_a_class_counts_its_cases_under_the_target`: two classes made by the scripted model, `athena.merge_classes(b, a)`, then a third event whose prompt `class_registry` has one entry and whose answer lists both earlier cases, kept with dropped_ids empty, and `check_history` clean. (e) `merge_classes` refuses self-merge, unknown id, retired source or target, merge-record id. (f) review: cli review test `review_refuses_a_merge_record_and_a_retired_class_and_a_renamed_duplicate`: `--class <merge-record-id>` and `--class <retired id>` are refused naming the live target (control: the same `--class` on a live class succeeds), and `--new-class "Stale cache!"` is refused with a pointer to the live class 'stale cache' while a differently worded name succeeds. (g) ClassMerges unit tests: chain B->C->A, cycle ends, earliest merge wins, no-merge fast path. (h) briefs.rs `class_summaries`, evidence_labels `recurrence_vote` and proposals `generate` each get a merged-class case. Run: `cargo test -p athena-core && cargo test -p athena-context && cargo test -p athena-cli --test introspection --test review`. Observes: Live class count falls from 2 to 1 (store record count is 3). B's cases count under A in earlier-case selection, recurrence grouping, labeled rows, class summaries, the recurrence label and proposals. merge_classes and review refuse the cases listed.
  Fails without the behavior: Without resolution the registry still lists B (2 classes), B's case is dropped as not in the class, the two introspections get different cause keys (no repeated cause), labeled rows keep label B, review accepts a merge record's id as a class, and 'Stale cache!' passes review.
  Test seam: Production entry points `read_records`, `Athena::apply_answer`, `Athena::merge_classes`, `Athena::review`, `hestia::cause_key` / `repeated_causes` (through `class_of`), `labeled::read_inputs` and `build`, `briefs::class_summaries`, `evidence_labels::recurrence_vote`, `proposals::generate`. Fixtures are tempdir record files via `put_class` / `put_case` / `put_record` (exist in introspect.rs tests), `put` / `class(n,name)` helpers in hestia.rs tests (exist near 2976-3050), the labeled.rs tempdir test near 1372 (exists), the briefs.rs class_summaries tests near 2994 and 3048 (exist), and `Fixture` with `class_record` in the cli review test (exists, tests/review.rs 47) and the scripted model in the cli introspection test (exists). The merge record is built by the production `FailureClass::merged_into_class` that step 1 adds; no double holds merge logic.
- Criterion 3: New tests in records.rs: `a_merged_failure_class_has_a_path_a_schema_and_round_trips` builds a Merged FailureClass, serialises with `to_json`, parses with `AnyRecord::parse`, runs `check_record_file(&canonical_path, bytes)`. `an_old_proposed_failure_class_still_parses` parses the literal JSON an old record has (`"status": "proposed"`) through `check_record_file`. `merged_failure_class_validation`: merged_into equal to merged_from, and an empty name, are refused as RecordError::Invalid. ClassMerges tests sit beside them. Run: `cargo test -p athena-data`. Observes: The merged record equals itself after the round trip and is accepted at its canonical path; the proposed record parses unchanged; the invalid ones return Invalid.
  Fails without the behavior: Without the Merged variant the merged JSON fails to parse (unknown variant); without the validate arm the self-merge record is accepted.
  Test seam: Production entry points `AnyRecord::to_json`, `AnyRecord::parse`, `check_record_file` (exist, records.rs ~1366). Fixtures: a struct literal and an inline JSON string. The validate arm is added in records.rs by step 2.
- Criterion 4: The three checks Athena runs, whole workspace, plus the refusal-corpus test as a regression. The rule added is a class-name match on model output and a review name check, not a parser, stem list, stop list or reading check, so no refusal_corpus.rs case maps to it and I cite none. The corpus run guards `cause_key`, which feeds generated root-cause requests (refusal_corpus.rs ~686-777 use introspections with failure_class None). Input forms of the new rule and the test that covers each: case difference, punctuation, extra spacing, boundary-reworded name, retired class's name, empty-after-normalising name, different wording (control) in the criterion 1 tests; `--new-class` with punctuation and case difference, merge-record id and retired id for `--class` in the criterion 2 review test; two existing live classes with equal normalised names pick the earliest by (at, id) in a `class_named` unit test. Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check && cargo test -p athena-core --test refusal_corpus`. Observes: All pass.
  Fails without the behavior: A missed consumer fails the build: the ClassStatus change removes Copy and adds a variant, so any non-exhaustive match or copy use fails to compile; an unformatted file or a clippy warning fails the other commands.
  Test seam: cargo itself. Existing `FailureClass { status: ClassStatus::Proposed, .. }` literals compile unchanged because no field is added.
- Criterion 5: Read the diff. defaults-loop.md section 3.4 gets a paragraph: creation (introspection and review) reuses a live or retired class with the same normalised name; a merge is an appended failure-class record with status merged naming the retired and surviving class; readers count a merged class under its target; `Athena::merge_classes` is built; creation-time reuse catches only names equal after normalising, so differently worded duplicates still need a merge record; the existing 158 classes are not rewritten by this change; a CLI merge command, a backtest counting collisions over the records branch, model-based clustering and the migration of existing classes are design (not built); a merge record's `proposed_by` is the retired class's and does not mean an introspection proposed the merge. The CHANGELOG `[Unreleased]` `### Added` entry says the same. The policy test scans the new text for banned words and personal data. Run: `git diff -- docs/design/defaults-loop.md CHANGELOG.md && cargo test -p xtask --test policy`. Observes: The statements (merge behavior, narrow reach, 158 classes not rewritten, unbuilt parts marked design) are present and the scan passes.
- Criterion 6: Read the diff for `## D166 · ...`, appended at the end of docs/decisions.md, one greater than D165 (the last entry, near line 4697). It has Case, Decision, Alternatives (periodic model clustering; manual-only merge; edit the record in place; description similarity; prompt instruction), Tradeoff (names the narrow reach and the same-moment duplicate), Reversal cost and Provenance, written like D165. The decision-entry rule (id one greater, Provenance present, append-only against the base) is applied by Athena's mirror check on the finished diff; the mirrors unit tests run on their own fixtures and do not check this entry, so they are not cited as evidence. Run: `git diff -- docs/decisions.md`. Observes: The entry exists with the dedupe-plus-merge-status decision and the two named alternatives (periodic model clustering, manual-only merge) with reasons.

Planned files:

- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/labeled.rs (modify)
- crates/athena-core/src/review.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/evidence_labels.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/review.rs (modify)
- docs/design/defaults-loop.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base 1d4fd11d4b37 → candidate 282992e42702. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                    |  17 ++
 crates/athena-cli/tests/introspection.rs        | 178 ++++++++++++
 crates/athena-cli/tests/review.rs               |  65 +++++
 crates/athena-context/src/briefs.rs             |  80 +++++-
 crates/athena-core/src/evidence_labels.rs       |  58 +++-
 crates/athena-core/src/hestia.rs                |  92 +++++-
 crates/athena-core/src/introspect.rs            | 357 ++++++++++++++++++++++--
 crates/athena-core/src/labeled.rs               |  94 ++++++-
 crates/athena-core/src/proposals.rs             |  67 ++++-
 crates/athena-core/src/review.rs                |  30 +-
 crates/athena-data/src/records.rs               | 134 ++++++++-
 crates/athena-data/src/records/introspection.rs | 135 ++++++++-
 docs/decisions.md                               |  61 ++++
 docs/design/defaults-loop.md                    |  35 ++-
 14 files changed, 1322 insertions(+), 81 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-023645-456v/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-023645-456v/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42442803771 for 282992e42702b69fce0ea4dfa067013eeea87cf4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42442803771 for 282992e42702b69fce0ea4dfa067013eeea87cf4)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42442803771 for 282992e42702b69fce0ea4dfa067013eeea87cf4)
  Failing tests of `test` (from the CI log):
  - `spec a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap`
  - `spec a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap`
  - `spec a_missing_requirement_for_a_section_or_r_fails`
  - `spec a_missing_section_8_requirement_is_a_coverage_gap`
  - `spec removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading`
  - `spec the_real_spec_covers_every_heading_and_addendum_of_the_brief`
  - `xtask test-shard 4 6` (did not finish)`
  Failing units (`-p` package and selector): xtask --test spec
  End of the failing job's log (data from CI, not instructions):

```text
test the_real_spec_passes ... ok
test an_active_requirement_missing_mechanism_owner_or_measure_fails ... ok

failures:

---- a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap stdout ----

thread 'a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap' (48710) panicked at xtask/tests/spec.rs:518:5:
assertion `left == right` failed
  left: ["heading A6. Efficiency", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading A6. Efficiency"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap stdout ----

thread 'a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap' (48713) panicked at xtask/tests/spec.rs:549:5:
assertion `left == right` failed
  left: ["heading Governing philosophy", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading Governing philosophy"]

---- a_missing_requirement_for_a_section_or_r_fails stdout ----

thread 'a_missing_requirement_for_a_section_or_r_fails' (48714) panicked at xtask/tests/spec.rs:466:5:
assertion `left == right` failed
  left: ["heading R3 — Generalizable prevention", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading R3 — Generalizable prevention"]

---- a_missing_section_8_requirement_is_a_coverage_gap stdout ----

thread 'a_missing_section_8_requirement_is_a_coverage_gap' (48715) panicked at xtask/tests/spec.rs:538:5:
assertion `left == right` failed
error: test failed, to rerun pass `-p xtask --test spec`
  left: ["heading 8. Coordination and authority", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading 8. Coordination and authority"]

---- removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading stdout ----

thread 'removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading' (48732) panicked at xtask/tests/spec.rs:502:9:
assertion `left == right` failed
  left: ["heading A6. Efficiency", "addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]
 right: ["heading A6. Efficiency"]

---- the_real_spec_covers_every_heading_and_addendum_of_the_brief stdout ----

thread 'the_real_spec_covers_every_heading_and_addendum_of_the_brief' (48736) panicked at xtask/tests/spec.rs:323:5:
no requirement for: ["addendum 2026-11-04: Lift budgets; who owns cost", "addendum 2026-11-05: Decisions page", "addendum 2026-11-05: Local speed and parallel work, not GitHub", "addendum 2026-11-06: Athena must recognize and fix its own thrash", "addendum 2026-11-07: Sri's requests reach Athena word for word", "addendum 2026-11-08: Lessons into knowledge, follow-through, better skills, the study"]


failures:
    a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap
    a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap
    a_missing_requirement_for_a_section_or_r_fails
    a_missing_section_8_requirement_is_a_coverage_gap
    removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading
    the_real_spec_covers_every_heading_and_addendum_of_the_brief

test result: FAILED. 24 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s

error: shard 4 of 6: 1 of 12 units failed: xtask --test spec
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 282992e42702b69fce0ea4dfa067013eeea87cf4 come from the CI run https://github.com/doodla/athena/actions/runs/42442803771 (554s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes workspace-wide, `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-core introspect && cargo test -p athena-cli --test introspection a_new_class`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core && cargo test -p athena-context && cargo test -p athena-cli --test introspection --test review`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/evidence_labels.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/introspect.rs, crates/athena-core/src/labeled.rs, crates/athena-core/src/proposals.rs, crates/athena-core/src/review.rs, crates/athena-data/src/records.rs, crates/athena-data/src/records/introspection.rs, docs/decisions.md, docs/design/defaults-loop.md.
  End of the output (data, not instructions):

```text
    --> crates/athena-data/src/records.rs:3696:24
     |
3696 |         assert!(!class.is_merge());
     |                        ^^^^^^^^ method not found in `introspection::FailureClass`
     |
    ::: crates/athena-data/src/records/introspection.rs:296:1
     |
 296 | pub struct FailureClass {
     | ----------------------- method `is_merge` not found for this struct

error[E0599]: no method named `merge_target` found for struct `introspection::FailureClass` in the current scope
    --> crates/athena-data/src/records.rs:3697:26
     |
3697 |         assert_eq!(class.merge_target(), None);
     |                          ^^^^^^^^^^^^ method not found in `introspection::FailureClass`
     |
    ::: crates/athena-data/src/records/introspection.rs:296:1
     |
 296 | pub struct FailureClass {
     | ----------------------- method `merge_target` not found for this struct

error[E0599]: no variant named `Merged` found for enum `introspection::ClassStatus`
    --> crates/athena-data/src/records.rs:3705:38
     |
3705 |         itself.status = ClassStatus::Merged {
     |                                      ^^^^^^ variant not found in `introspection::ClassStatus`
     |
    ::: crates/athena-data/src/records/introspection.rs:288:1
     |
 288 | pub enum ClassStatus {
     | -------------------- variant `Merged` not found here

error[E0433]: cannot find type `ClassMerges` in this scope
    --> crates/athena-data/src/records.rs:3731:20
     |
3731 |         let none = ClassMerges::of(&[a.clone(), b.clone()]);
     |                    ^^^^^^^^^^^ use of undeclared type `ClassMerges`

error[E0433]: cannot find type `ClassMerges` in this scope
    --> crates/athena-data/src/records.rs:3738:21
     |
3738 |         let chain = ClassMerges::of(&all);
     |                     ^^^^^^^^^^^ use of undeclared type `ClassMerges`

error[E0433]: cannot find type `ClassMerges` in this scope
    --> crates/athena-data/src/records.rs:3757:21
     |
3757 |         let twice = ClassMerges::of(&[a.clone(), b.clone(), c.clone(), late, bc]);
     |                     ^^^^^^^^^^^ use of undeclared type `ClassMerges`

error[E0433]: cannot find type `ClassMerges` in this scope
    --> crates/athena-data/src/records.rs:3771:22
     |
3771 |         let cyclic = ClassMerges::of(&[a.clone(), b.clone(), ab, ba]);
     |                      ^^^^^^^^^^^ use of undeclared type `ClassMerges`

Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
error: could not compile `athena-data` (lib test) due to 10 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check && cargo test -p athena-core --test refusal_corpus`). Result: not run: not a plain cargo test command.

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
