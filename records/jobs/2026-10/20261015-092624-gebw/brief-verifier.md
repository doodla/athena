# Job 20261015-092624-gebw: verify "Keep rejected producer reports and match criteria tolerantly"

## Request (verbatim)

> Jobs 20261015-054026-k2ej and 20261015-085652-5qem failed at the producer self-check with criteria 'not met or not reported', including plain ones like 'cargo test passes', and the producer report that triggered it is recorded nowhere, so nobody can tell whether the producer or the check was wrong. Keep that report, and make sure the check is not failing producers that did the work.

## Reading (accepted by the user)

When the producer self-check in run.rs fails a job (unmet criteria or missing full test run), the redacted report that triggered it is kept in the job's records. Criterion matching is loosened to ignore numbering, kind tags, case, whitespace and trailing punctuation, so producers that did the work are not failed on wording. The audience is the maintainer diagnosing failed jobs such as k2ej and 5qem.

Assumptions:
- Cause is unknown from the records; the exact-text match in unmet_criteria (run.rs) is the likely brittle point, so the change both records the report and loosens the match while staying strict about missing or unmet criteria.
- The report is stored as a file in the job directory (or an existing record field) rather than a new record schema.
- The send-back wording stays; only matching and persistence change.

## Acceptance criteria

1. [behavior] When the self-check fails a job, both on the final send-back and on the first, the redacted producer report (summary, checks_run, criteria_check with statuses and evidence) is written under the job's directory or records, and the failure reason names where it is. Run cargo test to check a test that fails a job through a fake producer and finds the report.
2. [behavior] A report whose criteria_check entry differs from the accepted criterion only by a leading number, a [content]/[behavior] kind tag, letter case, surrounding or repeated whitespace, or trailing punctuation counts as reporting that criterion; cargo test has unit tests for unmet_criteria covering each variant.
3. [behavior] A criterion absent from criteria_check, or reported with a status other than met, is still listed as unmet, and a different criterion with similar words is not matched; unit tests show it.
4. [behavior] A plain criterion such as 'cargo test passes' reported as met with evidence is not failed by the check, shown in a test.
5. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass for the workspace.
6. [content] The kept report goes through the redactor and holds no secrets or personal data.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/records.rs (modify)

## Out of scope

- Re-running or repairing jobs k2ej and 5qem
- Changing the full-test-run requirement or the number of send-backs
- Changing the verifier's criteria checking

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for matching and for report persistence; clippy and fmt clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Persist the evidence and fix the matching, not only the two jobs.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The kept report is redacted first.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in run.rs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base b14266b0ba2a → candidate 9ddbd9a79ba7. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |   8 ++
 crates/athena-core/src/run.rs           |  50 +++++++++++-
 crates/athena-core/src/run_pin_tests.rs | 140 ++++++++++++++++++++++++++++++++
 3 files changed, 194 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (672 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
