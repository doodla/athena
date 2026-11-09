# Job 20261017-205613-cjbq: verify "Validate each study against its own pin; tighten study admission"

## Request (verbatim)

> Validate every study against its own pinned source, so a second study can be checked at all. Today the kept validation test in crates/athena-cap-study/src/rules.rs hard-codes firstmate (STUDY_SHA, study_dir(), mounted_tree()), and study checks run cargo test with the job's pinned tree mounted, so a study of any other system would fail its own test check and nothing would validate it against its tree. The capability's conventions (enforcement tags, no placeholders, a "none" guard is never checked by code) exist only as firstmate tests, not in crates/athena-cap-study/prompts/producer.md. The boundary test in xtask/tests/policy.rs would also fail on a studied system's own words (Hermes Agent's work-handling source uses three of the words that test bans; read the list in xtask/tests/policy.rs and do not write those words anywhere outside a claim's quote).
> Plan: (1) The kept test checks the conventions for every studies/*/ directory, and runs full validation against the tree only for the study whose pin matches ATHENA_PIN_URL and ATHENA_PIN_SHA (set by the pinning change in the previous study job); with no tree mounted it checks conventions only. (2) State the conventions in the study producer prompt. (3) Admission refuses a study job's change that touches more than one studies/<id>/ directory, or anything outside it and CHANGELOG.md. (4) In the boundary test, exempt only text that quotes a studied system's source (the quote fields of a study's claims), never a study's own prose, and keep scanning machine-path fragments everywhere. Plant banned words in tests the way the_boundary_scanner_flags_a_planted_term does (split strings joined at run time), so no test file contains them.
> Acceptance:
> - cargo test: with no tree mounted, every study's conventions are checked and the repository passes; with firstmate mounted, the computed validation equals studies/firstmate-work-handling/validation.json.
> - cargo test on temporary copies: an untagged claim, a "none" guard checked by code, a source SHA that differs from the pin, and a pin.json for the mounted URL with a different SHA each fail and name the offending id.
> - cargo test: admission refuses a study change spanning two study directories, and one touching a file outside its directory and CHANGELOG.md.
> - cargo test: the boundary scan still flags a planted banned word in a study's own prose and in any file outside studies/, and a machine-path fragment inside studies/, but not a banned word inside a claim's quote of studied source.
> - clippy with -D warnings and fmt --check are clean; CHANGELOG.md has an entry; no file under studies/firstmate-work-handling/ changes.
> Out of scope: hiding one study from another's jobs, athena.toml, and the site.

## Reading (accepted by the user)

Generalize the kept study test in athena-cap-study so conventions are checked for every studies/*/ and full validation runs only for the study matching ATHENA_PIN_URL/SHA. Document conventions in the study producer prompt, make study admission refuse multi-directory or out-of-directory changes, and exempt only claim quotes in the xtask boundary test. Goal: a second study can be validated; audience is Athena's study jobs and verifiers.

