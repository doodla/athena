# Continuous landing inside the drain: design for the landing and rebuild stage

Status: proposed (D155). Nothing here is built. Nothing in `crates/`, `xtask/`, `athena.toml`, `defaults.toml` or
`.github/workflows/` changes with this note. Written for the Athena maintainers and for the next throughput review,
which scores the prediction in section 5.

Source: throughput review `20261105-070420-vq95`. It named landing and rebuild as the biggest wait and proposed
making landing a continuous step of the drain. The figures below are cited to the text of the request that review
queued (job `20261105-070552-kx1y`); `THROUGHPUT.md`, the status section the review read, is not in the evidence
snapshot of this job, so no figure was re-measured here.

Labels used below: **measured** (a figure from the review request, not re-measured), **prediction** (the review's
expected saving), **estimate** (reasoned here, not measured), **existing** (built; file named), **new** (not built).

Related: throughput review `20261108-183648-5eya` named a second cause of the same stage, pull requests that carry
other jobs' commits from a stacked local branch. It is designed in `per-job-landing-branches.md`. The two changes
are independent in code; the later review attributes a drop to the phase it came from.

## 1. Existing code

Nothing in this table is created by this note. Each row says whether the design **reuses** it as it is or
**changes** it, and how.

| What exists | Where | This design |
| --- | --- | --- |
| One ordered landing queue: jobs land one at a time, in the order their checks and verifier passed; the head rebases, a clean rebase lands, a conflict holds the job back | `docs/design/async-drain.md` section 6 | **Reused** as written. The only new rule is how long a job may stay at the head (section 3, step 3). |
| CI wait as a background await; the queue head is parked and the next entry does not merge until it lands; self-rebuild at a safe point with no job in a landing or records stage; a failed rebuild holds landing | `docs/design/async-drain.md` section 7 (D140) | **Reused** for the parked wait and the failed-rebuild rule. **Changed**: the rebuild runs once per batch at a bounded boundary instead of after each landing (section 3, step 5). |
| Native pull-request landing: one pull request per job, a wait for green checks, a merge through the host's REST endpoint only with `[landing] merge_via_rest` on, a landing-intent record before the merge, a merged-content check after it | `docs/design/native-pr-landing.md` section 5 (5.1, 5.2, 5.2a) | **Reused** as the "automatic merge". `athena.toml` line 189 sets `merge_via_rest = true`. No new merge path and no host setting is needed. |
| `LandingQueue` (first in, first out, `join` when a run passes, `is_head`, `leave`) and `LandingSlot` | `crates/athena-cli/src/drain.rs` (about line 1236) | **Reused**. A worker already joins when its run passes. **Changed**: the worker is freed at that point and the queue head is driven by the drain, not by the worker's own thread (build job 2). |
| `Drain::land`: takes the `landing` lock, runs the re-check after a rebase, calls `Steps::land`, and records the paths landed and `rebuild_due` | `drain.rs` `land` (line 1714) | **Reused**. **Changed**: the lock is no longer held for the whole wait on checks when the job is only waiting (build job 3). |
| `land_pr` with the merge gate (`merge_via_rest` and green checks), the refusal handling for a merge the host rejects while a check runs, and `wait_for_checks` | `crates/athena-core/src/gh/landing.rs` (about lines 228 to 300) | **Reused** unchanged. |
| `Landing::Waiting`, `landing_from_report` and the later `athena land --pr`: a poll that ends with the pull request open and its checks pending is `Waiting`; the job is marked "waiting to merge", `work` returns `ItemEnd::Waiting`, the item is passed over, and a separate `athena land --pr` finishes it | `drain.rs` (`landing_from_report` line 3136, `ItemEnd::Waiting` lines 1860 and 2511); module comment lines 123 to 130 | **Changed**. A `Waiting` job stays in the ordered landing queue and is polled again inside the drain, within a time limit (section 3, step 3). A merge the host refused is told apart from pending checks and goes to the user at once. |
| `PrLander` and the checks poll: thirty looks, twenty seconds apart (10 minutes) | `drain.rs` `PrLander`; `crates/athena-cli/src/wiring.rs` `checks_poll` (lines 41 to 48) | **Reused**. The 10-minute window is the unit for the limit H. |
| The landing record (`landings/`, written after the merge) and the landing-intent record on the records branch | `native-pr-landing.md` 5.2; `crates/athena-core/src/status.rs` `landing_end` | **Reused**. The landing record is still written at merge confirmation, so the review's span still ends where it does today. |
| `rebuild_if_due` and `finish`: after each `ItemEnd::Landed`, take the `landing` lock and, if `rebuild_due`, run `setup.sh` in the checkout; the lock is held because a landing moves the checkout the build reads | `drain.rs` (`rebuild_due` set at line 1726 and taken at 2404; `rebuild_if_due` lines 2402 to 2424; `finish` line 2428) | **Changed**: only when it is called (section 3, step 5). The lock and the invariant stay. |
| Rebase before landing, `max_conflict_returns`, the re-check rule `rebase_needs_recheck`, and `recheck_after_rebase` | `async-drain.md` section 6; `crates/athena-core/src/gh/landing.rs` (`plan_rebase`, `rebase_job_branch`); `drain.rs` line 1716 | **Reused**. The conflict hold wiring is a dependency of the build jobs (section 9). |
| The in-flight table (job id, stage, worktree), written at each stage change | `async-drain.md` section 7, build job 3 (D145) | **Reused**. A job waiting in the queue is one more stage value; nothing else changes. |

