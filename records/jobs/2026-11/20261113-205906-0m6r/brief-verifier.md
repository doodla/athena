# Job 20261113-205906-0m6r: verify "Give reader and planner the field-meaning fact before reusing a field"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Planning checks that named files and seams exist, but nothing checks that a field the reading reuses
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: reading-maps-requirement-to-existing-field-with-different-meaning-fails-planning-rounds (20261112-090746-vvvp)
> Cause: Planning checks that named files and seams exist, but nothing checks that a field the reading reuses actually means what the requirement needs. The reading matched the words "fixed in Athena" to a field by name and plausibility without read...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261112-090730-g560 (job 20261030-013421-tfwf); prevention it proposed: Add a reading-stage check: when the reading maps a requirement onto an existing struct field, CLI argument or record key, it must quote that field's documented meaning from the code and state why it matches. A mismatch, or a requirement wit...
> Introspection 20261113-205845-yyfk (job 20261113-194336-scfc); prevention it proposed: Before a reading is accepted, run a step that resolves each config key or field it reuses against the live athena.toml and the code that reads it. It should report the effective value for each role and whether the change would alter behavio...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: neither the reader's brief nor the planner prompt told the stage that a reused struct field, CLI argument or record key must be read for its documented meaning, so jobs 20261030-013421-tfwf (restart `via` read as "fix in Athena") and 20261113-194336-scfc (hard_producer_* read as a planner upgrade) matched by name. The fix supplies that rule as information in the reader brief's Reading rules and the planner prompt, not as a refusal check. Both earlier candidates changed no files, so none is kept; the earlier findings are addressed as stated in the assumptions.

