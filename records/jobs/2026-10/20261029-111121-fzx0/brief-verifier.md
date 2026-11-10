# Job 20261029-111121-fzx0: verify "Refuse readings that omit a request acceptance bullet"

## Request (verbatim)

> Priority: high
> A reading may drop one of the request's acceptance items into a follow-up (under the small-job default), and nothing catches it until the final verdict marks the reading a misread, after the full job has run: on 2026-10-29 a job passed its checks and the verifier and was then rejected for a split the reading itself chose. Check at reading acceptance that every acceptance bullet of the request is covered by the reading's criteria; a reading that leaves one out is refused at once with the missing bullet named, unless the request itself allows a split. The small-job default may propose a split only as a question back to the request, never by dropping a bullet.
> Acceptance:
> - A test shows a reading that omits one of the request's acceptance bullets is refused at reading time naming the bullet, and one that covers them all is accepted.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/reading.rs, CHANGELOG.md

## Reading (accepted by the user)

Add a reading-acceptance check that parses the request's "Acceptance:" bullets and refuses, naming the bullet, any reading whose criteria do not cover one, unless the request allows a split. Tests live in reading.rs, with a refusal-corpus case and a CHANGELOG entry. Audience: the Athena owner and the verifier, who judge readings at acceptance.

Assumptions:
- Code survey: read crates/athena-core/src/reading/checks.rs (check_reading_acceptance calls refusing_check, which runs step!(check_...) lines in order; the new check is added as one step! line after check_outcome_state, never as a direct call), crates/athena-core/src/reading/outcome.rs (check_outcome_state, the closest model for a request-versus-reading check), crates/athena-core/src/reading.rs (the tests module and the existing check_reading_acceptance tests) and crates/athena-core/tests/refusal_corpus.rs (it fails when a step! check has no corpus case in which it refuses). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. Coverage is measured directly: the check compares each parsed request bullet with the reading's criteria text, and the tests call check_reading_acceptance with a real request string.
- Request bullets are the '- ', '* ' or 'N.' lines under the request's 'Acceptance:' heading, up to the next blank line or 'Files:' line. A bullet is covered when a criterion contains its significant words, or a distinctive quoted or path token from it. The matching reuses existing stem and stop-list helpers where they exist.
- The request 'allows a split' when its text says so, for example 'split', 'follow-up' or 'in parts' outside the Acceptance bullets. The check then does not refuse. A reading may propose a split only through the question field, never by dropping a bullet.
- The refusal error names the missing bullet verbatim and says to cover it or ask a question. It is a new refusal rule, so the CHANGELOG states that it refuses more readings, that the higher refusal rate is intended, and 'expected 10% of newly refused readings' (an estimate, not measured).
- The new check lives in crates/athena-core/src/reading/checks.rs, or in a small new module under reading/, and is wired into refusing_check. The request lists only reading.rs and CHANGELOG.md, but the gate code and its mandatory corpus case sit in those other files, so they are deliverables too.
- Existing readings in reading.rs tests that use a request with an Acceptance section may need criteria that cover every bullet, so those fixtures are adjusted. Consumers: none found (no config deliverable).

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/reading.rs calls check_reading_acceptance with a request holding two Acceptance bullets and a reading whose criteria cover only one. It asserts an Err whose text names the omitted bullet, and a second reading covering both returns Ok. Run: cargo test -p athena-core.
2. [behavior] A test shows a request that allows a split (it says so in its text) does not refuse a reading that covers fewer bullets. Run: cargo test -p athena-core.
3. [behavior] The refusal-corpus test passes with a new case in crates/athena-core/tests/refusal_corpus.rs in which the new step! check is the one that refuses. Run: cargo test -p athena-core --test refusal_corpus.
4. [behavior] The new check is one step! line in refusing_check in crates/athena-core/src/reading/checks.rs, and the doc of check_reading_acceptance links it. Check with Grep and cargo test -p athena-core.
5. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
6. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 10% of newly refused readings (an estimate).
7. [content] CHANGELOG.md has an Unreleased entry naming the new check, that it refuses more readings, that the higher refusal rate is intended, and 'expected 10% of newly refused readings'.
8. [content] The doc comment of the new check names the 2026-10-29 job that was rejected for a split the reading chose, and says a split may be proposed only as a question, never by dropping a bullet.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading/coverage.rs (create)

## Out of scope

