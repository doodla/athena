# Job 20261103-123447-c3fa: verify "Add required plan 'dependents' section with admission refusal and critic"

## Request (verbatim)

> try again on 20261102-063419-0beg

## Reading (accepted by the user)

Retry of job 20261102-063419-0beg (dispatched, never verified, no findings recorded): add a required 'dependents' section to plans (PlanOutput schema, JobPlan record, planner brief), refused in plan_findings when missing or when a dependent lacks a test and a reason; the critic gains a "what else does this break" question; a CI-reuse replay test shows verify.rs's seen-commands rule must be named. Audience: Athena's planner, critic and the owner reviewing plans.

Assumptions:
- Code survey: read crates/athena-core/src/admission.rs (plan_findings at line 166, the admission entry point for plans, run after the planner and before the critic), crates/athena-context/src/outputs.rs (PlanOutput, line 367, with its JSON schema), crates/athena-data/src/records.rs (JobPlan, line 2415). The new field must be serde-default so stored JobPlan records without it still deserialize; this is covered by the existing records round-trip test and AnyRecord validation. No new record file or data file is added. The property (refusal of a plan) is measured directly by calling plan_findings on plan values.
- Job 20261102-063419-0beg was never verified and recorded no findings, so there is nothing to fix. Its approach is kept: its 23 changed files are all kept as deliverables below, and the work is redone from the base. That includes the producer's design: a dependents entry has a name, a dependent, a kind (code, rule or host setting), and a test or a reason; the refusal is a Major finding so the plan goes back to the planner without a critic call.
- The earlier candidate also edited docs/system-facts.md and crates/athena-context/src/lib.rs, run/mod.rs and several cli tests; these are kept as likely deliverables with lower probability, only where the new field requires them.
- Prompt growth stays within the headroom: planner 477 tokens left, critic 483 left. The contract, reader and verifier prompts are not touched.
- No governance file (athena.toml, defaults.toml, workflows) needs to change; the new rule is code in admission.rs.
- Expected 15% of newly refused readings: the rule refuses only stored or replayed plans that lack a dependents section, and replay of past plans is not migrated; the higher refusal rate is intended because the planner brief now asks for dependents.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-core admission: plan_findings on a plan with no dependents (empty list) returns a Major finding naming the missing section; an entry with a blank name, or with neither test nor reason, returns a Major finding naming that dependent; a complete section adds no finding.
2. [behavior] Run cargo test -p athena-core: a replay test for the CI-reuse case searches the real sources for the changed behaviour (exit_code) and shows the seen-commands filter in crates/athena-core/src/verify.rs turns up, and the dependents requirement is unmet unless the section names that rule; the test fails if it is not named.
3. [behavior] Run cargo test -p athena-context and cargo test -p athena-data: PlanOutput's JSON schema includes dependents, and a stored JobPlan record without the field still deserializes (records round-trip test passes).
4. [content] crates/athena-context/prompts/planner.md requires a 'dependents' section: for each behaviour, record, config key, check name or host setting the plan changes, list the code, rules (decisions, verdict rules, admission and gate checks) and host settings that read or rely on it, found by searching the repository and docs/system-facts.md, each with a test or a reason it is unaffected.
5. [content] crates/athena-context/prompts/critic.md asks, as its own question, what else this change breaks, and tells the critic to check the dependents list against a repository search.
6. [behavior] Run cargo test -p athena-context: the prompt token budget test passes with planner and critic each within 1300 tokens.
7. [content] CHANGELOG.md has an entry describing the dependents section, its admission refusal and the critic question; docs/design/job-planning.md describes the new field.
8. [behavior] The change refuses more plans than before (plans without dependents); expected 15% of newly refused readings, and the higher refusal rate is intended. Check by running cargo test -p athena-core refusal_corpus.

## Expected deliverables

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/system-facts.md (modify)

## Out of scope

- Automatically computing dependents by static analysis; the planner finds them by search
- Re-refusing or migrating plans already stored in records
- Changes to athena.toml, defaults.toml or workflows
- Fixing the four earlier breakages themselves

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New admission and replay tests; cargo test and clippy are standing, not listed as their own criterion per the request.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run plan_findings and the replay test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, CHANGELOG and design doc updated for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by admission refusal, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Adds the missing dependents question to planning rather than fixing single instances.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in the `admission.rs` test module. Build a `JobPlan` with a reading whose only criterion is Content, so no seam or demonstration rule fires. Vary only `dependents`: empty; an entry with a blank name; an entry with neither test nor reason (also whitespace-only strings); a complete entry with only a test; a complete entry with only a reason. Assert on the returned findings. Run: `cargo test -p athena-core admission`. Observes: An empty list gives exactly one Major finding, with source Athena, location `dependents`, and a summary naming the missing dependents section. A blank name gives a Major finding naming that entry (for example 'dependent 1'). An entry with neither test nor reason gives a Major finding naming the entry's `name`. Complete entries, test-only or reason-only, add no finding. The existing `seam_findings` fixtures now carry a complete entry, so their `is_empty()` assertions still hold.
  Fails without the behavior: Without the dependents rule an empty list adds no finding, so the 'one Major finding naming the missing section' assertion fails, as do the blank-name and no-test-or-reason assertions.
  Test seam: Production entry point `athena_core::admission::plan_findings(&JobPlan, &Reading, &Policy, &dyn Fn(&str)->bool)`, which is `pub` and called from `run/planning.rs`. Its guards are the demonstration, seam and path rules, which the fixtures satisfy: Content criterion, no files. Fixture: `JobPlan` literals built in the test module, injected as arguments (`findings_for_path` and `seam_findings` already do this). The plan is real data and no part of the rule is in a double. The helper that builds the plan with a `dependents` argument is added in step 6; the call path exists today.
