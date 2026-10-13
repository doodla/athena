# Running independent jobs in parallel

**Status:** proposed, reviewed: accept with changes (`docs/design/reviews/parallel-jobs-review-1.md`). Design
only: no code is built by this document. The review's changes are listed in the response at the end of
section 10 and are **not yet applied to the text above it**; where they differ, the review governs.

Claims are marked **(read)** when taken from the file named beside them, and **(design)** when they are
proposals. No command was run to reproduce any behavior; every **(read)** claim comes from reading source.

## 1. The problem

`athena go <job>` runs exactly one job and returns when it ends **(read, `Command::Go` and `go_and_report` in
`crates/athena-cli/src/main.rs`; `Athena::go` in `crates/athena-core/src/run.rs`)**. A user with several
proposed or queued jobs must start each one by hand, one after another, and wait for each to finish before
starting the next. Producer and verifier runs are the slow part of a job (they are agent invocations with
their own timeouts, `budget.producer_timeout_s` and `budget.verifier_timeout_s`) **(read, `run.rs`)**, and
they spend wall-clock time without using the machine. Jobs that touch different things could run side by
side, and today they cannot. That is the bottleneck the request names.

The request, verbatim: "Run independent jobs in parallel: today it is one job at a time and that is the
bottleneck."

## 2. Requirements

| # | Requirement |
|---|---|
| P1 | One command starts several jobs and runs them at the same time, up to a bound. |
| P2 | The kill switch and each job's budgets apply to every parallel job exactly as they do to a single job. |
| P3 | One job's failure, or a lock conflict on one job, never aborts the others. |
| P4 | `athena go <one job>` behaves as it does today, including its output and exit codes. |
| P5 | Two jobs that conflict never merge silently: the later one ends stale or blocked, as today. |
| P6 | Nothing that the existing gate guarantees (verified before landed, prediction before outcome) is weakened. |
| P7 | The design leaves room for landing each job as its own branch and pull request (section 8). |

## 3. What exists today

- **Job lock.** `StateDir::try_lock_job` takes an exclusive file lock on `jobs/<id>/job.lock` without waiting
  and returns `None` if another holder has it. `Athena::go` turns `None` into an error: "job {id} is already
  running in another athena process" **(read, `crates/athena-core/src/state.rs`, `run.rs` line 116)**.
- **Repository lock.** `StateDir::lock_repository` takes a blocking exclusive file lock on
  `repository.lock`. It is held while Athena writes to the user's repository: in `land_or_block`, in
  `retry_landing`, in `commits.rs`, and in `manage.rs` **(read)**.
- **Landing is already safe against a moving branch.** `land_or_block` holds the repository lock, then makes
  up to three attempts: read `HEAD`, run `plan`, build the commit, and advance the branch only if `HEAD` has
  not moved; on `Refusal::Moved` it tries again. `plan` returns `Plan::Stale` when the job's base is no longer
  an ancestor of `HEAD` or when governed base content changed, and the job then closes as stale **(read,
  `run.rs` lines 1156-1210, `crates/athena-core/src/land.rs`)**.
- **Kill switch.** `Athena::refuse_if_stopped` fails when `registry().stopped()` is true; `go` calls it first
  **(read, `crates/athena-core/src/lib.rs` line 132, `run.rs` lines 111 and 689)**. `athena stop` terminates
  every registered process group and blocks new work; the registry is a directory under the state directory
  **(read, `crates/athena-agent/src/procs.rs`, test `stop_terminates_every_registered_group_and_blocks_new_work`)**.
- **Budgets.** Each job reads its own limits from `config.budget` (`job_usd`, `reader_usd`, `producer_usd`,
  `verifier_usd`, the three timeouts, `max_readings`) and calls `check_budget` before each paid step **(read,
  `run.rs` lines 182-240, 340, 458, 765)**. Spend is tracked per job.
- **Exit codes.** 0 when the job landed; 2 when a job did not land (rejected, failed, stale, blocked) or a
  check found violations; 1 when Athena itself erred **(read, header of `main.rs`)**.
