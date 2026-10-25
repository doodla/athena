# Job 20261023-234313-h49d: verify "Stop requiring a full cargo test run from the producer"

## Request (verbatim)

> Producers rerun the full cargo test suite (9 to 11 minutes) many times: job 20261019-013309-nvbb's producer ran it six times and hit its 1800 s deadline, and the producer brief sends a report back without 'a real full cargo test --locked result' (job 20261023-165947-e7bd). Athena runs fmt, clippy and the full suite on the candidate itself after the producer, so the producer should run targeted tests while working and need not report a full-suite run; a send-back should not demand one. Keep any check that would let a failing candidate through.

## Reading (accepted by the user)

The producer prompt tells producers to run the full `cargo test --locked`, and run.rs sends a report back (once) without a real result for it. Remove both, tell producers to run targeted tests, and keep the post-producer fmt, clippy and full-suite checks that Athena runs on the candidate. The audience is Athena operators and producers, who lose 9 to 11 minutes per full run.

Assumptions:
- Athena already runs fmt, clippy and the full test suite on the candidate and a failure fails it, as the request states; I did not trace that code path in full.
- The send-back mechanism for unmet acceptance criteria stays; only the full-test demand is removed.
- Pinned study jobs were already exempt, so their behavior is unchanged.
- A new decision entry (next free number after the highest in docs/decisions.md) is appropriate because this reverses part of D95; D95 itself is not rewritten.
- The producer may still report a full-suite run if it chooses to; the report schema keeps checks_run.

## Acceptance criteria

1. [content] crates/athena-context/prompts/producer.md no longer tells the producer to run the full `cargo test --locked` or to report it. It says to run targeted tests (`cargo test -p <crate> <filter>`) while working and that Athena runs fmt, clippy and the full suite on the candidate.
2. [content] In crates/athena-core/src/run.rs the producer loop no longer sends a report back for lacking a real full `cargo test --locked` result. The `ran_full_tests` and `tests_missing` gating and the sent-back test text are removed. The send-back for unmet acceptance criteria is kept.
3. [content] The checks Athena runs on the candidate (fmt, clippy, full test) and their effect on the final verdict are unchanged. A failing check still fails the candidate. The producer-report-gaps note to the verifier no longer tells it to run the full suite, and the verifier prompt still says to cite recorded results.
4. [behavior] Run `cargo test --locked -p athena-cli --test lifecycle`. The tests show that a producer report with no full-suite check, or with `not-run`, goes to the verifier without a send-back. A report with unmet criteria is still sent back once. A candidate whose recorded test check fails is still rejected.
5. [behavior] Run `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`. Both are clean, with no unused-code warnings left by the removed gating.
6. [content] docs/decisions.md records the change as a new decision that amends D95 and its D95 follow-up wording, and docs/design/athena-v0.md and CHANGELOG.md [Unreleased] no longer say the producer must run the full suite. The documents state the reason with the figures from the two jobs (nvbb: six runs, 1800 s deadline; 9 to 11 minutes per run).

## Expected deliverables

- crates/athena-context/prompts/producer.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-context/src/outputs.rs (modify)

## Out of scope

