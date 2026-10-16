# Running independent jobs in parallel

**Status:** proposed, revised after review 1 (`docs/design/reviews/parallel-jobs-review-1.md`, findings F1-F8)
and again after review 2 (`docs/design/reviews/parallel-jobs-review-2.md`, findings N1-N6); review of this
revision pending. Design only: no code is built by this document. Every finding is answered in the text; the
tables map each one to its section. The revision does not mark itself accepted.

**First slice built: a bounded parallel drain.** `athena drain` now runs the independent items of one plan group
at the same time, up to `[parallel] max_concurrency` in `athena.toml` (default 1, which is the old serial drain;
the schema allows 1 to 8, `MAX_CONCURRENCY_CEILING` in `crates/athena-data/src/config.rs`). What it is, and is
not, as built **(read, `crates/athena-cli/src/drain.rs`)**:

- It runs in one process with one thread per worker; it is not the process-per-job launcher of sections 4 and 5.
  The reading, the run and the verification of a job overlap; the plan groups already guarantee no dependency
  and disjoint touched paths between the items that run together. A group starts when the group before it is
  done, so a dependent item starts only after the item it needs has finished.
- Landing (the `--pr` path) and the audience rating stay one at a time, behind one lock. A model call's cap is
  held against the budget while it is in flight, so jobs side by side cannot overspend it; with a bound of 1
  nothing is held across jobs and the counting is unchanged.
- The first reason to stop ends the run. A failed item (two failures) stops no job already in flight, and starts
  no further one. A refused item is only reported. A stop request (`athena drain-stop`) is read before every job
  and starts no further job.
- A worker that panics is not isolated: the drain ends with that panic once the other workers finish.
- To share the Athena handle between workers, `jev::Evaluate` and `gh::HttpPost` are now `Send + Sync`
  (`crates/athena-core/src/jev.rs`, `gh.rs`), and `RealSteps` holds `Sync` UI and lander references. The test
  doubles in `jev.rs` moved from `RefCell` to a mutex for the same reason.
- **Not built:** multi-job `athena go` (sections 5 and 6), per-job processes, the lock and kill-switch changes of
  section 5.1, and the `--pr` phase of sections 7 and 8, including the landing queue. Parallel landing is not
  attempted: landings are serialized instead.

| Finding | Answered in |
|---|---|
| F1 disjoint jobs do not land through local replay | sections 7, 8, 10 (build order) and question 2 in section 11 |
| F2 threads are not shown to be safe | section 4 (processes by default), section 5.1 |
| F3 shared append-only files collide | section 9 |
| F4 duplicate job refs | section 6.1, section 5.3 |
| F5 kill switch before the job lock | section 5.1 |
| F6 worst-case spend | section 5.2 |
| F7 line references | section 3 (references after native landing) |
| F8 `reread` in the multi-job path | section 6.2 |
| N1 launcher queue does not match local landing | sections 7 (point 2), 8 (point 2) |
| N2 rules 1 and 2 conflict with current job writing | section 9 |
| N3 telling `locked`, `refused`, `error`, `reread` apart | sections 4.2, 5.3, 6.2 |
| N4 exit code of an open pull request | section 6.3 |
| N5 `--all` buys work expected to go stale | sections 6.1, 7 |
| N6 field holding declared paths | section 9 (rule 3) |

Claims are marked **(read)** when taken from the file named beside them, and **(design)** when they are
proposals. Numbers marked **estimate** were not measured. No command was run to reproduce any behavior; every
**(read)** claim comes from reading source at the commit this revision was written on. Paths in this document
are relative to the repository root.

## 1. The problem

`athena go <job>` runs exactly one job and returns when it ends **(read, `Command::Go` and `go_and_report` in
`crates/athena-cli/src/main.rs`; `Athena::go` in `crates/athena-core/src/run.rs` line 110)**. A user with
several proposed jobs must start each one by hand and wait. Producer and verifier runs are the slow part of a
job (agent invocations with their own timeouts, `budget.producer_timeout_s` and `budget.verifier_timeout_s`)
**(read, `run.rs`)**; they spend wall-clock time without using the machine. Jobs could overlap those phases,
and today they cannot. That is the bottleneck the request names.

The request, verbatim: "Run independent jobs in parallel: today it is one job at a time and that is the
bottleneck."

**What review 1 changed about the promise.** The first version said disjoint change jobs would mostly land. They
do not: after one job lands governed content, a sibling from the same base is stale, even if its files are
disjoint (section 7). The gain this design promises is therefore the **overlap of the slow produce and verify
phases**, plus a landing path that does not fight over one branch. It does not promise that N change jobs from
one base all land.

## 2. Requirements

