# Job 20261112-194023-nyf7: verify "Show the reader the rules that judge criteria, in its brief"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Athena builds criteria from the request text and does not test each one against the decisions that g
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: criterion-contradicts-athena-default-or-invariant (20261023-183245-dd7r)
> Cause: Athena builds criteria from the request text and does not test each one against the decisions that govern how criteria are judged. D43 says what kind of evidence a behavior criterion may use. Nothing makes the reading stage check a criterio...
> Seen 4 time(s) in 4 jobs. The introspections that record it:
> Introspection 20261024-045710-yn3g (job 20261024-045548-7qht); prevention it proposed: Add a check at reading acceptance. For every criterion it reads the declared type (behavior or content) and the named evidence. It compares them with the rules from D43 and the other criterion-judging defaults. It rejects a reading, with th...
> Introspection 20261102-013736-k4jv (job 20261102-005332-8gkv); prevention it proposed: At reading acceptance, run a code check that scans criteria and deliverables for edits to append-only files (docs/decisions.md existing entries, records/) and refuses or rewrites them as appending a new decision that supersedes or amends D8...
> Introspection 20261106-062030-6pqw (job 20261106-041052-wzy7); prevention it proposed: Add a check at reading acceptance. Any criterion that names an existing line of docs/decisions.md for edit or annotation is refused, with a message saying to append a new superseding entry that cites the old one. The check should read the s...
> Introspection 20261112-135855-a39s (job 20261030-013721-4gwn); prevention it proposed: Add a code check run when the reading is accepted, before planning. It compares every criterion that forbids touching a file against the default-required deliverables (CHANGELOG.md from the changelog default) and the admission rules in admi...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader brief never states the decisions that govern how criteria are judged, so readings write criteria that contradict them. Examples: a behavior criterion citing evidence D43 won't accept, or an in-place edit to the append-only docs/decisions.md. Fix by adding a "Rules that judge criteria" section, built from live facts, to the reader brief in crates/athena-context/src/briefs/reader.rs, with a test. No new refusal rule. Prior jobs 20261024-045548-7qht, 20261102-005332-8gkv and 20261106-041052-wzy7 are addressed as noted in the assumptions.

