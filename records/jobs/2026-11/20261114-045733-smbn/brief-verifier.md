# Job 20261114-045733-smbn: verify "Prove the job budget reserve is checked before every spend, incl."

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Athena gives each job a fixed budget and only checks it after planning has already spent from it. Ad
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: job-budget-too-small-for-planning-plus-producer-verifier-caps-fails-before-code (20261028-091806-tng9)
> Cause: Athena gives each job a fixed budget and only checks it after planning has already spent from it. Admission and retry do not compare the remaining budget with the cap the producer and verifier need, which is fixed at $5.75. Planning rounds,...
> Seen 7 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261028-091754-xkyn (job 20261028-082151-67fa); prevention it proposed: Add a budget pre-check at admission, at retry and before each planning round. It should compute the reserve the producer and verifier need (the sum of their caps) and refuse to start or continue a planning round if the remaining budget minu...
> Introspection 20261030-151759-gmz2 (job 20261030-033434-3v30); prevention it proposed: Add a budget preflight in code. At admission, at retry, and before drain dispatches a job, compare the job's remaining budget with the planner cap plus the producer and verifier caps. If it falls short, do not start planning. Mark the job h...
> Introspection 20261030-153131-skzk (job 20261030-151827-48rm); prevention it proposed: Add one check, run at admission, when the queue picks a job, and before any retry or re-ask. It compares the job's remaining budget with the planning cost plus the producer and verifier caps. If the budget is short, it holds the job before ...
> Introspection 20261030-153335-xdtk (job 20261030-151810-0n3b); prevention it proposed: Add a code check that runs when a job fails at its budget cap, and again before a retry. It compares the remaining budget with the stage caps. If the cap is the cause, drain should hold that job with a clear reason, record the cap as the bl...
> Introspection 20261030-165437-3n07 (job 20261030-033434-3v30); prevention it proposed: Add a check at admission, before each retry and after each planning round. It compares the job's remaining budget with the producer cap plus the verifier cap. If the budget is short, it holds the job with a clear "budget below stage caps" r...
> Introspection 20261030-214133-cp86 (job 20261030-151818-2c8g); prevention it proposed: Define the reserve in one function that sums the stage caps from config, as planner + critic per round for max_rounds, plus producer and verifier. Use that same function in admission, retry, dispatch and the per-round check. Add a test that...
> Introspection 20261103-055632-pxgr (job 20261103-055218-kj1t); prevention it proposed: Before dispatching a retry, and at admission, check the job's remaining budget (cap minus all earlier attempts and planning spend) against the planning stage cap plus the producer and verifier caps. If it falls short, hold the job and show ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

The root cause (per-invocation budget check made after planning spent, with a reserve that left out the critic and later rounds) is already fixed in the repo by `reading::budget_reserve` and `Athena::preflight`. This change closes the remaining gap: a test per entry point (admission, retry, dispatch, planner, critic, run, drain, and a resumed job with prior spend) that fails when its preflight is removed. It supersedes the stalled jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm; the audience is the operator and verifier.

