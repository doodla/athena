# Job 20261029-233021-e5sg: verify "Match each job's landing stage to its own landing evidence"

## Request (verbatim)

> Priority: urgent
> The throughput section's landing stage is still wrong after the fix that landed as 2270e80: it now reports a median of 273.8 minutes for landing and rebuild, while real landings take minutes (job 20261029-130605-mbpm: outcome 20:41:09 UTC, its commit on main at 20:44:56 UTC). No landing record for that job exists under the landings path the code searches, so the matching picks some other record. The tests used only fixture records, so the error reached main. Fix: match each job to its own landing evidence (its landing record if one exists, else the merge commit on main that carries the job id in its records or the landed job's commit time), and when no evidence is found say so for that job instead of borrowing another record. Validate against this repository's real records, not only fixtures.
> Acceptance:
> - A test with fixture records covers a job with its own landing record, a job without one, and a neighbouring job's record that must not be used.
> - A check run on this repository's real records (shown in the job report and repeatable by the verifier with athena status) gives, for the five most recent landed jobs, landing durations within 3 minutes of the time between each job's outcome and its commit on main as shown by git log.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

Fix the throughput section's landing-and-rebuild stage in status.rs so each landed job ends at its own landing evidence (its landing record, else its merge commit on main that carries the job id, else the job's commit time). When none is found, report no evidence for that job and don't borrow another record. This follows the landed 2270e80 fix, which still reported a 273.8-minute median. It must be validated on this repository's real records.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs. landing_record_time (~line 772) filters records.landings for a JobLanding commit with job == this job, taking the newest created_at. landing_end (~788) takes the later of the outcome and that time, else the outcome. It feeds busy_intervals, stage measurement (~859) and the summary text (~976). The fix is in this resolution order: own landing record, then a main-branch merge commit carrying the job id, then the job's commit time, then explicit none. Landing time is measured directly as that evidence's timestamp minus the verifier end or outcome, not via a stand-in. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected.
- Goal and audience: the owner reading `athena status` throughput needs a trustworthy landing stage. The request is urgent because the wrong median reached main.
- The newest-record rule in landing_record_time and any commit-kind matching may pick a record in which the job id appears as a neighbour. The producer must find the actual root cause on the real records, not only assume this one.
- Evidence outside records/ is read through the existing history data (athena_data::history); no new dependency is added. If the merge commit isn't reachable from the loaded data, the fallback is the landed job's commit time.
- Job-level 'no evidence' is shown in the report text, for example 'landing evidence not found for N of M', and that job's landing stage is excluded from the median rather than counted as zero or borrowed.
- Governance files (athena.toml, defaults.toml, workflows) are not needed; CHANGELOG.md is a deliverable per the request.

## Acceptance criteria

1. [content] crates/athena-core/src/status.rs has a fixture test with three jobs: one with its own landing record (its landing stage equals that record's time minus the outcome), one with no landing record (reported as no evidence, or resolved only by its own merge commit or commit time), and one whose neighbouring job's landing record exists and is not used for it.
2. [behavior] Run `cargo run -- status` (athena status) on this repository's real records. For the five most recent landed jobs the reported landing durations are each within 3 minutes of the time between the job's outcome and its commit on main from `git log`. For job 20261029-130605-mbpm the landing stage is about 3.8 minutes (outcome 20:41:09 to commit 20:44:56 UTC), not hours. The overall median for landing and rebuild is no longer 273.8 minutes. The producer's job report shows this check.
3. [content] When no landing evidence is found for a job, the status output says so for that job. No other job's record is used, and the code path is covered by a test.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
5. [content] CHANGELOG.md has an entry that names the landing-stage matching fix and the real-records validation.
6. [content] Job 20261029-130605-mbpm, a landed job that only supplies the evidence case, is not modified. Its changed files (CHANGELOG.md, crates/athena-core/src/hestia.rs, docs/enforcement.md, docs/spec.toml, xtask/tests/spec.rs) are kept as landed and not touched, and its two minor findings (a stale spec.toml comment and OLY-B-WHO-01 lacking an open-question note) are out of scope here.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Fixing the two minor findings of job 20261029-130605-mbpm (stale spec.toml comment about two late headings; OLY-B-WHO-01 open-question note)
- Editing job 20261029-130605-mbpm's landed files: docs/spec.toml, xtask/tests/spec.rs, docs/enforcement.md, crates/athena-core/src/hestia.rs (kept as landed; CHANGELOG.md is edited only to add the new entry)
- Changing athena.toml, defaults.toml, workflow files or records/
- Redesigning other throughput stages or the busy-interval model beyond what landing matching needs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture test plus fmt, clippy -D warnings and the full test suite.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the five most recent landed jobs on the real records against git log.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix replaces the matching rule rather than patching the one job.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier repeats the athena status check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is landing durations matching real commit times, not just that code compiles.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A job with no evidence is reported as such, not counted as success or zero.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ca27c9d619b2 → candidate 40c5a445c7ab. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  11 ++
 crates/athena-core/src/status.rs | 218 ++++++++++++++++++++++++++++++++-------
 crates/athena-core/src/store.rs  |  41 ++++++++
 crates/athena-git/src/lib.rs     |  42 ++++++++
 4 files changed, 273 insertions(+), 39 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1719 tests ran)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "Run `cargo run -- status` (athena status) on this repository's real records. For the five most recent landed jobs the reported landing durations are each within 3 minutes of the time between the job's outcome and its commit on main from `git log`. For job 20261029-130605-mbpm the landing stage is about 3.8 minutes (outcome 20:41:09 to commit 20:44:56 UTC), not hours. The overall median for landing and rebuild is no longer 273.8 minutes. The producer's job report shows this check.")
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