Assumptions:
- Goal and audience: the reading stage (the reader agent) is given the facts that decide whether a criterion can be met. Maintainers judge the result by whether readings stop producing contradictory criteria.
- Code survey: read crates/athena-context/src/briefs/reader.rs (reference_and_rules_section, verifier_can_run_section, READING_RULES, CHECKABLE_CRITERIA_RULE), crates/athena-core/src/reading/checks.rs and reading/surveys.rs (check_criteria_evidence, check_governance_coverage), and crates/athena-core/src/mirrors.rs (append_only_problem). The entry point is the reader brief builder. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are untouched. The property is measured directly by building the brief and asserting its text.
- Existing code: extends crates/athena-context/src/briefs/reader.rs and reuses the READING_RULES pattern. It states D43 as implemented in crates/athena-core/src/verify.rs, the append-only rule from crates/athena-core/src/mirrors.rs, and the governance and default-deliverable rules from crates/athena-core/src/admission.rs.
- Prior-job handling: job 20261024-045548-7qht is landed and passed. Its D43/D115 change in verify.rs, run.rs, decisions.md, decision-links.toml and CHANGELOG.md is kept as is, and its two nits are not reopened. The reader brief should state the D115 rule, that a recorded passing check on the same tree counts as evidence. Job 20261102-005332-8gkv was rejected for editing D87 in place; the fix is to tell the reader that decision entries are append-only, and its gh/watcher/records work is out of scope here and not redone. Job 20261106-041052-wzy7 was rejected because a criterion demanded annotating D10 in place; this change keeps its retry-on-stale work out of scope and tells the reader to write 'record in a new appended decision plus a decision-links.toml link' instead. This change touches none of those jobs' candidate files, so they stay dropped from this change.
- Expected refusal rate: 0% of past accepted readings are newly refused, because no refusal rule is added. The change only adds information to the brief.
- The brief section is generated from the rule constants or text already used by the checks where possible, so it cannot drift from them. Otherwise a test pins it to the check's wording.
- CHANGELOG.md is touched because the changelog default requires an entry. docs/decisions.md gets a new appended decision, never an edit to an existing entry.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-context`. A new test builds the reader brief and asserts it contains a 'Rules that judge criteria' section that states D43's evidence rule (including that a recorded passing Athena check on the same tree counts), that docs/decisions.md and records/ entries are append-only (new decision, not an in-place edit), and that the default-required CHANGELOG.md deliverable cannot be forbidden. The test fails on the base commit and passes with the change.
2. [behavior] A second new test in crates/athena-context shows the section's rule text matches the constants or wording used by the enforcing code, so it fails if the enforcing rule changes and the brief does not. It covers the cause class, not one job.
3. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`; all pass for the workspace.
4. [content] crates/athena-context/src/briefs/reader.rs adds the section to reference_and_rules_section or a sibling function. A diff grep shows no new refusal function in crates/athena-core/src/reading/.
5. [content] A criterion states 'expected 0% of newly refused readings': no refusal rule is added, and the brief gives the information instead.
6. [content] CHANGELOG.md has an entry that names the root cause (the reader was never shown the rules that judge criteria) and states only what the diff does. docs/decisions.md ends with a new appended decision, and no earlier byte changed. docs/decision-links.toml gets any needed link.
7. [content] The reader prompt files are not edited. Prompt token budgets are unchanged, since the facts are added in code.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Adding any new refusal or check at reading acceptance.
- Editing existing entries of docs/decisions.md or anything under records/.
- Redoing or changing the candidates of jobs 20261024-045548-7qht, 20261102-005332-8gkv and 20261106-041052-wzy7, or their files: verify.rs, run.rs, gh/*, watcher.rs, records.rs, land.rs, main.rs and README.md.
- Changes to athena.toml, defaults.toml, .github/workflows, or prompt files.
- Changing how the verifier judges D43.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the brief section; fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the brief text by running a test, not that the wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the reader brief, where the missing information should be.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and decision entries are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A unit test in the `tests` module of crates/athena-context/src/briefs.rs builds `reader_brief(&reader_inputs(&reading()))`. It takes the text between `# Rules that judge criteria` and the next `\n# ` and asserts the following. (a) D43: the line begins `- D43` and quotes 'must cite commands that ran (D43)'. (b) D115: the line begins `- D115`, quotes the source, and says a passing check Athena recorded on the same tree counts, so fmt, clippy and test can be named as evidence, while a failed record or another tree's record does not. (c) D63: the line begins `- D63` and says the verifier runs whole simple commands, so a chained command is judged per simple command. (d) docs/decisions.md and records/ are append-only: the section contains both enforcing phrases, says to append a new decision with a Provenance bullet, and says a decision-links.toml link is needed only for a superseding or amending entry or a [[governs]] path. It must NOT contain 'always add a docs/decision-links.toml link'. (e) CHANGELOG.md is named as a default-written deliverable that a criterion must not forbid, with the DEFAULT_WRITTEN_FILES reason quoted. (f) The heading appears once, after `# Checkable criteria` and before `# What the verifier can run`, and `READING_RULES` still has 12 lines. Run: `cargo test -p athena-context the_reader_brief_states_the_rules_that_judge_criteria`. Observes: The test passes with the change. On the base commit it fails at the heading lookup, because no such section exists.
  Fails without the behavior: Without `criteria_judging_section` the brief has no `# Rules that judge criteria` heading, so the section lookup fails and every phrase assertion fails. The test also fails if the section mislabels the link rule or leaves out D63.
  Test seam: Production entry point `reader_brief` (reader.rs:319) through `reference_and_rules_section` (reader.rs:491). The fixture `reader_inputs(&reading())` exists (briefs.rs:3540) and is passed in directly. No double is used and the section text comes from the production function. `ReaderInputs` needs no new field. Step 2 adds the section function and step 4 adds the test.
- Criterion 2: Two tests in briefs.rs, both reading the enforcing sources with `include_str!` of ../../athena-core/src/verify.rs, mirrors.rs, admission.rs, reading/checks.rs and ../../athena-gate/src/lib.rs. Test 1, `the_criteria_rules_quote_the_wording_the_enforcing_code_uses`: (i) slice verify.rs from `const VERDICT_RULES` to `const REQUIRED_FIELDS`; helper `verdict_rows` returns every (id, source) pair; assert the ids D43, D63 and D115 are all found, then assert that for every pair the section has a line starting `- <id>` that contains the source. (ii) slice checks.rs from `const DEFAULT_WRITTEN_FILES` to `)];`; helper `string_literals` returns every literal; assert there is at least one and each is in the section, and that the section names `CHANGELOG_PATH`'s value. (iii) For mirrors.rs, admission.rs and athena-gate/src/lib.rs, take the text before the first `\n#[cfg(test)]`, assert it contains DECISIONS_APPEND_ONLY_PHRASE (mirrors.rs:265), CHANGELOG_DEFAULT_PHRASE (admission.rs:999) and RECORDS_APPEND_ONLY_PHRASE (lib.rs:101), and that the section contains each. Failure messages name the constant and the file to update. Test 2, `the_drift_helpers_report_a_new_or_reworded_enforcing_row`: gives `verdict_rows`, `string_literals` and the missing-line finder a synthetic table with four rows (a made-up D99 row, a reworded D43 source) and a second default-written row, and asserts each is reported as missing from the section. There is no tautological replace assertion. Run: `cargo test -p athena-context the_criteria_rules_quote_the_wording_the_enforcing_code_uses`. Observes: Both tests pass now. If a fourth VERDICT_RULES row is added, or a source or the default-written reason is reworded without the brief following, test 1 fails and names the id or literal. A copy of the old wording left in a test or comment cannot satisfy it, because the search covers only the table slice or the pre-`#[cfg(test)]` text.
  Fails without the behavior: The tests do not compile without the constants and helpers. With the constants but a reworded verify.rs source, a new VERDICT_RULES row, or a new DEFAULT_WRITTEN_FILES row, test 1 fails. Test 2 shows the helpers detect a drifted or extra row, so the detection is not an empty check.
  Test seam: Production entry points are `reader_brief` and the pub consts and `VERDICT_RULE_LINES` in reader.rs, which step 1 adds. The fixtures are the enforcing source files themselves, read by `include_str!` at compile time, and in test 2 a synthetic text passed as an argument to the parse helpers. The brief-side section text is not a copy in the test: it comes from `reader_brief`. The helpers are test-local, parse only, and add no logic under test to a double. The seam is new and added by step 4.
- Criterion 3: The verifier runs `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` as three separate whole simple commands on the final tree. Athena's recorded passes of the same three on the same tree count under D115. `cargo test --locked` runs the new tests, the READING_RULES size tests (briefs.rs:3650, 3709), `system_facts_budgets_match_computed`, `changelog_unproduced_asks_for_the_root_changelog_on_a_change_reading` (reading.rs:2334), the refusal_corpus target and the athena-cli backtest and decision tests. Run: `cargo test --locked`. Observes: All three exit 0. The test run lists both new tests by name and the unchanged existing tests.
  Fails without the behavior: `cargo test` fails if the section is missing (test 1), if an enforcing row drifts from the brief (test 2), or if the section was added by growing READING_RULES (the 12-line and 3045-byte tests). Deleting athena-core's local CHANGELOG_PATH without adding the import fails to compile in reading.rs and checks.rs. Keeping both the local const and the import, or an unused import, fails clippy under -D warnings. Unformatted new code fails fmt. On the base commit the new tests do not exist, so this run cannot show them passing there.
  Test seam: The existing workspace checks at the workspace root; no fixture, double or new seam. The code under test is the real `reader_brief` and the `CHANGELOG_PATH` use sites in athena-core, reached through the tests added in step 4 and the existing tests named above.
- Criterion 4: Content check of the diff. `fn criteria_judging_section` is in crates/athena-context/src/briefs/reader.rs and is called from `reference_and_rules_section`. Grep of the diff for `fn check_` finds nothing. `git diff -- crates/athena-core/src/reading` (the checks.rs and surveys.rs directory) is empty. reading.rs only swaps `const CHANGELOG_PATH` for an import. Run: `git diff --stat`. Observes: The section function sits next to `reference_and_rules_section`, and no refusal function appears under crates/athena-core/src/reading/.
- Criterion 5: Content check. The added CHANGELOG entry and the added D174 each contain the phrase 'expected 0% of newly refused readings' and say that no refusal rule is added and the brief gives the information instead. Run: `git diff -- CHANGELOG.md docs/decisions.md`. Observes: The phrase is present in both added entries. No criterion, check or corpus case is added.
- Criterion 6: Content check. (a) The CHANGELOG.md diff adds one entry under Unreleased that names the root cause (the reader was never shown the rules that judge criteria) and lists only what the diff does: the new brief section, the table-driven pin tests and the shared constant. (b) The decisions.md numstat shows added lines and 0 deleted lines. The added text starts after the last line of D173, with heading `## D174` and a `- **Provenance:**` bullet that says the effect on the reader's output is not measured. (c) docs/decision-links.toml is unchanged, because the added text has no 'supersede' or 'amends' phrase (Grep of the added text) and touches no [[governs]] path. Run: `git diff --numstat -- docs/decisions.md`. Observes: Deletions are 0. The mirror rules 'append only' (mirrors.rs:243) and 'decision entries' (mirrors.rs:346-396) accept the diff at verification.
- Criterion 7: Content check plus Athena's budget test. `git diff --stat` lists no path under crates/athena-context/prompts/. The budgets test checks the docs/system-facts.md budget lines against the computed values, with the reader set at 1998 of 2000. Run: `cargo test -p athena-cli --test budgets`. Observes: The diff touches only the files listed in the plan, and the budget test passes unchanged.

Planned files:

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base a187f7f81b6b → candidate 979f4b29c23a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   1 +
 crates/athena-context/src/briefs.rs        | 281 ++++++++++++++++++++++++++++-
 crates/athena-context/src/briefs/reader.rs |  81 +++++++++
 crates/athena-context/src/lib.rs           |   3 +-
 crates/athena-core/src/reading.rs          |   6 +-
 docs/decisions.md                          |  37 ++++
 6 files changed, 402 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261112-194023-nyf7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-194023-nyf7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43298667391 for 979f4b29c23aa200ebe4d14ad2c18adc906b2539)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43298667391 for 979f4b29c23aa200ebe4d14ad2c18adc906b2539)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43298667391 for 979f4b29c23aa200ebe4d14ad2c18adc906b2539)
- ci-result: `not a check` passed (Check results for 979f4b29c23aa200ebe4d14ad2c18adc906b2539 come from the CI run https://github.com/doodla/athena/actions/runs/43298667391 (758s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy` and `cargo test`; all pass for the workspace.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context the_reader_brief_states_the_rules_that_judge_criteria`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/lib.rs, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |
 282 | pub const READING_RULES_HEADING: &str = "# Reading rules";
     | ---------------------------------------------------------- similarly named constant `READING_RULES_HEADING` defined here
     |
help: a constant with a similar name exists
     |
3907 -         let heading = brief.find(CRITERIA_RULES_HEADING).unwrap();
3907 +         let heading = brief.find(READING_RULES_HEADING).unwrap();
     |

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:3974:36
     |
3974 |         assert!(changelog.contains(DEFAULT_WRITTEN_REASON), "{changelog}");
     |                                    ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `VERDICT_RULE_LINES` in this scope
    --> crates/athena-context/src/briefs.rs:4004:13
     |
4004 |             VERDICT_RULE_LINES
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:4016:42
     |
4016 |             literals.iter().any(|l| l == DEFAULT_WRITTEN_REASON),
     |                                          ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:4017:74
     |
4017 | ...   "DEFAULT_WRITTEN_FILES in reading/checks.rs no longer says `{DEFAULT_WRITTEN_REASON}`; update DEFAULT_WRITTEN_REASON in read...
     |                                                                    ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `CHANGELOG_PATH` in this scope
    --> crates/athena-context/src/briefs.rs:4023:34
     |
4023 |         assert!(section.contains(CHANGELOG_PATH));
     |                                  ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DECISIONS_APPEND_ONLY_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4033:14
     |
4033 |             (DECISIONS_APPEND_ONLY_PHRASE, "mirrors.rs", MIRRORS_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `CHANGELOG_DEFAULT_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4034:14
     |
4034 |             (CHANGELOG_DEFAULT_PHRASE, "admission.rs", ADMISSION_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `RECORDS_APPEND_ONLY_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4035:14
     |
4035 |             (RECORDS_APPEND_ONLY_PHRASE, "athena-gate lib.rs", GATE_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 11 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context the_criteria_rules_quote_the_wording_the_enforcing_code_uses`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/lib.rs, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |
 282 | pub const READING_RULES_HEADING: &str = "# Reading rules";
     | ---------------------------------------------------------- similarly named constant `READING_RULES_HEADING` defined here
     |
help: a constant with a similar name exists
     |
3907 -         let heading = brief.find(CRITERIA_RULES_HEADING).unwrap();
3907 +         let heading = brief.find(READING_RULES_HEADING).unwrap();
     |

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:3974:36
     |
3974 |         assert!(changelog.contains(DEFAULT_WRITTEN_REASON), "{changelog}");
     |                                    ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `VERDICT_RULE_LINES` in this scope
    --> crates/athena-context/src/briefs.rs:4004:13
     |
4004 |             VERDICT_RULE_LINES
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:4016:42
     |
4016 |             literals.iter().any(|l| l == DEFAULT_WRITTEN_REASON),
     |                                          ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:4017:74
     |
4017 | ...   "DEFAULT_WRITTEN_FILES in reading/checks.rs no longer says `{DEFAULT_WRITTEN_REASON}`; update DEFAULT_WRITTEN_REASON in read...
     |                                                                    ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `CHANGELOG_PATH` in this scope
    --> crates/athena-context/src/briefs.rs:4023:34
     |
4023 |         assert!(section.contains(CHANGELOG_PATH));
     |                                  ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DECISIONS_APPEND_ONLY_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4033:14
     |
4033 |             (DECISIONS_APPEND_ONLY_PHRASE, "mirrors.rs", MIRRORS_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `CHANGELOG_DEFAULT_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4034:14
     |
4034 |             (CHANGELOG_DEFAULT_PHRASE, "admission.rs", ADMISSION_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `RECORDS_APPEND_ONLY_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4035:14
     |
4035 |             (RECORDS_APPEND_ONLY_PHRASE, "athena-gate lib.rs", GATE_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 11 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/lib.rs, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
 282 | pub const READING_RULES_HEADING: &str = "# Reading rules";
     | ---------------------------------------------------------- similarly named constant `READING_RULES_HEADING` defined here
     |
help: a constant with a similar name exists
     |
3907 -         let heading = brief.find(CRITERIA_RULES_HEADING).unwrap();
3907 +         let heading = brief.find(READING_RULES_HEADING).unwrap();
     |

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:3974:36
     |
3974 |         assert!(changelog.contains(DEFAULT_WRITTEN_REASON), "{changelog}");
     |                                    ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `VERDICT_RULE_LINES` in this scope
    --> crates/athena-context/src/briefs.rs:4004:13
     |
4004 |             VERDICT_RULE_LINES
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:4016:42
     |
4016 |             literals.iter().any(|l| l == DEFAULT_WRITTEN_REASON),
     |                                          ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DEFAULT_WRITTEN_REASON` in this scope
    --> crates/athena-context/src/briefs.rs:4017:74
     |
4017 | ...   "DEFAULT_WRITTEN_FILES in reading/checks.rs no longer says `{DEFAULT_WRITTEN_REASON}`; update DEFAULT_WRITTEN_REASON in read...
     |                                                                    ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `CHANGELOG_PATH` in this scope
    --> crates/athena-context/src/briefs.rs:4023:34
     |
4023 |         assert!(section.contains(CHANGELOG_PATH));
     |                                  ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `DECISIONS_APPEND_ONLY_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4033:14
     |
4033 |             (DECISIONS_APPEND_ONLY_PHRASE, "mirrors.rs", MIRRORS_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `CHANGELOG_DEFAULT_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4034:14
     |
4034 |             (CHANGELOG_DEFAULT_PHRASE, "admission.rs", ADMISSION_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `RECORDS_APPEND_ONLY_PHRASE` in this scope
    --> crates/athena-context/src/briefs.rs:4035:14
     |
4035 |             (RECORDS_APPEND_ONLY_PHRASE, "athena-gate lib.rs", GATE_SRC),
     |              ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 11 previous errors
warning: build failed, waiting for other jobs to finish...
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
