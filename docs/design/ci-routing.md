# Moving compute to CI with per-job routing

**Status:** proposed; review 1 requested (`docs/design/reviews/ci-routing-review-1.md`), not yet done. Design
only: this document builds nothing and does not mark itself accepted. The review is not recorded, and the
design is not accepted. Build-plan items 2 and 3 were built anyway at the owner's request (section 10); items
4 to 7 are not built.

**Reader:** the owner and the design reviewers. Someone new to the repository should be able to follow it from
this file plus the files it names. Paths are relative to the repository root.

Claims are marked **(read)** when taken from the file named beside them, **(design)** when they are proposals,
and **(given)** when they come from the request. Numbers marked **estimate** were not measured. No command was
run to reproduce any timing.

## 0. Planned first

The brief's addendum of 2026-10-16, "Speed, planning first, and initiatives in parallel" (`BRIEF.md`, last
section), says two things that apply here: "Move compute to ci then, with intelligent routing", and "Get the
planning updates in first, then design" **(read)**. This design therefore went through the planning stage before
the design body was written. The planning notes below are the output of that stage; sections 1 to 10 follow
from them. The planning stage as built is `athena drain`'s plan step (`CHANGELOG.md`, Unreleased, D105)
**(read)**; this document is a planned item, not an implementation of that stage.

Planning notes:

