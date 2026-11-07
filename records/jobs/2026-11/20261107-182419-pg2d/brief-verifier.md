# Job 20261107-182419-pg2d: verify "Refuse readings that leave a structural request term undefined"

## Request (verbatim)

> try again on 20261107-110049-k8c2

## Reading (accepted by the user)

Retry of job 20261107-110049-k8c2 (failed in planning). Add check_defined_terms to refusing_check: it refuses a reading that does not define a structural term the request actually uses (ladder, rung, tier, layer) and give a behavior criterion for it. Quoted or cited mentions do not count as uses. The audience is Sri and the verifier judging readings.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (refusing_check at ~3228 with step! lines; request_words ~3507, which splits whole text and ignores quoting; check_outcome_state ~3680 and its doc tests ~8157-8430; replay_declaration ~4964) and crates/athena-core/tests/refusal_corpus.rs (CITED_REQUESTS, INPUT_FORMS, replay test) and the READING_RULES test in crates/athena-context/src/briefs.rs. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured by running the real refusing_check on request/reading pairs.
- Previous job 20261107-110049-k8c2 failed in planning with no candidate. Its findings are addressed: (1) major, matcher fired on quoted mentions: the new matcher ignores generated quote lines ('Introspection <id> ... prevention it proposed:', 'Cause:', 'Failure class:'), quoted text and parenthesised example lists, and counts only plain request prose; (2) minor, unmeasured 3%: the in-loop replay (loop_replay, replay_problem) is the measurement, and the declared figure is updated to the measured value if it sends work back; (3) minor, repo-defined terms: the doc and D164 say ladder and rung are refused until the reading restates them with a 'Term ladder:' line, which may cite the defining file, and that this is part of the intended refusal rate; (4) nit, step cross-references: corrected in the plan.
- The earlier deliverables and out_of_scope lists are kept unchanged. The earlier job changed no files, so there is no candidate work to keep or drop.
- Term list stays short (ladder, rung, tier, layer), matched as whole words in plain request prose. The job's own request is a corpus accept case.
- Governance files athena.toml, defaults.toml and workflows are not changed, and the reader prompt is not changed (it has 3 tokens of room).

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a refusal_corpus case built from the recorded request of job 20261014-174815-7bf8 (plain prose using 'ladder'), with a reading lacking a 'Term ladder:' definition, is refused by check_defined_terms; the same reading with a definition and a behavior criterion naming ladder is accepted.
2. [behavior] Run `cargo test -p athena-core`: a corpus case from this retry's own root-cause style request, which only quotes ladder, layer and tier inside 'Introspection ... prevention it proposed:' lines and a parenthesised list, is accepted by the real gate with a reading that has no Term lines; that input form is added to INPUT_FORMS.
3. [behavior] Run `cargo test -p athena-core`: requests using 'stepladder', 'delayer', or no structural word are accepted, so whole-word matching does not refuse ordinary requests.
4. [behavior] Run `cargo test -p athena-core`: the structural-term test fails when the step!(check_defined_terms(..)) line is removed from refusing_check and passes with it; the existing test requiring a refusing corpus case for every step! line passes.
5. [behavior] Outcome check: run `cargo test -p athena-core`: a replay of the cited requests (g6vy, 7bf8, 0wtc) shows g6vy caught by check_outcome_state, 7bf8 by check_defined_terms, and 0wtc by check_defined_terms only if its plain prose uses a structural word. Otherwise the test asserts it is outside the class.
6. [content] The doc comment on check_defined_terms in crates/athena-core/src/reading.rs names the root cause (readings are not checked against the outcome the user observes, so structural terms stay undefined), cites jobs 20261014-174815-7bf8 and 20261017-173508-0wtc, and states what is not covered: quoted mentions, terms outside the list, and the 0wtc Hermes term.
7. [content] The change refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings. The doc comment on check_defined_terms and D164 in docs/decisions.md say so, state that repo-defined terms (ladder, rung) are also refused until restated, and give the loop-replay measured share if it differs. The comment on check_outcome_state no longer says the other two jobs are outside any check.
8. [content] CHANGELOG.md has an entry naming the root cause and the new check_defined_terms; docs/decisions.md has D164 with the alternatives considered; the READING_RULES test in crates/athena-context/src/briefs.rs is updated for the new check.
9. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Re-doing the explainer site, pages.yml or any xtask changes from job 20261014-082328-g6vy (dropped)
- Changing the steward, roles or ladder work of job 20261014-174815-7bf8 (kept as landed)
- Editing docs/ontology.toml, its test, or docs/decisions.md D109 from job 20261017-173508-0wtc, or fixing its Hermes finding
- Making the check read docs/ontology.toml, or wiring the ontology into agent contexts
- Changes to athena.toml, defaults.toml, workflows, or prompt text (the reader prompt has no token room)
- Changing check_outcome_state or check_governance_coverage rules beyond their doc comments

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the real gate on request/reading pairs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix is in reading acceptance, with the cause named in docs.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A step! line in refusing_check enforces the rule.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Corpus replay of the cited jobs' requests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, one short word list.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the decision record explain the rule.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the real gate `refusing_check` on two corpus cases built from the recorded 7bf8 request (shared const `REQUEST_7BF8`, extracted from `CITED_REQUESTS`, refusal_corpus.rs:456). Case A `structural-term-ladder-undefined-in-7bf8-request`: `base_reading`, one deliverable, no Term line, expects a refusal by check_defined_terms. Case B `structural-term-ladder-defined-and-named-by-a-behavior-criterion-accepted`: adds an assumption `Term ladder: ...` citing docs/guide.md (in `repo_files()`) and a behavior criterion naming the ladder via `with_behavior`; expects acceptance. A reading.rs unit test also checks `Term ladders: ...` defines ladder. Run: `cargo test -p athena-core`. Observes: Case A gives Some(("check_defined_terms", err)); the error starts 'the reading was refused because', names ladder and shows the `Term ladder:` repair. Case B gives None. `corpus_cases_get_the_expected_outcome` and `the_wrapper_agrees_with_refusing_check_on_every_case` pass. I read the test names and case structure and ran nothing.
  Fails without the behavior: Without `check_defined_terms` and its step line, Case A is accepted and `corpus_cases_get_the_expected_outcome` fails. If the check ignored the Term line, Case B is refused.
  Test seam: Production entry point `athena_core::reading::refusing_check` (exists, reading.rs:3228), reached through `check_reading_acceptance`. Fixture: the corpus `Case` struct, `base_reading` and `with_behavior`, injected through `cases()` (exist, refusal_corpus.rs:128-271). Step 4 adds the shared const and the two cases. Nothing is doubled.
