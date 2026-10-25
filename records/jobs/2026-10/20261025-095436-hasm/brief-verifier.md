# Job 20261025-095436-hasm: verify "Add urgent priority and a priority-record command; guard decisions.md"

## Request (verbatim)

> try again on 20261025-034452-wqp8

## Reading (accepted by the user)

Retry of the rejected job: add Urgent above High in plan ordering, a CLI command appending a priority record that plan and status prefer over request text, and the reader's urgent-with-reason rule (in outputs.rs/briefs.rs, not reader.md). New decisions go at the end of docs/decisions.md, with a repo test guarding the log's structure so the D121/D122 damage cannot recur. Audience: the user and the verifier running cargo checks.

Assumptions:
- The retry starts from base; the earlier broken D121/D122 edit is absent, so the fix is to append a new decision after the last entry (currently D121) with its own Provenance.
- The introspection's decisions-log check is folded in as a small test, as the earlier failure came from it; the feature remains the goal.
- Fixture and loader files outside the listed ones (store, status, fake agent, test common, docs/records.md) are likely needed for the new record kind and field.
- The reader set is at 1997 of 2000 tokens, so the urgent rule goes in outputs.rs or briefs.rs net of removals; reader.md does not grow.
- Existing decision-links.toml uses line numbers, so only appending to decisions.md keeps links valid.

## Acceptance criteria

1. [behavior] Run cargo test: a plan test shows urgent before high, high before normal, with a dependency still ordered first even when the dependent is urgent.
2. [behavior] Run cargo test: a test shows a priority record for a queued request overrides the priority in its text; the order in plan and the priority in status change, and no new request record is written.
3. [behavior] Run cargo test: a reading test parses a ReadingOutput with an urgent reason and with null, and shows the reason reaches the recorded Reading; a speed-improving request is marked urgent with a reason and an ordinary one is not.
4. [content] The new CLI command appends a priority record, rejects unknown priority values and unknown or non-queued requests with a clear error; Priority::from_text parses 'Priority: urgent'; main.rs help mentions urgent and the command.
5. [content] The urgent-marking rule (makes Athena faster or cuts wasted time: failed or rejected attempts, planning stops, timeouts, redo-work retries, thrash, duplicated checks; reason required) is in outputs.rs or briefs.rs; reader.md does not grow.
6. [behavior] Run cargo test --locked -p athena-cli --test budgets: the reader set stays within 2000 tokens.
7. [behavior] Run cargo test: a new test parses docs/decisions.md into entries and checks each has a Provenance bullet, ids are unique and ascending, and against the base commit existing entries are byte-identical with additions only at the end; it passes on the final file.
8. [content] CHANGELOG.md has an entry; docs/decisions.md has a new decision appended after the last entry with alternatives and its own Provenance, and D121 is unchanged from base.
9. [behavior] cargo fmt --check, cargo clippy --all-targets and the full cargo test pass.

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
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/records.md (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-core/src/pr_check.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)

## Out of scope

- Rewriting or reordering existing decisions in docs/decisions.md
- Growing reader.md
- Changing governance files such as athena.toml or defaults.toml
- Rewriting request records

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for ordering, override, reading parse path, and decisions log structure; fmt, clippy, test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and check the ordering, override and reading behavior, not that types exist.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry with alternatives and its own Provenance, plus a CHANGELOG entry.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One feature; the log guard is a small test.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the named cargo commands.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The decisions-log structure rule is enforced by a repo test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Guards the append-only log structure rather than only fixing the one bad insertion.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in plan.rs build entries with Urgent, High, Normal and an urgent entry depending on a normal one; assert Plan::build order. Run: `cargo test --locked -p athena-core plan::tests`. Observes: Order is urgent, high, normal; the dependency precedes its urgent dependent.
  Fails without the behavior: Without Urgent sorting before High or the dependency rule, the ordering assertions fail.
- Criterion 2: Test: queued job with request `Priority: low` gets an urgent priority record via set_priority; load records, build plan and status line. Run: `cargo test --locked -p athena-core priority_record`. Observes: Plan puts job first, status shows urgent, request.json unchanged, exactly one new priority-1.json.
  Fails without the behavior: If the record were ignored, order and status stay at low; if a request were rewritten the file-count/bytes assertion fails.
