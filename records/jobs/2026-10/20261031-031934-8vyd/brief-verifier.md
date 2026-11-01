# Job 20261031-031934-8vyd: verify "Re-check rebased candidates only when the rebase touched their paths"

## Request (verbatim)

> Priority: urgent
> Async drain build job A7 (docs/design/async-drain.md, decision D145). Re-check rule after rebase (job 7): re-run checks only when the rebase touched paths the candidate changed, and record the share of rebases that needed it. With max_in_flight = 1 drain behaves as today.
> Depends on 20261030-013555-f5qm
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A7 (D145) of async drain: after the ordered landing queue rebases a verified candidate onto the latest main, the drain re-runs checks only when the rebase touched paths the candidate changed, and records the share of rebases that needed a re-check. With max_in_flight = 1 drain behaves as today. Follow-up to failed job 20261030-013555-f5qm (A6); its work is not kept as a candidate and its findings are addressed below. Audience: Sri and the verifier.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (module docs, a LandingQueue field at about line 1018, the Drain landing lock) and crates/athena-core/src/gh/landing.rs (the Rebase enum with Current, Moved, Conflict, Unchecked, and plan_rebase). The entry point is the drain loop and drain_concurrent; the rebase result is judged after verification and before the merge. The new decision is a pure function over (paths changed by the candidate, paths changed by the rebase onto the new main). No new record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The share is measured directly from counted rebases (needed re-check / total rebases) and printed in the drain summary.
- Existing code: crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/landing.rs and docs/design/async-drain.md (section on the re-check rule, build job 7) are extended and reused, not created.
- Job 20261030-013555-f5qm (A6) failed in plan critique with no candidate, so nothing from it is kept. It is a dependency: if the landing queue and Rebase handling are already in the tree (they appear to be: LandingQueue in drain.rs, Rebase in landing.rs), A7 builds on them. Its findings are addressed as follows. (1) [parallel] max_concurrency > 1 runs with max_in_flight = 1 and lands group jobs one after another onto a moved main: A7 must keep that path landing as today; a test through drain_concurrent with max_concurrency 2 pins it, and a rebase that touches none of the second job's paths does not trigger a re-check. (2) The old job after drop_landed and (3) the Rebase to Rebased mapping, (4) thrash handling and (5) size are A6 matters, not A7's; they apply only if the tree lacks them, in which case the producer reports it instead of building them. The mapping from rebase outcome to re-check is a small pure function with a unit test per arm.
- A rebase that changes no path the candidate changed lands without re-running checks. A rebase that touches an overlapping path re-runs checks before landing. Governed-file fingerprint handling stays as the existing merged-content check in land_pr decides.
- The share is reported in the drain summary output and kept in-memory for the run. No new persisted record is added.

## Acceptance criteria

