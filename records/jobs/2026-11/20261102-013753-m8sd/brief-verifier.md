# Job 20261102-013753-m8sd: verify "Check each criterion against D43 and append-only rules at reading"

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Athena builds criteria from the request text and does not test each one against the decisions that g
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: criterion-contradicts-athena-default-or-invariant (20261023-183245-dd7r)
> Cause: Athena builds criteria from the request text and does not test each one against the decisions that govern how criteria are judged. D43 says what kind of evidence a behavior criterion may use. Nothing makes the reading stage check a criterio...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261024-045710-yn3g (job 20261024-045548-7qht); prevention it proposed: Add a check at reading acceptance. For every criterion it reads the declared type (behavior or content) and the named evidence. It compares them with the rules from D43 and the other criterion-judging defaults. It rejects a reading, with th...
> Introspection 20261102-013736-k4jv (job 20261102-005332-8gkv); prevention it proposed: At reading acceptance, run a code check that scans criteria and deliverables for edits to append-only files (docs/decisions.md existing entries, records/) and refuses or rewrites them as appending a new decision that supersedes or amends D8...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reading stage fixes criteria from the request text, and no check compares them with the defaults that govern judging (D43 evidence rules, append-only decisions and records). Add one reading-acceptance check, registered as a `step!` in `refusing_check`, that refuses a behavior criterion whose evidence the verifier cannot produce and a criterion or deliverable that edits existing docs/decisions.md entries or records/ in place. Prior jobs 20261024-045548-7qht and 20261102-005332-8gkv are the two instances.

