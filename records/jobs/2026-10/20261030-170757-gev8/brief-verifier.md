# Job 20261030-170757-gev8: verify "Make refusal-rule plans run against a corpus of real request shapes"

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The planner specifies matching rules (parts parser, stems, stop list) by reasoning about the intende
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-for-new-refusal-rule-untested-against-real-request-shapes-fails-planning-rounds (20261029-181827-rnrn)
> Cause: The planner specifies matching rules (parts parser, stems, stop list) by reasoning about the intended behavior and hand-checking a few cases. It never executes them against a corpus of real past requests and readings. Specification gaps (un...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261029-181811-bs86 (job 20261029-111202-2bpc); prevention it proposed: Before the critic's first round, the planning stage for any job that adds or changes a reading-refusal or matching rule should have to do two things. First, list every input form the accepted reading names and map each to a parser case in t...
> Introspection 20261030-170739-hq09 (job 20261030-060751-5389); prevention it proposed: Add a planning-stage gate for any plan that adds or changes a refusal or matching rule. The gate runs the plan's stated rule, as a small executable spec, against a stored corpus of real past requests and readings. The corpus must include th...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Retries 20261029-111202-2bpc and 20261030-060751-5389 (both failed in round 3 of planning). Root cause: refusal and matching rules (parts parser, stems, stop list) are specified by reasoning and hand-checked on a few cases, never executed against real request and reading shapes. Fix: add a stored corpus of real request shapes, with a test that runs every reading-acceptance check against it, and a planner rule that requires this for any plan adding or changing such a rule. Audience: the Athena planner, critic and maintainers.

Assumptions:
- Root cause named in the change description: refusal and matching rules are never executed against real request and reading shapes before they are planned or landed, so spec gaps (numbered N. and N) parts, stop-list/stem order, cited-but-unrelied jobs, brand-word symbols) surface only in critic rounds.
- Job 20261029-111202-2bpc: its reading-coverage code change (parts parser, stems) is not retried here; its findings (missing N./N) parts, stem/stop-list order on the Steward bullet, quoted-complaint exemption, rebuilt-text nit) become corpus cases and planner guidance, so the class is covered. No candidate files were recorded, so none are kept; its out_of_scope items carry over (no athena.toml, defaults.toml, workflows, records/, reader or verifier prompts).
- Job 20261030-060751-5389: its findings (citing a job is not relying on it, GitHub/env-var/let-binding not symbols, crate-relative paths, run_pin_tests.rs scope) become corpus cases and planner guidance; its candidate is dropped, no files kept; out_of_scope carries over (no planner/critic behavior changes beyond the one added planner rule, no athena.toml or defaults.toml).
- Code survey: read crates/athena-core/src/reading.rs (is_bullet at 2239, request_text_acceptance 2249, check_reading_acceptance 3013 as the gate every reading passes, check_governance_coverage 3089) and crates/athena-context/src/briefs.rs (READING_RULES at 1681, rule-section test near 3362) and crates/athena-context/prompts/planner.md (15 lines, 494 tokens of headroom). The corpus is a Rust test fixture, not a record file under records/, so check_record_file and AnyRecord are not involved; if a data file is added under crates/, the round-trip test must cover it. The property is measured directly by calling check_reading_acceptance with fixture requests and readings.
- The corpus holds request shapes seen in the cited jobs (Steward three-bullet acceptance, numbered 1./2./3. parts, (a)(b)(c), root-cause requests citing several rejected jobs, retry complaints) with expected accept/refuse outcomes. Text rebuilt from the brief is labeled so in the fixture.
- Prompt budget: only planner.md is edited (about 100 tokens added, under its 494 headroom); the reader and verifier prompts are untouched.

## Acceptance criteria

