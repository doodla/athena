# Per-job pull-request landing: remove the shared local landing branch

Status: proposed. Nothing here is built. Nothing in `crates/`, `xtask/`, `athena.toml`, `defaults.toml`,
`.github/workflows/`, `THROUGHPUT.md` or `records/` changes with this note. Written for the Athena maintainers and
for the next throughput review, which scores the prediction in section 6.

Source: throughput review `20261109-055516-tzwd`. It named landing and rebuild as the biggest wait and proposed
removing the local landing branch from the pipeline. Any change that builds this design must cite that review id in
its request text, so the next throughput review can set the minutes it measures against section 6.

Labels used below: **measured** (a figure from `THROUGHPUT.md` or the review request, not re-measured here),
**prediction** (the review's expected saving), **estimate** (reasoned here, not measured), **existing** (built; file
named), **new** (not built).

## 1. Problem

**Measured** (`THROUGHPUT.md`, last 20 landings, as cited in the request of review `20261109-055516-tzwd`): the
median is 2063.6 minutes per landed job. Landing and rebuild takes 1996.9 of them. Reading, planning, producer,
checks and verifier together take about 37 minutes. The stage has grown with each review: 440.1 minutes in the
review that proposed continuous landing (`continuous-landing.md`), 1842.1 in the review behind
`per-job-main-landing.md`, 1996.9 now. These figures are not re-measured in this job.

Almost all of this time is not real waiting for work to happen. Today the path is: verifier, then a fast-forward
onto a shared local landing branch, then a pull request cut from that stack. Every verified job joins the one
landing branch, so:

1. A job's pull request carries the commits of the jobs stacked before it, and cannot merge until those do.
2. Each landing moves the branch, so every job behind it must rebase, and a rebase that touches code is checked
   again.
3. After a landing, the checkout is rebuilt, and the next job in line waits for it.

The queue is therefore serial, and each job's wait is the sum of those before it. That is a reading of the design
notes, an **estimate**; no record separates the rebuild from the rest of the stage (`hestia.md`).

## 2. Existing code

Nothing in this table is created by this note. Each row says what the file contributes and whether the design
reuses or extends it.

| What exists | Where | This design |
| --- | --- | --- |
| `Dispatched`, `dispatched_from`, `refuse_detached`, `plan`: decide where a verified job's commit goes and refuse a detached or moved head. | `crates/athena-core/src/land.rs` | **Extended.** `plan` today targets the shared landing branch. It would plan a single commit on a branch cut from the remote main, one per job (build job 1). |
| `pr_commits`, `classify_commit`: list and classify the commits a pull request carries against the remote base. | `land.rs` | **Reused.** With one job per branch, the list holds one job's commit, and any extra commit is a refusal. |
| `resync_onto_merge`: move the local checkout after a merge. | `land.rs` | **Extended.** The checkout would follow the remote main on its own schedule, not as a step of each landing (build job 3). |
| `PrLink` store and load, `record_bases`, `check_push`, `check_replace`, `check_merged`, `observe_merge`: the per-job link to a pull request, push safety (never non-fast-forward), and the merge check. | `crates/athena-core/src/pr.rs` | **Reused** as they are. They already key on one job id, which is the unit this design makes real. |
| Continuous landing: a waiting job stays in an ordered queue and is polled inside the drain; the rebuild runs once per batch. | `docs/design/continuous-landing.md` | **Reused** for the poll and the batched rebuild. It cuts the wait inside the shared stack; this design removes the stack. The two are independent and compose. |
| Native pull-request landing, with the landing record written after the merge. | `docs/design/native-pr-landing.md` | **Reused.** The landing record still ends the stage, so the measured span is the same one as before. |

## 3. Design

1. **Cut from current main.** After the verifier, the job's commit is applied to a fresh branch cut from the
   remote main (new). No shared local landing branch exists; the fast-forward onto it is removed.
2. **One pull request per job.** The branch is pushed and a pull request opened with the existing `pr.rs` link and
   push checks. The pull request holds one job.
3. **Independent merges.** Jobs merge in any order, each once its own checks are green. A job does not wait for the
   merge of another job unless their files overlap.
4. **Overlap is the only conflict.** If main has moved, the job's commit is rebased onto it (existing rebase rule);
   a conflict holds that one job and no other. A rebase that changes code runs the checks again.
5. **Rebuild decoupled.** The checkout is rebuilt from the remote main when a worker needs it, not as a lock held
   after each landing. A failed rebuild holds new work, not landed pull requests.
6. **Records unchanged.** Landing and intent records are written as today, so `StageMinutes` ends where it does.

## 4. How this removes the bottleneck

The wait in section 1 comes from sharing one branch: a serial queue, repeated rebases and repeated rebuilds. With a
branch per job, a job's landing time depends on its own checks and on real conflicts, not on the queue ahead of it.
Rebases fall to the jobs whose files overlap, and the rebuild leaves the landing path. This is an **estimate**
reasoned from the code and notes; it has not been run.

## 5. Alternatives considered

| Alternative | Why rejected |
| --- | --- |
| Keep the shared branch and make landing continuous (`continuous-landing.md`). | Shortens the wait inside the stack but keeps one serial queue and pull requests that carry other jobs' commits. Kept as a companion, not a substitute. |
| Cut each job's commit from main on a local branch (`per-job-main-landing.md`, `per-job-landing-branches.md`). | Closest to this design. That note does not adopt its own review's predicted saving after checking what the minutes measure; this design adds the removal of the shared branch itself and the decoupled rebuild. It builds on the same cut-from-main step. |
| Squash a chain of jobs in one pull request (`squash-chain-landing.md`). | Keeps jobs coupled: one failed or conflicted job blocks the chain. Stays for adopters who choose squash merges. |
| Merge without a pull request. | Gives up the checks gate and the push safety rules in `pr.rs`; contradicts `native-pr-landing.md`. |

## 6. Prediction and how it is measured

**Prediction** (review `20261109-055516-tzwd`): **5.0 minutes saved per landed job**, against a median of **2063.6
minutes** per landed job when the review ran. The figure is the review's, not re-measured here. It is small beside
1996.9 because the design notes show that most of the stage is not waiting on this path, and the saving counts only
what the shared branch itself adds.

**Measurement.** The next throughput review, after a change that implements this design has landed, recomputes the
median over its last 20 landings with `StageMinutes::median_of` (the same `THROUGHPUT.md` method) and takes the
difference in the median per landed job and in the landing-and-rebuild stage. It sets that difference against 5.0.
A drop within noise of 5.0 supports the design; a smaller one means the shared branch was not the cause. Because
continuous landing may land in the same window, the review should attribute the drop to the phase it came from.

## 7. Build jobs

1. Plan one commit on a branch cut from remote main (`land.rs`), with tests that a second job never shares a branch.
2. Open and push the per-job pull request through the existing `pr.rs` checks.
3. Decouple the checkout rebuild from the landing path.
4. Retire the shared landing branch and its docs.

Each is a separate change; the first cites review `20261109-055516-tzwd`.
