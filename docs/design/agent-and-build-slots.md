# Many agents at once, few builds at once: agent slots, build slots and shared compiled output

Status: design note, proposed. Nothing in `crates/`, `xtask/`, `athena.toml`, `defaults.toml` or
`.github/workflows/` changes with this note. Written for the maintainer, who decides the concurrency model, and
for the verifier, who checks every citation against the records.

Request: run dozens of agents in parallel on one machine with 4 CPUs, 15 GB of memory and about 27 GB of free
disk, where one job's Rust build takes about 10 to 12 GB.

Labels used below: **measured** (read from records, file named), **stated** (given in the request, not found in
any record), **estimate** (reasoned here, not measured), **existing** (built; file named), **new** (not built).
Record paths are relative to the `records/` directory of the evidence snapshot unless they start with
`landings/`, which is its sibling directory.

## 1. Existing code

Nothing in this table is created by this note. Each row says what the code does now and what this design adds.

| What exists | Where | What this design adds |
| --- | --- | --- |
| `athena drain` runs the independent items of one plan group at once, up to `[parallel] max_concurrency` (default 1, ceiling 8 in `MAX_CONCURRENCY_CEILING`; `athena.toml` sets 2). One thread per worker; landing stays one at a time behind one lock | `docs/design/parallel-jobs.md` (opening section, "First slice built"); `crates/athena-data/src/config.rs` (`Parallel`); `athena.toml` (`[parallel]`) | **Extends.** The worker count becomes the agent-slot count and its ceiling is raised (build job 6). A worker no longer implies a build: builds are a separate, smaller pool (section 4) |
| `[drain.pipeline]` with `max_in_flight`, `max_check_slots` (default 1), `reserve_disk_gb` (default 10), and the disk bound `floor((free - reserve - check_slots x T) / (T + S))` with T = one cold target directory, S = one worktree | `docs/design/async-drain.md` sections 5 and 9; `crates/athena-core/src/disk.rs` (`in_flight_limit`, `start_decision`, `Figures`, `ESTIMATE_T_BYTES` = 6 GB, `ESTIMATE_S_BYTES` = 0.2 GB); `crates/athena-data/src/config.rs` (`PipelineConfig`) | **Reused.** `max_check_slots` is read only inside the disk formula: a search of `crates/` finds it in `config.rs` and `disk.rs` and nowhere in `crates/athena-cli/`, so nothing counts running builds against it. Build job 3 turned it into an enforced count (`crates/athena-core/src/slots.rs`; built, see section 8) |
| Disk guard: `[disk] min_free_gb` (default 10) checked at dispatch, before the producer, before the checks and before the verifier; idle jobs' `target/` is removed first; a low disk holds the job, it does not fail it | `crates/athena-core/src/disk.rs` (module comment); `crates/athena-core/src/run/checks.rs` (`SnapExec::exec` gates once before the first local check; `free_producer_build` removes `target/produce` before the checks) | **Reused** as the last line of defence. The queue (section 4) keeps most starts from reaching it |
| Each job builds in its own directory: `CARGO_TARGET_DIR = "{job_dir}/target/{side}"`, and a build directory outside the job's directory is refused (D46) | `athena.toml` (`[capabilities.change.env]`); `crates/athena-agent/src/env.rs` (`BUILD_DIR_VARS`, module comment) | **Changes** how a job's directory reaches a slot's warm build (section 5, build job 5) without changing the rule that a job only ever builds in a directory it was given |
| Agents may run `cargo build`, `check`, `test`, `clippy`, `run`, `doc` through Bash; `cargo` options that set the build configuration or directories are refused | `athena.toml` (`producer_commands`, `verifier_commands`); `crates/athena-agent/src/policy.rs` (`check_cargo`) | **Extends.** The producer's own cargo runs are builds too (the producer is not build-free). They take a build slot per command (build job 4) |
| Checks: CI result first, local run as the fallback; local checks run once the disk gate passes, each under `check_timeout_s` (1800) | `docs/design/warm-baseline-build.md` sections 1 and 6; `crates/athena-core/src/routing.rs`; `crates/athena-core/src/run/checks.rs` (`SnapExec`) | **Extends.** The local fallback and the "fails without the change" reruns wait for a build slot. Nothing changes for a CI-reused check, which runs no local build |
| Warm baseline of `main` for the checks: measured at about 0.6 to 1.1 minutes per landed job and not recommended; a shared target directory across jobs was rejected (lock contention, fresh modification times); `sccache` was left unmeasured | `docs/design/warm-baseline-build.md` sections 3 to 6 | **Consulted, not repeated.** Section 5 keeps its two findings (workspace crates always rebuild in a fresh snapshot; third-party crates are reusable) and applies them per slot, not per `main` |
| `sccache` support: the pinned binary's path, a cold two-worktree measurement that records hits and misses (`athena measure-build`), the status line that prints them. The wiring in `athena.toml` was removed after a report of 0 hits (D84) | `crates/athena-data/src/paths.rs` (`sccache_root`, `sccache_binary`); `crates/athena-core/src/disk.rs` (`CargoBuilder`, `measure_build`); `crates/athena-core/src/status.rs` (`build_measurement_line`); `docs/design/async-drain.md` section 9 alternative e; `docs/design/roadmap.md` | **Reused** for the measurement. Section 5 records why `sccache` is the first thing to try again and why it does not cut disk |
| Landing and rebuild: one ordered landing queue, a `landing` lock, `rebuild_due`, `rebuild_if_due` running `setup.sh`; the proposal to batch the rebuild and keep waiting jobs in the queue | `docs/design/continuous-landing.md`; `crates/athena-cli/src/drain.rs` (`rebuild_if_due`, the `Steps::rebuild` step that runs `setup.sh`); `setup.sh` (line 29: `cargo run --locked --release -p xtask -- setup`) | **Reused** for the queue and batching jobs. This note adds one finding about the stage's measurement (section 6.1) and puts the rebuild behind a build slot (build job 7) |
| `StageMinutes`, `landing_record_time`, `landing_end`, `landed_jobs`: the six stages of a landed job, and the landing record used to end the last one | `crates/athena-core/src/status.rs` (lines 650 to 700 and 793 to 902); `crates/athena-git/src/snapshot.rs` writes the throwaway snapshot directory each agent works in (`Snapshot::create`), the per-agent source copy that the 0.2 GB figure in section 7 stands for | **Changes** `landing_record_time` (build job 2) and reads `snapshot.rs` only for what an agent's directory costs |

