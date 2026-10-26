# Job 20261025-214509-y179: verify "Cut slow athena-cli test time; full cargo test under 7 minutes"

## Request (verbatim)

> try again on 20261025-202447-xgk9

## Reading (accepted by the user)

Retry of job 20261025-202447-xgk9, keeping the user's decided approach. No nextest. Cut real time from the slowest athena-cli test binaries. Add the secrets.rs scan memo and a shorter run_check poll schedule, each with a behavior-preserving test. Goal: full `cargo test --locked` under 7 minutes. Audience: the user, who reads docs/decisions.md and later job times.

Assumptions:
- The user's replies to the interrupted attempt carry over to this retry unchanged.
- The job's machine is comparable to the 4-core brain machine; times are measured in the job.
- The previous attempt left no findings and nothing carries over, so the producer starts from the base.

## Acceptance criteria

1. [content] docs/decisions.md has a new decision with per-binary before and after times (lifecycle, job_plan, drain, retry, steward), measured the same way in the job and labeled measured, with the cause found for each and the total suite time.
2. [content] The decision records compile 67 s vs run 673 s (brain-measured, 4 cores). It lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as rejected. It lists sccache (D79) and a shared target dir as considered. It claims no unmeasured number.
3. [content] No test is deleted, ignored, filtered or has assertions weakened, and the test count is not lower than before. athena.toml, ci.yml and tool pins are unchanged. CHANGELOG.md has an entry.
4. [content] Production changes are limited to the scan memo in crates/athena-gate/src/secrets.rs and the shorter poll schedule in crates/athena-core/src/checks.rs. Each leaves behavior unchanged and has a test: the memo equals uncached scanning, and the poll schedule still detects completion, failure and timeout.
5. [behavior] Run `cargo test --locked`. It passes with every test running, and the run is under 7 minutes on the job's machine. A failing test still fails the run.
6. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`. Both are clean.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/secrets.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-core/src/run.rs (modify)

## Out of scope

- Adopting cargo-nextest or sccache
- Changing athena.toml, ci.yml or tool pins
- Removing, ignoring or weakening any test

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision labels measured numbers and cites the brain's measurements.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Test, clippy and fmt criteria are included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The suite time and the memo and poll behavior are checked by running tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Production changes are limited to two files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: At the base, run each binary alone and the full suite, and note each 'finished in' time and 'N passed' count. Then do the same after the change under the same conditions. Per test, add temporary timing and gitleaks and poll counters in a scratch run, and remove them before finishing. Write the table and causes into the new decision, labeled measured, together with the measured memo hit count per binary. Run: `cargo test --locked --test lifecycle`. Observes: The decision lists lifecycle, job_plan, drain, retry and steward with before and after seconds, a cause for each, and the total suite time. The numbers match the logged outputs of `cargo test --locked --test job_plan`, `--test drain`, `--test retry`, `--test steward` and the full `cargo test --locked`.
- Criterion 2: Read the added decision text in the diff. Run: `git diff docs/decisions.md`. Observes: It states compile 67 s vs run 673 s (brain-measured, 4 cores). It lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as rejected. It lists sccache (D79) and a shared target dir as considered. Every other number is one this job measured and labeled.
- Criterion 3: Inspect the diff for removed #[test], added #[ignore] or filters, weakened asserts, and changes to athena.toml, ci.yml or tool pins. Compare the sum of 'N passed' before and after; it must be at least as high. Confirm a CHANGELOG.md entry. Run: `git diff --stat`. Observes: The diff touches only the listed deliverables. The test count is not lower. CHANGELOG.md has an Unreleased entry.
- Criterion 4: New unit tests. In secrets.rs: (a) a clean range and a token range give results equal to a fresh uncached scan; (b) the hit counter rises on a repeat of an immutable range; (c) the counter stays flat for a moved ref (HEAD advanced) and after an Err result; (d) a later commit that adds a token is found at the new tip. In checks.rs: run_check with a spec that exits 0, one that exits 3, and one that sleeps past a short timeout still reports completion, failure and the deadline kill; the schedule function is tested directly (5, 10, 20 ... capped at 200, clamped to the time left), with no wall-clock threshold. Check that the production diff is limited to the two files. Run: `cargo test --locked -p athena-gate secrets`. Observes: The memo tests pass, including the hit-count assertions. `cargo test --locked -p athena-core checks` passes the completion, failure, timeout and schedule tests. `git diff --stat` shows production changes only in secrets.rs and checks.rs.
  Fails without the behavior: A memo that does nothing fails the hit-count-rises assertion. A memo keyed on the raw ref fails the moved-ref and later-token assertions. A poll that missed completion, failure or timeout fails the exit-0, exit-3 and timeout assertions, and a wrong schedule fails the schedule function test.
- Criterion 5: Run the full suite on the job's machine, read total wall time and the 'N passed' lines, and confirm no ignored or filtered tests. Separately, add a temporary failing test, run the suite, record the exit code in the report, then remove the test. Run: `cargo test --locked`. Observes: Exit 0, the passed count is not lower than at base, and total time is under 7 minutes. The run with the temporary failing test exits non-zero, and that exit code is recorded in the producer report.
  Fails without the behavior: If the memo or poll change broke a check, or a test were skipped or removed, the run would fail or the passed count would drop. If the time were over 420 s the goal would be unmet. The temporary failing test makes the run exit non-zero.
- Criterion 6: Run both commands. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Clippy exits 0 with no warnings. `cargo fmt --all --check` exits 0 with no diff.
  Fails without the behavior: A warning or unformatted code makes the command exit non-zero.

Planned files:

- crates/athena-gate/src/secrets.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-core/src/run.rs (modify)

## The change

Base a72cc153b7a7 → candidate 3455b035cf3d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   5 +
 crates/athena-core/src/checks.rs  |  55 ++++++++-
 crates/athena-gate/src/secrets.rs | 254 ++++++++++++++++++++++++++++++++++++--
 docs/decisions.md                 |  59 +++++++++
 4 files changed, 361 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1246 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the recorded check results: acceptance criteria not met or not reported: "Run `cargo test --locked`. It passes with every test running, and the run is under 7 minutes on the job's machine. A failing test still fails the run."; "Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`. Both are clean.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