1. [content] The change description names the root cause (refusal and matching rules are not executed against real request and reading shapes before landing) and the fix sits in the shared gate test plus the planner rule, not in one job's check; checkable in the diff and CHANGELOG.md.
2. [content] crates/athena-context/prompts/planner.md gains a rule: a plan that adds or changes a refusal or matching rule must list the input forms it handles, run the rule against the stored corpus, and cite the results in the plan; the planner prompt stays within its 1300-token budget.
3. [behavior] A new corpus test in crates/athena-core (run with cargo test -p athena-core) feeds at least eight real request shapes through check_reading_acceptance, including Steward bullets, N./N)/(N) numbered parts, a root-cause request citing rejected jobs the reading never relies on, and a retry complaint, asserting the expected accept or refuse for each. It fails on the code before the change (no corpus exists) and passes after.
4. [behavior] A mechanism test fails if any check_* function called by check_reading_acceptance is not exercised by the corpus test, so a new refusal rule cannot land without corpus cases; run with cargo test -p athena-core.
5. [behavior] A briefs test in crates/athena-context (cargo test -p athena-context) asserts the planner brief contains the corpus rule; it fails before and passes after.
6. [behavior] Expected 0% of newly refused readings: the change adds tests and a planner rule and does not tighten any refusal rule, so the refusal rate is unchanged.
7. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass across the workspace.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (create)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Implementing the parts-coverage check or the prior-job existence check from jobs 20261029-111202-2bpc and 20261030-060751-5389; they are separate follow-ups that will use this corpus.
- Editing athena.toml, defaults.toml, workflows, records/, or the reader, critic, producer and verifier prompts.
- Candidate files of jobs 20261029-084645-sza7, 20261029-110953-bw7k and 20261026-042558-5p65 and similar rejected jobs: dropped, not kept.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Corpus and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New corpus and mechanism tests; fmt, clippy and test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the planning and gate seam rather than one rule.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces that every acceptance check is covered by the corpus.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the gate on request shapes.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that real request shapes get the right outcome.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Fixtures state their source.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixtures contain no secrets or personal data.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: corpus, test and one planner rule.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff. CHANGELOG.md (Unreleased), the new docs/decisions.md entry, the doc comment on `refusing_check` and the header of refusal_corpus.rs each name the root cause: refusal and matching rules are never executed against real request and reading shapes before they are planned or landed. They also state the replay relationship and the residual parser/stem limit. The diff to reading.rs adds `refusing_check` and turns `check_reading_acceptance` into a wrapper; no check_* body changes. Run: `git diff -- CHANGELOG.md docs/decisions.md crates/athena-core/src/reading.rs crates/athena-core/tests/refusal_corpus.rs`. Observes: The root-cause sentence appears in the changelog, the decisions entry and the code comments. The fix is the shared gate (refusing_check, corpus test, mechanism test) plus the planner rule, with no change to any single check's logic.
- Criterion 2: Read the planner.md diff for the short clause (map each input form of a refusal or matching rule to a refusal_corpus case), then run the two budget tests. `planner_and_critic_frames_are_short_and_fit_the_budgets` checks that planner.md stays under 500 tokens and the planner set within 1300. `system_facts_budgets_match_computed` checks the system-facts line. The full rule, with the input forms, the case ids from refusal_corpus.rs and the producer/verifier command, is checked under criterion 5. Run: `cargo test --locked -p athena-context planner_and_critic_frames_are_short_and_fit_the_budgets && cargo test --locked -p athena-cli --test budgets`. Observes: Both pass. planner.md is at most 1497 bytes (499 tokens or fewer) and the planner set is within 1300 (806 now). The diff shows the rule clause and the trimmed existing words.
- Criterion 3: Run the corpus test. `corpus_cases_get_the_expected_outcome` iterates at least twelve cases. Each builds a request and a Reading, calls `refusing_check`, and asserts Accept or Refuse(check name) plus the bullet count from `request_text_acceptance`. It also asserts that `check_reading_acceptance` agrees (Ok/Err and identical error text). Cases (source job in each): (1) Steward three-bullet acceptance: accepted, 3 bullets. (2) `1.`/`2.`/`3.` parts: 3 bullets. (3) `1)` parts: 0 bullets, labeled the current parser behavior and a known gap. (4) `(a)(b)(c)` parts: 0 bullets, same label. (5) root-cause request that cites two rejected job ids as evidence ('Introspection ... (job ...)') with both passed as `referenced` and a reading that names neither: accepted, since citing is not following. (6) the same request with 'Follow up on job <id>' and a reading that does not name it: refused by check_prior_result_and_scope. (7) retry complaint with `attempt` set, reading names the job and lists deliverables and out_of_scope: accepted. (8) the same retry with the job id omitted: refused by check_prior_result_and_scope. (9) a criterion saying 'the producer's report lists ...': refused by check_criteria_evidence. (10) code reading with a behavior criterion and no `Code survey:`: refused by check_code_survey; with a survey naming crates/ paths: accepted. (11) request naming athena.toml, reading neither lists nor excludes it: refused by check_governance_coverage. (12) modify deliverable `src/reading.rs` (crate-relative) with a `repo_files` set holding only the crates/ path: refused by check_boundary_and_repo_files. (13) summary relying on an absent `crates/athena-core/src/prior_job_check.rs`: refused by check_reading_references; the same with 'does not exist yet': accepted. (14) text with `GitHub/env-var` and `owner/repo` tokens: accepted (brand word and env var are not repo paths). Request text rebuilt from briefs is labeled so; no secrets or personal data. A further test checks the union of `forms` tags covers the fixed form list. Run: `cargo test --locked -p athena-core --test refusal_corpus`. Observes: All cases pass; each outcome matches the live gate and names the refusing check. A mismatch fails with the case id and source job.
  Fails without the behavior: Before the change neither the test file nor `refusing_check` exists, so the command fails to compile and runs no tests. If the gate stopped refusing a retry that does not name its prior job, or accepted a modify of a missing file, the matching case would fail with its id and source. If the parser started reading `1)` parts, the pinned bullet count would fail.
  Test seam: Entry point: athena_core::reading::refusing_check, which check_reading_acceptance wraps and the test also calls. Guards: capability Some and question None for most checks; `repo_files` Some for the path checks; `attempt` or `referenced` plus a follow-up phrase for the prior-result check. Fixtures: a `reading(...)` builder in the test file built from athena_data Reading/Criterion/Deliverable/Forecasts (field set copied from the `final_reading` test fixture in reading.rs), `EarlierAttempt::default()` with job and request set, and a BTreeSet<String> of repo paths, all passed as arguments. No double replaces any check. Exists today: the checks, `Reading`, `EarlierAttempt` (Default) and `check_reading_acceptance` are pub. Step 1 adds `refusing_check`; steps 2 and 3 add the test file.
