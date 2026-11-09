# Early per-job pull requests and no local staging branch

Status: proposed. Nothing here is built. Nothing in `crates/`, `xtask/`, `athena.toml`, `defaults.toml`,
`.github/workflows/` or `records/` changes with this note. Written for the Athena maintainers and for the next
throughput review, which scores the prediction in section 5.

Source: throughput review `20261109-201520-z1xg`. It read `THROUGHPUT.md`, which gives the landing and rebuild stage
as 1811.5 of the 1859.9-minute median per landed job (measured, as the review reported it). This note was not able to
re-measure those figures; it read only the repository copy and ran nothing.

Labels: **measured** (a figure taken from `THROUGHPUT.md` or a named design note), **estimate** (reasoned here),
**prediction** (the saving in section 5), **existing** (built), **new** (not built).

Term: the **staging branch** is the long-lived local git branch on which each job's commit sits until its pull request
lands.

## 1. Problem

Each job's commit is added to the one staging branch. Each later pull request is cut from that branch, so it carries
the earlier jobs' commits again. The job's commit is then listed in many landing records, and the landing and rebuild
stage, which ends at the newest record that lists the job, keeps running.
`docs/design/per-job-main-landing.md` section 1 showed on the records that most of the headline figure is this repeated
listing and not a wait a job really experienced. It also named the real costs of the stack: replay of the stack in
each pull request, conflicts between jobs that have nothing to do with each other, and a resync of the branch after a
squash.

A second delay sits upstream of landing. The pull request is opened only after the verifier ends, so CI on the
change starts late even though the producer's output no longer changes unless the verifier sends it back.

## 2. Design (new)

1. **Open the pull request when the producer ends.** As soon as the producer's report is accepted, Athena builds the
   job's single landing commit and opens its pull request. CI starts while the checks and the verifier run. The
   verifier runs against the same commit.
2. **Build the commit on current main, with no staging branch.** Athena builds the commit on the host's current
   `main` with `merge_tree` and `commit_tree` (library calls, no working-tree checkout), using the job's candidate
   tree. The parent is the `main` tip read at that moment. The commit goes on a per-job branch named for the job, and
   that branch is the pull request's head. No other job's commit is in it.
3. **Drop the staging branch.** Nothing is appended to a shared local branch. The per-job branch is deleted when its
   pull request lands or is closed. There is no resync after a squash, because no branch holds the squashed commits.
4. **A verifier that fails the job closes the pull request.** If the verifier sends the work back, Athena either
   rebuilds the commit from the new candidate and force-pushes the per-job branch (same pull request) or, on a final
   failure, closes the pull request and deletes the branch. A pull request is merged only after the verifier passes:
   the merge step, not the opening, waits for the verifier.
5. **A moved `main` is handled by the existing landing step.** If `main` moved before the merge, Athena rebuilds the
   commit on the new tip with `merge_tree`. A conflict holds the job with the conflicting paths named, as
   `per-job-main-landing.md` section 6 describes.
6. **The record.** The landing record lists the job's commit once, in the one pull request that carried it. The
   measurement effect of repeated listings ends.

## 3. Existing code: kept, changed, superseded

- `docs/design/per-job-main-landing.md` (proposed). **Extended.** Its core is kept: one job, one commit, one pull
  request, cut from current main with `merge_tree` and `commit_tree`, and its premise check that the headline stage is
  mostly a measurement effect. This note adds the one thing it does not decide: the pull request opens at producer end,
  and merge waits for the verifier.
- `docs/design/per-job-landing-branches.md` (proposed). **Overlaps and is narrowed.** Its per-job branch is kept as the
  pull request head. Where it still keeps a local branch as the base for the branches, this note drops that branch.
- `docs/design/native-pr-landing.md`. **Kept.** The native pull-request path (opening, checking and merging a pull
  request, including for repositories Athena does not own) is the path used. This note changes only when the pull request
  opens and what its head holds, not how CI results or merges are read.
- `docs/design/continuous-landing.md`. **Superseded in part.** It lands inside the drain from the staging branch. The
  drain trigger stays; the stack it lands from does not.