Assumptions:
- Goal and audience: Athena's reading gate, judged by maintainers and by the verifier that later judges each criterion; prevent criteria that contradict governing defaults from being accepted.
- Code survey: read crates/athena-core/src/reading/checks.rs (check_reading_acceptance and refusing_check run the checks in order through step! lines), crates/athena-core/src/reading/surveys.rs (check_criteria_evidence, the model for a criterion-text check), crates/athena-core/tests/refusal_corpus.rs (needs a case where the new check refuses) and crates/athena-context/src/briefs/reader.rs (reading rules list). The new check adds no record or data file, so check_record_file, AnyRecord and round-trip tests are not involved. The property is measured directly by calling the check on built readings.
- Existing code: extend crates/athena-core/src/reading/checks.rs and crates/athena-core/src/reading/surveys.rs; reuse the producer_evidence_phrase style of phrase matching; D43 in docs/decisions.md and the append-only guard in crates/athena-cli/src/decisions.rs define the rules mirrored.
- Rules enforced: (1) a behavior criterion must not require a command the verifier cannot run (cargo fmt without --check, git commit/push, cargo install, environment variables); (2) no criterion or modify deliverable may edit existing entries of docs/decisions.md in place or any file under records/; the error says to append a new decision that cites or amends the old one. Matching is on criterion text and deliverable paths.
- The new check name is added to the reader rules list in crates/athena-context/src/briefs/reader.rs; the reader prompt is at 1998 of 2000 tokens, so an existing rule line is shortened rather than grown. docs/decisions.md gets a new appended entry only (no in-place edits), with docs/decision-links.toml updated if the repo's link test requires it.
- Environment: none; no CI, push or records read. Only local cargo commands.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a unit test builds readings with (a) a behavior criterion requiring `cargo fmt` without --check, and (b) a criterion editing an existing D87 entry in place in docs/decisions.md; each is refused by the new check, naming the criterion; it fails with the check removed and passes with it.
2. [behavior] Run `cargo test -p athena-core`: tests show that criteria such as 'cargo test passes', 'cargo fmt --check passes' and a deliverable that only appends a new decision to docs/decisions.md are still accepted.
3. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: the corpus has a case where the new check is the refusing one, and the test that every step! line has such a case passes.
4. [content] Grep crates/athena-core/src/reading/checks.rs shows a `step!(...)` line for the new check inside refusing_check, with a doc comment that names the root cause (criteria are not tested against D43 and the append-only rules) and jobs 20261024-045548-7qht and 20261102-005332-8gkv.
5. [content] The change refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings.
6. [content] The reader rules in crates/athena-context/src/briefs/reader.rs list the new check by name, and the brief token budget test still passes.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked` and `cargo test --workspace --locked`; all pass, including the decisions append-only guard.
8. [content] CHANGELOG.md has an entry, and docs/decisions.md changes only by appending new text at the end.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing verify.rs D43 judging itself (already amended by D115 in job 20261024-045548-7qht; its files verify.rs, run.rs, decisions.md, decision-links.toml and CHANGELOG.md are kept as landed)
- Job 20261102-005332-8gkv: its candidate (gh client, landing, transport, watcher, records.rs, tests) was rejected and is dropped here; its blocker (in-place D87 edit) is the class prevented, not re-implemented
- Editing any existing decision entry or any file under records/
- Changes to athena.toml, defaults.toml or .github/workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria run tests that fail without the check and pass with it.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion included.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix placed at the reading gate where criteria are first fixed.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in crates/athena-core/src/reading.rs named `judging_rules_*` call `check_judging_rules` directly (for the error text) and `refusing_check` through the existing `gate_refuser` (reading.rs:7206) for the refusing check's name. Refused Behavior criteria: 'Run `cargo fmt`: the tree is formatted'; 'Run `cargo fmt --all` and `cargo test`: both pass'; '`cargo fmt` passes' (opening span plus result verb); 'Run `cargo --locked fmt`: formatted'; 'Run `RUSTFLAGS=-D warnings cargo test`: passes'; 'Run `git commit -am x`: a commit exists'; 'Run `rm -rf target`: clean'. Refused criteria of any kind: 'Merge refusals no longer fire D87: append a note to D87 in docs/decisions.md' (modelled on 620c657), 'Edit the existing D87 entry in docs/decisions.md in place', 'Rewrite D87 in docs/decisions.md to say X', 'Edit existing entries of docs/decisions.md', 'Correct records/jobs/x/outcome.json to say landed', 'Edit records/ files by hand'. A cross-criterion test (finding 1): one reading holding 'CHANGELOG.md records the D87 fix' and 'Edit the existing D87 entry in docs/decisions.md in place' is refused, naming the second criterion. Refused deliverables: any action on `records/jobs/x/outcome.json` and Delete of docs/decisions.md. Each refused case also asserts that its repaired text ('Run `cargo fmt --check`: ...', 'Append a new decision to docs/decisions.md that amends D87') is accepted. A split-out test for 'Run `git push`: the branch is published' first asserts refuser Some("check_environment_reach"), then adds the Environment assumption (CI_ENVIRONMENT, reading.rs:5113) and asserts the refuser is Some("check_judging_rules") and the error contains 'Environment assumption does not let the verifier run it' and 'git log' or 'git status'. Run: `cargo test -p athena-core judging_rules`. Observes: Every refused reading gives Some("check_judging_rules") from gate_refuser. The error contains 'criterion N', the quoted criterion text and the repair advice. The cross-criterion reading is refused. The git push error states that an Environment assumption does not let the verifier run the command.
  Fails without the behavior: Removing the `step!(check_judging_rules(reading));` line makes gate_refuser return None and the assertions fail. Removing the function makes the tests not compile. Re-adding a new-id exemption makes the cross-criterion test fail.
  Test seam: Production entry points: `check_judging_rules` in surveys.rs (step 1) and `refusing_check` in checks.rs:520 (step line added by step 2). Guard: capability must be Some, else Ok, like check_criteria_evidence at surveys.rs:92. Fixtures, which exist: `reading_with_criterion` (reading.rs:4461), `governed_reading` (:4558), `add_behavior` (:4705) and `gate_refuser` (:7206); the Reading is an argument, so there is no double. A `judging_reading(kind, texts, deliverables)` helper is added by step 5.
- Criterion 2: Tests call `check_judging_rules` and `gate_refuser` on accepted shapes: 'cargo test passes'; 'cargo test --locked passes'; 'cargo fmt --check passes'; 'Run `cargo --locked test`: passes' (finding 6); 'Run `cargo doc`: builds' (the grant is not judged; a known limit); 'Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`: all pass'; 'Run `git status --short`: no file under records/ appears'; 'records/ is not changed'; 'the status line shows counts read from records/'; 'docs/decisions.md changes only by appending new text at the end'; 'The log gains a new entry D175 that amends D87'; 'Append a new decision to docs/decisions.md that supersedes D87'; 'Update docs/decisions.md with a new decision' and 'Update docs/decisions.md and CHANGELOG.md' (finding 2); 'Run `cargo test -p athena-core format`: the step runs `cargo fmt --all` before `cargo fmt --all --check`'; '`cargo fmt --all` runs in the format step before `cargo fmt --all --check`'; 'Run `cargo test -p athena-git`: the helper runs `git commit` and `git merge --ff-only`'; 'CHANGELOG.md is updated to name D174'; 'the fix changes the behavior D43 describes'; 'The fix is recorded as D174 in docs/decisions.md'; 'docs/decisions.md is updated with a new decision D174'; 'This change adds decision D174'; 'Update docs/decisions.md to add D174'; 'A squash commit that changes records/ but names no job is refused by the records check'; 'land updates records/jobs/<id>/outcome.json with the landed state'; 'Fix the records/ loader in store.rs'; the exact text of this job's criterion 1; and the existing `Run `athena status`` and `Run `status`` texts. Accepted deliverables: `docs/decisions.md` (Modify) with CHANGELOG.md. Run: `cargo test -p athena-core judging_rules_accept`. Observes: check_judging_rules returns Ok for each text, and gate_refuser is not Some("check_judging_rules"). The existing accept tests that run `check_reading_acceptance(..).unwrap()` keep passing under `cargo test -p athena-core`.
  Fails without the behavior: Over-matching fails them: reading every backticked span, refusing every Modify of docs/decisions.md, reading a decision id near an -s or -ed form or the nouns 'fix', 'change', 'update', an OR in rule (iv), taking the second word as the cargo subcommand, or refusing any `records/` mention near a verb.
  Test seam: Same production functions and in-file fixtures as criterion 1; nothing is injected. The `judging_reading` helper is added by step 5.
