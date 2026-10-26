# Job 20261026-104718-5y62: verify "Add post-mortem field, planning-stopped marker and retry findings"

## Request (verbatim)

> try again on 20261026-065228-02zf

## Reading (accepted by the user)

Retry of the failed job: add the post-mortem field ('context that would have prevented it', fixed outcome list) and planning-stopped marker to the introspection record, add the question to Jev's call, and carry the previous attempt's blocker and major findings into retry briefs in both seeded and unseeded paths. No 'resolved' state exists, so none is invented; the real parse_response must tolerate a missing answer; D130 is marked accepted.

Assumptions:
- Goal: complete job 4 of the context-engineer design so retries stop omitting prior blocker/major findings; audience is Athena's operator (Sri) and later producers reading retry briefs.
- Findings have no resolved state today (records.rs:750), so 'resolved ones left out' is dropped: all blocker and major findings of the previous attempt's final verification are carried. This supersedes the earlier acceptance bullet about resolved findings; the proposed retry request already says not to invent one.
- The real parse_response (jev.rs:147) fails the whole reply on any missing answer, so the new question is made optional there, with a test against the real parser.
- Seeded retry path keeps its full finding list but labels blocker and major ones, so both paths tell the producer what it must address.
- Files the design touches (reading.rs, crates/athena-cli/tests/introspection.rs, docs/records.md) are listed as deliverables, as the prior planning failures required.
- Symbols relied on were grepped: parse_response, tree_retry_seed, Finding (records.rs:750, introspect.rs:666), D130 (decisions.md:3272, status proposed).

## Acceptance criteria

1. [behavior] Run `cargo test`: a retry-brief test rejects a job with two major findings and a minor one, and asserts both major findings (summary, evidence, location) appear in the retry's producer brief in BOTH the seeded (tree_retry_seed) and unseeded paths, listed the same way; it fails if the builder drops one.
2. [behavior] Run `cargo test`: a test calls the real jev.rs `parse_response` (not FakeJev) on a reply with no answer to the new context-prevention question and asserts the class and repeat answers still come back and the new field is empty; a malformed or out-of-list answer is stored as an error in the field and blocks nothing.
3. [behavior] Run `cargo test`: a test through the introspection path shows a job failed with 'planning stopped before any code was written:' yields a record with the planning-stopped marker and a 'context that would have prevented it' outcome from the fixed list (system-facts-line, failure-class, brief-assembly, vocabulary-term, spec-fix, structure, none); rejected and otherwise-failed jobs yield the field, a landed job yields no record, and a failed Jev call leaves the job outcome unchanged.
4. [behavior] Run `cargo test`: a test shows Jev's introspection call text contains 'What one piece of context, if the agent had read it, would have prevented this?' and the fixed outcome list, and that a record written with the field reads back with it.
5. [content] No 'resolved' state is invented on Finding: every blocker and major finding of the previous attempt's final verification is carried, and CHANGELOG.md and docs/records.md state this plainly.
6. [content] docs/decisions.md D130 status reads accepted, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), noting job 1 (size and touch report) is covered by the code-health check (request 20261026-042558-5p65 and its retry).
7. [content] CHANGELOG.md has an entry for the post-mortem field, the planning-stopped marker and the retry findings change.
8. [behavior] cargo test, cargo clippy and cargo fmt --check pass for the workspace.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)

## Out of scope

- Inventing a resolved state or flag on Finding
- Build jobs 2, 3 and 5 of the context-engineer design
- Changing the code-health check or job 1's size and touch report

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each new behavior; cargo test, clippy and fmt --check in criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check briefs, records and the real parser behaving, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the omission in both retry paths, not one instance.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the listed files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs state the no-resolved-state choice for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in reading.rs tests: a job with two major findings (summary, evidence, location) and one minor, with locations in docs/ so tree_retry_seed qualifies. Build the producer brief through the seeded path (tree_retry_seed → RetrySeed → producer_brief) and the unseeded path (retry_findings → producer_brief). Assert both majors appear in each brief inside the 'must address' block, in identical lines, and the minor is absent from that block. Run: `cargo test`. Observes: Both briefs list both majors with summary, evidence and location in the same format; the minor is not in the must-address block.
  Fails without the behavior: If either builder dropped a major finding, or the unseeded brief carried none (current behaviour), the assertion fails.
