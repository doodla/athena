# Job 20261025-171957-p5ns: verify "Derive evidence-based class labels for Jev scoring"

## Request (verbatim)

> Priority: high
> Label Jev's class answers from evidence, not opinion (BRIEF.md addendum of 2026-10-25, 'Evidence should label jev imo'). athena score-jev needs labeled introspections (at least 10 in its test part) and has none: 187 introspections carry a recorded Jev shadow answer, and the only labeling path today is a person's athena review. Derive labels from what happened afterwards, each with the records it rests on: (1) the retry that followed the introspection's proposed request and prevention landed, or failed for the same cause; (2) after a prevention for the class landed, the class stopped or kept recurring; (3) the critic's or verifier's findings on the same job name the same cause. Write each derived label as a class-label record marked evidence-derived with its basis, leave cases with no evidence unlabeled, and let athena review still override. Then score Jev with score-jev.
> Acceptance:
> - A test with fixture records shows each of the three evidence kinds producing a label with its basis, and a case with no evidence left unlabeled.
> - Labels are records (append-only); a person's review label takes precedence.
> - Run on this repository's records, score-jev reports how many labels were derived and, if enough, a score and verdict.
> Files: crates/athena-core/src/introspect.rs, crates/athena-core/src/jev.rs, crates/athena-data/src/records.rs, crates/athena-cli/src/main.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Add an evidence-derived labeling path that writes append-only class-label records (labeler "evidence", with a basis citing record ids) from three kinds of later evidence: retry outcome after the proposed request/prevention, class recurrence after prevention, and critic/verifier findings naming the same cause. Cases without evidence stay unlabeled, a person's review label always wins, and score-jev reports the derived count and a score and verdict if at least 10 test rows exist. The audience is the maintainer running athena score-jev.

Assumptions:
- Labeler gains an evidence-derived variant and ClassLabel gains an optional basis field (list of cited record ids with the evidence kind), backward compatible with existing records.
- Derivation runs from a CLI step (a flag on athena score-jev or a subcommand) that appends labels not already derived, so reruns are idempotent.
- Precedence is implemented in newest_labels/build: any person or stronger-model label beats an evidence label regardless of time.
- Where evidence kinds conflict, the case is left unlabeled rather than guessed.
- Derived labels are real records written to records/ on this repository's run; the score-jev result is reported honestly even if under 10 test rows (no verdict then).
- Derivation logic may live in a new module in athena-core if introspect.rs/jev.rs would become too large.

## Acceptance criteria

1. [behavior] Run cargo test: a fixture-record test shows each of the three evidence kinds (retry outcome, recurrence after prevention, critic/verifier same-cause findings) yielding a class-label with evidence-derived labeler and a basis citing the record ids it rests on.
2. [behavior] Run cargo test: a fixture introspection with no evidence gets no label and counts as unlabeled.
3. [behavior] Run cargo test: when both a person label and an evidence label exist for one introspection, the person label is used by the labeled set, whichever is newer; and derivation never edits or deletes an existing record, and rerunning does not duplicate labels.
4. [behavior] Run cargo run -- score-jev on this repository's records: output states how many labels were derived from evidence (and per evidence kind) and, if the test part has at least 10 rows, a score and verdict; otherwise states the shortfall with no verdict.
5. [content] ClassLabel schema and docs/records.md-relevant schema tests accept old labels without a basis and round-trip new ones; CHANGELOG.md and docs/decisions.md record the evidence-derived labeling call and its alternatives.
6. [behavior] cargo clippy and cargo fmt --check are clean.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/labeled.rs (modify)
- crates/athena-core/src/jev.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/evidence_labels.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/review.rs (modify)
- crates/athena-cli/tests/labeled.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/records.md (modify)

## Out of scope

- Changing Jev's prompt or model
- Editing or deleting existing records
- Changing the MIN_TEST_CASES threshold or scoring formulas
- Changing athena review's own flow beyond precedence

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-record tests per evidence kind, plus cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check labels produced with basis, and precedence, by running tests and score-jev.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Zero derived labels is reported as such, never as a passing score.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Run score-jev on the repository's records.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and decisions entry written for outside readers.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Labels come from outcomes, not from Jev's own answers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base ca2ad279345a → candidate ee38c3f3efef. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |   13 +
 README.md                                 |    2 +-
 crates/athena-cli/src/display.rs          |   30 +
 crates/athena-cli/src/main.rs             |   21 +-
 crates/athena-cli/tests/labeled.rs        |  169 ++++-
 crates/athena-core/src/evidence_labels.rs | 1082 +++++++++++++++++++++++++++++
 crates/athena-core/src/labeled.rs         |  138 +++-
 crates/athena-core/src/lib.rs             |    5 +
 crates/athena-core/src/reading.rs         |    2 +-
 crates/athena-core/src/review.rs          |    5 +-
 crates/athena-data/src/records.rs         |  140 +++-
 docs/decisions.md                         |   49 ++
 docs/design/defaults-loop.md              |    7 +-
 docs/records.md                           |    2 +-
 14 files changed, 1638 insertions(+), 27 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1394 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo clippy and cargo fmt --check are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
