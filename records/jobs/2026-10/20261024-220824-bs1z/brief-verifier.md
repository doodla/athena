# Job 20261024-220824-bs1z: verify "Add incremental attribution checking with checkpoint records"

## Request (verbatim)

> Build job 6 of the records-history plan (docs/design/records-history.md section 9, decision D111): incremental checking. After a passing local attribution check (athena check --attribution, and athena land after a landing record), write a checkpoint record to athena/records with the records writer, naming the last passing main commit, the records-branch tip it was judged against, and the check's version; a later check judges only main's commits and records added since the newest checkpoint. A full-replay mode ignores checkpoints, judges everything from the start, and fails if it disagrees with the newest checkpoint. CI's token is read-only, so CI never writes checkpoints: its attribution job reads the newest one and runs incrementally on pushes, and as a full replay on its weekly schedule. Each run reports seconds taken and the number of commits and records judged. Tests: an incremental run judges only the new commits; a tampered checkpoint (naming a commit the full replay fails) is caught by the replay; a check-version change forces a full replay; a failing check writes no checkpoint. Timing: a producer's working copy has no history of this repository, so time a full replay and an incremental run on a synthetic repository of about 200 landings in a test or bench and report them as measured on that fixture; the timing on this repository is taken after landing, outside this job. This repeats job 20261024-220048-engm, which stopped because its request asked for timing here.

## Reading (accepted by the user)

Implement job 6 of the records-history plan (D111). A passing local `athena check --attribution` (and `athena land` after its landing record) writes a checkpoint record. Later checks judge only newer commits and records. A full-replay mode ignores checkpoints and fails on disagreement. CI reads checkpoints only, and the weekly run is a full replay. Timing is measured on a synthetic ~200-landing fixture, not on this repository. The audience is the maintainer and the CI attribution job.

Assumptions:
- The Checkpoint record type already exists in athena-data history.rs (main_sha, records_sha, verifier_version), and the records writer exists.
- audit_history in records_branch.rs has no range form, so a range or since-checkpoint audit is added there.
- The user has accepted the synthetic-fixture timing; timing on this repository is out of scope.
- CI is detected through the existing environment or a flag and forces read-only behaviour, with the workflow's permissions staying read-only.
- The newest checkpoint is chosen by records-branch order, and no checkpoint is written past a pending commit.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows an incremental run after a checkpoint judges only the commits and records added since it, with the reported counts equal to the new ones.
2. [behavior] Run `cargo test`: a tampered checkpoint that names a commit the full replay fails is caught by the full replay, which exits with failure.
3. [behavior] Run `cargo test`: a change in the check version forces a full replay, and the report says why.
4. [behavior] Run `cargo test`: a failing attribution check writes no checkpoint, and the records-branch tip is unchanged.
5. [behavior] Run `cargo test`: a passing `check --attribution` and a passing `land` each write a checkpoint naming the last passing main commit, the records tip judged against, and the check version.
6. [behavior] Run `cargo test`: the full-replay mode ignores checkpoints and judges everything, and a CI or read-only mode never writes a checkpoint.
7. [behavior] Run `cargo test`: each run output reports seconds taken and the number of commits and records judged.
8. [behavior] Run `cargo test`: a test or bench on a synthetic repository of about 200 landings times a full replay and an incremental run and prints both numbers, marked as measured on that fixture.
9. [content] .github/workflows/ci.yml keeps the token read-only. Pushes run the incremental check and the weekly schedule runs the full replay.
10. [content] The CHANGELOG entry describes the feature and the fixture timings, marked as measured on the fixture. It states that the timing on this repository is taken after landing.
11. [behavior] `cargo clippy` and `cargo fmt --check` are clean, and `cargo test` passes across the workspace.

## Expected deliverables

- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-data/src/history.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## Out of scope

- Timing on this repository itself
- Other jobs of the records-history plan
- Changes to the records branch format beyond using the existing checkpoint record

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each required behavior; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests to see counts, tamper detection, and no-write on failure.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: CI read-only is enforced by the workflow permission and by code.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to incremental checking.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Timings are measured on the fixture and labeled.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Fixture with 3 attributed landings, a passing run writes a checkpoint, 2 more landings are added, and a second run follows. A companion test, intent_before_checkpoint_landing_after_passes, writes an intent, writes a checkpoint, then merges and lands, and the incremental run must pass. Run: `cargo test --locked -p athena-gate attribution::tests::incremental_run_judges_only_new_commits`. Observes: The second run is incremental. Commits judged is 2 and records judged equals the files the 2 landings added, lower than a full replay. The intent-before-checkpoint case passes with no LandingMismatch.
  Fails without the behavior: A run ignoring the checkpoint reports all counts, so the equality assertion fails. A delta-only intent lookup gives LandingMismatch in the companion test.