- Criterion 3: `every_acceptance_check_has_a_refusing_corpus_case` (refusal_corpus.rs:2905) extracts each `step!(check_` line from `refusing_check` (via `function_body`, :2876) and requires a case in which that check refuses. Step 6 adds `judging_rules_cases()` to `cases()`, built with `case(..)` from `base_reading`. Refuse cases (6): `synthetic-class-of-7qht-cargo-fmt-without-check` (numbered `1.` parts); `modelled-on-620c657-append-note-to-d87` (star bullets; docs/decisions.md in `repo_files`); `criterion-edits-existing-d87-entry-in-place`; `other-criterion-records-d87-fix-does-not-excuse-in-place-edit`; `criterion-edits-records-file-imperative`; `deliverable-under-records-directory` (action Create, because check_boundary_and_repo_files, checks.rs:256-285, refuses a Modify or Delete of an absent path first; finding 4). Accept cases (18): `criterion-runs-cargo-fmt-check`; `criterion-cargo-test-passes`; `criterion-runs-cargo-locked-test`; `criterion-mentions-cargo-fmt-all-as-code-behavior`; `criterion-opens-with-cargo-fmt-span-as-code-behavior`; `criterion-appends-new-decision-amending-d87`; `deliverable-appends-a-new-decision`; `changelog-updated-to-name-d174`; `criterion-says-records-not-changed`; `fix-recorded-as-new-d174`; `decisions-updated-with-new-decision`; `update-decisions-with-new-decision-no-id`; `update-decisions-and-changelog`; `change-adds-decision-d174`; `update-decisions-to-add-d174`; `gate-sentence-changes-records`; `store-behavior-updates-records-outcome`; `fix-the-records-loader`. Input forms from the existing cases: `star-bullets`, `numbered-dot-parts`, `numbered-paren-parts-not-read-today` and `lettered-paren-parts-not-read-today` (the last two yield no criterion for this check). Ten tags are added to `INPUT_FORMS` (:90), array size 63 to 73: unrunnable-verifier-command, runnable-verifier-command, command-named-as-code-behavior, command-span-opens-criterion-as-code-behavior, in-place-edit-of-existing-decision, append-only-file-appended-only, decision-id-in-criterion-about-another-file, decision-id-recorded-as-new, records-path-edited, records-path-named-as-code-behavior. Behavior kind is set on criterion 0 as at :2022. I read the surrounding case code and ran none. A test `judging_rules_refusal_is_declared_intended` is added. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: Every case gives its expected outcome. `every_acceptance_check_has_a_refusing_corpus_case` passes with `check_judging_rules` among the extracted names. `corpus_covers_the_listed_input_forms` (:2826) passes with the ten tags used. `an_extra_check_with_no_case_is_reported` (:2926) and `no_check_bypasses_refusing_check` (:3210) still pass.
  Fails without the behavior: With the step line but no refusing case, the coverage test fails and names `check_judging_rules`. With the cases but no step line, the refuse cases get Accept and fail.
  Test seam: Production entry point `athena_core::reading::refusing_check`, called by `run(c)` in the corpus file (exists). Fixtures: `case`, `base_reading`, `deliverable` and the per-case `repo_files` map (exist). The new cases and tags are added by step 6.