Assumptions:
- Code survey: read crates/athena-data/src/config.rs (planning_usd, critic_usd, run_usd, budget_required, check_budget at 894-949), crates/athena-core/src/reading.rs (ReserveStage and budget_reserve 94-140, check_budget and preflight 467-503, first_reading 1187 with preflight at 1203, retry 1262 with preflight at 1303), crates/athena-core/src/run/mod.rs (dispatch preflight 1108, Run preflight 1367), crates/athena-core/src/run/planning.rs (preflight at 116 and 336), crates/athena-cli/src/drain.rs (budget short hold, doc at 266). The entry points are Athena::ask, retry, go and drain, and the order is admission, dispatch, each planner and critic, then run. No record or data file under crates/ is added, so no records allow-list or AnyRecord round-trip change. The property is measured by comparing the job's remaining budget (job_usd minus spent) with budget_reserve at each stage, and by the error type BudgetShort.
- Current state: CHANGELOG.md 2184-2200 already records the root-cause fix and the per-stage preflight, and athena.toml has job_usd = 50.00 against a reserve of 2 + planning + 15, so the shipped config no longer reproduces the $8.00 failures. The three previous jobs (20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm) stopped in planning with no candidate and no changed files. Their critic findings concern different features (an interrupted-job takeover, drain ordering), not this cause, so they no longer apply to this request. Their file scopes (run/mod.rs, disk.rs, manage.rs, drain.rs, plan.rs and the athena.toml budget) are excluded here except where listed as deliverables, and their work is dropped.
- Existing code: this extends reading::budget_reserve, Athena::preflight, ReserveStage and BudgetShort in crates/athena-core/src/reading.rs and Config::planning_usd and budget_required in crates/athena-data/src/config.rs. It builds no second reserve computation.
- The gap to close, if the audit finds one, is a path where spend happens without a preflight. The likely places are a resumed or interrupted job whose earlier spend is not counted, a retry judged at spend 0 while its seeded stage already ran, and a drain dispatch. If the audit finds no gap, the change is tests, a short doc note and a CHANGELOG entry only.
- Expected newly refused readings: 0%. The shipped job_usd of 50.00 exceeds the largest reserve, so no past accepted reading is refused. No new refusal rule is added. The preflight already exists, and the change only proves it and extends it to any uncovered entry point.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry naming the root cause (the budget checked per invocation, after planning had spent, with a reserve that omitted the critic and later rounds) and jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm, and stating that 0% of past accepted readings are newly refused.
2. [content] The reserve for every ReserveStage is still computed only in reading::budget_reserve, from Config methods. Grep for `planner_usd + ` or `producer_usd + ` under crates/athena-core/src shows no second sum of caps outside Config.
3. [behavior] A new test per entry point (ask, retry, dispatch, planner round, critic round, run stage, drain, and a resumed job with prior spend) gives a job a budget one cent below budget_reserve for that stage, asserts a BudgetShort error before any invocation is recorded, and asserts success at exactly the reserve. Run `cargo test` and check that these tests pass.
4. [behavior] The cause is tested, not one job: a table test varies each cap in a Config (reader, planner, critic, producer, verifier, max_rounds) and asserts the reserve and the refusal threshold move by exactly that amount at every stage. Removing the preflight call in any one entry point (shown by the verifier running `cargo test` on a mutated tree, or by reading the test) makes its test fail. Run `cargo test`.
5. [behavior] Through the drain entry point, a queued item whose remaining budget is below the reserve is left queued with one `item <id> is left queued` line and outcome `budget short`, and the next affordable item still runs. Run `cargo test -p athena-cli`.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, as a recorded Athena check on the final tree.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Changes to athena.toml, defaults.toml, .github/workflows or records/; the [budget] values stay as they are
- The interrupted-job takeover and the retry head-staleness work from job 20261028-082151-67fa (run/mod.rs timeout_seed, disk.rs, manage.rs)
- Drain ordering and plan.rs changes from jobs 20261030-033434-3v30 and 20261030-151827-48rm
- Any new refusal rule beyond the existing preflight; changing the producer, verifier or planner prompts
- A partial-plan resume for the planner

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that a short budget is held before spend, not that a function exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One reserve function and a cap-varying table test cover the whole class.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test pass is a criterion.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Add one bullet at the top of `### Fixed` under `## [Unreleased]` in CHANGELOG.md. It starts with the marker sentence 'The job budget is checked before every spend, and each entry point is proved to do it.' It names the root cause: the budget was checked per invocation, after reading and planning had already spent, against a reserve that left out the critic and every later planning round. It names jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm. It says the audit found no spend path that starts planning before the reserve is judged, that `correct` and the re-read after a configuration change keep only the reader cap check and are pinned by tests, that no refusal rule is added, and that 0% of past accepted readings are newly refused. It must not contain '3 of the 3 cited jobs' (negated at refusal_corpus.rs:1148 and 1229), 'expected N% of newly refused readings', or any `reading::check_` name, because other tests find their items by those phrases. The test `changelog_names_the_budget_reserve_fix` in `budget_scan_tests` (end of run/tests.rs) reads the file with `include_str!("../../../../CHANGELOG.md")` (the depth run/tests.rs:311 uses for athena.toml), finds the bullet by the marker up to the next line starting with `- ` or `#`, collapses white space, and asserts the three ids, 'critic', 'after planning' and '0% of past accepted readings are newly refused'. Run: `cargo test -p athena-core changelog_names_the_budget_reserve_fix`. Observes: The test passes only when the new entry carries the root-cause words, all three job ids and the 0% statement. The verifier can also run `git diff -- CHANGELOG.md` and read the entry.
  Fails without the behavior: Deleting the entry, or dropping one job id or the 0% sentence, fails the test. The older entry at CHANGELOG.md:2184-2200 does not satisfy it, because the test reads only the bullet that starts with the marker.
  Test seam: Production entry point: none (a content criterion). The test reads CHANGELOG.md with `include_str!`, a pattern that exists at run/tests.rs:311 and in refusal_corpus.rs. No fake. The test is added in step 5 and the entry in step 7.
