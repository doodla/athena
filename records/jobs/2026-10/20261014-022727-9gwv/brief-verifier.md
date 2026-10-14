# Job 20261014-022727-9gwv: verify "Make backtest evidence postdate decisions and validate each citation"

## Request (verbatim)

> 'athena backtest' now passes on the real records, but its evidence is wrong, so the pass is false confidence. Fix the three root causes. (1) Evidence must postdate the decision: D87 reached main at 2026-10-13T19:21Z (commit 383edca, PR #41), yet it cites interventions 20261013-150702-kn3q (15:07Z) and 20261013-162049-qk2d (16:20Z), which predate it. Take a decision's start time from the history of its text, and count only later records. (2) Derivation matched the word 'merged' anywhere: kn3q's 422 came from opening a PR (the wrong-base bug), and 'merged' there describes the operator's own action. Derive a merge refusal only when the text ties the HTTP code to the refused merge itself; when unsure, derive nothing. (3) The pass conditions checked counts and that ids exist, not that each citation is valid. Make the backtest validate each cited record against the signal's fields and time window, and fail when any citation is invalid. Expected on the real records: D87 under review citing only ar1x, both directly and through its derived record; the introspection record names ar1x's event; nothing else is flagged. Report the real result either way.

## Reading (accepted by the user)

Fix three root causes in 'athena backtest'. (1) Take D87's start from the git history of its text in docs/decisions.md and count only later records. (2) Derive a host-refused merge only when the text ties the HTTP code to the refused merge. (3) Validate every citation against the signal's fields and time window, and fail on any invalid one. Report the real-records result either way.

Assumptions:
- The decision start time comes from running `git log` over the repository's own history at run time. Tests build temporary git repos or inject the start time.
- The start is the first commit that added the entry's text (D87: 383edca, 2026-10-13T19:21Z), not a later amendment. Only strictly later records count.
- I could not read the real records or the git history here, so the expected real result is taken from the request and not measured.
- The derivation is narrow: the HTTP code and a refused merge must be tied in one sentence or clause, and wording about opening a PR excludes it.

## Acceptance criteria

1. [content] backtest.rs gets each decision's start time from the git history of the decision's text in docs/decisions.md. For D87 that is the commit that introduced the entry. Only host-error and intervention records dated after that start are counted by the signals. The start time is a field of the report.
2. [content] derive_host_error derives a refusal only when the text ties an HTTP code to the refused merge itself. A bare occurrence of 'merge', 'merges' or 'merged' no longer suffices. When unsure it returns None.
3. [content] The backtest validates each cited record (observed host-error, derived record with its source intervention, and intervention): it matches the signal's fields and falls inside the signal's time window, meaning it postdates the decision's start. Any invalid citation adds a Finding and makes Report::passed() false. The old checks of counts and id existence remain.
4. [content] The citation check also covers the derived record's source intervention and the introspection record's intervention. An introspection whose triggering event predates the decision is a finding.
5. [behavior] Run `cargo test` and `cargo clippy --all-targets`. Both pass. New tests cover: a record before the decision start is not counted; an intervention that mentions 'merged' in the sense of opening a pull request, with HTTP 422, derives nothing; a text that ties HTTP 422 or 405 to the refused merge derives a record; an invalid citation fails the backtest.
6. [behavior] Run `cargo run -- backtest` on the real records. D87 is under review citing only ar1x, both directly and through its derived record. The introspection record names ar1x's event. Nothing else is flagged. The kn3q and qk2d interventions are not cited. The actual output is reported, even if it differs.
7. [behavior] Run `cargo fmt --check`. It passes.

## Expected deliverables

- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-cli/tests/backtest.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/decision-sets.md (modify)

## Out of scope

- Changing D87 or D88 text, or docs/decisions.md and docs/decision-links.toml
- Editing records/ or writing derived records to disk
- Adding signals or decisions beyond D87 and D88
- Changing governance files such as athena.toml, defaults.toml, CI workflows or lint configuration

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover each of the three fixes; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the three causes, not the single D87 case.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The real-records run of the backtest is exercised and its result reported.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the backtest and the derivation.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the structure from backtest.rs, watcher.rs and the test fixtures.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 01ee7478d301 → candidate c95f1307ddc8. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |  13 ++
 crates/athena-cli/src/backtest.rs   | 298 +++++++++++++++++++++++++++++++++++-
 crates/athena-cli/tests/backtest.rs | 266 +++++++++++++++++++++++++++++++-
 crates/athena-core/src/watcher.rs   | 119 ++++++++++++--
 docs/design/decision-sets.md        |  15 +-
 5 files changed, 682 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (436 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
