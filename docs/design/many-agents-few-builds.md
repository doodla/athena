# Many agents at once, few builds at once

Status: design note, proposed. Nothing in `crates/`, `xtask/`, `athena.toml`, `defaults.toml` or
`.github/workflows/` changes with this note. Written for the owner, who decides whether to fund the work, and for
a reader outside the repository.

The owner wants dozens of agents running at once on one machine: 4 CPUs, 15 GB of memory, about 27 GB of free
disk. Today `[parallel] max_concurrency` may not exceed 8 (`MAX_CONCURRENCY_CEILING`), and each job builds Rust
in its own target directory of about 10 to 12 GB, so the disk holds about two builds. Most of a job's time is
model time, so the idea is to run many agents and few builds.

Labels: **measured** (read from the job records in the evidence snapshot, source named), **estimate** (not
measured, reasoning shown), **new** (not built), **already present** (built; file named).

## 1. Short answer

| Question | Answer | Basis |
|---|---|---|
| How many agents at once? | **16** to start, up to about 20 if one agent measures at or below 0.4 GB | Memory arithmetic in section 6.1 (estimate: agent memory is not recorded today) |
| How many builds at once? | **2** build slots, each with `CARGO_BUILD_JOBS=2` | CPU arithmetic in section 6.2; disk arithmetic in section 6.3 |
| Must `MAX_CONCURRENCY_CEILING` be raised? | **Yes**, from 8 to 32 (16 needs more than 8; 32 leaves room without another change) | Section 6.4 |
| Does more agents mean more landed jobs? | **Not by itself.** Over the last 3 days the second agent slot was used for 0.17 percent of busy time, and with today's local build time two build slots keep only about 3 agents busy | Sections 2.3 and 6.5 |
| How much disk per job? | **About 0.9 GB averaged over 16 jobs in flight**, against 10 to 12 GB now | Section 5.3 (estimate, to be replaced by step 1's measurement) |

Funding view: the build-slot queue and shared compiled output are worth building, but the number of agents
will not pay off until the local build time per job falls to about 135 seconds (it was 688 seconds on average
over the last 3 days; on 2026-11-05 it was about 108 seconds because CI carried the checks). Section 7 orders the
work so each step measures this.

## 2. Measurements: where a job's wall time goes

### 2.1 Window and method

Window: the last 3 UTC days, 2026-11-03, 2026-11-04 and 2026-11-05 (up to the snapshot, last job directory
`20261105-163159-vsfy`). Source: `records/jobs/2026-11/<job id>/` for every job whose id starts `20261103`,
`20261104` or `20261105`. About 136 such directories exist; **59** have an `outcome.json` and at least one file
under `invocations/`, and those 59 are measured here. The other directories were not classified (this note does not
say why they have no outcome). All 59 are listed with their numbers in Appendix A.

Per job, from the files named:

- **Wall time**: `outcome.json` field `totals.wall_time_execution` ("athena clock, from dispatch to the final
  verdict"). Five dropped jobs have no such field (`20261103-075009-jzqq`, `20261104-102639-h6ep`,
  `20261104-105736-w8b9`, `20261104-214954-x6v9`, `20261104-215118-d9mm`, marked `*` in Appendix A); for them the
  wall time is the first invocation start to `outcome.json` field `at`.
- **Model time**: the sum of `wall_time.value` over `invocations/NN-<role>.json` (roles reader, planner, critic,
  producer, verifier). Time that an agent spends running `cargo` through its Bash tool is inside its invocation,
  so it counts as model time here. How large that share is was **not measured** (step 1 measures it).
- **Local build and test time**: the sum of `checks[].duration.value` over checks whose `reused` is not true, in
  `verification.json` and `baseline.json` of the job. A check whose result was taken from a CI run has duration
  0.0 and `"reused": true`, so it adds nothing here.
- **Waiting**: wall time minus model time minus local check time. It is a residual, not a clock reading.

The figures come from a throwaway reader program that parsed those fields; it is not committed. Anyone can redo
the sums from the files named. Cost is the sum of `cost.value` of the same invocations.

### 2.2 The split (measured)

| Group | Jobs | Wall time | Model time | Local build and test | Waiting |
|---|---|---|---|---|---|
| All 59 jobs | 59 | 155,038 s (43.1 h) | 63,594 s (41.0 %) | 40,595 s (26.2 %) | 50,849 s (32.8 %) |
| Without the 5 dropped jobs | 54 | 135,349 s | 63,235 s (46.7 %) | 40,595 s (30.0 %) | 31,519 s (23.3 %) |
| Landed only | 28 | 85,715 s | 44,680 s (52.1 %) | 22,474 s (26.2 %) | 18,561 s (21.7 %) |
| Rejected only | 18 | 39,248 s | 10,462 s (26.7 %) | 15,968 s (40.7 %) | 12,818 s (32.7 %) |
| Failed (7) and timed out (1) | 8 | 10,387 s | 8,093 s | 2,152 s | 142 s |

Per job: median wall time 2,123 s, median model time 593 s, median local check time 907 s; mean model time 1,078
s and mean local check time 688 s. Of the 59 jobs, 42 ran at least one check locally; their median local check time
is 970 s. Total cost of the 59 jobs: 136.59 USD (about 2.31 USD per job, reported at list price).

Model time by role (all invocations, `invocations/*.json` of the 59 jobs):

| Role | Invocations | Total | Median per invocation |
|---|---|---|---|
| producer | 51 | 35,262 s | 305 s |
| verifier | 46 | 8,977 s | 174 s |
| planner | 65 | 9,835 s | 146 s |
| critic | 57 | 6,252 s | 91 s |
| reader | 107 | 3,267 s | 30 s |

What "waiting" holds is **not decomposed**. For landed jobs it averages 663 s (18,561 s over 28). The measured CI
run duration (median 583 s, range 400 to 690 s, 53 runs) in `docs/design/warm-baseline-build.md` section 2.1
suggests that most of it is polling for a CI result before a check can be reused. That reading is an **estimate**.
Disk holds and the drain's own scheduling are the other candidates; step 1 records the reasons.

### 2.3 How much of the machine was used at once (measured)

From the same `invocations/*.json` files, `started_at` and `ended_at` of all invocations of the 59 jobs, merged
into one timeline:

- Peak number of agent invocations running at once: **2**.
- Busy time with 1 invocation running: 63,376 s. With 2 running: 109 s, **0.17 percent** of 63,485 busy seconds.

`athena.toml` sets `[parallel] max_concurrency = 2`, so the setting was there and was almost never used. The drain
runs only the independent items of one plan group side by side and starts the next group when the group before it
is done (`crates/athena-cli/src/drain.rs`, `run_batches`). The likely cause is a shortage of independent items,
not of slots. That is an **estimate**: the records do not say why the second slot was idle, and step 2 measures it.
Raising the number of agent slots alone would therefore have changed little over these 3 days.

### 2.4 The local build share is falling (measured)

Jobs with local check time above 0 s, by job-id day: 2026-11-03: 20 of 26; 2026-11-04: 19 of 24; 2026-11-05: 2 of
9 (`20261105-044134-zycx`, `20261105-070552-kx1y`). On 2026-11-05 the nine jobs together used 974 s of local check
time, about 108 s per job, against 688 s per job over all 3 days. This matches `warm-baseline-build.md` section
2.2, where CI reuse rose from about 1 percent (October) to about 86 percent (November). The sample for 2026-11-05 is
small (9 jobs), so treat 108 s as a rough figure.

### 2.5 What was not measured

- Memory and CPU use of an agent process or a cargo run: no record holds them.
- Size of a target directory (T) on this machine: the repository has a record type for it
  (`athena.build-measurement/1`, written by `athena measure-build`), but **no such record exists in the evidence
  snapshot**, so `athena status` falls back to the design's estimate of 6 GB (`ESTIMATE_T_BYTES` in `disk.rs`). The
  owner's figure of 10 to 12 GB is used below as the working value for T, and the 6 GB estimate is stale.
- Time spent inside agent `cargo` calls, and how long two builds take when run at the same time.

## 3. Existing code this design builds on

| What exists | Where | What the design does with it |
|---|---|---|
| `MAX_CONCURRENCY_CEILING = 8` bounds `[parallel] max_concurrency` (default 1) and `[drain.pipeline] max_in_flight`; `max_check_slots` (1 to `MAX_CHECK_SLOTS_CEILING` = 8, default 1) bounds "checks running at once" | `crates/athena-data/src/config.rs`: `Parallel`, `PipelineConfig`, validation in `Config::validate` (the `ConfigError::Parallel` and `DrainPipeline` errors) | **Extends.** Raise the ceiling; make `max_concurrency` count agent slots; add an optional `max_build_slots`. `max_check_slots` today only feeds the disk formula |
| The disk guard: `[disk] min_free_gb` (default 10, `DEFAULT_MIN_FREE_GB`), the dispatch, pre-producer, pre-check and pre-verifier gates, removal of a job's `target/produce` before the checks, `reclaim_interrupted_targets`, and the in-flight bound `floor((free - reserve_disk_gb - max_check_slots x T) / (T + S))` (`in_flight_limit`, `start_decision`, `Athena::start_hold`) with T and S from the latest build measurement | `crates/athena-core/src/disk.rs` | **Extends.** The bound becomes a lane-based formula (section 5.3). `start_hold` is a gate, not a lock, and its own comment says two jobs starting at the same instant can both pass |
| `athena measure-build` builds the workspace cold in two worktrees, records T, S, build wall time and sccache counts | `disk.rs`: `Athena::measure_build`, `CargoBuilder`, `CargoBuild`; Hestia runs it when no record exists | **Reuses.** Step 1 adds the third-party versus workspace split to the same record |
| A build of `athena` for base and candidate, both into one shared `target` directory under the job's scratch directory, used for the reading replay | `crates/athena-core/src/run/mod.rs`: `replay_checkers` (the `CARGO_TARGET_DIR` set at the build loop) | **Precedent.** Two revisions of the workspace already share one target directory there, serially. It is not the producer's build directory |
| The producer's and verifier's `CARGO_TARGET_DIR` is `{job_dir}/target/{side}`, filled in from `athena.toml` `[capabilities.change.env]`; a build directory outside the job's directory is refused (D46); an agent starts with an empty environment | `crates/athena-agent/src/env.rs`: `build`, `expand`, `BUILD_DIR_VARS`, `check_build_dirs`; `crates/athena-core/src/state.rs`: `produce_dir`, `remove_produce` | **Changes** (step 5): the directory becomes a lane directory owned by a build slot, which needs a recorded decision because it relaxes D46 |
| Athena's own checks run locally through one function, after a CI-first routing step | `crates/athena-core/src/checks.rs`: `run_check`; `crates/athena-core/src/routing.rs`: `route`, `run_local` | **Extends.** `run_local` takes a build slot before it runs a check |
| The agent's `cargo` commands are allowed by a policy hook (`producer_commands`, `verifier_commands` in `athena.toml`) | `crates/athena-agent/src/policy.rs`, `crates/athena-cli/src/hooks.rs` | **Extends.** The allowed `cargo` goes through a slot-taking wrapper (step 4) |
| Exclusive file locks without a daemon: `try_lock_job`, `lock_repository`, `lock_file` | `crates/athena-core/src/state.rs` | **Reuses** the same lock style for build slots. A lock held by a process ends when the process dies, so a killed job frees its slot |
| Several jobs at once in one process, one thread per worker, up to `max_concurrency`; landing and rating stay one at a time | `crates/athena-cli/src/drain.rs`: `run_parallel`, `run_batches`; `docs/design/parallel-jobs.md` (first slice built) | **Reuses.** Agent slots are these workers; this design does not change landing |
| The accepted async-drain design: several jobs in flight, one check slot, one landing queue, with T and S as the disk figures | `docs/design/async-drain.md` sections 3, 5 and 11 | **Generalises.** A build slot is the check slot extended to every cargo run, including the producer's. It also records that sccache was removed (D84) after a report of 0 hits |
| CI-first checks, so most jobs do not build locally; warm baseline not recommended; shared target directory across jobs rejected; sccache "revisit only if compile dominates" | `docs/design/warm-baseline-build.md` sections 1, 3, 5 and 6 | **Builds on and refines.** Section 5 here keeps its finding that fresh mtimes force every workspace crate to rebuild and that only third-party dependencies survive. The "shared target directory" it rejected is one directory used by jobs *at the same time*; section 5 uses a directory used by *one job at a time* |

## 4. Agent slots and build slots

### 4.1 Two kinds of slot

- **Agent slot.** One job in flight, from dispatch to its verdict. It covers every model stage: reader, planner,
  critic, producer, verifier. A job holds its agent slot while it waits for a build slot, because it still owns a
  work tree and a place in the drain. The slot costs memory and spend, little CPU, and (with the changes below)
  about 0.4 GB of disk. Limit: `[parallel] max_concurrency` (existing).
- **Build slot.** The right to run `cargo` on this machine: any `cargo build`, `check`, `test`, `clippy` or `doc`
  command, whether Athena's own check or the agent's. `cargo fmt` is cheap and needs no slot. A slot owns one
  **lane**, a persistent target directory (section 5). Limit: `[parallel] max_build_slots` (new, optional; absent
  means no gate, which is today's behavior).

An agent slot is cheap on this machine because model work waits for a remote answer. A build slot is scarce
because a build uses all CPUs and a lot of disk and memory.

### 4.2 The queue

One queue for the machine, in the state directory (the same place as the job locks), first in, first out. It is a
directory of ticket files plus one lock file per slot:

1. A cargo run asks for a slot: it creates a ticket file `build-queue/<sequence>-<job id>-<purpose>` with a
   monotonic sequence number.
2. It tries each `build-slots/slot-<k>.lock` in turn with a non-blocking exclusive lock, but only when its ticket
   is the oldest waiting. The first lock it wins is its slot, and it uses lane `k`.
3. While it has no slot it sleeps (about 1 s), so waiting costs no CPU and no model tokens.
4. It deletes its ticket when it gets a slot or gives up.

### 4.3 Acquire and release around a cargo run

- **Athena's checks** (`routing.rs`, `run_local`, calling `checks.rs`, `run_check`): acquire before the check
  starts, hold a guard through the check, drop the guard when the check process ends, whether it passed, failed or
  timed out. The check's `CARGO_TARGET_DIR` is the slot's lane. Baseline checks use the same path. A CI result
  that stands in for a check never asks for a slot.
- **The agent's cargo** (step 4): the policy hook allows `cargo` only through a wrapper placed first on the
  agent's `PATH` (set in `env.rs`, `build`). The wrapper is a hidden subcommand of `athena` that (a) takes a
  ticket, (b) waits for a slot for at most 240 s (below the 600 s `bash_default_timeout_ms` in `athena.toml`),
  (c) runs the real `cargo` with `CARGO_TARGET_DIR` set to the lane and `CARGO_BUILD_JOBS=2`, (d) releases the slot
  when `cargo` exits. If no slot comes in 240 s it exits with code 75 and prints "build queue: position N, run the
  command again", so the agent retries instead of hitting the Bash timeout.
- **Release in every case.** The slot is an OS file lock, so it is released when the holding process exits, is
  killed by `athena stop`, or crashes. No cleanup process is needed.
- **Deadlines.** The producer deadline (`producer_timeout_s`, 5400 s) must not burn while the agent only waits in
  the queue; step 4 excludes the wrapper's recorded wait from that deadline. The failure class
  `producer-deadline-exceeded-on-small-job` has 12 recorded cases, and a queue that costs deadline time would add
  to it.
- **Order.** First in, first out in the first version. Whether Athena's checks should go ahead of an agent's
  exploratory `cargo check` is decided by the measured wait (step 3), not assumed.
- **Fairness and starvation.** FIFO cannot starve a ticket. A job that holds a slot is never preempted.

## 5. Sharing compiled output

### 5.1 What is shared and why it is safe

`warm-baseline-build.md` section 3 established that a fresh work tree gives every file a new modification time,
so every workspace crate rebuilds, and that only third-party dependencies (sources from the registry, keyed by
`Cargo.lock` and the toolchain) are reused. Sharing therefore means sharing **third-party build output**. Nothing
that a verifier relies on is taken from another job's workspace output: a lane is trimmed of workspace-crate
artifacts after every job (step 5), so a check always compiles the job's own code.

### 5.2 Alternatives

| Option | Disk per build | Build time | Risk and cost | Verdict |
|---|---|---|---|---|
| A. **Lanes**: B persistent target directories, one per build slot, used by one job at a time and trimmed of workspace output after each job | B x L, not N x T (L is the lane size, section 5.3) | Third-party dependencies warm after the first job in a lane. Workspace crates always rebuild | Contradicts D46 ("one job's checks never run another job's build"), so it needs a decision. A lane can carry stale state: trim it and compare verdicts of a cold and a warm run on a sample (step 5) | **Recommended** |
| B. One shared target directory, all jobs at once, per `Cargo.lock` hash | One T | Same warm gain as A | Cargo's directory lock makes jobs wait on each other anyway; one job's bad artifact reaches all. Already rejected in `warm-baseline-build.md` section 5 | Rejected |
| C. `sccache` with a local disk cache | Per-job T is still written while a build runs; the cache adds its own size (capped, for example 5 GB) | Caches by content, so it survives fresh mtimes and helps workspace crates too | Needs `RUSTC_WRAPPER` and `CARGO_INCREMENTAL=0`; was removed by D84 after a report of 0 hits that was never explained; does nothing for linking or test run time | Not now. Revisit only if step 1 shows compile time dominates local builds. `measure-build` already records sccache counts |
| D. Copy a warm baseline target into each job, by hardlink or reflink | Near zero extra on a filesystem with reflinks; a full copy where there are none | Warm | The filesystem type is not known to this note (not checked). Hardlinked files that cargo rewrites in place would damage the baseline; a refresh process is needed (the cost `warm-baseline-build.md` section 4 weighed) | Rejected for now |
| E. **Make each target smaller**: `CARGO_PROFILE_DEV_DEBUG=line-tables-only` and `CARGO_INCREMENTAL=0` in the capability environment | Cuts L and T, estimate 30 to 50 percent | Fewer bytes written, so builds are no slower (estimate) | Reduces debug information in local builds; changes `athena.toml` (a governance file, so a step for the owner to approve) | **Recommended with A**, independent of it |

Recommendation: **A together with E.** Add C later only on measured evidence.

### 5.3 Expected disk per job (estimates, from the arithmetic below)

Inputs: free disk 27 GB; `min_free_gb` 10 (the default, `DEFAULT_MIN_FREE_GB`), so **17 GB is usable**. T = 11
GB, the middle of the owner's 10 to 12 GB (not measured here). S_job = 0.4 GB per job in flight: two trees (work
tree and verifier snapshot) at the 0.2 GB design estimate of S (`ESTIMATE_S_BYTES`). Lane size L: third-party
output kept plus the running job's own workspace output. With no data, estimate L = 7 GB without E (about 60
percent of T) and 4.2 GB with E (40 percent less). Step 1 replaces these with a measurement.

