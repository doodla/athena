# Job 20261024-015920-ywyz: verify "Stop requiring producers to run the full test suite"

## Request (verbatim)

> try again on 20261023-234313-h49d

## Reading (accepted by the user)

Redo job h49d from the base: remove the producer's full-suite requirement and the run.rs send-back for it, keep Athena's post-producer checks, and amend every doc that states the old rule. The old CHANGELOG D95 entry stays with a note that D113 supersedes it, and the producer runs the full suite at most once.

Assumptions:
- Goal: producers stop rerunning the 9 to 11 minute full suite; audience is Athena operators and producers, and the verifier.
- The code and prompt change is redone as the last attempt made it, since changes were discarded; only the docs differ (CHANGELOG D95 entry, D103/D95 pointers).
- D113 is the next free decision number, as in the last attempt.
- The older CHANGELOG D95 entry is kept with a supersession note, per the user's reply.
- The producer runs the full suite at most once, per the user's reply.

## Acceptance criteria

1. [content] crates/athena-context/prompts/producer.md no longer tells the producer to run or report the full `cargo test --locked`. It says to run targeted tests (`cargo test -p <crate> <filter>`) while working and that Athena runs fmt, clippy and the full suite on the candidate.
2. [content] In crates/athena-core/src/run.rs the producer loop no longer sends a report back for lacking a real full-suite result. The `ran_full_tests` and `tests_missing` gating and its send-back text are removed. The send-back for unmet acceptance criteria is kept. The producer-report-gaps note no longer tells the verifier to run the full suite.
3. [content] The checks Athena runs on the candidate (fmt, clippy, full test) and their effect on the verdict are unchanged. The verifier prompt still says to cite recorded results.
4. [behavior] Run `cargo test --locked -p athena-cli --test lifecycle` and `cargo test --locked -p athena-cli --test job_plan`. A producer report with no full-suite check, or with `not-run`, goes to the verifier without a send-back. A report with unmet criteria is still sent back once. A candidate whose recorded test check fails is still rejected.
5. [behavior] Run `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`. Both are clean, with no unused-code warnings from the removed gating.
6. [content] docs/decisions.md has a new decision, D113, amending D95 and D103. The D95 and D103 entries each get a short pointer forward to D113. D113 states once that the earlier text stays as history, and gives the reason with the figures: job nvbb ran the suite six times and hit its 1800 s deadline, and a run takes 9 to 11 minutes.
7. [content] CHANGELOG.md [Unreleased]: the older D95 entry (near line 624) is kept, not deleted, and gets a short note that D113 supersedes it. A D113 entry gives the same reason and figures. docs/design/athena-v0.md no longer says the producer must run the full suite.
8. [content] A grep of decisions.md, athena-v0.md and CHANGELOG.md for 'full `cargo test --locked`', 'second miss' and 'ran_full_tests' finds no statement of the requirement outside quoted history or the supersession notes. The criterion evidence cites that search command and its output.

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

