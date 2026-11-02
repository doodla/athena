# Job 20261031-215030-b0nv: verify "Scan only deliverable paths and content criteria for boundary words"

## Request (verbatim)

> Priority: high
> Stop refusing a reading over a boundary word in its prose; keep the check where the word can reach a file
> 
> The reading check (`check_reading_acceptance` in crates/athena-core/src/reading.rs) refuses any reading whose free text holds a boundary word. Readings are records, and records are outside the boundary scan (xtask/tests/policy.rs skips records/). The risk the check guards against is a word reaching a tracked file, and the writer already gets the word list and runs the scan itself (D73). Meanwhile readings keep being refused for prose alone: one request was refused six times on 2026-10-31, and another twice on 2026-10-30.
> 
> Keep the check on deliverable paths and on acceptance criteria that tell the writer what text to write into a file. Drop it for the summary, assumptions, out-of-scope and other prose fields. Update the module docs and docs/decisions.md with the reason.
> 
> Acceptance:
> - A test shows a reading whose assumptions or summary contain a boundary word is accepted.
> - A test shows a reading whose deliverable path, or an acceptance criterion quoting text to write into a file, contains a boundary word is still refused.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

Narrow `check_boundary_and_repo_files` in reading.rs so boundary words are refused only in deliverable paths and in content-kind acceptance criteria (text to write into a file). Title, summary, assumptions, out_of_scope and other prose fields stop being scanned. Audience: readers, whose readings keep being refused for prose alone, and the maintainers who read the decision log.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: `check_reading_acceptance` (about line 3014) runs `check_boundary_and_repo_files` last (about line 2771), which scans every field from `reading_text_fields` (about line 2709) and each deliverable path with `athena_context::briefs::boundary_scan`. The repository-file checks (create/modify/delete against the base commit) stay unchanged. This is an in-memory text check, so the existing unit tests in reading.rs measure it directly; it adds no record or data file, so `check_record_file`, `AnyRecord` and the round-trip test are not involved.
- Assumed meaning of 'acceptance criteria that tell the writer what text to write into a file': criteria with kind `content` are scanned, and criteria with kind `behavior` are not, since content criteria describe file text and behavior criteria name commands to run.
- The title, summary, decline_reason, question, departures, hard_reason, urgent_reason, assumptions, out_of_scope and defaults_applied are no longer scanned. The request-text scan (around line 2257, on the user's own request) is a different check and stays.
- Existing tests in reading.rs that expect refusal for summary, assumptions and similar fields (e.g. `a_reading_whose_summary_holds_a_boundary_word_is_refused_and_the_word_is_named`, `a_boundary_word_in_any_text_field_or_path_is_refused_with_the_field_named`) are rewritten to match the new scope. Banned words in tests are still built from pieces so the file passes the boundary scan.
- The latest decision in docs/decisions.md is D152, so the new entry is D153 and cites D73. The CHANGELOG.md entry goes under the unreleased heading.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/reading.rs builds a reading with a boundary word in its summary and in an assumption, and `check_reading_acceptance` returns Ok; `cargo test -p athena-core` passes.
2. [behavior] A unit test shows a reading whose deliverable path holds a boundary word is still refused with the path and word named, and a reading with a `content` acceptance criterion holding a boundary word is still refused naming `acceptance_criteria`; `cargo test -p athena-core` passes.
3. [behavior] A unit test shows a `behavior` criterion, out_of_scope, title and defaults_applied text holding a boundary word are accepted; `cargo test -p athena-core` passes.
4. [behavior] The change narrows a refusal rule, so it refuses fewer readings and none newly: expected 0% of newly refused readings.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] The doc comments of `check_boundary_and_repo_files` and the module docs in crates/athena-core/src/reading.rs state which fields are scanned, which are not, and why: readings are records outside the boundary scan, and the writer already gets the word list (D73).
7. [content] docs/decisions.md has a new D153 entry giving the reason, the alternatives considered and the cited refusals (six on 2026-10-31, two on 2026-10-30), and CHANGELOG.md has an entry for the change.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the boundary word list or the boundary scan in athena_context::briefs.
- Changing the request-text scan that refuses a user's request holding a boundary word.
- Changing the repository-file checks for deliverable actions.
- Changing xtask/tests/policy.rs, athena.toml, defaults.toml or any prompt file.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New and rewritten unit tests; fmt, clippy -D warnings and the test suite must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests show the outcome: prose with a boundary word is accepted and a file-bound word is still refused.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests that exercise acceptance and refusal.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Module docs and the decision entry state the reason for an outside reader.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is in Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to one check.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule is narrowed to where a word can reach a file, not special-cased per refused request.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test calling `check_reading_acceptance` on a reading with a boundary word in the summary and in an assumption. It runs with the repo-file list `None`, with `Some(&files_at_head())`, and through `check_reading_text`. Run: `cargo test -p athena-core --lib reading::tests::a_boundary_word_in_prose_fields_is_accepted`. Observes: `check_reading_acceptance` returns Ok(()) and `check_reading_text` returns `ReadingCheck::Accept` for a reading whose summary and assumption hold the word.
  Fails without the behavior: With the old all-fields scan, `check_reading_acceptance` returns Err naming the summary or assumption and the test's unwrap panics.
  Test seam: Entry point `check_reading_acceptance` (pub fn in reading.rs, about line 3014); guards: capability set, no question, governance coverage and criteria-evidence checks pass for the `governed_reading` fixture. Fixtures `governed_reading`, `banned_word`, `files_at_head` exist in the tests module. No fake. The test is added in plan step 3.
