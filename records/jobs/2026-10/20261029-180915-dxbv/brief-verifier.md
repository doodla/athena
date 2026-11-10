# Job 20261029-180915-dxbv: verify "Track mistake classes before and after improvements (OLY-1-07)"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-1-07 a mechanism
> 
> Requirement OLY-1-07 in docs/spec.toml is active and has no mechanism.
> Requirement: Olympus measures whether a problem class becomes less frequent after an improvement, and keeps tracking comparable failures over time.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-1-07
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-1-07 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Extend the mistake report so each class of work shows its mistake count before and after the improvement recorded for it (a recorded introspection with a prevention), plus the trend over time. Print it in `athena status` and record that as OLY-1-07's mechanism in docs/spec.toml. Audience: the Athena owner judging whether improvements cut a problem class.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (mistake_report, MistakeReport, MistakeProposal at ~l.521-602; Athena::mistakes l.1051), crates/athena-cli/src/display.rs (mistakes() l.469; status printing l.1321 only when total > 0), crates/athena-cli/src/commands.rs l.161-175. Today mistakes are counted per class and per cause with no time split, so no before/after comparison exists.
- Existing code: extends crates/athena-core/src/status.rs mistake_report and crates/athena-cli/src/display.rs mistakes(); reuses introspections from records (status Recorded, intervention link, prevention) and intervention timestamps (InterventionRow.at in crates/athena-core/src/introspect.rs). No new record or data file is added, so check_record_file/AnyRecord are not touched.
- Assumed definition: an improvement for a class is the earliest Recorded introspection with a prevention linked to a mistake of that class; mistakes of that class are counted before and after its time. Comparable failures are mistakes of the same class_of_work, tracked per UTC month.
- Measure line ends `status: mistakes`, which is the label of the existing `mistakes ...` line athena status prints; the new before/after text is printed in that same section.
- No refusal rule is added; this is reporting only. Governance files (athena.toml, defaults.toml, workflows) are not needed.
- Test placement: unit test in crates/athena-core/src/status.rs tests module building Records with interventions and introspections; optional CLI display test in display.rs tests.

## Acceptance criteria

1. [content] docs/spec.toml OLY-1-07 has mechanism naming crates/athena-core/src/status.rs (the before/after trend function) instead of "none", owner kept as Athena, and a measure ending in `status: mistakes`.
2. [behavior] Run `cargo test -p athena-core status`: a new unit test builds records where class X has mistakes before and after a recorded introspection, and asserts the report gives the correct before and after counts for X and a per-month series; the test fails when the trend function is stubbed out or removed.
3. [behavior] Run `cargo test -p athena-cli`: a display test shows the mistakes section of athena status prints a line per class with its before/after counts when an improvement exists, and omits it otherwise (no empty-success: a class with no improvement is reported as 'no improvement recorded', not as improved).
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
5. [content] The existing mistake_report counts (total, by_class, by_cause, proposals) are unchanged, shown by existing tests in status.rs still passing without edits to their expectations.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how interventions or introspections are recorded or their schema
- Automatically queuing requests from a failed improvement
- Edits to athena.toml, defaults.toml or CI workflows
- Other requirements in docs/spec.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Added unit and display tests; fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and check counts, not file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused extension of the existing mistake report.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 17aaaae6b1ab → candidate 391df57eb7b8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   7 +++
 crates/athena-cli/src/display.rs |  53 +++++++++++++++++
 crates/athena-core/src/status.rs | 120 ++++++++++++++++++++++++++++++++++++++-
 docs/spec.toml                   |   4 +-
 4 files changed, 181 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-180915-dxbv/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-180915-dxbv/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42864492123 for 391df57eb7b886d51802ec9ac86a70004a721ff1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42864492123 for 391df57eb7b886d51802ec9ac86a70004a721ff1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42864492123 for 391df57eb7b886d51802ec9ac86a70004a721ff1)
- ci-result: `not a check` passed (Check results for 391df57eb7b886d51802ec9ac86a70004a721ff1 come from the CI run https://github.com/doodla/athena/actions/runs/42864492123 (753s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
