# Warm baseline build for the checks stage: measured, and not recommended now

Status: design note, revised after a rejected first attempt. Nothing in `crates/`, `xtask/` or
`.github/workflows/` changes with this note.

Source: throughput review `20261029-155931-wfkr` (279 landings, median 25.4 minutes per landed job, checks
stage 14.1 minutes, producer 6.8 minutes). The review proposed one warm baseline build of `main` for the checks,
with an expected saving of 8.0 minutes per landed job (request job `20261029-160024-80qj`). This note tests that
proposal against the records and against code that already exists.

Labels used below: **measured** (read from records in the evidence snapshot, source named), **estimate** (not
measured), **new** (not built), **already present** (built; file and step named).

## 1. Existing code

The checks stage already gets its result from CI first and runs locally only as a fallback. Nothing here is
created by this note; the options in section 5 build on it.

| What exists | Where | Status |
|---|---|---|
| Ask for a CI result before running anything locally, for every job kind but `build` | `crates/athena-core/src/routing.rs`: `route` (`Route::reuse`), `run_routed_with_record`, module comment lines 10 to 20 | already present |
| Poll the GitHub push run for the candidate SHA every 20 s up to `ci_wait_s` (1500 in `athena.toml`, so 76 reads) | `routing.rs`: `CI_POLL`, `ci_polls`, `Paced`; `crates/athena-core/src/run/mod.rs` `with_ci_source(.., jc.config.budget.ci_wait_s, ..)` | already present |
| Local fallback when no green result for the SHA arrives (another SHA is ignored, no GitHub origin or a nested run uses `NoCi`) | `routing.rs`: `Fallback`, `run_local`, `NoCi`; a red run fails without a local rerun | already present |
| `build` jobs never reuse a result | `routing.rs`: `route` (`Route::workflow_run`) | already present |
| Target cache restored on every event, pull-request runs included (they restore and do not save; only the default branch saves) | `.github/workflows/ci.yml`: step "Restore the target cache" in the `fmt`, `clippy` and `test` jobs, with no event condition; `docs/design/ci-routing.md` section 4.1 notes | already present |
| Test job split into 6 shards (`xtask test-shard <k> 6`), `fail-fast` off, coverage asserted by `xtask/tests/ci_shards.rs` | `ci.yml` job `test` (`test (shard N)`); `ci-routing.md` section 4.4 (built) | already present |
| Cache outcome (exact, restore key, miss) and run duration reported per CI run | `ci.yml` step "Report the cache outcome"; `ci-routing.md` section 4.3 | already present |
| Known weak spot: the six shards share one `target-test-` key and only shard 0 saves it, so the other shards compile some packages cold | `ci.yml` header comment; `ci-routing.md` section 4.4 "known effects" item 2 | already present (documented, not fixed) |

The CI-first flow was added by job `20261101-061448-sdya` (2026-11-01), which introduced the bounded polling
(the CHANGELOG entry for `ci_wait_s` records it). The review of 2026-10-29 was taken before it.

## 2. Problem: the 14.1 minutes, split by path

### 2.1 Measured durations per path

| Path | Number | Source |
|---|---|---|
| CI-reuse path: CI run duration | median 583 s (9.7 min), range 400 to 690 s, n = 53 | measured: lines `CI run <id> (<n>s)` in `records/jobs/2026-11/*/brief-verifier.md` (53 files in the snapshot; sorted, the 27th value is 583) |
| CI-reuse path: recorded checks time | 0.0 s per reused check | measured: `verification.json` of reused jobs (`"reused": true`, basis "recorded result") |
| Local path, October window | checks stage median 14.1 min | measured: review `20261029-155931-wfkr` (record in `records/throughput-reviews/2026-10/`); about 99 percent of that window ran locally (section 2.2) |
| Local path, November | fmt + clippy + test, median 1107 s (18.5 min), range 953 to 1248 s; test alone median 1070 s (17.8 min); clippy 34 to 39 s; fmt 2 to 10 s; n = 6 | measured: `verification.json` of `20261101-000734-04qa`, `20261101-061448-sdya`, `20261102-102740-qx57`, `20261103-101228-2485`, `20261104-091422-p3kx`, `20261104-153450-y72x` (basis "athena monotonic clock") |

Cautions. The CI run duration is the time a job can wait; the recorded check time of a reused result is 0.0 s,
so the checks stage of a CI-reuse job does not show that wait. Which stage the polling time lands in
(verifier or landing) is not measured here. The six November local runs are all `change` jobs, which run all
three checks, while the October 14.1 median also contains docs-only jobs that skip fmt and clippy; the
difference from 14.1 to 18.5 is probably that mix (estimate, not tested). Eight November jobs ran no CI
reuse; two of those eight were not read.

### 2.2 Local-run share: the review window versus November

Counted from `verification.json` files in the snapshot (a file counts as reused when any check has
`"reused": true`). These files include jobs that did not land, so the shares are of verified jobs, not of the
279 landings.

| Window | Verification records | Reused CI | Ran checks locally |
|---|---|---|---|
| Review window: 2026-10-10 up to the review (2026-10-29 15:59) | 393 | 4 (`20261010-154013-a9gz`, `20261010-225741-yvk0`, `20261016-005805-p806`, `20261027-054251-33v7`) | 389, about 99 percent |
| 2026-10-29 16:00 to 2026-10-31 | 41 | 12 | 29, about 71 percent |
| November (2026-11-01 to 2026-11-05) | 59 | 51 | 8, about 14 percent |

Counts: 434 October files (393 + 41) and 59 November files in `records/jobs/`. Why those 4 early jobs reused CI was not
checked.