- **Per-job directories.** Snapshots, live records and build directories live under `jobs/<id>/` in the
  state directory **(read, `state.rs`)**, so two jobs do not share scratch space.

The existing locks therefore already express the two scopes parallelism needs: per-job exclusion, and
exclusion of the one short step that writes to the user's repository.

## 4. Concurrency model

### 4.1 Alternatives for the model

**A. Threads inside one `athena go` process (recommended).**
- For: one process owns the worker bound, one view of the kill switch, and one place that collects results
  and chooses the exit code. File locks already work between threads of one process only if each thread opens
  its own file handle, which `StateDir` does (each call opens a new `File`) **(read, `lock_file`)**.
- Against: the code was written for one job per process. Whether `Athena` and the agent drivers can be shared
  across threads (`Send` and `Sync`), and whether any global state exists, has **not been checked** by this
  document. This is open question 1. A panic must be caught per thread so it does not take down the others.

**B. One child `athena go <id>` process per job, launched by a parent.**
- For: the strongest isolation; no shared memory; reuses the single-job path unchanged, so P4 holds by
  construction; a crash in one job cannot touch another.
- Against: more machinery (spawn, collect output, forward the kill switch), and per-job output must be
  captured and replayed to keep result lines whole. Terminal prompts (`prompt_loop` in `main.rs`) do not
  work in children. Kill switch and exit codes would be read back from child statuses.

**C. An async runtime.**
- Against: the work is process-bound (agent child processes and checks), not I/O multiplexing; it would add a
  runtime dependency and recolour the whole call tree. Rejected.

**D. A long-running daemon with a queue.**
- Against: a new always-on component, new state to keep consistent, and a new attack surface, for a problem
  a bounded pool solves. Rejected for now.

**Recommendation:** A, threads, **provided the review finds `Athena` shareable** (open question 1). If it is
not, B is the fallback; the CLI surface, the locks and the result lines in this document are identical for
both, so the choice can change without changing the rest.

### 4.2 The worker bound

- A new setting `parallel.max_workers` in `athena.toml`, and a flag `--jobs N` (short `-j`) that overrides it
  for one run **(design; names are not fixed)**.
- **Default: 2** **(design; an estimate, not a measurement)**. Each worker runs agent processes that cost
  money against a shared budget (section 5) and may share a machine and a rate limit. Two gives a real gain
  without surprise. `--jobs 1` runs jobs strictly in sequence, which is the safe fallback.
- `N` must be at least 1. A value above the number of jobs is harmless. A hard ceiling (for example 8) is
  proposed so a typo does not start dozens of agents **(design; the number is an estimate)**.

### 4.3 How the locks are reused

No new lock kind is added.

- **Job lock.** Each worker calls `try_lock_job` for its job before doing anything, as `go` does now. A job
  already held by another process, or named twice in the list, yields a "locked" result for that job only
  (section 5.3). The lock is held for the whole job and released by drop.
- **Repository lock.** Unchanged and still taken only inside the landing step. Because the lock is blocking,
  concurrent landings queue behind one another; each one re-reads `HEAD` under the lock. Producing and
  verifying (the slow part) do not hold it, which is where the gain comes from. The lock is held across
  `plan`, `build_commit` and `advance`, which is short compared with a job.
- **Lock order.** A worker holds its job lock, and may then take the repository lock. No path takes them in
  the other order **(read: `try_lock_job` is called at the top of `go` and `manage.rs` line 80; the repository
  lock is taken later in each path)**, so they cannot deadlock. The review should confirm that for
  `manage.rs` and `commits.rs`.
- **Shared reads.** Every worker starts from the same `HEAD` at launch, so its job's base is the same commit.
  This is the reason that the first landing succeeds and later ones are tested against the moved branch.

## 5. Kill switch, budgets and failure isolation

### 5.1 Kill switch

`athena stop` already terminates every registered process group and blocks new work **(read)**.

