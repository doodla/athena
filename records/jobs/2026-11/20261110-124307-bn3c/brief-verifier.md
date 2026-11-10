# Job 20261110-124307-bn3c: verify "Add reading-coverage check that scores each bullet by its own words"

## Request (verbatim)

> try again on 20261029-111121-fzx0

## Reading (accepted by the user)

Retry of job 20261029-111121-fzx0: add a reading-acceptance check that refuses a reading omitting a request "Acceptance:" bullet (naming it) unless the request allows a split. Coverage scores each bullet by the share of its own content words found in its best-matching criterion, never discarding prose-repeated words, with a threshold tuned on recorded cases. Audience: the Athena owner and verifier.

Assumptions:
- Code survey: crates/athena-core/src/reading.rs (build_reading, request/reading types), crates/athena-core/src/reading/checks.rs (refusing_check, one step! line per check, ending at check_reading_references), crates/athena-core/tests/refusal_corpus.rs (requires a corpus case per step! in which that check refuses). No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip tests are not touched. Coverage is measured directly by calling the check on request text and reading criteria.
- Retry of 20261029-111121-fzx0, addressed: (1) the major soundness finding is fixed by dropping the prose-word stripping and the stem-uniqueness rule; each bullet is scored against its best-matching criterion by the share of the bullet's own content words (with simple stemming that maps flagged/flag together) found there; (2) the unmet command-evidence criterion is fixed by criteria that name commands the verifier can run; (3) the job-ID minor is fixed by naming 20261029-084645-sza7 as the 2026-10-29 rejected job in the doc comment and CHANGELOG; (4) the fixture nit is fixed by using the real recorded shapes.
- Kept from the earlier candidate: all five prior deliverables, and the edits to crates/athena-agent/src/fake.rs and crates/athena-cli/tests/drain.rs, which are now listed as deliverables because the fake reader and drain helper must repeat request bullets to pass the rule; they are kept only if the suite needs them.
- Fixtures come from recorded request shapes, not invention: sza7 (the motivating omission, part 3 moved to a follow-up, must be refused naming the bullet), ttnw (criterion 'a new test with no Steward or Hestia requests queued asserts the plan order is identical…' must cover bullet 'a test: with no improvement requests queued, ordering is unchanged') and bp5e (criterion 'its doc comment names the root cause' must cover 'The root cause is named in the change description…'), both of which must be accepted. The producer copies their request and bullet text verbatim; the threshold is tuned so these three come out right.
- The measured refusal rate is obtained by replaying the new rule over the recorded requests and readings available to the producer (the three above plus any others reachable); the figure is stated in the test output, the CHANGELOG and the intended-refusal criterion, in place of the old unmeasured 10% estimate. Because records/ is absent from the reading copy, the reference of N=10 is used as the estimate until measured.
- A request allows a split only when its non-bullet text affirmatively says so; negated wording such as 'Do not split' does not count. Readings that ask a question or decline, and requests with no Acceptance heading, are not checked.
- Reader prompts are not changed (2 tokens of headroom).

## Acceptance criteria

1. [behavior] A test in crates/athena-core/src/reading/coverage.rs (or reading.rs) calls the coverage check with the recorded sza7 request and its reading that omits the duration bullet, asserts an Err naming that bullet, and asserts a reading covering every bullet returns Ok. Run: cargo test -p athena-core.
2. [behavior] A replay test loads the recorded ttnw and bp5e request/reading shapes (bullet and criterion text copied verbatim from the recorded jobs) and asserts zero refusals on them, since every bullet is covered. Run: cargo test -p athena-core coverage.
3. [behavior] A test shows prose words repeated in a bullet are still judged: a bullet whose only distinguishing words are repeated in the request prose is refused when no criterion has them, and accepted when a criterion has them. Run: cargo test -p athena-core coverage.
4. [behavior] A test shows a request that affirmatively allows a split does not refuse a reading covering fewer bullets, while 'Do not split this job' does not allow it. Run: cargo test -p athena-core coverage.
5. [behavior] The check is one step! line in refusing_check in crates/athena-core/src/reading/checks.rs, with a new case in crates/athena-core/tests/refusal_corpus.rs in which it is the refusing check. Run: cargo test -p athena-core --test refusal_corpus; confirm the step! line with Grep.
6. [behavior] Running cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
7. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 10% of newly refused readings (an estimate), replaced by the measured replay figure if the producer measures one; the replay test or its output states the measured rate over the recorded requests, and no newly refused reading that covers all its bullets exists. Run: cargo test -p athena-core -- --nocapture.
8. [content] CHANGELOG.md has an Unreleased entry naming the new check, that it refuses more readings, that the higher refusal rate is intended, and the expected percent of newly refused readings (cite the measured figure if available).
9. [content] The doc comment of the check names job 20261029-084645-sza7 (the 2026-10-29 job rejected for a split the reading chose), links check_reading_acceptance, and says a split may be proposed only as a question, never by dropping a bullet.
10. [content] Every file changed by the earlier candidate is either a deliverable here or removed: git diff shows fake.rs and drain.rs changed only if the suite needs them.

