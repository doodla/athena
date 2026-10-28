# Job 20261028-162948-9a36: verify "Replace replay refusal-phrase exemption with a declared percentage"

## Request (verbatim)

> Priority: urgent
> The replay gate for new refusal rules has a loophole: a candidate is exempt from the 10% hold whenever its criteria contain a phrase saying a higher refusal rate is intended (INTENDED_REFUSAL_PHRASES). On 2026-10-28 a rule that newly refused 40 of the last 50 accepted readings (80%) landed that way. Replace the phrase exemption with a stated number: a candidate may declare an expected share of newly refused readings (a percentage in its acceptance criteria), and the replay holds it when the measured share exceeds the declared share by more than 5 points, or when it declares none and exceeds 10%. Any declared share above 25% also needs the reading's out_of_scope or criteria to say why past readings should be refused (for example a new required section that the reader brief now asks for), and the verifier is shown that reason beside the replay list.
> Acceptance:
> - A test shows a candidate declaring 'expected 30%' with a measured 80% is held, and one declaring 80% with 80% measured passes the replay.
> - A test shows a phrase such as 'the higher refusal rate is intended' without a number no longer exempts a candidate.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/reading.rs, CHANGELOG.md

## Reading (accepted by the user)

The replay gate stops exempting candidates that say a higher refusal rate is intended. A candidate may declare an expected percentage of newly refused readings in its criteria. It is held when the measured share exceeds the declared share by more than 5 points, or exceeds 10% when none is declared. A declared share above 25% needs a stated reason, which the verifier is shown beside the replay list. The audience is the maintainer and the verifier relying on the replay gate.

Assumptions:
- Code survey: I read crates/athena-core/src/reading.rs (INTENDED_REFUSAL_PHRASES, criteria_intend_higher_refusal, replay_is_held, ReplayRun, and the replay function that takes the running job's criteria; ReplayRun.record builds the stored Replay) and crates/athena-core/src/run/mod.rs (replay_stage, then replay_run, then replay_checkers; replay_view builds the verifier's ReplayView; a held replay fails the verdict in judge). The replay measures the share directly as newly_refused.len() out of total, by running base and candidate checkers over replay_cases. It does not use a stand-in. The new rule is integer math: held when newly*100 > total*(declared+5), or total*10 when none is declared.
- Records validation: this change keeps the stored Replay record shape in crates/athena-data/src/records.rs unchanged. It adds no record or data file and does not edit records.rs. The existing `intended` bool is kept and now means a declared share. Existing replay.json files still pass check_record_file and AnyRecord, which the existing replay record round-trip test in crates/athena-data covers. If a field must be added to the record, that is a separate change.
- Declared share is parsed from criteria text as 'expected N%' or 'N% of newly refused', whole percent from 0 to 100. The first such number wins. A phrase without a number declares nothing.
- The reason for a share above 25% is any out_of_scope entry or criterion mentioning why past readings should be refused. Concretely, it must contain 'because', 'now asks' or 'new required'. The producer picks a simple deterministic detector and documents it. Without a reason, the declared share is treated as not declared, so the 10% limit applies. The reason is passed through ReplayView in run/mod.rs and shown in the verifier brief beside the list.
- Tests go in the existing test modules of reading.rs and run/mod.rs. The 5-point margin is inclusive: exactly declared+5 is not held, matching the existing 'exactly the threshold is not held' rule.

## Acceptance criteria

1. [behavior] A unit test run with `cargo test -p athena-core` shows a candidate declaring 'expected 30%' with 40 of 50 (80%) newly refused is held, and one declaring 'expected 80%' with a stated reason and 80% measured passes the replay.
2. [behavior] A unit test run with `cargo test -p athena-core` shows a criterion such as 'the higher refusal rate is intended' with no number no longer exempts a candidate: at 80% measured it is held, and at exactly 10% it is not.
3. [behavior] Unit tests show the 5-point margin edge (declared 30% with 35% measured passes, 36% is held), and a declared share above 25% with no reason in out_of_scope or criteria is held at the 10% limit.
4. [behavior] A test shows the reason for a declared share above 25% appears in the ReplayView or verifier brief beside the replay list.
5. [content] INTENDED_REFUSAL_PHRASES and the phrase-based exemption are removed from crates/athena-core/src/reading.rs (Grep finds no remaining use), and replay_is_held takes the declared share instead of an `intended` bool.
6. [content] CHANGELOG.md has an entry that describes the declared-share rule, the 5-point margin, the 10% default and the 25% reason requirement.
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
8. [behavior] The change refuses more readings than before: an 80% newly refused candidate with only a phrase is now held. The higher refusal rate is intended, and a test covers it.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/brief.rs (modify)

