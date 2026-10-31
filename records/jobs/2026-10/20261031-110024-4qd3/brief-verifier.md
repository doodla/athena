# Job 20261031-110024-4qd3: verify "Hold jobs on low disk and auto-resume instead of failing"

## Request (verbatim)

> try again on 20261031-095235-zq9x

## Reading (accepted by the user)

Retry of job 20261031-095235-zq9x (failed: low disk before the verifier, never verified). After idle-build cleanup, if free disk is still short before the checks or before the verifier, hold the job resumable (not failed, no retry, producer work kept); a later drain resumes it. Audience: the maintainer running drains and the verifier running cargo test.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs (guard, ATHENA_FREE_BYTES override, idle-build cleanup), crates/athena-core/src/run/mod.rs and run/verification.rs (the 'before the checks'/'before the verifier' failure sites), crates/athena-core/src/status.rs, crates/athena-cli/src/drain.rs (maps a run result to Held/attempts), and tests crates/athena-cli/tests/drain.rs, binary.rs, retry.rs. The property is measured directly through the job's end state, recorded outcome text and status output under a simulated free-bytes value. No record or data file is added under crates/, so check_record_file/AnyRecord/round-trip are unaffected; the change adds no refusal rule on readings.
- Prior job 20261031-095235-zq9x: its finding (low disk 9.4 GB free vs 10.0 GB before the verifier ended it failed) is fixed by holding instead of failing. Its candidate's 16 changed files are kept as the starting point: all listed as deliverables except none dropped; the extras display.rs, run/close.rs and test_support.rs are added as deliverables. The rebuilt change must reproduce that approach (hold_for_disk committing attempt records without Score/Outcome, shared closing-records helper) from base since attempt changes were discarded.
- Hold keeps the producer's work and uses no drain attempt; spend counts against budgets but the hold is not a work failure for introspection, failure classes or the scope guard.
- Earlier job 20261031-095208-bksh (planner budget cap) is a different class and is out of scope.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test with simulated low space (ATHENA_FREE_BYTES) after the build shows the job ends held, not Failed, with no `athena retry` needed, no verifier or producer verdict recorded, and outcome text naming `environment: low disk` with free and required amounts. It fails on pre-change code.
2. [behavior] Run `cargo test -p athena-cli --test drain`. A new test shows a later drain pass, after the simulated free space is raised, resumes the held job from its kept candidate with no new producer run and no `athena retry`, and the job finishes.
3. [behavior] Run `cargo test -p athena-cli --test drain`. A new test shows space that stays short across passes leaves the job held and uses no drain attempt.
4. [behavior] Run `cargo test -p athena-core`. Tests show a low-disk hold before the checks and before the verifier is not counted as a work failure in introspection, failure classes or the scope guard, and idle-build cleanup still runs first and lets the job proceed when it frees enough space; cleanup removes only Athena-owned finished-job directories.
5. [behavior] Run `cargo test -p athena-cli --test binary`. `athena status` shows the held job with its low-disk reason and the amounts.
6. [content] Grep of crates/athena-core/src/run/mod.rs, run/verification.rs and disk.rs shows the low-disk path no longer builds a failure with a `before the checks` or `before the verifier` message telling the person to run `athena retry`.
7. [content] CHANGELOG.md has an entry for hold-and-auto-resume; module docs in crates/athena-core/src/disk.rs and crates/athena-cli/src/drain.rs describe it instead of fail-and-retry; docs/decisions.md records the call.
8. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; all pass.
9. [content] Governance files are unchanged: athena.toml, defaults.toml, .github/workflows, Cargo.toml, Cargo.lock and rust-toolchain.toml are not in the diff.

## Expected deliverables

- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Automatic proposing after introspection (job 20261025-033738-yz2d): main.rs, introspect.rs, proposals.rs and their tests; rerun separately
- The budget-cap hold for planner, producer and verifier stages; a separate job
- Test-suite speedup and other work of job 20261025-202447-xgk9
- Guarding combined disk use of parallel jobs, or planning around disk
- Changing the [disk] min_free_gb default or config schema, or editing athena.toml, defaults.toml or workflows
- Deleting committed records or touching a running job's files in the cleanup
- Removing `athena retry` or its handling of old failed environment jobs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt to pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria exercise drain and status via cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check the held end state, resume and status output, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Low disk becomes a hold at both gates, not only the verifier gate.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by tests that fail on pre-change code.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is a job held then finishing without retry.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog and module docs updated.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 6760fca0c405 → candidate 0c6ecf39a5eb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  20 ++
 crates/athena-cli/src/display.rs           |   8 +-
 crates/athena-cli/src/drain.rs             |  31 ++-
 crates/athena-cli/tests/binary.rs          |  68 +++++++
 crates/athena-cli/tests/drain.rs           | 300 ++++++++++++++++++++++++-----
 crates/athena-core/src/disk.rs             |  93 ++++++---
 crates/athena-core/src/reading.rs          |  19 +-
 crates/athena-core/src/run/close.rs        |  86 +++++----
 crates/athena-core/src/run/mod.rs          |  56 ++++--
 crates/athena-core/src/run/tests.rs        | 183 +++++++++++++++++-
 crates/athena-core/src/run/verification.rs |  29 +--
 crates/athena-core/src/scope_guard.rs      |   6 +-
 crates/athena-core/src/state.rs            |  27 +++
 crates/athena-core/src/status.rs           |   9 +-
 crates/athena-core/src/test_support.rs     |  34 ++++
 docs/decisions.md                          |  39 ++++
 16 files changed, 839 insertions(+), 169 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` failed (192 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. All pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