- `docs/design/async-drain.md`, `docs/design/agent-and-build-slots.md`, `docs/design/warm-baseline-build.md` and
  `docs/design/ci-routing-baseline.md`. **Kept.** Slots still bound parallel builds, the warm baseline build is still
  the starting point for each job's CI, and CI routing still decides which checks a pull request needs. Early pull
  requests raise the number of CI runs in flight (section 6), so the slot limit for CI-bound work is the one thing
  to re-check when this is built.

## 4. How it removes the bottleneck

`THROUGHPUT.md` (measured, as read by review `20261109-201520-z1xg`) gives 1811.5 of the 1859.9-minute median as
landing and rebuild. Two mechanisms act on it:

- **Repeated listing ends.** A job's commit is in exactly one pull request, so the newest record that lists it is its
  only one. This removes the largest part of the 1811.5 minutes, but that part is a measurement effect
  (`per-job-main-landing.md` section 1), so it changes the headline number without shortening any job's real wait.
- **CI overlaps with the verifier.** The pull request opens at producer end, so the CI run and the checks and verifier
  stages run side by side. The real saving is the time CI used to start after the verifier, up to the verifier's own
  duration at best.

The other five stages (the remaining 48.4 minutes of 1859.9, measured) are not changed.

## 5. Predicted saving and how the next review measures it

**Prediction (not measured): 7.0 minutes saved per landed job**, against the 1859.9-minute median when review
`20261109-201520-z1xg` ran. This is the review's own prediction, adopted here. It is a small share of the 1811.5-minute
stage because the rest is largely the listing effect, which this note does not claim as a real saving. The prediction is
consistent with the verifier-and-checks overlap: the verifier stage in `per-job-main-landing.md` is a few minutes
(sample), so 7.0 minutes is about the most one overlap can give.

**How the next throughput review measures it.** After a change implementing this lands, the next review recomputes
from `THROUGHPUT.md` the median minutes per landed job over its window, using only jobs that landed after the change,
and reads:

1. the median total (compare with 1859.9, less 7.0 as the predicted figure);
2. the landing and rebuild median (compare with 1811.5);
3. the minutes from producer end to the pull request's `created_at` (should be near zero) and from the verifier end to
   the merge (should be below the pre-change figure).

The review reports measured minus predicted. Because the headline stage also drops from the end of repeated listings,
the review should state separately the part of the change from the first-landing stage (the real wait: verifier end to
the first landing record) and the part from the number of records listing each job. A fall in the headline with no fall
in the first-landing stage is the listing effect, not this prediction.

## 6. Alternatives considered

- **Keep the staging branch and open the pull request early from it.** Rejected: the pull request would carry every
  earlier job's commit and be listed many times, and a failed job's commit would sit in the stack.
- **Open the pull request after the verifier, as now, but from current main (per-job-main-landing only).** Kept as the
  fallback. It removes the listing effect but gives no CI overlap.
- **Open the pull request at reading or planning time.** Rejected: there is no commit to build, and CI would run on
  an empty change.
- **Merge when the producer ends, without waiting for the verifier.** Rejected: the verifier is the gate for work that
  reaches `main`.
- **Batch several jobs into one pull request.** Rejected: one failing job holds the rest, and it recreates the stack.

## 7. Risks

- **Wasted CI.** A job the verifier rejects has already used a CI run. Estimate: the cost is bounded by the
  verifier-reject share of jobs; the next review reads that share.
- **More pull requests open at once.** The number of open pull requests grows by the jobs in the verifier stage.
  Check the CI concurrency limit and the slots in `agent-and-build-slots.md`.
- **Force-push on a rebuilt commit.** CI restarts on each rebuild. Rebuilds only follow a verifier send-back or a
  moved `main`.
- **A stale `main` parent.** The parent is the tip when the producer ended. The merge step rebuilds if `main` moved.

## 8. Out of scope

Implementing any of this in `crates/` or `xtask/`; editing `athena.toml`, `defaults.toml` or workflows; changing the
other design notes beyond cross-links.

## 9. Citation duty

A change that implements this design must cite throughput review `20261109-201520-z1xg` in its request text. The next
throughput review then sets the minutes it measured after that change landed against the 7.0-minute prediction in
section 5.
