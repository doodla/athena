# Job 20261029-142853-x4nr: verify "Add in-flight table written at stage changes, read at drain start"

## Request (verbatim)

> Priority: urgent
> Async drain build job A3 (docs/design/async-drain.md, decision D145). Add the in-flight table (job 3): a local file of job id, stage and worktree, written at each stage change and read at drain start. Depends on 20261029-142849-04d9. With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/state.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A3 (D145, design section 7 crash bullet): add a local in-flight table file (job id, stage, worktree path) in athena-core state.rs, written by the drain at each stage change and read at drain start. Follows job 20261029-142849-04d9 (A2, [drain.pipeline] config), whose result is kept and unchanged; with max_in_flight = 1 drain behaves as today.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (entry points drain and drain_concurrent -> drain_with, which reads the queue then runs run_serial/run_parallel; Steps trait, Drain struct) and crates/athena-core/src/state.rs (StateDir with root, job_dir, kept_work_dir; ResumeStage/ResumeMark). The table is a new type in state.rs beside StateDir, stored as a file under the state root; no record or data file is added under crates/, so the records allow-list, check_record_file and AnyRecord are not touched. The behavior is measured directly by calling drain/drain_concurrent with test Steps and reading the file back.
- Job 20261029-142849-04d9 (A2): its landed changes to config.rs, CHANGELOG.md and docs/design/async-drain.md are kept and not edited here. Its findings: the stale line 21 in the design doc (minor), the unhelpful DrainPipeline error text (minor) and the MAX_CHECK_SLOTS_CEILING nit were all in A2's own files; they are left to a separate job and do not affect A3, which does not read the [drain.pipeline] values.
- Stages recorded follow the design: at least producer and landing (plus checks); an entry is removed when a job closes. Atomic write via temp file and rename; a missing file means an empty table, a corrupt file is reported as a note, not a crash.
- Read at drain start means loaded and surfaced (a note or line listing leftover entries, and entries kept in memory); acting on them to resume jobs is a later build job. Drain does not change decisions from the table, so existing behavior is preserved.
- Not a refusal rule: nothing newly refuses readings, so no refusal-rate criterion is needed.

## Acceptance criteria

1. [behavior] A test calls drain (the real entry point) with test Steps and max_in_flight = 1 and shows the in-flight file holds the job id, stage and worktree for the job at each stage change, and no entry for it after the job closes. Run: cargo test -p athena-cli drain.
2. [behavior] A test pre-writes an in-flight file with a leftover entry, starts drain, and shows the entry is read at start and reported; a missing file and a corrupt file are both handled without failing the drain. Run: cargo test -p athena-cli and cargo test -p athena-core state.
3. [behavior] state.rs unit tests show the table round-trips (write then read gives the same job id, stage, worktree), updates a job's stage in place, and removes an entry, with an atomic write. Run: cargo test -p athena-core.
4. [behavior] All existing drain tests pass unchanged: git diff shows no existing test in drain.rs modified or deleted, and cargo test passes.
5. [behavior] cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace all pass.
6. [content] CHANGELOG.md has a new entry describing the in-flight table (file with job id, stage, worktree; written at stage changes; read at drain start; behavior unchanged at max_in_flight = 1). Check with git diff CHANGELOG.md.
7. [content] The diff touches only crates/athena-cli/src/drain.rs, crates/athena-core/src/state.rs and CHANGELOG.md (plus athena.toml, defaults.toml and workflows unchanged), and the new public items have doc comments written for an outside reader.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/state.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)

## Out of scope

- Resuming or acting on leftover table entries (producer retry, landing re-check): later build jobs
- Running more than one producer at once, or raising max_in_flight to 3 (build job 10)
- Reading [drain.pipeline] values in the drain runtime
- Fixing A2's findings in config.rs and the design doc (job 20261029-142849-04d9 work is kept as landed)
- Changes to athena.toml, defaults.toml, .github/workflows
- Disk-guard, check-slot, read-ahead, conflict-return behavior

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust in the two named crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the file contents through drain, not that types exist.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the three named files.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog written for outside readers.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: File holds only job ids, stages and paths; no secrets.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 12500fe01aad → candidate 90e92066339d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                    |   6 +
 crates/athena-cli/src/drain.rs  | 292 +++++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/state.rs | 187 +++++++++++++++++++++++++
 3 files changed, 484 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1682 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