| Plan | Disk used by build output | Disk used by 16 jobs' trees (16 x 0.4) | Total | Jobs in flight that fit in 17 GB | Per job at 16 in flight |
|---|---|---|---|---|---|
| Today: each job builds in its own T | 11 GB per job | 0.4 GB per job | 11.4 GB per job | **1** (17 / 11.4 = 1.49) | 11.4 GB |
| Lanes (A), 2 slots, L = 7 | 2 x 7 = 14 GB | 6.4 GB | 20.4 GB | 7 ((17 - 14) / 0.4) | 1.28 GB, but over 17 GB |
| Lanes (A), 1 slot, L = 7 | 7 GB | 6.4 GB | 13.4 GB | 25 | 0.84 GB |
| **Lanes with smaller targets (A + E), 2 slots, L = 4.2** | 2 x 4.2 = 8.4 GB | 6.4 GB | **14.8 GB** | 21 ((17 - 8.4) / 0.4) | **0.93 GB** |

Reading: today's disk holds one build (the owner's "about two" counts a job's build running while another's
finishes). With lanes and smaller targets the 16 jobs fit in 14.8 GB, about **0.9 GB per job against 10 to 12
GB**, and one more job in flight costs about 0.4 GB. Lanes without E fit 16 jobs only with one build slot. Records,
transcripts and logs are not in these figures.