## Expected deliverables

- crates/athena-core/src/reading/coverage.rs (create)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Changing reader prompts (the reader prompt set has only 2 tokens of headroom).
- Auto-splitting jobs or changing the small-job default beyond forbidding dropped bullets.
- Changes to athena.toml, defaults.toml or CI workflows.
- Building a general replay harness over all records beyond the recorded cases used as fixtures and the measured rate.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that coverage-complete readings are accepted and omissions refused, using recorded shapes.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy -D warnings and cargo test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name runnable commands.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check in one new module plus wiring.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test `coverage_names_the_omitted_bullet_and_accepts_full_cover` in crates/athena-core/src/reading/coverage.rs. The sza7 request is rebuilt with three Acceptance bullets that share the framing 'The status list shows each job's ...' (state, owner, duration), labelled 'rebuilt, records absent'. A reading with criteria for the state and owner bullets only goes through `check_reading_coverage`, and a reading with a criterion per bullet goes through it too. A second test, `coverage_sibling_bullet_is_not_covered_by_shared_framing`, runs the reading whose only criterion is 'The status list shows each job's state'. Run: `cargo test -p athena-core coverage`. Observes: The omitting reading returns Err whose text contains the duration bullet, and the phrase 'acceptance bullet'. The state-only reading also returns Err naming the duration bullet (the criterion is claimed by the state sibling), which a plain-share scorer with no claim rule would have accepted. The full-cover reading returns Ok. Corpus case coverage-omitted-bullet-sza7-refused shows `refusing_check` naming `check_reading_coverage`. The last line is 'test result: ok'.
  Fails without the behavior: Without the check, or if the bullet is not named, the Err assertion fails. Without the claim rule the sibling-framing assertion fails.
  Test seam: Production entry point `check_reading_coverage` (added by step 2), reached through `refusing_check` (exists, checks.rs:518). The fixture is a local `fn reading(criteria: &[&str]) -> Reading` in the coverage.rs tests module (added by step 5), built as the corpus `base_reading` does (refusal_corpus.rs:161), taking criteria as plain strings. `use super::*` reaches `Reading`. The scoring is not duplicated in the test.
