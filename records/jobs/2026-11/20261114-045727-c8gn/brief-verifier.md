# Job 20261114-045727-c8gn: verify "Show per-prompt-file token room in the reader brief"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Readings choose where a prompt edit goes without seeing how much token budget each prompt set has le
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: reading-edits-prompt-without-checking-token-budget (20261024-002642-bm4c)
> Cause: Readings choose where a prompt edit goes without seeing how much token budget each prompt set has left. The shared verifier prompt is composed into several sets, so an addition there costs every set that includes it, and one set had 4 token...
> Seen 7 time(s) in 7 jobs. The introspections that record it:
> Introspection 20261024-002725-eqs1 (job 20261024-002644-1as8); prevention it proposed: Add a pre-dispatch check for any job whose reading touches prompt files. It computes the token headroom of every prompt set that composes each target file, the same figure the budgets test prints, and adds it to the reading. If the planned ...
> Introspection 20261024-104248-8p9c (job 20261024-093833-vava); prevention it proposed: Before a reading is accepted, run a check that computes the remaining token room for every composed prompt (producer per capability, verifier, and so on) and puts the table into the reading stage's context. Add a check that any reading or p...
> Introspection 20261025-090447-2zm3 (job 20261025-034452-wqp8); prevention it proposed: Add a check at reading admission. It takes each prompt file the reading or plan says it will grow, and compares the planned growth with the remaining budget the budgets test computes. It rejects the reading, naming the file and the room lef...
> Introspection 20261027-170109-ycpz (job 20261027-054203-wkp5); prevention it proposed: Before dispatch, a code check computes the remaining token room for each prompt set a reading or plan will edit. It compares that room with the estimated addition and puts the result in the reading. Where room is short, the reading must eit...
> Introspection 20261108-004800-fka0 (job 20261024-232057-r4pj); prevention it proposed: Add a check that runs before the verifier on any job whose diff touches prompt files. It runs the prompt, brief and budget tests (the athena-context lib tests and the budgets tests) and reports the failing test name and assertion. It also f...
> Introspection 20261108-013638-a1y0 (job 20261025-154650-h9ea); prevention it proposed: Before dispatch, compute the remaining token room for any prompt a request edits and put the figure in the reading. After the producer edits, run a code check on the prompt diff. It must confirm that every old bullet is still present as its...
> Introspection 20261109-052112-m8x3 (job 20261109-035221-jj8g); prevention it proposed: Add a code check that runs before verification on any job editing files under crates/athena-context/prompts/. It extracts the rule statements (bullets and must/never lines) from the old and new version of each prompt. It then requires every...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader's "Prompt token headroom" lists sets by component id (e.g. frame.verifier), so a reading must map a file path to every set composing it and take the tightest room itself; 7 jobs (e.g. 20261024-002644-1as8) planned prompt growth past a set's room. Fix it at the source by adding to the reader brief a per-file table: each prompt file path, the sets that compose it, and the smallest room left. No refusal rule is added.

