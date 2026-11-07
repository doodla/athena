# Job 20261107-094333-4z98: verify "Add honest delivery-verb outcome check with replay over real jobs"

## Request (verbatim)

> try again on 20261027-054230-8nz8

## Reading (accepted by the user)

Retry of job 20261027-054230-8nz8: add `check_outcome_state` to reading acceptance so delivery-verb requests (publish, deploy, share, release) must name an observable state. Claim only what a replay test over the recorded cited jobs shows (1 of 3 real delivery-verb misreads), fix the verifier's findings, and correct CHANGELOG and docs to match. Audience: the Athena operator and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (refusing_check at ~line 3225 runs one `step!(check_...)` line per check, ending with check_governance_coverage, check_boundary_and_repo_files, check_reading_references) and crates/athena-core/tests/refusal_corpus.rs (case lists, run(), and a test that every step! has a refusing case). No record or data file under crates/ is added, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly by calling refusing_check on real request text.
- Job 20261027-054230-8nz8 is addressed finding by finding. (a) Major, 'changelog claims 3 of 3 but only 1 caught': the CHANGELOG and the check_outcome_state doc will say the check covers 1 of the 3 cited jobs (20261014-082328-g6vy, a real publish misread). 20261014-174815-7bf8 (ladder of requirements) and 20261017-173508-0wtc (ontology decision) belong to another class, figurative or structural terms and glossary, and are stated as not covered. (b) Major, 'reading does not match request': decisions.md and CHANGELOG state the narrowed scope and the count instead of claiming the whole class. (c) Minor, substring match in the out_of_scope exit: use whole-word has_phrase matching, including the -ing form, so 'shared helpers' or 'Release notes' do not satisfy it. (d) Minor, common words in OBSERVABLE_STATE such as tag and url: drop or tighten ambiguous words so a criterion must name a reachable or served state. (e) Minor, verbs only found at sentence start: also detect the verb after 'and', 'can you', 'I want to' and similar, with tests for 'Build the docs and publish them', 'Can you deploy the site' and 'I want to release v1'. (f) Nit, step placement after check_governance_coverage is kept and explained in decisions.md.
- The replay test uses the recorded request text of 20261014-082328-g6vy, written as a fixture string in the test, and asserts the check refuses it when the reading's criteria only say a file exists. It also asserts the check does not fire for the other two cited requests and states the count 1 of 3 in the test. The CHANGELOG count must equal it.
- Past accepted readings are replayed as accepting cases: ordinary requests that name a user-visible outcome, and requests with no delivery verb such as 'Add a timeout setting', stay accepted.
- The refusal-rate estimate of 5% is a guess, not a measurement. The check applies only to requests with a delivery verb whose reading names no observable state.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`. A unit test in crates/athena-core/src/reading.rs shows check_outcome_state refusing a reading for the real g6vy request 'Publish a first explainer page on how firstmate handles work.' whose criteria only say a file exists. The error names the missing observable state (reachable URL or served page). The test fails when the check is removed.
2. [behavior] Run `cargo test -p athena-core`. A replay test runs refusing_check over the recorded request text of all three cited jobs (20261014-082328-g6vy, 20261014-174815-7bf8, 20261017-173508-0wtc), asserts the check refuses exactly the real delivery-verb misread (g6vy) and not the other two, and asserts the caught count is 1 of 3.
3. [behavior] Run `cargo test -p athena-core`. Unit tests show acceptance when a criterion names a reachable URL or served state, for deploy, share and release as well as publish, and show no refusal for 'Add a timeout setting'. Tests show detection of 'Build the docs and publish them', 'Can you deploy the site' and 'I want to release v1'.
4. [behavior] Run `cargo test -p athena-core`. Tests show an out_of_scope item such as 'Changes to shared helpers' or 'Release notes are unchanged' does not satisfy the check for a Share or Release request, and a criterion containing only 'tag' or 'URL' as incidental words (for example 'the <title> tag names the page') does not satisfy it for a publish request.
5. [content] Grep crates/athena-core/src/reading.rs shows a `step!(check_outcome_state(` line inside refusing_check, with a doc comment naming the root cause (a delivery verb read as a narrow deliverable, with no check against the state the user observes) and stating it covers 1 of the 3 cited jobs (20261014-082328-g6vy), naming the other two as outside the check.
6. [behavior] Run `cargo test -p athena-core --test refusal_corpus`. The corpus has a case in which check_outcome_state refuses and a matching accepting case, and the rule that every step! has a refusing case still passes.
7. [behavior] The change refuses more readings. Expected 5% of newly refused readings. A replay of past readings is expected to hold the job only above 10%. The higher refusal rate is intended: the check applies only to requests with a delivery verb whose reading names no observable state.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All three pass, including the briefs.rs test that asserts READING_RULES has 6 entries.
9. [content] CHANGELOG.md has an entry naming the root cause and the new check, with the count 1 of 3 equal to the replay test, and not claiming the other two jobs are caught. docs/decisions.md records the scope, the alternatives and the placement after check_governance_coverage. No governance file (athena.toml, defaults.toml, workflows, toolchain or lint config) changes.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Redoing job 20261014-082328-g6vy: xtask site, .github/workflows/pages.yml, docs/design/explainer-site.md, README.md, .gitignore, xtask/tests/policy.rs and requirements-map.md edits are dropped
- Job 20261014-174815-7bf8's landed files (steward, roles, ladder) are kept and not touched
- Job 20261017-173508-0wtc's docs/ontology.toml, ontology test and decisions entry are dropped; no glossary or ontology is built here
- Changing the reader prompt or READING_RULES in crates/athena-context/src/briefs.rs, which has 3 tokens of headroom
- Defining figurative terms such as ladder, layer or tier, beyond delivery verbs; those two cited jobs are another class
- Changes to athena.toml, defaults.toml, .github/workflows or toolchain and lint configuration
- Enabling GitHub Pages or any deployment
- The files job 20261027-054230-8nz8 changed (CHANGELOG.md, reading.rs, refusal_corpus.rs, decisions.md) are kept as deliverables but its candidate is dropped and rewritten from the base

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit, replay and corpus tests; fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check lives in reading acceptance; the claim is limited to what the replay shows.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that refuse and accept real request text.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The changelog count is tied to the replay test result.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a step! check in refusing_check with a corpus case.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, four files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria use commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test `check_outcome_state_refuses_the_g6vy_misread` in `mod tests` of reading.rs. It builds `governed_reading(&["docs/explainer.md"], &["docs/explainer.md exists"], &[])` (Content criterion, class feature) and calls `check_outcome_state`, `check_reading_acceptance` and `refusing_check` with the request 'Publish a first explainer page on how firstmate handles work.' Each must refuse; the error must contain `reachable URL`, `served page` and `publish`; `refusing_check` must return the name `check_outcome_state`. Run: `cargo test -p athena-core --lib check_outcome_state_refuses_the_g6vy_misread`. Observes: The g6vy request with a file-only criterion is refused by check_outcome_state, and the error names a reachable URL and a served page. 'page' is the 4th word after 'Publish', inside the 5-word window.
  Fails without the behavior: Without `check_outcome_state` in `refusing_check`, `refusing_check` returns None and `check_reading_acceptance` returns Ok, so the unwrap_err and name assertions fail; without the function the test does not compile.
  Test seam: Production entry points: `check_outcome_state` (added by step 3), `refusing_check` (exists, reading.rs line 3225) and `check_reading_acceptance` (exists, line 3197). Fixture: the plain `Reading` from `governed_reading` (exists, line 7013, over `scoped_reading` at 6157), passed as an argument; no double. Test added by step 5.
- Criterion 2: Test `delivery_verb_replay_over_the_cited_jobs` in refusal_corpus.rs. A table of three entries (job, request, `verbatim: bool`, source label): g6vy with the request as quoted in the brief, verbatim true; 7bf8 and 0wtc as stand-in text built from D94 and D152, verbatim false, labelled 'stand-in, not the recorded request'. For each it builds `base_reading(request)`, runs `refusing_check(.., Some(&repo_files()))` and records whether the first refusal is `check_outcome_state`. Measured block: over verbatim entries only, asserts g6vy is refused and computes `caught` from the table. Illustration block (separate comment: scope, not a measurement): the stand-ins are not refused, so the whole caught set is {g6vy} and caught is 1 of total 3. CHANGELOG (via include_str!, white space collapsed) must contain `format!("covers {caught} of the {total} cited jobs")` and `format!("replayed for {verbatim} of the {total}")`, and 'not replayed' exactly while verbatim < total. If the host supplies the texts the entries flip to verbatim and the same test still holds. Run: `cargo test -p athena-core --test refusal_corpus delivery_verb_replay_over_the_cited_jobs`. Observes: g6vy is refused by check_outcome_state; the two stand-ins are accepted; the caught count is 1 and the total is 3; the CHANGELOG states the same count and says which requests were replayed.
  Fails without the behavior: Without the check the caught set is empty and the g6vy assertion fails. A CHANGELOG count that differs from the table, or a missing 'not replayed' while a stand-in remains, fails the phrase assertions.
  Test seam: Public `athena_core::reading::refusing_check` with fixtures `base_reading` (exists, refusal_corpus.rs line 106, class docs, so check_code_survey does not fire) and `repo_files` (line 162). CHANGELOG read with `include_str!` as at refusal_corpus.rs line 859. Test added by step 7. The stand-in half cannot fail on the recorded text and says so in its comment.
- Criterion 3: Unit tests in reading.rs. (1) `check_outcome_state_accepts_a_behavior_state_for_each_delivery_verb`, through `check_reading_acceptance`, with a Behavior criterion added by a test helper: 'Publish the explainer page' / 'The page is reachable at its public URL'; 'Deploy the site on every push' / 'The site is served at the deployed address'; 'Share the report with the team' / 'The shared link is reachable by the recipient'; 'Release v1' / 'The v1 release assets are downloadable'. Each is accepted. The same requests with only 'docs/x.md exists' are refused, and with the content criterion 'docs/explainer.md says the page is reachable' are refused, while the same text as a Behavior criterion is accepted. 'Add a timeout setting' with a file-only criterion is accepted. (2) `delivery_verbs_are_found_after_lead_ins_and_markers` calls the private `delivery_verbs` on 'Build the docs and publish them', 'Can you deploy the site', 'I want to release v1', '* Publish the page', '- Publish the page', '1. Share the report', '2) Deploy the site', '(a) Release v1' and 'Priority: high\n\nPublish the page' and asserts the verb is found; it asserts none is found in 'Add release notes', 'Add a share button', 'Rename shared helpers', 'Add tests for share and release.', 'We deployed it', 'Add a timeout setting', 'Release the lock before awaiting', 'Workers should share the HTTP client', 'Publish the event to the bus', 'Share the parsing code between crates', 'Workers should share the package cache', 'Release the image buffer before returning' and 'and release the slot on failure'; and finds one in 'Publish the crate to crates.io' and 'Release the binary as v1'. (3) Corpus cases `delivery-verb-after-and`, `delivery-verb-after-can-you`, `delivery-verb-after-want-to` (Refuse), `delivery-state-in-a-content-criterion-refused` (Refuse) and `delivery-verb-technical-use-is-accepted` (Accept) run the same shapes through `refusing_check`. Run: `cargo test -p athena-core --lib delivery`. Observes: A Behavior criterion with a state phrase is accepted for publish, deploy, share and release; a file-only or content-kind state criterion is refused for each; detection works at every listed position, fails on noun uses and on technical uses with no visible object; 'Add a timeout setting' is not refused.
  Fails without the behavior: Without lead-in detection the three lead-in requests are not found, so the unit test and the Refuse corpus cases fail. Without the state list the accepting pairs are refused. Without the object rule and the package-noun rule the technical-use accept cases are refused. Without the kind rule the content-kind state criterion is accepted.
  Test seam: Private `delivery_verbs` is called from `mod tests` in the same file (added by step 2). Public entry points `check_reading_acceptance` and `refusing_check`. Fixture `governed_reading` (exists, line 7013) plus a small test helper that pushes a Criterion with kind Behavior (added by step 5). Corpus cases use the case `run(c)` helper (exists, refusal_corpus.rs line 660); cases added by step 6.
- Criterion 4: Unit test `out_of_scope_and_incidental_words_do_not_satisfy_the_check`. A Share request ('Share the report with the team') with out_of_scope ['Changes to shared helpers'] is refused; a Release request ('Release v1') with ['Release notes are unchanged'] is refused; the same requests with ['Sharing the report is out of scope'] and ['Releasing is out of scope: no tag is cut'] are accepted. A publish request ('Publish the explainer page') is refused when the only extra criterion, of kind Behavior, is each of 'the <title> tag names the page', 'The URL field is documented', 'A reader can open docs/explainer.md' and 'docs/explainer.md is available at docs/'. Corpus cases `delivery-verb-out-of-scope-shared-helpers-refused`, `delivery-verb-out-of-scope-release-notes-refused` (Refuse) and `delivery-verb-out-of-scope-names-it-accepted` (Accept) run the same shapes through `refusing_check`. Run: `cargo test -p athena-core --lib out_of_scope_and_incidental_words_do_not_satisfy_the_check`. Observes: Substring and incidental words (shared, Release notes, tag, URL, can open, available at) do not satisfy the check, even in a behavior criterion; a whole-word verb or -ing form in out_of_scope does.
  Fails without the behavior: A substring match on out_of_scope accepts 'Changes to shared helpers' and 'Release notes are unchanged', so the Refuse assertions fail. A state list that holds tag, url, can open or bare available at accepts the refused criteria.
  Test seam: `check_reading_acceptance` and `refusing_check` called with `governed_reading` fixtures that set out_of_scope and criteria (helpers exist; the test and corpus cases are added by steps 5 and 6).
- Criterion 5: Pinning test `check_outcome_state_doc_names_root_cause_and_scope` reads `include_str!("reading.rs")`. It takes the body of `refusing_check` and asserts one `step!(check_outcome_state(` line directly after the `step!(check_governance_coverage(` line. It takes the doc block above `pub fn check_outcome_state(` (technique of `design_existing_code_refusal_is_declared_intended`, refusal_corpus.rs line 834; one `///` paragraph, no blank `///` line) and asserts it contains 'delivery verb', 'narrow deliverable', 'state the user observes', '1 of the 3 cited jobs', '20261014-082328-g6vy', '20261014-174815-7bf8', '20261017-173508-0wtc' and 'not replayed'. It also asserts the `check_reading_acceptance` doc names `check_outcome_state`. A reviewer can run the Grep below. Run: `cargo test -p athena-core --lib check_outcome_state_doc_names_root_cause_and_scope`. Observes: One step line sits after the governance step in refusing_check; the doc comment names the root cause and the 1-of-3 scope with the other two jobs named as outside the check. Grep `step!\(check_outcome_state\(` in crates/athena-core/src/reading.rs finds the line.
- Criterion 6: The corpus gains refusing cases whose first refusing check is `check_outcome_state` and matching accepting cases. Refuse: `delivery-verb-first-reading-names-only-a-file` (g6vy request, 0 bullets, labelled as quoted in the brief), `delivery-verb-in-a-dash-bullet`, `delivery-verb-in-a-star-bullet`, `delivery-verb-in-a-numbered-dot-part`, `delivery-verb-in-a-numbered-paren-part`, `delivery-verb-in-a-lettered-paren-part`, `delivery-verb-after-and`, `delivery-verb-after-can-you`, `delivery-verb-after-want-to`, `delivery-state-in-a-content-criterion-refused`, `delivery-verb-out-of-scope-shared-helpers-refused`, `delivery-verb-out-of-scope-release-notes-refused`. Accept (state criteria added as kind Behavior): `delivery-verb-first-reading-names-a-served-state`, `delivery-verb-out-of-scope-names-it-accepted`, `delivery-word-as-a-noun-is-accepted`, `delivery-verb-technical-use-is-accepted`. Refuse by governance first: `delivery-request-naming-a-workflow-fails-governance-first`. Each case states its source (job 20261014-082328-g6vy, or the request in the brief of job 20261107-094333-4z98) and whether it is verbatim or rebuilt. `INPUT_FORMS` goes from 17 to 23 tags. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: `corpus_cases_get_the_expected_outcome` gives every case its expected outcome and bullet count; `corpus_covers_the_listed_input_forms` finds the six new tags; `every_acceptance_check_has_a_refusing_corpus_case`, `an_extra_check_with_no_case_is_reported` and `the_wrapper_agrees_with_refusing_check_on_every_case` pass. I read these test names and the runner (lines 660 to 817) but did not run them.
  Fails without the behavior: Without a refusing case, `every_acceptance_check_has_a_refusing_corpus_case` reports `check_outcome_state` uncovered. Without the check the Refuse cases are accepted and `corpus_cases_get_the_expected_outcome` fails.
  Test seam: `refusing_check` through the corpus `run(c)` helper (exists, refusal_corpus.rs line 660) with `base_reading` (line 106) mutated per case. `an_extra_check_with_no_case_is_reported` anchors on the `step!(check_reading_references(` line, which stays. The cases and tags are added by step 6.
- Criterion 7: Unit test `outcome_state_refusal_is_declared_intended` in reading.rs, modeled on `design_existing_code_refusal_is_declared_intended` (refusal_corpus.rs line 820). It asserts `replay_declaration` (reading.rs, exists) over a criterion 'The change refuses more readings: expected 5% of newly refused readings' gives a share of Some(5); that the doc block above `check_outcome_state` contains 'This check refuses more readings; the higher refusal rate is intended.' and 'estimate'; and that the white-space-collapsed CHANGELOG contains 'refuses more readings', 'higher refusal rate is intended', 'expected 5% of newly refused readings' and 'estimate'. The 5% is not measured; records for a real replay are not in the snapshot. Run: `cargo test -p athena-core --lib outcome_state_refusal_is_declared_intended`. Observes: The declared share parses as 5 (below the 10% hold level of `check_text_replay`); the doc and CHANGELOG say the higher rate is intended and the figure is an estimate.
  Fails without the behavior: Without the doc sentence, the CHANGELOG wording or the declared share, the assertions fail; the replay would hold the job if the share were undeclared and the real rate above 10%.
  Test seam: Public `replay_declaration` called with a `Criterion` built in the test; `include_str!` of reading.rs and CHANGELOG.md. Added by step 5.
- Criterion 8: Run the three project checks over the workspace. `cargo test` includes `the_reader_brief_names_each_reading_rule_in_one_short_line` in crates/athena-context/src/briefs.rs (6 READING_RULES), the xtask policy test (boundary words, personal data) and the decisions-log format tests, so a D161 wording that the parse reads as supersession would fail there. Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`. Observes: All three exit 0; the briefs.rs test still sees 6 rules.
  Fails without the behavior: A formatting or lint fault, a changed READING_RULES, a malformed D161, or any existing test left with a file-only delivery reading fails one of the three.
  Test seam: Whole-workspace commands; no double.
- Criterion 9: Content check by reading the diff. CHANGELOG.md has one entry at the top of `[Unreleased]` / `### Added`: root cause, `check_outcome_state`, 'covers 1 of the 3 cited jobs', g6vy as the one covered job, 7bf8 and 0wtc as another class, not caught and not replayed, and the replayed count. The replay test's CHANGELOG assertions tie the count to the table. docs/decisions.md has D161 with scope, alternatives, the placement after check_governance_coverage, the behavior-kind rule, the missing reader rule and its cost, limits and stand-in provenance, and no 'supersedes', 'superseded by' or 'amends'. No governance file changes. Run: `git status`. Observes: Only reading.rs, refusal_corpus.rs, CHANGELOG.md and docs/decisions.md are modified; athena.toml, defaults.toml, .github/workflows, rust-toolchain.toml and lint config are unchanged; `git diff -- CHANGELOG.md docs/decisions.md` shows the entry and D161; the replay test and the decisions format tests pass under `cargo test`.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 9ec8ce96ff47 → candidate f7766208c289. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  21 +
 crates/athena-core/src/reading.rs          | 685 ++++++++++++++++++++++++++++-
 crates/athena-core/tests/refusal_corpus.rs | 288 +++++++++++-
 docs/decisions.md                          |  50 +++
 4 files changed, 1036 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-094333-4z98/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-094333-4z98/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42364764348 for f7766208c2892ceca5e097a78d3758c99169ccd4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42364764348 for f7766208c2892ceca5e097a78d3758c99169ccd4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42364764348 for f7766208c2892ceca5e097a78d3758c99169ccd4)
- ci-result: `not a check` passed (Check results for f7766208c2892ceca5e097a78d3758c99169ccd4 come from the CI run https://github.com/doodla/athena/actions/runs/42364764348 (540s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All three pass, including the briefs.rs test that asserts READING_RULES has 6 entries.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib check_outcome_state_refuses_the_g6vy_misread`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7177:21
     |
7177 |         let error = check_outcome_state(&reading, G6VY_REQUEST)
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `delivery_verbs` in this scope
    --> crates/athena-core/src/reading.rs:7265:25
     |
7265 |             let verbs = delivery_verbs(request);
     |                         ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `delivery_verbs` in this scope
    --> crates/athena-core/src/reading.rs:7289:25
     |
7289 |             let verbs = delivery_verbs(request);
     |                         ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `request_words` in this scope
    --> crates/athena-core/src/reading.rs:7293:21
     |
7293 |         let words = request_words("Use crates.io v1.2. Share it");
     |                     ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7335:21
     |
7335 |         let error = check_outcome_state(&reading, "Publish the page. Deploy the site.")
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7366:9
     |
7366 |         check_outcome_state(&declined, request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7369:9
     |
7369 |         check_outcome_state(&asking, request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7371:9
     |
7371 |         check_outcome_state(&none, request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 8 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus delivery_verb_replay_over_the_cited_jobs`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.24s
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 1 test
test delivery_verb_replay_over_the_cited_jobs ... FAILED

failures:

---- delivery_verb_replay_over_the_cited_jobs stdout ----

thread 'delivery_verb_replay_over_the_cited_jobs' (28320) panicked at crates/athena-core/tests/refusal_corpus.rs:488:5:
assertion `left == right` failed: caught [] of 3
  left: []
 right: ["20261014-082328-g6vy"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    delivery_verb_replay_over_the_cited_jobs

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib delivery`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7177:21
     |
7177 |         let error = check_outcome_state(&reading, G6VY_REQUEST)
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `delivery_verbs` in this scope
    --> crates/athena-core/src/reading.rs:7265:25
     |
7265 |             let verbs = delivery_verbs(request);
     |                         ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `delivery_verbs` in this scope
    --> crates/athena-core/src/reading.rs:7289:25
     |
7289 |             let verbs = delivery_verbs(request);
     |                         ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `request_words` in this scope
    --> crates/athena-core/src/reading.rs:7293:21
     |
7293 |         let words = request_words("Use crates.io v1.2. Share it");
     |                     ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7335:21
     |
7335 |         let error = check_outcome_state(&reading, "Publish the page. Deploy the site.")
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7366:9
     |
7366 |         check_outcome_state(&declined, request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7369:9
     |
7369 |         check_outcome_state(&asking, request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:7371:9
     |
7371 |         check_outcome_state(&none, request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 8 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core --lib out_of_scope_and_incidental_words_do_not_satisfy_the_check`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core --test refusal_corpus`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-core --lib outcome_state_refusal_is_declared_intended`). Result: not run: only the first few distinct commands are run.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
