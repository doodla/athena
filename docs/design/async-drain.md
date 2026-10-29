# An asynchronous, pipelined drain

Status: proposed (decision D145 in `docs/decisions.md`). Nothing here is built. Written for Sri, who accepts or
rejects it, and for the independent verifier who judges the document.

Related decisions: D98 (drain), D105 (planning), D112 (per-job build directories), D140 (self-rebuild after
landing), D111 (records branch), D46 (per-job directories), D10 (fast-forward landing), D72 and D87 (native PR
landing), D79 and D84 (sccache), D142 (environment check before the verifier), D143 (planning for hard jobs only).

## 1. The problem and the measured baseline

Today `athena drain` takes one queue item and runs it start to finish (read, plan if hard, producer, Athena's
checks, verifier, landing, self-rebuild) before it looks at the next. Every wait, whether on the model, on
`cargo`, or on CI, blocks the whole queue.

Measured by the user over 2026-10-27 to 2026-10-29 (the request's own figures; this design measured nothing):

| Stage | Minutes per landed job |
| --- | --- |
| Reading and planning | up to 5 |
| Producer | 15 |
| Athena's checks (fmt, clippy, test) | 13 to 15 |
| Landing: early CI wait, push, self-rebuild | about 6 to 8 |
| Verifier | 2.3 |
| **End to end, median** | **42** |

Over those two days drain landed 35 of 61 jobs (57 percent). The stage figures add to about 43 to 45 at their
midpoints against a median of 42: the median of a sum is not the sum of the medians, and the reading figure is a
ceiling. The rest of this document uses 42 as the baseline.

Two things follow. First, the stages use different resources: reading, producer and verifier wait on a model;
checks use the CPU and the disk; landing waits on CI and on a rebuild. A serial drain leaves each idle while the
others work. Second, this design raises throughput. It does not raise the landed share (35 of 61); a job that
fails still costs its stages. Section 8 shows both.

## 2. Design overview

A job moves through stages. Each stage has its own bounded pool, and jobs flow forward without waiting for the
job ahead to finish everything:

```
read-ahead -> producer pool -> checks -> verifier -> landing queue (one at a time) -> background awaits
   (R=2)        (P=2..3)        (own        (V=2)        rebase, push                CI, self-rebuild
                              worktree)
```

Proposed configuration, all in a new `[drain.pipeline]` table, none of it present today, so the default
`max_in_flight = 1` keeps today's behaviour exactly:

```toml
[drain.pipeline]
max_in_flight   = 1     # proposed default; 2 or 3 after the disk probe in job 4
read_ahead      = 2     # items read or planned beyond the one building
reserve_disk_gb = 20    # never start a producer if free disk after it would fall below this
```

The in-flight count is the number of jobs between "producer started" and "landed or closed".

## 3. Read-ahead and hard-job planning (point 1)

Reading (D98's reader) and planning (D105, run for hard jobs only since D143) read the repository and the queue;
they write nothing to the working tree. So they can run for items n+1 and n+2 while item n builds.

- A read-ahead task takes the next queue item, runs the reader, and for a hard reading the planner and critic. It
  stores the result as the item's reading, as the reading is stored today, tagged with the base commit it read.
- A reading is **stale** when main has moved on a path the reading names as a deliverable or a reference. At the
  moment a read-ahead item reaches the producer stage, Athena compares the reading's base commit with main. If
  none of the named paths changed, the reading is used as is. If any did, the reading is run again (cost: up to
  5 minutes, back on the critical path for that item only). This keeps the reading honest for items that touch
  the same files as the job ahead, which is also where the landing queue is likely to conflict.
- The user must still accept a reading before a job builds (the existing acceptance step). A reading that waits
  for acceptance does not block the items behind it; the pipeline skips it and takes the next accepted item,
  keeping queue order for everything else. The skipped item goes back in when accepted.
- Read-ahead is capped by `read_ahead` because each reading costs money and goes stale.
- Planning cost per reading is unchanged (D143: hard jobs only). Read-ahead only moves when it is paid.

Effect: reading and planning, up to 5 minutes today, leave the critical path for every item except the first
and the stale ones.

## 4. Early producer start in its own worktree (point 2)

The producer of item n+1 starts when item n leaves the producer stage, that is, when n's producer has returned a
result and n moves to checks. It does not wait for checks, verifier or landing of n.

- **Own worktree.** D46 and D112 already give each job its own directory and its own build directory. The
  pipeline makes that a hard rule: each in-flight job has `jobs/<id>/work` (a worktree of the repository at the
  job's base commit) and `jobs/<id>/target` (its build directory). Two in-flight jobs share no mutable path.
- **Base commit.** A producer starts from the current tip of main at the time it starts, not from the tip when
  the queue was read. A job in flight does not see its predecessors' changes until they land; section 5 handles
  the gap by rebasing at landing.
- **Shared build cache.** D79 and D84 put sccache in front of `rustc`. All in-flight jobs share one sccache
  directory and one cache size cap, so the second job's crates compile mostly from cache. Cargo's own target
  directory is not shared (shared target directories serialise on a lock and cross-contaminate artefacts).
- **Bounds.** A producer starts only if (a) in-flight count is below `max_in_flight`, and (b) free disk, minus
  this job's budget from section 9, stays above `reserve_disk_gb`. The existing low-disk check (D142's
  neighbour, the `low_disk` drain test) becomes the gate for (b). When neither is satisfied the item waits in the
  queue; there is no failure.
- **Check contention.** Athena's checks (`cargo test`, 13 to 15 minutes) use the machine's CPU. Two checks at once
  slow each other. The pipeline runs at most `max_check_slots = 1` checks at a time by default; the producer of
  the next job overlaps with the checks of this one, and a finished producer waits for the slot. This is the
  design's main throughput limit (section 8).

## 5. One ordered landing queue (point 3)

Verified candidates enter a single landing queue ordered by the time the verifier passed them. One lander works
at a time:

1. Take the head candidate. Fetch main. Rebase the candidate's commit onto the latest main in the candidate's
   worktree.
2. **Clean rebase.** Rerun the cheap checks that the rebase can invalidate: `cargo fmt --check` and `cargo build`
   (estimate: 1 to 2 minutes with the sccache). If the rebase moved no file the candidate also touches (only
   unrelated files changed on main), the full test suite is not rerun; the candidate's earlier verified result
   holds, and CI after the push is the full check (D10's fast-forward rule is kept: the pushed commit is a
   descendant of main). If the rebase brought in a change to a file the candidate touches, or to a Cargo
   manifest, the full checks run again before landing (13 to 15 minutes, off the critical path of other jobs but
   not of this one). Then land by the existing path (D72, D87: native PR landing).
3. **Conflict.** The candidate goes back to its **producer** as a new round of the same job, with the conflicting
   files, both sides of each conflict, and the commits on main that caused it, as a finding in the brief. The
   producer resolves on a fresh base of the current main, and the job re-enters checks and verifier. A conflict
   returns a job at most **2 times**; a third stops it (section 10, thrash stop).
4. The lander never resolves a conflict itself, and never pushes a candidate that did not pass checks on the
   exact tree it pushes.

Order guarantees: landings reach main in verifier-pass order; a candidate sent back for a conflict re-enters at
the tail, so it cannot hold up the ones that are clean.

## 6. CI waits and self-rebuild as background awaits (point 4)

Today landing waits through the early CI wait, then pushes, then waits for the self-rebuild (D140), all blocking.

- **CI.** After a push or the opening of a PR, the lander records an **await** (job id, PR or commit, what is
  waited on, a deadline) and moves to the next candidate. A background poller checks CI at a fixed interval
  (proposed 30 seconds, an estimate) and, when the status is final, writes the result.
- **Self-rebuild.** When a landed job changed Athena's own code (D140's rule), a rebuild await is recorded. The
  rebuild builds the new binary in its own target directory off the critical path. Drain keeps running in the old
  binary until the pipeline is **idle at a safe point**: no candidate is mid-landing. At that point it switches to
  the new binary and re-executes, carrying the pipeline state (section 7 describes the state file). Jobs in
  flight continue after the switch from their stored state. If the rebuild fails, drain keeps the old binary, logs
  the failure, stops landing further Athena-code jobs, and lets the rest drain (section 10).
- **What stays blocking.** Only two waits remain on a job's own critical path: the producer and its checks. A
  candidate is "landed" when CI is green on main after the push; until then it is in state `awaiting-ci`, counts
  against `max_in_flight`, and keeps its worktree (it might need to be fixed).
- **A red CI after push.** See section 10.

## 7. Records commits stay ordered (point 5)

Records (D111) are committed to the records branch. Today one drain writes them in the order events happen. With
several jobs in flight, two writers could interleave or race on the branch tip.

- **One writer.** All records commits go through a single writer task inside drain, fed by an in-memory ordered
  channel. Stage tasks do not commit; they send an event (job id, kind, payload). The writer makes the commit and
  pushes the records branch. There is no second writer, so there is no race on the tip.
- **Order.** Each event carries a per-drain sequence number from one counter, taken when the event is *sent*.
  The writer commits in sequence order. Within one job, its events are therefore in the order its stages
  produced them. Between jobs, the order is the order in which events were sent, which is a total order that
  every reader of the branch sees the same way.
- **Job identity in each commit.** Each commit's subject names the job id and the event kind, so interleaved jobs
  stay readable in `git log` and `athena status` can group them. A job's records are the commits that name it,
  not a contiguous run.
- **State for restart.** The writer also keeps `drain-state.json` in the drain's state directory (not in the
  records branch): in-flight jobs, their stage, the next sequence number, and the open awaits. A restart reads
  it, resumes from the stored stage, and continues numbering from the last committed sequence number found in
  the branch's commit trailers (`Seq: n`), so a crash between the commit and the state write loses nothing.