- The launcher calls `refuse_if_stopped` once before it starts anything, and **each worker calls it again**
  immediately before it starts its job, because the switch can be turned on while earlier jobs run. A job
  refused that way is reported as `refused (kill switch)` and is not started.
- When the switch turns on mid-run, no queued job starts. Jobs already running are ended by `athena stop`
  itself, as for a single job, and the existing paths record them as interrupted **(read, `go` on a
  dispatched job: "was interrupted")**.
- The pool does not cache the switch's state; it asks each time.

### 5.2 Budgets

- Budgets stay **per job** and unchanged: each job reads `config.budget` and spends against its own
  `job_usd`, `producer_usd` and so on. Parallelism does not divide, pool or raise them **(design)**.
- Consequence, said plainly: the worst-case spend of a run is the sum of the per-job caps, up to N times the
  per-job cap at once. `--jobs` is therefore also a spend-rate control.
- An optional **run-level cap** (`budget.run_usd`, off by default) is proposed as an open question rather
  than a requirement, because a global counter needs shared mutable state across workers (question 3).
- A job that hits its budget ends as it does today; it does not stop other jobs.

### 5.3 Failure isolation

Each worker's result is one of these, reported for that job alone **(design)**:

| Result | Meaning | Source today |
|---|---|---|
| `landed`, `rejected`, `failed`, `stale`, `blocked` | the job's outcome state | `GoResult::Finished` |
| `reread` | a queued job was read again and now needs the user | `GoResult::Reread` |
| `locked` | another process holds the job lock | the `bail!` at `run.rs` line 116 |
| `refused` | kill switch on | `refuse_if_stopped` |
| `error` | Athena erred for this job (an `Err`, or a caught panic) | `anyhow` error from `go` |

- Every worker's `Result` and any panic are caught at the worker boundary and turned into a result for that
  job. A worker never propagates an error to its siblings.
- A lock conflict is a result, not a stop. The other jobs go on.
- A failing job's own records are written by the existing code (it closes the job with a failed outcome).
  The launcher adds nothing to records.
- The launcher always waits for every started worker before it exits, so no worker is left half-done by an
  early return.
- Jobs not yet started when the bound is reached wait in a queue in the order given. They are started
  whatever happened to earlier jobs, except when the kill switch is on (5.1).

## 6. CLI surface

### 6.1 Forms **(design)**

- `athena go <job>`: as today. **Single-job behavior is unchanged**: same output, same `--json` shape, same
  exit codes, same interactive prompt after a reading. The multi-job path is taken only when two or more jobs
  are named or `--all` is given.
- `athena go <job> <job> ...`: runs the named jobs, at most `--jobs` at a time.
- `athena go --all`: runs every job that can be run now. The set is the jobs in state `proposed` and `blocked`
  (the states `go` acts on by running or retrying landing). `queued` jobs are **not** included, because
  `go` on a queued job only reads it and may need a human answer; `declined` and `dispatched` jobs are not
  included because `go` refuses them **(read, the `match job.state()` in `run.rs` lines 120-154)**. The
  exact set is open question 4.
- `--jobs N` bounds the workers. `--json` prints one JSON array.

### 6.2 Output

- While running, one line per job as it finishes (not interleaved mid-line), so the user sees progress:
  `<job-id>  landed  <landed commit short>` or `<job-id>  stale  <reason>`.
- Each worker's detailed text (the same text `display::finished` prints today) is kept and printed whole
  after the run, in the order the jobs were given, so output from two jobs never mixes.
- No prompts in the multi-job path. A `reread` job prints the usual follow-up text and ends the run for that
  job with the code below.
- `--json`: an array of objects, each with the single-job fields (`job`, `state`, `reason`,
  `prediction_commit`, `closing_commit`, `landed_commit`, `totals`, `score`) **(read, `go_and_report`)**,
  plus a `result` field for `locked`, `refused` and `error`.

### 6.3 Exit codes

