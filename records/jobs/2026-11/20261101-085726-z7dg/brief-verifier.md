# Job 20261101-085726-z7dg: verify "End drain cleanly when landed config needs a newer build"

## Request (verbatim)

> After a landing that adds a new key to athena.toml, the drain's running binary re-reads the landed config, cannot parse the new key, records the landed job as failed and stops the run with 'repeated failure' even though the job landed. Make the drain treat a config that only the newer landed build understands as a reason to end the run cleanly (and say to reinstall), not as a failure of the job.
> - A test lands a job whose diff adds a key the running config schema does not know; the job is recorded as landed, not failed, and the run ends with a note to reinstall rather than 'repeated failure'.
> - Changes stay in crates/athena-core/src/run/ and the drain code; CHANGELOG.md gets an entry.

## Reading (accepted by the user)

When a landed job's athena.toml adds a key the running binary's schema rejects, the drain should keep the job recorded as landed and end the run with a "reinstall" note, not count a failure or stop on "repeated failure". Goal: a drain operator whose run lands a config-extending job sees a clean stop with reinstall guidance.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (Stop enum ~l.923-985, land() ~l.1617, item loop ~l.1700-1762, ItemEnd::Failed -> Stop::RepeatedFailure ~l.2308, RealSteps::caps/priority_caps calling JobConfig::at(head) ~l.3248/3332), crates/athena-core/src/capability.rs (JobConfig::at parses athena.toml with Config::parse, strict on unknown fields), crates/athena-core/src/admission.rs (configuration rule, unknown field), and the file list of crates/athena-core/src/run/. The exact place where the post-landing config parse error becomes a failed job was not pinned down; the producer must locate it by running the new test first.
- The drain code lives in crates/athena-cli/src/drain.rs (and its integration tests in crates/athena-cli/tests/drain.rs); 'the drain code' in the request is read as that, plus crates/athena-core/src/run/ if the landing/record logic is there.
- The new stop is a new Stop variant (or reuses Stop::Rebuilt-style handling) with a label and meaning that tells the user to reinstall; Stop::ALL and display/help text in crates/athena-cli/src/display.rs and main.rs stay consistent with it, which counts as drain code.
- No new record or data file is added under crates/, so check_record_file/AnyRecord are unaffected; the job's recorded state stays landed.
- Only a config parse failure at the landed commit is treated this way; a config that fails to parse on its own (not a newer-schema issue) cannot be told apart cheaply, so the producer may treat any parse error of the landed head config, when the landing itself succeeded, as this case, and says so in a code comment.
- No change to athena.toml, defaults.toml or workflows; the test uses a fixture config with a key the schema lacks.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (or the drain.rs unit tests) lands a job whose diff adds an unknown key to athena.toml; `cargo test -p athena-cli` passes, the test asserts the job's recorded outcome is landed (not failed), and the report's stop is not Stop::RepeatedFailure.
2. [behavior] In that test the run's final report/output contains a note telling the user to reinstall, and the run ends cleanly (no failure counted for the item); checked by `cargo test -p athena-cli`.
3. [behavior] A regression test shows a genuine failure (job that does not land twice) still ends with Stop::RepeatedFailure; `cargo test -p athena-cli` passes.
4. [content] The change touches only files under crates/athena-core/src/run/, crates/athena-cli/src/drain.rs, its stop display text, and their tests (git diff --stat), plus CHANGELOG.md.
5. [content] CHANGELOG.md has a new entry describing that a landed config the running build cannot parse ends the run cleanly with a reinstall note instead of a false repeated failure.
6. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass (tested default).
7. [content] The new stop reason (if added) has a label and meaning and is listed wherever stops are enumerated (Stop::ALL, help text in crates/athena-cli/src/main.rs, display), verified by the existing test that every Stop has a meaning.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Making the config parser tolerant of unknown keys
- Automatically reinstalling or rebuilding the binary
- Changes to athena.toml, defaults.toml, or admission rules
- Changes to the athena-gate crate or other crates beyond the drain and run code

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and test are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test, clippy, fmt criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix is in how the drain classifies post-landing config parse errors, not a special case for one key.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the recorded landed state and stop reason by running the test.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test runs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Reinstall note and CHANGELOG entry written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change confined to drain and run code.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a test rather than prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two tests in crates/athena-cli/tests/drain.rs. (a) Stub test: `Stub` gains `config_unreadable_after_landing: Option<&'static str>`, returned by its new `config_after_landing` once a `land ..` entry is in its log. Two items `a` and `b` with `passing()` outcomes. Assert `report.stop == Stop::NeedsReinstall`, `report.stop != Stop::RepeatedFailure`, `report.landed == 1`, `report.jobs[0].outcome == "passed"` and `report.jobs[0].landing == "landed"`. (b) Real-steps test: `Fixture::new()`, two requests from `queue_steward_request`, `RealSteps` with a test `Lander` that calls `commit_config(&fx, &(test_config() + "\n[brand_new_section]\nnew_key = 1\n"))` and returns `Landing::Landed`. After `drain::drain`, load `Records::load(&fx.git, &fx.head())` and assert the first job's state is `JobState::Closed(OutcomeState::Landed)`, the stop is `NeedsReinstall`, and the second job was not run. Run: `cargo test -p athena-cli --test drain a_landing_whose_config_the_running_build_cannot_read`. Observes: The landed job is recorded landed (report line passed/landed; record state Closed(Landed)), the report's stop is NeedsReinstall and not RepeatedFailure, and no second attempt or failure is counted.
  Fails without the behavior: Without the classification the next item's read or run fails on `JobConfig::at(head)`, the item is counted failed twice, and the stop is `Stop::RepeatedFailure` (or the item is skipped), so the stop assertion fails.
  Test seam: Production entry point: `drain::drain(&steps, limits, &Quiet)` driving `Drain::land`/`finish`. Seam (a) is `Stub` in tests/drain.rs; the new `Steps::config_after_landing` is added by plan step 1 with a default of `None`, and the Stub field by step 6. Seam (b) is `RealSteps` with a custom `Lander`, which exists today (`Lander` is a pub trait; `RecordingLander` at tests/drain.rs ~l.1626 is the same pattern). `RealSteps::config_after_landing` (step 2) calls the real `JobConfig::at`, with the fixture repo injected via `fx.athena(..)`. The unknown-key config is a real commit made by the test lander, so the parse failure comes from the production strict parser. The drain logic is not in the double.