- **Working branch.** Records commits go to the records branch and never to a job's code branch, so they cannot
  conflict with the rebase in section 5.

## 8. Expected minutes per landed job

All target figures below are **estimates**. The baseline figures are the user's measurements (section 1).

Model: with P producers overlapping, the drain's steady state is limited by its busiest serial resource. The
resources: the model slot (producer, reading, verifier; parallel up to P), the check slot (1), the lander (1).

Per-job stage times, from the baseline at midpoints: producer 15, checks 14, verifier 2.3, reading 5 (hidden by
read-ahead except for stale items), landing split into a serial lander part and a background part.

Landing split (estimate): serial lander part 3 minutes (fetch, rebase, cheap rebuild, push); the rest of the 6 to
8 (CI wait, self-rebuild) is background: about 4 minutes.

Serial resources, per landed job:

| Resource | Minutes per job | Source |
| --- | --- | --- |
| Check slot (1) | 14 (plus 14 for a job needing a recheck after rebase; see below) | measured 13 to 15 |
| Lander (1) | 3 | estimate |
| Producer slots (P=2) | 15 / 2 = 7.5 | measured 15, divided by P |
| Verifier and reading | 2.3 and 0 (read-ahead) | measured 2.3 |

The check slot is the bottleneck: one job lands per 14 minutes at best. Two producers keep it fed (15 of producer
time per 14 of checks). Add a rebase recheck for the share of jobs whose rebase touched their files (estimate
20 percent, each costing 14 more on the check slot): 14 x 1.2 = 16.8. Add contention with the producer's own
`cargo` runs on the same machine (estimate 15 percent slower checks): 16.8 x 1.15 = 19.3. Round up for stale
rereads and idle gaps (estimate 5 percent): **about 20 minutes per landed job** at `max_in_flight = 3` (P=2, one
job in checks, one landing), against 42 today. If checks can run two at a time on a larger machine, the next
limit is the producers at 7.5 and the lander at 3, and about 12 minutes is plausible (estimate, not planned for).

