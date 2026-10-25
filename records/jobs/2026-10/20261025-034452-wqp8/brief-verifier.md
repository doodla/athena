# Job 20261025-034452-wqp8: verify "Add urgent priority and a priority-record command that plan uses"

## Request (verbatim)

> Priority: high
> Speed first (BRIEF.md addendum of 2026-10-25, 'Athena should prioritize anything that'll help it move faster ( includes the time wasted )'): add a priority level above high, urgent, and make Athena rank by it. The reader marks a request urgent, with its reason recorded in the reading, when it makes Athena faster or cuts wasted time (failed or rejected attempts, planning stops, timeouts, retries that redo work, thrash, duplicated checks). Add a command that records a priority for a queued request as a record (requests stay append-only), which plan uses over the priority in the request's text, so the order can change without re-queuing.
> Acceptance:
> - A test shows urgent ordered before high, high before normal, with dependencies still first.
> - A test shows the command's priority record overriding the text's priority, and the change shown in athena plan and status.
> - A reading test shows a speed request marked urgent with a reason, and an ordinary one not.
> Files: crates/athena-core/src/plan.rs, crates/athena-core/src/reading.rs, crates/athena-cli/src/main.rs, crates/athena-data/src/records.rs, crates/athena-context/prompts/reader.md, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Add an Urgent priority above High to plan ordering, and a CLI command that appends a priority record for a queued request, which plan and status use over the request text's priority. The reader's urgent-marking rule goes in outputs.rs or briefs.rs (reader.md is at its token budget), with a reason recorded in the reading.

Assumptions:
- Goal: let Athena's operator reorder queued work without re-queuing and have speed-improving requests run first; audience is the operator and the reader agent.
- Per the user's reply, the reader rule goes in outputs.rs or briefs.rs, net of removals, since reader.md is at 1997/2000 tokens; reader.md is not edited unless text is removed.
- The reading records urgency as a priority plus reason field or text in the existing output; exact schema chosen by the producer.
- Priority records are a new record kind in athena-data records.rs; latest record wins.

## Acceptance criteria

1. [behavior] Run cargo test: a plan test shows urgent before high, high before normal, with a dependency still ordered first even when the dependent is urgent.
2. [behavior] Run cargo test: a test shows a priority record for a queued request overrides the priority in its text, and the order shown by plan and the priority shown by status change, with no new request record written.
3. [behavior] Run cargo test: a reading test shows a speed-improving request marked urgent with a recorded reason, and an ordinary request not urgent.
4. [content] The new CLI command appends a priority record (requests are not rewritten), rejects unknown priority values and unknown or non-queued requests with a clear error; Priority::from_text parses 'Priority: urgent'; main.rs help mentions urgent and the command.
5. [content] The urgent-marking rule (makes Athena faster or cuts wasted time: failed/rejected attempts, planning stops, timeouts, redo-work retries, thrash, duplicated checks; reason required in the reading) lives in the outputs.rs field doc or the briefs.rs reader brief, not in reader.md; reader.md does not grow.
6. [behavior] Run cargo test --locked -p athena-cli --test budgets: the reader set stays within its 2000-token budget.
7. [content] CHANGELOG.md has an entry and docs/decisions.md records the decision with alternatives considered.
8. [behavior] cargo clippy, cargo fmt --check are clean and the full cargo test passes.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/reader.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Rewriting or mutating existing request records
- Growing reader.md beyond the token budget
- Changing athena.toml or defaults.toml
- Changing priority semantics beyond adding urgent and the override

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for ordering, override, and reading; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing ordering and override behavior.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Ordering enforced in plan code and tested; prompt budget enforced by existing budgets test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and changelog written for outside readers.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Decision with alternatives recorded in docs/decisions.md.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A unit test in plan.rs builds entries with Urgent, High, Normal and Low priorities and a dependency chain. It asserts the order urgent < high < normal < low, and that an urgent dependent still comes after its low-priority prerequisite. Because the prerequisite inherits the urgent dependent's priority, it also comes before unrelated high items. It also asserts `Priority::Urgent < Priority::High`. Run: `cargo test -p athena-core plan::`. Observes: The test passes, showing the order urgent, high, normal, low, and the dependency before its urgent dependent.
  Fails without the behavior: Without the Urgent variant sorted before High, the urgent entry would not come first, or the test would not compile. Without the dependency rule, the urgent dependent would precede its prerequisite.
- Criterion 2: Unit tests in reading.rs: a job whose text says `Priority: normal` plus a priority record of urgent. Assert that `plan_entry(...).priority` is Urgent, that Plan::build puts it before an older high item, that the status line's priority (`status::job_line`) is Urgent, and that the request is unchanged. A CLI integration test in crates/athena-cli/tests/priority.rs uses the Fixture to queue two requests with `ask --later`, then reads the plan order and status. It runs set_priority (the same function the command calls), reads them again, and asserts the order and status changed. It also checks that exactly one new `priority-1.json` file was added, request.json is byte-identical, and an unknown level or a non-queued or unknown job errors. Run: `cargo test --locked -p athena-core priority && cargo test --locked -p athena-cli --test priority`. Observes: Plan order and the status priority flip after the record, with no new or changed request record. Errors name the bad level or job.
  Fails without the behavior: If plan_entry or job_line ignored the record, the order and status would not change. If the command rewrote the request, the byte-identity check would fail.
- Criterion 3: Reading tests. (a) A reading with `urgent_reason: Some("cuts retries that redo work")` gives `plan_entry(...).priority == Urgent`, and the reason is on the Reading. (b) A reading with None, for an ordinary request, is not urgent. (c) The parse path (the function around reading.rs:385) takes a ReadingOutput JSON with a reason and one with null, and checks the recorded `urgent_reason` is redacted-and-kept or None. A blank reason counts as None. (d) The schema from `reading_schema` lists `urgent_reason` as required, with the rule in its description (an outputs.rs test). Run: `cargo test --locked -p athena-core reading::`. Observes: The speed request comes out Urgent with its reason recorded, and the ordinary one does not.
  Fails without the behavior: If urgent_reason were not carried from the output into the record, or not used by plan_entry, test (a) would find a Normal priority.
- Criterion 4: Tests: `Priority::from_text("Priority: urgent") == Urgent` (the old assertion at plan.rs:644 expecting Normal is updated). The command's tests cover an unknown level, an unknown job id, and a non-queued job (a read job), each failing with a message and writing nothing. A valid call appends only a `priority-<n>.json`. A clap parse test or the existing help test checks that the help text names urgent and the command. Run: `cargo test --locked -p athena-core plan:: && cargo test --locked -p athena-cli`. Observes: from_text parses urgent. The error cases fail with clear messages. The help contains 'urgent' and 'priority'.
  Fails without the behavior: None of the error cases would be rejected without the validation, and from_text would still return Normal for 'urgent'.
- Criterion 5: A test in outputs.rs asserts that the generated reading schema's `urgent_reason` description contains the rule terms (faster, wasted time, failed or rejected attempts, planning stops, timeouts, retries, thrash, duplicated checks, reason). `git diff` shows reader.md unchanged, or only shorter if text was removed. Run: `git diff --stat`. Observes: The outputs.rs doc holds the rule. reader.md does not appear in the diff, or has fewer lines.
  Fails without the behavior: If the rule were missing from the field doc, the schema test would fail.
- Criterion 6: Run the existing budgets test. The reader set (prompt, defaults, and so on) must stay within 2000 tokens. reader.md is untouched. Run: `cargo test --locked -p athena-cli --test budgets`. Observes: every_role_set_is_within_its_budget passes. The reader count is at or below the budget.
  Fails without the behavior: If reader.md grew, the test would fail by exceeding the budget.
- Criterion 7: Inspect the new CHANGELOG.md entry (urgent level, priority command, reader urgent_reason, record kind). Inspect the new docs/decisions.md decision with its alternatives. Both are written in the prevailing house style. Also check docs/records.md lists `athena.priority/1`. Run: `git diff CHANGELOG.md docs/decisions.md docs/records.md`. Observes: The diff shows an entry in each file, and the decision lists the rejected alternatives.
- Criterion 8: Run the three checks Athena runs. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: Any missed fixture (a Reading literal lacking the new field) fails compilation and so the whole suite.

Planned files:

- crates/athena-core/src/plan.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/pr_check.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/priority.rs (create)
- docs/records.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-context/prompts/reader.md (modify)

## The change

Base 813c32d07e79 → candidate 6e044340e025. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   9 ++
 crates/athena-agent/src/fake.rs         |   3 +-
 crates/athena-cli/src/main.rs           |  34 +++++-
 crates/athena-cli/tests/common/mod.rs   |   3 +-
 crates/athena-cli/tests/drain.rs        |  97 ++++++++++++++++++
 crates/athena-context/src/briefs.rs     |   1 +
 crates/athena-context/src/outputs.rs    |  34 ++++++
 crates/athena-core/src/admission.rs     |   1 +
 crates/athena-core/src/plan.rs          |  61 +++++++++--
 crates/athena-core/src/pr_check.rs      |   1 +
 crates/athena-core/src/reading.rs       | 176 +++++++++++++++++++++++++++++++-
 crates/athena-core/src/run_pin_tests.rs |   3 +-
 crates/athena-core/src/status.rs        |   4 +-
 crates/athena-core/src/store.rs         |   9 +-
 crates/athena-data/src/records.rs       | 100 ++++++++++++++++++
 crates/athena-gate/src/landing_tests.rs |   1 +
 docs/decisions.md                       |  44 ++++++++
 docs/records.md                         |   1 +
 18 files changed, 560 insertions(+), 22 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1218 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