- Criterion 4: A content test modelled on `scope_limit_doc_and_changelog_name_the_cause` (reading.rs:8904) reads checks.rs and surveys.rs with include_str!. It asserts that `refusing_check` has exactly one `step!(check_judging_rules(reading));` line and it is last (after check_reading_references); that the `check_reading_acceptance` doc links [`check_judging_rules`] and contains 'criteria are not tested against D43 and the append-only rules' with 20261024-045548-7qht and 20261102-005332-8gkv; that the `//` comment above the step names the same cause and jobs; and that the doc comment of `check_judging_rules` names the grant-independent rules and the follow-up. Also `Grep step!\(check_judging_rules crates/athena-core/src/reading/checks.rs`. Run: `cargo test -p athena-core judging_rules_doc_and_changelog_name_the_cause`. Observes: One step line in last position, and the doc text, comment text and both job ids found.
  Fails without the behavior: Without the step line, the doc clause or the job ids the assertions fail.
  Test seam: include_str! of the source files, as at reading.rs:8908 (exists). The test is added by step 5.
- Criterion 5: A test modelled on `records_measure_refusal_is_declared_intended` (refusal_corpus.rs:3138) calls `athena_core::reading::replay_declaration` on the 'expected 3% of newly refused readings' criterion text, on the doc comment of `check_judging_rules` and on the CHANGELOG item; each gives share Some(3). It also asserts that the changelog and D174 state 'higher refusal rate is intended' and call 3% an estimate, because the planner snapshot has no records to replay. The 3% is an estimate, not a measurement; the replay at verification measures it. Run: `cargo test -p athena-core --test refusal_corpus judging_rules_refusal_is_declared_intended`. Observes: A declared share of 3 in the criterion, the doc comment, the CHANGELOG item and D174.
  Fails without the behavior: If the doc, changelog or D174 omit the declaration, the replay holds the job over its newly-refused limit and this test fails.
  Test seam: Production function `replay_declaration` (exists, called at refusal_corpus.rs:3140). Doc text via include_str! as at :3152. The test is added by step 6.
- Criterion 6: Step 4 adds the 13th `READING_RULES` line in reader.rs (array type `[&str; 13]`) naming `check_judging_rules`, D43 and 'append'. The test at briefs.rs:3625-3713 expects 13 lines, adds the check name to its list and asserts the new line's content. It keeps the 220-byte line cap and the section bounds. Also run `cargo test -p athena-cli --test budgets` for `budget reader: 1998 of 2000`, unchanged because the list is outside BudgetSet::Reader (lib.rs:285). Run: `cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line`. Observes: The reader brief shows 13 rule lines, one naming check_judging_rules. Section bytes are within the bounds. The budgets test passes.
  Fails without the behavior: Without the new line the length and name assertions fail. Without shortening the old lines the section-size assertion fails.
  Test seam: Production entry point `reader_brief`, with the test's own `reader_inputs(&reading())` fixture (exists, used at briefs.rs:3626).