## 2. What the records say about where a job's time goes

### 2.1 The population and the sample

`status --snapshot` on this job's evidence snapshot prints: 370 landed, 98 failed, 130 rejected, 107 dropped,
193 queued and 976 jobs in all, 174 landing records, and a throughput line for 2026-11-04 of 14 landings.
Command: `cargo run -p athena-cli -- status --snapshot <evidence directory>`.

**Sample.** The 14 jobs whose `outcome.json` has `"state": "landed"` and an `at` on 2026-11-04 (found by searching
`jobs/**/outcome.json` for that state and date; 14 files, the same number as the status throughput line for that
day). Of the 14 I read **5 in full**: `20261104-054754-b4f5`, `20261104-141009-hsg4`, `20261104-153614-pzfw`,
`20261104-171135-911m`, `20261104-194933-s8v3` (all under `jobs/2026-11/`). Five jobs is a small sample: the
figures show how the time splits in these five and are not population medians. The population view is the
`status --snapshot` median in section 2.4.

**Left out, exactly:**

- The other 9 of the 14, **not read** (no property of theirs excluded them; I ran out of budget): `20261104-002852-g39r`,
  `20261104-065707-4pvy`, `20261104-091422-p3kx`, `20261104-091432-d10c`, `20261104-103213-f1cw`,
  `20261104-124034-ysc4`, `20261103-231410-asf2`, `20261103-231510-20z0` and `20261010-154013-a9gz`. The last was
  dispatched in October and closed as landed on 2026-11-04T03:35 (`jobs/2026-10/20261010-154013-a9gz/outcome.json`,
  `wall_time_execution` 3689 s), so it would also be a poor sample member for a one-day view.
- Every job that did not land on 2026-11-04, whatever its state. This includes directories on 2026-11-04 that never
  ran a model: `jobs/2026-11/20261104-123907-qrwr/` and `jobs/2026-11/20261104-142256-p9e4/` hold only
  `request.json`, `outcome.json` (`"state": "dropped"`) and `score.json`, with no `invocations/` and no
  `job.json`. `jobs/2026-11/20261104-074021-6z29/` holds only `request.json`, `reading-1.json` and a reader
  invocation, with no `job.json` and no `outcome.json`. `jobs/2026-11/20261104-102639-h6ep/` holds a request, a
  reading, a reader invocation, `priority-1.json`, `outcome.json` (`"state": "dropped"`) and `score.json`, with no
  `job.json`. None of these four is among the 14 landed.
- Directories whose first invocation is not a reader (`01-audience.json`, `01-steward.json`): these are audience
  ratings and steward runs, not request jobs. There are 16 of them under the date prefix `20261104-` (ten
  audience, six steward; read from the listing of `jobs/2026-11/20261104-*/invocations/01-*.json`, which has 42
  entries, 26 of them readers).
- Counts for scale: `jobs/*/*/job.json` matches 638 files and `jobs/*/*/invocations/01-*.json` matches 909 files, so
  the job directories are not uniform: some hold invocations without a `job.json`. The 976 in the status line counts
  jobs by state and is not a count of directories. I did not classify the rest of the directories.

### 2.2 The split, job by job

Definitions, so a reader can reproduce each cell:

- **Start** is the reader's `started_at` (`invocations/01-reader.json`). **End** is the `created_at` of the *first*
  record in `landings/` that lists the job as a `job_landing` commit (reason in section 6.1).
- **Model time** is the sum of `wall_time.value` over every file in the job's `invocations/` (reader, planner,
  critic, producer, verifier). Basis in the records: "athena monotonic clock".
- **Local build and test** is the sum of `duration.value` of the checks in `verification.json` that were not reused
  from CI (`"reused": true` gives 0.0 s). Cargo runs the producer made inside its own invocation are in model
  time and cannot be separated by any record (the invocation record lists `commands_run` but no per-command time).
- **Waiting** is End minus Start minus the other two. It holds the wait for the CI run of the candidate, the wait
  for the CI result of the base, queueing, and the gap to the landing record.

| Job | Start to first landing record | Model time | Local build and test | Waiting |
| --- | --- | --- | --- | --- |
| `20261104-054754-b4f5` | 1957.0 s (05:50:06 to 06:22:43) | 202.8 s (reader 40.2, producer 93.9, verifier 68.7) | 0 s (all three checks reused) | 1754.2 s |
| `20261104-194933-s8v3` | 1292.2 s (19:49:35 to 20:11:08) | 105.0 s (22.4, 57.6, 25.1) | 0 s (reused) | 1187.2 s |
| `20261104-171135-911m` | 4850.8 s (17:11:38 to 18:32:29) | 1910.0 s (reader 21.0, planner 149.1, critic 54.2, producer 1604.0, verifier 81.7) | 903.3 s (`fails-without-2` 20.0, `fails-without-3` 883.1 over 2174 tests, one more 0.2) | 2037.5 s |
| `20261104-153614-pzfw` | 2456.5 s (18:37:23 to 19:18:20) | 439.7 s (50.8, 210.8, 178.1) | 0 s (reused) | 2016.8 s |
| `20261104-141009-hsg4` | 3080.7 s (20:22:50 to 21:14:11) | 2120.8 s (reader 37.3, planner 201.4 and 133.9, critic 92.5 and 90.6, producer 1412.5, verifier 152.6) | 75.2 s (three `fails-without` runs, 41.7 + 16.8 + 16.8) | 884.7 s |
| **Total of the 5** | **13637.2 s** | **4778.4 s (35.0%)** | **978.5 s (7.2%)** | **7880.3 s (57.8%)** |