- Criterion 2: Replay test in the `admission.rs` test module. It walks the real `CARGO_MANIFEST_DIR/src` tree for `.rs` files and keeps those containing `exit_code`. The needle is built as `["exit","code"].join("_")` so the test source does not match itself. It asserts that `src/verify.rs` is among the hits and that its text still holds the filter `c.reused && c.exit_code.is_none()`. It then calls `dependents_unnamed` on two dependents sections. Run: `cargo test -p athena-core ci_reuse`. Observes: A section naming only `checks.rs` and the `reused` record leaves `crates/athena-core/src/verify.rs` in the unnamed list, so the requirement is unmet and the test asserts that. A section with an entry whose `dependent` text names `verify.rs`' seen-commands filter (`judge`) and a test leaves it out, so the requirement is met. Other hits (such as `status.rs`) are not asserted either way. The test is part of `cargo test -p athena-core`.
  Fails without the behavior: If `dependents_unnamed` ignored names, or `verify.rs` were not found by the search, the assertion that the incomplete section leaves `verify.rs` unnamed fails. If the filter were removed from `verify.rs`, the source assertion fails.
  Test seam: Production entry point `athena_core::admission::dependents_unnamed(&[PlanDependent], &[String]) -> Vec<String>`, a pure function that step 6 adds. Fixture: the real `athena-core/src` files, read from disk via `CARGO_MANIFEST_DIR` and not a copy. The `PlanDependent` values are built in the test. No double is used. The guard is that the section names a path or file name. The search itself is the test's own walk; production does not run it (see the open questions).
- Criterion 3: Three tests. In `outputs.rs`: `plan_schema()` lists `dependents` in `properties` and `required`, with its own object (additionalProperties false), and a plan JSON with no `dependents` key still parses to an empty list. In `records.rs`: a `sample_plan` with no dependents serializes without the key, parses back equal and re-serializes identically, and `check_record_file` accepts it. A plan with dependents round-trips. Run: `cargo test -p athena-context && cargo test -p athena-data`. Observes: The schema property and `required` entry exist, and the nested-object count in `the_plan_and_critic_schemas_forbid_extra_properties_at_every_level` goes from 5 to 6. A record written without `dependents` parses and re-serializes byte for byte (the existing `a_demonstration_seam_is_optional...` and `a_plan_records_where_it_resumed_from...` pattern). A record with `dependents` keeps them through `AnyRecord::parse`.
  Fails without the behavior: Without the schema field the properties assertion fails. Without `serde(default)` on `JobPlan.dependents` the old-record parse fails with a missing field. Without `skip_serializing_if` the old record's bytes change.
  Test seam: Production entry points `athena_context::outputs::plan_schema()` and `serde_json::from_value::<PlanOutput>` (exist), and `AnyRecord::parse` / `to_json` / `check_record_file` in `records.rs` (exist). Fixtures: the `sample_plan` JSON in `outputs.rs` tests and the `sample_plan(round)` builder in `records.rs` tests. Both exist, and step 2/3 extends them with `dependents`. No double is used.
- Criterion 4: Read `planner.md` and check the clauses. A computed-budget test already guards the size. Run: `cargo test -p athena-context`. Observes: `prompts/planner.md` has a bullet 'dependents:' saying: for each behaviour, record, config key, check name or host setting the plan changes, list the code, rules (decisions, verdict rules, admission and gate checks) and host settings that read or rely on it, found by searching the repository and `docs/system-facts.md`, each with a test or a reason it is unaffected. The `athena-context` budget tests keep the planner set within 1300 tokens.
- Criterion 5: Read `critic.md` and check the clauses. A computed-budget test guards the size. Run: `cargo test -p athena-context`. Observes: `prompts/critic.md` has its own bullet asking 'what else does this change break?' and telling the critic to check the plan's dependents against a repository search with Grep. The critic brief shows the dependents because `plan_body` prints them; a `briefs.rs` test asserts the critic brief for a plan with dependents contains the 'Dependents:' block and one without it is unchanged.
- Criterion 6: Run the prompt budget tests. They compose `CONTRACT` + `PLANNER_FRAME` and `CONTRACT` + `CRITIC_FRAME` and compare the estimate ⌈bytes/3⌉ with `BudgetSet::budget` (1300 each, since the planner runs on the producer's budget and the critic on the verifier's). `system_facts_budgets_match_computed` also checks that the budget lines in `docs/system-facts.md` equal the computed sizes. Run: `cargo test -p athena-context && cargo test -p athena-cli --test budgets`. Observes: `check_budget` returns Ok for the planner and critic sets (`planner_and_critic_frames_are_short_and_fit_the_budgets`, `every_role_set_is_within_its_budget`). The new `budget planner` and `budget critic` lines in `docs/system-facts.md` equal the computed values (currently 823 and 817), both at most 1300.
  Fails without the behavior: If a prompt grew past 1300 tokens, `check_budget` returns Err and the test fails. If the system-facts lines were not updated, `system_facts_budgets_match_computed` fails with 'update docs/system-facts.md'.
  Test seam: Production entry points `athena_context::check_budget` and `compose` over the `include_str!` prompts (exist), and `athena_context::briefs::SYSTEM_FACTS` for the file lines. The fixture is the real prompt files, with no injection. The budgets test is `crates/athena-cli/tests/budgets.rs` and exists.