## Out of scope

- Changing the replay window size or the stored Replay record shape in crates/athena-data/src/records.rs
- Changing athena.toml, defaults.toml or workflow files
- Re-evaluating or reverting the 2026-10-28 landing

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new passing tests, clippy -D warnings and fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check held and passing outcomes in tests, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The phrase exemption is replaced by a measured number instead of a longer phrase list.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by the replay gate code.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in reading.rs `mod tests` that call the production `replay_declaration` on criteria and then the production `replay`. Test A: criteria [ACCEPTED_CRITERION, 'expected 30% of newly refused readings'] feed `replay(&marked_cases(50, 40), &base_checker(), &marker_candidate(), declaration.effective_share())`. Test B: 'expected 80% of newly refused readings' plus out_of_scope ['readers now ask for a new required section, because past readings lack it'] run the same way. The doubles marked_cases, base_checker and marker_candidate exist today (used by the current replay tests) and only decide which cases refuse. The declaration parser, replay_is_held and replay are the real code. Run: `cargo test -p athena-core replay_declared_share`. Observes: Test A: total 50, newly_refused 40, held true. Test B: held false, intended true, and `record(..).validate()` is Ok. A held record from test A also passes validate.
  Fails without the behavior: Without the declared-share comparison (the old phrase rule), 'expected 30%' declares nothing and 80% is held in both tests, so test B fails; if any intended phrase passed, test A would not be held. Both cannot hold under the old code.
- Criterion 2: Unit tests in reading.rs. Criteria 'the higher refusal rate is intended' and 'the change refuses more readings than the base'. Assert replay_declaration(..).share is None. Call the production `replay` with effective_share() over marked_cases(50, 40) (80%) and marked_cases(10, 1) (exactly 10%). Same existing doubles as criterion 1. Run: `cargo test -p athena-core replay_phrase_without_a_number`. Observes: 80% measured: held true, intended false. Exactly 10% (1 of 10): held false.
  Fails without the behavior: The old code exempts any candidate whose criterion has 'higher refusal rate' or 'refuses more', so the 80% case is not held and the held assertion fails.
- Criterion 3: Unit tests in reading.rs. (a) replay_is_held(100, 35, Some(30)) is false and replay_is_held(100, 36, Some(30)) is true; a total-50 table (25 not held, 26 held, for Some(30)) and Some(0) against None. (b) Criteria 'expected 80% of newly refused readings' with empty out_of_scope: replay_declaration gives share Some(80) but effective_share None; the production `replay` over marked_cases(10, 2) is held and marked_cases(10, 1) is not. (c) 'expected 25% ...' needs no reason; 'expected 26% ...' with no reason falls back to 10%. (d) Negative reason tests: out_of_scope ['records.rs is out of scope because it needs a migration'] (marker, no anchor) does not unlock 80% (held at 20% measured); an item with an anchor but no marker ('past readings are refused') does not unlock it either; one item with marker and anchor does. (e) Parser negatives: a criterion quoting "'expected 30%' as test data" or 'the log shows 30% progress' declares nothing; 'expected 2.5% ...' declares nothing. Run: `cargo test -p athena-core replay_declared_margin && cargo test -p athena-core replay_reason`. Observes: The boundary table matches (35 passes, 36 held). An undocumented 80% is held at 20% measured and not at 10%. The reason negatives are held; the positive passes. Quoted example percentages declare nothing.
  Fails without the behavior: Without the declared+5 comparison, 36% against 30% would not be held or 35% would be; without the marker-and-anchor rule the unrelated 'because' item would unlock 80%.