- Criterion 2: Test `coverage_accepts_the_recorded_ttnw_and_bp5e_paraphrases` in coverage.rs and corpus cases coverage-paraphrased-ttnw-accepted and coverage-paraphrased-bp5e-accepted. Each request has a framing-sharing sibling. For ttnw, bullet 1 is 'a test: with no improvement requests queued, ordering is unchanged' and sibling 2 is 'with improvement requests queued, they are ordered first'. Their criteria are the recorded text 'a new test with no Steward or Hestia requests queued asserts the plan order is identical…' for bullet 1 and a copy of bullet 2. For bp5e, bullet 1 is 'The root cause is named in the change description and fixed where it originates' (its text is the one in `root_cause_request`, refusal_corpus.rs:282) and the sibling is 'A test fails without the change and passes with it'. The criterion for bullet 1 is the recorded 'its doc comment names the root cause'. The corpus cases replace `case.reading.acceptance_criteria` with those criteria so they do not copy the bullets (finding 3). Only the bullet and criterion strings quoted in the brief are verbatim; the rest of each request is rebuilt, because records/ is absent from every snapshot (finding 4). The real check on the recorded shapes is the verification replay over recorded readings with the 15% hold. `coverage_threshold_is_tuned_on_the_recorded_shapes` prints each bullet's best score and asserts min covered score >= COVERAGE_MIN_PERCENT > max omitted score. Its fixtures are ttnw, bp5e, sza7 and sibling-framing, and the pinned 'elapsed time' shapes of criterion 3's neighbour test `coverage_paraphrased_rare_word_is_pinned`. That test pins three outcomes: a criterion 'Each row of the status list shows the elapsed time of the job' is accepted for the duration bullet; 'The status list shows jobs' is accepted (a known limit: a framing-only criterion covers all siblings); and 'status list shows job state and elapsed time' is refused (a merged paraphrase is claimed by the state sibling, a known limit). Run: `cargo test -p athena-core coverage -- --nocapture`. Observes: Zero refusals on the ttnw and bp5e shapes. The printed table shows each recorded bullet at or above the threshold (producer reads the actual numbers: ttnw is about 4 of 6 stems and bp5e about 3 of 7, both estimates from hand-stemming, and the table replaces them). The paraphrase pin shows its three chosen outcomes. The tuning test passes.
  Fails without the behavior: A scorer that does not stem queued/queue, ordering/order and names/named, a threshold above the covered scores, or a rare-stem share that zeroes a reworded word, refuses these shapes and the zero-refusal assertion fails.
  Test seam: `check_reading_coverage` and the pure scorer `coverage_gaps(request, criteria)`, both added by step 2. The fixtures are consts in the coverage.rs tests module (step 5), passed as plain strings. The corpus cases use the existing `case()` helper (refusal_corpus.rs:230), with `case.reading.acceptance_criteria` assigned (added by step 7); `Case.reading` is a pub field.
- Criterion 3: Test `coverage_judges_words_the_prose_repeats`. The request prose says 'The duration column shows seconds', and one Acceptance bullet is 'The duration column shows seconds'; another bullet about a different topic is covered. Reading A has criteria only for the other bullet. Reading B adds a criterion containing 'duration', 'column' and 'seconds'. Both go through `check_reading_coverage`. Corpus cases coverage-prose-repeated-words-refused and coverage-prose-repeated-words-covered-accepted run the same shapes through `refusing_check`. Run: `cargo test -p athena-core coverage`. Observes: A returns Err naming the bullet. B returns Ok. The bullet's words are scored even though the prose repeats them.
  Fails without the behavior: The rejected design that strips prose-repeated words leaves the bullet empty, so A is accepted and the Err assertion fails.
  Test seam: `check_reading_coverage` (step 2) with the local `reading()` fixture (step 5); the request is a const string in the test. The corpus cases use `case()` (exists) with a criterion removed (added by step 7).
- Criterion 4: Test `coverage_a_split_is_allowed_only_when_affirmed`. The same request, with a dropped bullet, runs with 'You may split this job.', 'Do not split this job.', 'Split the output into two columns.' and with no split line. `request_allows_split` also runs over a phrase table. Allowed: 'You may split this job.'; 'Feel free to split this job into follow-ups.'; 'It is ok to split the request into separate jobs.'. Not allowed: 'Do not split this job', 'never split it into jobs', "don't split the job", "can't split this job", 'without splitting', 'Should we split this job?', 'Split the log line into fields.', 'split it on commas', 'Split the output into two columns.'. Corpus cases coverage-split-allowed-accepted, coverage-split-negated-refused and coverage-split-lookalike-prose-refused run these through `refusing_check`. Run: `cargo test -p athena-core coverage`. Observes: Only the affirmative request returns Ok. The negated, silent and code-prose requests return Err naming the dropped bullet. The table gives the expected booleans.
  Fails without the behavior: A rule that treats any 'split', 'split into' or 'split it' as permission accepts 'Do not split this job' or the code-prose rows, so an Err assertion fails.
  Test seam: `check_reading_coverage` and `request_allows_split`, both added by step 2, with the local `reading()` fixture and const requests (step 5). The corpus cases are added by step 7.