The landing-record times are read from the record file names (`landings/20261104-062243-y5nm.json` for `b4f5`,
`20261104-201108-2zzz` for `s8v3`, `20261104-183229-6dhz` for `911m`, `20261104-191820-ny02` for `pzfw`,
`20261104-211411-rts4` for `hsg4`); the file name's time matched `created_at` to the second where I opened the
file (`y5nm`: `2026-11-04T06:22:43.3Z`). Check durations are from `jobs/2026-11/<job>/verification.json`.

### 2.3 Where one job's waiting sits (`b4f5`, the job read in the most detail)

| Gap | Seconds | Records |
| --- | --- | --- |
| Dispatch (`job.json` `dispatched_at` 05:51:00.8) to the producer's `started_at` 06:06:12.0 | 911 | `jobs/2026-11/20261104-054754-b4f5/job.json`, `invocations/02-producer.json` |
| Producer `ended_at` 06:07:45.9 to verifier `started_at` 06:17:18.3 (checks stage; all three checks were reused from a CI run, recorded 0.0 s each) | 573 | `invocations/02-producer.json`, `invocations/03-verifier.json`, `verification.json` |
| Verifier `ended_at` 06:18:27.1 to the first landing record 06:22:43.3 | 256 | `invocations/03-verifier.json`, `landings/20261104-062243-y5nm.json` |

So for this job the model worked for 3.4 minutes of 32.6, and the 573 s gap matches the CI run duration
(`warm-baseline-build.md` section 2.1 measured a median CI run of 583 s). The 911 s before the producer is not
explained by any record I read: `warm-baseline-build.md` section 1 says the baseline waits for the base's CI run
up to `ci_wait_s`, which is a candidate cause, not a measurement.

### 2.4 The population view from `status --snapshot`

`status --snapshot` prints, for the last 20 measured landed jobs, the median per stage in minutes: reading 0.6,
planning 4.3, producer 12.8, checks 0.7, verifier 3.1, **landing and rebuild 1069.2**. The first five stages sum to
21.5 minutes (0.6 + 4.3 + 12.8 + 0.7 + 3.1). Idle time between jobs is 16072.1 minutes and the 296 jobs that did
not land took 4937.5 minutes. The stage definition is `StageMinutes` in `crates/athena-core/src/status.rs`.

What this means for sizing: model plus local build and test is where the machine is busy; most of a job's wall
time is waiting (57.8% in the sample), and the "longest stage" in `status` is, for these five jobs, mostly a result
of which landing record the code picks (section 6.1). Both facts point the same way: an agent slot must not be held while the
job waits.

## 3. Two kinds of slot

| | Agent slot | Build slot |
| --- | --- | --- |
| What it admits | One model invocation (reader, planner, critic, producer, verifier) and its bookkeeping | One cargo command that compiles or runs tests: a producer Bash `cargo` call, a local check, a "fails without" rerun, the landing rebuild |
| How long it is held | The invocation's wall time (sample: 22 to 1604 s) | One command (bounded by `check_timeout_s` 1800 for checks, by the Bash timeout for a producer command: `bash_default_timeout_ms` 600000 and `bash_max_timeout_ms` 1200000 in `athena.toml`) |
| Limited by | Memory, CPU for the host process, plan-group independence, the API rate limit and cost caps | Disk (target directories) and CPU (compiles) |
| Count on this machine | 12, raised to 16 after measurement (section 7) | 1 today, 2 after build jobs 3 to 5 (section 7) |
| Where it lives | `[parallel] max_concurrency` (existing; ceiling raised in build job 6) | `[drain.pipeline] max_check_slots` (existing field; enforced in build job 3; the name stays, its meaning widens to "build slots") |

The point of the split: an agent is mostly thinking, not compiling. Model time was 35.0% of the sample's wall
time and local build and test 7.2%; the rest was waiting. A single pool of "workers" would either hold a
10 to 12 GB build directory for an agent that is waiting, or hold an agent back because a build is running.
Two pools let many agents wait cheaply and keep the disk-heavy work to a number the disk can hold.

## 4. The build-slot queue

**A slot** is a counting semaphore of `max_check_slots` entries, kept as lock files under the Athena state
directory so that the drain, `athena go` and a producer's shell all see the same count. The existing job and
repository locks in `crates/athena-core/src/state.rs` are the model; a held lock dies with its process, so a
crashed job frees its slot without a sweeper.

**Who asks.** Every cargo command that compiles. In code: `SnapExec::exec` in `crates/athena-core/src/run/checks.rs`
(local checks and reruns); the producer's cargo commands (a wrapper placed first on the agent's `PATH`, build job 4);
`rebuild_if_due` in `crates/athena-cli/src/drain.rs` (the landing rebuild, build job 7).

**Order.** Strict priority classes, first in first out inside a class, with ageing:

1. Landing rebuild. It runs under the `landing` lock, so every ready landing waits behind it.
2. A check or rerun of a candidate (a job that is about to finish, which frees its worktree and directory).
3. A producer's cargo command, ordered by the job's priority tier (`priority-1.json`: urgent, then high, then the
   rest) and then by the time the command asked.

