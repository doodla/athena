# Job 20261116-004006-cvfn: verify "Give the planner a mechanical caller list and exit table for changed"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The planner designs from the reading and from the code paths it first finds. It does not enumerate e
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: plan-leaves-code-answerable-questions-open-and-skips-adjacent-existing-path-fails-planning-rounds (20261031-030110-hnnx)
> Cause: The planner designs from the reading and from the code paths it first finds. It does not enumerate every existing caller and mode that reaches the function it changes, such as the `[parallel] max_concurrency` path into landing. It also may ...
> Seen 9 time(s) in 9 jobs. The introspections that record it:
> Introspection 20261031-030054-6wkt (job 20261030-013555-f5qm); prevention it proposed: Add a plan-structure check that runs before each critic round. (a) A plan may not list an open question that is marked answerable by reading code. The planner must settle it by reading the code and cite the file and line. (b) Any plan that ...
> Introspection 20261031-031632-fdjf (job 20261031-030119-tw3v); prevention it proposed: Add a required step before the critic in planning for any change to a core flow such as landing. The planner must list every state the change touches, including records and job state read from the landing branch, and every existing caller a...
> Introspection 20261103-092900-v31e (job 20261103-075205-wbtm); prevention it proposed: Add a planning rule, enforced by code, that every open question and every 'rejected alternative' resting on a guess is checked against the snapshot before the critic's first round. Questions about a value or rule that existing code or recor...
> Introspection 20261103-135735-32k5 (job 20261103-135304-bwqp); prevention it proposed: Add a planning step that checks the reading's state claims before the plan is built. Every claim about what a job's state does next (stays queued, is retried, is picked up) must name the function that consumes that state, and the plan must ...
> Introspection 20261103-163901-xve1 (job 20261103-162416-m9th); prevention it proposed: Make the planning stage, for any plan that classifies errors or states into categories, enumerate the real producers of each class from the code (every bail!/Err site feeding the arm) and list them in the plan with their classification. Hav...
> Introspection 20261106-002309-v2xt (job 20261105-180330-tb41); prevention it proposed: When a plan adds a wrapper, rule or parser for commands that an existing rule already accepts in several forms, the planner should have to do four things in the plan. First, list the real accepted forms from the existing code: global flags ...
> Introspection 20261107-045737-r0sh (job 20261107-031917-r96w); prevention it proposed: Give planning a code-answerable check before the critic rounds. For a change to a core flow, the planner must list each claim about existing behavior (what counts as an outcome, which records the gate reconciles, which states the scope guar...
> Introspection 20261107-220018-3qj3 (job 20261107-212343-z26k); prevention it proposed: Add a repair step to planning for code-answerable findings. When the last round still has only findings that name a specific file and function, run one extra bounded round in which the planner must settle each finding by reading that code. ...
> Introspection 20261116-003943-6ch3 (job 20261115-202532-6mng); prevention it proposed: Add a planning rule, checked by the critic and by code where possible, that a plan changing a core flow must give a table of every exit of each new or changed stage (success, fail, error, hold, interrupt). For each exit it must state which ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the planner is told in prose to list every caller and mode (jobs 20261030-013555-f5qm, 20261031-030119-tw3v, 20261103-075205-wbtm), but its brief never shows it the callers of the functions in the deliverable files, and it has no rule to tabulate each exit of a changed stage. Fix in the planner brief inputs (information, not a refusal): Athena computes the callers of each existing Rust deliverable's functions from the base snapshot and puts them in the brief with a new exits rule.

Assumptions:
- Code survey: I read crates/athena-context/src/briefs/planner.rs (planner_brief, OPEN_QUESTION_RULE, CODE_CLAIMS_RULE, FIELD_MEANING_RULE, DeliverableFact), crates/athena-context/src/briefs.rs tests around lines 1800-1850 and 4169-4212, and crates/athena-core/src/run/planning.rs lines 95-155, where the planner brief is built and deliverable facts come from git.show_file. The entry point is planning_round, which calls planner_brief. No record or data file is added, so check_record_file and AnyRecord are not touched. The property is measured directly by asserting the rendered brief text.
- Prior jobs: 20261030-013555-f5qm, 20261031-030119-tw3v and 20261103-075205-wbtm each stopped at round 3 with code-answerable findings (the max_concurrency path, record loss on drop, role limits). Their candidates changed no files, so there is nothing to keep or drop. Each finding is addressed as a class: the brief now carries the callers and exits that the missing information concerned. Their own deliverables (drain.rs, landing.rs, xtask site.rs and similar) are out of scope here and need separate retries.
- The prose rules landed earlier (OPEN_QUESTION_RULE, CODE_CLAIMS_RULE) and the admission check on open questions already cover part of the class. The remaining gap is the computed facts and the exit table, which the latest introspection (job 20261115-202532-6mng) names.
- Callers are found by textual search of the base snapshot (fn name followed by an opening parenthesis), capped per function. This is approximate, and the brief labels it as a starting list the planner must extend, not a complete one.
- Goal and audience: the planner and critic roles are the audience; fewer plans stop at the round cap on findings that reading code would settle.
- Existing code: crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs.rs and crates/athena-core/src/run/planning.rs are extended, not replaced.
- Environment: no CI workflow, push or records read; the work is local to the snapshot and tests.

## Acceptance criteria

1. [content] The change description names the root cause: the planner brief gives no computed list of existing callers or exits for the functions in the deliverable files, only prose, so plans skipped adjacent paths such as `[parallel] max_concurrency` into landing. Checked in the diff of CHANGELOG.md and the doc comment on the new code in crates/athena-context/src/briefs/planner.rs.
2. [behavior] DeliverableFact in crates/athena-context/src/briefs/planner.rs gains a callers field. A unit test in crates/athena-context/src/briefs.rs builds a planner brief and asserts that each caller (file:line) appears in the deliverables section. Run: cargo test -p athena-context.
3. [behavior] A test in crates/athena-core/src/run/planning.rs or run/tests.rs computes callers from a fixture snapshot where a function in a deliverable file has callers in two other files. It asserts both are listed, and that a deliverable with no callers is shown as such, not omitted. It fails without the change. Run: cargo test -p athena-core.
4. [content] A new exits rule const in crates/athena-context/src/briefs/planner.rs is added to the planner Report. It requires a plan that changes a stage or core flow to tabulate every exit (success, fail, error, hold, interrupt) with the file:line that handles it. A test in briefs.rs asserts that wording appears exactly once, in a first-round brief and in a revision brief.
5. [content] No refusal rule is added. admission.rs and the reading checks are unchanged, so expected 0% of newly refused readings.
6. [content] CHANGELOG.md has an entry for the change.

## Expected deliverables

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Any new refusal rule or change to crates/athena-core/src/admission.rs
- Retrying the content of jobs 20261030-013555-f5qm, 20261031-030119-tw3v and 20261103-075205-wbtm (the landing queue, send_back, owner briefs)
- Changes to prompt .md files, which have almost no token headroom
- Changes to athena.toml, defaults.toml and .github/workflows
- Any file under records/

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the brief text a planner receives, not only that the code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests in athena-context and athena-core; the verifier runs cargo test for the two crates.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the planner's inputs and brief, not in a per-job workaround.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of the changelog entry and the doc comments on `CallerList`, `DeliverableFact` and `EXITS_RULE`. Run: `git diff -- CHANGELOG.md crates/athena-context/src/briefs/planner.rs`. Observes: Each says the root cause: the planner brief gave no computed list of existing callers or exits for the functions in the deliverable files, only prose, so plans skipped adjacent paths such as `[parallel] max_concurrency` into landing. The entry and the comments cite jobs 20261030-013555-f5qm, 20261031-030119-tw3v, 20261103-075205-wbtm and 20261115-202532-6mng, and say no refusal rule is added.
- Criterion 2: Run two new unit tests in briefs.rs (their names contain `callers`): `a_planner_brief_lists_the_callers_of_each_deliverable` and `the_callers_lines_are_capped_and_the_other_cases_are_labelled`. The first builds `planner_inputs(None)`, pushes a `DeliverableFact` for crates/a/src/x.rs with a `CallerList` (a function with 5 other-file sites of which 4 are shown, one a test site; a function called only in its own file; a function with none), calls `planner_brief`, and takes the text between the Deliverables heading and the next `\n## `. It asserts each shown `path:line`, '(test)' on the test site, '(+1 more)', the line 'Called only inside this file', the line 'No callers found', the label words ('textual search', 'starting list', 'Grep'), that none of it is after `## Report`, that the CHANGELOG.md deliverable (`callers: None`) has no callers line, and that `- athena.toml: exists, 1234 bytes` is unchanged. The second builds 35 functions in each of the three groups and asserts exactly `CALLERS_SHOWN` (30) names per line and '(+5 more)'; it also asserts the 'did not parse' line for `deliverable_parsed: false`, the unparsed file names (5 shown, then '+N more') and the 'declares no functions' line. Run: `cargo test -p athena-context callers`. Observes: All call-site strings are in the deliverables section and none in the Report; the none, same-file, unparsed and capped cases print as such.
  Fails without the behavior: Without the `callers` field and its rendering the tests do not compile. With the field but no rendering, the assertion that 'crates/b/src/y.rs:20' appears between the Deliverables heading and the next heading fails. Without the cap on the second and third lines, the count of 30 names per line fails.
  Test seam: Production entry point `planner_brief` (crates/athena-context/src/briefs/planner.rs:107, exists). Fixture: `PlannerInputs` from the test helper `planner_inputs` (briefs.rs:2756, exists), with `deliverable_facts` pushed in the test. The `CallerList` types, the `callers` field and the rendering are added by steps 2 and 3; the two existing literals at briefs.rs:2780-2790 get `callers: None`. No double: the test passes plain data to the real renderer.
- Criterion 3: Run new tests in athena-core (the names and module contain `caller`). (a) `caller_scan_tests` in planning.rs on in-memory source: `scan_source` lists live functions and skips a `#[test]` fn and a `#[cfg(test)]` mod; a production fn after a mid-file `#[cfg(test)] mod seam { .. }` is still listed and its calls are production calls (finding 1); `x.m(..)`, `Foo::m(..)`, `m(..)` and `assert_eq!(m(..), ..)` inside a macro are calls; `"m("` in a string, `// m(`, `fn m(` and a function passed by name are not; a call inside a `#[test]` fn or test module is flagged test; unparseable text gives None. (b) In run/tests.rs `deliverable_callers_lists_both_callers_and_shows_a_deliverable_with_none`: a real repo from `repo()` plus `commit_file` holds crates/x/src/core.rs (`land_job`, `unused_helper`, `used_here`, `run_all` calling `used_here`, a `#[cfg(test)] mod seam {}`, then `after_the_seam`, and a test module calling `super::land_job()`), a.rs (calls `crate::core::land_job()` on line 2), b.rs (calls `land_job` inside `assert_eq!` on line 3), crates/x/tests/it.rs (a test caller), text.rs (the name only in a string and a comment), other.rs (one function, no callers), broken.rs (does not parse) and README.md. It calls `athena.deliverable_facts("HEAD", paths)` for core.rs, other.rs, README.md and a missing path. It asserts `land_job` has elsewhere == 3, same_file == 1 and sites [a.rs:2, b.rs:3, tests/it.rs:1 (test)] in that order, that `unused_helper` and `lonely` are present with zero sites and `after_the_seam` is listed, that `seam_only` is not, that README.md and the missing path give `callers: None`, and that `unparsed` names broken.rs. (c) `deliverable_callers_reach_the_planner_prompt`: the same fixture under `repo_with_planning(1)`, a script wrapper over `planning_script` that sets the reader's deliverable path to crates/x/src/core.rs (the pattern at run/tests.rs:10876) and makes the planner answer TimedOut (as at run/tests.rs:1443-1454) so the job stops in planning; it asserts the first planner prompt contains `crates/x/src/a.rs:2`, `crates/x/src/b.rs:3` and 'No callers found'. Run: `cargo test -p athena-core caller`. Observes: Both caller files are listed by file:line in order, comments and strings are not counted, a deliverable with no callers reads 'No callers found', and the real planner prompt carries the lists.
  Fails without the behavior: Without `deliverable_facts` and the scan, every `callers` is None: the assertions on `land_job`, `lonely` and the prompt lines fail. A column-0 cut-off at `#[cfg(test)]` would fail the `after_the_seam` assertion; a text scan would fail the string and comment assertions.
  Test seam: Production entry point `Athena::deliverable_facts` (new `pub(super)` method in crates/athena-core/src/run/planning.rs, added by step 4), which calls `Git::ls_tree`, `Git::cat_blobs` and `Git::show_file` (crates/athena-git/src/lib.rs:516, :988, :539; exist). Fixture: a real temp git repository from `repo()` (run/tests.rs:289) plus `commit_file` (run/tests.rs:359), with `Athena` from `athena(&repo, &state)` (run/tests.rs:180, exists). The wiring test injects the planner and critic answers through `planning_script` (run/tests.rs:1316, exists) and reads `Calls`; the double returns canned answers and holds none of the scan logic. `scan_source` is a private fn in the same file (step 4), tested in the file's own test module.
- Criterion 4: Run the new test `the_planner_brief_requires_an_exit_table_once_in_both_rounds` in briefs.rs, modelled on `planner_brief_requires_code_claims_to_cite_the_code_in_both_rounds` (briefs.rs:1826). It loops over `planner_inputs(None)` and a `PlanRevision` input, takes the text after `## Report`, asserts `EXITS_RULE` occurs exactly once, has no newline and no `## Report`, and contains the words success, fail, error, hold, interrupt, file:line, stage and core flow. The Report size bound (briefs.rs:4164-4179) is updated to add `EXITS_RULE.len() + 2` and run in the same command filter through its own name. Run: `cargo test -p athena-context exit_table`. Observes: One occurrence of the rule in the first-round brief and in the revision brief; the size bound holds with the new rule counted.
  Fails without the behavior: Without `EXITS_RULE` the test does not compile. If the rule were not in the shared Report format string or were written twice, the count of 1 fails for one of the two briefs.
  Test seam: Production entry point `planner_brief` (planner.rs:107, exists), fixtures `planner_inputs`, `plan(1)` and `finding` (briefs.rs:2756 and nearby, exist). `EXITS_RULE` is a `pub(super)` const added by step 2, reached as `super::planner::EXITS_RULE` as the CODE_CLAIMS_RULE test does at briefs.rs:1827.
- Criterion 5: Show that the diff touches none of the admission file, the reading checks or the refusal corpus. Run: `git diff --stat -- crates/athena-core/src/admission.rs crates/athena-core/src/reading.rs crates/athena-core/tests/refusal_corpus.rs`. Observes: An empty diff stat. No check reads `DeliverableFact` or the Report text (searched `DeliverableFact` in crates/: only briefs.rs, planner.rs, run/mod.rs and planning.rs), so the expected share of past accepted readings newly refused is 0%. The plan adds no refusal or matching rule, so the three-part corpus mapping does not apply and no corpus case is added.
- Criterion 6: Read the diff of CHANGELOG.md. Run: `git diff -- CHANGELOG.md`. Observes: One new bullet at the head of `### Changed` under `## [Unreleased]` (CHANGELOG.md:8-12), in the style of the entries below it. It names the root cause, the three code changes (the computed callers, the `callers` field and rendering, the exits rule), the limits (approximate name search, caps, parse failures listed, the model may ignore the table, the effect on round-cap stops is not measured), that no refusal rule is added (0% newly refused), the share of code_health helpers made `pub(crate)`, and the test filters.

Planned files:

- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 48265ce35cbe → candidate 869e10e2d183. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   1 +
 crates/athena-context/src/briefs.rs         | 180 ++++++++++++-
 crates/athena-context/src/briefs/planner.rs | 156 ++++++++++-
 crates/athena-core/src/code_health.rs       |   8 +-
 crates/athena-core/src/run/planning.rs      | 390 +++++++++++++++++++++++++++-
 crates/athena-core/src/run/tests.rs         | 171 ++++++++++++
 6 files changed, 890 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261116-004006-cvfn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261116-004006-cvfn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43797169766 for 869e10e2d183c964c8dc6c0f123243c2ed9eb509)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43797169766 for 869e10e2d183c964c8dc6c0f123243c2ed9eb509)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43797169766 for 869e10e2d183c964c8dc6c0f123243c2ed9eb509)
