# Continuous landing: one drain-owned step from verifier pass to landing record

Status: proposed. Nothing here is built. Written for the Athena maintainer and for the next throughput review.

Source: throughput review `20261104-192053-745k`. **An implementing change must cite review 20261104-192053-745k in
its request text**, so that the next throughput review can set the minutes it measures after that change landed
against the prediction in section 3.

## 1. Problem

Athena's throughput section (`THROUGHPUT.md`, the last 20 landed jobs, measured) gives a median of **528.2 minutes per
landed job**. Of these, **499.1 minutes are landing and rebuild**. The other five stages add up to about 32 minutes:
reading 0.8, planning 4.1, producer 24.2, checks 0.0, verifier 2.9. No change inside those five can move the total
by more than a few percent.

"Landing and rebuild" is defined in `crates/athena-core/src/status.rs` (`StageMinutes`): the time from the end of the
job's last invocation (the verifier) to the later of the landing outcome and the landing record (`landings/`, written
after the merge). So a job that passed the verifier but has not yet merged is counted as waiting, minute by minute.

The review's reading of the cause: a verified job's merge is noticed by a later command, not by the step that
verified it. The job sits between "verifier passed" and "landing record exists" until that command runs. This note
takes that reading from the review. It was not re-measured here; the split of the 499.1 minutes between idle waiting
and actual landing work (rebase, CI wait, merge) is not in `THROUGHPUT.md`. Section 7 makes the next review record it.

## 2. Existing code

The design builds on the following and adds no new mechanism for what they already do.

| Piece | What it already provides | How this design uses it |
| --- | --- | --- |
| `docs/design/async-drain.md` | An ordered landing queue (section 6: first in, first out in the order runs pass, one landing at a time); the rebase and the re-check rule (`Steps::rebase`, `rebase_needs_recheck`, `Steps::rerun_checks`); the landing retry; records committed in the queue slot after the landing commit (section 8); CI waits as background awaits (section 7); the failure table (section 10). Its status says the queue wiring of the rebase is a follow-up. | Extended: the queue slot is made to own the whole sequence through the landing record, with no later command. |
| `docs/design/native-pr-landing.md` | Landing by pull request: push `athena/<job>`, open the PR, read the checks, merge by REST only when `[landing] merge_via_rest` is true, and on a later run find the merge by the stored PR number and run the merged-content check (landed, or `merged-unverified`). | Reused as the merge mechanism. The "run the command again to find the merge" step is the one this design makes the drain do on its own. |
| `crates/athena-git/src/lib.rs` | The git primitives: `replay` (rebase by replaying commits onto a base), `merge_tree` (conflict test without touching the work tree), `merge_ff_only`, `is_ancestor`, `fetch_remote_branch`, `push_branch`, `commit_with_records`, `squash_text`. | Reused unchanged for rebase, conflict detection, "already landed?" tests and the records commit. |
| `crates/athena-core/src/gh/landing.rs` | `plan_rebase`, `rebase_job_branch` and `land_pr`, including the merged-content check. | Called by the queue slot in order. |
| `crates/athena-core/src/status.rs` | The throughput stages and the `landed_at` rule. | Unchanged; it is the measuring instrument (section 7). |
| `docs/design/squash-chain-landing.md`, `docs/design/parallel-jobs.md` | Landing of stacked commits and the concurrency model. | Read for constraints (stacked commits ride along; disjoint-paths rule). Not changed. |

## 3. Predicted saving

- Baseline (measured, `THROUGHPUT.md`, last 20 landed jobs): median 528.2 minutes per landed job, landing and rebuild
  499.1 of them.
- **Predicted saving: 470.0 minutes per landed job** (an estimate, from the review). The median would fall from
  528.2 to about 58.2 minutes per landed job, that is the 32-odd minutes of the other stages plus about 29 minutes of
  landing and rebuild.
- Where the remaining 29 minutes (estimate) come from: the real landing work, which `async-drain.md` section 2 puts
  at 6 to 8 minutes for rebase, CI wait and merge, plus queue waiting behind other verified jobs, plus the self-rebuild
  when a landed job changes Athena's own code. The 29 is the 499.1 minus 470.0 and is not a separate measurement.
- If the saving is under 470.0, the shortfall is the idle share that stayed, and section 7 says how it is found.

## 4. Design

One queue slot owns the whole sequence. When the verifier passes, the job joins the drain's landing queue (the
existing `LandingQueue`). The head of the queue runs these steps without returning control to any other command:

1. **Rebase** the job branch on current main (`rebase_job_branch`). Already current: nothing moves.
2. **Verify again only when needed**: re-run checks if `rebase_needs_recheck` says the rebase touched the job's paths
   or a manifest (`Steps::rerun_checks`).
3. **Merge**: push, open the pull request, wait for CI as a parked await, merge (`land_pr`), then run the
   merged-content check.