- Criterion 3: Reading tests parse ReadingOutput with an urgent reason and with null, run build_reading; fake-agent test with a speed-improving and an ordinary request. Run: `cargo test --locked -p athena-core urgent_reason`. Observes: Some(reason) recorded for urgent and speed request, None for null and ordinary; plan priority urgent only for the first.
  Fails without the behavior: Without the field deny_unknown_fields rejects the output; without build_reading wiring the Reading lacks the reason; without the fake rule the speed request is unmarked.
- Criterion 4: Tests call set_priority with `bogus`, unknown job, non-queued job; assert errors and no commit; assert Priority::from_text("Priority: urgent")==Urgent; render help and check for 'urgent' and 'priority'. Run: `cargo test --locked -p athena-cli priority`. Observes: Clear errors, no record committed, urgent parsed, help mentions both words.
  Fails without the behavior: Without validation bad input commits a record; without the parse arm urgent stays Normal.
- Criterion 5: Test in outputs.rs generates the reading schema and checks the urgent_reason description contains the rule terms; git diff on reader.md shows nothing. Run: `cargo test --locked -p athena-context outputs`. Observes: Description contains faster, failed or rejected attempts, planning stops, timeouts, redo, thrash, duplicated checks, reason required.
  Fails without the behavior: If the rule were absent from the schema the description assertion fails.
- Criterion 6: Run the existing budgets test unchanged. Run: `cargo test --locked -p athena-cli --test budgets`. Observes: Reader set at or under 2000 tokens.
  Fails without the behavior: A rule added to reader.md would push the reader set over 2000 tokens and fail.
- Criterion 7: Test in decisions.rs parses docs/decisions.md into `## D<n> · ` entries, checks Provenance, unique ascending ids, and SHA-256 of the first 230046 bytes against the pinned base hash; negative cases on small strings. Run: `cargo test --locked -p athena-cli decisions`. Observes: All checks pass on the final file; negative cases return errors.
  Fails without the behavior: A mid-file insertion, duplicate or descending id, missing Provenance, or edit of the first 230046 bytes fails the guard.
- Criterion 8: Inspect the diff of the two docs. Run: `git diff -- docs/decisions.md CHANGELOG.md`. Observes: decisions.md diff is additions only after D121 (new D122 with alternatives and Provenance); CHANGELOG has a new entry; no removed lines.
- Criterion 9: Run the three gate commands; each must exit 0. The full test run includes every new test above, so it exercises all behaviors. Run: `cargo test --locked`. Observes: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked all exit 0.
  Fails without the behavior: If any new behavior were missing or broken, its new test in the full cargo test run fails; unformatted code fails fmt --check; warnings (e.g. an unused or unhandled new Priority variant or unupdated Reading literal) fail clippy or compilation.

Planned files:

- crates/athena-core/src/plan.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/pr_check.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## The change

Base 0bec0de70d93 → candidate 6a4d1a43e8aa. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   8 ++
 crates/athena-agent/src/fake.rs         |  17 +++
 crates/athena-cli/src/decisions.rs      | 124 ++++++++++++++++++++++
 crates/athena-cli/src/main.rs           |  25 ++++-
 crates/athena-cli/tests/common/mod.rs   |   1 +
 crates/athena-cli/tests/drain.rs        | 182 +++++++++++++++++++++++++++++++-
 crates/athena-context/src/briefs.rs     |   1 +
 crates/athena-context/src/outputs.rs    |  63 +++++++++++
 crates/athena-core/src/admission.rs     |   1 +
 crates/athena-core/src/plan.rs          |  57 ++++++++--
 crates/athena-core/src/pr_check.rs      |   1 +
 crates/athena-core/src/reading.rs       |  52 ++++++++-
 crates/athena-core/src/status.rs        |   7 +-
 crates/athena-core/src/store.rs         |  35 +++++-
 crates/athena-data/src/records.rs       |  43 ++++++++
 crates/athena-gate/src/landing_tests.rs |   1 +
 docs/decisions.md                       |  47 +++++++++
 docs/records.md                         |   4 +-
 18 files changed, 645 insertions(+), 24 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1217 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