- Criterion 2: Hand-write a checkpoint naming a commit the replay fails, then run the full replay in the gate and through the binary. Run: `cargo test --locked -p athena-gate attribution::tests::tampered_checkpoint_is_caught_by_full_replay && cargo test --locked -p athena-cli --test binary full_replay_tampered_checkpoint_exits_2`. Observes: CheckpointDisagrees finding, passed() false, exit status 2.
  Fails without the behavior: A replay that trusted the checkpoint would not report the bad commit, so there is no finding and the exit is 0.
- Criterion 3: Write a checkpoint with version A, then run with Options version B. Run: `cargo test --locked -p athena-gate attribution::tests::check_version_change_forces_full_replay`. Observes: Full replay, counts equal the whole history, and the reason text names both versions.
  Fails without the behavior: The run stays incremental with low counts and no reason.
- Criterion 4: Fixtures with an unattributed commit and with a pending commit, writing enabled. Run: `cargo test --locked -p athena-gate attribution::tests::failing_check_writes_no_checkpoint`. Observes: The report fails, checkpoint_written is None, and the records tip is identical before and after.
  Fails without the behavior: A writer not gated on passed() moves the records tip, so the sha-equality assertion fails.
- Criterion 5: Gate test, CLI helper test for the land path, and a binary test with CI removed from the environment. A further test, checkpoint_write_failure_keeps_verdict, covers a failed local append. Run: `cargo test --locked -p athena-gate attribution::tests::passing_check_writes_checkpoint && cargo test --locked -p athena-cli checkpoint_after_landing && cargo test --locked -p athena-cli --test binary check_attribution_writes_checkpoint`. Observes: The parsed Checkpoint has the last passing main sha, records_sha equal to the tip judged against, and CHECK_VERSION. The tip advances by one commit. On append failure the verdict and exit code are unchanged and the output says the checkpoint was not written.
  Fails without the behavior: No checkpoint record exists after a pass, so reading it fails.
- Criterion 6: A valid checkpoint is present. Compare full-replay counts with the incremental run, then run read-only and with CI=true. Run: `cargo test --locked -p athena-gate attribution::tests::full_replay_ignores_checkpoints_and_read_only_never_writes && cargo test --locked -p athena-cli --test binary check_attribution_in_ci_writes_nothing`. Observes: Replay counts equal the whole history. The records tip is unchanged in the read-only cases.
  Fails without the behavior: A replay using the checkpoint shows lower counts, and a writer ignoring read-only moves the tip.
- Criterion 7: Run the binary on a fixture in text and --json. Run: `cargo test --locked -p athena-cli --test binary check_attribution_reports_seconds_and_counts`. Observes: The output has mode, seconds, commits judged and records judged with the expected numbers, and the JSON has matching keys.
  Fails without the behavior: The assertions on seconds and counts fail.
- Criterion 8: Generate 200 landings, time a full replay, write a checkpoint, add 1 landing, time an incremental run, and print both. Run: `cargo test --locked -p athena-gate attribution::tests::timing_on_synthetic_200_landings -- --nocapture`. Observes: stdout 'measured on the synthetic 200-landing fixture: full replay X s (N commits, M records); incremental Y s (1 commit, 2 records)'. The test asserts counts only, never seconds.
  Fails without the behavior: If incremental mode re-judged everything, the asserted incremental counts would fail. If the printout were missing, the labelled figures would not appear.
- Criterion 9: Read the diff. Run: `git diff -- .github/workflows/ci.yml`. Observes: Top-level permissions stay contents: read with no write. Push runs athena check --attribution and the schedule runs athena check --attribution --full-replay.
- Criterion 10: Read the entry. Run: `git diff -- CHANGELOG.md`. Observes: It describes checkpoints, incremental mode, full replay and CI behaviour, and quotes the fixture timings labelled as measured on the fixture. It states that the timing on this repository is taken after landing.
- Criterion 11: Run the three Athena checks together. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0 across the workspace.
  Fails without the behavior: Any formatting difference, clippy warning or failing test makes it exit non-zero.

Planned files:

- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-data/src/history.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- .github/workflows/ci.yml (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## The change

Base 42ead8014432 → candidate 79dd5bcb9602. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml                |  11 +-
 CHANGELOG.md                            |  20 +
 crates/athena-cli/src/main.rs           | 297 +++++++++++++-
 crates/athena-cli/tests/binary.rs       | 228 +++++++++++
 crates/athena-gate/src/attribution.rs   | 690 +++++++++++++++++++++++++++++++-
 crates/athena-git/src/records_branch.rs | 147 ++++++-
 docs/design/records-history.md          |  30 ++
 7 files changed, 1408 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1158 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