- Criterion 7: Read the two files. A `refusal_corpus` test (`design_existing_code_refusal_is_declared_intended`) already reads CHANGELOG, so the entry must not break its phrases. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: `CHANGELOG.md` has an `Unreleased` / `Added` entry naming the `dependents` section, the Major admission refusal, the critic's 'what else does this break' question and the root cause (planner plans not asked who relies on a changed behaviour; four earlier breakages). `docs/design/job-planning.md` section 3 lists `dependents` in the planner's output and in 'Athena's own checks'.
- Criterion 8: Plan-side cases in `refusal_corpus.rs`, each with an id, source and expected outcome, run through `plan_findings`. Cases are `dependents-missing` (refused: Major), `dependents-blank-name` (refused), `dependents-no-test-or-reason` (refused), `dependents-whitespace-only-test-and-reason` (refused), `dependents-test-only` (accepted), `dependents-reason-only` (accepted) and `dependents-complete-ci-reuse` (accepted; names the seen-commands rule in `verify.rs`). Sources: rebuilt from job 20261102-063419-0beg and the four earlier breakages. The existing reading cases are unchanged because `refusing_check` gets no new step. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: The plan-side cases get the expected outcome, and the existing request and reading cases, `corpus_covers_the_listed_input_forms` and `every_acceptance_check_has_a_refusing_corpus_case` still pass. Plans without dependents are now refused, which is the intended higher refusal rate. The changelog and the new cases state it as 'expected 15% of newly refused readings; the higher refusal rate is intended'.
  Fails without the behavior: Without the dependents rule the `dependents-missing`, `-blank-name`, `-no-test-or-reason` and whitespace-only cases are accepted, so the plan-side expectation test fails with a mismatch line naming the case.
  Test seam: Production entry point `athena_core::admission::plan_findings` (`pub`, exists; `athena_core::admission` is `pub mod` in `lib.rs`). Fixture: `JobPlan` and `Policy` literals built in `refusal_corpus.rs`; `athena_agent::policy::Policy` is reachable because `athena-core` depends on `athena-agent`. A reading with one Content criterion is injected as an argument. Step 9 adds the plan-side case table and test. I did not run the corpus: the case ids here are ones this plan adds, and the ids already in the file are not cited as results.

Planned files:

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/system-facts.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 15% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 2db16c6df729 → candidate 3f739015a04d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  13 ++
 crates/athena-agent/src/fake.rs            |   7 +
 crates/athena-cli/tests/introspection.rs   |   4 +
 crates/athena-cli/tests/job_plan.rs        |  45 ++++
 crates/athena-cli/tests/lifecycle.rs       |   4 +
 crates/athena-context/prompts/critic.md    |   1 +
 crates/athena-context/prompts/planner.md   |   1 +
 crates/athena-context/src/briefs.rs        |  72 ++++++-
 crates/athena-context/src/lib.rs           |   4 +-
 crates/athena-context/src/outputs.rs       |  74 ++++++-
 crates/athena-core/src/admission.rs        | 321 ++++++++++++++++++++++++++++-
 crates/athena-core/src/reading.rs          |   1 +
 crates/athena-core/src/run/mod.rs          |  14 +-
 crates/athena-core/src/run/planning.rs     |  15 ++
 crates/athena-core/src/run/tests.rs        |   4 +
 crates/athena-core/src/run_pin_tests.rs    |   1 +
 crates/athena-core/src/status.rs           |   1 +
 crates/athena-core/src/store.rs            |   1 +
 crates/athena-core/tests/refusal_corpus.rs | 238 +++++++++++++++++++++
 crates/athena-data/src/records.rs          |  80 +++++++
 docs/design/job-planning.md                |  23 ++-
 docs/system-facts.md                       |   4 +-
 22 files changed, 910 insertions(+), 18 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41644523769 for 3f739015a04d085e969f2cbf3972f0a188aad1f4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41644523769 for 3f739015a04d085e969f2cbf3972f0a188aad1f4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41644523769 for 3f739015a04d085e969f2cbf3972f0a188aad1f4)
- ci-result: `not a check` passed (Check results for 3f739015a04d085e969f2cbf3972f0a188aad1f4 come from the CI run https://github.com/doodla/athena/actions/runs/41644523769 (615s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
