# Job 20261025-031612-6239: verify "Stop writing Athena trailers in commits after C0 (D111)"

## Request (verbatim)

> Priority: high
> Stop writing Athena commit-message trailers in new commits after C0 (decision D111, job 7 part 1b), using the fixture from 20261025-031609-k0tt. Depends on 20261025-031609-k0tt. Retries the design of job 20261025-022144-emm8, rejected for its fixture.
> Acceptance:
> - On the fixture, a new job branch holding an intervention commit, a records-only commit and the job's landing is landed through land_pr with a squash merge; the squash commit on main and the three pull-request commits (reached from the records branch) carry no Athena- lines, the landing record classifies them, and athena check --history and --attribution pass.
> - A test covers retry_landing for a blocked job after C0 (no trailer search, no Athena-Verified-Content).
> - athena status's trace and the auditor read a post-C0 job through their real entry points.
> - With no cutover_job, trailers are written as before and existing tests pass.
> Files: crates/athena-core/src/commits.rs, crates/athena-core/src/gh.rs, crates/athena-core/src/run.rs, crates/athena-core/src/status.rs, crates/athena-core/src/auditor.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

When [history] cutover_job is set, new commits after C0 (intervention, records-only, landing, squash and pull-request commits) carry no Athena- trailers. Tests use the k0tt cutover fixture (test_support.rs) and cover a landing through land_pr, retry_landing after C0, and the status trace and auditor entry points. With no cutover_job, behavior is unchanged. The audience is the maintainer and the verifier, who run the tests.