- Criterion 2: Unit test calling `check_reading_acceptance` on two readings per word (the word and `banned_fragment()`): a deliverable path `docs/<word>.md`, and a `governed_reading` whose content criterion 0 is 'README.md says: the <word> decides'. It asserts the error names the word and the path (path case) or `acceptance_criteria[0]` (criterion case), and holds 'deliverable path or content acceptance criterion'. A companion end-to-end test runs `Athena::ask` with a fake reader that puts the word in `acceptance_criteria[0].text`. Run: `cargo test -p athena-core --lib reading::tests::a_boundary_word_in_a_path_or_content_criterion_is_still_refused && cargo test -p athena-core --lib run_pin_tests::a_reading_with_a_boundary_word_is_not_accepted_and_the_word_reaches_the_caller`. Observes: Err naming the word and the path for a deliverable path; Err naming `acceptance_criteria[0]` and the word for a content criterion. End to end: no reading is accepted, the error contains 'was refused', the word and 'acceptance_criteria', the reader is briefed twice and the second brief's refusal section holds the word.
  Fails without the behavior: If the narrowing also dropped the path or content scan, `check_reading_acceptance` returns Ok and `unwrap_err()` panics; with labels or message changed, the contains-assertions fail; in run_pin_tests the reading would be accepted and `proposal.reading.is_none()` fails.
  Test seam: Entry points `check_reading_acceptance` (exists) and `Athena::ask` through `athena_with`, `reader_edits` and `config(true)` in run_pin_tests.rs (exist; the fake reader edits the JSON the reader returns, and the checks run for real). Fixtures `governed_reading` (Content kind) and `reading_with_deliverable` exist. Criterion text avoids producer-report phrasing and unknown repo paths so earlier checks pass. Added in plan steps 4 and 8.
- Criterion 3: Table-driven unit test: an `Edit` table over title, out_of_scope[0], defaults_applied[0], departures, hard_reason, urgent_reason and a criterion switched to `CriterionKind::Behavior`; separate readings for question and decline_reason. Each calls `check_reading_acceptance`. It also checks a word embedded in a longer word is accepted. Run: `cargo test -p athena-core --lib reading::tests::boundary_words_outside_paths_and_content_criteria_are_not_scanned`. Observes: Ok(()) for every unscanned field, including a behavior criterion, title, out_of_scope and defaults_applied.
  Fails without the behavior: With the old scan each of these fields makes `check_reading_acceptance` return Err and the table's unwrap fails naming the field.
  Test seam: Entry point `check_reading_acceptance`; fixtures as in criterion 1; `Criterion`, `CriterionKind` and `DefaultApplied` are already imported in the tests module. Added in plan step 5.
