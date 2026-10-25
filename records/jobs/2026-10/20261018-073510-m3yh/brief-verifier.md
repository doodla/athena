# Job 20261018-073510-m3yh: verify "Mark base-failing checks pre-existing and judge each criterion anyway"

## Request (verbatim)

> At verification, a check that already failed on the base commit (recorded before the producer started, part 1) is marked pre-existing instead of rejecting the job, and each acceptance criterion is judged on its own instead of leaving them all unjudged when one unrelated check fails (part 2 of 2; run after part 1 lands).

## Reading (accepted by the user)

At verification, a check that failed in the job's baseline record (same name, failed on the base commit) is marked pre-existing and no longer fails the verdict. The verifier also runs when some other check fails, so every acceptance criterion gets its own result instead of none. Audience: the Athena operator reading verification records and verdicts.

Assumptions:
- Part 1 landed: the Baseline record is taken before the producer and held in closing.baseline in run.rs.
- A check is pre-existing only if the baseline has an entry with the same name and passed=false.
- The verifier now runs even when a non-pre-existing check fails, so criteria are judged; the final verdict still fails on that check.
- Prefer using the check note over a new CheckRun field, to avoid a schema change under deny_unknown_fields.
- No athena.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] A unit test in verify.rs shows judge() passes when a check failed in the baseline and fails again at verification, with the failure recorded as pre-existing and not listed in the verdict's failure reasons. Run `cargo test -p athena-core verify`.
2. [behavior] A unit test shows a check that passed in the baseline (or has no baseline entry) and fails at verification still makes the final verdict fail, with a reason naming it. Run `cargo test -p athena-core verify`.
3. [behavior] A lifecycle test shows that when one check fails, the verifier still runs and every acceptance criterion in the verification record has its own result, while the final verdict still fails on that check. Run `cargo test -p athena-cli --test lifecycle`.
4. [behavior] A lifecycle test shows a job whose base commit already fails a check, and whose change leaves it failing, can pass verification when everything else passes. Run `cargo test -p athena-cli --test lifecycle`.
5. [content] The verification record marks pre-existing failures distinctly (for example in the check note), so a reader sees which failures were already on the base.
6. [content] A missing baseline, or one with a note saying it could not run, treats no failure as pre-existing. An empty baseline is never read as success.
7. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass with no new warnings.
8. [content] CHANGELOG.md records the new behavior in plain language for an outside reader.

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how the baseline is taken (part 1).
- Re-running checks or retrying flaky ones.
- Changing landing, CI routing or budget logic.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and lifecycle tests; cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise the verdict outcomes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the verdict computation and the verifier-skip condition.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A missing or empty baseline marks nothing pre-existing.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across verify.rs and run.rs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in verify.rs: baseline holds check `test` passed=false with no note; verification check `test` also fails. Run: `cargo test -p athena-core verify`. Observes: final_verdict is Pass; reasons contain no 'check test failed'; mark_pre_existing gives the check a note starting with the pre-existing phrase.
  Fails without the behavior: Without baseline handling judge pushes 'check test failed' and the verdict is Fail, so the assertion fails.
- Criterion 2: Unit tests: baseline has `test` passed=true; baseline None; baseline with only another check name. `test` fails at verification in each. Run: `cargo test -p athena-core verify`. Observes: final_verdict is Fail, reasons include 'check test failed', note not marked pre-existing.
  Fails without the behavior: If every failed check were treated as pre-existing the verdict would pass and the assertion fails.
- Criterion 3: Lifecycle test (rewrite of the 'without spending a verifier' test): candidate fails a check not failing at base. Read verification.json. Run: `cargo test -p athena-cli --test lifecycle`. Observes: Verifier role invoked; one result per acceptance criterion, each from the verifier (not 'did not assess'); final_verdict Fail; state Rejected; nothing landed.
  Fails without the behavior: With the gate in place the verifier is never invoked and every criterion shows 'the verifier did not assess this criterion'.
- Criterion 4: Lifecycle test: config with a check `sh -c 'exit 1'` failing at base and after the change; everything else passes. Counter case: check passes at base, fails after. Run: `cargo test -p athena-cli --test lifecycle`. Observes: State Landed; baseline.json has the check failed; verification.json check failed with pre-existing note; final_verdict Pass. Counter case is Rejected.
  Fails without the behavior: Without pre-existing handling the job is Rejected on the failing check.
- Criterion 5: Assertions on the stored check note in the verify.rs unit test and lifecycle test 4, plus an introspect.rs unit test that failure_details omits pre-existing checks. Run: `cargo test -p athena-core verify && cargo test -p athena-cli --test lifecycle`. Observes: The pre-existing check note begins with the pre-existing phrase; a non-pre-existing failure's note lacks it.
  Fails without the behavior: With no mark, the note assertion fails.
- Criterion 6: Unit tests on Baseline::failed_before and judge: baseline None, empty checks with no note, a note 'the baseline could not run: ...', and a matching failed entry plus a note. Run: `cargo test -p athena-core verify`. Observes: No check is pre-existing and the failure still fails the verdict in each case.
  Fails without the behavior: Reading an empty or noted baseline as success would pass the verdict and fail the assertion.
- Criterion 7: Run the full gates, then confirm the new tests are in the run. Run: `cargo test`. Observes: All tests pass, including the new verify, records, introspect and lifecycle tests; then `cargo clippy --all-targets` and `cargo fmt --check` report no warnings or diffs.
  Fails without the behavior: If the change broke an existing test (for example one still expecting no verifier on a failed check, or a VerdictInputs call missing the baseline field) cargo test or cargo clippy --all-targets fails to pass or compile; unformatted code makes cargo fmt --check fail.
- Criterion 8: Read the new CHANGELOG entry under the unreleased heading. Run: `git diff CHANGELOG.md`. Observes: A plain-language entry says base-failing checks are marked pre-existing and no longer reject a job, and the verifier now judges every criterion even when a check fails.

Planned files:

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)

## The change

Base 69013d547797 → candidate 668f706c0954. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   7 ++
 crates/athena-cli/tests/lifecycle.rs |  77 ++++++++++++++++++--
 crates/athena-core/src/introspect.rs |  27 ++++++-
 crates/athena-core/src/run.rs        |  17 +++--
 crates/athena-core/src/verify.rs     | 133 +++++++++++++++++++++++++++++++++--
 crates/athena-data/src/records.rs    |  35 ++++++++-
 6 files changed, 280 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (948 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