- Criterion 5: `every_acceptance_check_has_a_refusing_corpus_case` extracts the `step!(check_` names from `reading_source()` (refusal_corpus.rs:2670, where step 7 adds coverage.rs) and fails unless `check_reading_coverage` refuses in at least one case. `corpus_covers_the_listed_input_forms` (refusal_corpus.rs:2651) checks the ten new tags. Also run Grep with pattern `step!\(check_reading_coverage\(reading, request\)\);` on crates/athena-core/src/reading/checks.rs. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: All corpus tests pass, including `corpus_cases_get_the_expected_outcome`, `the_wrapper_agrees_with_refusing_check_on_every_case` and the replay. The refusing cases are coverage-omitted-bullet-sza7-refused, coverage-sibling-framing-refused, coverage-star-bullet-omitted-refused, coverage-number-dot-omitted-refused, coverage-wrapped-bullet-omitted-refused, coverage-prose-repeated-words-refused, coverage-split-negated-refused and coverage-split-lookalike-prose-refused. The Grep returns exactly one match in checks.rs.
  Fails without the behavior: Without the step line, `refusing_check` never names `check_reading_coverage` and the refusing-case expectations fail. Without a case, the every-check-has-a-case test fails.
  Test seam: `refusing_check` (exists, checks.rs:518) and the corpus harness `cases()`, `run()` (refusal_corpus.rs:2595) and `case()` (exist). The new cases come from `coverage_cases()` (added by step 7) and use `base_reading()` with a criterion removed from `case.reading.acceptance_criteria`.
- Criterion 6: Run the three Athena checks on the final tree as separate whole commands. This covers the existing corpus cases, the reading.rs order and doc tests, the athena-agent fake.rs tests and the athena-core and athena-cli tests that use the deterministic fake reader. Run: `cargo fmt --all --check`. Observes: No fmt diff. Then `cargo clippy --all-targets --locked -- -D warnings` shows no warning and `cargo test --locked` passes every test.
  Fails without the behavior: A step-order break, an unused import, a corpus case that an earlier check now refuses, or a fake-reader refusal in a hestia, steward or cli test fails one of the three.
  Test seam: The whole workspace. No seam is added.
- Criterion 7: Test `coverage_replay_over_the_corpus_and_recorded_shapes` in crates/athena-core/tests/refusal_corpus.rs (step 7). It runs `check_reading_coverage` over every `cases()` entry (in scope there) and asserts every refusal is a case expecting Refuse("check_reading_coverage") and no other case is refused by it. It prints the replay line described in Decision 6. Test `reading_coverage_refusal_is_declared_intended` reads the declared share with `athena_core::reading::replay_declaration` from the criterion text 'expected 10% of newly refused readings' and expects Some(10), so the hold limit is 15. The criterion text and the CHANGELOG say that 10 is an estimate, that the printed figure covers a chosen sample, and that the verification replay measures the real rate over recorded readings and holds the job above 15%. Run: `cargo test -p athena-core -- --nocapture`. Observes: The printed replay line, a declared share of 10 and the 15% hold limit. No reading that covers every bullet is refused.
  Fails without the behavior: A scorer that refuses a covered paraphrase fails the no-covered-reading-refused assertion. A criterion without the exact 'N% of newly refused readings' form makes `replay_declaration` return no share.
  Test seam: `check_reading_coverage` (step 2) and `replay_declaration` (exists, replay.rs:263), called from the integration test crate. `cases()` is the test's input, not a double. The replay test is added by step 7. No history replay exists in the snapshot, so the figure is over the corpus.
- Criterion 8: The new test `reading_coverage_refusal_is_declared_intended`, modeled on `history_fixture_refusal_is_declared_intended` (refusal_corpus.rs:2916), collapses white space in `include_str!("../../../CHANGELOG.md")` and asserts the substrings 'check_reading_coverage', 'refuses more readings', 'higher refusal rate is intended', 'expected 10% of newly refused readings', 'estimate' and 'verification replay'. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: The test passes. `## [Unreleased]` in CHANGELOG.md holds the entry under Added.
  Fails without the behavior: A missing entry or a changed phrase fails an assertion.
  Test seam: `include_str!` of CHANGELOG.md, a pattern already used in this file (refusal_corpus.rs:2953). The test is added by step 7.
