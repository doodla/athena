# Job 20261024-170225-89rk: verify "Retry from the rejected candidate's tree when only content criteria"

## Request (verbatim)

> A retry starts from the base and redoes all the work, even when the rejected candidate passed every check and failed only a content criterion such as a missing change description: job 20261024-161559-rxjg passed fmt, clippy and 1094 tests and was rejected only for criterion 7 (a change description listing callers), and its retry costs about 40 minutes and 3 dollars to redo the code (job 20261023-234313-h49d was the same, rejected over one CHANGELOG line). When every check passed and the unmet criteria are content criteria, let the retry start from the rejected candidate's tree and send the producer only the findings, keeping the full verification of the new candidate.

## Reading (accepted by the user)

When a rejected job's checks (fmt, clippy, tests) all passed and the only unmet criteria are content criteria, the retry starts its producer snapshot from the rejected candidate's tree. The producer's brief carries only the verifier's findings. The new candidate gets the full verification. Any other rejection still restarts from the base. The audience is the Athena operator, who pays the roughly 40 minutes and 3 dollars of redone work.

Assumptions:
- Content criteria are criteria of kind 'content' in the accepted reading. A rejection qualifies only when every check recorded for the rejected candidate passed and every unmet criterion is of that kind.
- The rejected candidate's tree is available from the candidate record (candidate_tree/candidate_commit), and the retry's base commit is the same as the original job's.
- A retry whose rejection does not qualify (a failed check, an unmet behavior criterion, no candidate tree, a failed or interrupted job) behaves exactly as today and starts from the base.
- Today's behavior: run.rs produce_and_verify builds the producer snapshot with SnapshotRepo::Base from the base, and the retry brief says 'the retry starts from the base' (briefs.rs ~951). Both change only for the qualifying case.
- The new candidate is still computed as a change list against the base and fully verified (fmt, clippy, full tests, verifier). No verification is skipped.
- The verifier's judgment of unmet criteria is read from the rejected job's verification record. The spec of what counts as 'unmet' comes from existing records, not a new model call.

## Acceptance criteria

1. [behavior] A test that retries a rejected job whose checks all passed and whose only unmet criterion is a content criterion shows the producer's working snapshot containing the rejected candidate's files, not the base's. Run: cargo test --locked -p athena-core -p athena-cli (the new test by name).
2. [behavior] A test shows that the producer brief for such a retry includes the verifier's findings and states that the work starts from the rejected candidate. It does not claim the retry starts from the base. Run: cargo test --locked -p athena-context.
3. [behavior] Tests show a retry starts from the base, as before, when a check failed, when a behavior criterion is unmet, when the rejected job has no candidate tree, and when the job failed or was interrupted. Run: cargo test --locked.
4. [behavior] A test shows the new candidate from a tree-seeded retry still goes through the full check suite and the verifier, and its change list is computed against the original base so landing works unchanged. Run: cargo test --locked -p athena-core.
5. [behavior] A test shows that a tree-seeded retry whose producer changes nothing fails as an empty candidate, as today. It does not pass for inheriting the rejected tree. Run: cargo test --locked -p athena-core.
6. [behavior] cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked all pass.
7. [content] The qualifying rule (all checks passed, only content criteria unmet) is implemented in one named function with a doc comment. Its inputs are records, not strings parsed from prose.
8. [content] The change has a CHANGELOG [Unreleased] entry and a docs/decisions.md decision. Both say what changed, list each caller or writer touched with its covering test, and record the commands run. The decision names the alternatives (always from base, always from the candidate) and says any CHANGELOG or decision text that said retries always start from the base is superseded.
9. [content] The retry-related documentation and brief text (README or docs describing retry, briefs.rs wording) no longer says every retry starts from the base.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/pin.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- README.md (modify)

## Out of scope

- Skipping or weakening any verification of the new candidate.
- Changing the retry rule for failed or interrupted jobs, or for rejections caused by failed checks or unmet behavior criteria.
- Changing the producer's send-back loop or the full-suite policy.
- Edits to athena.toml, defaults.toml or the workflows.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Each new path has passing tests, with clippy and fmt clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the snapshot contents and brief text through tests, not that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria test that the redone work is avoided and that verification still runs in full.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule is general over any all-checks-passed, content-only rejection, not the two named jobs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision and CHANGELOG entry are written for an outside reader.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: A decision with alternatives is recorded in docs/decisions.md.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the retry seeding.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: End-to-end test in crates/athena-cli/tests/retry.rs. Reject a job whose scripted producer adds a file and whose scripted verdict leaves one content criterion unmet, with all checks passing. Run `athena.retry` then `go` on the retry. The scripted producer records the file listing of its working directory from the invocation spec. The test asserts the listing holds the rejected candidate's added file. Run: `cargo test --locked -p athena-core -p athena-cli a_content_only_rejection_retries_from_the_rejected_tree`. Observes: The producer's working directory contains the rejected candidate's file, which the base lacks.
  Fails without the behavior: With the old `SnapshotRepo::Base` seeding the listing lacks the rejected file and the assertion fails.
