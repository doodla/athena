# Job 20261014-005808-h792: verify "Record first decision-sets backtest and revise its design doc"

## Request (verbatim)

> Record the result of the first decision-sets backtest and revise docs/design/decision-sets.md from it. I ran 'athena backtest' on the real records: fail. (a) Supersession parse: 24 hits, 0 lacking a target, plus one hit the review's pass list did not expect: D11 'Revised ... superseded by D39' at line 322. It is a real, partial supersession, so the expected list was incomplete; decide whether partial revisions are 'amended_by' or 'superseded'. (b) The D87 signal (intervention did-work-themselves, class docs, threshold 2 jobs) found 0 jobs. The real interventions are class code: 20261013-192156-ar1x (job 9h2j, the hand squash of PR #41 after GitHub refused a merge commit with HTTP 405) is the only one after D87 was recorded; kn3q and qk2d predate it. Findings to design for: a host error that falsifies a decision's stated assumption must put it under review on the first occurrence, while soft signals such as friction need repeats; host errors need their own record (review F3), since today the 405 exists only in intervention prose; signal field values must be validated against the record vocabulary so a signal cannot name a class nobody records. Also add a signal from the BRIEF.md addendum '2026-10-14: Common sense over rigid rules': a rule that rejects work which does not go against the rule's stated intent is a failure signal for the rule. Write the backtest result into the design with the real ids. Design only.

## Reading (accepted by the user)

Design-only edit of docs/design/decision-sets.md: add a backtest-result section with real ids (D11/D39, D87, job 9h2j, 405), decide partial revisions, and add new signals and validation. Assumes partial revisions are 'amended_by', not 'superseded'.

Assumptions:
- Partial revisions are recorded as 'amended_by', since the original decision still stands in part.
- The BRIEF.md addendum exists as quoted and can be cited.
- Real ids come from the user's request; records/ is not readable here, so the producer must not invent other details.
- Design only: no implementation of the host-error record or validation.

## Acceptance criteria

1. [content] The design has a backtest-result section stating the run failed, with the supersession parse (24 hits, 0 lacking a target, plus the unexpected D11 'Revised ... superseded by D39' at line 322) and the D87 signal finding (0 jobs for class docs, threshold 2).
2. [content] The doc decides partial revisions: they are 'amended_by', not 'superseded', with the reason and how the expected list is corrected to include D11/D39.
3. [content] The doc names the real interventions: job 20261013-192156-ar1x (9h2j), the hand squash of PR #41 after the HTTP 405, class code, the only one after D87; kn3q and qk2d predate it. D87's signal is revised to class code.
4. [content] The doc specifies that a host error falsifying a decision's stated assumption puts it under review on first occurrence, while soft signals such as friction require repeats.
5. [content] The doc specifies a host-error record (review F3) so the 405 is no longer only in intervention prose.
6. [content] The doc requires signal field values to be validated against the record vocabulary so a signal cannot name an unrecorded class.
7. [content] The doc adds a signal from the BRIEF.md addendum '2026-10-14: Common sense over rigid rules': a rule rejecting work that does not go against its stated intent is a failure signal for that rule, citing the addendum.
8. [content] Only docs (and optionally CHANGELOG) change; no code, config, or records files are touched.

## Expected deliverables

- docs/design/decision-sets.md (modify)
- CHANGELOG.md (modify)
- docs/design/reviews/decision-sets-review-1.md (modify)

## Out of scope

- Implementing the host-error record, signal validation, or new signals in code
- Changing the backtest command or records
- Editing decisions.md or records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Backtest numbers and ids are cited from the user's run and the design's line references.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused doc revision.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Section written so an outsider can follow the ids.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Design addresses the signal-vocabulary gap, not just the D87 instance.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base fafb5d5911d2 → candidate 442a6b677169. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                  |   6 ++
 docs/design/decision-sets.md                  | 105 +++++++++++++++++++++++++-
 docs/design/reviews/decision-sets-review-1.md |  12 +++
 3 files changed, 119 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (394 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