- Criterion 2: Run the real gate on requests built by Athena's real generators. (a) `generated_root_cause_request(Some(sentence))` calls `hestia::root_cause_request` with a constructed `RepeatedCause` and `Records` of two `IntrospectionRecord` values whose class, cause and prevention mention ladder, layer and tier; the spliced sentence adds a double-quoted word, a single-quoted word and '(ladder, layer, tier)'. Case `structural-term-quoted-or-cited-mentions-are-accepted`: `base_reading`, no Term lines, `referenced` as in `root-cause-cites-two-jobs-without-relying`, 3 bullets, expects acceptance. (b) `generated_approval_request(title, source_text)` calls `work_decisions::approve_request` with a `WorkItem`, `Records::default()` and a temporary git repository, the source file holding 'The ladder has five rungs in this layer.'. Case `structural-term-approve-request-generated-text-is-accepted` expects acceptance with no Term lines. Companions that expect refusal by check_defined_terms: `structural-term-plain-use-beside-quoted-mentions-is-refused`, `structural-term-approve-request-title-use-is-refused` (title 'Fix the ladder ordering'), `structural-term-possessive-use-is-refused`, and a hand-written anchorless 'Rung of the requirements ladder: x.' with '## Source' (unit test). Shapes tests `generated_request_has_the_generators_shapes` and `approve_request_has_the_generators_shapes` pin the anchors and fixed lines. A case runs the public `steward::blind_spot_change` request through the gate and expects acceptance. INPUT_FORMS gains `structural-term-plain-prose`, `structural-term-quoted-or-cited`, `structural-term-defined`, `structural-term-generated-approval` (34 becomes 38). The athena-cli approval tests (work_decisions.rs:96, drain.rs:5328, 6210, 6276) are the end-to-end regression run. Run: `cargo test -p athena-core && cargo test -p athena-cli`. Observes: Both generated, quoted-only requests give None from `refusing_check` with a reading that has no Term lines. The companions give Some("check_defined_terms"). The shapes tests pass, so the accepted cases go through the anchor-gated skips. `corpus_covers_the_listed_input_forms` passes because each new tag has a case. In athena-cli, `approving_queues_a_job_whose_request_carries_the_cited_content` still ends in JobState::Proposed. I read the helpers and generators and ran nothing.
  Fails without the behavior: If quoted or generated mentions counted as uses, both accepted cases are refused and the athena-cli approval tests fail on `approve_work(..).unwrap()`. If a skip were blanket, the plain-use and title companions would be accepted and fail. If bare-prefix skipping came back, the hand-written Introspection, Cause and Rung unit cases fail. If hestia.rs or work_decisions.rs reworded their lines, the shapes tests fail.
  Test seam: `refusing_check` (exists). `hestia::root_cause_request`, `hestia::RepeatedCause`, `store::Records` and `athena_data::records::IntrospectionRecord` are public and exist (hestia tests build the record at hestia.rs:2842-2867). `work_decisions::approve_request` is public (exists, work_decisions.rs:279); `athena_git::Git` and `tempfile` are normal dependencies of athena-core (exist). Step 4 adds both helpers, the cases and the tags. The generators and the gate are real; nothing is doubled. The athena-cli tests use the existing fake reader and `Fixture` (exist).