The existing formula (section 3) changes to `floor((free - reserve_disk_gb - B x L) / S_job)` in step 6. Even today
it is built on the stale 6 GB estimate. With T = 11 GB it gives `floor((27 - 10 - 1 x 11) / 11.2) = 0` additional
jobs, and with the 6 GB estimate `floor((27 - 10 - 6) / 6.2) = 1`.

## 6. How many agents and builds this machine can run

### 6.1 Agents: memory

15 GB total. Reserve 3 GB for the operating system, Athena, git and the page cache (estimate). Reserve 4 GB for 2
builds at about 2 GB each, the peak of a link or test build (estimate). That leaves 15 - 3 - 4 = **8 GB for
agents**. An agent host process is a Node process of an estimated 0.4 to 0.5 GB resident (no measurement exists):
8 / 0.5 = **16**, or 8 / 0.4 = **20**. So 16 agents, up to 20 if measured.

CPU is not the limit for agents: an agent waiting on a model uses an estimated 5 percent of one core, so 16
agents use about 0.8 of 4 cores (estimate).

### 6.2 Builds: CPU

4 CPUs minus about 0.8 for agents leaves about 3.2. One cargo build already uses all cores for compiling and test
runs, so two builds do not double throughput: each gets about 1.6 cores. Two slots help because linking and test
runs leave cores idle, and a second build fills those gaps. Three or more only lengthen every build. So **B = 2**,
with `CARGO_BUILD_JOBS=2` per build. How much two simultaneous builds slow each other is **not measured**; the
accepted async-drain design (section 3) carries the same open question and step 8 measures it.