- Changing reader prompts (the reader prompt set has only 2 tokens of headroom).
- Auto-splitting jobs or changing the small-job default beyond forbidding dropped bullets.
- Changes to athena.toml, defaults.toml or CI workflows.
- Replaying past readings beyond the declared estimate.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new unit tests, a corpus case, and cargo test, clippy -D warnings and fmt to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run check_reading_acceptance on accepted and refused readings.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is a refused reading naming the missing bullet, not the existence of the check.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced as a step! in refusing_check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in the tests module of crates/athena-core/src/reading.rs. Case A (distinct wording): request with an `Acceptance:` heading and bullets 'A reading that omits a bullet is refused naming the bullet' and 'CHANGELOG.md has an Unreleased entry'; a reading with one criterion covering only the first gets Err whose text contains the second bullet verbatim and starts 'the reading was refused because'; `refusing_check` on it returns the name `check_request_coverage`. Case B (finding 1): bullets 'A test shows a star bullet is refused' and 'A test shows a numbered part is refused'; the reading's only criterion is the first bullet; Err naming the second. Case C (token): bullets 'docs/guide.md is documented' and 'docs/notes.md is documented'; the reading covers only the first; Err naming the second. Case D: for A, B and C a reading with criteria covering both (one paraphrased in A) returns Ok. Parser tests (`coverage_parser_*`): dash, star and `1.` bullets, an indented two-space bullet is found, a two-line (wrapped) bullet is one bullet whose body holds both lines and whose tokens from line 2 are required (finding 3), headings `Acceptance:`, `Acceptance criteria:`, `## Acceptance`, `**Acceptance:**`, stop at a blank line and at `Files:`, a bullet after the section is ignored, `1)` parts ignored, no heading gives nothing. Stem tests (`coverage_stems_*`): name/names/naming/named all related (finding 2), omit/omitted, accept/acceptance, cover/coverage, change not counted for a bullet whose word is CHANGELOG.md, shared stems removed, empty specific set falls back. Run: `cargo test -p athena-core request_coverage_refuses_an_omitted_bullet_and_accepts_full_coverage && cargo test -p athena-core coverage_parser && cargo test -p athena-core coverage_stems`. Observes: The omitting readings return Err naming the omitted bullet in all three shapes, including the near-identical and path-token ones; the covering readings return Ok; wrapped bullets and indented bullets are parsed; name/naming match.
  Fails without the behavior: Without the step, check_reading_acceptance returns Ok for each omitting reading and the unwrap_err() panics; under the old 'tokens OR half of all stems' rule cases B and C are accepted; without the continuation join the wrapped-bullet test fails; with the 4-letter stripping floor the name/naming test fails.
  Test seam: Production entry points `check_reading_acceptance(&reading, request, None, &[], None)` (checks.rs:490) and `refusing_check` (checks.rs:518), both exist. Fixture `governed_reading(&["docs/guide.md"], &[criteria], &[])` (reading.rs:4527, exists). The request is a plain string argument; no double. `check_request_coverage` and its step line are added by plan steps 1 and 3.
- Criterion 2: Unit test in reading.rs. The same omitting reading is run against: (a) the request plus 'You may split this job into parts.' -> Ok; (a2) plus 'The work can be split into follow-ups.' -> Ok; (b) plus 'Do not split this job.' -> Err naming the bullet; (c) the text of this job's request (says 'split' and 'follow-up', allows nothing) -> Err; (d) plus 'The log is read in parts and the parser may split into tokens.' -> Err; (e) plus 'If it is large you may split this job.' -> Err; and the omitting reading with `question = Some(..)` against the unmodified request -> Ok; a declining reading (`capability` None) -> Ok. Run: `cargo test -p athena-core request_coverage_lets_a_request_that_allows_a_split_through`. Observes: Only an affirmative, unnegated permission to split the job or work in the request's own text, or a question in the reading, lets a bullet be uncovered; unrelated 'in parts' or 'split into' sentences do not.
  Fails without the behavior: Without the allow-split rule (a) and (a2) are refused; without the negation guard (b) is accepted; with a bare 'split' keyword (c) is accepted; with a phrase list (d) is accepted.
  Test seam: `check_reading_acceptance` with request strings as arguments; reading from `governed_reading(..)` (exists, reading.rs:4527) with `question` set in the question case. No double.
