# Job 20261029-155620-djjt: verify "Fix throughput figures: landing time, time spent, and stated bases"

## Request (verbatim)

> Priority: urgent
> Two figures in the new throughput section of athena status are wrong, and the throughput review ranks proposals by them. (1) 'landing and rebuild 0.1' minutes per landed job: landing waits for the early CI run, pushes, and rebuilds Athena after a self-landing, which takes several minutes; measure it from the verifier's end to the landing record and the rebuild's end. (2) 'did not land: 206 job(s), 124132.6 min': that is about 2000 hours, measured as wall span from first call to close, which counts jobs left open for days; measure time spent instead (the sum of invocation wall times, check durations and waits recorded for the job). Check idle time between jobs the same way. Each figure in the section must state its basis (measured from which records).
> Acceptance:
> - A test with fixture records for a landed job with a CI wait and a rebuild shows the landing stage reports their duration.
> - A test with a job dispatched, run for 20 minutes and closed days later shows 20 minutes of spent time, not the span.
> - athena status prints the basis of each figure.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

The throughput section of `athena status` misreports two figures. "Landing and rebuild" is measured only to the outcome record, so it misses the CI wait and the rebuild. "Did not land" is a first-call-to-close wall span, so it counts days a job sat open. I'll measure landing through the recorded CI wait, push and rebuild end, measure unlanded and idle time as time spent, and print each figure's basis. The audience is the throughput review and whoever reads `athena status`.

Assumptions:
- Code survey: I read crates/athena-core/src/status.rs. `Throughput::compute` (about line 804) builds the section. `landed_jobs` (about line 775) sets `landing_and_rebuild` to the time from the last invocation's end to the closing outcome's `at`. The unlanded figure sums `working_span` (first invocation start to closing outcome or last invocation end). Idle time is the gaps between those merged spans. `Throughput::text` prints the lines. I also read crates/athena-data/src/records.rs (Outcome, Verification, CheckRun, Intervention) and crates/athena-core/src/store.rs (`JobRecords`). Landing writes the early-run wait as a warning string (crates/athena-core/src/gh/early.rs) and writes landing records under landings/ (crates/athena-core/src/gh/records.rs). I did not confirm whether `JobRecords` or `Records` exposes either.
- Time spent for a job is the sum of its invocation wall times, its measured check durations, and its recorded waits. The unlanded figure and the idle-time computation both use that sum instead of the first-to-close span.
- The CI wait and rebuild are read from whatever records already carry them, such as the outcome's time, landing records, or the 'waited Ns' note. If one is not reachable from `JobRecords`, the producer adds the smallest reader inside status.rs and states the basis. If the rebuild's end is not recorded, the stage reports what is recorded and its basis says so. Changes stay within status.rs and CHANGELOG.md.
- Idle time is the time between jobs that no job spent working: the gaps between merged spans, where a span covers only recorded invocation, check and wait intervals rather than first call to close.
- No new record type or file under crates/ is added, so the record validation allow-list is untouched. No refusal rule is added or tightened.
- The existing test throughput_reports_landed_per_day_stage_medians_idle_time_and_time_on_jobs_that_did_not_land (about line 2966) and the helper `timed_job` will need updating for the new bases.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/status.rs builds fixture records for a landed job with a CI wait and a rebuild. Running `cargo test -p athena-core status` shows the landing stage reports their duration (it covers the wait and the rebuild, not just verifier end to outcome).
2. [behavior] A new test dispatches a job, runs it 20 minutes, and closes it days later. `cargo test -p athena-core status` shows it contributes 20 minutes to the 'did not land' figure, not the multi-day span.
3. [behavior] A test shows idle time between jobs is computed from time spent, so a job left open for days does not hide the gap. Run `cargo test -p athena-core status`.
4. [behavior] A test on `Throughput::text` output shows each figure in the section (landed per day, median per landed job and its stages, idle between jobs, did not land) states its basis, naming the records it was measured from. `athena status` prints that text, so the basis shows there.
5. [content] CHANGELOG.md has an entry describing the corrected landing, unlanded and idle figures and the printed bases.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] No path other than crates/athena-core/src/status.rs and CHANGELOG.md changes. No governance file (athena.toml, defaults.toml, workflows) changes.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Changing how landing or the rebuild records are written (crates/athena-core/src/gh, land.rs) beyond reading what exists.
- Re-ranking or changing the throughput review's proposals or its trigger.
- Changing athena.toml, defaults.toml or workflow files.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Changes are Rust in the athena-core crate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New fixture tests are required, with fmt, clippy -D warnings and the test suite passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the printed or computed figures from fixtures, not that fields exist.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The measurement basis is fixed for both figures and for idle time, not only the one instance.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The change is limited to status.rs and the changelog.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Each printed figure states its basis in plain words.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Success is that the figures match time actually spent, shown by tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3946375e3dd8 → candidate 6dcad18f986d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   7 ++
 crates/athena-core/src/hestia.rs |   4 +-
 crates/athena-core/src/status.rs | 218 ++++++++++++++++++++++++++++++++++-----
 3 files changed, 202 insertions(+), 27 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1668 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