| # | Requirement |
|---|---|
| P1 | One command starts several jobs and runs them at the same time, up to a bound. |
| P2 | The kill switch and each job's budgets apply to every parallel job exactly as they do to a single job. |
| P3 | One job's failure, or a lock conflict on one job, never aborts the others. |
| P4 | `athena go <one job>` behaves as it does today, including its output and exit codes. |
| P5 | Two jobs that conflict never merge silently: the later one ends stale or blocked. |
| P6 | Nothing that the existing gate guarantees (verified before landed, prediction before outcome) is weakened. |
| P7 | Each parallel job lands through its own pull request (section 8). |
| P8 | Shared append-only files (`CHANGELOG.md`, `docs/decisions.md`, `README.md`) do not collide (section 9). |

## 3. What exists today

- **Job lock.** `StateDir::try_lock_job` takes an exclusive file lock on `jobs/<id>/job.lock` without waiting
  and returns `None` if another holder has it (`crates/athena-core/src/state.rs` line 75). `Athena::go` turns
  `None` into "job {id} is already running in another athena process" (`run.rs` lines 116-118) **(read)**.
- **Repository lock.** `StateDir::lock_repository` (`state.rs` line 68) is a blocking exclusive file lock on
  `repository.lock`, taken while Athena writes to the user's repository: `land_or_block` (`run.rs` line 1178),
  `retry_landing` (line 1424), `commit_records_with` (`commits.rs` line 138) and `manage.rs` line 370
  **(read)**.
- **Local landing.** `land_or_block` spans `run.rs` lines 1165 to about 1307 (from review 1; the start line
  1165 and the lock at 1178 were re-read). It makes up to three attempts (`for _attempt in 0..3`, line 1181):
  read `HEAD`, run `plan`, build the commit, advance only if `HEAD` has not moved. `retry_landing` (starts at
  line 1344; its lock at 1424 and loop at 1425) has the same loop for a blocked job **(read)**. `plan`
  (`crates/athena-core/src/land.rs` lines 33-83) returns `Stale` when the job's base is no longer an ancestor of
  `HEAD`, when classification rules changed, or when **governed content changed on the branch** (lines 65-74).
  It replays the candidate with `merge_tree` only when `HEAD` differs from the base by records-only changes
  (lines 55-64) **(read)**.
- **Kill switch.** `Athena::refuse_if_stopped` (`lib.rs` line 137) fails when `registry().stopped()` is true.
  `go` calls it first (`run.rs` line 111) and again before a paid step (line 698) **(read)**. `athena stop`
  terminates every registered process group and blocks new work **(read, `crates/athena-agent/src/procs.rs`,
  test `stop_terminates_every_registered_group_and_blocks_new_work`)**.
- **Budgets.** Each job reads its own limits from `config.budget` and calls `check_budget` before each paid
  step **(read, `run.rs` lines 182-240 as cited by review 1; not re-read for this revision)**. Spend is tracked
  per job.
- **Exit codes.** 0 when the job landed; 2 when a job did not land; 1 when Athena itself erred **(read, header
  of `main.rs`)**.
- **Terminal prompts.** `go_and_report` calls `prompt_loop` only when standard input and standard output are both
  terminals (`main.rs` lines 225-226) **(read)**, so a child with piped output never prompts.
- **Per-job directories.** Snapshots, live records and build directories live under `jobs/<id>/` in the state
  directory **(read, `state.rs`)**.
- **Native pull-request landing (shipped).** `docs/design/native-pr-landing.md` describes and records what was
  built: `athena land <job> --pr` pushes the job's branch, opens a pull request, reads checks, and, when the
  user's setting `[landing] merge_via_rest` is on, requests a merge using the first allowed method in the order
  merge, squash, rebase (§5.2a there). This repository allows squash only. After a squash is seen merged and
  the merged-content check passes, Athena **rebuilds the local branch on the merged commit**
  (`Athena::resync_onto_merge`, `resync_tip` in `land.rs`, `Git::replay`), replaying later commits on top; a
  conflict leaves the branch unchanged (§5.2a there). A job whose pull request merged but whose merged content
  differs from the verified fingerprint is `merged-unverified` (§5.4 there). A job whose base is no longer the
  target's tip is stale and is re-verified only on the user's request, as a new budgeted job (§5.7 there)
  **(read)**.
- **Types.** `Capability` and `Pinner` require `Send + Sync` (`capability.rs` line 86, `pin.rs` line 20). The
  `AgentDriver` trait (`crates/athena-agent/src/lib.rs` line 139) and the `Ui` trait (`lib.rs` line 56) have no
  `Send` or `Sync` bound. `Athena` (`lib.rs` line 69) holds the driver as a boxed trait object, so it cannot be
  shared by reference across threads as written. `run.rs` line 1128 uses `RefCell` for the running score
  **(read; not every driver implementation was read)**.

