# Async drain: several jobs in flight, one ordered landing queue

Status: accepted (D145, decided by the brain under Sri's delegation, 2026-10-29). Nothing here is built. Written for Sri and for the verifier of the build jobs in section 12.

The drain runs one job at a time today (`[parallel] max_concurrency = 1` in `athena.toml`). Over 2026-10-27 to 29, 35 of
61 jobs landed and the median job took 42 minutes. This design overlaps the stages of different jobs so that the
slow shared resources, the check slot and the landing queue, are the only serial parts.

## 1. Premises checked

Each dependency below was read in the repository before this design was written. The build cache row is the one a
previous attempt got wrong, so it is first.

| Dependency | Current state | Where |
| --- | --- | --- |
| Build cache | **Off.** The sccache wiring was removed after a report of 0 hits; the cause was not reproduced. Support code for an optional pin remains, unused. No `*.toml` in the repository names sccache. | D84; D79, D80 (removed by D84); `athena.toml` has no `[tools.sccache]` |
| Drain | Runs one job at a time unless `[parallel] max_concurrency` is raised (schema allows 1 to 8, set to 1). It rebuilds Athena after landing its own code. | `crates/athena-cli/src/drain.rs`, `athena.toml` `[parallel]`, D140 |
| Producer | One agent run per job in the job's own directories, with a per-job `CARGO_TARGET_DIR`. Hard jobs get a stronger setting. | D46, D136, `athena.toml` `[capabilities.change.env]` |
| Verifier | Runs after the checks on a fresh snapshot, and the environment is checked again right before it. | D142 |
| Disk guard | `[disk] min_free_gb` (default 10) checked at dispatch, before the checks and before the verifier; idle jobs' `target/` is removed first. | `crates/athena-core/src/disk.rs`, `crates/athena-data/src/config.rs` `DEFAULT_MIN_FREE_GB`, D142 |
| Config limits | `[parallel] max_concurrency` 1 to 8; `[job_plan]` plans hard jobs only. No `[drain.pipeline]` table exists yet. | `athena.toml`, D143 |
| Planning | Runs for hard jobs only, so reading and planning is short for most jobs. | D143 |

Consequence: every number in this design is computed with **no shared build cache**. If the cache is re-enabled, that
is a separate, costed option (section 9, alternative e), not an assumption of this design. Figures that would change
with a cache are marked *conditional*.

## 2. Baseline (measured, 2026-10-27 to 29)

Source: the request for this design, from the drain's records. Not re-measured here.

| Stage | Minutes per job |
| --- | --- |
| Reading and planning | up to 5 |
| Producer | 15 |
| Checks (fmt, clippy, test) | 13 to 15 |
| Verifier | 2.3 |
| Landing (rebase, CI wait, merge) | 6 to 8 |
| **Median job, start to finish** | **42** |

Outcome: 35 of 61 jobs landed (57%). The stage times sum to about 43 with mid-range values (5 + 15 + 14 + 2.3 + 7),
which agrees with the 42 median.

Per landed job, serial today: 42 x 61 / 35 = **about 73 minutes** (estimate: assumes a failed job costs the median
too; failed jobs stop earlier, so the true figure is somewhat lower).

## 3. Target and arithmetic

Unit: wall minutes of drain time per landed job, in a steady stream of queued work. Ratio of attempts to landings:
61 / 35 = 1.74 attempts per landing.

Which stage each attempt uses:

- Check slot: every attempt that reaches checks. Assume all 61 do (a conservative estimate; some fail earlier). Cost per
  landing: 14 x 1.74 = **24.4 min** with one slot.
- Landing queue: only landed jobs, 7 min each. Cost per landing: **7 min**.
- Producer, reading, verifier: per attempt 5 + 15 + 2.3 = 22.3 min. These run in parallel across jobs, so they do
  not set the pace if enough jobs are in flight. Per landing: 22.3 x 1.74 = 38.8 min of work, spread across
  in-flight jobs.

The pace is set by the slowest serial stage: the check slot at 24.4 min per landing.

- **Target with `max_check_slots = 1`: about 25 minutes per landed job** (estimate; down from about 73).
- In-flight jobs needed (Little's law, estimate): one attempt's latency is about 43 min, and one attempt leaves the
  check slot every 14 min, so 43 / 14 = about 3 in flight.
- **Stretch with `max_check_slots = 2`: about 12 to 13 minutes per landed job** (12.2 for checks, then landing at 7 is
  not the limit). Conditional on two simultaneous check runs not slowing each other; unmeasured, since the 13 to 15
  figure was taken one run at a time. Build job 9 measures it before the stretch is adopted.
- Failed jobs are in the numbers above through the 1.74 ratio. Failed jobs that stop before checks lower the cost; a
  failed job that is retried by the producer after a rebase conflict (section 5) adds a producer run but not a check
  run, unless the rebased tree changes.

The target does not rely on a cache. If a cache were re-enabled and did cut cold builds, checks and producer time
would fall; that gain is conditional and is not counted.

## 4. Read-ahead and hard-job planning

Point 1. While job N runs, the drain reads and plans job N+1 ahead of time, up to `read_ahead` jobs.

- Reading (up to 5 min) uses only the queue and the repository at the current main; it needs no worktree.
- Planning runs for hard jobs only (D143), so read-ahead for a hard job also covers its planner and critic rounds.
- A reading is made against main as of a moment. If main moves in a way that touches the same paths before the job
  starts, the reading is stale: the drain keeps the reading but re-checks the touched paths when the job starts,
  and queues a fresh reading if they overlap (the premise check the planner already needs).
- Read-ahead of jobs that touch the same paths as a job in flight is skipped, using the same disjoint-paths rule as
  `[parallel]` today.
- Cost: read-ahead spends reader and planner dollars on a job that may later be dropped. Estimate: no more than the
  read cost of one job per dropped read; `read_ahead = 1` keeps that to one job.

## 5. Early producer start in its own worktree, bounded by disk

Point 2. A job's producer starts as soon as its reading is ready, in its own git worktree and its own
`CARGO_TARGET_DIR`, without waiting for earlier jobs to land.

- The worktree is made from main at start. It is rebased at landing time (section 6).
- Starts are bounded by `max_in_flight` and by free disk (below). The drain does not start a producer if the disk guard
  would hold it (`min_free_gb`, D142); it waits for a job to finish and free its directory.

### Disk budget per in-flight job (no shared cache)

Without a shared cache every worktree builds from cold, so each pays for its own `target/`.

Let T be the size of one cold `target/` for the workspace and S the worktree's source size. **T and S are not
measured**; estimates: T about 6 GB, S about 0.2 GB. Build job 1 measures them and replaces these figures.

- Per in-flight job: T + S = about 6.2 GB (estimate).
- The check slot builds in a fresh snapshot with its own `target/`, one more T per slot: 6 GB per slot (estimate).
- Fixed: `reserve_disk_gb` (proposed 10, the same size as the existing `min_free_gb`, kept above it so the guard does
  not trip mid-run).

Limit: `in_flight = floor((free_gb - reserve_disk_gb - max_check_slots x T) / (T + S))`.

Worked example with 60 GB free (an assumed machine, not measured): (60 - 10 - 6) / 6.2 = 7.1, so 7. Choose
`max_in_flight = 3`, the number the throughput target needs, and let the disk limit cap it lower on a smaller
machine. With 3 in flight, one check slot: 3 x 6.2 + 6 + 10 = 34.6 GB (estimate). With two slots: 40.6 GB.

Idle jobs' `target/` is removed first by the existing guard (D142), which keeps the peak below the sum.

## 6. One ordered landing queue: rebase or return to the producer

Point 3. Jobs finish in any order, but they land through one queue, in the order their checks and verifier passed.

For the head of the queue:

1. Rebase the job branch on current main.
2. Clean rebase: if main changed, re-run the checks on the rebased tree only when the change touches paths the job
   touches or the workspace manifests (estimate: most rebases need no re-run; build job 7 measures the rule), then
   land by the existing path.
3. Conflict: the job returns to its producer with the conflict text and the new main. It is not abandoned. The
   producer fixes the conflict in its own worktree, the checks run again, and the job re-enters the queue behind
   the jobs already waiting.
4. Thrash stop: a job returned to the producer for conflicts more than twice (`max_conflict_returns`, proposed 2)
   closes as an environment-type stop with the reason and the paths in conflict, and the drain moves on.

One landing at a time keeps main linear and keeps the history check meaningful.

Note on the first half of build job 6. The drain's landing queue (`LandingQueue` in `drain.rs`) is first in, first
out, and a worker joins it when its run passes and keeps its place through the landing retry. So the queue order is
the order runs pass, not the order their landing commits were stacked: a job that passes later lands later, whatever
its commit sits on. That half adds the queue alone and does not rebase. The rebase before landing (steps 1 to 4
above) is a later request that depends on it.

Note on the core of the rebase (second half of build job 6, core part). `plan_rebase` and `rebase_job_branch` in
`crates/athena-core/src/gh/landing.rs` replay the landing commits stacked on the local branch, one at a time, onto
origin's main, and move the local branch to the result. They do not push. Landing commits are stacked on one local
branch, so the replay covers the whole stack from the fork with main up to the branch head, and the job's own
landing commit is found in it. What the drain will do with each result:

- Rebased cleanly: the job lands from the new landing commit.
- Already current (main is an ancestor of the landing commit): nothing moves.
- A conflict in the job's own landing commit: the job is held back. Nothing is landed, main and the local branch
  are untouched, there is no retry and no drain stop; the next queued candidate still lands. Returning the job to
  its producer, and the thrash stop above, are not part of this change.
- A conflict in an earlier commit of the stack: that commit is kept as records only. Its `records/` paths are
  carried and its code paths are dropped, so no job's records leave the branch and the held-back job's record
  still loads from the new tip. The kept commit's message names the old commit in an `Athena-Records-Only:` line
  and drops the job trailers, so it is never found again as that job's landing commit; a job whose only commit is
  such a kept commit is held back in the same way as a conflict.

Limits. Earlier unlanded commits that replay cleanly still ride along in the next job's pull request, as they do
today. The re-check rule of step 2 does not cover stacked content: it is about the paths the landing job touches,
not about the other jobs' commits that travel with it. A rebased result may differ from the content that was
verified; deciding whether to re-check is build job 7, and the merged-content check of `land_pr` stays the safety
net. The queue order is still the order runs pass, not the stacking order. The drain wiring (`Steps::rebase`,
`Athena::rebase_for_landing`, and the drain test of a held-back head) is a follow-up request that depends on this
one.

Note on the re-check rule (build job 7). `Drain::land` in `drain.rs` applies step 2 under the landing lock, before
`Steps::land`. It asks `Steps::rebase`, which answers `Rebased::Unchanged` (the branch is current, or nothing was
rebased) or `Rebased::Moved` with the paths the candidate changed and the paths the rebase touched. The pure
`rebase_needs_recheck` decides, and it is the same notion of touching that read-ahead and the plan use: a touched
path equals a candidate path or lies inside it, or the other way round, with `CHANGELOG.md`, `docs/decisions.md` and
`README.md` ignored; and any touched `Cargo.toml` or `Cargo.lock`, at any depth, also forces a re-check, because
a manifest change can break a candidate that never touched it. Otherwise the job lands with no checks run again;
the merged-content check of `land_pr` stays the safety net. The re-check is `Steps::rerun_checks`, and its
`Steps::rerun_cap` is held against the budget first. A cap that does not fit ends the run as budget spent, with no
failure counted and the job left for a later run. A re-check that fails is owed again on the landing retry
(the retry finds the branch already moved and `rebase` answers `Unchanged`, so the drain remembers the debt), and
failing it twice ends like any landing that fails twice. Only moved rebases count: the share is the rebases that
needed a re-check over the rebases that moved a branch, kept in memory for the run, in `Report::rebases` and in a
note of the drain summary that names the rule, for example `rebases: 1 of 2 needed a re-check (...)`. No record
is written. Not built here: the real steps keep the defaults (`Unchanged`, checks pass, cap 0), because the drain
wiring and the conflict hold above are a follow-up, so the share is not measured in a live drain yet; the estimate
in step 2 that most rebases need no re-run stays an estimate until then.

## 7. CI waits and self-rebuild as background awaits

Point 4. A wait is not work. While one job waits for CI or while the drain rebuilds itself, the other jobs go on.

- CI wait: a landing job's CI wait is a background await. The queue head is parked as "waiting on CI"; the next queue
  entry may rebase but does not merge until the parked job lands, so order holds. Producers and checks of other jobs
  are not blocked.
- Self-rebuild (D140): when a landed job changes Athena's own code, the drain rebuilds and restarts in the new binary.
  With several jobs in flight, the rebuild waits until no job is in a stage that writes records or lands, runs, and
  the drain resumes in the new binary with the in-flight table read from disk. Producers already running keep
  running on the old binary until their stage ends.
- Failed CI: the job returns to its producer with the CI log, once; a second CI failure closes it. Queue order for
  the others is unchanged.
- Failed self-rebuild: the drain stops starting new producers, finishes jobs already past checks only up to landing
  (landing is held), and stops with a message, as D140 does today. Nothing lands on top of a main the drain cannot
  build.
- Crash mid-flight: the in-flight table (job id, stage, worktree path) is written to a local file at each stage change.
  A stage change is the run entering the producer (again on each send-back), the checks, the verifier, or the landing;
  the run reports it itself, and no output text is matched. A rejected or failed job's row waits for its retry, which
  moves the row to the new job and sets it to producer when the retry's producer starts. On restart the drain re-reads
  the file: a job in the producer stage resumes through the existing retry path; a job in the landing stage re-checks
  whether its branch already landed before doing anything. Build job 3 only reads the file and reports leftover rows;
  acting on them is build job 9.

## 8. Ordering of records commits with several jobs in flight

Point 5. Records are history; their order must be stable even when jobs overlap.

- Each job writes its records to its own local files while it runs and does not commit them.
- The landing queue commits a job's records immediately after that job's landing commit, in the same queue slot. So the
  records order equals the landing order, and a records commit never refers to a job that has not landed.
- Failed or closed jobs commit their records through the same queue, at the point they close, so there is one writer
  to the records history.
- Records files that several jobs would touch in the same month shard are rebased by the queue like any other
  conflict; they are append-only, so a rebase conflict there is resolved by keeping both sides in queue order. This
  reading of the record formats is to be confirmed by build job 8 before it is relied on.

Note on the records step (build job 8). `Drain::land` in `drain.rs` asks a new `Steps::commit_records(job)` right
after `Steps::land` answers `Landed`, while the job still holds the head of the landing queue. The queue is what
keeps the order: `await_head` lets only the head reach `Drain::land`, and the job's `LandingSlot` is kept until its
attempts end, so the next job cannot start landing before the records step has returned. Records commits therefore
follow landing order (not start order and not run-end order), each comes after its own landing, and none comes
for a job that did not land (`NotLanded`, `Waiting` and a budget stop never reach the step). The step only adds
files, so the append-only rule holds. A step that fails is noted in `Report::notes` and printed; the landing still
counts, because a landed job cannot be un-landed and answering `NotLanded` would land it twice. The default does
nothing, so a bound of 1, `drain`, and every steps type that does not override it behave as before. The tests in
`tests/drain.rs` (`records_order`) drive `drain_concurrent` with three jobs in flight, finishing in the order c, a,
b, and show the landing and records sequences side by side, together with the byte-for-byte check of an earlier
record and the absence of any record for a job that was rejected.

Not built here: the real steps keep the default. Today `go` still commits a job's closing files and its score and
outcome records at close (`land_or_block` in `crates/athena-core/src/run/close.rs`), before the job joins the landing
queue, so those commits follow close order. `Athena::land_pr` writes the landing-intent and landing records inside
`Steps::land`, so those already follow landing order. Moving the close-time commit into the queue slot changes the
run path and is later work, as is routing the records of failed or closed jobs through the queue (the third bullet
above). The month-shard conflict reading is not confirmed: per-job record files and landing records are new paths,
so no test here can show a conflict on them.

## 9. Alternatives

| Option | Why not (or why later) |
| --- | --- |
| a. Raise `[parallel] max_concurrency` only | Runs whole jobs side by side but lands them racing; no order, no conflict return, no disk bound. |
| b. Several check slots from the start | Unmeasured slowdown (section 3); it is the stretch, after job 9. |
| c. Merge-queue service in the forge | Adds an outside dependency and does not return conflicts to the producer. |
| d. Skip the producer return; fail on conflict | Wastes a producer run's work (15 min) for a conflict a producer can fix. |
| e. Re-enable sccache to cut cold builds | **A proposed option, not assumed.** D84 removed it; the 0-hit cause is not found. Per-job worktrees and target directories make path differences larger, so it may hit less. Costed separately after job 1; if hits are measured, the disk budget and checks time here improve (conditional). |
| f. One shared `target/` across worktrees | Cargo locks it, so builds serialize; and one failed build can corrupt the others' state. |

## 10. Failure and conflict summary

| Case | Behavior |
| --- | --- |
| Rebase conflict | Back to the producer; re-enters the queue behind waiting jobs |
| Failed CI | Back to the producer once; second failure closes the job |
| Failed self-rebuild | Landing held, no new producers, drain stops with a message |
| Crash mid-flight | In-flight table re-read; producer resumes by retry; landing re-checks whether it landed |
| Thrash | More than `max_conflict_returns` conflict returns closes the job |
| Low disk | The guard holds a start; the drain waits (D142) |

## 11. Config block (proposed)

```toml
[drain.pipeline]
max_in_flight = 1        # producers running at once; 1 keeps today's behavior. Raised to 3 only by build job 10.
max_check_slots = 1      # checks running at once; 2 only after build job 9 measures the slowdown
read_ahead = 1           # jobs read and planned ahead of the one starting
reserve_disk_gb = 10     # kept free beyond the per-job budget in section 5
max_conflict_returns = 2 # conflict returns before a thrash stop
```

The block ships with every value at today's behavior. Job 10 is the one that raises `max_in_flight`.

Build job 2 implemented this block in `athena-data`'s `Config` (`[drain.pipeline]`). The config refuses
`max_in_flight` outside 1 to 8 (the `[parallel]` ceiling), `max_check_slots` outside 1 to 8, `read_ahead` of 0,
`max_conflict_returns` of 0 and any unknown key. `reserve_disk_gb` may be any non-negative whole number. The table is
left out of a reading's config digest. Build job 4 reads `max_in_flight`, `max_check_slots` and `reserve_disk_gb` for
the start bound (section 12); the other values are not read yet.

## 12. Ordered build jobs

Each job is sized for one producer run (about 15 minutes, one focused change, tests included) and lists what must
come first. Nothing here changes a limit until job 10.

1. **Measure the build cache and disk.** Depends on: nothing. Find the cause of D84's 0 hits (run `sccache --show-stats`
   in a job's build, which the producer could not do then) and measure hits across two per-job worktrees with
   different target directories; measure T and S. Output: numbers in a records note; no `athena.toml` change. Decides
   whether alternative e is worth costing. A producer's working copy cannot take these measurements, so this job
   became a command: `athena measure-build` builds cold in two worktrees and commits an
   `athena.build-measurement/1` record (T, S, both build times, the `sccache` counts of each build or that it is not
   installed). The Hestia sweep runs it when no record exists and when `Cargo.lock` changes, and `athena status`
   shows the latest. The numbers are those records, read once they exist.
2. **`[drain.pipeline]` config table.** Depends on: none. Schema, validation, defaults at today's behavior, tests.
3. **In-flight table.** Depends on: 2. The local file of job id, stage and worktree, written at each stage change and
   read at start. *Implemented (D145).* The file is `in-flight.json` in the state directory, one row per job being
   worked (`InFlight` in `crates/athena-cli/src/drain.rs`). The stage comes from the run: `Athena::go_with_stages`
   tells a `StageSink` when it enters the producer (again after a send-back), the checks, the verifier and, for a
   blocked job, the landing; the drain's own landing step reports landing. The worktree is the producer's live work
   tree, `StateDir::work_dir` (`jobs/<id>/work`), the path the producer's snapshot uses, and not the kept work of a
   timed out job. A row is removed when the item's work ends, and after a landing; the retry of a rejected job
   takes over the old job's row, so a send-back after the checks or the verifier shows as the same row going back
   to producer. A drain reads the file at its start and notes each leftover row without acting on it. A missing file is
   empty; a file that cannot be read is moved to `in-flight.json.corrupt` and noted. A failed write is a warning
   line, never a job failure. At `max_in_flight = 1` nothing else about the drain changes.
4. **Per-job worktree start.** Depends on: 3. Create and remove a worktree and its target directory per job; the disk
   bound from section 5 computed from job 1's numbers. *Implemented (D145).*
   - The worktree is the producer's isolated snapshot repository, `StateDir::work_dir` (`jobs/<id>/work`), made by
     `Snapshot::create`; its build directory is `jobs/<id>/target/produce` (its `CARGO_TARGET_DIR`). It is a one-commit
     repository of its own, **not a linked `git worktree`** of the user's repository (`git worktree list` does not show
     it). A linked worktree shares the object store, the refs and the history, so the producer could read what the
     snapshot keeps out of its reach (history, `records/`, private files). Whether to move to a linked worktree is an
     open question that needs its own job.
   - `StateDir::job_worktree` returns a `JobWorktree` guard. It clears both directories when it is made (a killed run's
     leftover) and removes both when dropped, so a verdict, a stop and an error all leave neither behind. It lives for
     the whole of `produce_and_verify`, so the build directory a failed check keeps for a send-back is still there while
     the checks and the verifier run.
   - The bound is `Athena::start_hold`, called at dispatch right after the `min_free_gb` gate. It computes the
     section 5 limit, `floor((free - reserve_disk_gb - max_check_slots x T) / (T + S))` (`disk::in_flight_limit`), with
     `free` counted before the other jobs were placed: the bytes their worktrees and build directories already use are
     added back. T and S are `target_bytes` and `source_bytes` of the latest `athena.build-measurement/1` record
     (`athena measure-build`, build job 1); with none, the design estimates in this section (6 GB, 0.2 GB) are used,
     and the hold line names which. When the limit is below `max_in_flight` and that many jobs or more are already in
     flight, the start waits: it is the existing disk hold (`LowDisk`, a hold marker, `go` returns it, the job stays
     queued), so a later drain starts the same job.
   - Not held: any start with `max_in_flight = 1` (the default and the repository's setting; today's behavior and its
     tests are unchanged), and any start when no other job is in flight, because waiting would free nothing and a held
     lone job could never start. `min_free_gb` still guards a lone job. A bound of zero therefore holds a start only
     while another job runs.
   - A job counts as in flight when its job lock is held and it has a worktree or build directory. A leftover of a dead
     process holds no lock and does not count; acting on leftovers is build job 9. The check is a gate, not a lock: the
     probe takes a shared lock on the other job's lock file for microseconds (never creating it), so a command on that
     same job that races it can be refused as already running, and two jobs that start at the same instant can both
     pass. Nothing runs more than one producer before build job 10, so this does not bite yet.
5. **Read-ahead.** Depends on: 2. Read and plan up to `read_ahead` jobs, with the stale-reading re-check.
6. **Landing queue, ordered.** Depends on: 3, 4. One queue with rebase and conflict return to the producer, thrash stop.
7. **Re-check rule after rebase.** Depends on: 6. Re-run checks only when paths overlap; measures the share of rebases that
   need it. Built as the rule and the count, with the real steps still on the defaults (section 6, note on build
   job 7); the share is not yet measured in a live drain.
8. **Records commit ordering.** Depends on: 6. Commit records in the queue after each landing; confirm the
   append-only conflict rule.
9. **Background awaits and crash recovery.** Depends on: 6, 8. Park CI waits, defer self-rebuild until a safe point,
   restart from the in-flight table; measure two simultaneous check runs for `max_check_slots = 2`.
10. **Raise `max_in_flight`.** Depends on: 1 to 9. Set `max_in_flight = 3` in `athena.toml` (a governance file, so this job
    needs Sri's yes), watch the per-landed-job minutes against the section 3 target for a week, and keep the change
    only if the number falls and no landed job later causes harm.

Rollback at any step: set `max_in_flight = 1`.
