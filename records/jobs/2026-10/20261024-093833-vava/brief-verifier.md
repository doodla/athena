# Job 20261024-093833-vava: verify "Make the first producer report list every criterion with evidence"

## Request (verbatim)

> Most producers are sent back once because their first report does not account for every acceptance criterion: jobs 20261024-025358-kb23, -033758-c2h3, -045548-7qht, -055549-0qcy, -033935-c4w4 and -082859-skh5 each had a send-back, and the second producer run cost 9 to 17 minutes, often for a report that only restates what was already done. Find why the first report misses criteria (the producer prompt, the report schema, or how criteria are shown), make the first report list every criterion with its evidence, and measure the send-back rate before and after on the jobs' records.

## Reading (accepted by the user)

Goal: stop the producer's first report from missing acceptance criteria, which costs the user a 9–17 minute second run. Fix the cause in the change producer prompt and the rendered brief (not the budget-tight shared prompt), then measure the send-back rate before and after from job records.

Assumptions:
- Audience: the user, who pays for the send-back, and the verifier who judges the change.
- Read in this copy: run.rs unmet_criteria already ignores numbering, kind tag, case, spacing and trailing punctuation, but any other wording difference counts as unmet. A paraphrased or merged entry therefore triggers a send-back, and the send-back message may only restate what was done.
- Read in this copy: the shared producer.md has one line on criteria_check, and the CriterionCheckOutput schema asks for the text 'without its number or kind tag'. The producer must retype criteria from memory, so wording drifts. Rendering the criteria so they can be copied verbatim is the likely fix.
- Records of the named jobs are absent from my copy, so the cause is not yet measured. The producer must read the records, if it can, to confirm the cause and compute the before rate for the six jobs. If it cannot read them, it names that gap and does not invent numbers.
- The after rate can only come from jobs run after the change. The change reports this honestly as a small sample or as not yet measurable, and gives a repeatable way to compute it.
- Per the user's reply, new producer instructions go in crates/athena-cap-change/prompts/producer.md or the rendered brief in briefs.rs, net of removals. The shared producer.md is not grown (study at 1276/1300). The change verifier prompt is not touched (1298/1300).
- The design call and alternatives (prompt only, schema only, tolerant matching) are recorded in docs/decisions.md.

## Acceptance criteria

1. [content] docs/decisions.md has a note naming the cause of the missed criteria. It cites crates/athena-core/src/run.rs (unmet_criteria, criterion_key), the criteria rendering in crates/athena-context/src/briefs.rs, the producer prompts, and the CriterionCheckOutput docs in outputs.rs. It says which of prompt, schema and display contributed and lists the alternatives considered (prompt only, schema only, tolerant matching).
2. [content] The note states the send-back rate before the change for kb23, c2h3, 7qht, 0qcy, c4w4 and skh5, with the records read to get it. Each number is marked measured or estimate, with its sample size. Any unreadable record is named, and the after rate is stated for jobs run after the change, or the note says there are none yet.
3. [behavior] Run `cargo test -p athena-context`: a test shows the rendered producer brief lists every accepted criterion verbatim, with a copyable form (no number or kind tag needed to match), and instructs one criteria_check entry per criterion, in order, with evidence, including criteria already satisfied.
4. [content] New producer instructions are in crates/athena-cap-change/prompts/producer.md or the rendered brief. Shared crates/athena-context/prompts/producer.md and crates/athena-cap-change/prompts/verifier.md are not made longer, and `cargo test --locked -p athena-cli --test budgets -- --nocapture` passes.
5. [behavior] Run `cargo test -p athena-core`: a test shows a first report with a met entry for every criterion is not sent back, and a report omitting one criterion is sent back once, with the send-back naming the missing criterion's exact text.
6. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass.

## Expected deliverables

- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- docs/decisions.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding to the change verifier prompt (at 1298 of 1300 tokens)
- Growing the shared producer.md beyond its budget
- Changing the verifier or planner behavior
- Editing records/ or the named jobs' outputs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests in athena-context and athena-core; cargo test and clippy must be clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The finding cites file locations; the rates are marked measured or estimate, with sample size.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria check that the brief and the send-back logic behave as intended, not that text exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The finding names which of prompt, schema or display causes the misses before the fix.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The before and after send-back rates are the outcome criterion.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision note is written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the brief, the change producer prompt and, if needed, the field docs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the note diff. Run: `git diff -- docs/decisions.md`. Observes: The note names display (numbered, tagged list that cannot be copied), the shared prompt line and the 'without its number or kind tag' field doc as contributors. It cites run.rs unmet_criteria and criterion_key, the criteria rendering in briefs.rs, both producer prompts and outputs.rs CriterionCheckOutput. It lists prompt only, schema only and tolerant matching as alternatives.
- Criterion 2: Read the rates section of the diff. Run: `git diff -- docs/decisions.md`. Observes: The note gives a before entry for each of kb23, c2h3, 7qht, 0qcy, c4w4 and skh5. Each is marked unmeasured (the producer sandbox cannot read records/, briefs.rs:224-225) with sample size 0 of 6 readable, and all six records are named as unreadable, unless the brief supplied them. The note gives the repeatable procedure: per job, count producer invocation records under records/<job>/invocations/ whose detail contains 'self-check failed', where more than one producer invocation means a send-back. The note says there are no jobs since the change, so the after rate is not yet measurable.
- Criterion 3: New briefs.rs unit test builds a producer brief from a reading with several criteria. It asserts each criterion text appears as its own bare line, with no number or kind tag, in order. It asserts the brief instructs one criteria_check entry per criterion, in order, with evidence, including already-satisfied ones. It asserts the planner brief lacks the section. Run: `cargo test -p athena-context`. Observes: The new test passes.
  Fails without the behavior: The old brief has only '1. [kind] text' lines and no instruction. The bare-line assertion and the one-entry-per-criterion assertion would both fail.
- Criterion 4: Run the budgets test and inspect the prompt diffs. Run: `cargo test --locked -p athena-cli --test budgets -- --nocapture`. Observes: The test passes. The shared producer.md diff is a net shortening. The change verifier.md has no diff. New instructions are only in the change producer.md and the rendered brief.
  Fails without the behavior: If the shared producer.md or the change verifier.md grew past its token budget, the budgets test would fail.
- Criterion 5: Three tests. (a) A unit test in run.rs: send_back_text with an unmet list contains the omitted criterion's exact text and the copy-the-line sentence, and not 'without its number'. (b) A full-run test in run_pin_tests.rs where the first producer report omits one criterion: two producer invocations, and the second brief contains the omitted text. (c) A full-run test where the report has a met entry for every criterion: one producer invocation. Run: `cargo test -p athena-core`. Observes: All three pass: the complete report is not sent back, and the incomplete one is sent back once with the exact text named.
  Fails without the behavior: If a missing entry were ignored, (b) would see one invocation. If a complete report were sent back, (c) would see two. If the send-back dropped the criterion text, (a) and (b) would fail.
- Criterion 6: Run the three checks in turn. Run: `cargo test`. Observes: cargo test, cargo clippy --all-targets and cargo fmt --check each exit 0, with no warnings and no formatting diff, including the new tests.
  Fails without the behavior: A failing new test, a clippy warning or unformatted code makes the matching command fail.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base 7ffdd0e9691a → candidate 950d58734cba. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |  7 +++
 crates/athena-cap-change/prompts/producer.md |  2 +-
 crates/athena-context/prompts/producer.md    |  2 +-
 crates/athena-context/src/briefs.rs          | 62 +++++++++++++++++++++++++
 crates/athena-context/src/outputs.rs         |  2 +-
 crates/athena-core/src/run.rs                | 34 ++++++++------
 crates/athena-core/src/run_pin_tests.rs      | 67 ++++++++++++++++++++++++++++
 docs/decisions.md                            | 46 +++++++++++++++++++
 8 files changed, 205 insertions(+), 17 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1058 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