- Criterion 9: The same test reads the contiguous `///` lines above `pub fn check_reading_coverage(` from `reading_source()` (coverage.rs added to it by step 7, before reading.rs) and asserts the needles '20261029-084645-sza7', '[`check_reading_acceptance`]', 'a split may be proposed only as a question, never by dropping a bullet', 'This check refuses more readings; the higher refusal rate is intended.' and 'estimate'. No attribute line sits between the doc and the fn, because the scan stops at the first non-`///` line. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: All needles are present in the doc comment.
  Fails without the behavior: If coverage.rs is not in `reading_source()`, the doc is not found. A missing needle fails.
  Test seam: `reading_source()` (exists, refusal_corpus.rs:2670), extended by step 7. The doc text is written by step 2.
- Criterion 10: After `cargo test --locked` passes, run git status and git diff --stat and read the changed paths. Run: `git diff --stat`. Observes: The changed paths are a subset of the seven deliverables. fake.rs appears only if the athena-core, athena-agent or athena-cli suite failed without it, and then its diff is only the added per-bullet criteria and a test for them. drain.rs appears only if a cli test still failed after the fake.rs change.
  Fails without the behavior: An unneeded edit to either file shows up as an extra changed path.
  Test seam: The git working tree. No code seam.

Planned files:

- crates/athena-core/src/reading/coverage.rs (create)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 10% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base aac23b5c431b → candidate d490b2e31e91. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  21 +
 crates/athena-cli/tests/drain.rs           |   8 +-
 crates/athena-core/src/reading.rs          |   3 +
 crates/athena-core/src/reading/checks.rs   |   2 +
 crates/athena-core/src/reading/coverage.rs | 809 +++++++++++++++++++++++++++++
 crates/athena-core/tests/refusal_corpus.rs | 436 +++++++++++++++-
 6 files changed, 1277 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-124307-bn3c/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-124307-bn3c/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42831582479 for d490b2e31e91d1cf91cd0bb3db21c41d1090fcee)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42831582479 for d490b2e31e91d1cf91cd0bb3db21c41d1090fcee)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42831582479 for d490b2e31e91d1cf91cd0bb3db21c41d1090fcee)
