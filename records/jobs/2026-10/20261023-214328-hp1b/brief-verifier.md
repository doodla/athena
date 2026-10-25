# Job 20261023-214328-hp1b: verify "Add mistake category with per-class and per-cause counts"

## Request (verbatim)

> try again on 20261023-183334-aej6

## Reading (accepted by the user)

Retry of job 20261023-183334-aej6: add a `mistake` intervention category. Status, the audit and the JSON count mistakes per class and per cause, apart from interventions_by_category. Introspecting a mistake proposes the preventing check or knowledge. This time a test must read the audit output, and the audit must print mistakes even when no job audit is produced. The audience is Sri.

Assumptions:
- The request as typed is 'try again' on the original mistakes job, so the original request is the goal. The pre-handover evidence check proposed by the introspection is a separate pipeline change and is not built here.
- The producer maps every criterion to a named test before finishing; this is the lesson of the earlier rejection, kept as a working practice and not as a new mechanism.
- Mistakes are interventions with category `mistake`, plus legacy interventions with category `other` whose text starts 'Mistake:'. Existing records are not rewritten.
- The previous attempt's file list is the starting point. It adds crates/athena-cli/tests/auditor.rs for the missing audit test and docs/design/athena-v0.md, which lists the category set.
- The audit's mistake report must not depend on a job audit being produced, so it prints before the early return in Command::Audit in main.rs.

## Acceptance criteria

1. [behavior] Run `cargo test`: new tests show an intervention logged with category `mistake` is accepted by the parser and appears in `InterventionCategory::ALL`, and an unknown category is still rejected with an error listing `mistake`.
2. [behavior] Run `cargo test`: a test with several mistake interventions of different classes and causes shows `athena status` output and its JSON report mistake counts per class and per cause, in fields separate from `interventions_by_category`.
3. [behavior] Run `cargo test`: a CLI test in crates/athena-cli/tests (auditor.rs or introspection.rs) runs the audit command and asserts its output reports mistakes per class and per cause. A second case asserts the mistake counts still print when no job audit is produced.
4. [behavior] Run `cargo test`: a test shows that introspecting a `mistake` intervention yields a proposal naming the check or knowledge that would have prevented it, and that mistakes with the same cause aggregate into one proposal.
5. [content] Existing records with category `other` and text starting 'Mistake:' are counted as mistakes in status and the audit. Records are not rewritten. The text-prefix rule is documented in a code comment or doc.
6. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean, and `cargo test` passes in full.
7. [content] Outcome-check: a reader of the status or audit output can see how many mistakes occurred, which classes and causes recur, and which prevention work was proposed. README.md, CHANGELOG.md and the category list in docs/design/athena-v0.md mention the `mistake` category.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/auditor.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/athena-v0.md (modify)

## Out of scope

- Building the pre-handover evidence gate in the job pipeline, which the introspection proposed as a separate next request
- Rewriting or migrating existing intervention records
- Changing athena.toml, defaults.toml or workflow files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Every new behavior has a named test; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check counts appearing in status and audit output, not just the new type.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires a reader to see mistake counts, recurring causes and proposed prevention.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The pipeline evidence gate is left out and the change stays on the mistake category.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust and Markdown.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests records.rs `mistake_is_a_category_and_in_all`, `parse_accepts_mistake`; manage.rs `intervene_accepts_mistake_and_rejects_unknown_category_listing_mistake`. Run: `cargo test`. Observes: ALL contains Mistake, parse("mistake") is Some, intervene commits category Mistake, and a bogus category errors with text listing mistake.
  Fails without the behavior: Without the Mistake variant parse returns None, intervene bails on "mistake", and the ALL assertion fails.
- Criterion 2: lifecycle.rs `status_reports_mistakes_per_class_and_per_cause_apart_from_categories`: 3 mistakes (2 docs, 1 code), two sharing a cause by normalisation; asserts StatusView.mistakes, display::status text, and `athena status --json` by_class/by_cause with interventions_by_category holding only "mistake": 3. Run: `cargo test`. Observes: Per-class and per-cause counts in text and JSON, in fields separate from interventions_by_category.
  Fails without the behavior: Without the mistakes field there is no "mistakes" JSON key and no status text line; assertions fail.