- Criterion 4: Two tests joined at ReplayView. (1) run/mod.rs `mod tests`: take fixture_reading (exists), set acceptance_criteria to include 'expected 80% of newly refused readings' and out_of_scope to a marker-and-anchor reason; build a Replay with the production `ReplayRun::record`; call the production `replay_view(&record, &reading)`, which is what the verifier-inputs site at run/mod.rs:2510 calls with ctx.reading. Assert declared == Some(80) and reason equals the out_of_scope text; also a >25% reading with no reason gives declared None and unsupported Some(80), and a reading with no declaration gives all None. (2) briefs.rs `mod tests`: use the existing `replay_view` helper and `plain_verifier_inputs` (exist today), set declared Some(80) and a reason, render `verifier_brief(&inputs, true)`, and take `replay_part`. Step 5 adds the three ReplayView fields. A full go() run is not used because the fake driver's reading criteria cannot be set. Run: `cargo test -p athena-core replay_view_carries && cargo test -p athena-context replay_summary`. Observes: The replay section contains 'Declared share: 80%' and the reason text, both before the first '- <job>:' list line. The unsupported case prints that the declared share has no stated reason and the 10% limit applies.
  Fails without the behavior: If the view is not filled from the reading, or the section does not print the reason, the reason text is missing from ReplayView or from the rendered brief and the tests fail.
- Criterion 5: Content check by reading the diff plus a compile check. Use the Grep tool for INTENDED_REFUSAL_PHRASES and criteria_intend_higher_refusal over crates/ (expect no match). Read the replay_is_held signature (`declared: Option<u32>`). A caller still passing a bool does not compile. Run: `git diff -- crates/athena-core/src/reading.rs`. Observes: The diff deletes the const, the function and every use. replay_is_held takes `declared: Option<u32>`. `cargo build -p athena-core` succeeds.
- Criterion 6: Content test in reading.rs next to code_survey_refusal_is_declared_intended: include_str! of CHANGELOG.md, white space collapsed, asserts the Unreleased section names the declared-share form ('of newly refused readings'), the 5-point margin, the 10% default and the 25% reason requirement. The verifier can also Read the entry. The older entry that says 'refuses more ... higher refusal rate is intended' stays, since an existing test reads it. Run: `cargo test -p athena-core changelog_describes_the_declared_share_rule`. Observes: The test passes: CHANGELOG.md has an entry under Unreleased that states all four, and says briefs.rs holds the verifier brief change.
- Criterion 7: Run the three Athena checks. All existing tests that call `replay(.., &[])`, `criteria_intend_higher_refusal`, or build ReplayView with `intended` are updated in steps 8 and 9. The athena-data replay round-trip test is untouched. Also run system_facts_budgets_match_computed (part of cargo test) in case the CODE_SURVEY_GUIDE edit moves a budget. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A leftover use of the removed bool, an unused import (has_phrase, REPLAY_HOLD_PERCENT), or a stale replay(.., &[]) call fails compile or clippy.
- Criterion 8: Unit test in reading.rs replay_phrase_only_candidate_is_now_held: criterion 'the change refuses more readings than the base' (the phrase the old test treated as intended) with 'the higher refusal rate is intended', run through the production `replay` over marked_cases(50, 40) with the declaration's effective share. Compare with replay_is_held(50, 40, None). The doubles exist today. Also the go()-level test replay_holds_over_threshold_rejects_the_job_and_records_the_list still asserts held && !intended with the fake reading's criteria (no declaration). Run: `cargo test -p athena-core replay_phrase_only_candidate_is_now_held`. Observes: held true, intended false, 40 of 50 newly refused: the 2026-10-28 case no longer passes.
  Fails without the behavior: Under the base code the same candidate over marked_cases(50, 40) is exempt by phrase and not held, so the held assertion fails.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/verify.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. The criteria state a higher refusal rate is intended, so the job is not held.

## The change

Base 387358d797c8 → candidate 618b3cf4ab98. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  20 ++
 crates/athena-context/src/briefs.rs | 109 ++++++--
 crates/athena-core/src/reading.rs   | 508 ++++++++++++++++++++++++++++++------
 crates/athena-core/src/run/mod.rs   |  62 ++++-
 crates/athena-core/src/verify.rs    |   2 +-
 5 files changed, 605 insertions(+), 96 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1587 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