Assumptions:
- Job k0tt has landed, so test_support.rs and its CutoverFixture exist. I confirmed they are in the repository.
- Decision D111 defines C0 as the host commit of the cutover job's landing record. Whether a commit is post-C0 is read from the config at the relevant commit.
- The athena-git helpers that strip lines (squash_text_relaxed) may need a change, as the earlier emm8 attempt showed.
- The fixture's fake host squash message has no '(#n)' suffix. Tests must account for that.

## Acceptance criteria

1. [behavior] A test on the cutover fixture lands a new job branch (intervention commit, records-only commit, job landing) through land_pr with a squash merge. The squash commit on main and the three pull-request commits reached from the records branch have no Athena- lines. The landing record classifies them. check --history and check --attribution report no violations. Run: cargo test -p athena-core.
2. [behavior] A test covers retry_landing for a blocked job after C0: no trailer search, no 'closing commit not found' bail, and no Athena-Verified-Content written. Run: cargo test -p athena-core retry_landing.
3. [behavior] Tests call athena status's trace and the auditor through their real entry points on a post-C0 job and get correct results. Run: cargo test.
4. [behavior] With no cutover_job configured, trailers are written as before. All existing tests pass, cargo clippy is clean and cargo fmt --check passes. Run: cargo test, cargo clippy, cargo fmt --check.
5. [content] CHANGELOG.md has an entry, and docs/decisions.md records the D111 trailer-stop design, the alternatives and the retry_landing post-C0 behavior. It removes the stale 'retry_landing has no test' statement.
6. [content] The fixture is the k0tt helper, with a job landed the old way through the real writers. No hand-built records are substituted.

## Expected deliverables

- crates/athena-core/src/commits.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-git/src/lib.rs (modify)

## Out of scope

- Job 7 part 2.
- Rewriting history or changing pre-C0 commits.
- Changes to athena.toml or defaults.toml.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Each changed reader and writer has a test. cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise real entry points: land_pr, retry_landing, the status trace and the auditor.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs the tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Trailer writing is gated at the commit writers, not patched per call site.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the trailer writers and their readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A core test on the CutoverFixture using real writers: Athena::intervene (with staged changes, then without), ask/go for the job landing, then land_through_pr (land_pr with the FakeHost squash). Run: `cargo test -p athena-core post_c0_landing_through_land_pr`. Observes: The landing record's commits are [DeclaredIntervention, RecordsOnly, JobLanding]. The squash commit on remote main (no '(#n)' suffix) and the three PR commits reached from athena/records have no line starting 'Athena-'. The functions behind check --history and check --attribution report no violations.
  Fails without the behavior: Without the writer strip the squash and PR commits carry Athena- lines. Without the gate content rule the intervene and reading commits are refused ('the gate refused Athena's own commit'). Without the relaxed squash text and the records-based landed-commit lookup, land_pr refuses before merging.
- Criterion 2: Block a post-C0 job with a conflicting local edit, resolve it, call go (retry_landing), then land the retried job through land_through_pr. A paired test with no cutover_job. Run: `cargo test -p athena-core retry_landing`. Observes: go returns Landed with no 'closing commit ... was not found' error. The new commit has no Athena- line (so no Athena-Verified-Content) and holds outcome-landed.json. landed_commit returns the retry's landing commit. The no-cutover counterpart still writes trailers.
  Fails without the behavior: retry_landing bails 'the blocked closing commit of job ... was not found', or writes Athena-Verified-Content.
- Criterion 3: On the post-C0 job from criterion 1, call Athena::trace. In a fresh clone of the remote (main and athena/records fetched, candidate commit absent) first assert the old trailer-only lookup fails with 'none could be read', then call auditor::run and assert it succeeds. Run: `cargo test -p athena-core post_c0_job_is_read_by`. Observes: The trace lists the job's commits including the host squash commit, with state and outcome. The auditor reads the changed files from the squash commit and records an audit.
  Fails without the behavior: The trace's commits is empty for the post-C0 job, and the auditor refuses 'none could be read'.
- Criterion 4: Run the full suite, which includes the commits.rs unit tests comparing a built message with and without cutover_job, the gate tests, and the athena-git tests. Then clippy and fmt. Run: `cargo test --locked`. Observes: With no cutover_job all trailers are kept and the gate behaves as before, and the existing suite passes. Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`; both are clean.
  Fails without the behavior: Without the cutover_job condition the strip would remove trailers pre-C0, and tests that read trailers (athena-cli/tests/binary.rs) and the gate refusal tests would fail.
- Criterion 5: Read the diffs of the two docs files. Run: `git diff CHANGELOG.md docs/decisions.md`. Observes: CHANGELOG has an Unreleased entry naming the trailer stop and the readers and gate changed. The D111 addendum names build_commit_in, the config at the parent, the gate content rule and why it was chosen over running the gate before the strip, the replay note, the rejected alternatives, and the retry_landing post-C0 behavior (no trailer search, no Athena-Verified-Content). The stale 'retry_landing has no test' text is removed if found. Otherwise the change description says it was not found.
- Criterion 6: Inspect the test_support.rs diff and the new tests for hand-built records or commits. Run: `git diff crates/athena-core/src/test_support.rs`. Observes: The fixture comes from CutoverFixture::build with a new CutoverSpec (old-way job and cutover job through the real writers). Later jobs use Athena::intervene, ask/go and land_through_pr. There is no new hash_object/commit_tree use beyond the existing FakeHost squash.

Planned files:

- crates/athena-core/src/commits.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base 646476fc47f8 → candidate 45afcdf7e823. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   6 +
 crates/athena-cli/tests/common/mod.rs   |   8 +-
 crates/athena-cli/tests/trailer_stop.rs | 355 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/auditor.rs       |  68 +++++-
 crates/athena-core/src/commits.rs       |  52 ++++-
 crates/athena-core/src/gh.rs            |  48 ++++-
 crates/athena-core/src/manage.rs        |   3 +
 crates/athena-core/src/run.rs           |  66 +++---
 crates/athena-core/src/run_pin_tests.rs |   6 +-
 crates/athena-core/src/status.rs        |  27 ++-
 crates/athena-core/src/test_support.rs  |   7 +-
 crates/athena-gate/src/invariant.rs     |  83 +++++++-
 crates/athena-git/src/lib.rs            |  67 ++++++
 docs/decisions.md                       |  48 +++++
 14 files changed, 805 insertions(+), 39 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1335 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: With no cutover_job configured, trailers are written as before. All existing tests pass, cargo clippy is clean and cargo fmt --check passes. Run: cargo test, cargo clippy, cargo fmt --check.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