- Single job: unchanged (0 landed; 2 did not land; 1 Athena erred).
- Several jobs: **1** if any result is `error`; else **2** if any job did not land (including `reread`,
  `locked`, `refused`); else **0**, which means every job landed **(design)**. A script therefore sees "Athena
  itself erred" before "a job did not land", the same precedence a single job has.

## 7. Conflicting jobs: stale or blocked, never merged silently

Jobs are "independent" if they do not edit the same governed content. Athena does not need to decide that in
advance; the landing step decides it for each job, as it does today.

- All parallel jobs start from the same `HEAD`. The first to reach `land_or_block` takes the repository lock,
  passes `plan`, and advances the branch. A later job takes the lock, reads the new `HEAD`, and `plan` checks
  its base against it **(read, `land.rs` header)**:
  - base is no longer an ancestor of `HEAD`, or governed base content changed: **stale** (the job closes
    with a stale outcome record). Athena does not rebase or merge it, and re-verification stays a user
    decision, as D10 left it **(read, `native-pr-landing.md` §5.1 step 6)**.
  - the gate refuses the landing commit, or the advance fails: **failed** or **blocked** by the existing
    paths, with the reason recorded.
- **There is no automatic merge of two jobs' changes.** Disjoint jobs whose base is merely an older ancestor
  of `HEAD` are handled by `plan` as today, whatever it already decides; this document does not change
  `plan` **(design)**. The review should check how `plan` treats a job whose base is an ancestor of `HEAD`
  but is not `HEAD` (open question 2), because that decides how often parallel jobs end stale.
- A stale or blocked job is reported in its result line and counts for exit code 2. The user then runs
  `athena correct`, `athena go` (blocked retries landing) or `athena close`, as for a single job.
- The sequential chain case of the squash design (job 2 based on job 1's landed state) is not parallel work
  and is outside this document; chained jobs should be run in order **(design)**.

## 8. Landing each parallel job as its own branch and pull request

**Designed here as an interaction. Not built. It depends on native pull-request landing, which is itself
only proposed (`docs/design/native-pr-landing.md`, status "proposed (D72)").**

Per-job pull requests remove the main source of stale results: jobs no longer compete for one local branch.

1. **One branch per job.** Each parallel job's candidate is built on its own local work branch, whose name is
   derived from the job id, as `native-pr-landing.md` §5.1 step 1 proposes **(read)**. Two jobs never write
   the same branch, so `land_or_block`'s advance of the shared branch is replaced, for this mode, by building
   the work branch only.
2. **One pull request per job.** `athena land --pr`'s flow (publish with a regular push, open, wait, observe
   the merge) is applied per job **(read, `native-pr-landing.md` §5.1; name not fixed)**. The pool may run
   the publish step for several jobs at once; a push of one job's own branch needs no repository lock beyond
   the branch it creates, and the existing rule holds: Athena never forces a push and never rewrites
   published history (§5.2 there).
3. **Merge order is the host's and the user's.** Athena does not call a merge. When one pull request merges
   first, another that no longer applies is **stale** by the same rule as in section 7 and Athena asks
   again instead of rebasing silently.
4. **Squash and rebase merges.** A pull request holding one job is the case the gate already handles, with
   a squash judged by the chain rule (M30) and a rebase by trailers and content **(read,
   `squash-chain-landing.md` §4, and `native-pr-landing.md` §5.3)**. Parallel per-job pull requests keep
   one job per pull request, so they never create the multi-job squash that the chain design addresses. Putting
   several parallel jobs in one pull request is **not** proposed.
5. **Stealth mode.** Each job's pull request holds only that job's change, with the leak checks of
   `native-pr-landing.md` §5.4. Records stay in the state directory, one set per job, so parallel jobs do not
   write one shared records file.
6. **Result lines** gain a state `pr-open` with the pull request address, which counts as "not landed yet"
   for the exit code (2) until a later `athena` command observes the merge **(design)**.

Nothing in this section is needed for the local-landing build in section 10; the local build must not
block it. In particular, the worker pool is written so that the "land" step is one function that can be
swapped for "publish a branch".

## 9. Alternatives considered

