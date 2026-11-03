# Job 20261103-015346-82n3: verify "Add athena focus command and urgent/high priority caps in plan"

## Request (verbatim)

> try again on 20261031-082442-92qg

## Reading (accepted by the user)

Retry of job 20261031-082442-92qg (dispatched, never verified, no findings): add `athena focus "<text>"` backed by a new focus record, shown in `athena status`, plus athena.toml caps (3 urgent, 10 high) that make `athena plan` demote excess items one tier in memory. Audience: the person running Athena, who needs priority to order work again.

Assumptions:
- Code survey: read crates/athena-data/src/records.rs (PriorityRecord near line 525, AnyRecord enum near line 2623, with its validation and round-trip test), crates/athena-core/src/plan.rs (Plan::build at line 520), crates/athena-core/src/store.rs (priorities loaded as Vec<PriorityRecord>), crates/athena-core/src/reading.rs (builds NewFile::record(&AnyRecord::Priority(..))) and crates/athena-cli/tests/*.rs (e.g. drain.rs builds priority records). The change adds a Focus variant to AnyRecord, so the records validation allow-list, path and schema name and the AnyRecord round-trip test must cover it. Plan demotion is measured directly on Plan output, not through a stand-in.
- Job 20261031-082442-92qg is retried from the base commit. It was never verified and recorded no findings, so there is nothing to fix; its approach is kept as guidance. Its 12 changed files are all kept as deliverables, including crates/athena-cli/src/drain.rs, which the earlier reading missed.
- The focus record directory (records/focus/) does not exist at the base commit. This change adds it only at runtime, when a test or user runs `athena focus`. No file under records/ is created or edited by hand.
- The focus record holds text, set_by (git user.name) and an ordering counter or timestamp. The latest record is the current focus.
- The schema struct for the caps defaults to urgent=3 and high=10, so existing configs and reading digests do not change. Demotion happens in memory in plan only. A person's priority record always keeps its item at that tier.
- Demotion order: person-set items first, most recent first, then the rest in plan order. Urgent excess drops to high and counts in the high pass. High excess drops to normal.

## Acceptance criteria

1. [behavior] Run `cargo test`: the new crates/athena-cli/tests/focus.rs runs `athena focus "speed first"` in a temp repo, asserts a focus record file exists in the new focus records directory, that `athena focus` prints "speed first", and that `athena status` output contains it.
2. [behavior] Run `cargo test`: a test sets a second focus after the first and asserts `athena focus` and `athena status` print only the later text.
3. [behavior] Run `cargo test`: a plan test with 5 urgent items asserts exactly 3 are treated as urgent, a person-set urgent item (via a priority record) is among them, and the plan output has one demotion line for each of the 2 demoted items.
4. [behavior] Run `cargo test`: a test asserts the high cap of 10 demotes excess high items to normal, a person's priority record still wins for its own item, and no request or priority record file is modified by planning.
5. [content] athena.toml sets the urgent cap to 3 and the high cap to 10, and crates/athena-data/src/config.rs declares matching schema fields with those defaults; Grep shows both.
6. [content] crates/athena-data/src/records.rs adds the focus record to AnyRecord with validation (non-empty text and set_by), a path and a schema name. The AnyRecord round-trip test covers it, and check_record_file accepts it.
7. [content] CHANGELOG.md has an entry describing `athena focus`, the status line and the priority caps.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/focus.rs (create)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- The steward triage pass that uses the focus (a separate request)
- Re-tiering or rewriting existing queue items or priority records
- Editing anything under records/ by hand
- Changes to defaults.toml, .github/workflows and prompt files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for focus and caps; cargo test and clippy -D warnings must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run focus, status and plan and check their output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the plan demotes the excess items and focus persists, not that the wiring exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to focus and the caps.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verified by running the tests and commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test in the new crates/athena-cli/tests/focus.rs. It uses the `common` Fixture (a temp git repo) and runs the real binary as binary.rs does. It runs `athena focus "speed first"`, lists records/focus/ in the fixture working tree, then runs `athena focus` and `athena status`. Run: `cargo test --locked -p athena-cli --test focus`. Observes: A file records/focus/focus-1.json exists. `athena focus` stdout contains `speed first`. `athena status` stdout has a `focus` line containing it.
  Fails without the behavior: With no `focus` subcommand the command errors; with no Focus record the file is missing; with no focus line in `display::status` the status output lacks the text.
  Test seam: Production entry points: the `athena focus` clap command (`Command::Focus`, main.rs, added in step 9) calling `Athena::set_focus` (step 6), and `athena status`. Fixture: `common::Fixture`, which sets git user.name; the binary is the real one, so there is no double. Everything exists today except the focus command and `set_focus`, which the steps add.
- Criterion 2: A second test in focus.rs sets focus `first thing`, then `speed first`, then runs `athena focus` and `athena status`. Run: `cargo test --locked -p athena-cli --test focus`. Observes: Both outputs contain `speed first` and neither contains `first thing`. Both focus-1.json and focus-2.json exist, so the later record is chosen and the first is not overwritten.
  Fails without the behavior: If 'latest' were the first record or the last file in directory order, the earlier text would be printed.
  Test seam: Same real binary and Fixture as criterion 1. The highest-n choice lives in `Records::current_focus` (step 4). The test checks only output.
- Criterion 3: Three tests. (a) A unit test in plan.rs `mod tests` calls `Plan::build_capped` on five urgent entries (existing `entry` and `with_priority` helpers) with caps {urgent:3, high:10} and one person-set id. (b) A unit test in plan.rs with cap+1 person-set urgent entries asserts none is in `plan.demoted`. (c) An integration test in focus.rs queues 5 urgent requests with acceptance bullets, sets a priority record on one of them via `Athena::set_priority`, and calls `drain::plan` with `RealSteps::live`. It renders `display::plan` on the report. A fourth test in drain.rs `mod tests` runs `Drain::prepare` with a Steps double that returns caps {urgent:3} on 5 urgent entries plus one older native high entry, and asserts no note contains `contradicts priority`. Run: `cargo test --locked -p athena-core plan:: && cargo test --locked -p athena-cli --test focus && cargo test --locked -p athena-cli drain::`. Observes: Exactly 3 urgent ids come first in `plan.order`, the person-set id among them, and `plan.demoted` has 2 entries (urgent to high) whose ids come after the first 3. The rendered plan text has exactly 2 lines starting `note        demoted`, one per demoted id. With cap+1 person-set items, `demoted` has none of them. The prepare test has no 'contradicts priority' note, which proves violations are judged against the capped entries.
  Fails without the behavior: With no capping all 5 stay urgent, `demoted` is empty and no demotion note is printed. If the person-set item were ranked by plan order it could be demoted. If `priority_violations` still ran on the uncapped entries, the prepare test would see a false contradiction note.
  Test seam: Production entry points: `Plan::build_capped` (step 5), `Drain::prepare` and `drain::plan` (step 8), `display::plan` (exists). The unit tests pass plain entries, so there is no double. The integration test uses the real `RealSteps` over the Fixture with fake agents (`athena_agent::fake::deterministic_turn`, as drain.rs tests do), and real priority records from `Athena::set_priority` (exists, reading.rs:915). The drain.rs prepare test uses a small test-only Steps whose `priority_caps` and `person_set` overrides return fixed values; the capping and the violation check run in production code, not in the double. The `Steps` default methods and the `RealSteps` overrides are added in step 8.
- Criterion 4: A plan.rs unit test with 12 high entries, one of them person-set, asserts the person-set item stays high and is absent from `plan.demoted`, and exactly 2 others are listed `high -> normal`. Another unit test shows an urgent item demoted to high counts in the high pass and is not demoted again. An integration test in focus.rs sets a priority on one queued job, snapshots the bytes of every file under records/ and the HEAD commit, runs `drain::plan` with `RealSteps`, then compares both and asserts `Records::load` gives the same priorities. Run: `cargo test --locked -p athena-core plan:: && cargo test --locked -p athena-cli --test focus`. Observes: The person-set item is not demoted. The excess high items are listed `high -> normal`. Request and priority files are byte-identical and HEAD is unchanged after planning.
  Fails without the behavior: If planning rewrote a priority or request record, the byte comparison or HEAD check would differ. If the record did not win, the person-set item would appear in `demoted`.
  Test seam: Production entry points: `Plan::build_capped` and `drain::plan` with `RealSteps`. The Fixture, fake agents and `Athena::set_priority` exist today. The `RealSteps` overrides come from step 8. No logic sits in a double.
- Criterion 5: A config.rs unit test parses the repository athena.toml (the existing `REPO_CONFIG` constant) and asserts `priority_caps()` gives 3 and 10. It also parses a copy with the table removed and asserts the same defaults and an unchanged `reading_inputs()`. Then Grep both files. Run: `cargo test --locked -p athena-data config && grep -n -A3 "priority_caps" athena.toml crates/athena-data/src/config.rs`. Observes: athena.toml has `[priority_caps]` with urgent = 3 and high = 10. config.rs declares `PriorityCaps` with `DEFAULT_URGENT_CAP = 3` and `DEFAULT_HIGH_CAP = 10`. The strict parser accepts the table.
  Test seam: Entry points `Config::parse` and `Config::priority_caps`. `REPO_CONFIG` exists today (config.rs tests).
- Criterion 6: Tests in records.rs `mod tests`. A round trip builds `AnyRecord::Focus`, serializes with `to_json`, parses with `AnyRecord::parse` and compares. It asserts `canonical_path()` is `records/focus/focus-1.json` and `schema()` is `athena.focus/1`. `check_record_file` accepts the canonical path and rejects a misplaced one. Blank text and blank set_by are rejected. Run: `cargo test --locked -p athena-data records`. Observes: The round trip is equal. The path and schema are as named. `check_record_file` returns Ok(Some(Focus)) at the canonical path and an Invalid error for blank text, blank set_by or a wrong path.
  Test seam: Entry points `AnyRecord::parse`, `AnyRecord::canonical_path` and `check_record_file`, all existing. No double.
- Criterion 7: Read the CHANGELOG.md diff for the Unreleased section. Run: `git diff -- CHANGELOG.md`. Observes: A new Unreleased entry names `athena focus`, the `focus` line in `athena status`, and the `[priority_caps]` caps (3 urgent, 10 high, demoted in memory in `athena plan` and `drain`, person-set items never demoted).
- Criterion 8: Run the three checks Athena runs on the whole workspace, then the refusal corpus as a regression check. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0, including existing drain.rs and reading.rs tests that build PlanEntry and Plan, and the boundary and policy tests. `cargo test --locked -p athena-core --test refusal_corpus` is part of `cargo test --locked`.
  Fails without the behavior: An exhaustive `AnyRecord` match missing Focus, a missing `Plan` or `StatusView` field in a struct literal, or a clippy lint would fail one of the three.
  Test seam: The full workspace suite. No seam of its own.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/focus.rs (create)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c87af70676f4 → candidate 4fb309d8d69e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  14 ++
 athena.toml                       |   7 +
 crates/athena-cli/src/display.rs  |   3 +
 crates/athena-cli/src/drain.rs    | 164 ++++++++++++++++++++++-
 crates/athena-cli/src/main.rs     |  26 +++-
 crates/athena-cli/tests/focus.rs  | 269 ++++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/plan.rs    | 228 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs |  55 +++++++-
 crates/athena-core/src/status.rs  |   4 +
 crates/athena-core/src/store.rs   |   9 ++
 crates/athena-data/src/config.rs  |  91 +++++++++++++
 crates/athena-data/src/records.rs |  74 +++++++++++
 xtask/shard-timings.txt           |   1 +
 13 files changed, 939 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41567855572 for 4fb309d8d69e35b702ab5a4e83e96f67c7557b72)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41567855572 for 4fb309d8d69e35b702ab5a4e83e96f67c7557b72)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41567855572 for 4fb309d8d69e35b702ab5a4e83e96f67c7557b72)
- ci-result: `not a check` passed (Check results for 4fb309d8d69e35b702ab5a4e83e96f67c7557b72 come from the CI run https://github.com/doodla/athena/actions/runs/41567855572 (604s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