- Criterion 2: The same two tests also render `display::drain(&report)` and assert it contains `stopped     reinstall needed` and the word `reinstall`. They assert `report.notes` has a note with the parse error text that tells the user to reinstall and run `athena drain` again. They assert the stub log has no `rate` entry and the second item has no `run` entry, and that no job line has outcome `failed`, `rejected` or `no outcome` (zero failures counted). Run: `cargo test -p athena-cli --test drain a_landing_whose_config_the_running_build_cannot_read`. Observes: The text the user sees says to reinstall, the run ends with the dedicated stop, no rating was attempted, and no failure line exists.
  Fails without the behavior: Without the new stop and note the output reads 'repeated failure: item .. failed twice' and has no reinstall wording.
  Test seam: Production entry points: `drain::drain` and `display::drain(&report)` (exists, display.rs ~l.617). Fixtures as in criterion 1: Stub field added in step 6, real fixture and test lander exist today.
- Criterion 3: Add `a_job_that_does_not_land_twice_still_ends_as_repeated_failure_when_the_config_is_readable`. It uses a `Stub` with `unlandable: vec!["a"]` and `config_unreadable_after_landing: None`. It asserts `Stop::RepeatedFailure` with `failed_item == Some("a")`. The existing repeated-failure assertions in tests/drain.rs also run and must pass. Run: `cargo test -p athena-cli --test drain repeated_failure`. Observes: A genuine landing failure is still counted and stops the run as repeated failure; the new check does not swallow it, because it runs only after `Landing::Landed`.
  Fails without the behavior: If the check ran on every landing result, or the stop were set whenever the config read fails, this test and the existing repeated-failure tests would end with NeedsReinstall or a different stop.
  Test seam: Production entry point: `drain::drain` with `Stub` (exists). The `unlandable` field exists today. `Steps::config_after_landing` is added in step 1 with a default of `None`, so existing fakes need no change.
