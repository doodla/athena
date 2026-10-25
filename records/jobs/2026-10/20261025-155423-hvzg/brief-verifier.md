# Job 20261025-155423-hvzg: verify "Keep work and progress note when producer hits deadline; resume on retry"

## Request (verbatim)

> try again on 20261017-044405-4zcq

## Reading (accepted by the user)

When the producer stage hits its deadline, keep the job's worktree edits and write a progress note listing changed files. Record the job as timed out, not failed. Make `athena retry` resume from the kept work with the note, and make status show timed out versus failed. Producer stage only; the audience is the Athena operator who loses 20-30 minutes per timeout.

Assumptions:
- Scope is producer stage only, per the user's replies: no verifier-stage timeouts, no planning split, no deadline length change.
- Existing record formats must stay readable.
- Deliverable paths are carried over from the earlier attempt and not re-verified in the repository.

## Acceptance criteria

1. [behavior] A test with the fake agent exceeds a short producer deadline after editing files. Afterwards the edits remain in the job worktree, a progress note listing the changed files exists, and the job is recorded as timed out, not failed. Run: cargo test.
2. [behavior] A test retries a timed-out job. The new producer starts from the kept worktree (not a clean checkout) and is given the progress note naming the earlier changed files. Run: cargo test.
3. [behavior] Status output distinguishes a timed-out job from a failed one, and the deadline reason (e.g. 'deadline of 1800s reached') is kept in the record. Run: cargo test.
4. [behavior] A test shows existing records with killed or failed outcomes still deserialize unchanged, and a job that finishes in time behaves as before. Run: cargo test.
5. [content] The new timed-out outcome is documented and CHANGELOG.md mentions the change.
6. [behavior] cargo test, cargo clippy and cargo fmt --check all pass with no new warnings. Run each.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/invocation.rs (modify)

## Out of scope

- Verifier-stage timeouts
- Splitting multi-part requests in planning
- Changing the deadline length
- Pre-dispatch size checks

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy, fmt to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check worktree edits, note, and recorded outcome in tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the first step: keep and resume.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New lifecycle test. The fake producer script edits two files and returns `InvocationOutcome::TimedOut` with detail 'deadline of 1800s reached'. After `go`, the test reads the head records. Run: `cargo test --locked -p athena-cli --test lifecycle a_producer_that_hits_its_deadline_keeps_its_work_and_closes_timed_out`. Observes: The outcome record state is `timed_out`, not `failed`. A `progress-note.md` record in the job's records lists both edited paths. The kept directory `<state>/kept/<job>/work` holds both edited files.
  Fails without the behavior: The producer's TimedOut outcome would close the job as Failed, with no note and no kept directory, because the work tree is deleted by clean_job.
- Criterion 2: New lifecycle test. After the timeout above, run `athena.retry(job)` then `go`. The second script records, inside the producer call, whether the edited files exist in `spec.workdir` and whether the brief text contains the note and the changed paths. A second case changes a base file first and asserts the retry starts clean. Run: `cargo test --locked -p athena-cli --test lifecycle retrying_a_timed_out_job_starts_from_the_kept_work_with_the_note`. Observes: The files are present in the new work directory before the producer edits anything. The brief has the 'Resume' section naming the earlier changed files. The new job is a different job. With a moved base, the files are absent and the brief has no Resume section.
  Fails without the behavior: A clean checkout would be seeded with no kept files, and the brief would carry no note.
- Criterion 3: New test. It closes one job as timed out and one as failed (using the existing killed_producer scenario), then renders status or the job listing for each. Run: `cargo test --locked -p athena-cli --test lifecycle status_shows_timed_out_apart_from_failed`. Observes: The timed-out job prints `timed_out` (or 'timed out') and the failed job prints `failed`. The outcome reason and the invocation detail both contain 'deadline of 1800s reached'.
  Fails without the behavior: The status text would show `failed` for both, or the reason would be lost.
- Criterion 4: New unit test in records.rs deserializes literal JSON with `"outcome":"killed"` (invocation) and `"state":"failed"` (outcome), and round-trips `timed_out`. The existing killed-invocation and kill-switch lifecycle tests, plus the happy-path lifecycle tests, must pass unchanged. Add one drain.rs test if drain's retry flag needs it. Run: `cargo test --locked -p athena-data records::tests::old_killed_and_failed_outcomes_still_deserialize && cargo test --locked -p athena-cli --test lifecycle a_killed_invocation_is_recorded_with_bounds`. Observes: Old values parse to Killed and Failed. A Killed producer still closes Failed. A job that finishes in time still lands with the same records.
  Fails without the behavior: A rename or reshape of the enums would break the old-record parse. Treating every Killed as a timeout would turn the existing Killed/Failed expectations into TimedOut.
- Criterion 5: The doc is the `OutcomeState::TimedOut` and `InvocationOutcome::TimedOut` rustdoc plus a CHANGELOG.md Unreleased entry. A small test in lifecycle.rs or records.rs asserts that CHANGELOG.md contains 'timed out', if the repo already tests changelog content. Otherwise the verifier reads the entry with git diff. Run: `cargo test --locked`. Observes: The Unreleased section of CHANGELOG.md names the timed-out outcome, the kept work, the progress note and the retry resume. The enum doc comments describe the outcome.
- Criterion 6: Run the three checks Athena runs, in that order. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All pass with no warnings. That includes the budgets test `system_facts_budgets_match_computed`, which stays green because PRODUCER_FRAME and the always-loaded prompt text are not edited.
  Fails without the behavior: A missed match arm for the new variants, or an unformatted file, would fail the build, clippy or fmt.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## The change

Base 8461f13fb8e3 → candidate 72b8ddd26a3a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  13 ++
 crates/athena-agent/src/claude.rs    |  15 +-
 crates/athena-agent/src/fake.rs      |   5 +-
 crates/athena-agent/src/procs.rs     |   5 +
 crates/athena-cli/src/drain.rs       |   7 +-
 crates/athena-cli/tests/drain.rs     |  18 +++
 crates/athena-cli/tests/lifecycle.rs | 256 +++++++++++++++++++++++++++++++++++
 crates/athena-context/src/briefs.rs  |  36 ++++-
 crates/athena-core/src/reading.rs    |   8 +-
 crates/athena-core/src/run.rs        | 151 +++++++++++++++++++--
 crates/athena-core/src/state.rs      |  12 ++
 crates/athena-data/src/records.rs    |  25 +++-
 crates/athena-git/src/snapshot.rs    |  87 ++++++++++++
 docs/enforcement.md                  |   4 +-
 docs/records.md                      |   9 +-
 15 files changed, 626 insertions(+), 25 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1239 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