## 2. Problem: the 440.1 minutes

**Measured** (review `20261105-070420-vq95`, cited from its request text): the median is 464.5 minutes per landed
job. Landing and rebuild takes a median of 440.1 of them. Reading, planning, producer, checks and verifier together
take about 21 minutes. Idle time is about 44 minutes per landed job (16070.4 / 366). The medians are per stage, so
they do not add exactly to the median of the total; 440.1 + 21 is 461, a little under 464.5.

The stage is defined in `crates/athena-core/src/status.rs` (`StageMinutes`): from the end of the job's last
invocation (the verifier, which runs after the checks) to the later of the landing outcome and the landing record
(`landings/`, written after the merge). The review compares medians over the newest 20 landed jobs
(`THROUGHPUT_WINDOW`) with `StageMinutes::median_of`. `docs/design/hestia.md` records that no record separates the
rebuild from the rest of this stage.

What today's code does after the verifier passes (read in `drain.rs` and `wiring.rs`):

1. `Drain::land` takes the `landing` lock and holds it for the whole land call, which includes the wait on the
   pull request's checks: up to thirty looks, twenty seconds apart (10 minutes).
2. A poll that ends with the pull request open and its checks pending is `Landing::Waiting`. The job is marked
   "waiting to merge", the item is passed over, and the drain goes on. A later, separate `athena land --pr` merges
   it. Nothing in the drain brings the job back.
3. After each landing that touched the build paths (`touches_build`), `finish` calls `rebuild_if_due`, which takes
   the same `landing` lock and runs `setup.sh`. Every landing waits while it runs.

The median CI run is 583 seconds (`warm-baseline-build.md` section 2, **measured**), so a wait on checks is about
10 minutes. A 440-minute median is about 45 times that (440.1 x 60 / 583). A wait that long is not one CI poll. It is most likely the
time a job spends behind others, or in `Waiting` until someone runs `athena land --pr`. This is an inference from
the code: no record in the snapshot splits the 440.1 minutes by phase (section 9, build job 1 produces that split).

## 3. Design

Landing becomes a continuous step of the drain. The unit of work is the same; what changes is who waits.

1. **Join at the verifier.** When the verifier passes, the job joins the one ordered landing queue
   (`async-drain.md` sections 6 and 7) and its worker is free. The worker starts the next reading, producer or
   check at once, bounded by `max_in_flight` and the disk guard as today. Order is first in, first out by the
   moment the verifier passed.
2. **Rebase, then merge.** The queue head rebases on current main (section 6 of `async-drain.md`, with its re-check
   rule). Then `land_pr` waits for the checks and the pull request merges automatically by the existing REST path
   (`merge_via_rest = true`): as soon as the checks are green, in the same locked step. Athena does not depend on
   the host's own auto-merge setting. The landing-intent record is written before the merge and the merged-content
   check runs after it, as today.
3. **A pending poll stays in the queue.** A job whose poll ends pending (`Landing::Waiting`) stays in the ordered
   landing queue and is polled again inside the drain. It may hold the head for a limit **H** (initially one more
   poll window, 10 minutes; a parameter, not a measured value). After H it steps aside and keeps its place in
   order: it is polled again behind the job that lands next and re-enters the head once its checks are green. After
   a second limit it goes back to the user with its pull request open, as it does today. A merge the host refused
   under its repository rules is mapped to `Waiting` today (`landing_from_report`); a build job tells it apart from
   pending checks by the report's `checks` field and sends it to the user at once, without waiting H. This limit
   is what keeps one stuck job from blocking every later landing (section 7, head-of-line blocking).
4. **The landing record is written at merge confirmation**, as today. The review's span still ends at the later of
   the outcome and that record.