### 6.3 Builds: disk

17 usable GB (section 5.3). Today that holds one T plus its tree. With lanes and smaller targets, two lanes use
8.4 GB (estimate), so B = 2 fits with 8.6 GB left for trees.

### 6.4 The ceiling

16 agents need `max_concurrency` of 16, but `MAX_CONCURRENCY_CEILING` is 8, and `Config::validate` rejects a larger
value. **It must be raised.** Raise it to 32: this makes 20 possible if memory measures well, and 32 is not itself
a plan, only the validation bound. The same constant bounds `[drain.pipeline] max_in_flight`, so that bound rises
too. Raising the constant changes nothing at run time, because `athena.toml` keeps its own value (currently 2).
`MAX_CHECK_SLOTS_CEILING` (8) is enough for B = 2 and needs no change.

"Dozens" (24 or more) at once on this machine needs more memory: 24 agents x 0.5 GB = 12 GB, which leaves 3 GB
for everything else and cannot hold two builds. The owner can still have dozens of jobs **queued and in flight in
stages**; the machine supports about 16 to 20 running model work at the same time.

### 6.4a Spend

At 16 agents flat out, one job takes a mean of 1,078 s of model time (section 2.2), so the machine would finish
about 16 / 1,078 x 3,600 = 53 jobs an hour. At 2.31 USD per job (measured, section 2.2) that is about 122 USD an
hour (estimate; it assumes the failure mix of the last 3 days, of which 28 of 59 jobs landed, and enough
independent work to fill 16 slots). The per-job budgets in `athena.toml` do not limit this rate; the owner should
set a drain-level spending cap before step 8.