- Criterion 3: auditor.rs `the_audit_output_reports_mistakes_per_class_and_per_cause` (audit Some) and `the_audit_output_reports_mistakes_when_no_audit_is_produced` (nested, None); introspection.rs `athena_audit_prints_mistakes_before_any_audit` runs the binary `athena audit` with ATHENA_NESTED (fallback job-less repo where the audit errors) and asserts stdout counts. Run: `cargo test`. Observes: Audit output lists total, per-class and per-cause counts with an audit, with none, and when it errors.
  Fails without the behavior: Without printing before the early return, the None and binary cases emit no mistake lines and fail.
- Criterion 4: introspection.rs `a_mistake_intervention_is_introspected_with_guidance_for_the_preventing_check_or_knowledge` (captured prompt has mistake_guidance; non-mistake prompts lack it) and `mistakes_with_the_same_cause_aggregate_into_one_proposal` (2 proposals, shared one covers 2 mistakes); status.rs kind-mapping unit test. Run: `cargo test`. Observes: Prompt asks for the preventing check or knowledge; proposal names kind and text; same-cause mistakes give one proposal.
  Fails without the behavior: Without the guidance key the prompt assertion fails; without cause grouping there are 3 proposals, not 2.
- Criterion 5: status.rs `a_legacy_other_record_starting_mistake_colon_is_counted_and_not_rewritten`, records.rs `is_mistake_rule`; lifecycle check that the stored record file is byte-identical and still shows under "other"; the rule is in the is_mistake doc comment. Run: `cargo test`. Observes: Legacy 'Mistake:' other records are counted in status and audit; 'A mistake:' is not; file unchanged.
  Fails without the behavior: Without the prefix rule the legacy record is missing from totals and assertions fail.
- Criterion 6: Run the three gates on the finished change. The new tests (criteria 1-5) are part of the suite, and the new enum variant and Option-taking display functions must compile everywhere they are used. Run: `cargo clippy --all-targets`. Observes: clippy and fmt print no diagnostics (cargo fmt --check), and cargo test passes in full, including the new named tests and existing tests that use InterventionCategory.
  Fails without the behavior: If the variant were unhandled or the new code malformed, the build, a clippy warning, a fmt diff or a failing new or existing test would fail the gates; a missing criterion test would show as absent from the cargo test output.
- Criterion 7: Status and audit tests assert rendered text shows the mistake total, classes and causes with counts above 1, and a 'prevention' line per proposal (kind and text). A lifecycle.rs test reads README.md, CHANGELOG.md and docs/design/athena-v0.md via include_str! and asserts each contains "mistake". The fresh-reader rating is prose-only, done by the reviewer. Run: `cargo test`. Observes: Output shows how many mistakes, recurring classes and causes, and proposed prevention; the three docs name the category.
  Fails without the behavior: Without the docs edits the include_str! assertions fail; without proposal lines the output assertions fail.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/auditor.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/athena-v0.md (modify)

## The change

Base 46c01ad2c9b3 → candidate 9d3f44dfce2d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   7 +
 README.md                                |   4 +-
 crates/athena-cli/src/display.rs         |  45 ++++++
 crates/athena-cli/src/main.rs            |  24 ++-
 crates/athena-cli/tests/auditor.rs       |  60 ++++++++
 crates/athena-cli/tests/introspection.rs | 152 +++++++++++++++++++
 crates/athena-cli/tests/lifecycle.rs     | 136 +++++++++++++++++
 crates/athena-core/src/introspect.rs     |  15 +-
 crates/athena-core/src/status.rs         | 252 +++++++++++++++++++++++++++++++
 crates/athena-data/src/records.rs        |  55 ++++++-
 docs/design/athena-v0.md                 |   2 +-
 11 files changed, 741 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (994 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
