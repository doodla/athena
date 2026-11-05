# Warm baseline build for the checks stage

Status: proposal, revised after a rejected first attempt (job 20261029-160024-80qj). Nothing in this note is
built. It answers throughput review 20261029-155931-wfkr, which named the checks stage the largest cost and
predicted 8.0 minutes saved per landed job.

## 1. Existing code: how checks get results today

The review assumed that every check starts from an empty local build directory. That is true only for one of
two paths.

- **Ask CI first.** `crates/athena-core/src/routing.rs` (module comment, lines 10 to 20): every job kind except
  `build` asks for a reused CI result for the candidate SHA, through `ci_result::resolve`, when the
  capability's checks cover `fmt`, `clippy` and `test`. For a GitHub origin the push run on the job's CI branch
  is read every `CI_POLL`, up to the configured bound. `athena.toml` sets `[budget] ci_wait_s = 1500` (25
  minutes). A result for another SHA is ignored. A red result is a failed check; the local run does not hide it.
- **Local checks are the fallback.** When there is no GitHub origin, in a nested run, for a `build` job, when
  the CI run is missing, late or does not cover a configured check, the checks run locally in the verifier's
  fresh snapshot with an empty per-job `CARGO_TARGET_DIR`.
- **CI already has a target cache.** `docs/design/ci-routing.md` section 4.1 keys the CI `target/` cache by the
  hash of `Cargo.lock` and `rust-toolchain.toml`, with a restore key for the latest entry of the same
  toolchain; only the default branch saves it (built per section 10 item 3). This is the same key this note
  would use for a local baseline.

This note changes none of that code. It reads `routing.rs` and `ci-routing.md` as they are.

## 2. Problem

Sources: the review record 20261029-155931-wfkr (14.1 of a 25.4-minute median per landed job in the checks
stage, window of 279 landings, producer 6.8 minutes) and the `verification.json` record of each job
(per check `duration`, and `reused: true` plus a note naming the CI run when the result came from CI). The
`ci_result` record itself and `THROUGHPUT.md` are not in this copy; the figures below are counted by hand
from the verification records, so treat counts as approximate.

| Path | Share of jobs | Checks-stage time | Source |
|---|---|---|---|
| CI reuse | about 85 percent (52 of 61 jobs with a verification record in 2026-11) | the wait for the CI run, not recorded in the reused check's own `duration` (0.0 s, basis "recorded result"). One CI `test` run took 463 s (CI run 41829411253, cited by the verifier of job 20261029-160024-80qj). Total wait: estimate, 8 to 15 minutes including queue and polling. | counted from 2026-11 `verification.json` files; 463 s from the earlier verifier finding |
| Local run | about 15 percent (9 of 61) | measured sample, job 20261101-061448-sdya: fmt 2.4 s, clippy 34.6 s, test 1119.4 s, about 19.3 minutes | that job's `verification.json` |

The 15 percent is a lower bound for older months, when CI reuse was newer, and the sample for the local path is
one job; a proper figure needs the median over all local-run jobs. The 14.1-minute median is a mix of the two
paths. It is dominated by the CI-wait path for most jobs, and the local run is a long tail of about 19
minutes.

A warm local baseline can shorten only the local test and clippy builds. It cannot shorten the wait for a CI run.

## 3. Proposal

Keep one warm build of `main` on the machine that runs Athena, keyed by the hash of `Cargo.lock` and
`rust-toolchain.toml`, refreshed in the background after each landing. A local check run copies it into its
per-job `CARGO_TARGET_DIR` first. Nothing changes for the CI path.

### Freshness: what is actually reused

Stable Cargo decides whether a crate is fresh by comparing file modification times against its dep-info, not by
content hash. A verifier snapshot is a fresh copy of the tree, so every source file has a new mtime. All
workspace crates therefore rebuild; only third-party dependencies (compiled once per lock and toolchain) are
reused. The warm baseline therefore saves the dependency compile, not the workspace compile, and the workspace
(including `athena-core` and its tests, which dominate `test`) is still built in full. The first attempt
overstated this.

## 4. Why this removes the bottleneck (derivation)

- The first estimate was 8.0 minutes saved per landed job, applied to every job. That assumed every job runs
  checks locally from empty.
- Only the local-fallback share, about 15 percent, can gain. Scaled: 8.0 x 0.15 = 1.2 minutes per landed job
  (estimate; the 15 percent share and the 8.0 both are estimates).
- Allowing for mtime freshness, only third-party dependencies are reused, so the saving per local run is lower
  than 8.0. Assuming third-party compile is about half of a cold clippy and test build (estimate, unmeasured),
  per-local-run saving is about 4 minutes, and the whole-pipeline saving is about 4 x 0.15 = 0.6 minutes per
  landed job, against a 14.1-minute stage and 25.4-minute total.
- **Most jobs would not benefit.** About 85 percent of jobs already take the CI path and see no change.
  The warm baseline does not remove the bottleneck.
- **Recommendation.** Do not build the local warm baseline now. Extend CI-result reuse and warm the CI cache
  instead (section 5): shorten the CI run itself (restore the `target-` cache on pull-request branches, check
  the cold-start ratio in `ci-routing.md` section 9), cut the polling delay, and cover more local-fallback
  cases (so a missing or partial CI result becomes a short wait rather than a 19-minute local run).

## 5. Alternatives considered

1. **No change.** Costs nothing; leaves a median checks stage of 14.1 minutes. Reasonable only if review
   shows the stage is not the limit.
2. **Warm the CI path and its target cache (recommended).** The CI path already serves about 85 percent of
   jobs, so any cut to the CI run (cache restore on pull-request runs, which section 4.1 currently does not
   do, a nearer restore key, splitting `test` across jobs) pays on most jobs. It also needs no new local state.
   Risk: CI minutes and cache size; saves depend on cache hit rate.
3. **Extend CI-result reuse.** Cover a configured check CI does not name, or bring the fallback share down
   (late runs, the `ci_wait_s` bound). Directly shrinks the 15 percent. Cheap, but does not speed jobs that
   already wait on CI.
4. **Local warm baseline (this note's proposal).** Helps only the local path; with mtime freshness only
   third-party deps are reused; adds a background refresher and cache invalidation to maintain. About 0.6
   minutes per landed job (estimate).
5. **Shared target directory across jobs.** Parallel jobs would contend for the build lock and cross-job
   pollution breaks per-job isolation; mtimes still invalidate workspace crates. Rejected.
6. **sccache.** Caches compiled units by content, so it can reuse workspace crates despite new mtimes. It
   helps the local path more than a warm baseline, and could be a second step for the CI run, but adds a
   tool and a cache store to trust; worth a separate measurement.

## 6. Risks

- A stale baseline after a toolchain or lock change: the key covers both; a miss means a cold build, as today.
- Disk use: another full `target/` copy per machine.
- Numbers here come from a small hand count; the next review should replace them.

## 7. How the next throughput review measures it

Source review: 20261029-155931-wfkr. The next review takes the median minutes per landed job over the last 20
landings, and reports the checks stage against 14.1 and the whole against 25.4, split by CI result versus local
run (the `source` and duration of each `ci_result` record). The saving is the change in the median of each
split, multiplied by that path's share. A local warm baseline would be judged by the local split only; a CI
cache change by the CI split.