Assumptions:
- Code survey: read crates/athena-cap-study/src/rules.rs (validate(), per-rule fns, kept test consts STUDY_SHA at ~1400 and study_dir() at ~1434, FixtureRepo, mounted tree reading) and crates/athena-cap-study/src/lib.rs (Capability::admit at line 112, the entry point for study admission), crates/athena-core/src/capability.rs (admit trait at line 121, ChangeList/PolicyViolation). Study records are data under studies/<id>/ (claims, pin.json, validation.json), not new files under crates/, so no new AnyRecord or check_record_file entry is needed; the existing validation.json comparison is the round-trip check. Validation is measured directly by running validate() on the parsed study and comparing to validation.json.
- Existing code: extend crates/athena-cap-study/src/rules.rs tests, crates/athena-cap-study/src/lib.rs admit(), crates/athena-cap-study/prompts/producer.md and xtask/tests/policy.rs; none are created by this job.
- Term Hermes: in this request, Hermes Agent is an example third-party studied system whose work-handling source contains words the xtask/tests/policy.rs boundary test bans; it is only a motivating example, not something this job adds or reads, and the ban list is not to be written anywhere outside a claim quote.
- Environment: no CI workflow, push or PR is touched; tests run locally with cargo, reading ATHENA_PIN_URL and ATHENA_PIN_SHA from the process environment, with no network.
- Consumers: none found for config; athena.toml and defaults.toml are not changed and are out of scope.
- Env-var matching uses studies/<id>/pin.json url and sha; with the variables unset or not matching any pin, only conventions are checked. A pin.json whose URL equals ATHENA_PIN_URL but with a different SHA fails naming the study id.
- Admission treats allowed paths as studies/<id>/** for exactly one id plus CHANGELOG.md; the expected newly refused share of past study readings is small.
- The boundary test plants banned terms as split strings joined at run time so no test file contains a banned word.

## Acceptance criteria

1. [behavior] Run cargo test: with no ATHENA_PIN_* set, the kept test in crates/athena-cap-study/src/rules.rs checks conventions for every studies/*/ directory and passes; with firstmate's tree mounted and the matching ATHENA_PIN_URL/ATHENA_PIN_SHA, the computed validation equals studies/firstmate-work-handling/validation.json.
2. [behavior] Run cargo test: tests on temporary copies of a study show an untagged claim, a "none" guard checked by code, a source SHA differing from the pin, and a pin.json for the mounted URL with a different SHA each fail, and the failure message names the offending id.
3. [behavior] Run cargo test: admit() in crates/athena-cap-study/src/lib.rs returns a violation for a change spanning two studies/<id>/ directories and for one touching a file outside its study directory and CHANGELOG.md, and none for a change inside one directory plus CHANGELOG.md. This refuses more study changes and the higher refusal rate is intended: expected 5% of newly refused readings.
4. [behavior] Run cargo test (xtask/tests/policy.rs): the boundary scan flags a planted banned word in a study's own prose and in any file outside studies/, and flags a machine-path fragment inside studies/, but not a banned word inside a claim's quote field; planted words are split strings joined at run time (Grep of xtask/tests/policy.rs finds no banned word).
5. [content] crates/athena-cap-study/prompts/producer.md states the conventions: enforcement tags, no placeholders, and that a "none" guard is never checked by code; the study producer prompt stays within its token budget (182 tokens of headroom).
6. [behavior] Run cargo clippy --all-targets -- -D warnings and cargo fmt --check: both are clean.
7. [content] CHANGELOG.md has a new entry describing the change; git diff shows no file under studies/firstmate-work-handling/ changed; athena.toml is unchanged.

## Expected deliverables

