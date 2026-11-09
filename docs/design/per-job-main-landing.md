# Per-job landing from current main: remove the landing wait

Status: proposed. Nothing here is built. Nothing in `crates/`, `xtask/`, `athena.toml`, `defaults.toml` or
`.github/workflows/` changes with this note. Written for the Athena maintainers and for the next throughput review,
which scores the prediction in section 5.

Source: throughput review `20261109-014617-5cg0`. Figures marked **quoted** come from the review's request text and
`THROUGHPUT.md`; neither file is in the working copy of this job, so they were not re-measured here. **Estimate**
means reasoned here, not measured.

## 1. Relation to earlier notes

`docs/design/per-job-landing-branches.md` already proposes the same shape (one job, one commit, one pull request, cut
from current main) for the earlier review `20261108-183648-5eya`, which quoted 1537.1 of 1582.1 median minutes. This
note does not repeat its build plan. It does three things:

- It restates the mechanism so a reader of this review needs only this file (section 3).
- It re-bases the prediction on the newer figures and gives the next review its own scoring rule (section 5).
- The citation duty moves to review `20261109-014617-5cg0`. Where the two notes disagree on figures, this note governs
  for scoring. The earlier note's build jobs (measure, cut, use the cut in `land_pr`, remove the stack replay) stay
  the build order; no build is started by either note.

The landing wait grew between the two reviews (1537.1 to 1842.1 minutes quoted), which is what a growing stack would
produce. That is consistent with the stacking hypothesis but does not prove it.

## 2. Problem

**Quoted:** landing and rebuild is 1842.1 of the 1903.5 median minutes per landed job. The other five stages add up to
about 38 minutes (0.4 + 11.0 + 22.5 + 1.0 + 3.3, which sums to 38.2). The stage is defined in
`crates/athena-core/src/status/throughput.rs` (`StageMinutes::landing_and_rebuild`): from the end of the job's last
invocation (the verifier) to the later of the landing outcome and the landing record.

Likely cause (hypothesis): landing commits pile up on one long-lived local branch, so each job's pull request carries
every unlanded commit under it, not only its own. Each landing then pays for replaying the stack (`plan_rebase`),
conflicts in commits it did not write, and waiting for earlier commits to reach main.

## 3. Mechanism

One job, one commit, one pull request, cut from current main.

1. The job's checks and verifier pass.
2. Athena fetches the host's main.
3. It builds the job's single landing commit directly on that main with a three-way merge: base is the job's recorded
   base commit, ours is the fetched main, theirs is the job's verified content plus its closing records. The tools are
   `Git::merge_tree` and `Git::commit_tree` with one parent.
4. The commit is pushed as the job's own branch and one pull request opens from it. The pull request holds one commit.
5. After the merge the local branch moves to the merged commit. It is no longer a staging area for landings.

Why this removes the wait: the commit has no other job beneath it, so there is no stack to replay, no pull request
that re-carries earlier jobs, no conflict inherited from a commit the job did not write, and no wait for earlier
commits to reach main before this job's diff is the right size. The cost of a landing becomes one fetch, one merge and
one push, whatever the number of unlanded jobs.

It does not shorten a CI run, and it does not remove time behind the landing queue (`continuous-landing.md`).

## 4. Existing code

Nothing in this table is created by this note.

| What exists | This design |
| --- | --- |
| `docs/design/per-job-landing-branches.md`: the same cut, its build jobs and risks | **Extended**: kept as the build plan; this note adds the new figures and scoring rule. |
| `docs/design/squash-chain-landing.md`: one squash commit adds the records of several chained jobs | **Kept, not used**: a pull request holds one job, so the single-job gate rule applies and no chain check runs. |
| `docs/design/native-pr-landing.md`: `land_pr`, the merge gate, the head check, the resync after a squash merge | **Kept** for `land_pr` and the gate (the head check now passes in the normal case). **Changed**: the resync after a merge becomes a move to the merged commit. |
| `crates/athena-core/src/land.rs`: `plan`, `resync_tip`, `pr_commits`, `classify_commit` | **Kept**: `pr_commits` and `classify_commit` show the commits in a pull request and let the build check it holds one. **Changed**: `resync_tip` moves the branch to the merged commit instead of replaying a stack. |
| `crates/athena-core/src/gh/landing.rs`: `plan_rebase`, `rebase_job_branch` | **Changed**: the stack replay is replaced by the cut and later removed. |
| `crates/athena-core/src/status/throughput.rs`: stage definition and window | **Kept unchanged**, so the next review compares like with like. |