- Criterion 3: `coverage_cases()` in crates/athena-core/tests/refusal_corpus.rs added to `cases()` (line 292). Each case starts from `case(..)`/`base_reading(request)` (criteria are the request's bullet lines verbatim; exist, refusal_corpus.rs:161, 230) and edits one field. 15 cases (id -> outcome, new tag): request-coverage-omitted-bullet-in-job-fzx0-request (this job's request verbatim, second criterion removed; Refuse check_request_coverage; acceptance-bullet-omitted, request-mentions-split-without-allowing); request-coverage-all-bullets-covered-in-job-fzx0-request (Accept); request-coverage-paraphrased-criteria-cover-bullets (this job's reading criteria 1 to 8 in the reading's own words against the verbatim bullets; Accept; paraphrased-criteria-cover-bullets; a leniency probe); request-coverage-near-identical-bullets-one-omitted (Refuse; near-identical-bullets-one-omitted); request-coverage-path-token-bullet-omitted (docs/guide.md vs docs/notes.md; Refuse; path-token-bullet-omitted); request-coverage-wrapped-bullet-second-line-omitted (a two-line bullet whose second line holds a path token the criteria lack; Refuse; wrapped-bullet); request-coverage-numbered-parts-one-omitted (`1.` `2.` `3.`, part 2 dropped; Refuse); request-coverage-indented-bullet-omitted (Refuse; indented-bullet); request-coverage-star-bullets-covered-by-one-merged-criterion (Accept; acceptance-bullet-covered-across-criteria); request-coverage-request-allows-split (Accept; request-allows-split); request-coverage-request-forbids-split (Refuse); request-coverage-non-permitting-split-wording (Refuse; non-permitting-split-wording); request-coverage-reading-asks-the-split-as-a-question (Accept; split-proposed-as-question); request-coverage-no-heading-not-read-today (Accept; no-acceptance-heading; known gap like `numbered-paren-parts-not-read-today` :1719); request-coverage-bullet-after-the-section-is-not-acceptance (Accept; bullet-after-acceptance-section). The `1)` parts form stays with `numbered-paren-parts-not-read-today` (:1719) and `(a)` parts with `lettered-paren-parts-not-read-today` (:1732); dash/star/`1.` bullets with `steward-three-dash-bullets` (:1677) plus the new cases; a request that cites a job without relying on it with `root-cause-cites-two-jobs-without-relying` (:1751) and `follow-up-on-a-job-the-reading-ignores` (:1764, refused earlier by check_prior_result_and_scope); retry complaint, crate-relative or governance path, brand word and environment variable are read as ordinary words inside bullets and covered by identical text, so existing cases keep their outcome. 13 new tags (acceptance-bullet-omitted, near-identical-bullets-one-omitted, path-token-bullet-omitted, wrapped-bullet, acceptance-bullet-covered-across-criteria, paraphrased-criteria-cover-bullets, request-allows-split, request-mentions-split-without-allowing, non-permitting-split-wording, split-proposed-as-question, no-acceptance-heading, bullet-after-acceptance-section, indented-bullet) are added to INPUT_FORMS, whose length changes from 52 to 65. Each case pins `bullets_expected` as `request_text_acceptance` counts it. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: `corpus_cases_get_the_expected_outcome` gives the stated outcome for every case; `every_acceptance_check_has_a_refusing_corpus_case` finds `check_request_coverage` among the refusing checks; `corpus_covers_the_listed_input_forms` finds the new tags held; `the_wrapper_agrees_with_refusing_check_on_every_case` agrees on error text. The planner did not run it; no result is claimed.
  Fails without the behavior: Without the step the coverage check is invisible to `every_acceptance_check_has_a_refusing_corpus_case`; with the step but no refusing case it fails by naming `check_request_coverage`; with the check removed the Refuse cases are accepted.
  Test seam: Production entry point `athena_core::reading::refusing_check` via `run(c)` (refusal_corpus.rs:2595, exists). Fixtures `case(..)` and `base_reading(..)` exist. `reading_source()` (refusal_corpus.rs:2670) gains `include_str!("../src/reading/coverage.rs")` (step 4). No double.
- Criterion 4: Test modelled on `environment_reach_has_one_step_line` (reading.rs:5376): takes the body of `refusing_check` from `reading_source()`, asserts exactly one line equals `step!(check_request_coverage(reading, request));`, that it follows `step!(check_reading_references(reading, repo_files));`, and that the text before `pub fn check_reading_acceptance(` contains "[`check_request_coverage`]". Also Grep `step!(check_request_coverage` over crates/athena-core/src/reading/checks.rs: one line. Run: `cargo test -p athena-core request_coverage_has_one_step_line`. Observes: One step line, last, and the doc link; the corpus test that scans check_reading_acceptance for a direct check call still passes.
  Fails without the behavior: A direct call or a missing doc link fails the equality or `contains` assertion.
  Test seam: Source text via `reading_source()` (reading.rs:1491, exists; step 2 adds coverage.rs to its list). No double.
