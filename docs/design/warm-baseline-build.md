# Warm baseline build of main

Status: proposed. This note designs the change; it does not implement it.

Source: throughput review 20261029-155931-wfkr. A change that implements this design must cite that review
in its request text, so the next throughput review can set its measured minutes against the prediction below.

## Problem

The median landed job took 25.4 minutes over the last 20 (measured, THROUGHPUT.md, as quoted by the review).
The checks stage (fmt, clippy, test) took 14.1 of them. The producer took 6.8, the verifier 2.5. Checks are
the largest single cost, more than twice the producer's.

The review read THROUGHPUT.md and the design documents and timed nothing itself. Its cause is this: checks run
in the verifier's fresh snapshot with an empty per-job, per-side `CARGO_TARGET_DIR`, so each run compiles
every dependency and the whole workspace from scratch before any check can report. The share of the 14.1
minutes spent compiling rather than running tests is an estimate, not a measurement; the first step of the
implementation records it (see Measurement).

## Design

Keep one warm baseline build of main and let every check run start from a copy of it.

1. **Key.** `key = hash(rust-toolchain.toml bytes, Cargo.lock bytes)`. Both files are only read; this change
   does not edit either. A new toolchain or a changed lockfile gives a new key and so a cold baseline.
2. **Baseline directory.** One directory per key under Athena's state area, holding a Cargo target directory
   built from main for the three check commands' profiles: `cargo fmt` needs none, `cargo clippy
   --all-targets --locked` and `cargo test --locked` need the check and dev profiles.
3. **Background refresh.** After a job lands, and when the key of main differs from the newest baseline, a
   background task builds main into a fresh directory (`cargo build --all-targets --locked` plus a clippy
   pass), then renames it into place atomically. Readers never see a half-built baseline. Refresh is
   best-effort: failure or a full disk leaves the previous baseline, or none, in place. It obeys the disk
   budget used for other builds, and old keys are deleted, keeping at most two.
4. **Checks start from it.** When the verifier's snapshot has the same key as a baseline, the check run copies
   the baseline into its own per-job, per-side `CARGO_TARGET_DIR` (reflink or hard-link copy where the
   filesystem allows, plain copy otherwise) before running fmt, clippy and test. Cargo's own fingerprints then
   rebuild only the crates whose sources changed. If no baseline matches the key, the run proceeds from an
   empty directory exactly as today.
5. **Isolation.** The baseline is never written by a check run. Each job still has its own target directory,
   so concurrent jobs and the two sides (base and candidate) cannot corrupt each other or the baseline.

## Why this removes the bottleneck

A typical job changes a few files in one or two crates. With a warm target directory Cargo recompiles those
crates and their dependents but not the third-party dependencies or untouched crates. Compilation then stops
being the dominant part of the 14.1 minutes, leaving test execution and the incremental rebuild. The refresh
runs while the pipeline would otherwise be idle or waiting on other stages, so its cost is off the critical
path of a landed job.

## Predicted saving

**8.0 minutes per landed job** (the review's prediction), taking the median from 25.4 to about 17.4 minutes
and the checks stage from 14.1 to about 6.1. Only the 25.4 and 14.1 are measured; the 8.0 is an estimate. It
assumes about 57 percent of the checks stage is avoidable compilation (estimate), and an incremental rebuild
and copy cost that leaves about 6 minutes.

## How the next throughput review measures it

The review that follows the landing of this change reads THROUGHPUT.md as before: the median minutes per
landed job over the last 20 landed jobs, and the checks-stage median in the same window. It sets them against
this prediction:

- Baseline: 25.4 minutes median per landed job; checks 14.1 (producer 6.8, verifier 2.5).
- Predicted: 17.4 and 6.1, a saving of 8.0 minutes per landed job.
- Measured saving = 25.4 minus the new median per landed job; the checks-stage saving = 14.1 minus the new
  checks median. A saving well under 8.0 means the prediction was wrong, and the review should say which part
  (cold baselines, copy cost, test time) explains the gap.

Since the last 20 jobs may mix jobs from before and after the change, the review should count only jobs that
started after it landed, or wait until 20 have. It should also report the share of check runs that started
from a warm baseline, which the implementation must record per run (hit, miss and the key). Without that
count, a low saving cannot be told apart from a low hit rate. The implementation also records the checks
stage's compile time and test time separately, replacing the estimate above with a measurement.

## Alternatives considered

- **No change.** Zero risk, zero saving. Rejected: the checks stage is the largest cost and the cause is
  specific and fixable.
- **One shared target directory for all jobs.** Simple and always warm, but concurrent jobs and the base and
  candidate sides race on Cargo's lock and write different artifacts into one place, so a failed or killed job
  can leave state that affects another job's verdict. Rejected: it trades speed for verification
  correctness.
- **sccache or another compiler cache.** Isolation stays per job, and it caches per compilation unit, but it
  adds an external tool and a cache service, does not cache linking or proc-macro and build-script work well,
  and still pays for Cargo's fresh-fingerprint work. It remains a later option if hit rates on the
  copy-from-baseline design prove poor. Not chosen now (judgement, not measured).
- **Baseline copy keyed on the hash only versus per-commit baselines.** Per-commit baselines would be
  warmer but cost a build per landed job and much disk. Keying on the toolchain and lockfile and letting Cargo
  rebuild changed crates is the cheaper middle.

## Risks and review

- **Stale baseline.** A baseline built from an older main differs from the snapshot. Cargo's fingerprints
  compare source mtimes and content hashes, so differing crates are rebuilt; the key guards the two inputs
  Cargo fingerprints do not cover well across files, the toolchain and the lockfile. Residual risk: mtime
  quirks after the copy. Mitigation: the check run only reuses the baseline when the keys match, and a
  verification-only escape hatch (`ATHENA_COLD_CHECKS`, name provisional) forces an empty directory.
- **Correctness of verification from a warm build.** A warm build must not turn a failing candidate into a
  passing one. Cargo treats the copy as an ordinary incremental target directory, which is the normal
  developer workflow, and the final tests run in the candidate's own copy. To be sure, the implementation
  should compare, over a sample of jobs, verdicts from warm and cold runs; any disagreement stops the rollout.
  The landing path may still run one cold check where the owner wants it.
- **Disk use.** A built workspace is large (size not measured here). Two baselines plus per-job copies
  multiply it; the copy is a reflink or hard link where possible, and the refresh obeys the disk budget so it
  cannot starve a producer. Recent jobs have failed on low disk, so this must be tested, not assumed.
- **Hard links and mutation.** A hard-link copy shares files with the baseline. Cargo replaces rather than
  edits artifacts, but this must be confirmed by a test that checks the baseline is unchanged after a run;
  if it is not, use reflink or plain copy.
- **Refresh contention.** A background build competes for CPU with live jobs. Run it at low priority and skip
  it when jobs are running.
- **Cold start and key churn.** The first run after a toolchain or lockfile change is cold, as today.
  Dependency updates are infrequent, so most runs should hit (estimate).

## Out of scope

Implementation in crates/ or xtask/, and any change to rust-toolchain.toml, athena.toml, defaults.toml or the
workflows. Those belong to a later job citing review 20261029-155931-wfkr.
