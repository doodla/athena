# Job 20261030-013710-8gwp: verify "Add capability preflight with blocker and workaround for OLY-B-RUN-01"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-RUN-01 a mechanism
> 
> Requirement OLY-B-RUN-01 in docs/spec.toml is active and has no mechanism.
> Requirement: A preflight checks and records each needed capability first, and anything missing becomes a blocker with a workaround plan.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-RUN-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-RUN-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a preflight in athena-core that checks each tool a job needs (git, cargo, and per-capability needs) before work starts, records the result, and turns anything missing into a blocker with a workaround plan. Record it as the mechanism, with owner and a measure ending `status: not shown` plus the test, for OLY-B-RUN-01 in docs/spec.toml. Audience: the Athena maintainer and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/run/pin.rs (pin_repo fails when a pinning capability lacks repo_url/repo_ref), run/planning.rs (an existing budget `preflight` method before planner/critic), run/mod.rs (job lifecycle entry, Athena::go), pin.rs, capability.rs (Capability trait, pins_repo), slots.rs and checks.rs (spawn cargo/git). No capability preflight exists today; the existing `preflight` is a budget reserve check and is not reused. The new module probes tools on PATH before the producer runs, then records and blocks.
- Records: the preflight result is written as a finding or note on the job through existing record types (athena_data::records Severity::Blocker, FindingSource::Athena), so no new record or data file is added under crates/; if one were added, check_record_file, AnyRecord and the round-trip test would need to cover it.
- Existing code: extends crates/athena-core/src/run/pin.rs and run/mod.rs flow and reuses Severity::Blocker findings from crates/athena-data; docs/spec.toml entries such as the role-split mechanism show the mechanism/owner/measure format.
- Measure: no `athena status` line shows the preflight, so the measure ends with `status: not shown` and names the test that shows it. Adding a status line is out of scope.
- The preflight probes tools by running `<tool> --version` through an injectable probe trait, so the test can simulate a missing tool without changing the machine.
- Because a missing capability now blocks the job instead of failing later, this refuses more jobs when a tool is absent; for a normal environment with git and cargo present, expected 0% of newly refused readings.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-RUN-01 has `mechanism` naming the new preflight source path under crates/athena-core/src (a file that exists in the diff), `owner = "Athena"`, and a `measure` ending in `status: not shown` plus the test name.
2. [behavior] A new unit test in crates/athena-core (run via `cargo test -p athena-core preflight`) uses a fake probe reporting a needed tool missing and asserts the preflight returns a blocker naming that tool with a non-empty workaround plan; the same test with all tools present asserts no blocker. The verifier checks the test fails when the preflight function body is stubbed out (reviewed in diff) and passes as written.
3. [behavior] A test run via `cargo test -p athena-core` shows that the job's run path calls the preflight before the producer starts and that a missing capability stops the job with the blocker recorded, with each probed capability recorded even when all are present.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root.
5. [content] The change refuses more jobs only when a needed tool is missing; expected 0% of newly refused readings, and the stricter behavior is intended.
6. [content] No governance files (athena.toml, defaults.toml, workflows) and no records/ change; CHANGELOG.md may gain one line.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/preflight.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/pin.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding an `athena status` line for the preflight
- Changing the existing budget preflight in run/planning.rs
- Changes to athena.toml, defaults.toml, workflows or other requirements in docs/spec.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require a new failing-without/passing-with test and cargo test, clippy and fmt passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check a missing tool producing a blocker with workaround, not mere existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module plus a hook in the run path.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test `capability_preflight_spec_entry_names_this_file_owner_and_test` in preflight.rs, shaped like `iris_spec_entry_names_files_that_exist` (iris.rs:227-243). It parses the real spec with `crate::hestia::parse_spec(include_str!("../../../docs/spec.toml"))`, finds OLY-B-RUN-01 and asserts: active, owner `Athena`, mechanism contains `crates/athena-core/src/preflight.rs`, measure contains `capability_preflight_missing_tool_becomes_a_blocker_with_a_workaround_plan` and ends with `status: not shown`, `hestia::measure_problem(&measure)` is None, `hestia::spec_gaps(..)` has no gap for OLY-B-RUN-01, and the preflight.rs file exists under `env!("CARGO_MANIFEST_DIR")`. Run: `cargo test -p athena-core capability_preflight_spec_entry && cargo test -p athena-core the_real_spec_records_every_field && git diff -- docs/spec.toml`. Observes: Both tests pass. The diff changes only OLY-B-RUN-01 (docs/spec.toml:2292-2296): mechanism from `none` to text naming preflight.rs, the run/mod.rs hook, run/pin.rs `stop_preflight` and the reading.rs retry link; saying bare-name check programs are probed on Athena's own PATH, path programs and relative PATH entries are recorded as not probed, git is checked earlier by Athena's own use, gh is not probed, the agent binary is checked when the driver is built (claude.rs:49-63, 89), and the per-tool record is only in job.json `note`; `owner = "Athena"`; a measure naming the specific test and ending `status: not shown`.
  Fails without the behavior: With the spec entry unchanged the mechanism is `none` and the measure ends `status: requirement`. The existing hestia test accepts that (hestia.rs:4293), but the new test fails on its own assertions: no preflight.rs in the mechanism, no test name in the measure, no `status: not shown` ending, and `spec_gaps` still lists OLY-B-RUN-01.
  Test seam: No double. Production `hestia::parse_spec` (hestia.rs:1172), `measure_problem` (1219) and `spec_gaps` (1291) read the real docs/spec.toml through include_str!. They exist today (read); the test is added by plan step 6.