- Criterion 3: Run the real gate on `structural-term-inside-other-words-is-accepted` ('Rename the stepladder helper and remove the delayer module.', existing form `trigger-word-inside-other-word`). `defined_terms_matches_whole_words_only` also checks a request with no structural word, `ladder.md`, `layer_count`, the path `crates/x/layer/mod.rs`, and a quoted 'layer'. Positive controls: 'the layers' and 'rungs' are refused when undefined. Run: `cargo test -p athena-core`. Observes: Every ordinary request gives None from `refusing_check` and Ok from `check_defined_terms`; the positive controls give Err naming the terms.
  Fails without the behavior: A substring matcher would refuse the stepladder and delayer case. A tokenizer that split on `_` or `/` would refuse the identifier and path requests. A check that refused requests with no structural word would fail existing accepted corpus cases such as `steward-three-dash-bullets`.
  Test seam: `refusing_check` (exists) for the corpus case. `check_defined_terms` is a new `pub fn` (step 1), called directly in the unit test with `governed_reading` (exists in reading.rs tests). Nothing is doubled.
- Criterion 4: `defined_terms_refusal_needs_the_step_line` (reading.rs, shaped like `history_fixture_refusal_needs_the_step_line`, reading.rs:10552-10567) asserts `refuser_for(reading, request) == Some("check_defined_terms")` for an undefined-ladder reading and that the `refusing_check` body contains `step!(check_defined_terms(reading, request));`. `defined_terms_has_one_step_line_after_environment_reach` pins the position like `environment_reach_has_one_step_line`. The existing `every_acceptance_check_has_a_refusing_corpus_case` requires a refusing corpus case for every step line and passes through Case A of criterion 1. Run: `cargo test -p athena-core`. Observes: The gate refuses the undefined-ladder reading under the name `check_defined_terms`. The body holds exactly one such step line. Every step line has a refusing corpus case. I read the existing tests and ran nothing.
  Fails without the behavior: With the `step!(check_defined_terms(..))` line removed, `refusing_check` returns None for the undefined-ladder reading, so `defined_terms_refusal_needs_the_step_line`, `defined_terms_has_one_step_line_after_environment_reach` and the corpus refusing case all fail. With the line present but no refusing corpus case, `every_acceptance_check_has_a_refusing_corpus_case` fails.
  Test seam: `refusing_check` (exists). The new helper `refuser_for(reading, request)` in reading.rs tests (step 3) takes the request, because `gate_refuser` fixes it. The `refusing_check` body is read with `include_str!("reading.rs")` (pattern exists). Nothing is doubled.