- Criterion 4: Run the mechanism tests in the same file. `acceptance_check_names(source)` extracts `step!(check_*(` names from the body of `refusing_check` in reading.rs (include_str!) and asserts at least six. `uncovered(names, covered)` returns names with no corpus case expecting a refusal by that name; `covered` comes from running every corpus case through `refusing_check`. Tests: the real source has no uncovered check; the real source with one inserted line `step!(check_new_rule(reading));` yields exactly ['check_new_rule']; the body of `check_reading_acceptance` holds no direct `check_*(` call. Run: `cargo test --locked -p athena-core --test refusal_corpus every_acceptance_check_has_a_refusing_corpus_case an_extra_check_with_no_case_is_reported no_check_bypasses_refusing_check`. Observes: The six current checks (check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_governance_coverage, check_boundary_and_repo_files, check_reading_references) each have a refusing case. The doctored source reports exactly the invented check.
  Fails without the behavior: If a check_* step were added to `refusing_check` with no refusing case, `every_acceptance_check_has_a_refusing_corpus_case` would fail naming it; `an_extra_check_with_no_case_is_reported` proves this on a doctored copy and would fail if the extractor missed steps. If a check were called directly in `check_reading_acceptance`, `no_check_bypasses_refusing_check` would fail.
  Test seam: Entry points: athena_core::reading::refusing_check (live coverage) and the text of crates/athena-core/src/reading.rs via include_str!("../src/reading.rs"). Fixture: the criterion 3 corpus. The extractor and `uncovered` are small pure functions in the test file; the doctored source is the real text plus one inserted line, so the logic under test is never inside the double. Step 1 adds the `step!`-based refusing_check; step 4 adds the mechanism tests.