## 5. Predicted saving and how it is measured

**Prediction** (quoted from review `20261109-014617-5cg0`, an estimate): **1830.0 minutes saved per landed job**. The
median was **1903.5** minutes per landed job when the review ran.

Implied remainder: 1903.5 - 1830.0 = 73.5 minutes (estimate), about 38 for the other stages and about 35 for the
landing and rebuild stage, a few CI windows (the earlier note puts a median CI run near 10 minutes). It is reachable
only if almost all of the 1842.1 minutes comes from stacking.

**How the next throughput review measures it.** It uses the same function as this review: the median over the newest
20 landings (`THROUGHPUT_WINDOW`, `StageMinutes::median_of` in `crates/athena-core/src/status/throughput.rs`). It
reports:

- the total median minutes per landed job, compared with 1903.5;
- the **landing and rebuild** stage (`landing_and_rebuild`), compared with 1842.1;
- the saving as 1903.5 minus the new total, set against 1830.0.

It counts only landings made after the implementing change landed, so the whole window must be post-change. It also
reports the number of commits in each landed pull request: one commit everywhere shows the design was in force. If the
stage falls less than predicted and the count is one, the remainder was not stacking, and the review says so. A drop
caused by another change (for example `continuous-landing.md`) is credited to it by phase, not to this change.

## 6. Alternatives considered

| Option | Why rejected |
| --- | --- |
| Keep stacking, squash on merge | The squash collapses the stack on main, not in the pull request, and breaks ancestry so a resync is needed. The cause stays. |
| Rebase the stack more often | `plan_rebase` already does it at each landing; cost grows with the stack and earlier conflicts become records-only commits. |
| Chain pull requests, each based on the previous job's branch | Each still carries earlier jobs until they merge; one early failure blocks all later ones. It is the stack under another name. |
| One squash pull request for the stack (`squash-chain-landing.md`) | One failing job holds all, and the chain rule is weaker than the single-job rule. |
| Cherry-pick onto main | The same operation without a defined base; a three-way merge from the recorded base is chosen instead. |
| Raise `[parallel] max_concurrency` | Does not change what a pull request carries. |
| Do nothing | The stage stays at 1842.1 of 1903.5 minutes. |

## 7. Failure cases

- **Merge conflict with main.** The job alone is held and goes through the existing conflict path (return to the
  producer, bounded by `max_conflict_returns`). No other job's code is dropped.
- **Main moved after the cut.** Before merge, the cut is repeated on the new main and the pull request head is
  replaced: one more merge, not a stack. A cut that merges cleanly onto an unchanged governed fingerprint needs no new
  check.
- **Verified content differs.** The cut tree can differ from what the verifier saw because main moved. If the governed
  fingerprint changes, the job is re-checked before it lands (`rebase_needs_recheck`, `recheck_after_rebase`); the
  merged-content check after the merge stays. A landing never proceeds on content nobody verified.
- **Job depends on an unlanded job.** The base content is missing from main, so the cut fails. The job returns to the
  queue behind its predecessor, not to the user. This is the one source of waiting that stays.
- **Branches already stacked.** They drain through the old path until none remain.

## 8. Out of scope

- Any change to `crates/`, `xtask/`, `athena.toml`, `defaults.toml` or workflows.
- Editing `THROUGHPUT.md`, `records/` or the review record.
- Changing the stage definition or the merge method.

## 9. Citation duty

A change that implements this design must cite throughput review `20261109-014617-5cg0` in its request text. The next
throughput review then sets the minutes it measured after that change landed against the 1830.0-minute prediction, by
the rule in section 5.