What the number does not say:

- It is steady-state throughput. End-to-end latency of one job does not fall; it is 42 or longer, because a job
  queues for the check slot and the lander. Only the interval between landings falls.
- It assumes the 57 percent landed share (35 of 61) is unchanged. A failed job still uses a producer and a check
  slot. If the share stays, the cost per landed job is the same ratio against today's.
- Whether the machine can run a producer's `cargo` and checks together without slowing both is the largest
  unknown. Job 4 (below) measures it before `max_in_flight` is raised above 1.

## 9. Disk budget per in-flight job

Every figure here is an **estimate**; none was measured for this design. Job 4 measures them on the real machine
and the budget is set from that.

| Item | GB (estimate) |
| --- | --- |
| Worktree: source checkout, no history copy | 0.1 |
| Per-job `target` directory, debug build, after checks (D112) | 5.0 |
| Logs, outputs, records scratch | 0.3 |
| Share of sccache (one shared cache, capped at 10 GB, divided by 3 jobs) | 3.3 |
| **Per in-flight job** | **8.7, call it 9** |

In-flight limit: `floor((free_disk - reserve_disk_gb) / 9)`, capped by `max_in_flight`. With a 20 GB reserve and
80 GB free: floor(60 / 9) = 6, capped at 3 by the check-slot analysis. With 50 GB free: floor(30 / 9) = 3. With
35 GB free: floor(15 / 9) = 1, so the drain runs as it does today. Athena checks free disk before each producer
start (section 4) and again when a target directory is created. A finished job's `target` is deleted when the job
lands or closes (not at the producer stage), so a job that lingers in `awaiting-ci` holds its worktree but not
its build directory unless a fix is pending (estimate saved: 5 GB each).