Assumptions:
- Code survey: read crates/athena-context/src/briefs/reader.rs (the READING_RULES list around lines 290-300 lists rules with check names or `(prose only)`), crates/athena-context/prompts/planner.md and critic.md, crates/athena-core/src/reading/surveys.rs (check_code_survey, which this change does not touch), and crates/athena-core/src/run/planning.rs. Entry point is brief rendering for the reader and planner; no record or data file is added, so AnyRecord and check_record_file are unaffected. The property is measured directly by asserting the rendered brief text.
- Existing code: crates/athena-context/src/briefs/reader.rs and crates/athena-context/prompts/planner.md are extended; the new criteria reuse them.
- Job 20261030-013421-tfwf: its candidate changed no files, so nothing is kept. Its major finding (via is documented as what the new context read to resume, crates/athena-cli/src/main.rs:450, yet was counted as a fix) is addressed at the origin by the new rule, which would have made the reader quote that doc and add a new `fixed` field. Its OLY-B-O5-02 feature work (docs/spec.toml, status.rs, display.rs, manage.rs) is out of scope here and stays queued by the drain.
- Job 20261113-194336-scfc: its candidate changed no files, so nothing is kept. Its finding (hard_producer_model/effort do not upgrade the planner under mode = hard; the reading did not check the live athena.toml) is addressed by the same rule, which also tells the stage to check the live config value. Its planning.rs, tests.rs, config.rs, drain.rs and spec.toml work is out of scope.
- Goal and audience: the reader and planner roles are the audience; the goal is that they receive the information before mapping a requirement to an existing field. This is preferred over a refusal rule, since the reading's meaning cannot be judged by mechanism.
- Athena's reader prompt has 2 tokens of headroom, so the rule goes in the reader brief's Reading rules, which is code, not in prompts/reader.md. Whether the brief has its own token budget test is unconfirmed; the producer must run the tests.
- Consumers: none found (no athena.toml or defaults.toml change).

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-context (run `cargo test -p athena-context`) renders the reader brief and asserts its Reading rules section contains a rule requiring, for each reused existing struct field, CLI argument, record key or config key, a `Field meaning:` assumption quoting the documented meaning with a crates/ path:line and why it matches, or a new field when it does not. The test fails on the code before the change.
2. [behavior] A new unit test in crates/athena-context (`cargo test -p athena-context`) asserts the planner prompt/brief carries the same instruction, so a plan that reuses a field must quote its documented meaning and check the live athena.toml value; it fails before the change.
3. [content] The new rule is generic (any field, CLI argument, record key or config key), not specific to restart `via` or `hard_producer_*`; the diff of crates/athena-context/src/briefs/reader.rs shows it added to the Reading rules list marked `(prose only)`.
4. [content] The diff adds no new refusing check in crates/athena-core; the information is given in the brief and prompt, so expected 0% of past accepted readings are newly refused.
5. [content] The CHANGELOG.md entry names the root cause (the reading and plan stages were never told to verify a reused field's documented meaning) and cites jobs 20261030-013421-tfwf and 20261113-194336-scfc; any decision is recorded as a new appended docs/decisions.md entry with a Provenance bullet.
6. [content] Token budgets hold: `cargo test` (including the prompt-budget tests) passes with planner.md growth within the 95 tokens left; no edit to the reader prompt, which has 2 tokens left.
7. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (Athena's recorded checks on that tree count).

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/prompts/critic.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Any refusal check in crates/athena-core that rejects readings
- Implementing OLY-B-O5-02 or the restart `fixed` field
- Planner effort or model escalation (job 20261113-194336-scfc work)
- Editing athena.toml, defaults.toml, workflows or docs/spec.toml
- Editing the reader prompt prompts/reader.md

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the rendered brief text, not file existence
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to brief and prompt text
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the stage lacking the information
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A new unit test in crates/athena-context/src/briefs.rs, placed next to `the_reader_brief_names_each_reading_rule_in_one_short_line` (briefs.rs:3628). It renders `reader_brief(&reader_inputs(&reading()))`, splits at READING_RULES_HEADING and cuts at the next `\n# ` (as briefs.rs:3630-3631 does). From the rendered section it picks the bullet line that contains the text `Field meaning:` in backticks and asserts it equals `- {rule}` for an entry of READING_RULES. It then asserts on that rendered line, case-sensitive except where stated: `Field meaning:`, `CLI argument`, `record key`, `config key`, `documented meaning`, `crates/`, `path:line`, `why it fits`, `athena.toml`, `live athena.toml value`, `new field`, `quote`, and (lowercased) `reused field`. Run: `cargo test -p athena-context the_reader_brief_requires_a_field_meaning_assumption_for_a_reused_field`. Observes: One line in the rendered Reading rules section tells the reader that, for each reused field, CLI argument, record key or config key, it must write a `Field meaning:` assumption quoting the documented meaning with a path:line (crates/ or config), why it fits, and for a config key the live athena.toml value, or plan a new field. The test passes with 13 rule lines.
  Fails without the behavior: On the code before the change the rendered Reading rules section has no line containing `Field meaning:`, so the lookup `expect` fails and so does every phrase assertion. If the rule were added to READING_RULES but not printed, the check on the rendered brief fails, because the test reads the brief and not the constant.
  Test seam: Production entry point: `reader_brief(&ReaderInputs)` (crates/athena-context/src/briefs/reader.rs:367). Fixture: the existing test helpers `reader_inputs(&reading())` (briefs.rs:3543; `reading()` at briefs.rs:1458) and the constant READING_RULES_HEADING. Both exist today and nothing is injected. The test adds no double; the text under test is the real rendered brief.
- Criterion 2: A new unit test in briefs.rs next to `the_planner_brief_states_the_real_records_rule_once_within_its_bound` (briefs.rs:3598). It renders `planner_brief(&planner_inputs(None))` and takes the text after `## Report` (as briefs.rs:3601 does). It asserts the report holds `FIELD_MEANING_RULE` exactly once, that the rule has no newline, and that it comes after REAL_RECORDS_RULE. It asserts literal phrases on the rendered report: `struct field`, `CLI argument`, `record key`, `config key`, `documented meaning`, `crates/`, `path:line`, `live athena.toml value`, `each role` and `new field`. It asserts the rule is generic: split on non-alphanumeric characters, no token equals `via` (ignoring case), and the lowercased text has neither `hard_producer` nor `restart`. Run: `cargo test -p athena-context the_planner_brief_requires_a_reused_field_to_quote_its_meaning_and_check_the_live_config`. Observes: The planner brief's Report tells the planner that a plan reusing a field must quote its documented meaning with a path:line, say why it matches, check the live athena.toml value for each role a config key affects, and plan a new field when the meaning differs.
  Fails without the behavior: Without the constant and its line in the Report, the literal-phrase assertions on the rendered report fail (and the test does not compile against the missing constant). If the line were printed twice or not at all, the `matches(..).count() == 1` assertion fails. Because the phrases are asserted on the rendered report, a constant that is declared but not printed does not pass.
  Test seam: Production entry point: `planner_brief(&PlannerInputs)` (crates/athena-context/src/briefs/planner.rs:78). Fixture: the existing `planner_inputs(None)` (briefs.rs:2500), which needs no injection. `FIELD_MEANING_RULE` is declared `pub(super)` like REAL_RECORDS_RULE (planner.rs:73), imported in the test with `use super::planner::FIELD_MEANING_RULE;` as briefs.rs:3599 does. Plan step 2 adds the constant; the fixture exists.
- Criterion 3: Extend the existing test, which already checks that each rendered line equals `- {rule}`, is at most 220 bytes, and that the prose-only lines end `(prose only).` (briefs.rs:3633-3637, 3673-3684). Add: the line containing the `Field meaning:` text in backticks ends with `(prose only).`; split into tokens on non-alphanumeric characters it has no token equal to `via` (ignoring case); lowercased it contains neither `restart` nor `hard_producer`; it contains no `check_`. Then read the diff of reader.rs with `git diff` to see that exactly one element is added to the READING_RULES list and the array type goes from 12 to 13. Run: `cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line`. Observes: The reader.rs diff adds one generic element to READING_RULES, marked `(prose only).`, names no field from the two jobs, and changes the array type from [&str; 12] to [&str; 13]. The test passes with 13 lines, each at most 220 bytes.
  Fails without the behavior: Without the line the extended `Field meaning` assertions fail. With a line that carries a `check_` name or lacks the ending `(prose only).` the prose-only assertion fails, and with a line over 220 bytes the length assertion (briefs.rs:3636) fails.
  Test seam: Production entry point: `reader_brief` (reader.rs:367) through the existing test `the_reader_brief_names_each_reading_rule_in_one_short_line` (briefs.rs:3628). Fixture: the existing `reader_inputs(&reading())`. It exists and nothing is injected.
- Criterion 4: Read the diff stat and the file list. The changed paths are crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md and docs/decisions.md. Confirm that no path under crates/athena-core appears. No refusal or matching rule is added or changed, so no refusal_corpus case is needed. The reading-rule replay still runs, because `touches_reading_rules` (crates/athena-core/src/reading/replay.rs:314) is true for any crates/ path; it compares the candidate with the base on past accepted readings, and no check code changed, so it should report 0 newly refused. Run: `git diff --stat`. Observes: The diff has no crates/athena-core path and adds no check, so the information is given in the brief and prompt and the expected share of past accepted readings newly refused is 0%.
- Criterion 5: Read the diff. CHANGELOG.md gets one entry under `[Unreleased]` / `### Added` (CHANGELOG.md:10) that names the root cause (the reader and planner stages were never told to verify a reused field's documented meaning before mapping a requirement to it), cites jobs 20261030-013421-tfwf and 20261113-194336-scfc, says no refusal rule is added so the expected share of past readings newly refused is 0%, and names the tests. docs/decisions.md gets D180 appended after D179 with a Provenance bullet, and no existing line changes. The full `cargo test --locked` run covers the decision-entries rule (crates/athena-cli/src/decisions.rs:210-235) and the append-only mirror check. Run: `git diff CHANGELOG.md docs/decisions.md`. Observes: The changelog entry text, and an append-only diff of docs/decisions.md that ends with D180 and its Provenance bullet.
- Criterion 6: Run the whole suite, which includes `system_facts_budgets_match_computed` (crates/athena-cli/tests/budgets.rs:125) and the other budget tests, then run `git diff --stat` to confirm prompts/planner.md, prompts/critic.md and prompts/reader.md and docs/system-facts.md are not in the diff. planner.md growth is 0 tokens, within the 95 left, and the reader prompt is untouched. Run: `cargo test --locked`. Observes: The suite passes, and the lines `budget planner: 1105 of 1200`, `budget critic: 1002 of 1300` and `budget reader: 1998 of 2000` in docs/system-facts.md stay true because no prompt file changes. The new brief texts sit outside the budget sets.
- Criterion 7: Run Athena's three recorded checks on the final tree, each as its own whole command: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. A passing record Athena made on the same final tree counts as a run (D115). `cargo test --locked` includes the two new tests and every moved pin. Run: `cargo test --locked`. Observes: All three checks exit 0 on the final tree.
  Fails without the behavior: Without step 3 the existing tests fail on the new text: `the_reader_brief_names_each_reading_rule_in_one_short_line` fails on `READING_RULES.len() == 12` (briefs.rs:3653) and on the 3045-byte baseline (briefs.rs:3712), the criteria-rules test fails on its `len() == 12` assert (briefs.rs:3983), and `the_planner_brief_states_the_real_records_rule_once_within_its_bound` fails on the Report bound (briefs.rs:3619). Without the rule the two new tests fail. A new line left unformatted fails `cargo fmt --all --check`.
  Test seam: Production entry points are the three cargo commands that Athena runs as its recorded checks on the candidate tree (the 'Checks Athena will run on the result' list: fmt, clippy, test). Fixture: the final candidate tree itself, with the test modules of athena-context and the budget tests of athena-cli. Athena injects nothing, because it runs the commands in the working tree. The seam exists today.

Planned files:

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 08a1eadf4aef → candidate 003e4f4c5c65. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   1 +
 crates/athena-context/src/briefs.rs         | 122 ++++++++++++++++++++++++----
 crates/athena-context/src/briefs/planner.rs |   9 +-
 crates/athena-context/src/briefs/reader.rs  |   6 +-
 docs/decisions.md                           |  29 +++++++
 5 files changed, 148 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261113-205906-0m6r/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-205906-0m6r/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43522350222 for 003e4f4c5c65c9de92c35d3e6eea8fb4ee3fee0a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43522350222 for 003e4f4c5c65c9de92c35d3e6eea8fb4ee3fee0a)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43522350222 for 003e4f4c5c65c9de92c35d3e6eea8fb4ee3fee0a)
- ci-result: `not a check` passed (Check results for 003e4f4c5c65c9de92c35d3e6eea8fb4ee3fee0a come from the CI run https://github.com/doodla/athena/actions/runs/43522350222 (681s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (Athena's recorded checks on that tree count).)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context the_reader_brief_requires_a_field_meaning_assumption_for_a_reused_field`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs/reader.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling generic-array v0.14.7
   Compiling zmij v1.0.23
   Compiling libc v0.2.190
   Compiling serde v1.0.229
   Compiling syn v3.0.7
   Compiling serde_json v1.0.151
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling getrandom v0.3.4
   Compiling thiserror v2.0.21
   Compiling cfg-if v1.0.5
   Compiling ref-cast v1.0.27
   Compiling itoa v1.0.18
   Compiling memchr v2.8.3
   Compiling winnow v1.0.4
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_derive_internals v0.30.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling digest v0.10.7
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling winnow v0.7.15
   Compiling cpufeatures v0.2.17
   Compiling jiff-core v0.1.1
   Compiling sha2 v0.10.9
   Compiling jiff v0.2.38
   Compiling toml v0.9.12+spec-1.1.0
   Compiling schemars_derive v1.2.2
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0432]: unresolved import `super::planner::FIELD_MEANING_RULE`
    --> crates/athena-context/src/briefs.rs:3635:30
     |
3635 |         use super::planner::{FIELD_MEANING_RULE, REAL_RECORDS_RULE};
     |                              ^^^^^^^^^^^^^^^^^^ no `FIELD_MEANING_RULE` in `briefs::planner`

error[E0425]: cannot find value `FIELD_MEANING_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:3623:39
     |
3623 |                     + super::planner::FIELD_MEANING_RULE.len()
     |                                       ^^^^^^^^^^^^^^^^^^ not found in `super::planner`

error[E0425]: cannot find value `FIELD_MEANING_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:3629:29
     |
3629 |             super::planner::FIELD_MEANING_RULE.len()
     |                             ^^^^^^^^^^^^^^^^^^ not found in `super::planner`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context the_planner_brief_requires_a_reused_field_to_quote_its_meaning_and_check_the_live_config`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs/reader.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0432]: unresolved import `super::planner::FIELD_MEANING_RULE`
    --> crates/athena-context/src/briefs.rs:3635:30
     |
3635 |         use super::planner::{FIELD_MEANING_RULE, REAL_RECORDS_RULE};
     |                              ^^^^^^^^^^^^^^^^^^ no `FIELD_MEANING_RULE` in `briefs::planner`

error[E0425]: cannot find value `FIELD_MEANING_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:3623:39
     |
3623 |                     + super::planner::FIELD_MEANING_RULE.len()
     |                                       ^^^^^^^^^^^^^^^^^^ not found in `super::planner`

error[E0425]: cannot find value `FIELD_MEANING_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:3629:29
     |
3629 |             super::planner::FIELD_MEANING_RULE.len()
     |                             ^^^^^^^^^^^^^^^^^^ not found in `super::planner`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs/reader.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0432]: unresolved import `super::planner::FIELD_MEANING_RULE`
    --> crates/athena-context/src/briefs.rs:3635:30
     |
3635 |         use super::planner::{FIELD_MEANING_RULE, REAL_RECORDS_RULE};
     |                              ^^^^^^^^^^^^^^^^^^ no `FIELD_MEANING_RULE` in `briefs::planner`

error[E0425]: cannot find value `FIELD_MEANING_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:3623:39
     |
3623 |                     + super::planner::FIELD_MEANING_RULE.len()
     |                                       ^^^^^^^^^^^^^^^^^^ not found in `super::planner`

error[E0425]: cannot find value `FIELD_MEANING_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:3629:29
     |
3629 |             super::planner::FIELD_MEANING_RULE.len()
     |                             ^^^^^^^^^^^^^^^^^^ not found in `super::planner`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
```
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