- Criterion 5: Edit `delivery_verb_replay_over_the_cited_jobs`: for each job in `CITED_REQUESTS` (refusal_corpus.rs:456), run `refusing_check` over `base_reading` and `repo_files()`. g6vy gives check_outcome_state, 7bf8 gives check_defined_terms, 0wtc gives None (its recorded text, refusal_corpus.rs:467-472, holds none of the four words; a word-scan over its plain prose asserts that, so a changed text fails loudly). Add `defined_terms_replay_over_the_cited_jobs`: the jobs caught by check_defined_terms are exactly [7bf8], and CHANGELOG.md contains 'check_defined_terms covers 1 of the 3 cited jobs' and the three job ids but not '3 of the 3 cited jobs'. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: Three real `refusing_check` runs on the cited request texts give check_outcome_state, check_defined_terms and None.
  Fails without the behavior: Without the check, 7bf8 is accepted and the assertion fails. If the check were too wide, 0wtc would be refused and the `is_none()` assertion fails. If check_outcome_state changed, the g6vy assertion fails.
  Test seam: `refusing_check` with `base_reading` and `repo_files()` (exist) over `CITED_REQUESTS` (exists). Step 6 changes the expected outcomes and adds the second test. CHANGELOG.md is read with `include_str!`, as the existing test does.
- Criterion 6: Content test `defined_terms_doc_and_changelog_name_the_cause` in reading.rs, shaped like `history_fixture_doc_and_changelog_name_the_cause`. It takes the `///` block above `pub fn check_defined_terms(` (one paragraph, no blank `///` lines), collapses white space, and requires: 'readings are not checked against the outcome the user observes', 'structural terms', '20261014-174815-7bf8', '20261017-173508-0wtc', 'quoted', 'terms outside the list', 'Hermes', and 'no deliverable'. Run: `cargo test -p athena-core defined_terms_doc_and_changelog_name_the_cause`. Observes: Each needle is found in the doc comment of the new function.
  Fails without the behavior: If the doc comment lacks the root-cause sentence, a job id, a not-covered item or the skip condition, the test fails with 'the doc comment lacks ...'.
  Test seam: Source text via `include_str!("reading.rs")` (the pattern exists in `check_outcome_state_doc_names_root_cause_and_scope`). The comment is written in step 2 and the test in step 3.
- Criterion 7: Test `defined_terms_refusal_is_declared_intended` in reading.rs, shaped like `outcome_state_refusal_is_declared_intended`. `replay_declaration` on 'expected 3% of newly refused readings' gives Some(3). The doc comment of `check_defined_terms`, CHANGELOG.md and D164 each contain 'This check refuses more readings; the higher refusal rate is intended.', 'estimate', that ladder and rung are refused until restated, that plain uses of tier (priority tiers) and layer (a caching layer) are refused too, that the 3% covers hand-written requests only, and that the measured share is recorded in the job's replay record at verification. CHANGELOG and D164 also contain 'held until' with 'athena go' for queued requests read before the change (finding 1). D164 also contains 'decide whether to narrow', 'approve_request', 'accept_request', and the follow-up 'shared delimiter' (finding 3). The check_outcome_state doc must contain 'check_defined_terms' and not 'which does not catch them', and keep 'covers 1 of the 3 cited jobs', the three job ids and 'outside the check'. The corpus case `structural-term-priority-tier-is-refused` pins the known false refusal. There is no needle for a measured figure; if a held send-back reports one, the producer adds it as 'measured by the loop replay' to the doc comment, CHANGELOG and D164 and the tests still pass. Run: `cargo test -p athena-core`. Observes: The declared figure parses to 3. Each document carries the sentence, the estimate, the false-refusal, scope, generators, recording, held-queue and narrowing statements. The old 'outside any check' wording is gone. The priority-tier and caching-layer requests are refused.
  Fails without the behavior: If the doc comment, CHANGELOG or D164 omits the sentence, the limits, the generators list, the recording path, the held-queue note or the narrowing question, the test fails. If the check_outcome_state doc still says the other two jobs are outside any check, the guard fails. If a later change narrowed 'tier' or 'layer', the pinning case fails.
  Test seam: `replay_declaration` (exists, reading.rs:4964). Source text via `include_str!` of reading.rs, CHANGELOG.md and docs/decisions.md. `refusing_check` for the pinning case. The docs are written in steps 2 and 8, the tests in steps 3 and 4.