- Changing verifier behavior or its prompt beyond the gaps note
- Changing athena.toml timeouts or budgets
- Changing which checks Athena runs on the candidate
- Changing the send-back for unmet acceptance criteria

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Lifecycle and job_plan tests are updated for the new behavior; cargo test and clippy must be clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The demand is removed in both the prompt and the run.rs enforcement, not only the wording of one.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A criterion runs the lifecycle tests and checks that no send-back happens and that failing candidates are still rejected.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The full-suite gate stays enforced by Athena's own post-producer check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision and CHANGELOG text states the reason and the measured run times.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the producer test requirement.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the producer.md diff with `git diff`; the athena-context prompt budget tests also run. Run: `cargo test --locked -p athena-context`. Observes: The bullet no longer says to run the full `cargo test --locked`, to report it, or that not-run is unacceptable. It names `cargo test -p <crate> <filter>` and says Athena runs fmt, clippy and the full suite on the candidate. The prompt budget tests pass.
- Criterion 2: Read the run.rs diff with `git diff`. A compile with `-D warnings` (criterion 5) proves nothing is left unused. Run: `cargo build --locked`. Observes: `ran_full_tests`, `enforce_tests`, `tests_missing` and the sent-back test paragraph are gone. The `unmet` send-back branch and `PRODUCER_SEND_BACKS` remain.
- Criterion 3: Read the diff to confirm that `checks.rs` and the verdict code are untouched. The lifecycle test `a_failing_check_still_runs_the_verifier_and_every_criterion_is_judged` still passes. The gaps note text is checked in the updated gap test, and verifier.md is untouched, so its recorded-results instruction remains. Run: `cargo test --locked -p athena-cli --test lifecycle`. Observes: A failing check still gives a Rejected state and a Fail verdict. The gaps note contains no instruction to run the full suite.
- Criterion 4: Rewrite the full-test tests in lifecycle.rs. A report with no full-test check and a report with `not-run` give 1 producer call, the verifier is reached and there is no 'sent back' line. A report with an unmet criterion is still sent back once, and the existing `a_criterion_*` tests are kept. A rejected candidate with a failing check stays (the existing README removal test). The no-change test with a not-run report fails with 'made no change' and without a test-related reason. Remove the 'no real full' assertion. Update job_plan.rs `a_send_back_keeps_the_plan_section...` to use an unmet-criterion report for its send-back instead of a missing test check. Run: `cargo test --locked -p athena-cli --test lifecycle`. Observes: All lifecycle and job_plan tests pass: no send-back for missing tests, one send-back for unmet criteria, and a failing check is rejected.
  Fails without the behavior: If the gate remained, the updated tests that assert exactly one producer invocation for a report with no full-test check or with `not-run` would see two, and would fail. If the unmet-criteria send-back were removed, the criterion tests would fail. If a failing check were let through, a_failing_check_still_runs_the_verifier... would fail.
- Criterion 5: Run clippy, then `cargo fmt --check`. Run: `cargo clippy --locked --all-targets -- -D warnings`. Observes: Both exit clean.
  Fails without the behavior: An unused `FULL_TEST_COMMAND`, `ran_full_tests` or `enforce_tests`, or an unused test helper, would produce a dead-code warning that -D warnings turns into an error.
- Criterion 6: Read the diff. In decisions.md, add D113 after D112 and amend D95 and D103 (line ~2377: the full-test demand and the gaps note). In athena-v0.md, update line ~772 (the checks list that says the producer 'must run the full cargo test --locked (D95)') and line ~1780 (the checks_run rule). In CHANGELOG.md [Unreleased], add an entry; leave the old 0.x entry alone as history. Each states the reason: job 20261019-013309-nvbb ran the suite six times and hit its 1800 s deadline, and each run takes 9 to 11 minutes. Also run `cargo test --locked -p athena-core` for any policy test that reads the docs. Run: `git diff -- docs/decisions.md docs/design/athena-v0.md CHANGELOG.md`. Observes: The documents state the new rule and the figures. No current document says the producer must run the full suite.

Planned files:

- crates/athena-context/prompts/producer.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-context/src/outputs.rs (modify)

## The change

Base 48092d2377ed → candidate a3155211718e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |   9 ++
 crates/athena-cli/tests/job_plan.rs       |  12 ++-
 crates/athena-cli/tests/lifecycle.rs      | 139 +++++++++++-------------------
 crates/athena-context/prompts/producer.md |   2 +-
 crates/athena-core/src/run.rs             |  55 ++++--------
 docs/decisions.md                         |  25 ++++++
 docs/design/athena-v0.md                  |  13 ++-
 7 files changed, 113 insertions(+), 142 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (993 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