| Item | Note |
|---|---|
| Goal | Cut the median time from dispatch to outcome, without weakening verification. |
| Ladder rung | Speed (the addendum's first bullet); it serves RM-0 in `docs/design/roadmap.md`. |
| Acceptance | A reviewer can find, for every job kind, where each check runs; where a result is reused; what happens when CI is down; and how before and after are measured. |
| Open questions | Settled or carried forward in section 11. |
| Dependencies | Parallel jobs (`docs/design/parallel-jobs.md`) for the shared-cache and cost fit (section 8); native pull-request landing (`docs/design/native-pr-landing.md`) for pushing a commit that CI can see. |
| Touched paths | This design touches only `docs/` and `CHANGELOG.md`. A later build would touch `.github/workflows/ci.yml`, `athena.toml` and Rust code; none of those change here. |
| Parallel group | Independent of the parallel-jobs build, but the two share caches and a cost bound (section 8). |

## 1. The problem and the goal

**Goal:** cut the median time from dispatch to outcome of one job, which the request gives as about 25 minutes,
mostly cold cargo builds repeated for every job.

**Figures (given).** The request states the median is 25 minutes from dispatch to outcome, split as producer
about 12, verifier about 6 and checks about 6 minutes. These are the requester's figures. They were not
re-measured for this design; section 9 says how they will be measured. The three parts do not obviously add up
to 25 (they sum to about 24), which is consistent with rounding and small gaps between phases; that is an
estimate, not a measurement.

**Where the cold builds come from (read).** `athena.toml` gives each job and each side its own build
directory: `CARGO_TARGET_DIR = "{job_dir}/target/{side}"` (`[capabilities.change.env]`). A new job therefore
starts with an empty target directory and compiles the whole dependency tree. Each job runs three checks in
the change capability, `fmt`, `clippy` and `test` (`athena.toml` `[[capabilities.change.checks]]`), and the
producer and verifier are told to run `cargo test --locked` as well. The same commit can thus be built and
tested several times: by the producer, by Athena's checks, by the verifier, and then by `.github/workflows/ci.yml`
after the push (`cargo run --locked -p xtask -- ci`) **(read)**.

**What the design promises.** Fewer repeated cold builds and no repeated test runs of one commit, with the same
checks run and the same refusal behavior. It does not promise a number. The target is set after the baseline
exists (section 9).

## 2. Requirements

| # | Requirement |
|---|---|
| C1 | Each job kind has a stated rule for what runs locally and what runs in CI (section 3). |
| C2 | Caches stay warm across jobs and are invalidated when the inputs change (section 4). |
| C3 | One run of `fmt`, `clippy` and `test` per commit SHA; the verifier reuses it (section 5). |
| C4 | Every check run today still runs somewhere; CI being down or red never produces a success (section 6). |
| C5 | It works with parallel jobs and the planning-first order (section 8). |
| C6 | Wall time per verified result is measured before and after (section 9). |

## 3. Routing: what runs where

### 3.1 What exists today (read)

- CI (`.github/workflows/ci.yml`) runs on `push`, `pull_request`, `workflow_dispatch` and weekly. Its `checks`
  job runs `./setup.sh --ci` then `cargo run --locked -p xtask -- ci`; its `history` job runs
  `athena check --history`. Actions are pinned by SHA and the token is read-only.
- Athena's own checks for the change capability are `fmt`, `clippy` and `test`; `fmt` and `clippy` are skipped
  when a change touches only `docs/` or `*.md`; `test` never skips because `xtask/tests/policy.rs` reads
  tracked files of every kind (`athena.toml` comments).
- The verifier may run only: `cargo` (`build`, `check`, `test`, `clippy`, `fmt`, `run`, `metadata`, `tree`),
  `git` (`status`, `diff`, `log`, `show`) and `ls`; `cargo fmt` only with `--check`
  (`athena.toml`, `verifier_commands`). It has no network tool and no `gh`.

### 3.2 The routing table (design)

"Local" means in the job's directories on the machine that runs Athena. "CI" means GitHub Actions, using the
existing workflow extended by the build job (section 10).

| Job kind | Producer | Athena's checks | Verifier | Rule and reason |
|---|---|---|---|---|
| code | Local, with `cargo check` for fast feedback; it may run `test` once | `fmt`, `clippy`, `test` run in CI on the pushed commit SHA | Reads the CI result for the SHA; reruns a named test locally only to reproduce a specific claim | Code is where cold builds cost the most. Compiling once in CI with a warm cache beats compiling per job. |
| tests | As code | As code | As code | Same cost shape as code; a test change must run the whole suite, which is the CI run. |
| docs | Local; no build | `test` in CI; `fmt` and `clippy` skipped as today | Reads the CI result; reads the diff | `test` must still run, because the boundary scan and policy tests read prose (`athena.toml` comment). Docs changes do not need a producer build. |
| design | As docs | As docs | As docs | A design is prose, so it is a docs-shaped job. |
| config | Local | `fmt`, `clippy`, `test` in CI, never skipped | Reads the CI result | Configuration changes can break policy tests, so the full check runs. |
| build | Local; the producer must compile to check its own work | As code, plus the workflow's own run | As code | A build job changes how CI works. Its own run is the check and cannot be reused from before. |
| study | Local; reads the pinned source | As docs for committed files | As docs | A study produces committed files, so it gets the same gates as a change (`athena.toml`, `[capabilities.study]`). |

**Why not all in CI.** The producer needs a local build to iterate. The routing moves the checks and the
verifier's test run, not the producer's edit loop.

**Why not all local.** Local builds are cold per job today (section 1), and the same commit is built again in CI.

### 3.3 The verifier's allowed commands (design)

The verifier's set stays exactly as it is. It has no `gh` or network access, and widening it would widen the
attack surface of a role that must be independent. So the verifier does not fetch the CI result. **Athena fetches
it** and writes it into the verifier's input (section 5.2) next to the diff. The verifier can still run the
`cargo` and `git` commands above to reproduce any claim it doubts; it is not required to rerun the whole suite.

## 4. Cache strategy

### 4.1 What is cached, and the keys (design)

CI uses the cache action pinned by SHA (the build job selects and pins it). Three layers:

| Layer | Path | Key | Restore keys |
|---|---|---|---|
| Toolchain | `~/.rustup` toolchain directory | `rustup-` plus the hash of `rust-toolchain.toml` (channel `1.97.0`, components `clippy` and `rustfmt`, profile `minimal`) **(read)** | none: a different toolchain is a different key |
| Registry | `~/.cargo/registry` and `~/.cargo/git` | `registry-` plus the hash of `Cargo.lock` | the latest `registry-` entry |
| Target | `target/` of the CI run | `target-` plus the hash of `Cargo.lock` and `rust-toolchain.toml` | the latest `target-` entry for the same toolchain hash |

Notes:

- Keys use content hashes, not branch names or job ids, so jobs on different branches share entries. That is
  the point: today each job has its own empty target (section 1).
- Only the default branch writes the target cache. Pull-request runs restore it and do not save it. This keeps
  an unreviewed branch from filling or poisoning the shared entry (the platform also restricts cache sharing
  between branches; the build job confirms the exact rule).
- Incremental compilation data is not saved, because it grows fast and is rarely reusable across commits
  (**estimate**, from general cargo practice, not measured here). The first build job measures target size.

### 4.2 Invalidation (design)

- **`Cargo.lock` changes.** The registry and target keys change, so the exact entry misses. The restore key
  falls back to the newest older target, so only changed crates rebuild. The newly built entry is saved under
  the new key on the default branch.
- **Toolchain changes** (edit to `rust-toolchain.toml`). All three keys change; the target fallback does not
  cross toolchain hashes, so no stale artifacts from another compiler are restored.
- **Source-only changes.** Keys do not change; the workspace crates recompile, dependencies do not.
- **Suspected corruption.** A manual run with the cache turned off (a `workflow_dispatch` input) is the
  escape hatch; the weekly scheduled run already in `ci.yml` also runs with the normal cache, so it is not that.
  The build job decides whether to make the weekly run cold on purpose.

### 4.3 Bounding cold-start cost (design)

A cold start happens on the first run after a toolchain or lockfile change, after the platform evicts old
entries, or on a fork or new repository. It is bounded three ways:

1. **Restore keys** make a lockfile change a partial rebuild (4.2).
2. **A scheduled warm-up.** The existing weekly run (`ci.yml`, cron `17 6 * * 1`) refreshes the default
   branch entry so eviction after inactivity does not strand the next job on a cold build. The platform's
   eviction rules are an external fact the build job checks.
3. **A time limit.** The CI job has a `timeout-minutes` set from the measured cold build time plus a margin
   (the number is set by the build job after the baseline in section 9; it is not guessed here). A cold run that
   exceeds it fails, which routes to "CI red" in section 6 rather than waiting without bound.

Each CI run records whether the cache hit exactly, by restore key, or missed, and the run's duration. That
record is a data source for section 9.

## 5. One run per commit: avoiding duplicate tests

### 5.1 The rule (design)

For a given commit SHA, `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and
`cargo test --locked` run **once**, in CI, in one workflow run. No other actor reruns the whole set for that SHA
unless the stale-result rules of 5.3 say so.

### 5.2 How the result travels (design)

1. After the producer finishes, Athena commits the candidate on the job's branch and pushes that commit (the
   mechanism is the pull-request path of `docs/design/native-pr-landing.md`; this design adds no new push).
2. CI runs on the commit (`ci.yml` already triggers on `push` and `pull_request`) and its result is identified
   by the full commit SHA and the workflow name.
3. Athena waits for the run, reads the conclusion for that exact SHA, and records it with the run address,
   the SHA, the per-step conclusions and the duration.
4. Athena writes the record into the verifier's input. The verifier reuses it for the `fmt`, `clippy` and
   `test` claims **only if the SHA in the record equals the SHA of the candidate it is verifying**. It states
   in its report that it used the CI result, with the run address.
5. Athena's own check step for those three names reads the same record instead of running them again.

The key is the commit SHA, not the branch, so a force-pushed or amended candidate with a new SHA cannot reuse
an old result.

### 5.3 Stale, missing and unknown results (design)

| Situation | Behavior |
|---|---|
| No run found for the SHA yet | Wait up to a bounded time (set by the build job from section 9 data). After that, treat as missing. |
| Run not finished | Same wait; never read a partial result as success. |
| Record SHA differs from the candidate SHA | Stale. Ignore it; do not use it for any claim. |
| Run cancelled, skipped, timed out or the CI service unavailable | Missing. Falls to the local fallback below. |
| Run red | The check failed. It is a failure, as a failing local check is today (section 6). The fallback does not run to hide it. |
| Run green but for a workflow file changed in the same candidate | The result is not trusted for that run, because the candidate controls the checker; run the local fallback, and the build-kind rule in 3.2 applies. |

**Local fallback.** When a result is missing, Athena runs the three checks locally, once, exactly as today
(`athena.toml` `[[capabilities.change.checks]]`), and records them as local. The verifier then reuses that local
record the same way, keyed by SHA. So a missing CI result costs time, never coverage, and the duplicate rule
still holds: one run per SHA, wherever it ran.

## 6. Verification is not weakened

### 6.1 Mapping: every check still runs somewhere (design)

| Check or step today | Where it runs after | Behavior if that place is down or red |
|---|---|---|
| Athena `fmt` | CI (`xtask ci`) on the SHA; local fallback | Missing: run locally. Red: job fails the check. |
| Athena `clippy` | CI on the SHA; local fallback | Same |
| Athena `test` (includes policy and boundary scans) | CI on the SHA; local fallback | Same |
| Skips for docs-only (`skip_when_only`) | Unchanged rule, applied to the same check list | Unchanged |
| Producer's own `cargo test --locked` before reporting | Local, unchanged | Producer's report is data; Athena reruns |
| Verifier reproducing a specific claim | Local, with its existing command set | Unchanged |
| `athena check --history` | CI `history` job, unchanged | Unchanged |
| Verifier independence (fresh snapshot, sees only the diff) | Unchanged; the CI record is an addition to its input, not a replacement for its judgment | Unchanged |

### 6.2 Failure behavior (design)

- **No empty success.** A job is never marked verified because CI said nothing. Missing means "run locally"; if
  the local run also cannot happen, the job ends `blocked` with the reason named, not `landed` or `verified`.
- **Red is red.** A red CI result is a failed check and ends the job as a failed check does today. It is not
  retried until it turns green. A rerun for a flaky failure is a new, recorded action, and a result that turns
  green on rerun is recorded as flaky with both runs (an estimate of flake rate comes from section 9 data).
- **CI is not the only judge.** The verifier still judges the criteria. CI supplies the check results; the
  verifier decides whether the candidate meets the request.
- **Trust.** The CI record is read by Athena from the platform's API for the exact SHA. Text from a run's log
  is data, not instructions.

## 7. Planning-first order and CI

Routing is decided at planning time from the reading: the job kind and the touched paths (planning notes in
section 0 list both). The plan states, per job, which row of 3.2 applies and whether the job touches
`.github/workflows/`, `rust-toolchain.toml` or `Cargo.lock`, since those change cache keys or the checker. A job
whose plan lacks a kind is not routed; it takes the all-checks path of 3.2's `config` row (the safe default).

## 8. Fit with parallel jobs

(`docs/design/parallel-jobs.md`; this design and that one do not depend on each other's build.)

- **Shared caches.** Parallel jobs on different branches now share cache entries by content key (4.1). That is a
  gain over today, where each job has its own empty target. Writes come only from the default branch, so
  concurrent pull-request runs never race to write one entry.
- **Local build directories.** The producer's local `target/{side}` per job (`athena.toml`) stays, so parallel
  producers cannot corrupt each other's builds. The design does not share a local target directory between jobs.
- **Concurrency limits.** The platform limits concurrent workflow runs per account (**external fact; the build
  job reads the current limit**). The CI workflow gets a `concurrency` group keyed by commit SHA with
  cancel-in-progress off, so a rerun for one SHA queues rather than duplicates, and runs for different SHAs
  proceed. The worker bound `parallel.max_workers` (default 2, ceiling 4, in the parallel design) is also the
  bound on how many commits are waiting for CI at once.
- **Cost limits.** CI minutes are a spend separate from agent spend. The build job adds a cap on CI minutes
  per day, checked before a push, in the same style as the budget checks of the parallel design (per job and
  visible at the start of a run). Moving checks to CI lowers local minutes and raises CI minutes; section 9 reports both.
- **Stale jobs.** The parallel design says most siblings of the first job to land end stale (its section 7).
  A stale job's CI run is wasted. Routing does not change that; it makes the waste visible, because the CI
  minutes per stale job appear in section 9's data.
- **Planning-first.** The plan's parallel groups (section 0, `athena drain`) tell which jobs may be in flight
  together. CI runs for the jobs of one group queue under the concurrency rule above. Jobs that touch the
  same shared append-only files stay serialized by the parallel design's section 9; CI routing adds no new
  collision, because it pushes only the job's own commit.

## 9. Measurement

### 9.1 The measure

**Wall time per verified result:** the elapsed time from a job's dispatch to the record that it was verified
(a `landed` or other verified-passing outcome), for jobs that did reach one. Jobs that fail or are rejected are
reported separately as time per job, so the number is not improved by dropping hard jobs. Phases are recorded
too: producer, CI wait, verifier, Athena checks, landing.

### 9.2 Baseline and after

- **Before.** The requester's median of about 25 minutes (producer ~12, verifier ~6, checks ~6) is **given**,
  not measured. The baseline for this design is recomputed from job records in the state directory (dispatch,
  phase and outcome timestamps) over the last N completed change-capability jobs before the build.
- **After.** The same computation over the first N jobs after the build.
- **Sample size.** N = 20 jobs per side as a starting size (**estimate**: chosen so a median and a spread can be
  read; it is not derived from the variance). If fewer than 20 exist, the report says so and gives the numbers
  with that caveat. Medians and the 90th percentile are reported, not the mean, because a few cold builds skew
  it.
- **Stratify** by job kind from section 3.2, since docs jobs and code jobs differ in cost.

### 9.3 Data source

Job records under the state directory (timestamps per phase; the roadmap's RM-0 item 4 and AD-11 already ask
for wall time per class of work), plus the CI run's start, end, duration, cache outcome (exact, restore key,
miss) and attempt count from the platform's API. The build job adds the missing timestamps; whether the records
already hold them was not checked for this design.

### 9.4 Regression detection

- A cold-start ratio: the share of CI runs that miss the target cache. A sustained rise means keys or
  eviction broke.
- The after-median wall time per verified result, compared with the baseline of 9.2 for each job kind: a
  regression is the after-median exceeding the baseline median, or the 90th percentile rising by more than a
  threshold the build job sets after seeing the baseline spread. The threshold is not fixed here, since a number
  chosen without data would be an estimate with no basis.
- The check runs on each recorded job and in the weekly summary; a regression is recorded as a finding, not
  silently adjusted.
- Strength guard: the number of checks verified per job must not fall. If wall time improves while
  checks-reused-without-SHA-match, or checks-missing counts rise, the improvement is not counted.

This measurement is the outcome check for the design: the change is judged by wall time per verified result with
equal checks, not by the existence of a workflow.

## 10. Build plan

Each item is a separate later job, after review.

1. **Review** (requested; section 12). Nothing below starts before it is recorded. Items 2 and 3 were built
   before it, at the owner's request; the review is still open.
2. **Baseline job.** Compute the 9.2 baseline from the records; add any missing timestamps; no routing change.
   *Built in part:* `cargo run --locked -p xtask -- walltime` computes the median and p90 of dispatch-to-outcome
   time by job kind (`docs/design/ci-routing-baseline.md`). Not done: running it on the real records, and adding
   per-phase timestamps.
3. **Cache job.** Add the three cache layers and exact/restore/miss reporting to `ci.yml`; measure the cold and
   warm build time and the target size; set `timeout-minutes`.
   *Built in part:* the three layers and the report are in `ci.yml`, with the `actions/cache` restore and save
   actions pinned by SHA; the target and the other layers are saved only by non-pull-request runs on the default
   branch. The two jobs keep separate target entries (`target-checks-` and `target-history-`), since they build
   different things. Not done: the cold and warm timing, the target size, and `timeout-minutes`.
4. **Result-reuse job.** The record of 5.2, the SHA check of 5.3, the local fallback, and tests that fail
   without it: a result for a different SHA is ignored; a missing result runs locally; a red result fails; a
   stale record never marks a job verified.
5. **Routing job.** Apply the table of 3.2 from the plan's job kind.
6. **Measurement job.** After N jobs, compute the "after" numbers and record the comparison.
7. **Docs job, alone and last.** README, CHANGELOG and decisions entry, as the parallel design's section 9 says.

## 11. Open questions

1. **Which cache action and what retention rule?** Carried to the cache job; it must read the platform's current
   documented limits.
2. **CI minute budget.** Carried to the routing job; needs the account's plan, which is not in the repository.
3. **Private or stealth repositories.** If a target repository is not on GitHub or has no Actions, the
   routing falls back to all-local. Carried: the doc assumes GitHub Actions because `.github/` exists.
4. **Should the producer also run its tests in CI?** Not in the first build; the producer's local loop stays
   (3.2).
5. **Waiting time for CI.** If waiting for the push-and-run costs more than a warm local test run, routing
   loses. The baseline job must show the CI queue and run time for this repository (not known now).

## 12. Alternatives considered

**A. Keep everything local; only share a build cache (the "do less" option).** Rejected as the sole answer:
a shared local target directory between jobs breaks the per-side separation of `athena.toml` and cannot be
shared safely across parallel jobs. A cache in CI gives the sharing with content keys. It remains a possible
addition for the producer's loop.

**B. Everything in CI, including the producer's builds.** Rejected: it puts a network round trip in every edit
and the producer has no CI access in its command set.

**C. Let the verifier fetch the CI result itself.** Rejected: it needs network and `gh` in the verifier's
commands (3.3), and a role that must be independent should not widen its own tools. Athena fetches and passes it.

**D. Trust any green CI result on the branch, not keyed by SHA.** Rejected: a later amend would reuse a result
for different content, giving an empty success.

**E. Drop the local fallback; wait for CI.** Rejected: CI downtime would block all work. The fallback costs time
only (5.3).

**F. A self-hosted persistent builder machine.** Rejected for now: a new always-on component and new secrets
surface for a problem the hosted cache solves; revisit if measured cold-start cost stays high.

**G. A compiler cache wrapper (sccache) locally.** Not pursued here: it was tried and reverted (RM-0 in
`docs/design/roadmap.md`, D84; it was not measured to work). The CI cache keys do not depend on it.

**H. Skip tests for docs-only jobs.** Rejected: `xtask/tests/policy.rs` reads tracked prose, so a prose change
can fail a test (`athena.toml` comment). The existing skip of `fmt` and `clippy` stays.

## 13. Review

This design was planned through the planning stage first (section 0) and is submitted for independent review:
`docs/design/reviews/ci-routing-review-1.md` records the review request and the questions the reviewer is
asked. The review has not been done; the status line says so. The design is not accepted, and the build plan of
section 10 does not start, until a review verdict is recorded there.

## 14. Out of scope

Implementing any workflow or cache; changing `athena.toml`, `defaults.toml` or Rust code; implementing the
planning stage or the parallel driver; changing existing landing flows.