A request that has waited longer than `build_age_promote_s` (a parameter; initial value 600 s, an estimate, about
one CI run) moves up one class, so class 3 cannot starve behind a steady stream of class 2.

**Timeout.** A request waits at most `build_wait_s` (parameter; initial value 1800 s, equal to `check_timeout_s`).
Past it:

- For a check or rerun, the job is **held, not failed**: it closes with no verdict and the reason
  `environment: build slot wait`, resumes by the existing hold-and-resume path (`disk.rs`, `LowDisk`, resume mark)
  and uses no attempt. A held job is not a work failure for introspection or the scope guard.
- For a producer's cargo command, the wrapper exits 75 with the message "no build slot free for 1800 s; try
  again" and the producer decides (it may retry); this is an environment message, not a policy denial.
- For the landing rebuild, the drain does not time out: a rebuild that cannot start stops landings, as a failed
  rebuild does (D140 in `async-drain.md`).

A slot's own run is bounded as today (`check_timeout_s`, Bash timeouts), so a stuck holder cannot block the queue
past those limits. A check killed at its deadline already closes `environment: check timed out` (`disk.rs`).

**What the agent slot does while a build waits.** Nothing is held but its place: a producer waiting for a build
slot is blocked in its cargo command, which counts as model-slot time (the agent process exists). That is why the
agent count is set by memory (section 7), not by builds.

## 5. Sharing compiled output so a build costs far less disk

### 5.1 The numbers this rests on

- **Cold build, one job: 10 to 12 GB (stated in the request).** No record in the snapshot measures it: the records
  directory has no `athena.build-measurement/1` record (listing of `records/`: no such directory), and the code
  falls back to `ESTIMATE_T_BYTES` = 6 GB (`disk.rs`), a design estimate that is below the stated figure. Build job 1
  produces the measured figure. Until then the arithmetic in section 7 uses 10 and 12 GB.
- **Worktree, one agent: 0.2 GB (estimate, `ESTIMATE_S_BYTES`).**
- **The local-fallback rate is small.** In the sample, 4 of 5 jobs ran no local check (reused from CI), so the
  disk is mostly taken by the producer's own builds and by the occasional local rerun.

### 5.2 Alternatives

| Option | Effect on disk per build | Effect on time | Call |
| --- | --- | --- | --- |
| A. One shared `CARGO_TARGET_DIR` for all jobs | Best (one directory) | Cargo locks the directory, so builds queue; jobs overwrite each other's workspace crates | **Rejected**, as in `warm-baseline-build.md` section 5 and D46 (`env.rs` refuses a build directory outside the job's directory) |
| B. `sccache` shared compile cache | **None**: every job still writes a full `target/`; it only skips compiling | Cuts compile CPU where path and flags match; wiring reverted after a report of 0 hits (D84), cause not found, and per-job directories widen path differences | **Try again first, as a measurement only** (build job 1), because the code already exists. It is not the disk answer |
| C. Warm baseline copy of `main`'s `target/` per job (reflink or hard-link copy) | Near zero extra if the filesystem supports reflink; a full copy otherwise. The filesystem was not checked | Third-party crates warm; workspace crates still rebuild (fresh modification times, `warm-baseline-build.md` section 3) | **Rejected for now:** depends on a filesystem feature not verified here, and the refresh job it needs was already judged not worth it |
| D. **Slot-owned warm target directories** | **One directory per build slot, not per job**: disk is slots x one directory, however many agents there are | Third-party crates stay compiled between the jobs that use the slot; workspace crates rebuild | **Chosen** (below) |
| E. Smaller outputs: `CARGO_PROFILE_DEV_DEBUG=line-tables-only` (or 0) and `CARGO_INCREMENTAL=0` in `[capabilities.change.env]` | Cuts the size of each directory | Slightly faster links (estimate); no effect on test results, though a test that reads debug information would notice | **Chosen, with D**; it is a Cargo feature, not new code. It edits `athena.toml`, a governance file, so it is its own job by the maintainer |

**Buy-first call.** Cargo's own profile settings (E) and the existing `sccache` support (B) cost no new code, so they
come first and are measured. A custom cache, a copy-on-write layer or a new cache service (C) is built only if
build job 1 shows that D and E leave a build over the slot budget. D is new code but small: a directory that
outlives a job, a lock, and one `cargo clean -p`. No maintained tool offers "a target directory owned by a slot"
as such, and Cargo has no stable shared third-party cache, so this part is built.

### 5.3 The scheme (D with E)