- Criterion 5: Add `planner_prompt_and_brief_carry_the_refusal_corpus_rule` to the briefs tests, beside `planning_prompts_and_brief_require_a_named_test_seam`. It builds `planner_brief(&planner_inputs(None))` and takes the text after '## Report'. It asserts the Report contains the pinned words: 'refusal or matching rule', 'input forms', 'corpus case', 'cargo test -p athena-core --test refusal_corpus', and a clause saying the planner cannot run it (so no results may be cited). It asserts `crate::PLANNER_FRAME.text` contains 'refusal_corpus'. Run: `cargo test --locked -p athena-context planner_prompt_and_brief_carry_the_refusal_corpus_rule`. Observes: The composed planner brief carries the full rule (list input forms, map each to a corpus case or a producer-added case, demonstrate with the corpus test) and the planner prompt carries the short clause.
  Fails without the behavior: Before the change neither PLANNER_FRAME.text nor the planner_brief Report section contains the rule, so the first assert fails. Removing either copy later fails it again.
  Test seam: Entry points: athena_context::briefs::planner_brief and the PLANNER_FRAME asset (include_str! of prompts/planner.md). Fixture: the existing `planner_inputs(None)` helper in the briefs tests module. Exists today (briefs.rs ~2081-2093). Step 5 adds the sentences and step 5 also adds the test.
- Criterion 6: Run the whole athena-core suite. Existing reading.rs unit tests call `check_reading_acceptance` directly (the `accept` helper at ~6278 and the code-survey, evidence, governance and reference tests) and pass unmodified. The corpus test asserts, for every case, that `check_reading_acceptance(...).is_ok()` equals `refusing_check(...).is_none()` and that the error strings match. Corpus accept cases stay accepted. The diff touches no check body. Other callers (the reader loop at reading.rs:637, `check_reading_text` at 3427 feeding BinaryChecker replay) call the unchanged wrapper; the existing replay and check-reading tests already guard their error text. Run: `cargo test --locked -p athena-core`. Observes: All pre-existing tests pass unmodified, plus the corpus accept cases and the equality assert, so 0% of shapes are newly refused.
  Fails without the behavior: If the refactor changed any check's order, short-circuit or error text, the existing unit tests that assert on messages (for example the `Code survey:` text) would fail, and so would the corpus equality assert. A newly refused real shape would fail its accept case.
  Test seam: Entry point: athena_core::reading::check_reading_acceptance, the unchanged public gate, called by the existing unit tests (`accept` helper, exists today) and by the corpus test (step 2). No double.
- Criterion 7: Run the three Athena checks across the workspace. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0, including planner_and_critic_frames_are_short_and_fit_the_budgets, the cli budgets test and the xtask policy scan over the new fixtures.
  Fails without the behavior: Any formatting, lint or test failure anywhere in the workspace fails this criterion, including the 500-token planner.md cap if the rule is added without trimming existing text.
  Test seam: Entry points are the workspace commands. No fixture. The budget tests exist today (lib.rs:394, crates/athena-cli/tests/budgets.rs); step 6 checks them.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (create)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 5628d16e3b53 → candidate 832b539a8e5b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  18 +
 crates/athena-context/prompts/planner.md   |  11 +-
 crates/athena-context/src/briefs.rs        |  32 +-
 crates/athena-core/src/reading.rs          |  54 ++-
 crates/athena-core/tests/refusal_corpus.rs | 711 +++++++++++++++++++++++++++++
 docs/decisions.md                          |  47 ++
 6 files changed, 860 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1857 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets and cargo test pass across the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