- Criterion 7: Run the checks Athena runs on the result: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. The test run includes the decisions guard (`the_decision_log_is_well_formed_and_only_appended_to`, decisions.rs:247), the decision-link tests (decisions.rs:413-414, :483), the mirror rule (mirrors.rs:354), buy_first and ontology, the budgets test, the boundary scan (xtask/tests/policy.rs), the steward and drain tests that call check_reading_text, and every test that calls check_reading_acceptance (reading.rs, replay.rs:55). Run: `cargo test --workspace --locked`. Observes: All pass. D174 is the next id after D173, has a Provenance bullet and lies after the pinned bytes. Its single 'Amends D43' line number equals the new [[link]] line.
  Fails without the behavior: A malformed D174, a wrong link line, a clippy warning, a format difference or a regression in an existing test fails the matching job.
  Test seam: Existing test suites. The one fixture change is moving the `Run `git push`` text out of the loop at reading.rs:5089-5116 into its own test (step 5).
- Criterion 8: The guard `check_log` (decisions.rs:218) confirms the first 359622 bytes of docs/decisions.md still hash to the pinned value and that D174 is the next id. A content check by Read and Grep: CHANGELOG.md has a new bullet at the top of `### Added` under `## [Unreleased]` naming `reading::check_judging_rules`, the root cause, both job ids, 'higher refusal rate is intended', 'expected 3% of newly refused readings' and the refused and accepted examples. `git diff docs/decisions.md` shows only added lines at the end. Run: `cargo test -p athena-cli decisions`. Observes: The pinned prefix is unchanged, the new entry follows D173, and the changelog item is present.
  Fails without the behavior: An in-place edit of any earlier entry changes the pinned prefix hash and fails the guard.
  Test seam: Existing guard `check_log` (decisions.rs:218, exists). The files are edited by step 7.

Planned files:

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 3% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base b057ae627bbd → candidate 3dc4127fcaaf. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   1 +
 crates/athena-context/src/briefs.rs        |  21 +-
 crates/athena-context/src/briefs/reader.rs |  15 +-
 crates/athena-core/src/reading.rs          | 426 +++++++++++++++++++++-
 crates/athena-core/src/reading/checks.rs   |   9 +-
 crates/athena-core/src/reading/surveys.rs  | 558 +++++++++++++++++++++++++++++
 crates/athena-core/tests/refusal_corpus.rs | 336 ++++++++++++++++-
 docs/decision-links.toml                   |   8 +
 docs/decisions.md                          |  49 +++
 9 files changed, 1407 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261102-013753-m8sd/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261102-013753-m8sd/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43266126634 for 3dc4127fcaaf22a8e0a323eefe3ca240164a09ad)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43266126634 for 3dc4127fcaaf22a8e0a323eefe3ca240164a09ad)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43266126634 for 3dc4127fcaaf22a8e0a323eefe3ca240164a09ad)