Assumptions:
- Code survey: read crates/athena-context/src/lib.rs (HeadroomRow, headroom_rows, component constants with include_str! of prompts/*.md), crates/athena-context/src/briefs/reader.rs (headroom_section, ReaderInputs.headroom), crates/athena-core/src/reading.rs (reader_headroom feeds the brief at reading.rs:629), crates/athena-cli/tests/budgets.rs (headroom rows equal budget figures), crates/athena-core/src/run_pin_tests.rs:1582. No record or data file is added, so no check_record_file/AnyRecord work.
- The brief already gives per-set headroom, but by component id (frame.verifier, capability.change.verifier), not by file path; the reader must infer which edited file feeds which sets and take the minimum. That missing mapping is the cause; the table is derived from the same HeadroomRow figures as the budgets test.
- Goal: the reader stage (audience: the reader agent and the verifier judging readings) sees, per prompt file path, the tightest room, so it picks a target file with room. Past jobs 20261024-002644-1as8 and the other six introspected jobs are the failure instances; the earlier retried jobs (vava, wqp8) are context, not retried here.
- Capability sections map by id: capability.<id>.producer/verifier to crates/athena-cap-<id>/prompts/producer.md / verifier.md; defaults and catalog are generated, not files, and are left out of the per-file table.
- The reader prompt (reader.md, 2 tokens left) is not edited; the new text lives in the brief, which has no token budget.

## Acceptance criteria

1. [content] crates/athena-context/src/lib.rs has a function that maps each prompt component (contract, frame.*, capability.<id>.producer/verifier) to its repository path (e.g. crates/athena-context/prompts/verifier.md, crates/athena-cap-change/prompts/verifier.md) and, from the HeadroomRow list, returns for each file the sets that compose it and the smallest tokens-left among them.
2. [content] headroom_section in crates/athena-context/src/briefs/reader.rs renders that per-file table under '# Prompt token headroom', as lines naming the file path, its sets and the tightest room, in addition to the existing per-set rows.
3. [behavior] A unit test in crates/athena-context builds rows where shared frame.verifier is composed by verifier (change) with 59 left and verifier (study) with 36 left, and asserts the per-file entry for crates/athena-context/prompts/verifier.md lists both sets and says 36 left; it fails without the change. Run: cargo test -p athena-context headroom.
4. [behavior] A reader-brief test asserts the rendered brief contains the line for crates/athena-context/prompts/contract.md with every set composing it and the minimum room, and that no per-file section appears when headroom is empty. Run: cargo test -p athena-context reader_brief.
5. [behavior] A test in athena-core (reading brief path, next to the_reading_brief_carries_the_token_headroom_of_the_reader_and_verifier_sets in run_pin_tests.rs) asserts the real reading brief includes a per-file line for crates/athena-cap-change/prompts/verifier.md. Run: cargo test -p athena-core token_headroom.
6. [content] CHANGELOG.md has an [Unreleased] entry naming the root cause (the brief gave per-set rows by component id, not per-file room); a new appended docs/decisions.md decision with a Provenance bullet may record it. Existing entries are not edited.
7. [behavior] The recorded fmt, clippy and test checks pass on the final tree (cargo fmt --check, cargo clippy, cargo test).
8. [content] No refusal rule is added; expected 0% of newly refused readings.

## Expected deliverables

- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Adding any admission or refusal check on readings or plans
- Editing any prompt file under crates/*/prompts/
- Post-producer diff checks of prompt rules (other introspection proposals)
- Changing token budgets or athena.toml, defaults.toml
- Editing records/ or existing docs/decisions.md entries

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix at the brief that lacks per-file room
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: tests fail without the change
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: criteria check the brief text rendered
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check of the diff in crates/athena-context/src/lib.rs, then run the unit tests that exercise the function. Run: `cargo test -p athena-context headroom`. Observes: `prompt_file_path` and `file_headroom` exist in lib.rs. `file_headroom` walks the rows in order and each row's components, adds a row's set once per path, and keeps the minimum `row.left()`. The tests `headroom_file_paths_map_every_prompt_component` and `headroom_file_paths_name_the_file_each_component_is_built_from` pass. The first asserts contract, frame.reader/producer/verifier/planner/critic, capability.change.verifier and capability.study.producer map to the right paths and defaults, catalog and unknown ids map to None. The second reads each mapped file from disk and compares it with the compiled-in text.
- Criterion 2: Content check of the diff in crates/athena-context/src/briefs/reader.rs, then render it through the tests of criterion 4. Run: `cargo test -p athena-context reader_brief`. Observes: headroom_section keeps the heading, intro and per-set loop unchanged. After the loop it calls `crate::file_headroom(&inputs.headroom)` and, when the result is non-empty, writes a lead line and one line per file in the form `- <path>: composed by <set>, <set>; tightest room <n> tokens left`. The existing test `the_reader_brief_shows_the_headroom_of_each_set` still passes, which proves the per-set rows are kept.
- Criterion 3: New unit test `headroom_per_file_takes_the_tightest_of_the_sets_that_compose_it` in the lib.rs tests module. It builds HeadroomRow literals (all fields are pub, lib.rs:240-249) and calls `file_headroom`. The rows are `verifier (change)` with components [contract, frame.verifier, capability.change.verifier], used 1241 and budget 1300 (59 left), and `verifier (study)` with components [contract, frame.verifier, capability.study.verifier], used 1264 and budget 1300 (36 left). It finds the entry for crates/athena-context/prompts/verifier.md and asserts `sets == ["verifier (change)", "verifier (study)"]` and `left == 36`. It also asserts crates/athena-cap-change/prompts/verifier.md lists only `verifier (change)` with 59 left, and that a row that lists frame.verifier twice names its set once. Run: `cargo test -p athena-context headroom`. Observes: The test passes. The shared frame file lists both sets with 36 left; the capability file lists only its own set with 59 left.
  Fails without the behavior: `file_headroom` does not exist, so the test does not compile. An implementation that took the first row's room (59), summed the rooms (95) or listed one set fails the `36` and both-sets assertions, so the test checks the minimum across composing sets and not one instance.
  Test seam: Production entry point: `athena_context::file_headroom` (added by plan step 1). The fixture is plain HeadroomRow literals, since every field is pub (lib.rs:240-249). Nothing is faked or injected, and the logic under test is not in the fixture. The seam is added by step 1; the HeadroomRow type exists.
- Criterion 4: Two new tests in crates/athena-context/src/briefs.rs, beside the_reader_brief_shows_the_headroom_of_each_set (briefs.rs:4263). `the_reader_brief_lists_the_tightest_room_per_prompt_file` starts from the existing fixture `reader_inputs(&reading())` (briefs.rs:3913, `headroom: vec![]` at :3926) and sets `inputs.headroom` to three HeadroomRow literals that all list `contract` and differ in room: `reader` (80 left), `planner` (20 left) and `verifier (change)` (41 left). It renders `reader_brief(&inputs)` and asserts the brief contains the exact line `- crates/athena-context/prompts/contract.md: composed by reader, planner, verifier (change); tightest room 20 tokens left`. `the_reader_brief_has_no_per_file_headroom_without_rows` leaves `inputs.headroom` empty and asserts the brief contains neither the lead line nor `prompts/contract.md: composed by`. No existing text in the brief contains `prompts/` or the lead line (grep of crates/athena-context/src/briefs found none), so the negative assertion is meaningful. Run: `cargo test -p athena-context reader_brief`. Observes: The rendered brief holds the contract.md line naming every composing set and the minimum room (20). With empty headroom no per-file text appears. The existing tests at briefs.rs:4263 and 4588 pass unchanged.
  Fails without the behavior: headroom_section prints only per-set rows, so the contract.md path line is absent and the `contains` assertion fails. A version that printed the first set's room (80) or omitted a set fails the exact-line assertion.
  Test seam: Production entry point: `reader_brief(&ReaderInputs)` in briefs/reader.rs, which calls headroom_section at reader.rs:387. The fixture `reader_inputs(&reading())` exists (briefs.rs:3913); the test sets the pub field `inputs.headroom` (reader.rs:266) directly, so nothing is faked. The seam exists today; no plan step adds it.
- Criterion 5: New test `the_reading_brief_names_the_token_headroom_per_prompt_file` in crates/athena-core/src/run_pin_tests.rs, modelled on the test at :1582-1646. athena-core cannot depend on athena-cap-change (cap-change depends on core, per the crate-reach facts), so the test defines `StandInChange`, a capability with id "change" that copies PinningStudy's card, grants, checks, env and admit (run_pin_tests.rs:31-77). Its producer and verifier sections are `PromptAsset { id: "capability.change.producer"/"capability.change.verifier", version: 1, text: "stand-in ..." }`. The test builds `let mut athena = athena_with(...)`, pushes `Box::new(StandInChange)` onto `athena.capabilities` (the field is built at :250), captures the reader brief with the existing `Tweak` hook (as at :1587-1591), and calls `athena.ask("study the demo", &Quiet)`. It asserts the brief contains `- crates/athena-cap-change/prompts/verifier.md: composed by verifier (change); tightest room {left} tokens left`, with `left` computed in the test from `athena_context::compose` of CONTRACT, VERIFIER_FRAME and the stand-in text and `BudgetSet::Verifier.budget()`. It also asserts the line for crates/athena-context/prompts/verifier.md names both `verifier (study)` and `verifier (change)`. The study set composes the frame twice (:1608-1619), so its room is 0 and that file's tightest room is 0. Run: `cargo test -p athena-core token_headroom`. Observes: The real reading brief built by read_once (reading.rs:607, headroom from reader_headroom at :629) carries the per-file lines. The existing `the_reading_brief_carries_the_token_headroom_of_the_reader_and_verifier_sets` also matches the filter and still passes, because its per-set lines are unchanged. `reader_headroom` includes the stand-in because `[capabilities.change]` is in athena.toml (line 81), which config(true) embeds (:80-107), and the filter at reading.rs:259-263 passes it. `Athena::capability(id)` looks up by id (lib.rs:264-270), so adding `change` does not disturb the scripted `capability: study` reply.
  Fails without the behavior: The brief has no per-file lines, so the contains assertion for crates/athena-cap-change/prompts/verifier.md fails. If the mapping ignored capability ids, the cap-change path would be missing.
  Test seam: Production entry point: `Athena::ask` (reading.rs:935) -> read_for -> read_once -> reader_brief. Fixtures: the fake agent driver, the `Tweak` hook and `athena_with` (run_pin_tests.rs:142-261), all of which exist. The stand-in capability and its push onto `athena.capabilities` are added by plan step 5; they double only the capability's prompt ids and text, not the headroom logic, and the expected room is computed independently with compose and BudgetSet.
- Criterion 6: Content check of the diff of CHANGELOG.md and docs/decisions.md. Run: `git diff CHANGELOG.md docs/decisions.md`. Observes: The diff holds only added lines. CHANGELOG.md gains one bullet under `## [Unreleased]` / `### Changed` (line 10) naming the root cause (the brief gave per-set rows by component id, not per-file room), the failure class reading-edits-prompt-without-checking-token-budget (20261024-002642-bm4c), the introspected jobs, the fix, 'no refusal rule (0% of past accepted readings refused)' and the tests. docs/decisions.md gets D185 appended after D184 (decisions.md:5387-5408) in the same format (Case, Decision with root cause, Alternatives, Tradeoff, Reversal cost, Provenance). Existing entries are untouched, so tests that read old entries (refusal_corpus.rs:1138-1250 include_str the changelog and decisions) are unaffected.
- Criterion 7: Run the three recorded checks on the final tree, as listed under 'Checks Athena will run on the result', after step 7 has formatted and linted the new code. This is a regression gate over the new code and the tests of criteria 3-5. Run: `cargo fmt --all --check`. Observes: fmt exits 0. Run the same way, `cargo clippy --all-targets --locked -- -D warnings` exits 0 with no warning in the new code, and `cargo test --locked` passes in full: the new tests of criteria 3, 4 and 5, the existing headroom and reader-brief tests, `the_reader_headroom_rows_equal_the_budget_figures_and_name_shared_files` (budgets.rs:396) and `system_facts_budgets_match_computed` (the budget lines in docs/system-facts.md are unchanged because no prompt file changes).
  Fails without the behavior: Without the change, the new tests of criteria 3, 4 and 5 are absent. With only part of it, `cargo test --locked` fails: no `file_headroom` means athena-context's test target does not compile, and a headroom_section that prints no per-file line fails the contract.md assertion of criterion 4 and the athena-core assertion of criterion 5. Unformatted new code fails `cargo fmt --all --check`. An unused or unlinted item fails clippy under `-D warnings`. A prompt-file edit or a budget change would fail `system_facts_budgets_match_computed` and the budgets test.
  Test seam: Production entry points: the three recorded commands, run at the repository root: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. Fixture: the final working tree and the committed Cargo.lock. `--locked` refuses a lockfile change, and the plan adds no dependency or crate. Nothing is faked or injected. The seam exists today (all three are in the verifier's allowed commands); plan step 7 runs them.
- Criterion 8: Content check that the diff adds no refusal, admission or check function. Run: `git diff --stat`. Observes: Changed files are only the six deliverables. crates/athena-core/src/reading.rs, run/, gate and every refusal_corpus input are untouched. The new code is `prompt_file_path`, `file_headroom` and formatting lines in headroom_section, none of which returns a refusal or error. The expected share of newly refused readings is 0%, because the change only adds brief text. No input form of a matching or refusal rule changes, so no refusal_corpus case is mapped.

Planned files:

- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 69498dbcf3ec → candidate 379b39b17dd2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   2 +
 crates/athena-context/src/briefs.rs        |  27 +++++
 crates/athena-context/src/briefs/reader.rs |  16 +++
 crates/athena-context/src/lib.rs           | 160 +++++++++++++++++++++++++++++
 crates/athena-core/src/run_pin_tests.rs    |  92 +++++++++++++++++
 docs/decisions.md                          |  24 +++++
 6 files changed, 321 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045727-c8gn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045727-c8gn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43675357705 for 379b39b17dd24443bca37496e0f206b49b6da7e8)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43675357705 for 379b39b17dd24443bca37496e0f206b49b6da7e8)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43675357705 for 379b39b17dd24443bca37496e0f206b49b6da7e8)
