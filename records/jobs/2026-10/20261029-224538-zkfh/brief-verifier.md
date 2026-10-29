# Job 20261029-224538-zkfh: verify "Measure landing time to the landing record; unlanded time as time spent"

## Request (verbatim)

> try again on 20261029-155620-djjt

## Reading (accepted by the user)

Retry of rejected job 20261029-155620-djjt. The throughput section of `athena status` will measure "landing and rebuild" from the verifier's end to the landing record (`landings/*.json`, `athena_data::history::Landing`, matched to the job by `commits[].job`), not to host refusals. "Did not land" and idle time use time spent (invocation wall times plus check durations), and every figure prints its basis. The audience is the throughput review and whoever reads `athena status`.

Assumptions:
- Code survey: I read crates/athena-core/src/status.rs (StageMinutes, landed_jobs, working_span, check_minutes, Throughput, and Throughput::compute called at line 1726), crates/athena-core/src/store.rs (the Records struct has no landings field), crates/athena-core/src/gh/records.rs (LANDING_DIR = "landings"; the snapshot() reader at line 399 collects HistoryRecord::Landing from it) and crates/athena-data/src/history.rs (the Landing struct at line 221: created_at, commits[].job). Time is measured directly from those records, not through a stand-in. The data record type already exists, so no new record file is added and check_record_file/AnyRecord need no change.
- Landing record named by the request: `athena_data::history::Landing`, written after the merge by GitLandingRecords::write_landing (gh/records.rs) and stored under `landings/` on the records branch. It exists in the current snapshot. Its `created_at` marks the end of the early-CI wait, push and merge. Its `commits[].job` (for kind job_landing) ties it to the job.
- Job 20261029-155620-djjt finding by finding: (a) blocker, landing_end used host refusals: dropped; the landing end is max(outcome time, landing.created_at). (b) major, the criterion-1 test used a 405 refusal: replaced by fixtures of real Landing records with no HostError. (c) major, the basis, CHANGELOG and hestia.rs said no record exists: reworded to name the landing record. (d) minor, waits left out of time spent: time spent is invocation wall times plus measured check durations plus the landing wait from the landing record, and the basis says so. (e) minor, check placement in idle time is an estimate: the basis line says it is placed by estimate. (f) reading mismatch: this reading names the landing record explicitly.
- Because Records does not load landings, the producer adds the smallest reader. Either Records::load gains a landings field read from the records tree (store.rs, if the record set exposes the directory) or status takes a landing-times map by job id. A job with no landing record falls back to the outcome time, and the basis says so.
- Idle time is the gap between merged busy intervals built from each job's invocations, check durations and landing wait, not from first call to close. A job left open for days adds only the time it was worked.

## Acceptance criteria

1. [behavior] A test in crates/athena-core/src/status.rs builds fixtures of real record kinds: a landed job with invocations, a verifier ending at T, a Landed outcome set at about T+0.1 min, and an athena_data::history::Landing record (commits[].job = the job) with created_at several minutes later, and no HostError. `cargo test -p athena-core status` shows the landing stage equals the minutes from the verifier's end to the later of the outcome and landing.created_at.
2. [behavior] A test shows that a landed job with no landing record reports the outcome-based landing figure and that the printed basis says no landing record was found. Run `cargo test -p athena-core status`.
3. [behavior] A test dispatches a job, runs it 20 minutes and closes it days later. `cargo test -p athena-core status` shows 'did not land' reports 20 minutes, not the multi-day span.
4. [behavior] A test shows idle time between jobs is computed from time spent: a job open for days does not hide or inflate the gap. Run `cargo test -p athena-core status`.
5. [behavior] A test on `Throughput::text` shows every figure (landed per day, median per landed job and its stages, idle between jobs, did not land) states its basis, and the landing basis names the landing record (landings/) and no longer says the CI wait and rebuild have no record. `athena status` prints that text.
6. [content] Grep of crates/athena-core/src/status.rs, crates/athena-core/src/hestia.rs and CHANGELOG.md finds no claim that the CI wait and rebuild have no record, and the hestia.rs throughput prompt and the CHANGELOG entry say they are measured from the landing record.
7. [content] CHANGELOG.md has an entry describing the corrected landing, unlanded and idle figures and the printed bases.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
9. [content] No governance file (athena.toml, defaults.toml, workflows) changes. Only the listed deliverables change.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/store.rs (modify)

## Out of scope

- Changing how landing records or the rebuild are written (crates/athena-core/src/gh, land.rs, athena-data history.rs); they are only read.
- Re-ranking the throughput review's proposals or changing its trigger.
- Changing athena.toml, defaults.toml, workflows or any other governance file.
- Keeping the earlier candidate's host-refusal landing_end logic: that work is dropped. Its status.rs, hestia.rs and CHANGELOG.md edits are redone, not kept.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover each figure; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The tests use the real landing record, not a stand-in.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the corrected figures and printed bases.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Landing is measured from its own record, not from refusals.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verification by cargo commands.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the throughput code.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Basis text and the CHANGELOG are written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 507d69e6599e → candidate 4ee576bd4720. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   9 ++
 crates/athena-core/src/hestia.rs |   5 +-
 crates/athena-core/src/status.rs | 300 +++++++++++++++++++++++++++++++++++----
 crates/athena-core/src/store.rs  |  34 +++++
 4 files changed, 323 insertions(+), 25 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1671 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