- Criterion 4: Subset argument plus corpus cases. Input forms of the rule and corpus cases: word in a deliverable path -> new `boundary-word-in-a-deliverable-path` (Refuse `check_boundary_and_repo_files`); word in a content criterion -> new `boundary-word-in-a-content-criterion` (Refuse the same check); word in the summary -> new `boundary-word-in-the-summary` (Accept); word in an assumption -> new `boundary-word-in-an-assumption` (Accept); crate-relative deliverable path (repo-file branch, unchanged) -> existing `crate-relative-deliverable-path`; full deliverable path -> existing `full-deliverable-path`; governance, absent-file and brand-word forms -> existing `request-names-a-governance-file-the-reading-ignores`, `request-names-a-governance-file-the-reading-excludes`, `text-relies-on-an-absent-file`, `text-says-the-file-does-not-exist-yet`, `brand-word-and-placeholder-are-not-repo-paths`. Other forms (bullets, numbered parts, job citations, retry complaints, producer-report evidence, code survey, design) do not touch this rule and keep their cases (for example `steward-three-dash-bullets`, `retry-complaint-reading-omits-the-job`, `criterion-rests-on-the-producer-report`). Subset argument: the scanned set after the change (paths plus content criteria) is contained in the old set (all fields plus paths) and the problem list gets no new entries, so no reading is newly refused. I did not run the corpus and cite no result; the 0% is by construction, not a measured rate over past readings. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: The whole corpus passes: no Accept case becomes a refusal, `crate-relative-deliverable-path` is still refused by `check_boundary_and_repo_files`, the two new Accept cases are accepted, the two new Refuse cases are refused by that check, and `corpus_covers_the_listed_input_forms` passes with 17 forms.
  Fails without the behavior: The two new Accept cases (word in summary, word in assumption) would be refused by `check_boundary_and_repo_files` under the old scan and the corpus test would fail. The existing Accept cases passing is a no-regression check only.
  Test seam: The corpus harness in crates/athena-core/tests/refusal_corpus.rs calls `refusing_check` and `request_text_acceptance` on built `Reading` values; `case(...)` builds from `base_reading` and one field is changed. The harness exists today; plan step 9 adds the four cases and the form tag, with source 'request of job 20261031-215030-b0nv citing refusals on 2026-10-31 and 2026-10-30; rebuilt from the request'.
- Criterion 5: Run the three Athena checks. No unused helper, import or stale test is left (`reading_text_fields` replaced; three reading.rs tests and the run_pin_tests.rs test rewritten). The policy test also runs in the full suite. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0; `xtask/tests/policy.rs` finds no tracked file holding a listed word.
  Fails without the behavior: A formatting slip, a dead `reading_text_fields` or import, a test still expecting the old refusal (the run_pin_tests.rs one) or a literal banned word in a test fails one of the commands.
  Test seam: Whole-workspace commands; no seam.
- Criterion 6: Read the module `//!` docs at the top of reading.rs and the doc comments of `check_boundary_and_repo_files` and `file_bound_text_fields`. Check each lists the scanned places (deliverable paths, `content` criteria) and the unscanned ones (title, summary, assumptions, out_of_scope, defaults_applied, reasons, question, departures, `behavior` criteria), and gives the reason: readings are records outside the policy scan and the writer gets the word list and runs the scan (D73). Check they state the criterion-kind edge cases and the commit-text gap (title, summary and criterion texts reach landing commit messages and the pull request title, which no scan covers), and make no claim that a backstop covers that path. Run: `cargo test -p athena-core --lib reading && cargo test -p xtask --test policy`. Observes: The two tests pass, so the docs carry no listed word. By Grep for 'D73' in crates/athena-core/src/reading.rs and a Read of the top of the file and the doc comment above `check_boundary_and_repo_files`, the docs name the scanned and unscanned fields, cite D73 and records being outside the boundary scan, and state the edge cases and the gap.
- Criterion 7: Read the new D153 entry after D152 in docs/decisions.md and the new CHANGELOG.md bullet. D153 uses D152's fields and cites the six refusals on 2026-10-31 and two on 2026-10-30 (stated by the request, not measured), D73, the alternatives and a Tradeoff with the criterion-kind edge cases and the commit-text gap, with the proposed follow-up. The CHANGELOG bullet under Unreleased / Changed names D153 and the root cause. Run: `cargo test -p xtask --test policy`. Observes: The policy test passes, so D153 and the CHANGELOG bullet hold no listed word. By Grep for '^## D153' in docs/decisions.md and 'D153' in CHANGELOG.md, D153 exists with the reason, alternatives, cited refusal counts and the gap, and the CHANGELOG bullet names D153.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base c9f546d567e8 → candidate 11e5938d81c5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   9 ++
 crates/athena-core/src/reading.rs          | 223 +++++++++++++++++------------
 crates/athena-core/src/run_pin_tests.rs    |  29 +++-
 crates/athena-core/tests/refusal_corpus.rs |  65 ++++++++-
 docs/decisions.md                          |  49 +++++++
 5 files changed, 282 insertions(+), 93 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41466186767 for 11e5938d81c592e57be72731c597ffa912b52fe6)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41466186767 for 11e5938d81c592e57be72731c597ffa912b52fe6)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41466186767 for 11e5938d81c592e57be72731c597ffa912b52fe6)
- ci-result: `not a check` passed (Check results for 11e5938d81c592e57be72731c597ffa912b52fe6 come from the CI run https://github.com/doodla/athena/actions/runs/41466186767 (518s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
