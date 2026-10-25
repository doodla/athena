# Job 20261023-181804-j42y: verify "Report space reclaimed when a closed job's build cache is deleted"

## Request (verbatim)

> Athena never deletes a job's build cache. Each job's state directory keeps its own Rust target directory (11 to 12 GB each), and on 2026-10-23 two of them, one from an interrupted job of 2026-10-17, filled the session's disk allowance mid-run, so job 20261019-013306-zvmr died committing its candidate. Delete a job's target directory once the job closes (landed, failed, rejected, abandoned or interrupted and then closed), and report the space reclaimed; consider sharing one target directory across jobs if the checks allow it.

## Reading (accepted by the user)

Closing already calls StateDir::clean_job, which silently removes the job directory (including its 11-12 GB target) on some paths. The change makes cleanup cover every closing path, including a close of an interrupted job, report bytes reclaimed, and stop swallowing removal errors; sharing one target directory is evaluated against the checks and recorded as a decision (expected: not adopted). Audience: the Athena operator who ran out of disk.

Assumptions:
- Goal: stop closed jobs from leaving 11-12 GB caches that fill the disk; audience is the Athena operator and the verifier reading output and tests.
- Reading of the code: clean_job (state.rs) is already called after close_with, close_dispatched, land_or_block (non-blocked), retry_landing, but it ignores errors, reports nothing, and is skipped when an earlier step errors (e.g. commit out of disk), so leftovers persist.
- Sharing one target directory is considered but not adopted by default: review 2 R14 shows a shared dir reuses another job's stale test binaries; per-job isolation is deliberate (D46).
- Deleting caches of already-closed historical jobs on this machine is an operational action, not a code deliverable; optionally a sweep could be added but is not assumed.

## Acceptance criteria

1. [behavior] A test creates a job directory containing a target/ tree of known size, closes the job through each closing path (landed, failed, rejected, abandoned, interrupted-then-`athena close`, closed while blocked), and asserts the job directory no longer exists and the reported reclaimed byte count equals the known size. Run: cargo test.
2. [content] The closing line shown to the user (athena go / close output) states the space reclaimed in human units, e.g. 'reclaimed 11.4 GB'; a job with nothing to remove reports no failure.
3. [behavior] If removal of the job directory fails, the user sees a warning naming the path rather than silent success (test with an unremovable path or injected error). Run: cargo test.
4. [behavior] A job that is still running or blocked keeps its directory; cleanup runs only after the closing commit succeeds. Run: cargo test.
5. [content] docs/decisions.md records the shared-target-directory question with alternatives and the evidence (docs/design/reviews/athena-v0-review-2.md R14: a shared target dir executed another job's test binaries), and the decision taken; the per-job CARGO_TARGET_DIR in athena.toml stays unless the decision says otherwise.
6. [behavior] cargo test, cargo clippy with warnings denied, and cargo fmt --check all pass.
7. [content] CHANGELOG.md has an entry describing that closing a job deletes its build cache and reports space reclaimed.

## Expected deliverables

- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/pin.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Switching to a shared or persistent cross-job CARGO_TARGET_DIR
- Manually deleting directories on the operator's machine
- Fixing the land --pr head-checking behavior of job 20261019-013306-zvmr
- Capping disk use during a running job

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each closing path and removal failure; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert directory gone and byte count reported, not that a function exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Covers all closing paths and error cases instead of the one that surfaced.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Shared-target alternative written down in decisions.md with evidence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to cleanup and reporting.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: state.rs unit tests: a dir with target/ files, job.lock, live/x, and a hard-linked pair. Assert the dir is gone and report.bytes equals a hand-computed value for a small tree (one file's blocks*512 once for the linked pair, plus each directory's blocks*512). Also compare to a test-local walk and check it is at least the planted target allocation. lifecycle.rs: one test per path (landed, failed, rejected, abandoned, interrupted then athena close, closed while blocked). Plant a multi-block target/ file before the closing commit (fake producer on go paths; pre-planted for close paths). Record an independent walk just before closing, then assert the dir is absent and the output has 'reclaimed ' plus format_bytes of that number. Run: `cargo test --locked`. Observes: No job directory remains on any closing path and the reported figure matches the independent measurement.
  Fails without the behavior: A path that skipped cleanup leaves the dir; without measurement no 'reclaimed' line appears.
- Criterion 2: format_bytes unit test (11_400_000_000 -> '11.4 GB', plus B/KB/MB/TB). Lifecycle tests capture the Lines ui output of go and close and assert a 'reclaimed <N> <unit>' line. A job whose dir holds only job.lock yields a 'reclaimed' line equal to format_bytes of its measured size and no line containing 'warning'. Run: `cargo test --locked`. Observes: Human-unit reclaimed line appears; the no-cache job shows no failure.
  Fails without the behavior: Missing helper means no line; a spurious warning would fail the no-warning assertion.
- Criterion 3: Lifecycle test with the injected remover returning an io::Error: close a job (lock taken, commit succeeds). Assert close returns Ok, a line starts with 'warning' and contains the job-dir path, and no line contains 'reclaimed'. A control in the same test uses the default remover on a normal dir and sees it removed with no error. state.rs unit tests put a regular file at the path and assert report.error carries the path and report_clean prints only the warning. Run: `cargo test --locked`. Observes: Warning naming the path, no reclaimed claim, close Ok.
  Fails without the behavior: With `let _ =` swallowing, no warning line and the assertion fails.
- Criterion 4: Tests: a job left Blocked after go still has its dir and no reclaimed line. A close whose commit fails (gate violation or unwritable record) returns Err with the dir intact and no reclaimed line. An interrupted job's dir exists before athena close. Run: `cargo test --locked`. Observes: Dir present and no reclaimed output in each case.
  Fails without the behavior: Cleanup before or regardless of the commit would remove the dir.
- Criterion 5: Inspect the diff: a new decisions.md entry (next free number) on the shared target dir, with alternatives, R14 evidence from docs/design/reviews/athena-v0-review-2.md, and the decision not to share. athena.toml has no diff. Run: `git diff -- docs/decisions.md athena.toml`. Observes: Entry present with evidence and decision; athena.toml unchanged.
- Criterion 6: Run the three checks. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0.
  Fails without the behavior: Format, lint or test failures give nonzero exit.
- Criterion 7: Inspect the diff for the CHANGELOG entry under the unreleased heading and the README note. Run: `git diff -- CHANGELOG.md README.md`. Observes: Entry says closing deletes the build cache and reports space reclaimed; README line present.

Planned files:

- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## The change

Base 8a364b2a7375 → candidate 720097f7841e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   5 +
 README.md                            |   2 +-
 crates/athena-cli/tests/lifecycle.rs | 301 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/manage.rs     |   4 +-
 crates/athena-core/src/run.rs        |   6 +-
 crates/athena-core/src/state.rs      | 219 ++++++++++++++++++++++++-
 docs/decisions.md                    |  35 ++++
 7 files changed, 558 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (978 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