5. **The rebuild keeps its lock and runs less often.** `rebuild_if_due` holds the `landing` lock because a landing
   moves the checkout the build reads (the comment on `rebuild_if_due`). That invariant stays: no landing overlaps a
   build. What changes is when the call is made. Today `finish` calls it after every landed job. The `rebuild_due`
   flag already merges several landings into one build (`|=` in `Drain::land`, `take` in `rebuild_if_due`), but
   with one landing at a time the flag is usually taken before the next landing. The design calls `rebuild_if_due`
   at a batch boundary: no verified job is in the queue and no job is in a landing or records stage (the section 7
   rule of `async-drain.md`, D140), or at most **K** landings or **M** minutes after the first due flag, whichever
   comes first, so a busy queue cannot starve the rebuild and the installed binary does not go stale without limit.
   K and M are parameters set after build job 5 measures the rebuild.
   - Lock time saved = (rebuilds today - rebuilds after batching) x R, where R is the duration of one `setup.sh`
     run. R is not recorded (`hestia.md`), so this is a formula, not a number. Build job 5 makes R measured.
   - Cost: the drain runs longer on the old binary, and landings in a batch land on it.
   - A failed rebuild keeps D140: landing is held, no new producers start, and the drain stops with the build
     output. Nothing lands on a main the drain cannot build.

## 4. How it removes the bottleneck

Measured: landing and rebuild is 440.1 of the 464.5 minutes; the other five stages are about 21 minutes together.
Making the worker free at the verifier and driving the merge from the queue changes three waits that sit inside the
440.1:

- **Wait to be reached.** A job that is verified no longer waits for a worker that is busy with a later job's
  producer or checks. The queue head is driven by the drain.
- **Time in `Waiting`.** A pending poll no longer ends in a pass-over that only a later `athena land --pr`
  finishes. The job is polled again inside the drain, and its limit H keeps it from blocking others. This is the
  likeliest home of most of the 440.1 minutes, since one poll window is 10 minutes (estimate; build job 1 measures
  it).
- **Lock time.** The rebuild runs once per batch, so landings wait on the lock for fewer builds. This helps the
  spans of the jobs behind a rebuild, not the span of the job that triggered it: that job's own rebuild runs
  after its landing (`finish` calls it after `ItemEnd::Landed`), so this note does not count it as a measurement
  effect for that job. Whether the outcome record is written before or after the rebuild is not read here; build
  job 1 checks it.

The change is not a faster CI wait. The CI wait is one window of about 10 minutes against a stage more than 20 times
the median CI run. It removes the waits around the CI wait. The saving depends on how the 440.1 minutes split
by phase, which no record shows today; that is why build job 1 is a measurement.

## 5. Predicted saving and how it is measured

**Prediction** (review `20261105-070420-vq95`, not a measurement): 420.0 minutes per landed job. The median was
464.5 minutes per landed job when the review ran.

The implied remainder is 464.5 - 420.0 = 44.5 minutes (estimate): about 21 for the other five stages and about 23.5
left for landing and rebuild, which is two or three CI poll windows.

**How the next throughput review measures it.** It takes the median over the newest 20 landings
(`THROUGHPUT_WINDOW`) with `StageMinutes::median_of`, the same function as the review that made the
prediction, so the two compare like with like. It reports:

- the total median minutes per landed job from the verifier's end to the landing record, with the other stages,
  compared with 464.5;
- the landing and rebuild stage compared with 440.1;
- the saving as 464.5 minus the new total, set against 420.0.

It uses only landings made after the change landed, so the window must be full of post-change landings before it
counts. The rebuild adds to later jobs' spans through the landing lock, so the review also reports rebuild time
separately: the duration R of each run and the number of runs in the window (build job 5 adds the record that
makes this possible). A drop is credited to this change only if the phase split of build job 1 shows it came from
the phases in section 4.

## 6. Alternatives considered

| Option | Why not (or why later) |
| --- | --- |
| Raise `[parallel] max_concurrency` only | `async-drain.md` section 9, alternative a: jobs land racing, with no order and no conflict return. It does not touch the queue or the passed-over `Waiting` jobs. |
| Enable the host's native auto-merge on the pull request at enqueue | It needs a repository setting the code does not show to be on (open question for the user). The merge would happen outside the ordered queue, so order and the landing-intent write would no longer hold. |
| A new merge code path for the drain | It duplicates `land_pr` and its safety checks for no saving. |
| A shorter CI wait | It does not address the queue, the passed-over `Waiting` jobs or the lock wait, in a stage more than 20 times the median CI run. |
| A warm baseline build for the checks | Estimated at about 0.6 to 1.1 minutes per landed job and not recommended now (`warm-baseline-build.md`). It acts on the checks stage, not this one. |
| Strict first-in, first-out at the head with no time limit | A `Waiting` job that never clears (a refused merge, a CI run that never finishes) blocks every later landing. The limit H is part of the design. |
| Build from a fixed commit in a separate worktree so landings carry on while it builds | Removes the lock, but needs a second cold `target/` (the disk budget of `async-drain.md` section 5) and changes where `setup.sh` runs and what it installs. Recorded as a later option, taken only if build job 5 shows the batched lock time is still large. |
| Drop the lock and run the rebuild beside landings | A landing would move the checkout under the build. This breaks the invariant above. |
| Count part of the 420.0 minutes as a measurement change because the rebuild leaves the span | The code runs a job's own rebuild after its landing, and no record separates the rebuild, so the share is unmeasured. Not claimed. |
| Do nothing | The review's data says the stage is more than 20 times every other stage together. Leaving it is the choice that keeps 464.5 minutes. |

