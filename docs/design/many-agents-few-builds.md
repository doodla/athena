# Many agents, few builds: agent slots apart from build slots, with shared compiled output

Status: design note, for the owner to decide whether to fund it. Nothing in `crates/`, `xtask/`, `athena.toml`,
`defaults.toml` or `.github/workflows/` changes with this note. It replaces an earlier attempt (job
`20261105-103606-svwx`, verdict fail), which is not in the repository.

Labels: **measured** (read from a record or file, source named), **estimate** (not measured; the basis is
given), **new** (not built), **already present** (built; file named).

## 1. Decision asked

Fund, in order, the steps of section 8. The first three are cheap and measure before they change anything. The
goal: run 16 agents at once (24 after step 1 measures memory) on this machine (4 CPUs, 15 GB memory, about 27 GB
free disk), while only 2 cargo runs execute at once and the disk never fills.

Short answer on value: the records say more agents will not speed up landing yet, because landing wait is the
largest wait by far (section 3). Fund steps 1 to 4 now; fund step 5 onward only if step 1 shows agents queueing
for builds.

## 2. Existing code (what each file already does, and what this builds on)

| File | What it already does | What this design does with it |
|---|---|---|
| `crates/athena-data/src/config.rs` | `MAX_CONCURRENCY_CEILING = 8` (line 241) bounds both `[parallel] max_concurrency` and `[drain.pipeline] max_in_flight`; its comment says each job runs "a model and a build". `PipelineConfig` has `max_in_flight`, `max_check_slots` (default 1, own ceiling), `reserve_disk_gb` (default 10). `athena.toml` sets `max_concurrency = 2`. | Extends: a separate agent-slot setting with its own ceiling (step 2) and a build-slot count. The ceiling of 8 is left as it is (section 7). |
| `crates/athena-core/src/disk.rs` | `df_free_bytes`; `sweep_idle_targets` and `reclaim_interrupted_targets` remove a job's `target/` when no process holds its job lock; `Figures` (T = one cold target, S = one worktree; design estimates `ESTIMATE_T_BYTES` 6 GB and `ESTIMATE_S_BYTES` 0.2 GB); `in_flight_limit` = `floor((free - reserve - max_check_slots x T) / (T + S))`; `start_decision` and `Athena::start_hold` hold a start when the disk fits fewer jobs than `max_in_flight`; `measure_build` measures a real target. Its own comment says `start_hold` is "a gate, not a lock": two jobs starting at the same instant can both pass. | Reuses `Figures`, `in_flight_limit` and `measure_build` unchanged, with the count of lanes in place of jobs (step 5). Does not reuse the gate for build slots: slots need a real lock (section 4). |
| `crates/athena-core/src/run/mod.rs` | Runs the pipeline. Athena's own cargo calls go through `run_bounded` with a timeout (for example the replay build near line 1817 sets `CARGO_TARGET_DIR` and runs `cargo build`); the checks stage runs the configured checks with `CARGO_TARGET_DIR = {job_dir}/target/{side}` (from `athena.toml`, filled in by `crates/athena-agent/src/env.rs`). | Extends: wrap each cargo call of the pipeline in a build-slot hold (step 3) and record the wait. |
| `crates/athena-core/src/state.rs` | `try_lock_job` (the job lock `sweep_idle_targets` relies on); `produce_dir` (`<job dir>/target/produce`); `remove_target`; `remove_produce` (line 272, removes only the producer's build directory and keeps the checks' directory); `other_jobs_in_flight`. | Extends: slot and lane lock files beside the job lock, and a trim at job close modelled on `remove_produce` (steps 3, 5, 6). |
| `docs/design/warm-baseline-build.md` | Measures a shared warm baseline of `main` and does not recommend it now: 44 of 47 recent verified jobs reuse a CI result, so a baseline speeds a path few jobs take; a shared target directory is rejected (Cargo's directory lock, poisoned artifacts); Cargo freshness uses file modification times, so workspace crates rebuild in a fresh copy and only third-party dependencies are reusable. | Builds on it. Step 7 shares only third-party output, never a live target directory, and is conditional on the same kind of measurement the note asks for. |

## 3. Measurements: where job wall time goes

Window: jobs of 2026-11-03 to 2026-11-05 in the evidence snapshot (`records/jobs/2026-11/20261103-*`,
`20261104-*`, `20261105-*`).

| Part of wall time | Number | Source |
|---|---|---|
| Model time (reading + planning + producer + verifier), median per landed job over the last 20 measured | 0.6 + 4.3 + 12.8 + 3.5 = 21.2 min | measured: `athena status --snapshot <snapshot>` throughput line (basis: invocation wall times). Medians do not add exactly; the sum is an approximation. |
| Local build and test (the checks stage), same 20 jobs | 0.6 min median | measured: same command, "checks 0.6" (basis: measured check durations). Low because CI results are reused (next row). |
| Waiting (landing and rebuild, from the verifier's end to the landing record), same 20 jobs | 1007.9 min median | measured: same command, "landing and rebuild 1007.9". This is queue time, not work. |
| CI reuse share of verified jobs | 44 of 47 `verification.json` files in the window have `"reused": true` on at least one check (39 files show three reused checks; 5 docs-only jobs show one); 3 ran every check locally: `20261103-101228-2485`, `20261104-091422-p3kx`, `20261104-153450-y72x` | measured: `records/jobs/2026-11/*/verification.json`, counting `"reused": true` and `"name": "test"` (47 files hold a test check) |
| A job that built locally, in full | `20261104-153450-y72x`: 4449.4 s dispatch to verdict (`outcome.json`, `wall_time_execution`). Checks of the verified tree: fmt 2.7 s, clippy 34.5 s, test 916.0 s = 953.2 s. Baseline run of the same checks on the base commit: 1.97 + 36.07 + 922.25 = 960.3 s (`baseline.json`). Both runs: 1913.6 s = 43 percent of the wall time. The other 57 percent (2535.8 s) holds model time and waits; the split was not read for this job. | measured: `verification.json`, `baseline.json`, `outcome.json` of that job |
| Cargo runs inside the producer's own model time | not measured. The producer runs targeted `cargo test -p` calls through the agent's Bash tool, which no record times. | estimate: unknown. Step 1 measures it. |
| Memory per cargo run and per agent process | not measured | estimate, section 6 |

What the jobs left out of the sample are: of 59 job directories created in the window, 4 have no `outcome.json`
(`20261103-005128-nfvz`, `20261104-215123-fpg0`, `20261105-001814-tw8d`, `20261105-103610-h1cj`; each holds
only a brief, a reading and a request, so they never ran to a verdict or were still in flight when the snapshot
was taken); the other 55 each carry a measured `wall_time_execution`. The status medians also leave out landed
jobs with no measurement.

Reading for the owner: model time is about 21 minutes of work per landed job and runs without the build
machine; local builds are rare (3 of 47) but long (about 16 minutes of checks when they happen); and the
median job then waits over 16 hours to land. The same command shows 8 to 14 landings per day from
`2026-11-01` to `2026-11-05` (12, 8, 12, 14, 10); a lane-bound pipeline (section 7) would carry far more than that. So more agents
only help if the landing queue moves too; that wait is outside this note and is flagged in section 9.

## 4. Slots: three separate things

| Name | What it counts | Held for | Limit (this machine) | Lives in |
|---|---|---|---|---|
| **Agent slot** | model processes (readers, planners, producers, verifiers), each with a worktree (about 0.2 GB, `ESTIMATE_S_BYTES`) and no build directory | the whole job | 16, later 24 | new setting beside `[parallel]` in `config.rs` |
| **Lane** | one build directory pair (`produce` and `verify`) with its worktree | the whole job: from the job's first cargo run until its verdict is written, then trimmed and returned. Never per cargo run. | 2 now, up to 4 with step 7 (estimates, section 6) | `state.rs` lane directories; disk gate in `disk.rs` |
| **Build slot** | cargo runs executing at one moment | one cargo run, from process start to process exit | 2 | slot lock files beside `try_lock_job` in `state.rs` |

The choice, stated once and used in every section: **a job holds its lane for the whole job and a build slot
for one cargo run.** Why a lane is not held per cargo run: a lane's value is the compiled output left by the
previous run, so a per-run hold would either wipe it each time (cold builds, the 10 to 12 GB cost again for
every run) or leave unowned directories behind. Holding it per job also puts the trim at one place, job close,
where `remove_produce` already works, and lets a lane be sized for one job's output.

A job needs no lane until its first cargo run, so a job that only reads and writes files holds an agent slot
and nothing else.

### 4.1 Queue for build slots

- One slot is one lock file `slots/build-<n>.lock` (n from 1 to the build-slot count), taken with a file lock
  that the system drops when the holding process dies. A killed or timed-out cargo run therefore frees its
  slot with no cleanup code.
- Waiters take a ticket, a file `slots/queue/<seconds>-<job>-<n>`, and may take a free slot only when theirs
  is the oldest ticket: first come, first served, so a producer is not starved by later checks.
- Acquire: write the ticket, poll every second for "oldest and a slot free", take the slot, delete the ticket.
  Release: the cargo process exits, the lock drops. The queue wait is written to the job's records
  (`build_wait_s`) so it is a measured number.
- Every cargo run takes a slot: Athena's own checks in `run/mod.rs`, and the producer's and verifier's calls.
  The agents run cargo through their Bash tool, which Athena does not wrap today. The proposal is a `cargo`
  name that Athena itself answers: `crates/athena-agent/src/env.rs` puts a directory first in the agent's
  `PATH` whose `cargo` is the Athena binary (the binary already acts on its own file name for git hooks,
  `crates/athena-cli/src/hooks.rs`), and it takes a slot, then runs the real cargo with `CARGO_BUILD_JOBS` set
  to 4 divided by the build-slot count. This is **new** and its fit with the `PreToolUse` policy
  (`crates/athena-agent/src/policy.rs`) has not been tested; step 4 is where it is checked.
- No deadlock: a build slot never waits for a lane, and a job holding a lane waits only for a slot, which a
  running cargo process always frees. A lane waits only for another job's verdict, which the existing
  producer and verifier time limits bound.

### 4.2 Lane queue

A job asks for a lane at its first cargo run, in the same ticket queue style, and holds it until its verdict is
written. The landing wait of section 3 happens after the lane is returned, so the 1007.9-minute wait does not
occupy a lane. A lane is also given back when the job ends in any other way (failed, rejected, timed out,
interrupted), by extending `reclaim_interrupted_targets` to return the lane of a job whose job lock is free.

## 5. Sharing compiled output: alternatives and the per-job disk figure

Today each job owns `target/produce` and `target/verify` at 10 to 12 GB per target directory (figure from the
request; this note did not measure disk). The code's design estimate for one cold target, `ESTIMATE_T_BYTES`,
is 6 GB; the 10 to 12 GB figure is the one used for the safe bound below.

| Option | How it shares | Disk per lane | Cost and risk |
|---|---|---|---|
| A. Keep a private target per job (today) | nothing | 10 to 12 GB | 17 GB usable disk (27 minus the 10 GB reserve) holds 1 target; 2 only if the reserve is ignored (27 / 12 = 2.25) |
| B. One shared target directory for all jobs | one live directory | one 10 to 12 GB total | Rejected by `warm-baseline-build.md` (directory lock serializes builds, artifacts from different trees poison each other) |
| C. `sccache` for rustc output | compile results by content, in a cache directory | no cut: each target still holds its own copy | Saves CPU time on repeat compiles, which helps build slots, but not disk. `xtask` already installs it for CI. Disk figure unchanged: 10 to 12 GB |
| D. Lanes seeded from one baseline of third-party output by hardlink | one baseline of `main` built from `Cargo.lock`; each lane starts as a hardlink copy of its third-party files, which cost no extra disk; workspace crates rebuild per lane (file modification times, `warm-baseline-build.md` section 3) | shared 6.6 GB once plus 4.4 GB per lane (estimate: third-party output about 60 percent of a 11 GB target, not measured) | Needs a file system that allows hardlinks inside the state directory (not checked); Cargo must replace, never rewrite in place, a shared file (not checked, so step 7 tests it before use) |
| E. D plus smaller dev debug information (an environment setting in `athena.toml`, owner decision) | as D | about 4.2 GB shared plus 2.8 GB per lane (estimate: 35 percent smaller, from general Cargo experience, not measured here) | Debug information is less detailed in test failures |

Expected per-job disk (every figure below is an estimate until step 7 measures it with `measure_build`):

- Job without a lane (most of the time, model work only): 0.2 GB for the worktree, against 10 to 12 GB today.
- Job holding a lane, option D: 4.4 GB private, plus a share of 6.6 GB; against 10 to 12 GB today.
- Job holding a lane, option E: 2.8 GB private, plus a share of 4.2 GB.
- Lanes that fit in 17 GB usable disk: option A, produce side only (6 GB, `ESTIMATE_T_BYTES`): 2 lanes; with the
  verify side built too (11 GB), 1 lane. Option D: (17 - 6.6) / 4.4 = 2.4, so 2 lanes. Option E:
  (17 - 4.2) / 2.8 = 4.6, so 4 lanes.
- The verify side is built locally in about 6 percent of jobs (3 of 47, section 3), so two lanes of 6 GB plus
  one occasional 5 GB verify build fit in 17 GB; the existing `start_decision` gate (with T taken from the
  latest `measure_build`) refuses the start that would not.

Recommendation: A with trimming and the gate first (steps 5 and 6); D then E only if step 1 and step 5 show
producers queueing for lanes. This follows `warm-baseline-build.md`: nothing live is shared, and the reuse of
third-party output is built only when a measurement says compile time matters.

## 6. What this machine can run at once

Inputs: 4 CPUs, 15 GB memory, about 27 GB free disk.

- **Builds at once: 2.**
  - CPU: one cargo run uses all 4 CPUs when left alone. Two runs with `CARGO_BUILD_JOBS` = 2 each use 4 CPUs
    in total. Agents mostly wait for the model service; at an assumed 5 percent of a CPU each (estimate), 16 agents
    use about 0.8 CPU, so builds run slightly slower than the 4-CPU figure (estimate).
  - Memory: an assumed 3 GB peak per cargo test or clippy run at 2 jobs (estimate; step 1 measures it): 2 x 3 = 6 GB.
  - Disk: 17 GB usable holds 2 lanes at 6 GB (produce side), as in section 5.
- **Agents at once: 16, then 24.** Memory: 15 GB total, minus 1.5 GB for the system and Athena (estimate),
  minus 6 GB for 2 builds, leaves 7.5 GB. At an assumed 0.35 GB per agent process (estimate): 7.5 / 0.35 = 21
  agents; with a 25 percent margin, 16. 24 agents fit only if each takes 0.31 GB or less (7.5 / 24), a figure
  step 1 can confirm. Three or four builds are not available: 3 x 3 = 9 GB leaves 4.5 GB, which is 12
  agents, and disk holds at most 2 lanes before option D or E.
- **Does `MAX_CONCURRENCY_CEILING` have to be raised?** Yes for 16 or 24 agents, because the ceiling is 8 and
  bounds both `max_concurrency` and `max_in_flight`. The proposal is not to raise it: its comment ties it to
  "a model and a build" per job, which is the coupling this design removes. Step 2 adds a separate agent-slot
  ceiling (32, a bound chosen here, not measured). `MAX_CONCURRENCY_CEILING` stays at 8 as the bound on
  jobs that hold a lane. It is not changed by this note.

## 7. Throughput of the lanes

A lane is held from the first cargo run to the verdict. Using the section 3 medians, one job holds a lane for
about 12.8 + 0.6 + 3.5 = 16.9 minutes (estimate: the medians do not add exactly, and a job may start cargo late
in its producer time). Two lanes carry about 7 jobs per hour; four carry about 14. The recorded rate is 8 to 14
landings per day, so lanes are not the limit today and become one only when the landing queue drains.

## 8. Build order: small steps, each landable alone

| Step | Change | Files and functions | How its effect is measured |
|---|---|---|---|
| 1 | Record peak memory and elapsed time of each cargo run; record agent-process memory per job | `crates/athena-core/src/run/mod.rs` (the `run_bounded` call sites of the checks stage), `crates/athena-core/src/disk.rs` (`measure_build`, record fields) | After 20 jobs, the median and the maximum of peak memory per cargo run and per agent replace the section 6 estimates; the per-job share of wall time in cargo is read from the new field |
| 2 | Add an agent-slot setting with its own ceiling; leave `MAX_CONCURRENCY_CEILING` at 8 | `crates/athena-data/src/config.rs` (new constant and field beside `Parallel`; validation near line 694), config tests | `cargo test -p athena-data`; a drain with the setting at 16 shows 16 agent processes at once in the process registry |
| 3 | Build-slot lock files and ticket queue; take a slot around each cargo call of the pipeline; write `build_wait_s` | `crates/athena-core/src/state.rs` (slot lock helpers beside `try_lock_job`), `crates/athena-core/src/run/mod.rs` (checks stage and replay build) | Unit test: with 2 slots, a third cargo run starts only after one ends (a counter shows at most 2 running at a time); in the records, median `build_wait_s` and the count of runs that waited |
| 4 | The `cargo` name for agents: take a slot, then run the real cargo with `CARGO_BUILD_JOBS` set | `crates/athena-agent/src/env.rs` (`PATH`), `crates/athena-cli/src/hooks.rs` (name dispatch), check against `crates/athena-agent/src/policy.rs` | Test: an agent's `cargo` call blocks while both slots are held and runs after one frees; peak running cargo count in step 1's field never exceeds 2 across a week |
| 5 | Lanes: a lane directory per job from its first cargo run; the lane count replaces the job count in the disk gate | `crates/athena-core/src/state.rs` (lane directory and lock, `other_jobs_in_flight`), `crates/athena-agent/src/env.rs` (`CARGO_TARGET_DIR` template gains a lane directory; the template in `athena.toml` is a governance file, so the owner edits it), `crates/athena-core/src/disk.rs` (`in_flight_limit`, `start_decision`) | Disk free after each job close over a week stays above the reserve; count of low-disk holds and the recurring low-disk failure class (21 cases in 19 jobs, from the brief's failure-class list) falls toward 0 |
| 6 | Trim at job close and on any end: return the lane, remove the job's output | `crates/athena-core/src/state.rs` (a trim modelled on `remove_produce`), `crates/athena-core/src/disk.rs` (`reclaim_interrupted_targets` returns a dead job's lane) | `measure_build` on the lane directory right after a verdict: 0 bytes of job output; no lane remains held 10 minutes after its job ended |
| 7 | Only if steps 1 and 5 show lane queueing: seed lanes from one third-party baseline by hardlink (option D), then smaller debug information (option E) | `crates/athena-core/src/disk.rs` (a seed function next to `sweep_idle_targets`), `crates/athena-core/src/state.rs` (baseline directory) | `measure_build` bytes per lane against the 10 to 12 GB figure (target: 4.4 GB private under D, 2.8 GB under E); cold-start cargo time of a first run in a seeded lane against an unseeded one; a test that a rebuild in a lane never changes a baseline file |

## 9. Risks and open items

- Landing wait (1007.9 min median) is the largest number in section 3 and is not addressed here. If the owner's
  goal is landings per day, look at it before funding steps 5 to 7.
- Memory and per-run time figures are estimates until step 1 lands; sections 5 to 7 use them.
- The `cargo` name for agents is untested against the tool policy; step 4 may need a policy change.
- Hardlink sharing depends on the file system and on Cargo's write behaviour; both are unchecked.
- Changing the `CARGO_TARGET_DIR` template and any debug-information setting touches `athena.toml`, which the
  owner edits; this note edits neither.
