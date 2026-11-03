# Job 20261031-031651-c3k6: verify "Refuse plan open questions that reading the code would settle"

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The planner designs from the reading and from the code paths it first finds. It does not enumerate e
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-leaves-code-answerable-questions-open-and-skips-adjacent-existing-path-fails-planning-rounds (20261031-030110-hnnx)
> Cause: The planner designs from the reading and from the code paths it first finds. It does not enumerate every existing caller and mode that reaches the function it changes, such as the `[parallel] max_concurrency` path into landing. It also may ...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261031-030054-6wkt (job 20261030-013555-f5qm); prevention it proposed: Add a plan-structure check that runs before each critic round. (a) A plan may not list an open question that is marked answerable by reading code. The planner must settle it by reading the code and cite the file and line. (b) Any plan that ...
> Introspection 20261031-031632-fdjf (job 20261031-030119-tw3v); prevention it proposed: Add a required step before the critic in planning for any change to a core flow such as landing. The planner must list every state the change touches, including records and job state read from the landing branch, and every existing caller a...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: Athena's own plan checks (plan_findings in admission.rs) accept open questions the planner could settle by reading code (e.g. whether a path lands today, or what an old job's records look like), so the critic rounds were spent on them; jobs 20261030-013555-f5qm and 20261031-030119-tw3v both stopped at round 3 this way. Fix: add a mechanical major finding in plan_findings, run before each critic round, for code-answerable open questions, and tell the planner in its brief to settle them with a file and line.

Assumptions:
- Code survey: read crates/athena-core/src/admission.rs (plan_findings at line 170 and dependents_findings at 314, which already run before each critic call), crates/athena-core/src/run/planning.rs (planning_round runs plan_findings, then up to MAX_MECHANICAL_PASSES revisions with no critic call), crates/athena-data/src/records.rs (JobPlan has open_questions: Vec<String> and dependents; no new record or data file is added, so check_record_file/AnyRecord/round-trip tests are unaffected). Entry point for the test: plan_findings with a JobPlan built like the existing dependents_findings_for fixture. The property is measured directly on the Finding list it returns.
- Job 20261030-013555-f5qm (finding: a plan left code-answerable questions open and skipped the [parallel] max_concurrency path into landing): addressed by the rule that refuses code-answerable open questions, and a dependents check that a plan for a core-flow file names every caller/mode; that job's own landing work is not redone here and its scope (drain.rs, landing.rs, CHANGELOG.md, tests/drain.rs, async-drain.md) is excluded; it changed no files, so nothing is kept or dropped.
- Job 20261031-030119-tw3v (finding: the plan left record-loss and ordering questions open): same fix, and it no longer applies as a retry; the same scope is excluded and it changed no files.
- Existing code: crates/athena-core/src/admission.rs, crates/athena-core/src/run/planning.rs, crates/athena-context/src/briefs.rs, docs/design/job-planning.md.
- The detection is a deterministic heuristic: an open question that cites a source path (a crates/ path or a .rs file or file:line) or says the answer is found by reading or checking the code is code-answerable. Questions for the user that name no code path stay allowed.
- The planner brief in briefs.rs gains one short sentence (planner has 334 tokens of headroom); the critic and other prompt sets are not touched.
- Check (b) from the introspections is kept small: when a plan touches a file whose dependents section is present, no new schema field is added; the existing dependents rule and brief wording carry the 'list every caller and mode' instruction.

## Acceptance criteria

1. [content] The change description names the root cause: plan_findings in crates/athena-core/src/admission.rs let code-answerable open questions through to the critic, and the planner brief did not require settling them by reading code; Grep finds the doc comment for the new rule in admission.rs.
2. [behavior] Run `cargo test -p athena-core admission`: a new test builds a plan whose open_questions contains a question citing a crates/ path (e.g. whether `crates/athena-cli/src/main.rs` passes max_concurrency into landing) and asserts plan_findings returns a Major finding located at open questions naming the question; the test fails on the base commit and passes with the change.
3. [behavior] Run `cargo test -p athena-core admission`: a new test shows a plan whose open question is a user decision naming no code path produces no open-question finding, and a plan with no open questions is unchanged, so the rule is not a blanket refusal.
4. [behavior] Run `cargo test -p athena-core run`: a planning test shows a plan with a code-answerable open question is sent back to the planner without a critic call (mechanical pass), not counted as a critic round.
5. [content] The planner brief in crates/athena-context/src/briefs.rs tells the planner to settle questions answerable from code by reading it and citing file and line, and to list every existing caller and mode that reaches a changed function; docs/design/job-planning.md and CHANGELOG.md describe the new rule.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including prompt token budget tests for the planner set.
7. [content] The change refuses more plans: expected 5% of newly refused plans, and the higher refusal rate is intended because those plans spent critic rounds on questions code could answer.

