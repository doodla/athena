# Job 20261026-002911-8gp8: verify "Hold jobs at dispatch on low disk; fail fast before checks"

## Request (verbatim)

> try again on 20261017-001109-qfax

## Reading (accepted by the user)

Retry of 20261017-001109-qfax, following the user's latest decision. Before dispatch, check free disk against a minimum (default 10G in config.rs). If low, clean closed and idle workspaces' target/. If still low, leave the job queued with 'environment: low disk' and the free and required amounts, and do not dispatch or reject it. Before the checks, low space gives a clear, retryable environment failure, not a hold, so nothing mid-run needs resuming. Audience: the user running drain.

Assumptions:
- The user's decision overrides the 'proposed next request': the free-disk hold happens only at dispatch, and a job is never left Dispatched with a hold marker, so no mid-run resume route is needed.
- A held job stays in its queued (Proposed) state and a later drain picks up the same job when space is free; no new job is created.
- Before the checks, low space after cleanup fails the job with a retryable environment reason, not a rejection; the existing retry route applies.
- Free space is read through a small injectable function so tests can simulate low and recovered disk.
- athena.toml is not edited. The 10G default lives in config.rs and applies when the setting is absent. If the config digest covers the new field, it is reset as the repository requires.
- The deletion of a closed job's target/ already landed in 173d12e and is not redone. The idle sweep reuses the existing remover and job-lock helpers, and skips locked workspaces.
- The user-facing text says 'the user', never 'operator'.

## Acceptance criteria

1. [behavior] With free disk simulated below the minimum, a test shows cleanup of closed and idle workspaces' target/ runs before dispatch and before the checks. Run `cargo test --locked`.
2. [behavior] A test shows that when space is still low after cleanup at dispatch, the job is not dispatched, stays queued (not rejected) with status 'environment: low disk', and the free and required amounts appear in status and drain output. Run `cargo test --locked`.
3. [behavior] A drain test through the real step mapping, not only a stub, shows a low-disk hold is not counted as a failure. Once the fake reports enough space, a later drain runs the same job (same id) to a verdict. Run `cargo test --locked`.
4. [behavior] A test shows that when space is low before the checks and still low after cleanup, the job fails with a clear retryable environment reason, is not rejected, and no hold marker is left. When space is freed, verification goes ahead. Run `cargo test --locked`.
5. [behavior] A test shows the idle-workspace sweep removes target/, leaves source files, and skips workspaces whose job lock is held. Run `cargo test --locked`.
6. [content] config.rs defines a 10G minimum-free-disk default used when the setting is absent, and a test shows it parses. athena.toml is unchanged. README and CHANGELOG name the setting and the 'environment: low disk' status and say 'the user', never 'operator'.
7. [behavior] `cargo clippy --all-targets --locked -- -D warnings` is clean and `cargo fmt --all --check` passes.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/disk.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Editing athena.toml
- Mid-run holds and a resume route for a job left Dispatched
- Re-running job 20261016-044847-hem1
- Deleting a closed job's target/, which landed in 173d12e
- Changing `athena retry` semantics

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New disk tests are added; cargo test, clippy and fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check hold, cleanup order and the later run through tests that simulate disk.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and messages are plain and name the setting and status.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The mid-run hold is dropped by the user's decision, so the change is small.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The minimum is enforced in code before dispatch and before the checks, not by prose.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests assert the outcome (held, not rejected, later runs), not only that the code exists.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Core tests plant an idle job dir with a target/ and a source file. The per-test-dir fake reports 3 GB while the planted target/ exists, then 50 GB. The tests run `go` with a recording UI, once for the dispatch gate and once for the pre-checks gate. Run: `cargo test --locked`. Observes: The sweep line and removal of the target/ come before the 'dispatched' line and before the first check line.
  Fails without the behavior: Without the sweep in dispatch and the pre-checks gate, target/ would remain and no sweep line would precede dispatch or the checks.
- Criterion 2: The fake stays at 3 GB. The test runs `go`, then status and the drain report. Run: `cargo test --locked`. Observes: `go` returns Err(LowDisk) with no dispatch commit. The state is still proposed, not closed or rejected. The status text and drain line contain 'environment: low disk', the free amount and 10.0 GB required.
  Fails without the behavior: Without the hold the job would be dispatched or rejected, and status would show plain 'proposed' without amounts.
- Criterion 3: A tests/drain.rs test uses RealSteps::live. The fake reads `<state>/simulated_free`. The test drains once low, writes high, then drains again. Run: `cargo test --locked`. Observes: First drain: outcome 'environment: low disk', failed count 0, stop is not RepeatedFailure, and the job is still queued. Second drain: the same job id runs to a verdict.
  Fails without the behavior: If the hold were mapped as a failure or a new job, the first drain would count a failure or the second would run a different id.
- Criterion 4: Core test with space high at dispatch and low at the pre-checks gate, still low after the sweep. A second case frees space after the first probe. After the first case, the test frees space and runs `athena.retry` then `go`. Run: `cargo test --locked`. Observes: The state is Failed (not Rejected), the reason starts 'environment: low disk' and names retry, there is no holds/<job>.json, and no check ran. After freeing, checks run in the second case and in the retry.
  Fails without the behavior: Without the gate the checks run on a full disk and fail opaquely, or the job is Rejected, or a marker is left, or retry is refused.
- Criterion 5: A disk.rs unit test with three job dirs: one unlocked with target/ and a source file; one locked via try_lock_job with target/; one with no target/. The test runs the sweep. Run: `cargo test --locked`. Observes: The first target/ is gone and its source file is kept. The locked job's target/ is intact.
  Fails without the behavior: A sweep that ignores the lock removes the locked target/. One that removes whole directories loses the source file.
- Criterion 6: config.rs tests parse athena.toml (min_free_gb() == 10) and a toml with `[disk] min_free_gb = 3` (== 3), and compare reading_inputs with and without [disk]. `git diff` shows no athena.toml change. Read README.md and CHANGELOG.md. Run: `cargo test --locked`. Observes: The default 10 and the parse of 3 pass, the digests are equal, and athena.toml is absent from the diff. The docs name `[disk] min_free_gb` and 'environment: low disk' and contain no 'operator'.
  Fails without the behavior: Without the config.rs default the absent-setting test would not find 10. A plain serialized field would fail the digest-equality test.
- Criterion 7: Run clippy with warnings denied, then `cargo fmt --all --check`, over the whole change. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Both exit 0 with no warnings or diffs.
  Fails without the behavior: Unformatted code, an unused item or a needless clone would make clippy fail under -D warnings, or make fmt --check print a diff.

Planned files:

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/disk.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)

## The change

Base bb8e22037bb0 → candidate 1925575e651a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  10 ++
 README.md                               |  13 ++
 crates/athena-cli/src/display.rs        |   7 +-
 crates/athena-cli/src/drain.rs          |  31 +++-
 crates/athena-cli/tests/common/mod.rs   |   3 +-
 crates/athena-cli/tests/drain.rs        | 294 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/disk.rs          | 252 +++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs           |   1 +
 crates/athena-core/src/run.rs           |  26 +++
 crates/athena-core/src/run_pin_tests.rs |   3 +-
 crates/athena-core/src/state.rs         |  52 ++++++
 crates/athena-core/src/status.rs        |  13 +-
 crates/athena-core/src/test_support.rs  |   8 +-
 crates/athena-data/src/config.rs        |  42 +++++
 14 files changed, 748 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1251 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