- ci-result: `not a check` passed (Check results for 379b39b17dd24443bca37496e0f206b49b6da7e8 come from the CI run https://github.com/doodla/athena/actions/runs/43675357705 (947s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: The recorded fmt, clippy and test checks pass on the final tree (cargo fmt --check, cargo clippy, cargo test).)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context headroom`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/lib.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
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
   Compiling thiserror v2.0.21
   Compiling memchr v2.8.3
   Compiling getrandom v0.3.4
   Compiling cfg-if v1.0.5
   Compiling winnow v1.0.4
   Compiling ref-cast v1.0.27
   Compiling itoa v1.0.18
   Compiling serde_derive_internals v0.30.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling thiserror-impl v2.0.21
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling winnow v0.7.15
   Compiling jiff-core v0.1.1
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling jiff v0.2.38
   Compiling toml v0.9.12+spec-1.1.0
   Compiling schemars_derive v1.2.2
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0425]: cannot find value `prompt_file_path` in this scope
   --> crates/athena-context/src/lib.rs:359:17
    |
359 |         let p = prompt_file_path;
    |                 ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `file_headroom` in this scope
   --> crates/athena-context/src/lib.rs:347:21
    |
347 |         let files = file_headroom(&rows);
    |                     ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `prompt_file_path` in this scope
   --> crates/athena-context/src/lib.rs:417:24
    |
417 |             let path = prompt_file_path(id).expect(id);
    |                        ^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context reader_brief`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/lib.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0425]: cannot find value `prompt_file_path` in this scope
   --> crates/athena-context/src/lib.rs:359:17
    |
359 |         let p = prompt_file_path;
    |                 ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `file_headroom` in this scope
   --> crates/athena-context/src/lib.rs:347:21
    |
347 |         let files = file_headroom(&rows);
    |                     ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `prompt_file_path` in this scope
   --> crates/athena-context/src/lib.rs:417:24
    |
417 |             let path = prompt_file_path(id).expect(id);
    |                        ^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 3 previous errors
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core token_headroom`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/lib.rs, docs/decisions.md.
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
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