- Criterion 2: Unit tests in preflight.rs call production `preflight::preflight(&needs, &probe)` and `needed_tools(&checks)`. The fake `FakeProbe { missing: Vec<&str>, relative: Vec<&str> }` only answers Present, Missing or NotProbed per name and is passed as an argument. `capability_preflight_missing_tool_becomes_a_blocker_with_a_workaround_plan`: checks `["cargo","fmt"]` and `["cargo","test"]`, FakeProbe missing `cargo`; asserts exactly one blocker with tool `cargo`, `needed_by` listing both check names, a non-empty workaround, a note containing `cargo: missing`, and `stop_reason(None)` starting `environment: preflight blocked:` and naming `cargo` and `workaround`. `capability_preflight_all_tools_present_gives_no_blocker`: same needs, nothing missing; no blockers, `stop_reason(None) == None`, every tool in `checked` and in the note as `present`. Also `capability_preflight_lists_path_programs_as_not_probed` (a path program is in `checked` as not probed, the note holds only its last component, never a blocker even with a probe that reports everything missing; an empty `run` is no need), `capability_preflight_shared_tool_listed_once`, `capability_preflight_needs_come_from_the_capability_checks` (specs shaped like athena.toml's cargo fmt/clippy/test give exactly `cargo`; a check running `git` gives `git`), `capability_preflight_git_alone_and_gh_are_not_needs`, `capability_preflight_stop_reason_names_the_earlier_job`, `capability_preflight_relative_path_is_not_judged_missing` (a tool absent from absolute entries with a relative entry present is `not probed (relative PATH)`, no blocker, via the real `SystemProbe::with_path`), and `capability_preflight_system_probe_finds_only_executable_files_on_path` (temp dir with an executable and a non-executable file via `SystemProbe::with_path`). Run: `cargo test -p athena-core capability_preflight_missing_tool && cargo test -p athena-core capability_preflight_all_tools_present && cargo test -p athena-core capability_preflight`. Observes: All pass. The missing-tool test shows one blocker naming `cargo` with a non-empty workaround; the all-present test shows none. The broad filter `capability_preflight` selects only the new tests.
  Fails without the behavior: If the body of `preflight()` is stubbed to return an empty `Preflight`, the missing-tool test fails (0 blockers not 1, no `cargo: missing` line, `stop_reason` None) and the all-present assertions on `checked` fail. Reviewed in the diff; step 9 stubs it and runs the test.
  Test seam: Entry points `preflight::preflight(&[Need], &dyn ToolProbe)` and `needed_tools`, added by plan step 1. `FakeProbe` is injected as a function argument and answers only the per-name status; blocker, workaround and note logic stay in production code. The real `SystemProbe` is exercised through `with_path` in two tests.
- Criterion 3: Tests in run/tests.rs go through `Athena::go` into `dispatch` (run/mod.rs:402, 1015) and `Athena::retry` (reading.rs:1185) with the existing harness (`repo()`, `athena_with(&repo,&state,check,plenty_of_space,script)`, `finished_job`, `counting`, `producer_ending`, Docs capability). Guards before the hook: paused-line check (mod.rs:1026), configuration-hash match (1030), `disk_gate`/`start_hold` (tests use `plenty_of_space`), budget reserve (1095), `pin_repo` (Docs does not pin). TEST A `capability_preflight_blocks_a_missing_tool_before_the_producer_and_records_it`: check `&["athena-preflight-absent-tool"]`, script `counting(counts, None)`; asserts Failed, a reason starting `environment: preflight blocked:` naming the tool and `workaround`, `is_environment_reason`, `StopCause::of == OtherEnvironment`, zero producer and verifier calls, and, loading `Records` at `finished.prediction_commit`, a Job `note` containing `athena-preflight-absent-tool: missing`, no candidate yet. TEST B `capability_preflight_records_every_probed_tool_when_all_are_present`: check `PASSES` (`true`); asserts the producer ran once, the reason does not start with `environment:`, and the Job record at `finished.prediction_commit` (before any candidate, showing the order) has a note containing `true: present`; a variant with a second check named by an absolute path under a fake home directory shows its `not probed (path)` line with only the last component and no directory. TEST C `capability_preflight_stop_on_a_resumed_retry_keeps_the_earlier_work`: a first Athena with `producer_ending(Error, &expired_login(), true, ..)` closes failed with a login-expiry stop and keeps a candidate commit but no kept directory (tests.rs:5368-5369), so after `finished_job` the test creates `athena.state.kept_work_dir(&first)` by hand with `create_dir_all` (as tests.rs:5249-5250). A second Athena over the same repo and state tempdirs (`athena_with`) has check `athena-preflight-absent-tool`; `retry(first)` then `go`. Asserts: Failed with the preflight reason naming `first`; `kept_work_dir(&first)` still exists; the first job's candidate commit is still in the repository. TEST D `capability_preflight_retry_of_a_stopped_job_resumes_the_job_it_retried`: a third Athena with check `PASSES` runs `retry(stopped)`; asserts the returned proposal's stored request text is `try again on <first>` and `retried_job(&records, text) == Some(first)` (the one function gate_counts, evidence_labels, steward, throughput, status/planning and scope_guard read the link through); then `go` finishes, the producer ran once, the dispatch commit's producer brief names `first` as the job retried and carries the first job's progress note, and `kept_work_dir(&first)` is gone. TEST E `capability_preflight_retry_of_a_plain_stop_is_unchanged`: a job that failed for another reason, and a preflight-stopped job whose own request is not a retry, both get `try again on <that job>` as before. Run: `cargo test -p athena-core capability_preflight_blocks_a_missing_tool && cargo test -p athena-core capability_preflight_records_every_probed_tool && cargo test -p athena-core capability_preflight_stop_on_a_resumed_retry && cargo test -p athena-core capability_preflight_retry_of_a_stopped_job && cargo test -p athena-core capability_preflight_retry_of_a_plain_stop && cargo test -p athena-core`. Observes: A: Failed with the environment reason, no producer call, blocker in the note and the outcome. B: the producer ran and the note lists each probed program, recorded in the prediction commit. C: the stop names the earlier job; its kept directory and candidate survive. D: the retry of the stopped job is stored as a retry of the first job and resumes its work, consuming its kept directory. E: other retries are unchanged. The last command runs the whole crate.
  Fails without the behavior: If the hook is removed or runs after the producer, A fails (the producer runs, reason not environment). If `Job.note` is not set, A and B fail on the note assertions. If the context keeps `resume` on a blocked job, C fails because close.rs:152-154 removes `kept/<first>`. If `Athena::retry` does not use `retry_target`, D fails: the stored request is `try again on <stopped>`, `retried_job` returns the stopped job, the brief does not name `first` and the kept directory stays. Step 9 removes `resume: None` and the `retry_target` call in turn and runs C and D.
  Test seam: Entry points `Athena::go` (run/mod.rs:402) into `dispatch` (1015) and `Athena::retry` (reading.rs:1185). Fixtures: `FakeAgentDriver` with a script, the Docs capability and a temp repo, all existing in run/tests.rs (read). The preflight uses the real `SystemProbe` on the process PATH: `true` exists there, `athena-preflight-absent-tool` does not; no injection needed. C and D build further `Athena` values over the same tempdirs through `athena_with`, which derives the state path from the tempdir (tests.rs:198); existing tests already retry then `go` (tests.rs:3707, 3770). Tests are added by plan step 7.
- Criterion 4: Run the repository's own checks from the root. New code avoids `unwrap` outside tests; lib.rs module list and doc bullet are updated. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. Existing tests that dispatch jobs with the real probe still pass (run/tests.rs, run_pin_tests.rs, athena-cli lifecycle, drain, retry, trailer_stop). Their bare-name check programs (`sh`, `true`, `cargo`, `git`, `grep`) are on PATH; the path program `/nonexistent/athena-baseline-check` is not probed. Existing `athena.retry` callers (tests.rs:1700, 3707, 3770, 4183, 4378, 4900, 5002, 5172, 5663; scope_guard.rs:1734-1963; run_pin_tests.rs:1821) retry jobs that were not preflight-stopped, so `retry_target` returns their own id.
  Fails without the behavior: A failing test, rustfmt difference or clippy warning makes the matching command exit non-zero; without the hook the criterion 3 tests fail.
  Test seam: None: the repository's own entry points. New tests use the seams named for criteria 2 and 3.
- Criterion 5: Show the stricter behavior is intended and bounded. (1) Test A: a job whose bare-name check program is absent is refused (the newly refused case; today it lands with a pre-existing failure, checks.rs:541-562). (2) Test B: with the program present the job proceeds and records it. (3) `capability_preflight_lists_path_programs_as_not_probed`, `capability_preflight_relative_path_is_not_judged_missing` and `capability_preflight_git_alone_and_gh_are_not_needs` show the need list holds only check programs, with paths and relative-PATH tools listed as not probed. (4) The unchanged lifecycle test `baseline_failure_does_not_stop_the_job` (lifecycle.rs:129-152) still lands a job whose check is a missing path; it pins that path programs are outside the preflight, not that only missing tools refuse. The 0% newly-refused figure is an estimate by reading, limited to this repository's checks: all six in athena.toml (lines 85, 91, 99, 116, 120, 125) run `cargo`. Run: `cargo test --locked -p athena-cli --test lifecycle baseline_failure_does_not_stop_the_job && cargo test -p athena-core capability_preflight_blocks_a_missing_tool && cargo test -p athena-core capability_preflight_records_every_probed_tool && cargo test -p athena-core capability_preflight_lists_path_programs && cargo test -p athena-core capability_preflight_git_alone_and_gh_are_not_needs`. Observes: All pass: an absent bare-name program is refused with the environment reason; a present program proceeds; path programs are listed as not probed and never block; the lifecycle job still lands. For a repository with an uninstalled bare-name check tool the share is not zero and is intended.
  Fails without the behavior: If path programs were probed, the lifecycle test would end `failed` with the preflight reason instead of landing, and the path-listing test would fail. If gh or git alone were added to the needs, `capability_preflight_git_alone_and_gh_are_not_needs` fails. Without the hook, Test A fails.
  Test seam: `Athena::go` as in criterion 3 with the real probe; the lifecycle test exists today (read). `needed_tools` is called directly in the unit tests.
- Criterion 6: Inspect the change list: athena.toml, defaults.toml, .github/workflows and records/ untouched; CHANGELOG.md gains one entry under `## [Unreleased]` / `### Added`; docs/spec.toml changes only OLY-B-RUN-01. Run: `git status && git diff --stat`. Observes: Changed paths: docs/spec.toml, CHANGELOG.md, crates/athena-core/src/preflight.rs (new), lib.rs, reading.rs, run/mod.rs, run/pin.rs, run/tests.rs. scope_guard.rs, athena-cli, athena-data and athena-gate are unchanged. No governance file or records/ path appears.

Planned files:

- crates/athena-core/src/preflight.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/pin.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 482eff8818ab → candidate be43344574ef. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   1 +
 crates/athena-core/src/lib.rs       |   4 +
 crates/athena-core/src/preflight.rs | 625 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs   |  18 +-
 crates/athena-core/src/run/mod.rs   |  25 +-
 crates/athena-core/src/run/pin.rs   |  11 +-
 crates/athena-core/src/run/tests.rs | 327 +++++++++++++++++++
 docs/spec.toml                      |   4 +-
 8 files changed, 1006 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-013710-8gwp/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013710-8gwp/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43202935428 for be43344574ef4ccf120585b30aa3d9f9fbf98bd0)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43202935428 for be43344574ef4ccf120585b30aa3d9f9fbf98bd0)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43202935428 for be43344574ef4ccf120585b30aa3d9f9fbf98bd0)
- ci-result: `not a check` passed (Check results for be43344574ef4ccf120585b30aa3d9f9fbf98bd0 come from the CI run https://github.com/doodla/athena/actions/runs/43202935428 (875s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-core capability_preflight_spec_entry && cargo test -p athena-core the_real_spec_records_every_field && git diff -- docs/spec.toml`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core capability_preflight_missing_tool && cargo test -p athena-core capability_preflight_all_tools_present && cargo test -p athena-core capability_preflight`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-core capability_preflight_blocks_a_missing_tool && cargo test -p athena-core capability_preflight_records_every_probed_tool && cargo test -p athena-core capability_preflight_stop_on_a_resumed_retry && cargo test -p athena-core capability_preflight_retry_of_a_stopped_job && cargo test -p athena-core capability_preflight_retry_of_a_plain_stop && cargo test -p athena-core`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test --locked -p athena-cli --test lifecycle baseline_failure_does_not_stop_the_job && cargo test -p athena-core capability_preflight_blocks_a_missing_tool && cargo test -p athena-core capability_preflight_records_every_probed_tool && cargo test -p athena-core capability_preflight_lists_path_programs && cargo test -p athena-core capability_preflight_git_alone_and_gh_are_not_needs`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
