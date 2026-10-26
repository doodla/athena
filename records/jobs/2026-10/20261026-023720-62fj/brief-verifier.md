# Job 20261026-023720-62fj: verify "Let producers leave full-suite criteria to Athena's checks"

## Request (verbatim)

> Priority: urgent
> Most jobs still lose about 7 minutes to a producer send-back over a criterion only Athena's own checks can show: readers write criteria such as 'cargo clippy --all-targets and cargo fmt --check are clean, and cargo test for the workspace passes', producers no longer run the full suite (d73effa), so the producer's self-check reports it unmet and the run is sent back (job 20261026-015416-94rc lost 431 s this way). When a criterion is satisfied by Athena's own recorded checks (fmt, clippy, the full test suite), the producer's self-check marks it 'left to Athena's checks' instead of unmet, and the verifier still judges it from the recorded checks.
> Acceptance:
> - A test shows a producer report that leaves a full-suite criterion to Athena's checks is not sent back, while a report that omits a targeted-test criterion still is.
> - The verifier still fails the job when the recorded full-suite check fails.
> Files: crates/athena-core/src/run.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md

## Reading (accepted by the user)

The producer self-check (unmet_criteria in run.rs) stops sending a report back when a criterion is marked 'left to Athena's checks' and is a full-suite, fmt or clippy criterion. The producer brief (briefs.rs) tells producers how to mark it. Targeted-test criteria that are omitted or unmet still go back, and the verifier still judges the left-over criteria from the recorded checks. Audience: Athena's run loop and its verifier, which pay for the wasted send-back.

Assumptions:
- Marking uses the existing met/unmet status plus a fixed evidence prefix "left to Athena's checks", so the CriterionStatusOutput enum and its JSON schema in outputs.rs stay unchanged (the request lists only run.rs, briefs.rs and CHANGELOG.md). A new enum variant would also touch outputs.rs.
- Only criteria that name Athena's own recorded checks (cargo fmt --check, cargo clippy, the full or workspace cargo test) may be left this way. A criterion about a targeted test or any other behavior must still be reported met, so an omitted or unmet targeted-test criterion is still sent back. The criterion text must be recognizable as a full-suite check, not just marked by the producer.
- Criteria left to Athena's checks are passed on to the verifier to judge from the recorded checks, the way a still-incomplete report is flagged today (run.rs near line 1947). The verifier fails the job when the recorded full-suite check fails, which is existing behavior and must keep working.
- The job is urgent because it cuts a redo-work send-back that costs about 7 minutes per job.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-core/src/run.rs shows unmet_criteria returns nothing for a report that marks a full-suite criterion (clippy, fmt --check and workspace cargo test) as left to Athena's checks, and still returns a targeted-test criterion that the report omits. Run: cargo test -p athena-core.
2. [behavior] A test shows a criterion marked 'left to Athena's checks' that is not an Athena-check criterion (for example a targeted-test or content criterion) is still returned as unmet and sent back. Run: cargo test -p athena-core.
3. [behavior] A test shows the verifier still fails the job when the recorded full-suite check fails, even though the producer left that criterion to Athena's checks. Run: cargo test -p athena-core.
4. [content] The producer brief's criteria_check section in crates/athena-context/src/briefs.rs tells the producer to mark a criterion that only Athena's recorded checks (fmt, clippy, the full test suite) can show as left to Athena's checks, with the exact marker text, and says targeted-test criteria must still be reported met with evidence. A briefs.rs test asserts that wording.
5. [content] CHANGELOG.md has an entry describing the change: self-check no longer sends back criteria left to Athena's checks.
6. [behavior] cargo fmt --check and cargo clippy --all-targets are clean, and cargo test passes for the workspace (Athena's recorded checks).

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/outputs.rs (modify)

## Out of scope

- Changing how the verifier judges criteria or how Athena records fmt, clippy and test checks.
- Re-running the full suite in the producer (decision d73effa stands).
- The scope-guard work from job 20261026-015416-94rc and its cosmetic docs finding.
- Changing the number of send-backs (PRODUCER_SEND_BACKS) or the send-back text beyond what the new marking needs.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust code and Rust tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests cover the self-check and the brief; cargo test, clippy and fmt --check stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check the self-check outcome (sent back or not) and the verifier's failure, not that types exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The checks are the send-back decision for a full-suite criterion versus a targeted-test criterion, and the verifier still failing on a failed recorded check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the self-check, the brief and the changelog.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog entry are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the self-check rule that wrongly counts Athena-owned checks as unmet, not in a retry or timeout.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in run.rs: accepted criteria are a full-suite one ('cargo clippy --all-targets and cargo fmt --check are clean, and cargo test for the workspace passes') and a targeted one ('cargo test -p athena-core passes the new test'). The report marks the first with evidence 'left to Athena's checks: ...' and omits the second. Call unmet_criteria. Run: `cargo test -p athena-core`. Observes: unmet_criteria returns only the targeted-test criterion; the full-suite criterion is not returned.
  Fails without the behavior: Without the marker handling the full-suite criterion would also be returned, so the report would be sent back.
- Criterion 2: Unit test in run.rs: mark with the same evidence prefix (a) a content criterion ('the docs name the flag'), (b) a targeted-test criterion ('cargo test -p athena-core passes'), and (c) a non-prefixed full-suite criterion reported unmet. Call unmet_criteria. Run: `cargo test -p athena-core`. Observes: (a) and (b) are returned as unmet. (c) is returned too, and the left-over helper lists none of the three.
  Fails without the behavior: If the marker alone sufficed, the content and targeted criteria would be skipped and the test would fail.
- Criterion 3: Unit test in run.rs: the report leaves the full-suite criterion to Athena's checks, so unmet_criteria is empty and the left-over helper lists it. Then call verify::judge with a failed CheckRun (passed false, exit 101) and the left-over criterion as a Criterion. Run: `cargo test -p athena-core`. Observes: final_verdict is Fail and reasons contains 'check test failed', while the self-check did not send the report back.
  Fails without the behavior: If the left-over handling suppressed or changed the recorded-check failure, the verdict would not be Fail.
- Criterion 4: briefs.rs test builds the producer brief, takes the criteria_check section and asserts it contains "left to Athena's checks" (via the constant), fmt, clippy and the full test suite, and says targeted-test criteria are still reported met with evidence. The existing test requiring the last two lines of the section to be the criteria stays green, because the new text goes before the criteria list. Run: `cargo test -p athena-context`. Observes: The section contains the exact marker text and the targeted-test sentence.
  Fails without the behavior: Without the brief wording the assertions on the marker and on the targeted-test sentence fail.
- Criterion 5: Read the diff for a new entry under Unreleased in CHANGELOG.md. Run: `git diff -- CHANGELOG.md`. Observes: The entry says the producer self-check no longer sends back full-suite, fmt and clippy criteria the producer left to Athena's checks, and that targeted-test criteria still go back.
- Criterion 6: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked, the checks Athena records. Run the fmt check with the allowed command cargo fmt. Run: `cargo test`. Observes: All three exit 0, including the budgets and policy tests, since the briefs.rs wording sits outside the budgeted prompt sets.
  Fails without the behavior: Any formatting, lint or test regression, including a schema or budget test broken by the outputs.rs doc comment.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- CHANGELOG.md (modify)

## The change

Base 82dd546a01b2 → candidate 9c2e38944e9b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   7 +
 crates/athena-context/src/briefs.rs  |  40 +++++-
 crates/athena-context/src/outputs.rs |   3 +-
 crates/athena-core/src/run.rs        | 240 ++++++++++++++++++++++++++++++++++-
 4 files changed, 281 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1279 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