- Criterion 2: The only sums of caps are in crates/athena-data/src/config.rs:900, 923 and 934, and `budget_reserve` (reading.rs:116-138) calls them. A grep of `planner_usd + ` or `producer_usd + ` under crates/athena-core/src finds only oracle lines in reading.rs after its `mod tests` (1669, 1681, 1682, 1691, 1705, 1739). The new `no_second_sum_of_stage_caps_in_core` replaces the grep with a syntax-tree scan, per the round-3 decision. It walks crates/athena-core/src from `env!("CARGO_MANIFEST_DIR")`, parses each file with `syn::parse_file` and visits it. It skips `#[cfg(test)]` subtrees only, and files named `tests.rs`, `*_tests.rs` or `test_support.rs`. It flags a `+` or `+=` with a cap-name identifier in either operand, and `.sum` over a cap name. An empty allow-list const is the only exemption. The structural pass asserts `fn budget_reserve` is defined once and that all calls of the four Config sum methods in core non-test code are inside it (today reading.rs:119-134 only; a grep of the four method calls across crates/ finds only reading.rs and athena-data/src/config.rs). I grepped every other production mention of the five cap names in core (planning.rs:161, 365; run/mod.rs:1260-1262; producer.rs:362; verification.rs:216, 240; auditor.rs:379; pin.rs:459; introspect.rs:1893; steward.rs:1288; audience.rs:265; reading.rs:586, 663; hestia.rs:2863; manage.rs:295). Each is a single argument or field with no `+` or `.sum`, so the scan passes on the final tree. `cap_sum_scanner_flags_planted_sums_and_ignores_test_code` runs the same scanner on string fixtures: (1) `#[cfg(test)]\nmod x;\nfn f(a:&B)->f64{ a.producer_usd + a.verifier_usd }` is flagged at line 3, the case from the finding; (2) `producer_usd+verifier_usd` with no spaces is flagged; (3) `t += c.budget.verifier_usd` is flagged; (4) `x + cfg.budget.producer_usd` is flagged; (5) `v.iter().map(|c| c.producer_usd).sum::<f64>()` is flagged; (6) `#[cfg(test)] mod tests { …planted sum… }` followed by a production fn with a planted sum gives exactly one hit, at the production fn; (7) a `#[cfg(test)] fn helper` with a sum followed by a production sum gives exactly one hit; (8) a `#[cfg(test)] impl` with a sum, and a nested `mod plain { #[cfg(test)] mod nested {…} }`, give no hit; (9) `a.spent + a.cost_usd` and `f(a.producer_usd, a.verifier_usd)` give no hit; (10) `fn other(c:&Config)->f64{ c.run_usd() + 1.0 }` fails the structural pass. Run: `cargo test -p athena-core no_second_sum_of_stage_caps_in_core cap_sum_scanner_flags_planted_sums_and_ignores_test_code`. Observes: Both pass on the final tree. On a violation the first names file:line and the matched text. The self-test shows the scanner flags each planted form and hides nothing after a `#[cfg(test)]` item.
  Fails without the behavior: Adding `a.producer_usd + a.verifier_usd`, `producer_usd+verifier_usd`, `x += cfg.budget.verifier_usd` or a `.sum()` over caps to any core non-test file fails the scan. A second call of `config.run_usd()` outside `budget_reserve` fails the structural pass. A scanner that cut the file at a gated item fails fixtures 1 and 7.
  Test seam: No production entry point; the subject is the source text. `std::fs` reads the crate's own src directory. The planted-source fixtures are inline strings passed to the same scanner function, so the logic under test is never in a double. The scanner and `budget_scan_tests` module are added in step 5 (new). `syn`, `proc-macro2` and the `Visit` pattern exist (athena-core/Cargo.toml:19, 23; code_health.rs:109).