- Criterion 2: Unit test in jev.rs tests calling the real parse_response with the three-question set and a body that answers only the class and repeat questions. Also cases for a boolean answer to the context choice question and an out-of-list choice, passed on to ask_jev through an Evaluate stub that returns the parsed result. Run: `cargo test`. Observes: Parse returns Ok with class and repeat answers; the context field is None. The wrong-typed and out-of-list answers give an error string in context_prevention and class and repeat are still stored; the Evaluation is not discarded.
  Fails without the behavior: With the old parse_response, the missing answer returns JevError::Malformed and the class and repeat answers are lost.
- Criterion 3: Tests in crates/athena-cli/tests/introspection.rs through run_job and athena.introspect_job: (a) a job failed by stop_planning (planning enabled, critic rejecting) with FakeJev answering a list item gives planning_stopped true and the outcome in context_prevention; (b) a rejected job and a failed job (rustfmt.toml edit) give the field and planning_stopped false; (c) a landed job gives no record; (d) a Jev Err leaves the job's outcome state unchanged and the field None. Run: `cargo test`. Observes: The record has the marker and a context outcome in the fixed list. Non-planning jobs have the field and no marker. No record for a landed job. The outcome state is unchanged when Jev errors.
  Fails without the behavior: No marker is set and no field exists; the assertions on planning_stopped and context_prevention fail.
- Criterion 4: Test in introspection.rs (and a unit test in introspect.rs): capture FakeJev calls; assert the question set contains the key whose instructions contain 'What one piece of context, if the agent had read it, would have prevented this?' and a Choice whose criteria keys are exactly the seven list items. Then write a record carrying the field, commit and reload it with AnyRecord::parse, and compare equal. Run: `cargo test`. Observes: The instruction text and all seven options are in the sent call. The reloaded record equals the written one and carries the field.
  Fails without the behavior: The call lacks the question, or deny_unknown_fields rejects the stored field on read.
- Criterion 5: Inspect the diff: no new field on Finding in crates/athena-data/src/records.rs, and the CHANGELOG.md and docs/records.md text says every blocker and major finding is carried and no resolved state exists. A test asserts both briefs carry a finding regardless of any later round. Run: `git diff`. Observes: Finding is unchanged; both docs state the choice plainly.
- Criterion 6: Inspect the D130 entry in docs/decisions.md. Run: `git diff`. Observes: Status accepted; decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26); notes job 1 is covered by the code-health check (request 20261026-042558-5p65 and its retry).
- Criterion 7: Inspect CHANGELOG.md Unreleased. Run: `git diff`. Observes: An entry names the post-mortem field, the planning-stopped marker and the retry findings change.
- Criterion 8: Run the workspace checks. Run: `cargo test`. Observes: cargo test, cargo clippy --all-targets -- -D warnings and cargo fmt --check all pass, including the budgets and boundary tests; no email-like text was added to the docs.
  Fails without the behavior: Compile errors from new struct fields, or fmt and clippy failures.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-core/src/labeled.rs (modify)
- crates/athena-cli/tests/review.rs (modify)
- crates/athena-cli/tests/labeled.rs (modify)
- crates/athena-cli/tests/proposals.rs (modify)
- docs/records.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base 180639cfbca3 → candidate 21ae39be0e47. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |  18 +++
 crates/athena-cli/tests/introspection.rs | 233 ++++++++++++++++++++++++++++++-
 crates/athena-cli/tests/labeled.rs       |   1 +
 crates/athena-cli/tests/proposals.rs     |   1 +
 crates/athena-cli/tests/review.rs        |   1 +
 crates/athena-context/src/briefs.rs      |  84 ++++++++++-
 crates/athena-core/src/introspect.rs     |  68 ++++++++-
 crates/athena-core/src/jev.rs            |  48 ++++++-
 crates/athena-core/src/labeled.rs        |   1 +
 crates/athena-core/src/proposals.rs      |   1 +
 crates/athena-core/src/reading.rs        | 205 +++++++++++++++++++++++----
 crates/athena-core/src/run.rs            |  24 +++-
 crates/athena-core/src/status.rs         |   1 +
 crates/athena-data/src/records.rs        |  50 +++++++
 docs/decisions.md                        |   9 +-
 docs/records.md                          |  17 +++
 16 files changed, 716 insertions(+), 46 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1322 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy and cargo fmt --check pass for the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