- ci-result: `not a check` passed (Check results for 869e10e2d183c964c8dc6c0f123243c2ed9eb509 come from the CI run https://github.com/doodla/athena/actions/runs/43797169766 (781s): fmt success, clippy success, test success.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context callers`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/planner.rs, crates/athena-core/src/code_health.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
1971 |     fn the_callers_lines_are_capped_and_the_other_cases_are_labelled<FunctionCallers>() {
     |                                                                     +++++++++++++++++

error[E0422]: cannot find struct, variant or union type `CallerList` in this scope
    --> crates/athena-context/src/briefs.rs:1977:20
     |
1977 |         let list = CallerList {
     |                    ^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `CallerList` in this scope
    --> crates/athena-context/src/briefs.rs:2003:36
     |
2003 |         let unparsed_deliverable = CallerList {
     |                                    ^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `CallerList` in this scope
    --> crates/athena-context/src/briefs.rs:2013:21
     |
2013 |         let empty = CallerList {
     |                     ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `EXITS_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:4343:39
     |
4343 |                     + super::planner::EXITS_RULE.len()
     |                                       ^^^^^^^^^^ not found in `super::planner`

error[E0425]: cannot find value `EXITS_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:4351:29
     |
4351 |             super::planner::EXITS_RULE.len()
     |                             ^^^^^^^^^^ not found in `super::planner`

error[E0560]: struct `planner::DeliverableFact` has no field named `callers`
    --> crates/athena-context/src/briefs.rs:1891:13
     |
1891 |             callers: Some(list),
     |             ^^^^^^^ `planner::DeliverableFact` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `planner::DeliverableFact` has no field named `callers`
    --> crates/athena-context/src/briefs.rs:2952:21
     |
2952 |                     callers: None,
     |                     ^^^^^^^ `planner::DeliverableFact` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `planner::DeliverableFact` has no field named `callers`
    --> crates/athena-context/src/briefs.rs:2958:21
     |
2958 |                     callers: None,
     |                     ^^^^^^^ `planner::DeliverableFact` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0422, E0425, E0432, E0560.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-context` (lib test) due to 16 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core caller`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/planner.rs, crates/athena-core/src/code_health.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
814 |         let mut index: BTreeMap<String, Vec<(String, u32, bool)>> = BTreeMap::new();
    |                        ^^^^^^^^ not found in this scope
    |
    = note: struct `crate::store::tests::BTreeMap` exists but is inaccessible
help: consider importing this struct
    |
740 +     use std::collections::BTreeMap;
    |

error[E0433]: cannot find type `BTreeMap` in this scope
   --> crates/athena-core/src/run/planning.rs:814:69
    |
814 |         let mut index: BTreeMap<String, Vec<(String, u32, bool)>> = BTreeMap::new();
    |                                                                     ^^^^^^^^ use of undeclared type `BTreeMap`
    |
    = note: struct `crate::store::tests::BTreeMap` exists but is inaccessible
help: consider importing this struct
    |
740 +     use std::collections::BTreeMap;
    |

error[E0425]: cannot find function `callers_of` in this scope
   --> crates/athena-core/src/run/planning.rs:825:20
    |
746 |     fn calls_of(scan: &SourceScan, name: &str) -> Vec<(u32, bool)> {
    |     -------------------------------------------------------------- similarly named function `calls_of` defined here
...
825 |         let list = callers_of("src/self.rs", &scan, &index);
    |                    ^^^^^^^^^^
    |
help: a function with a similar name exists
    |
825 -         let list = callers_of("src/self.rs", &scan, &index);
825 +         let list = calls_of("src/self.rs", &scan, &index);
    |

error[E0599]: no method named `deliverable_facts` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:1544:28
     |
1544 |         let facts = athena.deliverable_facts("HEAD", &paths).unwrap();
     |                            ^^^^^^^^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 | pub struct Athena {
     | ----------------- method `deliverable_facts` not found for this struct

warning: unused import: `super`
   --> crates/athena-core/src/run/planning.rs:740:9
    |
740 |     use super::*;
    |         ^^^^^
    |
    = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

Some errors have detailed explanations: E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0425`.
warning: `athena-core` (lib test) generated 1 warning
error: could not compile `athena-core` (lib test) due to 13 previous errors; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context exit_table`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/planner.rs, crates/athena-core/src/code_health.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
1971 |     fn the_callers_lines_are_capped_and_the_other_cases_are_labelled<FunctionCallers>() {
     |                                                                     +++++++++++++++++

error[E0422]: cannot find struct, variant or union type `CallerList` in this scope
    --> crates/athena-context/src/briefs.rs:1977:20
     |
1977 |         let list = CallerList {
     |                    ^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `CallerList` in this scope
    --> crates/athena-context/src/briefs.rs:2003:36
     |
2003 |         let unparsed_deliverable = CallerList {
     |                                    ^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `CallerList` in this scope
    --> crates/athena-context/src/briefs.rs:2013:21
     |
2013 |         let empty = CallerList {
     |                     ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `EXITS_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:4343:39
     |
4343 |                     + super::planner::EXITS_RULE.len()
     |                                       ^^^^^^^^^^ not found in `super::planner`

error[E0425]: cannot find value `EXITS_RULE` in module `super::planner`
    --> crates/athena-context/src/briefs.rs:4351:29
     |
4351 |             super::planner::EXITS_RULE.len()
     |                             ^^^^^^^^^^ not found in `super::planner`

error[E0560]: struct `planner::DeliverableFact` has no field named `callers`
    --> crates/athena-context/src/briefs.rs:1891:13
     |
1891 |             callers: Some(list),
     |             ^^^^^^^ `planner::DeliverableFact` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `planner::DeliverableFact` has no field named `callers`
    --> crates/athena-context/src/briefs.rs:2952:21
     |
2952 |                     callers: None,
     |                     ^^^^^^^ `planner::DeliverableFact` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `planner::DeliverableFact` has no field named `callers`
    --> crates/athena-context/src/briefs.rs:2958:21
     |
2958 |                     callers: None,
     |                     ^^^^^^^ `planner::DeliverableFact` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0422, E0425, E0432, E0560.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-context` (lib test) due to 16 previous errors
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