## 10. Failure and conflict handling

| Case | What happens |
| --- | --- |
| **Rebase conflict** | The candidate returns to its producer as a new round with the conflict as a finding (section 5, step 3). Counts as a conflict return; it does not count as a failed round. |
| **Failed CI after push** | The await resolves red. If main is red, the landing queue **pauses** (no more pushes) until the cause is known. The job becomes `ci-failed`: its commit is reverted on main only if the failure names it and the revert is clean; otherwise the pipeline stays paused and a fix job for that candidate returns to its producer with the CI log. Candidates already verified behind it wait, then rebase onto the repaired main. Nothing is force-pushed. |
| **Failed self-rebuild** | Drain keeps running the old binary. It stops landing jobs that change Athena's own code (the ones that would need a new binary), finishes the rest, and records the failure. The failing job's code is already on main, so a fix job for the rebuild failure is queued first and flagged as blocking. |
| **Crash mid-flight** | On restart, drain reads `drain-state.json` and the records branch, and for each in-flight job resumes from its recorded stage: a job in the producer stage is restarted from its base commit; a job in checks reruns its checks; a candidate in the landing queue is rebased again; an await is resumed by polling again. A push that happened but was not recorded is found by asking the remote whether the candidate's commit is on main, so it is never pushed twice. Orphan worktrees and target directories found with no state entry are removed. |
| **Thrash stop** | A job stops, with a reason, when it has returned for conflict more than 2 times, or failed its rounds as today, or when the pipeline as a whole has had 3 consecutive landings returned for conflict, the last case suggesting the queue holds items that touch the same files. The whole pipeline then drops to `max_in_flight = 1` for the rest of the drain and writes the reason. It does not retry by itself. |
| **Low disk** | No new producer starts; jobs in flight finish. Matches today's low-disk stop. |
| **Producer timeout** | As today for one job. The job leaves the pipeline; the slot is free for the next. |
| **Stale reading** | Section 3: read again before the producer starts. |

