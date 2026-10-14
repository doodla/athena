# Job 20261014-030342-j237: verify "Build introspection from whichever signal fired; flag missing one"

## Request (verbatim)

> In crates/athena-cli/src/backtest.rs, D87 is now correctly under review on the real records, citing only ar1x through its derived host-error record, but no introspection record is built: introspection_for takes only a fired run's intervention events, and the host-error run's evidence is a derived record. Root cause: the introspection is tied to one kind of evidence instead of to whatever fired. Build the introspection record from the earliest valid citation of whichever signal fired (for a derived record, from its source intervention), and make the backtest report a finding when a decision is put under review with no introspection record. Verify with fixture records that copy ar1x exactly (the real run comes after landing).

## Reading (accepted by the user)

In backtest.rs, introspection_for will build the record from the earliest valid citation of any fired D87 run: an intervention event, or for a derived host-error record its source intervention. run_on_from will add a finding when a decision is under review and no introspection record was built. Tests use fixture records copying ar1x's intervention and its derived host-error record.

Assumptions:
- The new finding uses condition `d`, the introspection condition, and applies to any decision under review, not only D87.
- For a derived record, the introspection takes its intervention, job, class of work and time from the source intervention row in records.interventions; the earliest valid citation is chosen by (time, id).
- Observed host-error records have no intervention row, so they are not used as introspection sources unless the Introspection struct can carry them without a schema change. Otherwise they are left out and the missing-record finding fires.
- Earlier cases are the same signal's earlier citations, resolved to intervention ids, so the existing check_introspection still applies.
- Citations that fail validate_citations are skipped when choosing the earliest.
- The fixtures are in-memory Records built in the existing tests module; no real records/ are read.

## Acceptance criteria

1. [content] introspection_for in crates/athena-cli/src/backtest.rs no longer requires run.events to be non-empty. It selects the earliest valid citation among the run's intervention events, observed host-error records and derived records, using a derived record's source intervention.
2. [content] The D87 filter in run_on_from no longer has the `!r.events.is_empty()` restriction.
3. [content] run_on_from pushes a Finding when a decision is in under_review and report.introspection is None, naming the decision.
4. [behavior] Run `cargo test -p athena-cli backtest`. A fixture whose records copy ar1x (the intervention and its derived host-error record) gives D87 under review, a Some(introspection) whose intervention is the source intervention, job and class of work match ar1x, and no findings.
5. [behavior] Run `cargo test -p athena-cli backtest`. A fixture where a decision is under review but no introspection can be built gives a finding naming that decision.
6. [behavior] Existing backtest tests still pass. `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` are clean.

## Expected deliverables

- crates/athena-cli/src/backtest.rs (modify)
- CHANGELOG.md (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Running the backtest on the real records (after landing).
- Changing signal definitions, thresholds or the derive_host_error logic in athena-core.
- Writing the language-model fields of the introspection record.
- Governance files.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix ties introspection to whichever signal fired, not to a patch for the ar1x case.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture tests added to backtest.rs; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by running the new fixture tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One file, one focused change.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base b82100c4445d → candidate a133501f5707. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |  7 ++++
 crates/athena-cli/src/backtest.rs   | 80 +++++++++++++++++++++++++++++++++----
 crates/athena-cli/tests/backtest.rs | 51 ++++++++++++++++++++++-
 docs/design/defaults-loop.md        |  5 +++
 4 files changed, 134 insertions(+), 9 deletions(-)
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