**A. Do nothing; use the shell (`athena go a & athena go b &`).**
- For: no work; the job lock and the repository lock already make it safe **(read)**.
- Against: no worker bound, so a long list starts everything; no combined result or exit code; two terminals'
  output interleaves; the kill switch is not re-checked per job. This is what users can do today and is the
  reason for the request. Rejected as the answer; remains the baseline to beat.

**B. A single global lock around each whole job (today's de facto behavior).** Rejected: it is the bottleneck.

**C. Run jobs in parallel and merge their changes automatically.** Rejected: P5. A silent merge of two jobs
produces content no verifier saw, which breaks "nothing lands unverified" (M7).

**D. One worker pool per capability or per job class.** Rejected as premature; one bound is enough to start.

**E. Rebase a stale job and re-verify it automatically.** Rejected here: it spends budget the user did not
approve and D10 (d) left re-verification manual. It could be a later option.

**F. Start jobs in parallel only when Athena can prove they touch different files.** Rejected: the diff is
unknown before the producer runs. The landing step is the only point with the facts.

(Threads versus processes, the daemon and async options are in section 4.1.)

## 10. Build plan

Each item is a separate later job.

1. **Review job (done).** An independent reviewer read this document and the code it cites and wrote
   `docs/design/reviews/parallel-jobs-review-1.md`, deciding the open questions below. Verdict: accept with
   changes. The response is at the end of this section.
2. **Spike job.** Check whether `Athena`, the drivers and the registry can be shared across threads; report
   the result. Decides threads versus processes (question 1). May be folded into step 3 if the review finds
   it clearly yes.
3. **Implementation job: pool and CLI.** `go` with several jobs and `--all`, `--jobs`, per-job result lines,
   exit codes, and the settings in `athena.toml` and `docs/` it needs.
4. **Tests.** Tests that fail without the change: two jobs run at the same time; the bound is respected;
   one job's failure and one lock conflict leave the others finishing; the kill switch turned on mid-run
   starts no further job; a conflicting second job ends stale; single-job output and exit codes are
   unchanged.
5. **Docs job.** README and CHANGELOG, and a `docs/decisions.md` entry. They are not part of this design job.
6. **Later job, separate from all of the above:** per-job branch and pull request landing (section 8), after
   native pull-request landing is built.

### Response to review 1

Accepted. The review's decisions on the seven questions in section 11 stand. In short: processes by default
(question 1); `plan` sends a job to stale whenever governed content moved, so parallel change jobs mostly go
stale after the first (question 2); no run-level cap yet (3); `--all` is `proposed` and `blocked` (4); default
2 with a ceiling of 4 (5); reject duplicate jobs up front (6); keep the exit-code precedence (7).

The text above is left as written except for the status line and item 1. Before the spike and pool jobs
start, a later design job applies these changes: rewrite section 7 and question 2 (finding F1); make
processes the recommendation in 4.1 (F2); add a "shared append-only files" section with the serialize-and-
docs-last rules and the collision test (F3); correct the line references (F7).

## 11. Open questions for the reviewer

1. Can `Athena` and its drivers be shared across threads without changes, or must the model be separate
   processes (alternative B in 4.1)? Is a caught panic enough isolation?
2. How does `plan` treat a job whose base is an ancestor of `HEAD` but not `HEAD`? If it always goes stale,
   parallel jobs will mostly end stale after the first, and the feature needs either a rebase-and-reverify
   option or the per-job pull request mode first.
3. Is a run-level spend cap needed in the first build, or are per-job caps and `--jobs` enough?
4. Which states does `--all` include? Should `queued` be included (it only reads, and may need an answer)?
5. Is a default of 2 workers right, and is a ceiling needed?
6. Should the multi-job form accept a job twice, or reject duplicates up front?
7. Should the exit-code precedence (error before did-not-land) match the single-job precedence, as proposed?

## 12. Out of scope

Implementation; the review file; README, CHANGELOG and `docs/decisions.md` entries (they belong to the
build job); building native pull-request landing; any change to `docs/design/athena-v0.md`.