- crates/athena-cap-study/src/rules.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- xtask/tests/policy.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cap-study/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Hiding one study from another study's jobs
- athena.toml
- The site
- Any change under studies/firstmate-work-handling/

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the tests' behavior (failures naming ids, refusals) rather than existence of code.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Cargo test, clippy -D warnings and fmt --check criteria included.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Conventions and single-directory rule are enforced by tests and admission code, not prose only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the study crate tests. The kept test `every_study_follows_the_conventions_and_the_matching_one_validates` calls `check_studies(studies root, env pair read from ATHENA_PIN_URL/ATHENA_PIN_SHA, mount ../../pinned-source if it is a directory)` and prints one line per study: `<id>: fully validated` or `<id>: conventions only`. `the_study_matching_the_pin_validates_against_a_tree_rebuilt_from_its_citations` passes the study's own URL and SHA as the env pair and a `synthetic_tree` as the mount. It asserts `bad_citation` and `quote_mismatch` are empty on that tree, then route == FullyValidated, no problems, and computed validation equal to validation.json. `a_matching_pin_runs_validation_and_reports_a_tampered_record_or_quote` copies the study to a TempDir and calls `check_study` with the pin's own URL and SHA and a tree built from the untouched study. Case (i) flips one rule result in the copy's validation.json. Case (ii) changes one claim quote. Case (iii) makes the same quote change with the env pair unset. `a_second_study_directory_is_checked_too` builds a temp studies root with the firstmate copy and a renamed copy holding an untagged claim. Mounted-case command for a host with pinned-source/: `ATHENA_PIN_URL=<repository_url in studies/firstmate-work-handling/pin.json> ATHENA_PIN_SHA=<sha in the same file> cargo test -p athena-cap-study rules::tests -- --nocapture`. Run: `cargo test -p athena-cap-study rules::tests -- --nocapture`. Observes: With no ATHENA_PIN_* set, every studies/*/ passes the conventions and the output says 'conventions only'. The rebuilt-tree test passes with route FullyValidated. Case (i) returns route FullyValidated and a problem containing the study id and 'validation.json'. Case (ii) returns a problem containing the claim id and 'quote-mismatch'. Case (iii) returns ConventionsOnly and no problems. The second-directory test fails naming the second directory and its claim id. In the mounted case (host only) the printed line says 'firstmate-work-handling: fully validated'. This job's verifier has no pinned-source/ and cannot see that case.
  Fails without the behavior: If the matching branch returned early without calling `validate()`, cases (i) and (ii) would return no problems and the route assertion would fail. If the loop skipped a directory, the second-directory test would not name the bad claim. If `synthetic_tree` collided on shared single-line ranges, the pre-check would name the claim and fail.
  Test seam: Production entry points `validate()` and the 18 rules exist in rules.rs. New test-module items `check_study` (returning `Outcome`), `check_studies`, `Loaded::read(dir)`, `NoTree`, `synthetic_tree(&Loaded) -> TempDir` and the copy helper are added by plan steps 1-4. `MountedTree` exists (rules.rs:1412) and is generalised in step 1. Fixtures are the real studies/ directory and `tempfile::TempDir` copies. Root, env pair and mount are injected as arguments. The doubles hold no rule logic.
- Criterion 2: The test copies the first study directory into a TempDir, edits its JSON with serde_json, then calls `check_study`. (a) Set claims[0].text to untagged prose: a problem containing that claim id and 'enforcement'. (b) Set transitions[0].guard to "none" and guard_evaluated_by to "code": a problem naming the transition id. (c) Set claims[0].sources[0].sha to a different 40-hex SHA: a problem naming the claim id (via `wrong_sha`). (d) Leave the copy unchanged and pass an env pair with the pin's URL and a different SHA, with a temp dir as mount: a problem naming the study id and both SHAs. A control (unmodified copy, env unset) returns no problems. A second test, `the_word_unresolved_in_prose_is_not_a_placeholder` (run with `cargo test -p athena-cap-study rules::tests::the_word_unresolved_in_prose_is_not_a_placeholder`), covers finding 2: a claim text 'conflicts are left unresolved' passes; the placeholder 'UNRESOLVED' in claim text fails naming the claim id; 'Not Examined' in a finding detail fails naming the finding id; 'UNRESOLVED' in a rationale `authors_say` fails naming the card id; 'UNRESOLVED' and 'Not Examined' inside a claim's `sources[].quote` and a card's `sources[].quote` pass. Run: `cargo test -p athena-cap-study rules::tests::a_study_that_breaks_a_convention_or_its_pin_fails_naming_the_id`. Observes: Four asserts of the form `problems.iter().any(|p| p.contains(<id>))` plus the control returning empty. Failure strings begin with the offending id. The lowercase 'unresolved' passes, the placeholder in prose fails, and the same words inside a source quote pass.
  Fails without the behavior: If the tag check, the none-guard check, the SHA check or the pin-mismatch check were removed, the matching case would return an empty list and its assertion would fail. If a message omitted the id, `contains(id)` would fail. A case-insensitive UNRESOLVED check would fail the 'left unresolved' test. A raw-text scan would fail the quote-exempt cases.
  Test seam: `check_study(dir, pin_env, mount)` is added in step 3 and the placeholder walk in step 2. Fixtures are `tempfile::TempDir` copies of the real study, with `tempfile` added to dev-dependencies in step 7. The pin env pair and mount are injected as arguments. The conventions run the real `wrong_sha` rule with a `NoTree` lookup (added in step 1) that only supplies absent git answers.
- Criterion 3: Unit tests in lib.rs call `Study.admit(&ChangeList, &job_config())`. Input forms and cases: (1) studies/a/claims.json, studies/a/model.json and CHANGELOG.md give none. (2) studies/a/claims.json and studies/b/claims.json give one `study-one-directory` violation naming a and b. (3) studies/a/claims.json and docs/x.md give one `study-outside-directory` violation with path docs/x.md. (4) CHANGELOG.md alone gives none. (5) docs/CHANGELOG.md, bare studies/x, studies/a/../b/c, and a deleted file under a second directory are refused. (6) target/debug/x with studies/s1/claims.json gives exactly one violation, the existing build-output case. athena-core's refusal_corpus covers request-reading checks. A grep of crates/athena-core/tests/refusal_corpus.rs for admit, ChangeList and studies/ found no case for change-path admission, so no corpus case maps to these forms and none is added. This is a change-path admission rule, not a request-reading rule. `cargo test -p athena-core --test refusal_corpus` can still be run to show the reading corpus is unchanged; no result is cited here. Run: `cargo test -p athena-cap-study admit`. Observes: Violations returned by `admit()` equal the expected rule names and paths for every case. The existing `build_output_is_refused` still passes. The tightening is intended: expected 5% of study readings newly refused.
  Fails without the behavior: With the old admit, the two-directory and outside-file cases return zero violations, so the count and rule-name assertions fail.
  Test seam: `Study::admit` exists (crates/athena-cap-study/src/lib.rs:112). Callers reach it through the `Capability` trait at crates/athena-core/src/run/candidate.rs:164 and run/mod.rs:783. The seam is a directly built `ChangeList` and the existing `job_config()` fixture (lib.rs:132). The helper `study_dir_of(path) -> Option<&str>` is added in step 5 and is called by the real code, not faked.
- Criterion 4: New tests call the new `boundary_scan_file(path, text)` (step 6). Claims JSON is built with `serde_json::json!` and `to_string_pretty`, and one case is hand-written JSON with escaped characters. The planted word and machine-path fragment are built with `.concat()` from split parts, as in `the_boundary_scanner_flags_a_planted_term` (policy.rs:187). Cases: a banned word in a claim's source quote in studies/s/claims.json gives no hit. The same word in the claim's `text` gives a hit. The word in studies/s/findings.json or studies/s/notes.md gives a hit. The same claims JSON at docs/x/claims.json, crates/x/claims.json or studies/a/b/claims.json gives a hit. A machine-path fragment inside a claim quote gives a hit. A word in a rationale-card quote in studies/s/rationales.json gives a hit. A hand-written claim quote with `\/`, `\"` and `é` escapes around the planted word gives no hit. Invalid JSON in claims.json exempts nothing. `athena_refers_to_no_one_who_runs_it_or_where` calls the new function over tracked files. A Grep of xtask/tests/policy.rs for each banned word in the list there (not written in this plan) must find no match outside the existing `boundary_words()` list itself. Run: `cargo test -p xtask --test policy`. Observes: All new tests and the existing M19 tests pass: `the_boundary_scanner_flags_a_planted_term`, `the_producer_brief_lists_exactly_the_scanners_terms`, `the_admission_scan_finds_what_the_policy_scan_finds` and the tree scan.
  Fails without the behavior: If the exemption were missing, the claim-quote case would report a hit. If the exemption covered all of claims.json, the claim-text case would report none. If fragments were also exempted, the fragment-in-quote case would report none. If matching used a re-serialized text instead of the decoded literal, the escaped-quote case would report a hit.
  Test seam: `boundary_scan_file` is a new function in policy.rs (step 6). `boundary_scan`, `boundary_words`, `boundary_fragments` and `athena_files()` exist (policy.rs:88-162). Fixtures are in-memory strings with no filesystem. The real regex and serde_json do the matching (xtask has serde_json as a dependency and regex as a dev-dependency).
- Criterion 5: Read crates/athena-cap-study/prompts/producer.md and Grep it for `[mechanism]`, `[prose-only]`, the placeholder rule (UNRESOLVED, not examined) with the statement that quotes are exempt and stay verbatim, and the `none` guard rule. Run the budget tests: `system_facts_budgets_match_computed` passes with docs/system-facts.md unchanged, which proves the study producer set did not exceed the recorded 1043 (so it is below the 1200 budget with at least 157 tokens left). Run `cargo test -p athena-context`, which asserts the prompt lacks 'agent-reference' (crates/athena-context/src/lib.rs:652-659). Run: `cargo test -p athena-cli --test budgets`. Observes: `system_facts_budgets_match_computed` and `every_role_set_is_within_its_budget` pass. `git diff` shows no change to docs/system-facts.md. The prompt contains the three conventions.
  Fails without the behavior: If the prompt omitted a convention, the Grep finds no match. If the study set grew past 1043 the budgets test fails reporting the computed value, and the producer must shorten the prompt. If it passed 1200, `every_role_set_is_within_its_budget` fails.
  Test seam: The budget tests in crates/athena-cli/tests/budgets.rs exist (`computed_rows` takes the maximum over capabilities, lines 21-75). The prompt is a plain file included at compile time. No fake. Step 8 edits the prompt.
- Criterion 6: Run `cargo fmt --all` once after the edits, then clippy on the whole workspace, then `cargo fmt --all --check`. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Clippy finishes with no warnings and fmt --check exits 0 with no diff.
  Fails without the behavior: An unused import or variable, a type without Debug (missing_debug_implementations = warn), a needless clone or unformatted lines would make clippy or fmt --check exit non-zero.
  Test seam: The workspace's own tooling over the changed crates. `--all-targets` compiles the athena-cap-study tests and the xtask tests. No fake or fixture.
- Criterion 7: Read CHANGELOG.md `## [Unreleased]` for a new bullet under `### Changed`. It covers per-study validation, the prompt conventions, the placeholder check skipping source quotes, the new admission refusal, the boundary exemption for claim quotes, the same-URL-several-studies note, the precondition (the pinning change that sets ATHENA_PIN_*) and the degradation that with the pair unset only conventions are checked. Run `git diff --stat` and `git status`. Run: `git diff --stat`. Observes: The changed files are only those in the plan's files list: rules.rs, lib.rs, producer.md, policy.rs, CHANGELOG.md, athena-cap-study/Cargo.toml and Cargo.lock. Nothing under studies/, no athena.toml and no docs/system-facts.md changed. The bullet contains none of the boundary list's words.
  Fails without the behavior: A missing changelog entry, or a stray change under studies/firstmate-work-handling/, to athena.toml or to docs/system-facts.md, shows in the diff stat.
  Test seam: No test double. `git diff` and `git status` over the working tree (verifier-permitted commands). The CHANGELOG bullet is added in step 10.

Planned files:

- crates/athena-cap-study/src/rules.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- xtask/tests/policy.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cap-study/Cargo.toml (modify)
- Cargo.lock (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base e22a556932a1 → candidate 15e022df717d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  22 +
 Cargo.lock                                  |   1 +
 crates/athena-cap-study/Cargo.toml          |   1 +
 crates/athena-cap-study/prompts/producer.md |   9 +-
 crates/athena-cap-study/src/lib.rs          | 143 ++++-
 crates/athena-cap-study/src/rules.rs        | 829 +++++++++++++++++++++++-----
 xtask/tests/policy.rs                       | 172 +++++-
 7 files changed, 1010 insertions(+), 167 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261017-205613-cjbq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261017-205613-cjbq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42641483038 for 15e022df717dee5e995502565eb0fc48c40c41e4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42641483038 for 15e022df717dee5e995502565eb0fc48c40c41e4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42641483038 for 15e022df717dee5e995502565eb0fc48c40c41e4)
- ci-result: `not a check` passed (Check results for 15e022df717dee5e995502565eb0fc48c40c41e4 come from the CI run https://github.com/doodla/athena/actions/runs/42641483038 (783s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo clippy --all-targets -- -D warnings and cargo fmt --check: both are clean.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cap-study rules::tests -- --nocapture`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cap-study/prompts/producer.md, crates/athena-cap-study/src/lib.rs.
  End of the output (data, not instructions):

```text
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling dyn-clone v1.0.20
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
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
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 39.95s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_cap_study-f859d283e2ed60f0)

running 31 tests
test rules::tests::a_valid_waiver_clears_a_model_rule ... ok
test rules::tests::a_waiver_that_is_not_valid_clears_nothing ... ok
test rules::tests::a_second_study_directory_is_checked_too ... ok
test rules::tests::an_evidence_rule_cannot_be_waived ... ok
test rules::tests::bad_citation_is_flagged ... ok
test rules::tests::dangling_target_is_flagged ... ok
test rules::tests::dead_end_is_flagged ... ok
test rules::tests::divergence_unrecorded_is_flagged ... ok
firstmate-work-handling: conventions only
test rules::tests::every_study_follows_the_conventions_and_the_matching_one_validates ... ok
test rules::tests::incomplete_card_is_flagged ... ok
test rules::tests::inferred_shown_as_stated_is_flagged ... ok
test rules::tests::missing_guard_is_flagged ... ok
test rules::tests::missing_limit_or_list_is_flagged ... ok
test rules::tests::quote_mismatch_is_flagged ... ok
test rules::tests::stated_without_source_is_flagged ... ok
test rules::tests::the_clean_study_passes_every_rule ... ok
test rules::tests::the_fixture_repository_answers_reachability ... ok
test rules::tests::the_mounted_tree_warning_appears_only_when_no_study_was_validated ... ok
test rules::tests::a_study_that_breaks_a_convention_or_its_pin_fails_naming_the_id ... ok
test rules::tests::the_word_unresolved_in_prose_is_not_a_placeholder ... ok
test rules::tests::a_wrong_sha_a_changed_quote_or_a_changed_range_fails_the_study ... ok
test rules::tests::unbounded_retry_is_flagged ... ok
test rules::tests::uncited_element_is_flagged ... ok
test rules::tests::uncited_claim_is_flagged ... ok
test rules::tests::undeclared_counter_is_flagged ... ok
test rules::tests::unreachable_state_is_flagged ... ok
test rules::tests::unexercised_path_is_flagged ... ok
test rules::tests::unsupported_number_is_flagged ... ok
test rules::tests::wrong_sha_is_flagged ... ok
test rules::tests::a_matching_pin_runs_validation_and_reports_a_tampered_record_or_quote ... ok
test rules::tests::the_study_matching_the_pin_validates_against_a_tree_rebuilt_from_its_citations ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.13s

```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cap-study rules::tests::a_study_that_breaks_a_convention_or_its_pin_fails_naming_the_id`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cap-study/prompts/producer.md, crates/athena-cap-study/src/lib.rs.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_cap_study-f859d283e2ed60f0)

running 1 test
test rules::tests::a_study_that_breaks_a_convention_or_its_pin_fails_naming_the_id ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 39 filtered out; finished in 0.04s

```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cap-study admit`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cap-study/prompts/producer.md, crates/athena-cap-study/src/lib.rs.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_cap_study-f859d283e2ed60f0)

running 1 test
test tests::a_change_inside_one_study_directory_and_the_changelog_is_admitted ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 39 filtered out; finished in 0.01s

```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p xtask --test policy`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-cli --test budgets`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