- Criterion 8: Update the READING_RULES test `the_reader_brief_names_each_reading_rule_in_one_short_line` in briefs.rs: length 7 becomes 8, `check_defined_terms` joins the list of checks that need a rule line, a line must name `Term ladder:` and the check, and the 'seven rule lines' comment and `section.len()` bound are set from the printed length. The new line stays at or under 220 characters. In refusal_corpus.rs, `defined_terms_decision_and_changelog` requires in CHANGELOG.md and docs/decisions.md: '## D164 ·', 'check_defined_terms', '**Alternatives:**', a Provenance bullet, and the root-cause sentence. The mirror rule 'decision entries' (mirrors.rs about 346) also checks D164. Run: `cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line && cargo test -p athena-core --test refusal_corpus defined_terms_decision_and_changelog`. Observes: The brief lists 8 rule lines of at most 220 characters, one naming the check. CHANGELOG.md has the entry and docs/decisions.md has D164 with its alternatives.
  Fails without the behavior: Without the eighth rule line, the length assertion and the 'no rule for check_defined_terms' assertion fail. Without D164 or the CHANGELOG entry, the content test fails.
  Test seam: `reader_brief` with `reader_inputs` in briefs.rs tests (exists). The READING_RULES array is changed in step 7. Markdown is read with `include_str!`, as the existing corpus tests do.
- Criterion 9: Run the three Athena checks over the whole workspace, so the athena-cli integration tests (approve-work and drain tests that reach the gate through `approve_request`, and the drain tests over `queued_work`) and athena-context are covered. No landing, history or attribution rule changes, so no commit-shape table or branch fixture applies. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0.
  Fails without the behavior: Any formatting, clippy or test failure fails the run. Without the approve_request skip, `approving_queues_a_job_whose_request_carries_the_cited_content` and the three drain.rs approval tests fail. Without the test updates, the old 7bf8 `is_none()` assertion, the READING_RULES length and size assertions and the check_outcome_state doc needles fail.
  Test seam: The cargo commands. Nothing is doubled.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 3% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base f729deb4af20 → candidate 44b72d3278b7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  20 +
 crates/athena-context/src/briefs.rs        |  14 +-
 crates/athena-core/src/reading.rs          | 661 ++++++++++++++++++++++++++++-
 crates/athena-core/tests/refusal_corpus.rs | 551 +++++++++++++++++++++++-
 docs/decisions.md                          |  73 ++++
 5 files changed, 1298 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-182419-pg2d/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-182419-pg2d/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42406902067 for 44b72d3278b76be16b23818302cd046d65094b43)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42406902067 for 44b72d3278b76be16b23818302cd046d65094b43)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42406902067 for 44b72d3278b76be16b23818302cd046d65094b43)