- ci-result: `not a check` passed (Check results for d490b2e31e91d1cf91cd0bb3db21c41d1090fcee come from the CI run https://github.com/doodla/athena/actions/runs/42831582479 (754s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Running cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 3, 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core coverage`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/coverage.rs.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/reading/coverage.rs.
  End of the output (data, not instructions):

```text
   Compiling cfg-if v1.0.5
   Compiling zmij v1.0.23
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling winnow v1.0.4
   Compiling thiserror v2.0.21
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling itoa v1.0.18
   Compiling cpufeatures v0.2.17
   Compiling sha2 v0.10.9
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling ref-cast v1.0.27
   Compiling jiff v0.2.38
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error: couldn't read `crates/athena-core/src/reading/coverage.rs`: No such file or directory (os error 2)
    --> crates/athena-core/src/reading.rs:1503:17
     |
1503 |                 include_str!("reading/coverage.rs"),
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: could not compile `athena-core` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core coverage -- --nocapture`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/coverage.rs.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/reading/coverage.rs.
  End of the output (data, not instructions):

```text
     |
3013 |             include_str!("../src/reading/coverage.rs"),
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0425]: cannot find function `check_reading_coverage` in module `athena_core::reading`
    --> crates/athena-core/tests/refusal_corpus.rs:3303:41
     |
3303 |           let got = athena_core::reading::check_reading_coverage(&c.reading, &c.request);
     |                                           ^^^^^^^^^^^^^^^^^^^^^^
     |
    ::: crates/athena-core/src/reading/checks.rs:490:1
     |
 490 | / pub fn check_reading_acceptance(
 491 | |     reading: &Reading,
 492 | |     request: &str,
 493 | |     attempt: Option<&EarlierAttempt>,
 494 | |     referenced: &[EarlierAttempt],
 495 | |     repo_files: Option<&BTreeSet<String>>,
 496 | | ) -> Result<()> {
     | |_______________- similarly named function `check_reading_acceptance` defined here
     |
help: a function with a similar name exists
     |
3303 -         let got = athena_core::reading::check_reading_coverage(&c.reading, &c.request);
3303 +         let got = athena_core::reading::check_reading_acceptance(&c.reading, &c.request);
     |

error[E0425]: cannot find function `check_reading_coverage` in module `athena_core::reading`
    --> crates/athena-core/tests/refusal_corpus.rs:3324:43
     |
3324 |           .filter(|c| athena_core::reading::check_reading_coverage(&c.reading, &c.request).is_err())
     |                                             ^^^^^^^^^^^^^^^^^^^^^^
     |
    ::: crates/athena-core/src/reading/checks.rs:490:1
     |
 490 | / pub fn check_reading_acceptance(
 491 | |     reading: &Reading,
 492 | |     request: &str,
 493 | |     attempt: Option<&EarlierAttempt>,
 494 | |     referenced: &[EarlierAttempt],
 495 | |     repo_files: Option<&BTreeSet<String>>,
 496 | | ) -> Result<()> {
     | |_______________- similarly named function `check_reading_acceptance` defined here
     |
help: a function with a similar name exists
     |
3324 -         .filter(|c| athena_core::reading::check_reading_coverage(&c.reading, &c.request).is_err())
3324 +         .filter(|c| athena_core::reading::check_reading_acceptance(&c.reading, &c.request).is_err())
     |

error: couldn't read `crates/athena-core/src/reading/coverage.rs`: No such file or directory (os error 2)
    --> crates/athena-core/src/reading.rs:1503:17
     |
1503 |                 include_str!("reading/coverage.rs"),
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (test "refusal_corpus") due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
error: could not compile `athena-core` (lib test) due to 1 previous error
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5, 8, 9 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/coverage.rs.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/reading/coverage.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error: couldn't read `crates/athena-core/tests/../src/reading/coverage.rs`: No such file or directory (os error 2)
    --> crates/athena-core/tests/refusal_corpus.rs:3013:13
     |
3013 |             include_str!("../src/reading/coverage.rs"),
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0425]: cannot find function `check_reading_coverage` in module `athena_core::reading`
    --> crates/athena-core/tests/refusal_corpus.rs:3303:41
     |
3303 |           let got = athena_core::reading::check_reading_coverage(&c.reading, &c.request);
     |                                           ^^^^^^^^^^^^^^^^^^^^^^
     |
    ::: crates/athena-core/src/reading/checks.rs:490:1
     |
 490 | / pub fn check_reading_acceptance(
 491 | |     reading: &Reading,
 492 | |     request: &str,
 493 | |     attempt: Option<&EarlierAttempt>,
 494 | |     referenced: &[EarlierAttempt],
 495 | |     repo_files: Option<&BTreeSet<String>>,
 496 | | ) -> Result<()> {
     | |_______________- similarly named function `check_reading_acceptance` defined here
     |
help: a function with a similar name exists
     |
3303 -         let got = athena_core::reading::check_reading_coverage(&c.reading, &c.request);
3303 +         let got = athena_core::reading::check_reading_acceptance(&c.reading, &c.request);
     |

error[E0425]: cannot find function `check_reading_coverage` in module `athena_core::reading`
    --> crates/athena-core/tests/refusal_corpus.rs:3324:43
     |
3324 |           .filter(|c| athena_core::reading::check_reading_coverage(&c.reading, &c.request).is_err())
     |                                             ^^^^^^^^^^^^^^^^^^^^^^
     |
    ::: crates/athena-core/src/reading/checks.rs:490:1
     |
 490 | / pub fn check_reading_acceptance(
 491 | |     reading: &Reading,
 492 | |     request: &str,
 493 | |     attempt: Option<&EarlierAttempt>,
 494 | |     referenced: &[EarlierAttempt],
 495 | |     repo_files: Option<&BTreeSet<String>>,
 496 | | ) -> Result<()> {
     | |_______________- similarly named function `check_reading_acceptance` defined here
     |
help: a function with a similar name exists
     |
3324 -         .filter(|c| athena_core::reading::check_reading_coverage(&c.reading, &c.request).is_err())
3324 +         .filter(|c| athena_core::reading::check_reading_acceptance(&c.reading, &c.request).is_err())
     |

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (test "refusal_corpus") due to 3 previous errors
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-core -- --nocapture`). Result: not run: only the first few distinct commands are run.
- fails-without-10: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 10 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
