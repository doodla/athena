# Job 20261014-024334-p71y: verify "Make backtest evidence postdate decisions and validate each citation"

## Request (verbatim)

> Retry 20261014-022727-9gwv. Agents' copies leave out records/, so verify with fixture records that copy the real interventions' text exactly: kn3q ('athena land --pr failed with HTTP 422 for job vbdn; operator opened PR #38 by hand, simulated the squash, and merged it with the job trailer.' at 2026-10-13T15:07:02Z), qk2d (16:20:49Z) and ar1x ('Operator squash-merged PR #41 by hand with the job's trailers after GitHub refused a merge commit (HTTP 405).' at 19:21:56Z), with D87 starting at 19:21Z. The fixtures must show D87 under review citing only ar1x. The run over the real records comes after landing and is not a criterion. Also fix the verifier's minor findings: derive a merge refusal only when the code is tied to the refusal of the merge itself (for example 'refused a merge ... (HTTP 405)'), so 'the push failed with HTTP 422' derives nothing; and date a decision from the first commit that introduced its id heading ('## D87 '), not from its current full heading text.

## Reading (accepted by the user)

Redo job 9gwv. The backtest dates a decision from the first commit that introduced its '## D87 ' id heading. It counts only later records, derives a merge refusal only when the HTTP code is tied to the refused merge, and fails on any invalid citation. Tests use fixture records with the exact kn3q, qk2d and ar1x text and times, and must show D87 under review citing only ar1x.

Assumptions:
- The previous attempt's code is not in this copy: backtest.rs has no start_from_history, so the work is redone in full from the current tree.
- With no records/ directory, the real-records run is not a criterion. The fixtures stand in for it.
- The decision start comes from git log in the real repo. Tests inject a start time so they do not depend on history.
- No governance files change.

## Acceptance criteria

1. [behavior] A backtest test builds fixture records with the verbatim text and timestamps of kn3q (2026-10-13T15:07:02Z), qk2d (16:20:49Z) and ar1x (19:21:56Z), with D87 starting 2026-10-13T19:21Z. D87 is under review citing only ar1x, directly and through its derived record. kn3q and qk2d are not cited. Run: cargo test.
2. [behavior] Unit tests: 'GitHub refused a merge commit (HTTP 405)' and 'Operator squash-merged PR #41 by hand ... after GitHub refused a merge commit (HTTP 405).' derive a merge refusal. kn3q's text derives nothing. 'I merged by hand because the push failed with HTTP 422' and 'the push failed with HTTP 422' derive nothing. Run: cargo test.
3. [content] The refusal derivation requires the HTTP code to be tied to the refusal of the merge itself, for example 'refused a merge ... (HTTP 405)'. A bare 'merged' elsewhere in the text is not enough.
4. [content] A decision's start time comes from the first commit that introduced its id heading ('## D87 '), not from its current full heading text. The code finds that commit with git history, and a test or code path shows that a renamed heading still dates from the first commit.
5. [behavior] The backtest counts only records after the decision's start. It validates each cited record against the signal's fields and time window, and fails when any citation is invalid. A test with a pre-start cited record fails the backtest. Run: cargo test.
6. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass.
7. [content] The clause splitting no longer breaks on decimal points or version numbers, or the derivation does not depend on it.

## Expected deliverables

- crates/athena-core/src/watcher.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/tests/backtest.rs (modify)
- crates/athena-cli/tests/watcher.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- docs (modify)

## Out of scope

- Running the backtest over the real records
- Changing records/
- Changing governance files or CI
- New signals or decisions beyond D87

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture-based tests for each fix; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the derivation tie, the start-time lookup and the citation validation, not only the D87 instance.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run with cargo test, since the verifier has no records/.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixture text is the public intervention text, with no personal data.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The change stays inside the watcher and backtest modules and their tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 58365da5cc02 → candidate 8cbf9fb3ffe3. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |   9 ++
 crates/athena-cli/src/backtest.rs   | 224 +++++++++++++++++++++++++++++++--
 crates/athena-cli/tests/backtest.rs | 239 +++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/watcher.rs   | 178 +++++++++++++++++++++++----
 docs/design/decision-sets.md        |  16 ++-
 5 files changed, 627 insertions(+), 39 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (434 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
