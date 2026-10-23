# Job 20261023-183334-aej6: verify "Track operator mistakes as their own intervention category"

## Request (verbatim)

> Sri (BRIEF.md addendum of 2026-10-23, 'Track every mistake'): 'Start tracking every mistake you make as work Athena should start being intelligent about as well'. Interventions whose text starts 'Mistake:' are mistakes of the role doing Athena's thinking for now (eleven back-filled on 2026-10-23, category other). Give them a category of their own (mistake) in the closed list, have status and the audit count mistakes per class and per cause, and have introspection of a mistake propose the check or knowledge that would have prevented it, so repeat causes become work.

## Reading (accepted by the user)

Add a `mistake` variant to the closed intervention category list. Status and the audit then count mistakes per class and per cause. Introspection of a mistake proposes the check or knowledge that would have prevented it. The audience is Sri, who wants repeat mistake causes to become Athena work.

Assumptions:
- The category list is the Rust enum InterventionCategory in crates/athena-data/src/records.rs, with parse and ALL.
- 'Class' means class_of_work of the affected job. 'Cause' is derived from the mistake record's text or an existing introspection field. If no cause field exists, the producer picks the smallest such field.
- Interventions with text starting 'Mistake:' but category `other` count as mistakes, so the back-filled eleven are included. Records stay immutable.
- No change to athena.toml or defaults.toml is needed.

## Acceptance criteria

1. [behavior] Run `cargo test`: new tests pass. They show that an intervention logged with category `mistake` is accepted by the parser and `InterventionCategory::ALL`, and that an unknown category is still rejected with the list including `mistake`.
2. [behavior] Run `cargo test`: a test with several mistake interventions of different classes and causes shows that status and the audit output report mistake counts per class and per cause, and that the JSON has those breakdowns separate from `interventions_by_category`.
3. [behavior] Run `cargo test`: a test shows that introspecting a `mistake` intervention produces a proposal naming the check or knowledge that would have prevented it. Repeat causes aggregate into one proposal.
4. [content] Existing records with category `other` and text starting 'Mistake:' (the eleven back-filled on 2026-10-23) are counted as mistakes in status and the audit. Records are not rewritten. The text-prefix rule is documented in a code comment or doc.
5. [behavior] `cargo clippy` is clean and `cargo fmt --check` passes.
6. [content] Outcome-check: a reader of the status or audit output can see how many mistakes occurred, which classes and causes recur, and which prevention work was proposed. Output is not only a type or field addition.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- README.md (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Rewriting or re-categorizing existing records
- Automatically implementing the proposed prevention checks
- Changing athena.toml, defaults.toml, or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the category, the counts, and the proposal; clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check counts and proposals in output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criterion checks that mistakes are visible and become work.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change on the existing intervention pipeline.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Prefix-marked records are counted too, not only new ones.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in records.rs (parse mistake, ALL has it, length 9) and lifecycle.rs tests: intervene with category mistake succeeds; category bogus fails with an error listing mistake. Run: `cargo test`. Observes: New tests pass: mistake accepted by parser and ALL; bogus rejected with a message that lists mistake.
  Fails without the behavior: Without the Mistake variant parse returns None, intervene is refused, and the rejection list omits mistake, so the tests fail.
- Criterion 2: lifecycle.rs and status.rs tests log several mistakes (one category mistake, one other with 'Mistake:' text) across two classes and two causes; read status --json and audit output with a stub model; assert mistakes.total, by_class, by_cause, and that interventions_by_category is separate and unchanged. Run: `cargo test`. Observes: Per-class and per-cause counts appear in status JSON, status text and audit output.
  Fails without the behavior: Without the report the mistakes key is missing and the lookups fail.
- Criterion 3: introspection.rs test with stub model: introspect a mistake, assert stored record has prevention_kind and non-empty prevention and proposed_request, and the prompt contains the mistake instruction; two mistakes sharing a failure class yield one cause entry with count 2 and one proposal. Run: `cargo test`. Observes: A mistake introspection names the preventing check or knowledge; repeat causes appear as one entry with one proposal.
  Fails without the behavior: Without mistake handling the prompt lacks the instruction and category stays other; without aggregation two entries appear.
- Criterion 4: Test seeds a category-other intervention with 'Mistake:' text, asserts status JSON and audit count it, and compares record file bytes before and after. The is_mistake doc comment states the prefix rule. Run: `cargo test`. Observes: The prefix-marked record is counted as a mistake and its file is byte-identical.
  Fails without the behavior: A category-only check counts zero for back-filled records and the test fails.
- Criterion 5: Run the formatter check and the lint check as separate whole commands; each exits nonzero on any diagnostic. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Clippy exits 0 with no warnings; run `cargo fmt --check` as well, which exits 0 with no diff.
  Fails without the behavior: If the new code were unformatted, `cargo fmt --check` would print a diff and exit nonzero; if it had an unused variant, non-exhaustive handling, needless clones or similar, clippy with -D warnings would exit nonzero.
- Criterion 6: Tests for criteria 2 and 3 assert on rendered status and audit text: total mistakes, per-class and per-cause counts, recurring causes marked, a line per cause with prevention kind and text, or 'not yet introspected' with the introspect command. Run: `cargo test`. Observes: Rendered output shows how many mistakes occurred, which classes and causes recur, and the proposed prevention.
  Fails without the behavior: A type-only change leaves rendered output unchanged and these assertions fail.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)

## The change

Base d2b4f4793315 → candidate ebf86e0ad91c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   8 ++
 README.md                                |   2 +-
 crates/athena-cli/src/display.rs         |  37 +++++++++
 crates/athena-cli/src/main.rs            |   2 +
 crates/athena-cli/tests/introspection.rs | 135 +++++++++++++++++++++++++++++++
 crates/athena-cli/tests/lifecycle.rs     |  30 +++++++
 crates/athena-core/src/introspect.rs     |  19 ++++-
 crates/athena-core/src/status.rs         |  88 ++++++++++++++++++++
 crates/athena-data/src/records.rs        |  52 +++++++++++-
 docs/design/athena-v0.md                 |   2 +-
 10 files changed, 370 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (983 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