### 6.5 Do builds limit the agents?

Per job the mean model time is 1,078 s and the mean local build time is 688 s (section 2.2). With B build slots and
A agents, build slots can serve B / 688 jobs per second and agents A / 1,078:

| Mix | Build seconds per job | Jobs per hour the 2 build slots support | Agents needed to keep 2 slots busy |
|---|---|---|---|
| Last 3 days | 688 s | 2 x 3,600 / 688 = **10.5** | 2 x 1,078 / 688 = **3.1** |
| 2026-11-05 only (9 jobs, rough) | 108 s | 2 x 3,600 / 108 = **67** | 2 x 1,078 / 108 = **20** |

At 16 agents the two slots keep up only while build time per job is at most 2 x 1,078 / 16 = **135 s**. This is the
number that decides whether the agents are used. It also excludes the cargo runs inside agent invocations, which
are unmeasured and are the new load that the queue governs (step 1 measures them).

## 7. Build order

Each step lands alone, changes no behavior until its setting is turned on, and states how its effect is measured.
"Measure" means a number that the step itself makes readable, so the next review can compare before and after.
Steps 1 and 2 are measurement only and come first because the estimates above depend on them.

| # | Change | Files and functions touched | How its effect is measured |
|---|---|---|---|
| 1 | **Measure the split in the product.** (a) `athena status` prints the time split of section 2 (model, local build, waiting) and the agent-concurrency histogram of section 2.3 for the last 3 days; (b) the policy hook records `started_at`, `ended_at` and the command of every agent `cargo` call to `jobs/<id>/cargo-runs.json`; (c) `measure-build` records third-party versus workspace bytes of the target; (d) the Hestia sweep logs free memory and free disk. | `crates/athena-core/src/snapshot.rs` and `crates/athena-cli/src/display.rs` (`snapshot_status`), `crates/athena-cli/src/hooks.rs`, `crates/athena-agent/src/policy.rs`, `crates/athena-core/src/disk.rs` (`measure_build`, `BuildMeasurement` in `athena-data`), `crates/athena-core/src/hestia.rs` | Readable result: the cargo share of producer time, L and T, memory per agent. Replaces every **estimate** in sections 5.3 and 6 with a measurement; success is that sections 5.3 and 6.1 can be recomputed with no estimated input |
| 2 | **Say why a slot sat idle.** The drain records, per worker slot, time idle by reason: no independent item in the group, group barrier, held for disk, held for CI. | `crates/athena-cli/src/drain.rs` (`run_parallel`, `run_batches`) | Share of drain time with an idle slot, by reason, against the 99.8 percent at one agent in section 2.3. Decides whether group barriers, not slots, are the limit |
| 3 | **Build gate for Athena's own checks.** Queue and slots of section 4.2, new module, used by `run_local`. Setting `[parallel] max_build_slots` is optional; absent means no gate. Each check records its queue wait. | New `crates/athena-core/src/build_slots.rs`; `crates/athena-core/src/state.rs` (slot and ticket paths beside `try_lock_job`); `routing.rs` (`run_local`); `checks.rs` (`run_check`); `crates/athena-data/src/config.rs` (new key); check record gains `queue_wait_s` | Median and 90th percentile of `queue_wait_s` per check; checks concurrent at any moment never exceed `max_build_slots` (a unit test with a fake clock fails without the gate); total local check time per job unchanged with the gate off |
| 4 | **Build gate for the agent's cargo.** The slot-taking wrapper and PATH entry of section 4.3, exit code 75 on a long wait, wait excluded from the producer deadline. | `crates/athena-agent/src/env.rs` (`build`: `PATH`), `policy.rs`, `hooks.rs`, `crates/athena-cli/src/main.rs` (hidden subcommand), `crates/athena-core/src/run/mod.rs` (the producer deadline) | Agent cargo calls waiting more than 0 s, and their wait, from `cargo-runs.json`; count of `producer-deadline-exceeded-on-small-job` failures per week must not rise |
| 5 | **Lanes.** A slot owns `lanes/<k>`; both sides use it; workspace-crate artifacts and incremental data are removed when a job leaves the slot; a decision records the change to D46. Used only when `max_build_slots` is set. | `crates/athena-core/src/state.rs` (`produce_dir` becomes a lane lookup, new `trim_lane`), `crates/athena-agent/src/env.rs` (`BUILD_DIR_VARS`, `check_build_dirs` accept a lane directory), `disk.rs` (`remove_produce`, `reclaim_interrupted_targets`), `docs/decisions.md` | Lane size after 10 jobs against T from step 1; warm check duration against cold (`verification.json` `duration`); for a sample of 5 jobs, rerun the checks cold and require identical `passed` on every check |
| 6 | **Disk gate by lanes.** The in-flight bound uses `floor((free - reserve - B x L) / S_job)`, with L and S_job from the latest measurement. | `crates/athena-core/src/disk.rs` (`Figures`, `in_flight_limit`, `start_decision`, `start_hold`), `crates/athena-data/src/records.rs` (`BuildMeasurement`) | Unit tests on the formula; jobs held for `environment: low disk` per day before and after (the `LOW_DISK` marker); minimum free disk seen by the Hestia log of step 1 |
| 7 | **Smaller targets.** Add `CARGO_PROFILE_DEV_DEBUG = "line-tables-only"` and `CARGO_INCREMENTAL = "0"` to `[capabilities.change.env]`. The owner approves the edit of this governance file. | `athena.toml` (`[capabilities.change.env]`, and the `study` copy) | T and L from `athena measure-build` before and after (target: at least 30 percent smaller); check durations before and after (no more than 10 percent slower); a failing test still prints a usable backtrace |
| 8 | **Raise the ceiling and ramp.** `MAX_CONCURRENCY_CEILING` 8 to 32 with its test and message; then, as a separate edit the owner approves, `athena.toml` `[parallel]` `max_concurrency` 4, then 8, then 16, with `max_build_slots = 2`. | `crates/athena-data/src/config.rs` (constant, `Config::validate`, the test that uses `MAX_CONCURRENCY_CEILING + 1`); `athena.toml`; a drain-level spending cap (section 6.4a) | At each ramp step, for a day: peak concurrent agents, free memory low point, queue wait 90th percentile, landed jobs per hour against the 3-day baseline (28 landed in 3 days, from Appendix A). Stop rising when the 90th-percentile queue wait exceeds 5 minutes or free memory drops below 2 GB |
| 9 | **Optional: `sccache`.** Only if step 1 shows compile time is most of local build time. | `athena.toml` env (`RUSTC_WRAPPER`), `disk.rs` (`CargoBuild` already sets it for measuring) | Hit rate and local check time per job, from the sccache counts that `measure-build` records |