- Criterion 5: Run the three checks over the workspace after step 6's fixture fixes and the producer-loop replay response. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0; the full run includes the other crates that reach check_reading_acceptance (cli drain tests, steward, run tests, hestia).
  Fails without the behavior: Any formatting difference, warning or failing test makes the command exit non-zero.
  Test seam: The cargo commands themselves; no fixture is added.
- Criterion 6: Test in reading.rs: (1) take the real Unreleased CHANGELOG item for the check (`include_str!("../../../CHANGELOG.md")`, as sibling tests do) and the collapsed `///` doc above `pub fn check_request_coverage(` from `reading_source()`; assert `declared_percent(&normalised(..)) == Some(10)` on each (both are pub(super), reached from the reading.rs tests) and that each holds 'refuses more readings' and 'the higher refusal rate is intended'; (2) a reading that `refusing_check` accepts (returns None) against a request with no `Acceptance:` heading is refused with name `check_request_coverage` once the request has a heading and a bullet the reading omits, so the refusal rate rises by this check alone (finding 5). Run: `cargo test -p athena-core request_coverage_refusal_is_declared_intended`. Observes: The declared share parses as 10 from the change's own changelog and doc; the same reading flips from accepted to refused by the new check.
  Fails without the behavior: Without the check the second half returns None and `assert_eq!(name, "check_request_coverage")` fails; without the declared sentence in the changelog item or doc the share is None.
  Test seam: `declared_percent`, `normalised` (replay.rs:215-227, exist) and `refusing_check` with `governed_reading` (exist). The 10% is an estimate, not a measurement, and is labelled so in the doc and changelog. No double.
- Criterion 7: Add one item under `## [Unreleased]` / `### Added` in CHANGELOG.md naming `reading::check_request_coverage`, what it refuses, its exemptions, 'refuses more readings', 'higher refusal rate is intended' and 'expected 10% of newly refused readings' (an estimate). The test, modelled on `scope_limit_doc_and_changelog_name_the_cause` (reading.rs:8493), parses the Unreleased items, collapses white space and asserts the item holds those phrases. Run: `cargo test -p athena-core request_coverage_doc_and_changelog_name_the_cause`. Observes: The Unreleased item exists and carries the phrases.
  Fails without the behavior: A missing or reworded item fails the `contains` assertions.
  Test seam: `include_str!("../../../CHANGELOG.md")` in the test, as sibling tests do (exists). No double.
- Criterion 8: The same test collapses the `///` lines above `pub fn check_request_coverage(` taken from `reading_source()` and asserts they hold '2026-10-29', 'rejected for a split the reading chose', 'only as a question, never by dropping a bullet', 'This check refuses more readings; the higher refusal rate is intended.' and 'estimate'. The doc is one paragraph with no blank `///` line and states the limits (a paraphrase sharing no specific word is refused; a bullet is judged on the words and tokens that tell it from the other bullets). Run: `cargo test -p athena-core request_coverage_doc_and_changelog_name_the_cause`. Observes: The doc comment of the new check names the 2026-10-29 job and states the question-only rule.
  Fails without the behavior: A missing phrase, or coverage.rs absent from `reading_source()`, fails the test.
  Test seam: Source text via `reading_source()`, extended in step 2 with coverage.rs. No double.

Planned files:

- crates/athena-core/src/reading/coverage.rs (create)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 2 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 10% of newly refused readings (the job is held above the declared share plus 5 points).

- 20261109-161505-bp5e: the reading was refused because its acceptance criteria leave out 1 of the request's 3 acceptance bullets: "The root cause is named in the change description and fixed where it originates, not at one 
- 20261107-031902-ttnw: the reading was refused because its acceptance criteria leave out 1 of the request's 5 acceptance bullets: "a test: with no improvement requests queued, ordering is unchanged". Cover each one in a cri

## The change

Base 6f198eb9cf0e → candidate b138f5b96f16. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  16 ++
 crates/athena-agent/src/fake.rs            |  59 +++-
 crates/athena-cli/tests/drain.rs           |  11 +-
 crates/athena-core/src/reading.rs          | 432 +++++++++++++++++++++++++++++
 crates/athena-core/src/reading/checks.rs   |   6 +-
 crates/athena-core/src/reading/coverage.rs | 402 +++++++++++++++++++++++++++
 crates/athena-core/tests/refusal_corpus.rs | 327 +++++++++++++++++++++-
 7 files changed, 1248 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-111121-fzx0/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-111121-fzx0/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42815145798 for b138f5b96f1663afacadd9ef580c71b8124a06bf)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42815145798 for b138f5b96f1663afacadd9ef580c71b8124a06bf)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42815145798 for b138f5b96f1663afacadd9ef580c71b8124a06bf)