### 2.3 The claim "the 14.1 minutes is mostly CI wait" is wrong

The first attempt of this note said the median "is dominated by the CI-wait path". The records say the
opposite. The review window ran its checks locally for about 99 percent of verified jobs, so the 14.1-minute
median is the local path. The CI-reuse path was not in that window in any real quantity; it came with the
bounded polling of job `20261101-061448-sdya`, and by November it carried about 86 percent of jobs. So the CI
reuse change, and not a warm baseline, has already moved the stage away from 14.1.

Consequence for the next throughput review: its checks-stage median over the last 20 landings will mix about
86 percent of jobs whose recorded check time is 0.0 s with a minority at roughly 15 to 20 minutes. A comparison
with 14.1 is only valid when split by CI-reuse versus local, section 7.

## 3. Freshness of a reused build (corrected)

The review assumed a warm baseline keeps everything built. On stable Cargo that is not so. Cargo decides
freshness from file modification times (and fingerprints that include them) of the package sources. A verifier
snapshot is a fresh copy of the tree, so every file has a new mtime: all workspace crates rebuild. Only
third-party dependencies, whose sources come from the registry and are keyed by `Cargo.lock` and the
toolchain, are reused. So the saving of a warm baseline is the compile time of third-party dependencies, not
of the whole build. Content-based freshness is a nightly Cargo option and is not available on the pinned
stable toolchain (`rust-toolchain.toml`, read only as a cache key input).

## 4. What the 8.0 minutes becomes

The 8.0-minute estimate in the review applied to a window that was about 99 percent local. Scaled to the
November local share it is much smaller:

- Upper bound, taking 8.0 minutes per local run at face value: 8.0 x 0.14 = **1.1 minutes per landed job**
  (estimate; 0.14 is 8 of 59 from section 2.2).
- With the freshness correction of section 3, assuming about half of the 8.0 minutes survives (about 4
  minutes per local run, estimate, not measured): 4 x 0.14 = **about 0.6 minutes per landed job**.
- Stated plainly: **most jobs would not benefit.** About 86 percent of jobs never run the build locally, so a
  warm baseline of `main` speeds up a path they do not take. The cost is real: a background refresh job, a
  cache key, disk, and a freshness rule to maintain.

## 5. Alternatives considered

| Option | Reason |
|---|---|
| No change | Costs nothing. The stage has already moved with CI reuse (section 2.3). Leaves 14 percent of jobs at about 18 minutes of local checks and the CI path at about 10 minutes. Chosen as the default until section 6 item 2 is answered. |
| Warm local baseline (the review's proposal) | Helps only the local path (about 14 percent now) and only third-party compile time (section 3). Saving about 0.6 to 1.1 minutes per landed job (estimate). Adds a refresh process and a staleness risk. Not worth building now; it was right for the October mix and is wrong for the November mix. |
| Warm or extend the CI-result path (more jobs reuse CI) | Directly attacks the cause of local runs: the 8 of 59 jobs that fell back. Cheapest lever is knowing why they fell back (not ready within `ci_wait_s`, `build` kind, red run, no origin); the records do not say. A new fix targets that reason, not the cache. |
| Warm or extend the CI target cache | The restore on pull-request runs and the 6-way test split exist (section 1); re-proposing them would be redundant. What is left is the per-shard key: only shard 0 saves, so other shards build some packages cold. Might cut the 583 s median run, but the size of the cold share is not measured (estimate unknown). |
| Shared target directory across jobs | Parallel jobs would contend for Cargo's directory lock and could poison each other's artifacts; it also keeps the mtime problem of section 3. Rejected. |
| `sccache` | Caches compilation units by content, so it would survive the fresh mtimes and help workspace crates too. Needs a cache backend and wrapper configuration and does nothing for linking or test run time, and test time (about 18 minutes locally) is far larger than clippy (34 s), which suggests compile is not the bulk. Not measured; revisit only if section 6 item 3 shows compile dominates. |

## 6. Recommendation

1. **Restore target cache on pull-request runs; split `test` across jobs.** Already present. No action; not
   re-proposed.
2. **Record why a job ran checks locally** (new). Add the reason (no green result within `ci_wait_s`, `build`
   kind, no origin, red run) to the check record. The 14 percent local share is the measured cause of the
   remaining local time, and today it can be counted but not explained. This is a small change in
   `routing.rs` and the record schema, to be done as its own job.
3. **Split local test time into build and run** (new). Measure, for the local fallbacks, how much of the roughly
   18-minute test check is compile and how much is test execution. The 34 s clippy time suggests compile may
   be the smaller part. This decides whether any build cache (warm baseline, `sccache`) can matter.
4. **Do not build the warm local baseline now** (new, the decision of this note). Reopen it only if the local
   share is above about 25 percent after item 2, and item 3 shows compile dominates (threshold is an estimate
   from 8.0 x share needing to exceed roughly 2 minutes per landed job).
5. **Per-shard cache key for the CI test job** (new, optional). Try only if item 2 shows jobs waiting out
   `ci_wait_s` because the CI run is slow; measure the run duration before and after.

## 7. How the next throughput review measures this

The next review takes the median minutes per landed job over the last 20 landings and reports the checks stage
against 14.1, and the whole against 25.4 (review `20261029-155931-wfkr`). It must split the 20 landings by CI
reuse versus local run (the `"reused"` field of `verification.json`), and report for each group the checks
median, the group's size, and the CI wait, since the reused group's recorded check time is 0.0 s. A drop in the
combined median is then credited to CI reuse (already built), and only a drop in the local group's median
counts for any later change to the local path. If item 2 is done, the local group can be split further by
fallback reason.