1. [behavior] Tests in crates/athena-cli/src/drain.rs drive the drain through its real entry point (drain or drain_concurrent) with a landing whose rebase touches a path the candidate changed, and show checks re-run before landing. Run cargo test -p athena-cli.
2. [behavior] A test through the same entry point shows a rebase touching only paths the candidate did not change lands with no check re-run, and the drain summary reports the share of rebases needing a re-check (e.g. 1 of 2) as a number. Run cargo test -p athena-cli.
3. [behavior] With max_in_flight = 1 the existing drain tests pass unchanged: git diff shows no existing test removed or edited, and cargo test passes. A new drain_concurrent test with max_concurrency 2 pins that the second job of a group still lands as today.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
5. [content] CHANGELOG.md has an entry describing the path-scoped re-check after rebase and the recorded share of rebases that needed it, citing D145 and build job 7.
6. [content] Governance files (athena.toml, defaults.toml, .github/workflows) are unchanged; no rule that refuses is newly added beyond re-running checks, so no refusal-rate figure applies.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- docs/design/async-drain.md (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Raising max_in_flight above 1 or running several producers at once (build job 10)
- Disk guard, check slots and read-ahead
- Recovery of leftover in-flight rows
- Changes to athena.toml, defaults.toml, .github/workflows, crates/athena-data/src/config.rs
- Rebuilding the A6 landing queue, send_back and thrash stop if they are absent; the producer reports that instead
- Changes to crates/athena-core/src/state.rs and crates/athena-core/src/run/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy -D warnings criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise the drain entry point, not wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to drain.rs and the changelog.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the share is recorded and checks re-run only on overlap.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration tests in a new `mod rebase_recheck`, plus unit tests of the pure function. (a) Overlap: scripted `Rebasing` steps answer `rebase` with `Moved` whose candidate and touched lists both hold `src/a.rs`, then `drain_concurrent(steps, limits, &Quiet, 1)` runs. (b) Failed re-check kept through the retry: the first `rebase` answer is Moved with overlap and the first `rerun_checks` fails; every later `rebase` answer is `Unchanged`. A passing variant lets the second `rerun_checks` pass. (c) A re-check that fails twice ends the run with `Stop::RepeatedFailure`. Unit tests in drain.rs `tests` pin `rebase_needs_recheck` per arm: exact path, a directory holding a path, root Cargo.toml, root Cargo.lock, `crates/x/Cargo.toml`, no overlap, only shared append-only files, and an empty candidate list with a non-shared touched path. Run: `cargo test -p athena-cli --test drain rebase_recheck && cargo test -p athena-cli --lib drain::tests::rebase`. Observes: (a) The log reads 'rebase a', 'rerun a', 'land a' in that order; report.landed is 1 and report.rebases is {total: 1, rechecked: 1}. (b) With the failing re-check the log has 'rerun a' twice and never 'land a', and the job line says 'not landed: checks failed after rebase'; report.landed is 0 although the second `rebase` answered Unchanged. In the passing variant the log is 'rerun a' (fail), 'rerun a' (pass), 'land a'. (c) The stop is RepeatedFailure. Each unit arm passes.
  Fails without the behavior: Without the re-check call in `Drain::land`, 'rerun a' is never logged and the order assertion fails. Without the `owing` set, the retry lands (the log shows 'land a' after the failed re-check), so the never-lands assertion fails. Without the overlap rule, a path both sides share triggers no re-check. Without the any-depth manifest rule, the `crates/x/Cargo.toml` unit arm fails.
  Test seam: Production entry points `drain::drain_concurrent` and `drain::drain` (exist; drain_concurrent at about drain.rs 2342). Guards: item read, run passed, head of the landing queue, landing lock held in `Drain::land`. Fixture: a Sync scripted `Rebasing` struct modelled on `Shared` in tests/drain.rs `mod parallel`, passed as the `steps` argument. It answers `Steps::rebase` from a Mutex<VecDeque<Rebased>> of scripted path lists, answers `rerun_checks` from a scripted pass/fail list, and logs calls. `Steps::rebase`, `rerun_checks`, `rerun_cap` and `Rebased` are added with defaults by plan step 1. The overlap decision and the owing set run in production code, not in the fixture.
- Criterion 2: Two independent jobs a and b with disjoint plan paths. a's candidate changed `src/a.rs` and its rebase touched only `docs/other.md`; b's candidate changed `src/b.rs` and its rebase touched `src/b.rs`. Run `drain_concurrent` with bound 1 and render the report with `display::drain(&report)`. A second test uses no rebase at all. A third gives `rerun_cap` a value above the remaining budget, with an overlapping rebase. Run: `cargo test -p athena-cli --test drain rebase_recheck`. Observes: The log has no 'rerun a' and has 'rerun b' before 'land b'. report.rebases is {total: 2, rechecked: 1}. report.notes holds 'rebases: 1 of 2 needed a re-check (measured by candidate paths and any Cargo.toml or Cargo.lock)', and the rendered `display::drain` text contains that line. With no rebase the notes have no 'rebases' line and rebases is {0, 0}. In the budget case the log has no 'rerun' and no 'land', report.stop is `Stop::BudgetSpent`, `failed_item` is None, the job line says the re-check does not fit the budget, and the plan pass-over line carries that reason.
  Fails without the behavior: If every rebase were re-checked, 'rerun a' would be logged and rechecked would be 2. If the tally were not recorded, the field and the note would be missing and the rendered text would lack the line. Without the cap hold the over-budget case would run the re-check. Without the `LandStep::Budget` mapping, the budget test would see `Stop::RepeatedFailure` and a `failed_item`, so the stop assertions fail.
  Test seam: Production entry points `drain::drain_concurrent` and `display::drain` (both exist). Fixture: the scripted Sync `Rebasing` steps, injected as the `steps` argument, with a scripted `rerun_cap` for the budget case. `Report.rebases`, the `State` counters and the note are added by plan step 3. The cap hold and `LandStep::Budget` are added by step 4. The counting, the note and the budget decision run in production code.
- Criterion 3: The existing suite runs unedited. A new test runs `drain_concurrent` with max_concurrency 2 and two jobs a and b in one plan group (disjoint paths). The scripted steps answer `Unchanged` for a and a `Moved` whose touched paths miss b's candidate paths for b. The verifier reads `git diff` for removed or edited existing test lines. Run: `cargo test -p athena-cli && git diff -- crates/athena-cli/src/drain.rs crates/athena-cli/tests/drain.rs`. Observes: Both jobs land (report.landed is 2). 'land a' is logged before 'land b'. The log has no 'rerun' entry. report.rebases is {total: 1, rechecked: 0}. The whole existing suite passes. The diff shows only added test lines, and the only changed existing lines in drain.rs are in `Drain::land`, `attempts`, `State`, `Report`, `drain_with` and the module docs.
  Fails without the behavior: If the re-check ran for a non-overlapping rebase, or the parallel path landed the second job differently, the log assertions fail. If the new `Steps` methods had no defaults, every existing stub (Stub, Housekeeping, World, Lane and the tests/drain.rs Stub and Replay) would fail to compile.
  Test seam: Production entry point `drain::drain_concurrent` with max_concurrency 2 (exists). Fixture: the scripted Sync `Rebasing` steps, injected as the `steps` argument. The defaults come from plan step 1. The landing order and the re-check decision run in production code.
- Criterion 4: Run the three Athena checks on the whole workspace, unchanged from how Athena runs them. They compile and run every test, including the new `rebase_recheck` integration tests (step 7) and the `rebase_needs_recheck` unit tests (step 2), and lint all code including the new code. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0: no formatting difference, no clippy warning, and every test passes, including the new drain tests and the unit tests for the pure function.
  Fails without the behavior: A formatting difference, a clippy warning, or any failing or non-compiling test (for example an existing `Steps` stub broken by a method without a default) makes the command exit non-zero.
  Test seam: Production entry points: the real commands `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`, the same three Athena runs. They are run at the workspace root, which injects the real source tree as their input. No fake or fixture is used. The commands exist today; the new tests they run come from plan steps 2 and 7.
- Criterion 5: Read the CHANGELOG.md diff under the Unreleased section, Added. Run: `git diff -- CHANGELOG.md`. Observes: One added bullet citing D145 and build job 7. It says that after a rebase the drain re-runs checks only when the rebase touched paths the candidate changed (or any Cargo.toml or Cargo.lock), and that the share of rebases that needed it is counted and shown in the drain summary as 'N of M' with the rule named. It also says the real steps do not turn the rebase on yet, because A6's drain wiring is absent, so the share is not measured in a live drain yet.
- Criterion 6: List the changed files and check that none is a governance file. Run: `git diff --stat`. Observes: Only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, docs/design/async-drain.md and CHANGELOG.md appear (landing.rs is unchanged by design). None of athena.toml, defaults.toml or .github/workflows is listed. The only new behavior is re-running checks, plus the budget stop that already exists. No parser, stem list or refusal rule is added, so no refusal-corpus case and no refusal-rate figure applies.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 85010362a58d → candidate aed33d74ec84. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  16 ++
 crates/athena-cli/src/drain.rs   | 192 +++++++++++++++++++
 crates/athena-cli/tests/drain.rs | 399 +++++++++++++++++++++++++++++++++++++++
 docs/design/async-drain.md       |  21 ++-
 4 files changed, 627 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41350542563 for aed33d74ec844268eda7ea836860dce86880b99d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41350542563 for aed33d74ec844268eda7ea836860dce86880b99d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41350542563 for aed33d74ec844268eda7ea836860dce86880b99d)
- ci-result: `not a check` passed (Check results for aed33d74ec844268eda7ea836860dce86880b99d come from the CI run https://github.com/doodla/athena/actions/runs/41350542563 (574s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