**References after native pull-request landing (F7).** Review 1's corrections stand against the current source:
the second `refuse_if_stopped` call is `run.rs` line 698 (not 689); `land_or_block` begins at line 1165 and
`retry_landing` at line 1344 with its loop at 1424-1433; `Ui` is now at `lib.rs` line 56 and `Athena` at line 69
(review 1 gave 51 and 64-77, which have drifted). Line numbers drift with every change, so the build job
should cite function names first and lines second.

## 4. Concurrency model

### 4.1 Alternatives for the model

**A. One child `athena go <id>` process per job, started by a launcher (recommended, default).**
- For: no shared memory; reuses the single-job path unchanged, so P4 holds by construction; needs no change
  to `AgentDriver` or `Ui`; a crash or panic in one job cannot touch another; the kill switch and the job
  lock already work between processes (they are file-based and the shell form `athena go a & athena go b &` is
  already safe **(read)**).
- Against: more machinery (spawn, collect, forward the kill switch); output must be captured per child and
  replayed whole; exit codes are read back from child statuses.

**B. Threads inside one `athena go` process (not chosen).**
Why not:
1. `AgentDriver` and `Ui` have no `Send` or `Sync` bound, and `Athena` holds a `Box<dyn AgentDriver>`
   (see section 3). Rust will not accept a shared `&Athena` until the trait is bounded and every implementer,
   including the fake driver, complies.
2. A caught panic is not enough isolation. The threads would share the process-wide registry directory handle,
   the repository lock file, and partially written usage files, and a panic leaves each in whatever condition
   it was in. A lock is released only on drop, so an abandoned lock in a panicking thread's frame is released
   late or not at all.
3. The code was written for one job per process (it uses `RefCell` for the running score, `run.rs` line 1128).
   Global state has not been searched for.

*What a spike would have to show to restore threads* **(design)**: (a) adding `Send + Sync` to `AgentDriver`
and `Ui` compiles for every implementer with no `unsafe` and no `Mutex` wrapped around the driver; (b) a search
of `crates/` finds no `static mut`, `thread_local`, `lazy`/`OnceLock` state that a second job would share, and
no use of the current directory or environment variables for per-job values; (c) a test with a panicking fake
driver in one thread leaves the repository lock, the registry and the other job's usage files intact and the
other job finishing; (d) the measured gain over processes is worth the change (process start is a few
milliseconds against jobs that run minutes, an estimate, so the gain is likely small). Without all four,
processes stay.

**C. An async runtime.** Rejected: the work is process-bound, and it would recolour the whole call tree.

**D. A long-running daemon with a queue.** Rejected for now: a new always-on component and new state for a
problem a bounded pool of children solves.

**Recommendation:** A. The CLI surface, locks and result lines are the same for either, so the spike can later
change the choice without changing the rest.

### 4.2 The process model

**Launcher** (the `athena go a b c` or `athena go --all` process) **(design)**:
1. Resolve every ref to a job id, reject duplicates (6.1), refuse if the kill switch is on (5.1), and print the
   worst-case spend (5.2).
2. Start at most `N` children at once. Each child is the same binary run as `athena go <id>` with `--json` and
   a child-only flag, `--child-report` (name not fixed), with standard input closed and output piped, so
   `prompt_loop` is never entered (`main.rs` lines 225-226). The launcher uses the id, not the ref, so every
   child resolves the same job.
3. Read each child's standard output into memory (or a file under `jobs/<id>/`) while it runs, so a full pipe
   never stalls it. Standard error carries progress lines and is replayed after the run.
4. On each child's exit, read the `result` field of the JSON object the child printed (6.2), not the message
   text. In the code, the kill-switch refusal and the lock refusal are plain errors that `main` prints as
   `error: ...` and turns into exit 1 (`run.rs` lines 111 and 116-118, `main.rs` lines 176-182), with
   nothing to tell them from other errors, and a `reread` exits 2 like a finished job that did not land
   **(read)**. So with `--child-report` the child catches those two refusals and prints
   `{"job", "result": "refused"}` or `{"job", "result": "locked"}`, any other error as
   `{"job", "result": "error", "reason"}`, and a reread as the `reread` object of 6.2; exit codes stay as
   today. The single-job command never sets the flag, so P4 holds. Death by a signal, or output that is not
   one such object, is `error` ("child ended abnormally"). Exit 0 with a finished object is `landed`; exit 2
   with one is the state named in it.
5. As a slot frees, check the kill switch and start the next queued job. Wait for every started child before
   exiting.
6. Choose the exit code by the rule in 6.3.

**Kill switch.** `athena stop` terminates every registered process group and blocks new work **(read)**.
Each child registers its own agent process groups through the existing registry, so `stop` reaches them
without help from the launcher. The launcher's part is only to start nothing new: it asks
`refuse_if_stopped` before each spawn and does not cache the answer. A child that starts after the switch
is turned on refuses itself at `run.rs` line 111, exits 1 and reports `result: refused` (step 4), which
the launcher shows as `refused (kill switch)`. A child ended by `stop` is recorded as interrupted by the
existing paths. If the launcher itself is killed, running children are not orphaned for long: they are in
the registry and `stop` ends them.

