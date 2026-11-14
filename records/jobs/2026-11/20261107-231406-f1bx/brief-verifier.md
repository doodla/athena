# Job 20261107-231406-f1bx: verify "Use earliest landing record for a job's landing and rebuild end"

## Request (verbatim)

> Fix the 'landing and rebuild' stage in the throughput figures. Today `landing_record_time` in crates/athena-core/src/status.rs takes the NEWEST landings/ record that carries a job's landing commit. On a stacked branch, later landings carry earlier jobs' commits, so this pushes each job's landing end later every time a new job lands. Measured: the five landed jobs npqp, pg2d, yp1e, abbg and 4z98 have main-branch CI push runs 5 to 12 minutes after their landed outcomes. Yet review 20261107-224336-2v6c reports a landing median of 1,327.8 minutes. Change the code to use the earliest landing record that carries the job's commit, or the main merge time if one is recorded. Acceptance: (1) a test fixture where one job's commit is carried by two landing records shows the stage ending at the first one; (2) running `athena status` on the real records gives landing durations within 15 minutes of outcome-to-main-push for those five jobs; (3) the changelog states the old and new landing medians. Do not queue or build a landing redesign on the old figure until this lands.
> 
> Acceptance:
> - The change is made as described: Throughput reviews keep naming 'landing and rebuild' as the bottleneck (median 440 to 1,328 min), but GitHub shows recent jobs reached main within about 5 to 12 minutes of their landed records
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/status.rs, landings/

## Reading (accepted by the user)

landing_record_time (now in crates/athena-core/src/status/throughput.rs, not status.rs) takes the newest landings/ record carrying a job's job_landing commit; change it to the earliest, so stacked later landings stop pushing the end later. Audience: the owner reading throughput reviews. A landing's created_at is its merge-time record; no separate main-merge field exists, so none is added.

Assumptions:
- Code survey: read crates/athena-core/src/status/throughput.rs (landing_record_time at ~391 uses .max() over records.landings filtered on JobLanding commits with job id; landing_end at ~407 takes outcome.max(record time); used by busy_intervals, ranked_landings and the landing figures) and crates/athena-data/src/history.rs (Landing has created_at, host_commit, commits: Vec<LandingCommit>; no main-merge timestamp field). No new record or data file under crates/ is added, so check_record_file/AnyRecord/round-trip are unaffected. The property is measured directly as the landing_end minus outcome time per landed job.
- Field meaning: Landing.created_at is documented at crates/athena-data/src/history.rs:224 as 'When the landing was recorded', written after the merge; it is used as the main-merge time because no field records a separate merge time.
- Environment: the verifier reads only the repository snapshot with cargo and git log/show, no network; the independent range comes from commit times of the five jobs' landings on main in git log of the snapshot, and GitHub CI push-run times quoted in the request (5 to 12 minutes after landed outcomes). The records directory used for status is the real records snapshot available to the verifier; if absent, the verifier reports it.
- The request names status.rs but the function lives in status/throughput.rs; that file is changed. The 'landings/' path in the request is records data and is not edited (records are append-only).
- Old and new landing medians in the changelog are measured by running athena status before and after the change on the real records; the old figure is the review's 1,327.8 minutes unless reproduced differently.

## Acceptance criteria

1. [content] In crates/athena-core/src/status/throughput.rs, landing_record_time selects the earliest (.min()) created_at among landings/ records carrying the job's job_landing commit, and its doc comment says earliest.
2. [content] A new unit test in crates/athena-core/src/status/throughput.rs (or its tests module) builds a fixture where one job's commit is carried by two landing records and asserts landing_end equals the first record's time, not the second.
3. [behavior] Run `cargo test -p athena-core` : the new fixture test and all existing tests pass.
4. [behavior] Run `cargo run -p athena-cli -- status --snapshot <records directory>` on the real records: for jobs npqp, pg2d, yp1e, abbg and 4z98 the landing duration printed is within 15 minutes of the outcome-to-main-push gap, the expected range being 5 to 12 minutes after the landed outcome (up to 27 minutes tolerance) taken from main-branch commit times in git log, an independent source the change does not compute; the landing median is far below 1,327.8 minutes.
5. [content] CHANGELOG.md has an entry stating the old landing median (1,327.8 minutes as reported, or as measured before) and the new landing median measured after the change, and that landing figures use the earliest landing record.
6. [behavior] No unrelated behavior changes: `git diff` touches only throughput.rs and CHANGELOG.md; cargo fmt --check, cargo clippy and cargo test pass.

## Expected deliverables

- crates/athena-core/src/status/throughput.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Queuing or building a landing redesign
- Editing anything under records/ or landings/
- Adding a new main-merge timestamp field to landing records
- Changing other throughput stages

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check status output on real records against independent commit times.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture test added; fmt, clippy and test required.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cde57ea0a4c2 → candidate 6b6736ff4152. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  1 +
 crates/athena-core/src/status.rs            | 13 +++++++++++++
 crates/athena-core/src/status/throughput.rs |  7 ++++---
 3 files changed, 18 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-231406-f1bx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-231406-f1bx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43571069667 for 6b6736ff4152dd5c1ab04c7e8994d350a8b18a20)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43571069667 for 6b6736ff4152dd5c1ab04c7e8994d350a8b18a20)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43571069667 for 6b6736ff4152dd5c1ab04c7e8994d350a8b18a20)
- ci-result: `not a check` passed (Check results for 6b6736ff4152dd5c1ab04c7e8994d350a8b18a20 come from the CI run https://github.com/doodla/athena/actions/runs/43571069667 (917s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No unrelated behavior changes: `git diff` touches only throughput.rs and CHANGELOG.md; cargo fmt --check, cargo clippy and cargo test pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