- ci-result: `not a check` passed (Check results for 3dc4127fcaaf22a8e0a323eefe3ca240164a09ad come from the CI run https://github.com/doodla/athena/actions/runs/43266126634 (909s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked` and `cargo test --workspace --locked`; all pass, including the decisions append-only guard.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core judging_rules`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/decision-links.toml, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling regex v1.13.1
   Compiling syn v2.0.119
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5160:9
     |
5160 |         check_judging_rules(&judging_reading(kind, texts, &[]))
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5169:9
     |
5169 |         check_judging_rules(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5255:25
     |
5255 |             let error = check_judging_rules(&reading).unwrap_err().to_string();
     |                         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5280:29
     |
5280 |                 let error = check_judging_rules(&reading).unwrap_err().to_string();
     |                             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5352:25
     |
5352 |             let error = check_judging_rules(&reading).unwrap_err().to_string();
     |                         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5360:21
     |
5360 |         let error = check_judging_rules(&reading).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5369:13
     |
5369 |             check_judging_rules(&reading).unwrap();
     |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5381:9
     |
5381 |         check_judging_rules(&reading).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 8 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core judging_rules_accept`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/decision-links.toml, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5160:9
     |
5160 |         check_judging_rules(&judging_reading(kind, texts, &[]))
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5169:9
     |
5169 |         check_judging_rules(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5255:25
     |
5255 |             let error = check_judging_rules(&reading).unwrap_err().to_string();
     |                         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5280:29
     |
5280 |                 let error = check_judging_rules(&reading).unwrap_err().to_string();
     |                             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5352:25
     |
5352 |             let error = check_judging_rules(&reading).unwrap_err().to_string();
     |                         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5360:21
     |
5360 |         let error = check_judging_rules(&reading).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5369:13
     |
5369 |             check_judging_rules(&reading).unwrap();
     |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_judging_rules` in this scope
    --> crates/athena-core/src/reading.rs:5381:9
     |
5381 |         check_judging_rules(&reading).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 8 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/decision-links.toml, docs/decisions.md.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 25 tests
test ambiguous_name_changelog_and_decision ... ok
test approve_request_has_the_generators_shapes ... ok
test defined_terms_decision_and_changelog ... ok
test defined_terms_replay_over_the_cited_jobs ... ok
test delivery_verb_replay_over_the_cited_jobs ... ok
test corpus_cases_get_the_expected_outcome ... FAILED
test an_extra_check_with_no_case_is_reported ... ok
test design_existing_code_refusal_is_declared_intended ... ok
test corpus_covers_the_listed_input_forms ... ok
test generated_request_has_the_generators_shapes ... ok
test environment_cache_scope_refusal_is_declared_intended ... ok
test judging_rules_refusal_is_declared_intended ... FAILED
test no_check_bypasses_refusing_check ... ok
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::an_impact_case_that_expects_the_wrong_outcome_is_reported ... ok
test environment_reach_refusal_is_declared_intended ... ok
test plan_side::impact_cases_get_the_expected_outcome ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined ... ok
test history_fixture_refusal_is_declared_intended ... ok
test records_measure_refusal_is_declared_intended ... ok
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok

failures:

---- corpus_cases_get_the_expected_outcome stdout ----

thread 'corpus_cases_get_the_expected_outcome' (28394) panicked at crates/athena-core/tests/refusal_corpus.rs:3065:5:
synthetic-class-of-7qht-cargo-fmt-without-check (synthetic: the class of job 20261024-045548-7qht, a behavior criterion whose command the verifier cannot run; the recorded criterion was not available): expected Refuse("check_judging_rules"), got accepted
modelled-on-620c657-append-note-to-d87 (modelled on the subject of commit 620c657 (merge refusals) and the D150 title; the recorded criterion of job 20261102-005332-8gkv was not available): expected Refuse("check_judging_rules"), got accepted
criterion-edits-existing-d87-entry-in-place (modelled on the subject of commit 620c657 (merge refusals) and the D150 title; the recorded criterion of job 20261102-005332-8gkv was not available): expected Refuse("check_judging_rules"), got accepted
other-criterion-records-d87-fix-does-not-excuse-in-place-edit (modelled on the subject of commit 620c657 (merge refusals) and the D150 title; the recorded criterion of job 20261102-005332-8gkv was not available): expected Refuse("check_judging_rules"), got accepted
criterion-edits-records-file-imperative (synthetic: a sentence about D43, the decision log or records/, written for check_judging_rules): expected Refuse("check_judging_rules"), got accepted
deliverable-under-records-directory (synthetic: a sentence about D43, the decision log or records/, written for check_judging_rules): expected Refuse("check_judging_rules"), got accepted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- judging_rules_refusal_is_declared_intended stdout ----

thread 'judging_rules_refusal_is_declared_intended' (28778) panicked at crates/athena-core/tests/refusal_corpus.rs:3488:5:
assertion failed: acceptance_check_names(reading_source()).contains(&"check_judging_rules".to_string())


failures:
    corpus_cases_get_the_expected_outcome
    judging_rules_refusal_is_declared_intended

test result: FAILED. 23 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core judging_rules_doc_and_changelog_name_the_cause`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core --test refusal_corpus judging_rules_refusal_is_declared_intended`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test --workspace --locked`). Result: not run: only the first few distinct commands are run.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test -p athena-cli decisions`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