## Expected deliverables

- crates/athena-core/src/admission.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Landing queue work of jobs 20261030-013555-f5qm and 20261031-030119-tw3v (drain.rs, landing.rs, tests/drain.rs, async-drain.md): not redone here; those jobs changed no files.
- Changing athena.toml, defaults.toml, .github/workflows, or the JobPlan record schema in crates/athena-data
- Adding model calls or changing the critic prompt
- Conflict return, send_back and old-job record handling

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The rule and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the new rule; fmt, clippy and test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the shared plan_findings gate, not for one job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a code check, not prose only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run plan_findings on real plans.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule plus one brief sentence.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff: the doc comment on `open_questions_findings` and on the `plan_findings` bullet list, and the CHANGELOG entry. Run: `git diff -- crates/athena-core/src/admission.rs CHANGELOG.md`. Observes: The admission.rs diff adds a doc comment with 'Root cause:' naming `plan_findings` letting code-answerable open questions through to the critic and the planner brief not requiring them to be settled by reading code; the CHANGELOG diff adds the same sentence. Grep for 'Root cause' in admission.rs finds it.
- Criterion 2: New unit test `an_unaddressed_open_question_is_a_major_finding_naming_it` in the admission.rs tests module, built on a new helper `open_question_findings_for(Vec<&str>)`. Run: `cargo test -p athena-core admission`. Observes: For each question the test asserts exactly one finding that is Major, source Athena, location 'open questions', with the question text in its summary. Questions: 'Does the [parallel] max_concurrency path reach landing?' and 'What do old job records look like on the landing branch?' (the failure class's wording, no path and no phrase); 'Is the answer found by reading code?'; 'Does `crates/athena-cli/src/main.rs` pass max_concurrency into landing?'; 'landing.rs:120 handles a conflict how?'. A second test asserts a `User:` question saying 'is this found by reading the code?' is also Major. The test fails on the base commit and passes with the change.
  Fails without the behavior: On the base commit `plan_findings` has no open-question rule and returns no finding for these plans, so the Major-finding assertion fails. The two path-free motivating questions also fail under the previous path-and-phrase heuristic, so the test pins the cause, not a synthetic path string.
  Test seam: Entry point `admission::plan_findings(&JobPlan, &Reading, &Policy, &dyn Fn(&str)->bool)`, called directly with a JobPlan literal, `reading()` and an empty-command Policy, as the existing `dependents_findings_for` helper (admission.rs line 783) does. Nothing is doubled. The sibling helper `open_question_findings_for` is added in step 4 and sets `dependents: vec![complete_dependent()]` plus `open_questions`, returning only findings at 'open questions'.
- Criterion 3: New unit test `a_user_or_host_question_and_an_empty_list_are_not_refused` in the admission.rs tests module, using the same `plan_findings` fixture. Run: `cargo test -p athena-core admission`. Observes: No finding is located at 'open questions' for: 'User: Should the CHANGELOG entry go under Added or Changed?'; '  user: does the user want edits to `crates/athena-core/tests/refusal_corpus.rs`, outside the deliverables?' (a scope question that cites a path, lower-case prefix, leading spaces); 'Host: Is the production host disk large enough?'; '**User:** Should we publish on docs.rs?'. A plan with `open_questions: []` yields the same findings as `dependents_findings_for` with the same dependents, which is none. The unprefixed lookalike 'Username: ...' is refused, so the prefix match is exact.
  Fails without the behavior: If the rule refused every non-empty open_questions, or refused any question citing a path or '.rs', the prefixed cases would produce a finding and the test would fail.
  Test seam: Same production entry point `plan_findings` and the same fixture and helper as criterion 2 (added in step 4). No double.
- Criterion 4: New planning test `a_plan_with_an_unaddressed_open_question_is_revised_before_the_critic` in run/tests.rs, using `run_planned` with `repo_with_planning(3)`. Run: `cargo test -p athena-core run`. Observes: The planner queue is [a good_plan() variant whose open_questions is ['Does the [parallel] max_concurrency path reach landing?'], a good_plan() variant whose open_questions is ['User: Should the change also cover the design docs?']]; the critic queue is [accepting_critique()]. The call order is Reader, Planner, Planner, Critic, Producer, Verifier, so no Critic call lies between the planners. Planner prompt 2 contains the finding text 'has no addressee' and the question; critic prompt 1 contains '## Plan under review (round 1)' and the `User:` question, so the mechanical pass used no round. An existing test, `a_plan_that_still_breaks_a_rule_after_two_mechanical_passes_stops_without_the_critic`, covers the cap path.
  Fails without the behavior: Without the rule the first plan passes `plan_findings` and goes to the critic: the order is Reader, Planner, Critic, planner prompt 2 never carries the finding, and the assertions fail.
  Test seam: Production entry point `Athena::go` through the existing `run_planned` helper (run/tests.rs line 611), which drives `planning_round`. The scripted planner and critic queues are injected by the existing `planning_script` (line 577) into `athena_with`. The seam exists today. Step 6 adds only a plan-JSON helper `plan_with_open_questions(&[&str])` that sets `open_questions` on `good_plan()`. The queues supply plan text only; the rule under test is production code.
- Criterion 5: New wording test `planner_brief_tells_the_planner_to_settle_code_questions_by_reading` in briefs.rs, on `planner_brief(&planner_inputs(None))`; the docs and CHANGELOG are shown by the diff. Run: `cargo test -p athena-context briefs`. Observes: After '## Report' the brief contains: settle every question the code can answer by reading it and cite file and line; begin an open question with `User:` or `Host:` and only for what the code cannot show; list every existing caller and mode that reaches a changed function. `git diff -- docs/design/job-planning.md CHANGELOG.md` shows the rule in the 'Athena's own checks' paragraph and an Unreleased Added entry, with the caller-and-mode half marked prose only.
  Fails without the behavior: Without `OPEN_QUESTION_RULE` in the Report section the wording assertions fail.
  Test seam: Entry point `briefs::planner_brief(&PlannerInputs)` with the existing `planner_inputs(None)` fixture (briefs.rs line 2868), as used by the refusal-corpus wording test. It exists today; nothing is doubled.
- Criterion 6: Run the three Athena checks: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Run: `cargo test --locked`. Observes: All pass. `planner_and_critic_frames_are_short_and_fit_the_budgets` (athena-context lib.rs) and `system_facts_budgets_match_computed` (athena-cli tests/budgets.rs) stay green because planner.md and docs/system-facts.md are unchanged and the brief is outside the budget sets.
  Fails without the behavior: A formatting slip, a clippy warning such as an unused helper, or an edit that changes a budget set would fail one of the three checks.
  Test seam: Whole-workspace fmt, clippy and test runs; no seam.
- Criterion 7: Read the diff for the intent statement in the doc comment of `open_questions_findings`, the job-planning.md paragraph, the CHANGELOG entry and the corpus table comment. Run: `git diff -- crates/athena-core/src/admission.rs docs/design/job-planning.md CHANGELOG.md crates/athena-core/tests/refusal_corpus.rs`. Observes: Each added passage says the rule refuses more plans, 'expected 5% of newly refused plans (the reading's estimate, not a measurement)', and that the higher refusal rate is intended because those plans spent critic rounds on questions code could answer.

Planned files:

- crates/athena-core/src/admission.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): The change refuses more plans: expected 5% of newly refused plans, and the higher refusal rate is intended because those plans spent critic rounds on questions code could answer.

## The change

Base 982d173133ab → candidate 66fbf9c38010. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  12 +++
 crates/athena-context/src/briefs.rs        |  29 +++++-
 crates/athena-core/src/admission.rs        | 155 ++++++++++++++++++++++++++++-
 crates/athena-core/src/run/planning.rs     |   4 +-
 crates/athena-core/src/run/tests.rs        |  50 ++++++++++
 crates/athena-core/tests/refusal_corpus.rs | 132 ++++++++++++++++++++++++
 docs/design/job-planning.md                |  25 ++++-
 7 files changed, 402 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41658443767 for 66fbf9c380106deee25f23ebc540e4825a46d9ef)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41658443767 for 66fbf9c380106deee25f23ebc540e4825a46d9ef)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41658443767 for 66fbf9c380106deee25f23ebc540e4825a46d9ef)
- ci-result: `not a check` passed (Check results for 66fbf9c380106deee25f23ebc540e4825a46d9ef come from the CI run https://github.com/doodla/athena/actions/runs/41658443767 (605s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including prompt token budget tests for the planner set.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