- ci-result: `not a check` passed (Check results for b138f5b96f1663afacadd9ef580c71b8124a06bf come from the CI run https://github.com/doodla/athena/actions/runs/42815145798 (826s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-core request_coverage_refuses_an_omitted_bullet_and_accepts_full_coverage && cargo test -p athena-core coverage_parser && cargo test -p athena-core coverage_stems`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core request_coverage_lets_a_request_that_allows_a_split_through`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/coverage.rs.
  End of the output (data, not instructions):

```text
8805 |                 assert!(related(a, b), "{a} {b}");
     |                         ^^^^^^^ not found in this scope

error[E0425]: cannot find function `related` in this scope
    --> crates/athena-core/src/reading.rs:8814:21
     |
8814 |             assert!(related(&stem(a), &stem(b)), "{a} {b}");
     |                     ^^^^^^^ not found in this scope

error[E0425]: cannot find function `related` in this scope
    --> crates/athena-core/src/reading.rs:8817:22
     |
8817 |             assert!(!related(&stem(a), &stem(b)), "{a} {b}");
     |                      ^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8824:22
     |
8824 |         let tokens = distinctive_tokens(body);
     |                      ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `stems_of` in this scope
    --> crates/athena-core/src/reading.rs:8826:21
     |
8826 |         let stems = stems_of(body, &tokens);
     |                     ^^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8829:22
     |
8829 |         let tokens = distinctive_tokens(path);
     |                      ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `stems_of` in this scope
    --> crates/athena-core/src/reading.rs:8831:20
     |
8831 |         assert_eq!(stems_of(path, &tokens), ["hold", "check"]);
     |                    ^^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8837:13
     |
8837 |             distinctive_tokens(
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8842:17
     |
8842 |         assert!(distinctive_tokens("read and/or write, i.e. both, v1.2 and 3.5").is_empty());
     |                 ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8844:13
     |
8844 |             distinctive_tokens("see crates/ and README.md."),
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 24 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/coverage.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error: couldn't read `crates/athena-core/tests/../src/reading/coverage.rs`: No such file or directory (os error 2)
    --> crates/athena-core/tests/refusal_corpus.rs:3001:13
     |
3001 |             include_str!("../src/reading/coverage.rs"),
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error: could not compile `athena-core` (test "refusal_corpus") due to 1 previous error
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core request_coverage_has_one_step_line`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/coverage.rs.
  End of the output (data, not instructions):

```text
     |
8805 |                 assert!(related(a, b), "{a} {b}");
     |                         ^^^^^^^ not found in this scope

error[E0425]: cannot find function `related` in this scope
    --> crates/athena-core/src/reading.rs:8814:21
     |
8814 |             assert!(related(&stem(a), &stem(b)), "{a} {b}");
     |                     ^^^^^^^ not found in this scope

error[E0425]: cannot find function `related` in this scope
    --> crates/athena-core/src/reading.rs:8817:22
     |
8817 |             assert!(!related(&stem(a), &stem(b)), "{a} {b}");
     |                      ^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8824:22
     |
8824 |         let tokens = distinctive_tokens(body);
     |                      ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `stems_of` in this scope
    --> crates/athena-core/src/reading.rs:8826:21
     |
8826 |         let stems = stems_of(body, &tokens);
     |                     ^^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8829:22
     |
8829 |         let tokens = distinctive_tokens(path);
     |                      ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `stems_of` in this scope
    --> crates/athena-core/src/reading.rs:8831:20
     |
8831 |         assert_eq!(stems_of(path, &tokens), ["hold", "check"]);
     |                    ^^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8837:13
     |
8837 |             distinctive_tokens(
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8842:17
     |
8842 |         assert!(distinctive_tokens("read and/or write, i.e. both, v1.2 and 3.5").is_empty());
     |                 ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `distinctive_tokens` in this scope
    --> crates/athena-core/src/reading.rs:8844:13
     |
8844 |             distinctive_tokens("see crates/ and README.md."),
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 24 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core request_coverage_refusal_is_declared_intended`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7, 8 (`cargo test -p athena-core request_coverage_doc_and_changelog_name_the_cause`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