## 11. Alternatives considered

- **(a) Run several drains in parallel on separate queues.** Rejected: they share one main, one records branch and
  one disk, and they would conflict at landing with no order. The design in this document is the same idea with
  one order.
- **(b) Keep the drain serial and make each stage faster** (a faster check, a smaller test run). Possible and
  worth doing, but it cuts the largest stage, 13 to 15 minutes, by only a fraction, and leaves reading and
  waiting on the critical path.
- **(c) Batch several candidates into one landing.** Rejected: one failing candidate holds the others, and the
  per-job landing records (D10, D72) are the audit trail.
- **(d) Let the lander resolve conflicts.** Rejected: a conflict is a design question for the producer's tools and
  its tests; the lander has neither.
- **(e) One shared target directory for all jobs.** Rejected: it serialises on the cargo lock and mixes
  artefacts of different bases.
- **(f) Stack candidates on each other (job n+1 built on job n's branch).** Rejected: if job n fails, n+1 must
  be redone; the base-on-main rule keeps jobs independent.
- **(g) Several records writers with a merge step.** Rejected: one writer gives a total order for free.
- **(h) Skip the recheck after a rebase.** Rejected: a clean textual rebase can still break the build; section 5
  limits the recheck to the cases where it can matter.

## 12. Build jobs, in order

Each is sized for one producer run (a producer run is about 15 minutes of model work and one reviewable diff).
Each ships behind `max_in_flight = 1`, so drain behaves as today until job 7 raises it.

1. **Pipeline state and the single records writer.** Add the ordered event channel, `Seq` trailers,
   `drain-state.json`, and the restart read, with the drain still serial. Tests: events from two fake jobs
   commit in sequence order; a restart continues numbering. Depends on: none.
2. **Stage pools and `[drain.pipeline]` config.** Split drain's per-job function into stage tasks, with
   `max_in_flight`, `read_ahead`, `reserve_disk_gb`, and `max_check_slots`; at 1 it must give the same results
   and records as now. Depends on: 1.
3. **Read-ahead.** Run the reader and the hard-job planner for the next items, with the stale check against the
   base commit. Depends on: 2.
4. **Disk probe and budget.** Measure worktree, `target` and sccache sizes for one job, replace the estimates in
   section 9, and add the free-disk gate before a producer start and the clean-up of a finished job's `target`.
   Depends on: 2.
5. **Early producer start.** Start the next producer when the current one leaves the producer stage, with its own
   worktree and target directory, the shared sccache, and the check slot. Depends on: 2, 4.
6. **Ordered landing queue.** Rebase onto the latest main, cheap recheck, the touched-files rule for the full
   recheck, and the conflict return to the producer with the conflict as a finding. Depends on: 2, 5.
7. **Background awaits.** CI poller and awaits, `awaiting-ci` state, the paused queue on red main. Depends on: 6.
8. **Background self-rebuild and safe-point switch.** Move D140's rebuild to an await and switch binaries when
   the pipeline is idle. Depends on: 7.
9. **Failure handling and thrash stop.** Crash resume per stage, orphan clean-up, the conflict-return limit, the
   consecutive-conflict drop to one in flight. Depends on: 6, 7, 8.
10. **Raise the default and measure.** Set `max_in_flight = 2`, drain a real queue, and compare minutes per landed
    job with the 42-minute baseline; raise to 3 only if disk and the check slot allow. Depends on: 9, and Sri's
    acceptance of the measured result.

Jobs 3 and 4 can run in either order once 2 is in; the rest follow the listed order.

## 13. What this design does not do

- It does not change the landed share, the verifier's rules, or any check.
- It does not change `athena.toml` or any workflow; the configuration above is a proposal for job 2.
- It does not use a second machine; section 8 notes the limit that implies.
