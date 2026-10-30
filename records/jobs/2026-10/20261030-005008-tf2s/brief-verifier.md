# Job 20261030-005008-tf2s: verify "Retry A3: in-flight table recording live work tree and checks stage"

## Request (verbatim)

> try again on 20261029-142853-x4nr

## Reading (accepted by the user)

Retry job 20261029-142853-x4nr (build job A3, D145): add the local in-flight table (job id, stage, worktree) in state.rs, written by the drain at each stage change and read at drain start, fixing the rejected candidate's major finding that RealSteps::worktree returned the kept-work directory instead of the live work tree. Audience: the drain's later crash-recovery jobs and the verifier. max_in_flight = 1 behaves as today.

Assumptions:
- Code survey: read crates/athena-core/src/state.rs (StateDir::job_dir at line 52, kept_work_dir at line 58 = <root>/kept/<id>/work) and crates/athena-core/src/run/producer.rs:87 (live work tree = job_dir.join("work")); drain entry point is drain in crates/athena-cli/src/drain.rs with the Steps trait and RealSteps; existing CLI tests in crates/athena-cli/tests/drain.rs. The in-flight file is a plain local state file under the state root, not a record, so check_record_file and AnyRecord do not apply; the round-trip is covered by state.rs unit tests. The property (recorded path equals the producer's live directory) is measured directly by comparing the table row with the path the real producer code uses, not through a stub.
- Job 20261029-142853-x4nr's candidate is dropped and rebuilt from the base; its design is reused: InFlightStage/InFlightEntry/InFlightTable (read, set, remove) in state.rs, in-flight.json in the state root written via temp file and rename, defaulted Steps methods in_flight_table and worktree, a row removed when the run fails or landing finishes, leftover rows reported at start, missing file = empty, corrupt file = note.
- Findings of 20261029-142853-x4nr addressed: (major) RealSteps::worktree now returns the live work tree via a StateDir helper (job_dir(id).join("work")) that producer.rs also uses; (minor) the checks stage is written so the table distinguishes producer, checks and landing; (minor) a corrupt table is not silently replaced: set/remove keep the bad file by renaming it aside before writing, and I/O errors are not treated as an empty table; (minor) the design doc gets a note in section 11 that build job 3 is implemented.
- Job 20261029-142849-04d9 (A2, [drain.pipeline] config) stays as landed and is not touched.
- Prior art tried for this class (a test of recorded state through the real implementation) is reused here, and made specific: the test must compare the recorded path with the directory the real producer code uses.

## Acceptance criteria

1. [behavior] A test using the real steps implementation (RealSteps), not a stub, shows the table row's worktree equals the producer's live work tree (the StateDir job_dir/work path that run/producer.rs uses) while the producer stage runs, and is not the kept/<id>/work path. Run: cargo test -p athena-cli drain.
2. [behavior] A test shows a timed-out job whose work is kept records the live work tree while it runs, and the kept-work directory is not used in the table. Run: cargo test -p athena-cli drain.
3. [behavior] A test calls drain with test Steps and max_in_flight = 1 and shows the in-flight file holds job id, stage and worktree at the producer, checks and landing stages, and no entry after the job closes. Run: cargo test -p athena-cli drain.
4. [behavior] Tests pre-write an in-flight file with a leftover entry and show it is read and reported at drain start; a missing file and a corrupt file do not fail the drain, and a corrupt file is kept (renamed aside) rather than silently overwritten. Run: cargo test -p athena-cli and cargo test -p athena-core state.
5. [behavior] state.rs unit tests show the table round-trips, updates a stage in place, removes an entry, and writes atomically. Run: cargo test -p athena-core.
6. [behavior] No existing test in drain.rs is modified or deleted (git diff), and cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace pass.
7. [content] CHANGELOG.md has a new entry describing the in-flight table (job id, stage, worktree; written at stage changes; read at drain start; unchanged at max_in_flight = 1). Check with git diff CHANGELOG.md.
8. [content] docs/design/async-drain.md section 11 notes that build job 3 is implemented. The diff touches only drain.rs, state.rs, CHANGELOG.md, the design doc, and tests in crates/athena-cli/tests/drain.rs if needed; athena.toml, defaults.toml and workflows are unchanged. New public items have doc comments for an outside reader.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/state.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Resuming or acting on leftover table entries (producer retry, landing re-check): later build jobs
- Running more than one producer at once, or raising max_in_flight to 3 (build job 10)
- Reading [drain.pipeline] values in the drain runtime
- Fixing A2's findings in config.rs (job 20261029-142849-04d9 work is kept as landed)
- Changes to athena.toml, defaults.toml, .github/workflows
- Disk-guard, check-slot, read-ahead, conflict-return behavior

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the recorded path through the real steps, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the work tree path at its source with a shared StateDir helper.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the named cargo commands.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the drain and state files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base abb7aeef4d78 → candidate f05c367fcc8c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   7 +
 crates/athena-cli/src/drain.rs         | 133 ++++++++++++++-
 crates/athena-cli/tests/drain.rs       | 288 +++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/producer.rs |   2 +-
 crates/athena-core/src/state.rs        | 267 ++++++++++++++++++++++++++++++
 docs/design/async-drain.md             |   9 ++
 6 files changed, 702 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1687 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No existing test in drain.rs is modified or deleted (git diff), and cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