1. At start, Athena makes `max_check_slots` slot directories `<state>/build-slots/<n>/target`.
2. A job that wins slot `n` gets `{job_dir}/target/<side>` as a symlink to it for the duration of one command. The
   path inside the job directory still satisfies the rule in `env.rs` (lexical, inside the job's directory), so the
   D46 isolation rule is not relaxed; whether the check in `env.rs` is lexical or resolves links must be confirmed
   by reading its body, which I did not (I read the module comment and the constant).
3. When the command ends, Athena runs `cargo clean -p <each workspace member>` in the slot directory, which removes
   workspace-crate output and keeps third-party output. This is what stops the slot growing by one set of workspace
   crates per job (each job's snapshot has a different path, so fingerprints differ).
4. A slot directory over `slot_max_gb` (parameter) is deleted whole and rebuilt cold on the next use.

**Disk per build slot (estimate): 5 to 6 GB**, that is, about half the stated 10 to 12 GB, reasoning: E removes most
debug information (which is the bulk of a debug `target/`), and D keeps only dependencies. This is an estimate with
no measurement behind it; build job 1 and build job 5 replace it with a measured figure. **Disk per additional job
is about 0 for the build** (the job reuses a slot) plus its 0.2 GB worktree.

Risks. A stale or corrupt slot gives a wrong build; `Cargo.lock` is `--locked` and the slot is deleted on a failed
build whose error is a cargo internal error, not a test failure. Two commands never share a slot at once. If the
workspace crates' output turns out to be most of the size (the estimate is wrong), D saves little, which build job 5
reports.

## 6. The landing-and-rebuild step

### 6.1 What `status --snapshot` shows, and why it is probably an artifact

`status --snapshot` shows landing and rebuild as a median of **1069.2 minutes** against about 21.5 for the other five
stages together (section 2.4): the longest stage by a factor of about 50. Code: `crates/athena-core/src/status.rs`: `landed_jobs` (line 895) sets the stage to
`minutes_between(last_end, landed_at)`, where `landed_at` comes from `landing_end` (line 811): the later of the
landing outcome and `landing_record_time` (line 795), which takes `.max()` of `created_at` over **every** landing
record whose commit list holds a `job_landing` commit of the job.

The records show that a job's commit appears in many landing records, not one. Job `20261104-054754-b4f5` has a
`job_landing` entry in **19** of the 174 `landings/` records (the files named dated 2026-11-04 unless marked: `20261104-062243-y5nm`, `073633-hkk0`,
`120040-nb2h`, `120408-rjyw`, `140457-d97c`, `183229-6dhz`, `191820-ny02`, `201108-2zzz`, `211411-rts4`,
`20261105-014757-7hd7`, `025930-qgxb`, `043742-nr97`, `053534-fjjm`, `063017-gfbj`, `070121-f1sw`, `083451-wmj7`,
`093306-sebk`, `162908-46xa`, `173027-xkkc`), always with the same commit tree `615c277b53116af0f093e19413ee544800d4ac83`. The
first is pull request 380 at 06:22:43 on 2026-11-04, 256 s after the job's verifier ended; the newest is pull
request 415 at 17:30:27 on 2026-11-05, 35.2 hours after it. `status` takes the newest.

For the five sampled jobs, minutes from the verifier's `ended_at` to the first landing record that lists the job
versus to the newest record (`landings/20261105-173027-xkkc.json`, `created_at` 2026-11-05T17:30:27Z):

| Job | To first record | To newest record |
| --- | --- | --- |
| `20261104-054754-b4f5` | 4.3 min | 2112.0 min |
| `20261104-194933-s8v3` | 11.0 min | 1290.4 min |
| `20261104-171135-911m` | 4.6 min | 1382.6 min |
| `20261104-153614-pzfw` | 4.8 min | 1337.0 min |
| `20261104-141009-hsg4` | 4.5 min | 1220.8 min |
| **Median** | **4.6 min** | **1337.0 min** |

The median to the newest record, 1337.0 minutes, is the same order as the 1069.2 that `status` prints (a different
window: the newest 20 landed jobs, which land closer to the newest record). The first-record figure is 4.6 minutes.

What is measured and what is inferred. Measured: the records above, and the code's `.max()`. Inferred, not checked:
that a landing record lists earlier jobs' commits again because the branch a squash lands from still carries them,
and that the first record is the real merge. Build job 2 checks the second against the pull request's merge time
before it changes anything. The earlier design note `continuous-landing.md` (section 2) reasons that the 440.1-minute
median it read is queueing behind other jobs or time in `Waiting`; the records above show the stage is dominated by
a different cause (the record chosen), so that note's "wrong phase" risk (section 7 there) is the one that
happened, and its build job 1 (phase split) should be done on the first-record measure.

The rebuild itself (`setup.sh`, a `--release` build of `xtask`, run in the checkout by `rebuild_if_due`) is in no record:
`continuous-landing.md` section 2 says `hestia.md` records that no record separates it from the stage (I did not
open `hestia.md`), and `continuous-landing.md` build job 5 adds its duration R. Until R exists, this note makes no claim about the rebuild's size.

### 6.2 How to shorten it

1. **Measure it right** (build job 2): end the stage at the first landing record that lists the job. Expected stage
   median for the same 20 jobs: single digit minutes instead of 1069.2 (estimate from the sample's 4.6).
2. **Record the rebuild's duration R and count per window** (`continuous-landing.md` build job 5), so a rebuild that
   is long is visible. Reused, not repeated here.
3. **Batch the rebuild and keep waiting jobs in the queue** (`continuous-landing.md` build jobs 2 to 4). Reused as
   written; this note changes only that the batched rebuild takes a build slot at class 1 (build job 7), so it never
   waits behind a producer's cargo command and never runs beside two other builds on 4 CPUs.
4. **Do not warm the rebuild with a slot directory.** `setup.sh` builds `--release` in the checkout, a different profile
   from the debug slots. Its own incremental `target/` (the checkout's) is what keeps it warm, and no slot can help.

## 7. How many at once on this machine

Inputs: 4 CPUs, 15 GB memory, about 27 GB free disk (stated); 10 to 12 GB per cold build (stated); reserve 10 GB
(`reserve_disk_gb` and `DEFAULT_MIN_FREE_GB` in `crates/athena-data/src/config.rs`); worktree 0.2 GB (estimate,
`ESTIMATE_S_BYTES`). Everything about memory and CPU use below is an **estimate**; none is in a record.

### 7.1 Build slots

Disk, today (one directory per job, no sharing):

- Usable for builds: 27 - 10 = 17 GB.
- Builds that fit: floor(17 / 12) = **1**; with 10 GB, floor(17 / 10) = **1**.
- The existing formula with `max_check_slots` = 1: floor((27 - 10 - 1 x 12) / (12 + 0.2)) = floor(5 / 12.2) = 0, and
  with T = 10, floor(7 / 10.2) = 0. Zero extra jobs fit beside the check slot: one build in total.
- `athena.toml` sets `max_concurrency` = 2. Two producers each at 10 to 12 GB need 20 to 24 GB against 17. On the
  stated free space, 27 - 20 = 7 GB or 27 - 24 = 3 GB would be left, below the 10 GB guard. The records show this
  failure: 6 `outcome.json` files under `jobs/` hold the reason text "low disk" (`jobs/2026-10/20261031-095235-zq9x`,
  `20261028-065911-m6m5`, `20261030-070002-d103`, `20261028-055601-hg0n`, `20261030-125332-sz3z`,
  `20261025-033738-yz2d`), and the brief for this job lists the class "low-disk-ends-job-failed-no-automatic-cleanup"
  at 21 cases in 19 jobs. Many held jobs end with no outcome file, so 6 is a floor.

Disk, with slot-owned directories (section 5.3), at 5 to 6 GB per slot (estimate):

- Slots by disk: floor(17 / 6) = 2; floor(17 / 5) = 3.

CPU, 4 CPUs (estimate): agents use about 1 core in all (section 7.2), leaving 3. A compile uses what
`CARGO_BUILD_JOBS` allows (set in `[capabilities.change.env]`, which agents cannot override: `policy.rs` refuses
cargo options that set the build configuration). Two slots at `CARGO_BUILD_JOBS` = 2 take 4 job-threads on 3 free
cores, a mild overload; three slots at 2 would be 6 on 3. Slots by CPU: **2**.

**Answer: 1 build at a time today; 2 at a time after build jobs 3 to 5.** A third never fits both limits.

### 7.2 Agent slots

Memory, 15 GB (estimate): keep 1.5 GB for the system, the drain and git. Two builds at about 3 GB each at
`CARGO_BUILD_JOBS` = 2 take 6 GB (a link of a large test binary can take more; 4 GB each would be 8). An agent
process (the host program and its tools) at about 0.5 GB:

- (15 - 1.5 - 6) / 0.5 = **15** agents.
- Pessimistic: 0.6 GB per agent and 8 GB of builds: (15 - 1.5 - 8) / 0.6 = 9.2, so **9**.
- Optimistic: 0.3 GB per agent and 6 GB of builds: 7.5 / 0.3 = **25**.

Disk for agent directories: 17 GB less the slots (2 x 6 = 12 GB) leaves 5 GB; 5 / 0.2 = **25** worktrees. Today,
with one 12 GB build: (17 - 12) / 0.2 = 25 as well.

CPU for agents: an agent is mostly waiting on the model, so assume 4% of one core on average (estimate); 16 agents is
0.6 core, 25 is 1.0 core.

**Answer: 12 agent slots to start, 16 after build job 1 has measured agent memory; "dozens" (24 or more) is possible only
if an agent costs 0.3 GB or less, which no record shows.** The binding limit is memory, not disk or CPU.

### 7.3 Is that enough for the work?

From the sample (section 2.2), the mean model time per job is 4778.4 / 5 = 955.7 s and the mean wall time 13637.2 / 5 =
2727.4 s. If an agent slot were held for the whole wall time, 12 slots would carry 12 / 2727.4 s = 15.8 jobs per
hour (about 380 a day). The landed rate is 14 on 2026-11-04 (status throughput line). So 12 slots is above the
present demand by a wide margin even with the wait held, and far above it if the wait is not held. Builds: if the
whole producer time were cargo time (an upper bound; the median producer stage is 12.8 min), 2 slots carry
2 / (12.8 x 60 s) = 9.4 jobs per hour, again above 14 a day (0.6 per hour). This is arithmetic on five jobs and on a
median, so it says only that this machine is not the limit at the present rate; it is the limit only at a rate
the records do not show.

Where "dozens" would bind first is not the machine but `plan.rs`: only the items of one plan group run side by
side (no dependency, disjoint touched paths), so the agent count in use is capped by group size. The group sizes
are not measured here; build job 6 records them.

## 8. Build order

Each change is one focused job with its own tests, lands on its own, and reads its effect from a named metric.
"Depends on" lists only a real need.

1. **Measure agent memory, build memory, CPU and the slot-relevant sizes.**
   Touches: `crates/athena-core/src/disk.rs` (`measure_build`, `CargoBuilder`: add the peak resident memory and
   cores used of each cold build), `crates/athena-data/src/records.rs` (`BuildMeasurement`: optional new fields,
   old records still read), `crates/athena-core/src/status.rs` (`build_measurement_line`: print them). Also a one-off
   `sccache` run through the same builder (code exists).
   The two build figures are **build totals**, not per-process figures: the peak memory is the highest resident memory
   summed over all processes of the build at one sampled moment, and the cores figure is the highest number of build
   processes alive at one moment. Both include the compilers an `sccache` server runs for the build (found by an
   environment mark the build carries), and `athena status` labels them "build total peak memory" and "peak
   concurrent processes". Resident memory counts shared pages once per process, so the sum can read high. Agent
   memory is not measured by this step; a later step measures it.
   Effect measured by: the new fields in the `athena.build-measurement/1` record in `records/`, read by the
   `build` line of `athena status`. Replaces every **estimate** in section 7 (T, peak memory) with a measured number
   and answers whether `sccache` gives a nonzero hit count (`SccacheFigures` in that record).
   Depends on: nothing.
2. **End the landing stage at the first landing record.**
   Touches: `crates/athena-core/src/status.rs` (`landing_record_time`: `.min()` instead of `.max()`; tests with two
   records that list the same job commit). First, confirm in the pull request's merge time that the first record is
   the merge.
   Effect measured by: the `landing and rebuild` median in the throughput section of
   `cargo run -p athena-cli -- status --snapshot <dir>` (before: 1069.2 min over the last 20; expected after: single
   digit, from 4.6 in the sample, an estimate), and the unit test. The throughput review's stored comparison
   (`StageMinutes::median_of`) is recomputed on the new rule, so reviews before and after are not compared directly.
   Depends on: nothing.
3. **Enforce `max_check_slots` as a counting build-slot queue around local checks.**
   Touches: new `crates/athena-core/src/slots.rs` (lock-file semaphore, class order, ageing, `build_wait_s`);
   `crates/athena-core/src/run/checks.rs` (`SnapExec::exec` takes a slot per check); `crates/athena-data/src/config.rs`
   (`PipelineConfig`: `build_wait_s`, `build_age_promote_s`, both with defaults); `crates/athena-core/src/disk.rs`
   (the new hold reason `environment: build slot wait`, resumable).
   Effect measured by: a `queued_s` field on each check in `verification.json` (basis "athena monotonic clock"),
   summed by a new "build wait" line in the throughput section of `status --snapshot`; and the count of jobs
   closed with reason text "low disk" (`records/jobs/**/outcome.json`; 6 today).
   Depends on: nothing (a limit of 1 reproduces today's one-build behaviour).
   **Built** (local-check part only). What was chosen where the note left room:
   - The slot files are `slots/slot-<n>.lock` under the state directory. A waiting request holds a locked ticket
     file under `slots/queue/`, created under a temporary name, locked and then renamed into place, so a visible
     ticket whose lock is free belongs to a dead process and is removed. Only the head ticket (class, then time
     asked) tries the slot files. Requests poll every 250 ms; there is no daemon.
   - Ageing moves a request up one class for each `build_age_promote_s` it has waited, down to the top class.
     Only the check class is requested now; the landing and producer classes exist in `slots.rs` for build jobs 4
     and 7.
   - The slot is taken after the first-check disk gate and held until the verifier's snapshot is restored. A
     check that had to wait runs the disk gate once more before it starts, since the first reading may be a
     queue wait old.
   - The slot is per check, not per job. The baseline run and the "fails without" reruns are not wrapped yet.
   - `athena stop` still ends a wait: the kill switch is read on every poll.
   - The hold marker for a slot wait shares the file of the low-disk marker with another shape; `athena status`,
     `athena go` and the drain read either, and the drain reports the outcome `environment: build slot wait`, so
     the count of "low disk" outcomes above stays a count of disk holds.
   - `queued_s` is stored only for checks that took a slot; the "build wait" line prints the sum, the number of
     such checks and the longest wait. The peak number of slots held at once is not recorded yet (build job 4).
4. **Take a build slot for each producer cargo command.**
   Touches: `crates/athena-agent/src/env.rs` (puts a wrapper directory first on the agent's `PATH`); new
   `crates/athena-cli/src/slot_exec.rs` and its wiring in `crates/athena-cli/src/main.rs` (a `build-slot -- <cmd>`
   subcommand that waits in the queue, runs the command and frees the slot at exit or signal);
   `crates/athena-agent/src/policy.rs` (the wrapper is the only form of `cargo` the policy allows).
   Effect measured by: the same `queued_s`, recorded per producer command in the invocation record's
   `commands_run`, read by the "build wait" line; and the peak number of slots held at once, which the queue records and the same line prints (it must never
   exceed `max_check_slots`).
   Depends on: 3.
5. **Slot-owned warm target directories and smaller outputs.**
   Touches: `crates/athena-core/src/state.rs` (slot directories under the state directory, outliving a job),
   `crates/athena-agent/src/env.rs` (the per-command symlink under `{job_dir}/target/<side>`),
   `crates/athena-core/src/disk.rs` (`Figures`: T becomes per slot; the bound uses slots, not jobs; a slot over
   `slot_max_gb` is removed). The `CARGO_PROFILE_DEV_DEBUG` and `CARGO_INCREMENTAL` settings go in
   `[capabilities.change.env]` of `athena.toml`, a governance file: a separate job by the maintainer, with this note as its
   citation.
   Effect measured by: `athena measure-build` run on a slot after a job (the record's `target_bytes`, read by
   `athena status`; target 5 to 6 GB per slot against the measured cold figure from job 1), and the wall time of the
   second build in the same slot (the record's two build times).
   Depends on: 1 (the cold figure to compare with) and 3.
   **Built** (local checks only; `athena.toml` not touched). What was chosen where the note left room:
   - `<state>/build-slots/<n>/target` exists for each of the `max_check_slots` slots from the first local check of a
     run (`StateDir::ensure_slot_targets`). Nothing removes it when a job ends, and closing a job's directory
     does not reach it. A slot's number is the `<n>` of its `slots/slot-<n>.lock`, so one lock holder owns one
     directory and two commands never share one.
   - After the slot is won, the job's `CARGO_TARGET_DIR` path (`{job_dir}/target/verify`) becomes a symbolic link to
     the slot's directory for that one command (`athena_agent::env::BuildLink`), and is removed when the command
     ends, on every path. The lexical check in `env.rs` (`check_build_dirs`, which resolves `.` and `..` and
     follows no link) is unchanged and passes, since the link lies inside the job's directory; `BuildLink::create`
     runs the same check on the link path before it links. If the slot's directory cannot be readied or linked, a
     warning line is written and the check builds in the job's own directory, cold, as before.
   - After the command, and before the verifier's snapshot is restored, `cargo clean --offline --target-dir <slot>
     -p <member>` runs for every workspace member found by `cargo metadata --no-deps`
     (`slots::clean_workspace_output`), which removes workspace-crate output and keeps third-party output. If the
     clean fails, the whole slot directory is deleted and made again, so a later job never builds on output it
     cannot trust. A snapshot with no `Cargo.toml` has nothing to clean.
   - Finding while building it: `cargo clean` refuses a target directory without a `CACHEDIR.TAG` file, and cargo
     writes that file only into a directory it creates itself. A slot directory made empty by Athena therefore
     gets the tag when it is made (`state.rs`); a first version without it deleted the slot after every command.
   - `slot_max_gb` (`[drain.pipeline]`, default 8, at least 1; 10^9 bytes) is read before each command: a slot
     directory whose allocated size (each hard-linked file once) is over it is deleted whole and made again
     empty, and a line `disk       build slot <n> target directory held ...: deleted` says so. The check is
     before use, not after, so a directory over the limit stays on disk until the slot is next won. The default
     8 GB sits above the estimate of 5 to 6 GB in section 5.3; it is a guess, not a measurement.
   - The in-flight bound in `disk.rs` (`in_flight_limit`) already counts `max_check_slots x T`, so it uses
     slots, not jobs, and needed no change. `Figures` still takes T from the latest cold-build measurement; T is
     not yet measured per slot.
   What was measured, and what was not. Tests show the behavior: two jobs one after the other get the same
   `build-slots/0/target` and the second sees the first's third-party output; after a command a real
   `cargo build` of a one-crate workspace leaves no `alpha` output in the slot while a planted third-party file
   stays; a directory over the limit is deleted and a smaller one is kept (in a test the gigabyte is shrunk, so
   no test writes one). **Not measured:** the wall time of the second build in the same slot, and the size of a
   slot after real jobs. `athena measure-build` still builds cold in its own worktree and records one build
   time, and no record carries the time of a build in a warm slot, so the "second build" effect named above
   cannot yet be read from `records/`. The check's own `duration` in `verification.json` is the only time kept.
   Recording a warm-slot build time and the slot's `target_bytes` after a job is left to a later job.
6. **Raise the agent bound and record group size.**
   Touches: `crates/athena-data/src/config.rs` (`MAX_CONCURRENCY_CEILING` from 8 to 16); `crates/athena-core/src/plan.rs`
   (record how many items each plan group held); `athena.toml` `[parallel] max_concurrency` and
   `[drain.pipeline]` (governance file, set by the maintainer after jobs 1 and 3 report).
   Effect measured by: peak in-flight jobs and mean group size in the drain's end-of-run lines, and the count
   of "low disk" outcomes (`records/jobs/**/outcome.json`; 6 so far, all dated 2026-10-25 to 2026-10-31) not rising
   while `max_concurrency` goes up. Rollback: set `max_concurrency` to 2.
   Depends on: 1, 3, 4.
   Built: `MAX_CONCURRENCY_CEILING` is 16 (it was 8), so `max_concurrency` and `max_in_flight` accept 1 to 16 and
   17 is refused; `athena.toml` sets `max_concurrency` to the starting value 12. `athena drain --help` takes the
   ceiling from the constant, so help and schema cannot differ. The drain `Report` now holds `group_sizes`: the
   item count of each plan group, in the order worked, and a group the run never reached is not listed. The
   end-of-run output prints one `groups` line with the count, the sizes and the mean. It is kept in memory for
   the run; no record file is written, and `plan.rs` needed no change. **Not measured:** peak in-flight jobs and
   the count of "low disk" outcomes after the raise; both are left to reading real drains.
7. **Put the landing rebuild behind a build slot at the top class.**
   Touches: `crates/athena-cli/src/drain.rs` (`rebuild_if_due` acquires a slot of class 1 before running
   `setup.sh`; the `landing` lock stays); `crates/athena-core/src/slots.rs` (class 1).
   Effect measured by: the rebuild duration R and runs per window recorded by `continuous-landing.md` build job 5, read by
   `status --snapshot`; and the "build wait" line showing no class-1 request waited more than one build.
   Depends on: 3, and `continuous-landing.md` build job 5 for the metric.

## 9. Alternatives considered for slot control

| Option | Why not |
| --- | --- |
| One pool of workers, as today, raised to dozens | Each worker would own a 10 to 12 GB directory while it waits: 17 / 12 = 1 worker fits. Held jobs pile up in the low-disk path |
| Raise only `max_concurrency` and rely on the disk guard | `async-drain.md` section 9 alternative a; the guard holds a job after it has used time, and the "low disk" outcomes (6 files) are the cost |
| A fixed `-j` per job and no queue | CPU shares evenly but disk still grows per job |
| A global lock so only one build ever runs | Right on disk today, but it wastes the second slot that the slot-owned directories (section 5) make affordable |
| Queue in the model host instead of Athena | Athena owns the clock, the records and the hold-and-resume path; a queue elsewhere would leave no record of the wait |

## 10. Out of scope

- Any change to `crates/`, `xtask/`, `athena.toml`, `defaults.toml` or the workflows (this note is design only).
- Editing the existing design notes or re-running any throughput review.
- Measuring the machine: no tool in this job could read memory, CPU use or the filesystem type, so every such
  figure is labeled **estimate** or **stated** above.

## Questions for the maintainer

- Which filesystem holds the state directory? If it supports reflink (for example XFS or btrfs), alternative C
  becomes cheap and may beat D on disk.
- Is 10 to 12 GB a debug build of the whole workspace with tests (`cargo test --locked`)? Build job 1 will say how
  much is debug information.
- Are 12 agent slots enough, or is the "dozens" target a throughput need? Section 7.3 says the present rate needs far
  fewer.