- Changing the post-producer fmt, clippy and full-suite checks or the verdict logic
- Adding a new pre-handover wording-search gate
- Deleting the old D95 entries

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Behavior criteria run the lifecycle and job_plan tests, clippy and fmt.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria exercise the send-back and rejection paths.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the behavior and wording, and name the grep to cite.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix every place the old rule is stated, not just the CHANGELOG line.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Docs cite job figures: six runs, 1800 s, 9 to 11 minutes.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the stated files change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff and grep the file for the old wording. Run: `git diff -- crates/athena-context/prompts/producer.md`. Observes: The old line (run the full `cargo test --locked`, `not-run` not acceptable) is gone. In its place: run targeted `cargo test -p <crate> <filter>` while working, and Athena runs fmt, clippy and the full suite on the candidate. A Grep for 'full `cargo test --locked`' in the file returns nothing.
- Criterion 2: Diff run.rs and grep for the removed names. Run: `git diff -- crates/athena-core/src/run.rs`. Observes: `ran_full_tests`, `FULL_TEST_COMMAND`, `enforce_tests`, `tests_missing` and the full-suite send-back text are removed. The unmet-criteria send-back remains. The producer-report-gaps note has no instruction to run the full suite.
- Criterion 3: Show that the diff touches no check or verdict code and that the verifier prompt is unchanged. Run: `git diff --stat`. Observes: No change under the check or verdict code, and none to crates/athena-cap-change/prompts/verifier.md, which still says to cite recorded results.
- Criterion 4: Run the rewritten lifecycle tests, then the job_plan tests, as two commands in a single demonstration. Lifecycle covers: a `not-run` report and a report with no full-suite check (one producer run, no send-back, verifier reached); an unmet criterion (two producer runs, sent back once); an unmet criterion still unmet (verifier gets producer-report-gaps); a recorded failing test check (rejected). job_plan's send-back test now triggers the send-back with an unmet criterion. Run: `cargo test --locked -p athena-cli --test lifecycle && cargo test --locked -p athena-cli --test job_plan`. Observes: Both test binaries pass. Producer call count is 1 for the not-run and missing-check reports and 2 for unmet criteria. The failing-test-check case is still rejected. The job_plan second prompt has '## Sent back' and the plan section, and the hash matches the plan record.
  Fails without the behavior: With the gating still in place, the not-run test would see 2 producer calls and a 'sent back' line and fail. If the unmet-criteria send-back were removed, the 2-call tests in lifecycle and job_plan would fail. If a failing test check no longer rejected, the rejection test would fail.
- Criterion 5: Run clippy, then fmt check. Run: `cargo clippy --locked --all-targets -- -D warnings && cargo fmt --check`. Observes: Both exit 0 with no warnings, including no unused constant, function or import from the removed gating.
  Fails without the behavior: If `FULL_TEST_COMMAND` or `ran_full_tests` were left unused, clippy would report dead code and exit non-zero.
- Criterion 6: Read the diff of decisions.md. Run: `git diff -- docs/decisions.md`. Observes: A new '## D113' after D112 amends D95 and D103, says once that the earlier text stays as history, and gives the figures: job nvbb ran the suite six times, hit its 1800 s deadline, and a run takes 9 to 11 minutes. D95 and D103 each get a one-line pointer to D113.
- Criterion 7: Read the diff of CHANGELOG.md and athena-v0.md. Run: `git diff -- CHANGELOG.md docs/design/athena-v0.md`. Observes: CHANGELOG [Unreleased] keeps the D95 entry and adds a note that D113 supersedes it. A new D113 entry gives the same reason and figures. athena-v0.md line 772 and the B.2 paragraphs no longer say the producer must run the full suite or that a second miss fails the job.
- Criterion 8: Run the Grep tool with pattern 'full `cargo test --locked`|second miss|ran_full_tests' over docs/decisions.md, docs/design/athena-v0.md and CHANGELOG.md, output_mode content, and cite the command and output. Run: `Grep tool: pattern 'full `cargo test --locked`|second miss|ran_full_tests', paths docs/decisions.md docs/design/athena-v0.md CHANGELOG.md, output_mode content`. Observes: Every hit is in D95 or D103 history, D113 text quoting it, or a 'superseded by D113' note. athena-v0.md has no hit. No hit states the requirement as current.

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

Base 23f2f3b9bcfe → candidate 7c9ecc8c84ac. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              | 12 +++++
 crates/athena-agent/src/fake.rs           |  2 +-
 crates/athena-cli/tests/job_plan.rs       | 12 +++--
 crates/athena-cli/tests/lifecycle.rs      | 64 ++++---------------------
 crates/athena-context/prompts/producer.md |  2 +-
 crates/athena-context/src/outputs.rs      |  1 +
 crates/athena-core/src/run.rs             | 77 ++++++++++---------------------
 docs/decisions.md                         | 24 ++++++++++
 docs/design/athena-v0.md                  | 18 +++-----
 9 files changed, 88 insertions(+), 124 deletions(-)
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