### 4.3 The worker bound

- A new setting `parallel.max_workers` in `athena.toml`, and a flag `--jobs N` (short `-j`) that overrides it
  for one run **(design; names are not fixed)**.
- **Default: 2**, **ceiling: 4** **(design; estimates, not measurements; review 1's decision on question 5)**.
  Each worker runs agent processes that cost money and may share a machine and a rate limit. Given that most
  change jobs from one base end stale after the first (section 7), a low ceiling limits wasted spend.
- `N` must be at least 1 and is clamped to the ceiling with a message. `--jobs 1` runs jobs strictly in
  sequence.

### 4.4 How the locks are reused

No new lock kind is added.

- **Job lock.** Each child calls `try_lock_job` through `go`, as today. Because the launcher rejects duplicate
  ids before spawning, a `locked` result now means only that **another athena process outside this run** holds
  the job (6.1).
- **Repository lock.** Unchanged, taken only where Athena writes to the user's repository. Concurrent writers
  queue behind one another; each re-reads `HEAD` under the lock.
- **Lock order.** A process takes its job lock, and may then take the repository lock; no path takes them in
  the other order. Review 1 confirmed this for `manage.rs` (lines 80 and 370) and `commits.rs` (line 138), and I
  found the same two `try_lock_job` callers in `run.rs` and `manage.rs` plus a third in `gh.rs` line 611
  (`athena land --pr`) **(read)**. Review 2 read `gh.rs` and confirmed it takes the repository lock only
  after its job lock (question 9).

## 5. Kill switch, budgets and failure isolation

### 5.1 Kill switch (F5)

- The launcher calls `refuse_if_stopped` once before it resolves or starts anything, and again before each
  spawn.
- In each child, `go` already calls it first (`run.rs` line 111), **before** `try_lock_job` (line 116) **(read)**.
  The rule for the build is to keep that order: a refused job must not create `jobs/<id>/job.lock`, because
  `lock_file` creates the file and its parent directories even when the caller then refuses (`state.rs` line 55,
  per review 1). The launcher's own check comes before any child opens a lock for the same reason.
- When the switch turns on mid-run, no queued job starts, and running jobs are ended by `athena stop` itself.

### 5.2 Budgets (F6)

- Budgets stay **per job** and unchanged: each child reads `config.budget` and spends against its own
  `job_usd`, `producer_usd` and so on. Parallelism does not divide, pool or raise them **(design)**.
- At the start of a multi-job run the launcher prints the worst case, **N times the per-job cap**, where the
  per-job cap is the configured `budget.job_usd` and N is the number of workers in use. For example, with
  `job_usd` of X and `--jobs 3` it prints "up to 3 x X at once". The line also states the number of jobs
  queued, so the total possible spend (jobs times cap) is visible. The configured value of `job_usd` was not
  read for this revision, so no dollar figure is given here.
- No run-level cap in the first build (question 3, settled): a global counter needs shared mutable state across
  workers, and `--jobs` with its ceiling bounds the rate. Revisit after the first real run, with a measured
  cost.
- A job that hits its budget ends as it does today and does not stop other jobs.

### 5.3 Failure isolation

Each job's result is one of these, reported for that job alone **(design)**:

| Result | Meaning | Source |
|---|---|---|
| `landed`, `rejected`, `failed`, `stale`, `blocked` | the job's outcome state | child exit 0 or 2 and its `state` |
| `merged-unverified` | the pull request merged but the content check failed | `native-pr-landing.md` §5.4 |
| `pr-open` | a pull request is open and not merged | section 8 |
| `reread` | a queued job was read again and needs the user | child `result: reread` (`GoResult::Reread`) |
| `locked` | another athena process outside this run holds the job lock | child `result: locked` (`run.rs` line 117) |
| `refused` | kill switch on | child `result: refused` (`refuse_if_stopped`) |
| `error` | Athena erred, or the child died abnormally | child `result: error`, or a signal |

The `result` field of the child's report (4.2 step 4) is what tells the last four rows apart; message text is
never parsed.

- A child's error, panic or crash is a result for that job. The launcher never propagates it to siblings.
- A failing job's own records are written by the existing code. The launcher adds nothing to records.
- Jobs not yet started wait in a queue in the order given, and start whatever happened to earlier jobs, except
  when the kill switch is on.

## 6. CLI surface

### 6.1 Forms **(design)**

- `athena go <job>`: as today. **Single-job behavior is unchanged**: output, `--json` shape, exit codes, the
  interactive prompt after a reading. The multi-job path is taken only when two or more jobs are named or
  `--all` is given.
- `athena go <job> <job> ...`: runs the named jobs, at most `--jobs` at a time.
- **Duplicates (F4, settled).** The launcher resolves each ref to a job id first (`Records::resolve`, as at
  `run.rs` line 115) and then compares ids. If any id appears twice, whether as the same ref twice or as two refs
  that resolve to one job, it exits 1 with a clear message naming the duplicate **before starting anything**.
  The old text, in which the second occurrence returned `locked`, is dropped: it raced for the lock and its
  message ("another athena process") would have been wrong.
- `athena go --all`: runs every job that can run now: those in state `proposed` and `blocked`. `queued` jobs
  are not included, because `go` on a queued job only re-reads it and may need an answer; `declined`,
  `dispatched` and closed jobs are not included because `go` refuses them **(read, the `match job.state()` at
  `run.rs` lines 120-157)**. This settles question 4. `--all` also applies the serialization rule of section 9.
- **Default for two or more change jobs (N5).** A job that ends stale has still spent its producer and
  verifier budgets, because `plan` runs only at landing (`land_or_block`). So when a run holds two or more
  change jobs (`--all` included), the launcher prints the expected-stale warning of section 7 with its
  cost: every job but the first to land may be paid in full and still land nothing. On a terminal it then
  asks for confirmation; without a terminal it starts nothing unless `--yes` is given (name not fixed).
  Limiting parallelism to jobs with non-overlapping `Reading.deliverables` is left to the build job.
- `--jobs N` bounds the workers. `--json` prints one JSON array.

### 6.2 Output

- One line per job as it finishes, not interleaved mid-line: `<job-id>  landed  <short commit>` or
  `<job-id>  stale  <reason>`.
- Each job's detailed text is kept and printed whole after the run, in the order the jobs were given.
- **No prompts in the multi-job path (F8, settled).** A `reread` job prints the usual follow-up text and ends
  with exit code 2. Because `--all` excludes `queued` jobs, `reread` arises only for a job named explicitly;
  that is accepted as is.
- `--json`: an array with one object per job, each carrying a `result` field (the names of 5.3) that says
  how to read the rest. A finished job has the single-job fields (`job`, `state`, `reason`,
  `prediction_commit`, `closing_commit`, `landed_commit`, `totals`, `score`) **(read, `go_and_report`)**. A
  `reread` job has different fields, `job`, `commit`, `reading`, `reader_error` and `spent_usd`, and **no
  `state`** **(read, `proposal_json`, `main.rs` lines 196-204)**; `result: reread` is how it is recognised.
  `locked`, `refused` and `error` carry `job` and, for `error`, `reason`; `pr-open` and `merged-unverified`
  add the pull request address.

### 6.3 Exit codes (question 7, settled)

- Single job: unchanged.
- Several jobs: **1** if any result is `error`; else **2** if any job did not land (including `reread`,
  `locked`, `refused`, `pr-open`, `merged-unverified`); else **0**. This keeps the single-job precedence, so a
  script sees "Athena itself erred" before "a job did not land".
- **`pr-open` differs on purpose (N4).** `athena land --pr` run alone returns 0 for an open pull request
  with green checks (`land_pr_code`, `main.rs` lines 210-218) **(read)**. In a multi-job run the same state
  counts as "did not land" and gives 2, because the run's question is whether each job landed, and a script
  that reads 0 may assume every change is in. The result line says `pr-open`, so the cause is visible.

## 7. Jobs that conflict, and a job whose base is behind the new HEAD (F1)

**What this revision drops.** The earlier text said disjoint jobs whose base is an older ancestor of `HEAD` are
handled by `plan` and mostly land. That is wrong. `plan` replays a candidate only over **records-only** changes
(`land.rs` lines 55-64). If any sibling landed a governed change (code, `README.md`, `CHANGELOG.md`, `docs/`),
the job's governed fingerprint differs and `plan` returns `Stale` (lines 65-74), **even when the two jobs'
files are disjoint** (review 1 F1; **read** at `land.rs`). Question 2 is answered: it goes stale. With N change
jobs from one base, at most one lands without re-verification (an estimate from reading, not measured).

**How jobs land instead.** Each job lands locally in its child, then through its own pull request by squash,
and after a merge Athena rebuilds the local branch on the merged commit (`docs/design/native-pr-landing.md`
§5.2a, **read**; summarized in section 3). The parallel pool builds on that and does not redesign it:

1. The slow phases (reading excepted) run in parallel in separate child processes.
2. **Local landing stays in the child, ordered by the repository lock** **(design)**. Each child runs `go`
   as today: produce, verify, then `land_or_block`, which reads `HEAD` and advances the branch under the
   repository lock (`run.rs` lines 1178-1182) **(read)**. The order of local landings is the order in which
   children finish and win that lock. The launcher does not queue them and no new lock is added. `go` never
   calls `land --pr` (`main.rs`, `gh.rs` line 600) **(read)**, so the pull request is a second phase:
   - With a `--pr` flag on the multi-job command (name not fixed), the launcher runs
     `athena land <id> --pr` for each job that landed locally, one at a time, in the order they landed.
     Without the flag, the user runs it later. The launcher learns that a pull request merged from that
     command's result, not from the child of `go`.
   - A job's result while its pull request waits is `landed` locally and then `pr-open`; it becomes `landed`
     (final) when `land --pr` sees the merge and the merged-content check passes. With `merge_via_rest`
     off (section 8 point 3) that happens after the run, so the run reports `pr-open`.
   - After a squash is seen merged, `resync_onto_merge` rebuilds the local branch on the merged commit and
     replays later local commits on top (`native-pr-landing.md` §5.2a). That keeps jobs that landed
     locally in the meantime, so the rebuild needs no ordering with other children's local landings.
3. **A job whose base is behind the new HEAD.** When a later child reaches `land_or_block`, its base is the
   commit it was dispatched on, which is behind the `HEAD` that the first landing moved. Then:
   - If only records moved, `plan` replays it as today.
   - If governed content moved, it is **stale**: the result line says so with `plan`'s reason, and nothing
     lands. Athena does not rebase or merge it, and does not re-verify it by itself.
   - If a host merge nevertheless accepted a stale pull request cleanly (disjoint files), the merged-content
     check compares the fingerprint and reports `merged-unverified` rather than `landed`
     (`native-pr-landing.md` §5.4).
   - **Leaving the state.** The user asks Athena to re-verify the job on the new tip. That starts a new job
     carrying the original request, with its own budget and its own prediction (`native-pr-landing.md` §5.7
     points 1-3). Whether the build of that request exists was not checked here; if it does not, the parallel
     build must either wait for it or document the manual path (`athena correct`, then `go`).
4. **There is no automatic merge** of two jobs' changes, and no automatic re-verification. A silent merge would
   produce content no verifier saw (P5, P6).

**Consequence for use.** Parallelism pays for jobs that touch different governed content only in the produce
and verify overlap, and for jobs that change no governed content; for ordinary change jobs the user should
expect a stale result for most siblings of the first to land, and should group jobs accordingly. A stale
job has still spent its producer and verifier budgets (N5). The launcher prints that warning, with that
cost, when two or more change jobs are in a run (design; wording not fixed; see 6.1). The chained case
(job 2 built on job 1's landed state) is not parallel work; run chained jobs in order.

## 8. Landing each parallel job as its own pull request

This section is no longer a future mode: it is the landing path. It uses the shipped mechanism of
`docs/design/native-pr-landing.md` (slice 2, `athena land <job> --pr`, `crates/athena-core/src/gh.rs`).

1. **One pull request per job.** One job per pull request is the choice stated in `native-pr-landing.md` §5.6.
   Putting several parallel jobs in one pull request is not proposed.
2. **What is pushed.** In owned mode the pushed commit is the job's locally landed commit, found from its
   trailers (`native-pr-landing.md` §5.1 step 2, **read**). So the job's local landing (`land_or_block`) still
   happens first, inside the child, under the repository lock, and the pull request is cut from that. Local
   landings are ordered by that lock, not by the launcher (section 7 point 2): two children cannot both
   advance the local branch from the same base, because the second one's `plan` sees the moved `HEAD`. The
   launcher (with `--pr`) or the user then runs `land --pr` for each landed job, one at a time. A pull
   request cut after an earlier job landed locally but has not yet merged also carries that earlier
   commit; this is part of question 8.
   *Open for the build:* building the pull request from a per-job work branch without advancing the shared
   branch (step 1 of §5.1 there, "a local work branch whose name is derived from the job id") would let the
   check against the target happen on the host only. That is a change to the shipped mechanism, so it is
   carried forward as question 8, not designed here.
3. **Merge method and rebuild.** The merge method follows the repository (squash here); a squash is judged by
   the chain rule (M30), and the local branch is rebuilt on the merged commit by `resync_onto_merge`
   (`native-pr-landing.md` §5.2a). The setting `[landing] merge_via_rest` stays off by default; with it off, a
   job's result is `pr-open` until the user merges and a later `athena land` run observes the merge.
4. **Pushes.** Athena never forces a push except the compare-and-swap replacement of the job's own branch head
   that §5.2 there describes, and never rewrites published history.
5. **Stealth mode.** Each job's pull request holds only that job's change under the leak checks of §5.4 there.
   Records stay in the state directory, one set per job.
6. **Result lines** gain `pr-open` (with the pull request address) and `merged-unverified`, both counting as
   "not landed yet" for the exit code.

The worker pool is written so that the landing step is one function (`land one job through its pull request`),
so the order and the mode can change without touching the pool.

## 9. Shared append-only files (F3)

`CHANGELOG.md`, `docs/decisions.md` (numbered entries such as `D81`) and `README.md` are edited by many jobs.
Evidence reported by the user, not reproduced: two hand-run parallel branches both edited `CHANGELOG.md` and
both took the next decision number, and a hand-merged squash was refused by the landing check. The root cause is
that **several jobs write the same shared file from the same base without knowing about each other**; the
rules below move the write to one place instead of fixing each collision.

**When the rules apply (N2).** Rules 1 and 2 take effect with the build job, once the docs job exists and
the people who write jobs know them. Today change jobs routinely list a `CHANGELOG.md` entry in their
acceptance criteria, and admission does not reject such a job (`admission.rs`, the part read). Until the
build, **rule 3 is the working rule**: jobs that name the same file are serialized, so most early runs
will serialize them.

**Rules (design):**

1. **Who writes them.** Code, test and design jobs do **not** edit `CHANGELOG.md`, `docs/decisions.md` or
   `README.md`. Those files are written by a **docs job that runs alone, after the jobs it describes have
   landed**. This extends the build plan's existing "docs job last" (section 10) into a rule. A job whose
   change needs a changelog line states it in its report; the docs job writes it.
2. **Number allocation.** Only the docs job allocates a decision number, and it does so at write time: it
   takes the next number after the highest in `docs/decisions.md` **at its own base**, which (because it
   runs alone, after the jobs it describes have landed) is the current tip. Two jobs therefore cannot take
   the same number: only one writer exists at a time. A job that needs to refer to its own decision before
   the docs job runs uses a placeholder in prose, such as "the decision recorded by the docs job", never a
   number.
3. **Serialization by declared path.** The declared paths are the `path` of each entry in
   `Reading.deliverables` (built in `reading.rs`, used in `admission.rs`). The pool does not run two jobs
   concurrently if their deliverables name the same append-only file; `--all` partitions the jobs by that
   list and runs the groups' members in sequence (each landing before the next starts), while other jobs
   proceed in parallel. A job with no reading yet (a `queued` job named explicitly) has no list: it is not
   serialized, and the landing check decides. The list is a **hint**: the true diff is unknown before the
   producer runs (alternative F, section 12), and admission only checks the list for governance changes.
4. **Landing check is the authority.** If a job touches a shared file it did not declare, nothing is lost: the
   second job's governed fingerprint no longer matches, so it is stale (section 7), or `merged-unverified` if
   the host merged it. The files are never merged by hand into unverified content: `plan` refuses to land when
   the landed fingerprint differs from the verified one (`land.rs` lines 75-81).

**Plan grouping (built).** `plan.rs` leaves these three files out of the overlap test when it splits a wave
into parallel groups (`SHARED_APPEND_ONLY`), because nearly every item lists `CHANGELOG.md`, and without the
exception every group was a single item. An item whose only touched paths are shared files names nothing else,
so it is still not grouped. This changes grouping only; the landing check (rule 4) remains the authority for
conflicting edits to a shared file, and rules 1 to 3 are unchanged.

**What is enforced and what is a hint:**

| Rule | Status |
|---|---|
| Content that lands equals the verified content; a second job that touched a moved shared file is stale or `merged-unverified` | Enforced by the landing check (`plan`, and the merged-content check of `native-pr-landing.md` §5.4) |
| No lost text: a refused job leaves the shared file as the first job left it | Enforced (nothing lands for a stale job) |
| Serialization of jobs that name the same file | Hint: depends on the brief's declared path list |
| Only the docs job allocates numbers and edits the three files | Prose rule, enforced by brief and review; not by code in the first build |
| Number allocation at write time from the current tip | Follows from the docs job running alone; not separately checked |

**The collision test** (a later build job; it must fail without the pool's partition rule). It must check:
1. Two jobs whose briefs both name `CHANGELOG.md`, run with `--jobs 2`, are **not** run at the same time: the
   second starts only after the first has landed, and then both land, or the second is refused with a named
   reason and **no lost text** (this matches the roadmap's RM-0 done-when, `docs/design/roadmap.md`).
2. Two jobs that both append to `CHANGELOG.md` **without declaring it**, run with `--jobs 2`, end with the
   second stale (or `merged-unverified`), and the file on the branch holds exactly the first job's text.
3. Two entries for `docs/decisions.md` never carry the same number: when the docs job runs, the numbers on the
   branch afterwards are distinct and consecutive.
4. A hand-merged candidate of two jobs is refused by the landing check.

## 10. Build plan

Each item is a separate later job.

1. **Review jobs (done).** Review 1 of the first version; this revision answers it. A second review of this
   revision is the next step before any build.
2. **Spike job** (only if threads are to be reconsidered). Runs the four checks listed in 4.1. Not on the path:
   processes are the default and need no spike.
3. **Settle the landing mechanism.** Decide question 8 (pull request from a work branch without advancing the
   shared branch), and confirm that the user-requested re-verify of a stale job (`native-pr-landing.md` §5.7)
   exists or is built. These come before the pool, because the pool's value depends on them (F1).
4. **Implementation: launcher and CLI.** `go` with several jobs and `--all`, `--jobs`, the process model of 4.2,
   duplicate rejection, the worst-case spend line, the second-phase `land --pr` run, the child report of 4.2,
   per-job result lines, exit codes, and the settings in `athena.toml` and `docs/` they need.
5. **Tests.** Tests that fail without the change: two jobs run at the same time as separate processes; the bound
   is respected; one job's failure, one child killed by a signal and one lock conflict leave the others
   finishing; the kill switch turned on mid-run starts no further job; duplicate refs are rejected before any
   child starts; a second job whose base is behind a governed change ends stale; single-job output and exit
   codes are unchanged; and the collision test of section 9.
6. **Docs job, alone and last.** README, CHANGELOG and a `docs/decisions.md` entry, under the rules of section 9.

### Response to reviews 1 and 2

Review 1 is accepted and applied; its decisions on its seven questions are applied as section 11 shows.
Review 2 (verdict: accept with changes) is applied in this revision: N1 first (sections 7 and 8), then N2
to N6 as the table at the top maps them. Its not-checked item stays open for the build job:
`registry().reap()` runs at the start of every `go` (`run.rs` line 112), and several children will now
call it at once.

## 11. Open questions, settled or carried forward

Question numbers are those of the first version; review 1 section 3 decided them.

1. **Threads or processes? Is a caught panic enough?** Settled: processes by default; a caught panic is not
   enough (4.1). Threads return only if a spike shows the four things listed there.
2. **How does `plan` treat a base that is an ancestor of `HEAD` but not `HEAD`?** Settled: it replays only over
   records-only changes and is otherwise stale (section 7). The feature rests on per-job pull requests with a
   user-requested re-verify, not on local replay.
3. **Run-level spend cap?** Settled: no in the first build; print the worst case instead (5.2).
4. **Which states does `--all` include?** Settled: `proposed` and `blocked` (6.1).
5. **Default workers and ceiling?** Settled: 2 and 4, both estimates (4.3).
6. **Duplicates?** Settled: reject up front after resolving to ids (6.1).
7. **Exit-code precedence?** Settled: error before did-not-land (6.3).
8. **New, carried forward.** Should a parallel job's pull request be cut from a per-job work branch, without
   advancing the shared local branch first (section 8, step 2)? *Decision stated:* not in the first build; the
   first build uses the shipped mechanism, with landings queued one at a time. Revisit after the first real
   run shows how many jobs end stale (to be measured, not estimated).
9. **New.** Does `athena land --pr` take the repository lock only after its job lock (`gh.rs` line 611)?
   Settled by review 2 section 2 (read): it holds the job lock through the function and
   `resync_onto_merge` takes the repository lock later (`land.rs` line 113), so the order of 4.4 holds.
   The build job may re-check.

## 12. Alternatives considered

**A. Do nothing; use the shell (`athena go a & athena go b &`).** The locks already make it safe **(read)**.
Against: no worker bound, no combined result or exit code, interleaved output, and no per-job kill-switch check.
This is the baseline to beat, and it is the process model of 4.1 without a launcher.

**B. A single global lock around each whole job.** Rejected: it is the bottleneck.

**C. Run jobs in parallel and merge their changes automatically.** Rejected (P5): content no verifier saw
breaks "nothing lands unverified" (M7).

**D. One worker pool per capability or job class.** Rejected as premature.

**E. Rebase a stale job and re-verify it automatically.** Rejected: it spends budget the user did not approve,
and D10 left re-verification manual. The user-requested form of `native-pr-landing.md` §5.7 is used instead.

**F. Start jobs in parallel only when Athena can prove they touch different files.** Rejected: the diff is
unknown before the producer runs. The declared path list of section 9 is a hint, and the landing step is the
authority.

(Processes versus threads, the daemon and async options are in section 4.1.)

## 12a. Related design: CI routing

`docs/design/ci-routing.md` (review requested, not built) moves the checks to CI with caches shared by content
key. It affects this design in three ways: parallel pull-request runs share cache entries and only the default
branch writes them; the worker bound of 4.3 also bounds how many commits wait for CI; and CI minutes are a
second spend beside the per-job agent budgets of 5.2. Neither design depends on the other's build.

## 13. Out of scope

Implementation; the review file and `docs/design/native-pr-landing.md`; the thread-safety spike; README,
CHANGELOG and `docs/decisions.md` entries for the build (they belong to the docs job of section 10); any change
to `docs/design/athena-v0.md`.