Order and dependencies: 1 and 2 first and independent. 3 before 4 and 5 (they use its slots). 6 after 5. 7 is
independent of the rest and can land at any point after 1. 8 last, after 3 and 6 are in, so the ceiling is not
raised before the disk and build gates exist. Steps 1 to 3 are small; 4 and 5 are the larger changes.

Not proposed: replacing the plan-group rule in the drain (step 2 decides), and changing landing (it stays one at
a time, `docs/design/parallel-jobs.md` sections 7 and 8).

## Appendix A: the 59 jobs measured

Source files: `records/jobs/2026-11/<job id>/outcome.json`, `invocations/*.json`, `verification.json`,
`baseline.json`. Columns in seconds. `*` marks a job without `totals.wall_time_execution` (wall time is first
invocation start to `at`).

| Job | State | Wall | Model | Local checks |
|---|---|---|---|---|
| 20261103-011406-wm2v | rejected | 1982 | 501 | 1023 |
| 20261103-015346-82n3 | landed | 3842 | 2175 | 970 |
| 20261103-020319-ef9q | rejected | 2266 | 394 | 1199 |
| 20261103-030153-mcr4 | landed | 4599 | 2594 | 1287 |
| 20261103-055218-kj1t | failed | 231 | 231 | 0 |
| 20261103-075009-jzqq * | dropped | 129 | 82 | 0 |
| 20261103-075205-wbtm | failed | 638 | 638 | 0 |
| 20261103-092929-s7ts | rejected | 2459 | 461 | 1287 |
| 20261103-101228-2485 | landed | 3602 | 542 | 2416 |
| 20261103-112515-q0mk | landed | 3646 | 1883 | 1147 |
| 20261103-123447-c3fa | landed | 4099 | 2039 | 1313 |
| 20261103-135304-bwqp | failed | 244 | 244 | 0 |
| 20261103-161454-nmrx | failed | 452 | 452 | 0 |
| 20261103-162416-m9th | failed | 782 | 782 | 0 |
| 20261103-163929-b9pp | timed_out | 5247 | 3928 | 1177 |
| 20261103-181027-bx8y | landed | 3943 | 2094 | 1127 |
| 20261103-193428-df2m | rejected | 2601 | 841 | 1063 |
| 20261103-201526-t4qv | rejected | 1822 | 343 | 903 |
| 20261103-201548-qt18 | rejected | 1975 | 319 | 990 |
| 20261103-202651-j4w5 | landed | 3183 | 1390 | 1022 |
| 20261103-222737-zzsr | rejected | 2717 | 996 | 1057 |
| 20261103-231410-asf2 | landed | 4811 | 3335 | 890 |
| 20261103-231422-8dzf | rejected | 2123 | 485 | 942 |
| 20261103-231510-20z0 | landed | 3637 | 1930 | 1009 |
| 20261103-231808-3gkq | rejected | 1936 | 508 | 872 |
| 20261103-235114-kawe | rejected | 2176 | 456 | 1032 |
| 20261104-002852-g39r | landed | 2353 | 924 | 878 |
| 20261104-054754-b4f5 | landed | 1648 | 203 | 907 |
| 20261104-062715-yv4m | rejected | 1690 | 219 | 948 |
| 20261104-065707-4pvy | landed | 2065 | 455 | 928 |
| 20261104-081830-js1p | rejected | 3105 | 1510 | 908 |
| 20261104-082023-06bz | landed | 5415 | 4887 | 20 |
| 20261104-091422-p3kx | landed | 4760 | 1132 | 1885 |
| 20261104-091432-d10c | landed | 4038 | 2524 | 913 |
| 20261104-102639-h6ep * | dropped | 11960 | 19 | 0 |
| 20261104-103213-f1cw | landed | 4656 | 2956 | 1008 |
| 20261104-105736-w8b9 * | dropped | 7329 | 93 | 0 |
| 20261104-123932-23d3 | rejected | 1769 | 321 | 872 |
| 20261104-124034-ysc4 | landed | 2940 | 2559 | 76 |
| 20261104-140950-gzp9 | failed | 1187 | 212 | 975 |
| 20261104-141009-hsg4 | landed | 2767 | 2121 | 75 |
| 20261104-153450-y72x | rejected | 4449 | 948 | 1914 |
| 20261104-153614-pzfw | landed | 2108 | 440 | 928 |
| 20261104-171104-yb47 | rejected | 1572 | 770 | 22 |
| 20261104-171135-911m | landed | 4550 | 1910 | 1839 |
| 20261104-192155-3b3e | rejected | 1826 | 232 | 937 |
| 20261104-194933-s8v3 | landed | 602 | 105 | 0 |
| 20261104-214954-x6v9 * | dropped | 103 | 32 | 0 |
| 20261104-215118-d9mm * | dropped | 167 | 132 | 0 |
| 20261104-215733-rf7z | landed | 3567 | 2115 | 863 |
| 20261105-001940-nvyd | landed | 1019 | 301 | 0 |
| 20261105-044134-zycx | landed | 1425 | 734 | 64 |
| 20261105-051400-dzr4 | failed | 1607 | 1607 | 0 |
| 20261105-051408-yhss | landed | 965 | 382 | 0 |
| 20261105-061014-gjpt | landed | 920 | 389 | 0 |
| 20261105-070552-kx1y | landed | 3363 | 1970 | 910 |
| 20261105-093606-199q | rejected | 1991 | 901 | 0 |
| 20261105-103552-6s9v | rejected | 787 | 257 | 0 |
| 20261105-160232-t9yq | landed | 1191 | 593 | 0 |

A job whose wall time equals its model time, such as `20261105-051400-dzr4`, has no waiting. Sums may differ from
the section 2.2 totals by rounding to whole seconds.