- Criterion 4: Inspect the diff against the base for touched paths. Expected: crates/athena-cli/src/drain.rs, display.rs (only if needed), main.rs, crates/athena-cli/tests/drain.rs, CHANGELOG.md, and run/ files only if the producer found a failure recorded there. Run: `git diff --stat`. Observes: Only the listed files appear; no athena.toml, defaults.toml, workflow, gate crate or other crate file.
- Criterion 5: Read the added CHANGELOG lines under `## [Unreleased]`, in its `### Fixed` section. Run: `git diff -- CHANGELOG.md`. Observes: A new entry says that when a landing adds config the running build cannot parse, the drain records the job as landed, ends the run with the stop `reinstall needed`, and tells the user to reinstall, where before it counted a false repeated failure.
- Criterion 6: Run the full workspace test suite: `cargo test --locked`. It runs the new tests from criteria 1-3 and 7 together with every existing test, including the budgets and help tests. Then, as two separate commands, run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`. All three must pass with the final tree. Run: `cargo test --locked`. Observes: Exit status 0 for all three commands: no existing test regresses, the new tests pass, clippy is clean and formatting is unchanged. The budget and help tests in crates/athena-cli/tests/budgets.rs are unaffected because no prompt asset or system-facts line changes.
  Fails without the behavior: Without the behavior the new tests in tests/drain.rs fail (a landed job counted as 'repeated failure'). A `Stop` variant missing a `label`/`meaning` arm, or a `Steps` method added without handling in the fakes, fails to compile; unused code or an unhandled match would fail clippy with -D warnings.
  Test seam: Production entry points: the `cargo test`, `cargo clippy` and `cargo fmt` gates over the whole workspace, i.e. the same three Athena runs on the result. The fixtures are the tests added by plan steps 1-6 (Stub field, RealSteps with test Lander, injected via `Fixture`), which exist in tests/drain.rs after those steps; the gates themselves exist today (CI and the 'Checks Athena will run' list). No double is involved.
- Criterion 7: `Stop::ALL` gets `NeedsReinstall` (array length 8 to 9). The unit test `every_stop_reason_has_a_label_and_a_meaning` (drain.rs ~l.4084) loops over `Stop::ALL`. The integration test `drain_help_documents_the_flags_and_lists_every_stop_reason` (tests/drain.rs ~l.1559) runs the real binary's `drain --help` and checks every `stop.label()` appears. The help line is added to main.rs (~l.261). The producer also adds an assertion that `NeedsReinstall.meaning()` contains `reinstall`. Run: `cargo test -p athena-cli every_stop_reason_has_a_label_and_a_meaning drain_help_documents_the_flags_and_lists_every_stop_reason`. Observes: The new stop has a non-empty label and a meaning with the reinstall guidance, and its label is printed in `athena drain --help`.
  Fails without the behavior: If the stop were left out of `Stop::ALL` or the help text, the help test fails; with a missing meaning arm the code does not compile.
  Test seam: Production entry points: `Stop::ALL`/`label`/`meaning` (exist) and the `athena` binary via `CARGO_BIN_EXE_athena` (exists). No double.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7206e8cc7063 → candidate 806666120f7d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  10 +++
 crates/athena-cli/src/drain.rs   |  72 ++++++++++++++++-
 crates/athena-cli/src/main.rs    |   2 +
 crates/athena-cli/tests/drain.rs | 167 +++++++++++++++++++++++++++++++++++++++
 4 files changed, 250 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41573690404 for 806666120f7df80f3aca1c9c480ee8ce1572fe54)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41573690404 for 806666120f7df80f3aca1c9c480ee8ce1572fe54)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41573690404 for 806666120f7df80f3aca1c9c480ee8ce1572fe54)
- ci-result: `not a check` passed (Check results for 806666120f7df80f3aca1c9c480ee8ce1572fe54 come from the CI run https://github.com/doodla/athena/actions/runs/41573690404 (573s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass (tested default).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