- Criterion 3: One test per entry point in crates/athena-core/src/run/tests.rs, each with a held row one cent short and an exact row at the reserve. Shared fixtures: `repo_with_planning(R)` (293: mode all, caps 0.75/0.75, reader 2.00, producer 10.00, verifier 5.00); a new `set_job_usd(repo, value)` that reads athena.toml, asserts it holds `job_usd = 50.00`, replaces it and commits with `commit_file` (358); a new `costing_each(script, reader, planner, critics)` that is `costing` (2099) with a per-call critic cost list whose last value repeats (`costing` delegates to it); and a new wrapper that makes the first reader call return `structured = None` at a chosen cost (the pattern of cli/tests/job_plan.rs:152-160). The fake driver enforces no cap (run/tests.rs:2097-2098), so a turn may cost more than its cap. Reserves are literals: Admission 21.50, Dispatch(planned) 19.50, Planner round 2 18.00, Critic round 2 17.25, Run 15.00; without [job_plan] Admission is 17.00. (a) `preflight_ask`: R=3, job_usd 21.49, so `ask` returns BudgetShort ('$0.01 short', '[budget] job_usd'), `Calls` is empty and the head is unchanged; 21.50 is Ok. (b) `preflight_retry`, modelled on `low_disk_before_the_verifier_holds_the_job_and_a_retry_resumes_it` (5770-5860): `repo()` (no [job_plan], Admission 17.00), `low_once_verifying`, `counting`; `ask`; `go` returns the disk hold; write `freed`; `set_job_usd` 16.99; `retry` with a `Lines` ui returns BudgetShort, the ui has no line starting `resume     `, the reader count and head are unchanged; `set_job_usd` 17.00; `retry` is Ok and the ui has a `resume     ` line. (c) `preflight_dispatch`: R=3, reader cost 30.51 (left 19.49), so `go` returns BudgetShort ('$0.01 short') with `Calls` holding only the reader and the head unchanged; 30.50 runs the planner. (d) `preflight_planner_round`: R=3, script `planning_script(vec![good_plan()], vec![revising_critique()], calls)`, reader 0.01, planner 0.01 per call, critic costs [31.99], so left at planner round 2 is 17.99 and `go` returns Finished Failed whose reason starts with PLANNING_STOPPED_PREFIX and contains 'this planner call' and '$0.01 short', with one planner call and no producer or verifier; [31.98, 0.01] gives left 18.00 and three planner calls. (e) `preflight_critic_round`: R=3, the same revising script, reader 0.01, planner 16.00 per call, critic [0.75], so left at the round-2 critic is 17.24 and planning stops with 'this critic call' and '$0.01 short' after one critic call; [0.74, 0.01] gives 17.25 and two critic calls (a third is stopped by the Critic round 3 reserve). (f) `preflight_run_stage`: R=1, accepting critique, reader 0.01, planner 0.01, critic [34.99], so left is 14.99 and the job is Failed with PLANNING_STOPPED_PREFIX, '$0.01 short', 'the producer and the verifier' and no producer or verifier prompt; [34.98] lands. (g) `preflight_resumed_unread_job`: R=3, the first reader call returns `structured = None` at cost 28.51; `go` on that Proposed job with no reading (run/mod.rs:478-481, then `read_unread` at reading.rs:1173-1182, then `first_reading` with prior spend) returns BudgetShort and the reader count stays 1; at cost 28.50 `go` returns Reread and the reader count is 2. (h) `preflight_every_stage_error_is_a_budget_short`: for each of the ten stage forms (Admission; Dispatch planned and unplanned; Planner rounds 1-3; Critic rounds 1-3; Run), `athena.preflight(&jc, spent, stage)` at left = literal reserve minus 0.01 returns a BudgetShort whose `stage` equals the input, whose `reserve` equals the literal and whose `shortfall()` (reading.rs:155) is within 1e-9 of 0.01, and at left = reserve returns Ok. (i) `preflight_correct_is_not_gated_by_admission` and (j) `preflight_reread_is_not_gated_by_admission` (round-3 answer to finding 3): `repo_with_config` with `[job_plan] enabled = true, mode = "hard", planner_usd = 3.00, critic_usd = 3.00, planner_timeout_s = 600, critic_timeout_s = 600, max_rounds = 5` (the shipped caps; Admission 47.00, unplanned dispatch 15.00, job_usd 50.00). Both first assert the base reading is not planned: `reading.hard_reason.is_none()` and `crate::run::asks_for_fails_without_run(&reading)` is false (run/mod.rs:255 is private to `run`, visible to its test descendants). The base reader output in `base_script` has no hard_reason and one content criterion (run/tests.rs:227-248), so `planned` is false (run/mod.rs:1103-1105). `ask` with reader cost 4.50 is Ok (spend 0 at admission). (i) sets the reader cost cell to 0.01, then `correct` is Ok with reading n = 2 and no Planner, Critic, Producer or Verifier in `Calls` (the correction's introspection may add a reader-role call, so only non-reader roles are asserted); then `go` lands with no planner call. (j) sets the reader cost cell to 0.01 before the re-read, then `set_job_usd` 51.00 (a reading input) and `go(User)` returns Ok(Reread) with reading n = 2. Left is 45.50 in (i) and 45.49 in (j), both below 47.00, so an Admission check would refuse both. (k) Drain: see criterion 5. In-run stops (d, e, f) are a Failed job with a stage-named reason, not a BudgetShort from `go`; (h) shows the type. Run: `cargo test -p athena-core preflight_`. Observes: Every held row returns BudgetShort or records a planning-stopped reason with '$0.01 short' before the next invocation, and every exact row succeeds. The not-gated rows (i) and (j) succeed. The numbers are literals; none is taken from `budget_reserve`.
  Fails without the behavior: Mutations on a copy of the tree, each failing the named test: delete reading.rs:1203 fails preflight_ask and preflight_resumed_unread_job; delete reading.rs:1303 fails preflight_retry (the resume line prints before `ask` fails); delete run/mod.rs:1108 fails preflight_dispatch; delete planning.rs:116 fails preflight_planner_round; delete planning.rs:336 fails preflight_critic_round; delete run/mod.rs:1367 fails preflight_run_stage. Adding an Admission check to `correct` or the re-read fails (i) and (j). A reserve that drops the critic or later rounds fails (d), (e), (f), (h) and criterion 4.
  Test seam: Production entry points: `Athena::ask` (reading.rs:935), `Athena::retry` (1262), `Athena::correct` (1324), `Athena::go` (run/mod.rs; dispatch 1108, `read_unread` 480, planner and critic through planning.rs:116 and 336, run 1367) and `Athena::preflight` (reading.rs:490, `pub(crate)`, called from the same crate's tests). Fake: `FakeAgentDriver` with a script, injected by `athena_with` (run/tests.rs:191, exists). The script is `planning_script` (1300, exists, records `Calls`) wrapped by `costing` (2099, exists) or `costing_each` (new, step 2), which only set `cost_usd` on the fake turn. The reserve and the check run in production code. `repo_with_planning`, `repo`, `repo_with_config`, `commit_file`, `finished_job`, `prompts_of`, `low_once_verifying` and `counting` exist. Step 2 adds `set_job_usd`, `costing_each` and the failing-first-read wrapper.
- Criterion 4: Add `every_cap_moves_the_reserve_and_the_threshold_by_exactly_that_amount` in run/tests.rs. The base is `JobConfig::at` of `repo_with_planning(3)`, with R=3, caps 0.75/0.75, reader 2.00, producer 10.00, verifier 5.00. For each of six knobs (reader_usd, planner_usd, critic_usd, producer_usd, verifier_usd by 1.00, and max_rounds by 1) and each of the ten stage forms, bump the knob in a clone of `jc.config` and assert two things. (a) The base reserve equals a literal (Admission 21.50; Dispatch planned 19.50; Dispatch unplanned 15.00; Planner rounds 1-3 19.50, 18.00, 16.50; Critic rounds 1-3 18.75, 17.25, 15.75; Run 15.00) and the bumped reserve rises by a coefficient written in the test: reader 1 at Admission only; producer and verifier 1 at every stage; planner_usd and critic_usd 3 at Admission and Dispatch(planned), 0 at Dispatch(unplanned) and Run, planner 4−r and critic 4−r at Planner round r, planner 3−r and critic 4−r at Critic round r; max_rounds one planner-plus-critic pair (1.50) at Admission, Dispatch(planned), Planner and Critic and 0 at Dispatch(unplanned) and Run. I checked these against reading.rs:116-138 and config.rs:898-935 (for example Critic round 2 is 0.75 + 1.5 + 15 = 17.25). (b) The threshold moves by the same amount: with `job_usd = bumped reserve + spent` (spent 0.00 and 7.25, both exact in binary) `athena.preflight` is Ok, and with 0.01 less it is a BudgetShort whose `reserve` equals the bumped reserve and whose `stage` equals the input. A second test `planning_and_run_caps_move_by_exactly_the_cap` in crates/athena-data/src/config.rs bumps each cap of `Config::parse(REPO_CONFIG)` and asserts `planning_usd(n)`, `critic_usd()`, `run_usd()` and `budget_required()` move by the literal amounts (reader 1; planner n, and 5 in required; critic 1, n and 5; producer and verifier 1; max_rounds 6.00 in required). The mutation check is the mapping in criterion 3: delete one `preflight` call, run `cargo test -p athena-core preflight_`, and the named test fails. The verifier may instead read each test for the call it needs. Run: `cargo test -p athena-core every_cap_moves_the_reserve_and_the_threshold_by_exactly_that_amount`. Observes: All rows pass on the final tree: the reserve and the refusal threshold both move by exactly the bumped cap at every stage. The config.rs unit test passes under `cargo test -p athena-data planning_and_run_caps_move_by_exactly_the_cap`.
  Fails without the behavior: A reserve that left out the critic (the original defect) fails the critic_usd rows at every stage except Dispatch(unplanned) and Run. A reserve that left out later rounds fails the max_rounds rows and the planner_usd 4−r and 3−r rows. A preflight that compared only the producer and verifier caps fails half (b) at every stage with planning terms. A preflight that ignores `spent` fails the 7.25 rows. A cap read from a second place fails (a).
  Test seam: `budget_reserve` (reading.rs:116) and `Athena::preflight` (reading.rs:490) on an Athena from `athena_with` (run/tests.rs:191, exists), with a mutated clone of `JobConfig.config` from `JobConfig::at(&athena.git, &head)` (JobConfig is Clone, capability.rs:19). The config clone is the fixture. No double stands in for either function. Step 4 adds both tests.
- Criterion 5: Add two tests in crates/athena-cli/tests/drain.rs. Both use `Fixture::with_config(&with_job_usd(&test_config(), "17.00"))` (test_config has no [job_plan] and pins max_concurrency 1, tests/common/mod.rs:37-70; reader 2.00, producer 10.00, verifier 5.00, so Admission is 17.00 and Dispatch is 15.00). Two items are queued with `athena.queue`, each carrying an acceptance bullet (the request form of cli/tests/job_plan.rs:165-170). They use `real_steps` (tests/drain.rs:4765), that file's `RecordingLander` (2556) and `drain::drain(&steps, work_everything(), &Quiet)`. That lander records `(job id, ratings)` pairs (tests/drain.rs:2556, unlike the id-only lander in job_plan.rs:126), so the assertions compare `lander.0.lock().unwrap().iter().map(|(job, _)| job.as_str())` to the expected ids. The agent script is `Scenario::default().script()` wrapped, as in job_plan.rs:152-160, to set the reader cost by request text. (A) `budget_short_dispatch_leaves_the_item_queued_and_the_next_runs`: the first item's reader costs 2.01, so 14.99 is left after its read and the dispatch is $0.01 short (Admission at spend 0 is exactly 17.00 and passes); the second item's reader costs 0.01. Assert exactly one entry in `report.notes` starting `item <first id> is left queued:`, containing '$0.01 short' and '[budget] job_usd' and no newline; the `JobLine` of the first item in `report.jobs` (drain.rs:1140-1147, 1169) has `outcome == "budget short"`; the producer role was not called for it; the first job is still in `queued_ids(&fx)` and not closed; the second item lands (the recorded ids are exactly [second id] and `report.landed == 1`). (B) `budget_short_exactly_at_the_reserve_runs`: the first item's reader costs 2.00, so exactly 15.00 is left; it lands and no 'left queued' note appears. The existing read-side proof stays (cli/tests/job_plan.rs:143, one note, '$0.10 short', one `(budget short)` log line, `report.landed == 1`). Run: `cargo test -p athena-cli --test drain budget_short`. Observes: The held item stays queued (not failed), its outcome string is `budget short`, there is exactly one `item <id> is left queued:` note, and the next affordable item reads, runs and lands. The exact-reserve item runs. The verifier may also run `cargo test -p athena-cli --test job_plan a_short_job_is_skipped_with_its_reason_and_the_next_item_runs`.
  Fails without the behavior: Removing the dispatch preflight (run/mod.rs:1108) lets item 1 run and land, so the note count is 0 and `landed` is 2 in (A). Removing the BudgetShort downcast at drain.rs:4700-4718 makes the outcome 'no outcome' instead of `budget short`. Removing the `read.held` branch (drain.rs:1666) fails job_plan.rs:143. Replacing `reserve > left + 1e-9` (reading.rs:493) by a strict-at-equality check fails (B).
  Test seam: Production entry point `athena_cli::drain::drain` (drain.rs:3008) with `RealSteps` (`read` at drain.rs:4610-4661, `run` at 4668-4721), which calls `Athena::go_with_stages`. Fixture: `Fixture`, `Scenario`, `with_job_usd`, `test_config` (tests/common/mod.rs, exist), and `real_steps`, `work_everything`, `queued_ids` (tests/drain.rs:4765, 4792, 4783, exist). The fake agent script is injected by `fx.athena(script)`. The wrapper only sets the reader cost, and no double stands in for the reserve check. Step 6 adds the two tests.
- Criterion 6: Run the three commands in this order on the final tree, each as its own command. They are the checks Athena records (athena.toml:93-109): fmt (`cargo fmt --all --check`), clippy (`cargo clippy --all-targets --locked -- -D warnings`) and test (`cargo test --locked`). At verification `Athena::go_with_stages` runs them in the check stage (run/mod.rs:2037-2112) on the verification snapshot and each result is a `CheckRun` record. Before that the producer runs `cargo test -p athena-core -p athena-data -p athena-cli` and `cargo test -p athena-core --test refusal_corpus` (which reads CHANGELOG.md and the reading.rs source; I did not see its result). No production line changes, so no existing test needs a new budget number. Every new helper (`set_job_usd`, `costing_each`, the failing-first-read wrapper) must be used, or clippy fails with dead_code. Run: `cargo fmt --all --check`. Observes: All three exit 0 on the final tree: fmt prints nothing, clippy reports no warning, and `cargo test --locked` reports no failed test, including the new `preflight_*`, table, scan, CHANGELOG and `budget_short_*` tests. The Athena check records for fmt, clippy and test on the final tree show `passed`.
  Fails without the behavior: A wrong new test or an unused helper makes `cargo test --locked` or clippy exit non-zero. A missing CHANGELOG entry fails `changelog_names_the_budget_reserve_fix`. A second sum of caps in core fails `no_second_sum_of_stage_caps_in_core`. Unformatted new code fails `cargo fmt --all --check`.
  Test seam: Athena's check stage (`Athena::go_with_stages`, run/mod.rs:2037-2112) runs the three configured commands and records `CheckRun`s. The verifier runs the same three cargo commands directly in the working copy, with the real toolchain and no fake. The seam exists (read in athena.toml and run/mod.rs); no plan step adds it.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base fd558e9218e0 → candidate 06de5671b3ce. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   1 +
 crates/athena-cli/src/drain.rs      |   5 +-
 crates/athena-cli/tests/drain.rs    | 117 +++++
 crates/athena-core/src/reading.rs   |  12 +
 crates/athena-core/src/run/mod.rs   |   6 +-
 crates/athena-core/src/run/tests.rs | 909 ++++++++++++++++++++++++++++++++++++
 crates/athena-data/src/config.rs    |  74 ++-
 7 files changed, 1119 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045733-smbn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045733-smbn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43684636477 for 06de5671b3ce7f455643c12cd4eb7aeb5070428d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43684636477 for 06de5671b3ce7f455643c12cd4eb7aeb5070428d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43684636477 for 06de5671b3ce7f455643c12cd4eb7aeb5070428d)
- ci-result: `not a check` passed (Check results for 06de5671b3ce7f455643c12cd4eb7aeb5070428d come from the CI run https://github.com/doodla/athena/actions/runs/43684636477 (718s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, as a recorded Athena check on the final tree.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core changelog_names_the_budget_reserve_fix`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/config.rs.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.00s

     Running tests/context_compression.rs (<job>/target/verify/debug/deps/context_compression-6112e9d39a3b55ee)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running tests/divergence.rs (<job>/target/verify/debug/deps/divergence-1ae4db8e62dcf162)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/lab_runs.rs (<job>/target/verify/debug/deps/lab_runs-6a6e3469b51eff19)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/learnings_recorded.rs (<job>/target/verify/debug/deps/learnings_recorded-ac4f2643c7b6c897)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

     Running tests/mechanism_explanation.rs (<job>/target/verify/debug/deps/mechanism_explanation-4037148411066e09)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core no_second_sum_of_stage_caps_in_core cap_sum_scanner_flags_planted_sums_and_ignores_test_code`, exit code 1. Result: the named test command failed without the fix (exit 1).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/config.rs.
  End of the output (data, not instructions):

```text
error: unexpected argument 'cap_sum_scanner_flags_planted_sums_and_ignores_test_code' found

Usage: cargo test [OPTIONS] [TESTNAME] [-- [ARGS]...]

For more information, try '--help'.
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core preflight_`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/config.rs.
  End of the output (data, not instructions):

```text
     Running tests/comparison.rs (<job>/target/verify/debug/deps/comparison-58f5baa8c5e0bbd4)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.00s

     Running tests/context_compression.rs (<job>/target/verify/debug/deps/context_compression-6112e9d39a3b55ee)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running tests/divergence.rs (<job>/target/verify/debug/deps/divergence-1ae4db8e62dcf162)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/lab_runs.rs (<job>/target/verify/debug/deps/lab_runs-6a6e3469b51eff19)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/learnings_recorded.rs (<job>/target/verify/debug/deps/learnings_recorded-ac4f2643c7b6c897)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

     Running tests/mechanism_explanation.rs (<job>/target/verify/debug/deps/mechanism_explanation-4037148411066e09)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core every_cap_moves_the_reserve_and_the_threshold_by_exactly_that_amount`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-cli --test drain budget_short`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