- Criterion 2: Unit tests in briefs.rs: `producer_brief` with `retry_from` set contains the job id, each finding and unmet criterion, and text saying the work starts from the rejected candidate, and does not contain 'starts from the base' or 'snapshot of the base commit'. Without it the brief is unchanged. The reader-brief test is updated for the conditional wording. Run: `cargo test --locked -p athena-context`. Observes: The brief text with and without the seed.
  Fails without the behavior: Without the new section the findings and start-from-candidate statement are absent, and the old Working directory text claims a base snapshot.
- Criterion 3: Table tests of `qualifies_for_tree_retry` in reading.rs: failed check, unmet behavior criterion, behavior criterion with unverified commands, no candidate tree, admission violations, failed state, interrupted (Dispatched) state, no unmet criteria, no checks recorded. Each returns false and a content-only case returns true. A CLI test in retry.rs shows a rejected job with a failed check retries with a base-only listing, and a test shows a changed visible base falls back to base. Run: `cargo test --locked`. Observes: The function returns false for every non-qualifying record set, and the producer listings of those retries lack the rejected file.
  Fails without the behavior: A rule that ignored check results or criterion kind would return true on those rows and the tests would fail.
- Criterion 4: Test in crates/athena-core (run_pin_tests.rs or a new run test) for a seeded retry: the scripted verifier is invoked, the check list is recorded in verification.checks (fmt, clippy, test as configured in the fixture), the verifier snapshot is the new candidate, and `candidate.base_commit` is the retry's prediction commit with `changes` listing the rejected file as an add against the base. Landing the retry produces the file in the repository. Snapshot unit test in athena-git: the BaseThenTree `change_list()` is against the base commit. Run: `cargo test --locked -p athena-core`. Observes: Verifier role invoked, all checks recorded, change entries against the original base, and landing works.
  Fails without the behavior: If seeding skipped verification or diffed against the candidate, the verifier would not run or the change list would be empty/wrong.
- Criterion 5: Test with a scripted producer that edits nothing in a seeded retry. The job ends failed with reason 'the producer made no change', no verifier invocation. A unit test in snapshot.rs checks `unchanged_since_seed` is true for an untouched seeded work tree and false after an edit. Run: `cargo test --locked -p athena-core`. Observes: Outcome failed with the empty-candidate reason; the verifier is never called.
  Fails without the behavior: With only `changes.is_empty()` the inherited tree yields a non-empty change list and the job would be verified and could pass.
- Criterion 6: Run the three checks. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0.
  Fails without the behavior: A formatting, lint or test failure anywhere in the change.
- Criterion 7: Read reading.rs: `qualifies_for_tree_retry` is a single `pub fn` with a doc comment, taking `JobState`, `Option<&Candidate>`, `Option<&Verification>`. Search for callers. Run: `git diff`. Observes: The function signature takes records only and is the sole home of the rule; no string matching on findings.
- Criterion 8: Read the CHANGELOG [Unreleased] entry and the docs/decisions.md decision. They list each touched caller or writer with its covering test: `dispatch`, `produce_and_verify`, `Snapshot::create`, `producer_brief`, `reader_brief`, and `tree_retry_seed`. They record the commands run, name the alternatives (always from base, always from candidate), and state that earlier CHANGELOG or decision text saying retries always start from the base is superseded. Run: `git diff -- CHANGELOG.md docs/decisions.md`. Observes: Entry text with callers, tests, commands and the supersession sentence.
- Criterion 9: Search for the old claim across README.md, docs and the Rust sources, and read the changed README lines (the paragraph near line 51), the reader brief in briefs.rs and the drain.rs retry doc comment. Run: `git diff -- README.md crates/athena-context/src/briefs.rs crates/athena-cli/src/drain.rs`. Observes: No remaining text says every retry starts from the base; each says a retry starts from the base unless the rejection qualifies.

Planned files:

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base cb5757825496 → candidate 6cd492c8659c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  42 +++++
 README.md                            |   6 +-
 crates/athena-cli/src/drain.rs       |   4 +-
 crates/athena-cli/tests/lifecycle.rs |  14 ++
 crates/athena-cli/tests/retry.rs     | 346 +++++++++++++++++++++++++++++++++++
 crates/athena-context/src/briefs.rs  | 126 ++++++++++++-
 crates/athena-core/src/reading.rs    | 284 ++++++++++++++++++++++++++++
 crates/athena-core/src/run.rs        |  58 +++++-
 crates/athena-git/src/snapshot.rs    |  84 ++++++++-
 docs/decisions.md                    |  58 ++++++
 10 files changed, 1003 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1107 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