## 7. Risks

- **Ordering.** Releasing the worker at the verifier lets later jobs join the queue while an earlier one waits on
  CI, and they could land out of order. Mitigation: one landing at a time under the existing ordered landing queue,
  first in, first out by verifier pass. The merged-content check in `land_pr` stays as the safety net. The only
  departure from strict order is a `Waiting` job stepping aside after H, which keeps its place in order.
- **Head-of-line blocking.** A `Waiting` job at the head can stall every later landing, and a merge the host
  refused for its repository rules may never clear. Mitigation: the limit H, then a second limit that returns the
  job to the user with its pull request open, and the refused-merge path that returns it at once. A test for each
  is part of build job 3.
- **Rebuild failure and the checkout invariant.** The rebuild reads the checkout and a landing moves it.
  Running the rebuild beside landings would build a moving tree. Deferring it too long leaves a stale binary. A
  failed rebuild could let jobs land on a main Athena cannot build. Mitigation: the lock stays, the rebuild runs
  at a boundary bounded by K landings or M minutes, and a failed rebuild keeps D140. Build job 4 carries tests
  that no landing overlaps a build, that the bound forces a rebuild under a busy queue, and that a failed
  rebuild holds landing.
- **Merge conflicts.** A job waiting in the queue meets a moved main. Mitigation: the section 6 rebase, the
  held-back conflict case, `max_conflict_returns` and the re-check rule are reused. The real conflict-hold wiring
  of `async-drain.md` (a follow-up there) is a dependency of build jobs 2 and 3.
- **The saving is misread.** A job's own rebuild already falls after its landing record, so the rebuild shows only
  as lock contention in later jobs' spans. Mitigation: section 5 labels 420.0 as a prediction, derives the remainder,
  asks the review to report rebuild time separately, and uses only landings after the change.
- **Wrong phase.** The 440.1 minutes may be mostly the wait before the drain reaches the job, or time in `Waiting`
  before a later `athena land --pr`, and the design removes less than predicted. Mitigation: build job 1 measures
  the split first, and H, K and M are set from it.
- **Stale binary.** Batching keeps landings on the old binary for longer. Mitigation: the K and M bounds, and the
  existing check that the running build can read a landed config (`config_after_landing`) still runs after every
  landing.

## 8. Out of scope

- Any change to `crates/`, `xtask/` or other code: this note is a design only.
- Any change to `athena.toml`, `defaults.toml` or the workflows, including turning on a host auto-merge setting.
- Editing or re-running throughput review `20261105-070420-vq95`.
- A separate-worktree rebuild, the warm baseline build, and a faster CI wait (section 6).
- Changing the stage definition in `status.rs`, so that the next review compares like with like.

## 9. Ordered build jobs

Each job is one focused change with its own tests. None changes a limit before its predecessors' numbers exist.

1. **Measure the 440.1 minutes by phase.** Wait for the drain, queue wait, lock wait, CI poll, time in `Waiting`
   before a later `athena land --pr`, merge, and rebuild; also whether the outcome record falls before or after the
   job's own rebuild. Output: numbers in a records note, no behavior change. H, K and M are set from it.
2. **Free the worker at enqueue.** Depends on: 1. The worker joins the ordered landing queue and moves on; the
   drain drives the head.
3. **Merge by the queue head, keep `Waiting` jobs in the queue.** Depends on: 2. The limit H, the second limit,
   and the refused-merge path; tests for each, and that the `landing` lock is not held while a job only waits.
4. **Batch the rebuild.** Depends on: 1. Call `rebuild_if_due` at the batch boundary with the K and M bounds;
   tests that no landing overlaps a build, that the bound forces a rebuild under a busy queue, and that a failed
   rebuild holds landing.
5. **Record the rebuild separately.** Depends on: none. The duration R of each `setup.sh` run and the number of
   runs per window, so the next review can report them. K and M are set from it.

## 10. Citation duty

A change that implements this design must cite throughput review `20261105-070420-vq95` in its request text. The
next throughput review then sets the minutes it measured after that change landed against the 420.0-minute
prediction, by the rule in section 5.

## Questions for the user and the host

- Is the host's native auto-merge enabled on the repository? The code does not show it. The design does not
  depend on it, but the alternatives table needs the fact to be exact.
- What share of the 440.1 minutes is each phase of build job 1? The snapshot has no records and no
  `THROUGHPUT.md`, so this note cannot say.