- ci-result: `not a check` passed (Check results for 44b72d3278b76be16b23818302cd046d65094b43 come from the CI run https://github.com/doodla/athena/actions/runs/42406902067 (601s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 3, 4, 7 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling bitflags v2.13.2
   Compiling jiff v0.2.38
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
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
error[E0425]: cannot find function `check_defined_terms` in this scope
     --> crates/athena-core/src/reading.rs:10300:9
      |
10300 |         check_defined_terms(&terms_reading(assumptions), request)
      |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_defined_terms` in this scope
     --> crates/athena-core/src/reading.rs:10496:9
      |
10496 |         check_defined_terms(&declined, request).unwrap();
      |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_defined_terms` in this scope
     --> crates/athena-core/src/reading.rs:10499:9
      |
10499 |         check_defined_terms(&asking, request).unwrap();
      |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_defined_terms` in this scope
     --> crates/athena-core/src/reading.rs:10502:9
      |
10502 |         check_defined_terms(&none, request).unwrap();
      |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 4 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core && cargo test -p athena-cli`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined ... FAILED
test an_extra_check_with_no_case_is_reported ... ok
test history_fixture_refusal_is_declared_intended ... ok
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok

failures:

---- defined_terms_decision_and_changelog stdout ----

thread 'defined_terms_decision_and_changelog' (6140) panicked at crates/athena-core/tests/refusal_corpus.rs:621:10:
docs/decisions.md holds D164
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- defined_terms_replay_over_the_cited_jobs stdout ----

thread 'defined_terms_replay_over_the_cited_jobs' (6142) panicked at crates/athena-core/tests/refusal_corpus.rs:574:5:
assertion `left == right` failed: caught [] of 3
  left: []
 right: ["20261014-174815-7bf8"]

---- delivery_verb_replay_over_the_cited_jobs stdout ----

thread 'delivery_verb_replay_over_the_cited_jobs' (6147) panicked at crates/athena-core/tests/refusal_corpus.rs:533:41:
the ladder request is refused

---- corpus_cases_get_the_expected_outcome stdout ----

thread 'corpus_cases_get_the_expected_outcome' (6027) panicked at crates/athena-core/tests/refusal_corpus.rs:1862:5:
structural-term-ladder-undefined-in-7bf8-request (job 20261014-174815-7bf8, its recorded request, verbatim; the reading is the file-only reading `base_reading` builds): expected Refuse("check_defined_terms"), got accepted
structural-term-plain-use-beside-quoted-mentions-is-refused (this retry, job 20261107-182419-pg2d; text built by the real generator): expected Refuse("check_defined_terms"), got accepted
structural-term-possessive-use-is-refused (the request in the brief of job 20261107-182419-pg2d; rebuilt, not a recorded request): expected Refuse("check_defined_terms"), got accepted
structural-term-priority-tier-is-refused (a known, intended false refusal: Athena's priority tiers are a technical use of the word; rebuilt, not a recorded request): expected Refuse("check_defined_terms"), got accepted
structural-term-caching-layer-is-refused (a known, intended false refusal: a caching layer is a technical use of the word; rebuilt, not a recorded request): expected Refuse("check_defined_terms"), got accepted
structural-term-approve-request-title-use-is-refused (this retry, job 20261107-182419-pg2d; text built by the real generator): expected Refuse("check_defined_terms"), got accepted

---- queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined stdout ----

thread 'queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined' (6250) panicked at crates/athena-core/tests/refusal_corpus.rs:1049:5:
assertion `left == right` failed: [("20261016-120000-aaaa", true), ("20261016-120001-bbbb", true), ("20261016-120002-cccc", true)]
  left: [("20261016-120000-aaaa", true), ("20261016-120001-bbbb", true), ("20261016-120002-cccc", true)]
 right: [("20261016-120001-bbbb", true), ("20261016-120002-cccc", true)]


failures:
    corpus_cases_get_the_expected_outcome
    defined_terms_decision_and_changelog
    defined_terms_replay_over_the_cited_jobs
    delivery_verb_replay_over_the_cited_jobs
    queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined

test result: FAILED. 14 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core defined_terms_doc_and_changelog_name_the_cause`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `check_defined_terms` in this scope
     --> crates/athena-core/src/reading.rs:10300:9
      |
10300 |         check_defined_terms(&terms_reading(assumptions), request)
      |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_defined_terms` in this scope
     --> crates/athena-core/src/reading.rs:10496:9
      |
10496 |         check_defined_terms(&declined, request).unwrap();
      |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_defined_terms` in this scope
     --> crates/athena-core/src/reading.rs:10499:9
      |
10499 |         check_defined_terms(&asking, request).unwrap();
      |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_defined_terms` in this scope
     --> crates/athena-core/src/reading.rs:10502:9
      |
10502 |         check_defined_terms(&none, request).unwrap();
      |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 4 previous errors
```
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line && cargo test -p athena-core --test refusal_corpus defined_terms_decision_and_changelog`). Result: not run: not a plain cargo test command.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