4. **Landing record**: commit the job's closing records and the landing record in the same slot, directly after the
   landing commit (`async-drain.md` section 8).
5. **Release the slot** and start the next queue head. If the landed job changed Athena's own code, the self-rebuild
   runs at the next safe point (`async-drain.md` section 7).

How this removes the wait: today the steps after the verifier are separated by gaps where nothing is running and the
job is waiting for a later command to take the next step or to notice the merge. In this design a step finishes by
starting the next step in the same slot, and the slot ends only when the landing record exists. The time between
"verifier passed" and "landing record exists" becomes the sum of the working steps and the wait for the slot, with
no idle gap that depends on a later command.

State: the drain writes the job's stage as `landing` in its in-flight table when it takes the slot and at each step
(`async-drain.md` build job 3), so a stopped drain can resume at the step it was in.

A drain that cannot run (stopped, or the job was verified by a hand-run command) still has a fallback: the
existing `athena land <job>` command performs the same steps for one job. It shares the code of the slot, so the two
cannot disagree.

## 5. Alternatives considered

| Option | Why not |
| --- | --- |
| a. Run `athena land` on a timer or from the sweep more often | Shortens the gap but keeps the design: a second actor must notice the merge, with the race between it and the drain. The wait would shrink to the timer interval, not to zero, and two actors would need a lock. |
| b. Let the forge merge queue or auto-merge land the job | Adds an outside dependency, does not return conflicts to the producer, and Athena still has to notice the merge to write the landing record (same problem, `async-drain.md` section 9c). |
| c. Land inside the verifier's own invocation | Mixes a judge with an actor: the verifier would hold write access to main. Rejected on role grounds. |
| d. Write the landing record at verifier pass, before the merge | Makes the number small by changing the record, not the pipeline, and a record of a landing that has not happened is false. |
| e. Raise `[parallel] max_concurrency` | Does not touch the wait for a job already verified (`async-drain.md` section 9a). |

## 6. Risks and failure paths

| Case | Behavior |
| --- | --- |
| Rebase conflict in the job's own commit | The job is held back; nothing lands; main and the local branch are untouched; the next queue entry still lands. The job returns to its producer with the conflict text, as `async-drain.md` section 6 steps 3 and 4 say, and re-enters behind waiting jobs. After `max_conflict_returns` returns it closes as an environment-type stop with the paths in conflict. |
| Ordering | One slot at a time, in the order runs pass. A later job never merges before an earlier head. A parked CI wait keeps the slot, so main stays linear. The cost: one slow CI run delays every job behind it. Risk accepted; the 470.0 estimate does not rely on parallel landing. |
| Failed landing (CI red, merge refused, `merged-unverified`) | The job returns to its producer with the log once; a second failure closes it. The slot is released, so a failed job never blocks the queue. A `merged-unverified` result is kept as such and stops further landings until it is resolved. |
| Drain stops or crashes mid-slot | On restart the in-flight table shows `landing`; the drain first asks whether the branch already landed (`is_ancestor`, the stored PR number) before it repeats any step. A merge is never made twice. |
| Self-rebuild fails | Landing is held and the drain stops with a message, as `async-drain.md` section 7 says. |
| Stacked unlanded commits ride along in the PR | Known limit from `async-drain.md` section 6; unchanged by this design. |

**Rollback.** The change ships behind a setting in the `[drain.pipeline]` table (a proposed key, for example
`continuous_landing`, default off at first). Turning it off returns the drain to the present behavior, where a verified
job waits for `athena land`. No record format changes, so jobs landed either way read the same. Turning it on
by default is a separate decision that needs the owner's yes, because it edits `athena.toml`, a governance file; that
edit is not part of this note.

## 7. How the next throughput review measures the saving

- The measurement is the one that produced the baseline: the median `landing and rebuild` minutes and the median total
  minutes per landed job over the last 20 landed jobs, from `THROUGHPUT.md` and the `athena.throughput-review/1`
  record. It is read by `StageMinutes::median_of` in `status.rs`, the same definition as the baseline.
- The review uses only jobs that landed **after** the implementing change landed. With fewer than 20 such jobs, it
  reports the count and the median of those, labeled as a small sample, and does not claim the full window.
- It compares the measured total with the baseline 528.2 and the predicted 58.2, and the measured landing and rebuild
  with 499.1 and the predicted 29. Saving measured = 528.2 minus the new median total; the prediction is met when that
  is at least 470.0 within the sample's noise.
- It finds the implementing change through the citation of review `20261104-192053-745k` in the request text.
- To separate idle waiting from working time, the implementing change should record the time the slot was taken and
  the time each step ended. This note leaves the format to that change; without it the split stays unmeasured.
- Failure to meet the prediction is reported as such, with the stage that kept the minutes.

## 8. Out of scope

Implementing any of this in `crates/`, changing `athena.toml`, `defaults.toml` or the workflows, and editing
`THROUGHPUT.md` or `records/`.
