# Per-job landing branches: one job, one commit, one pull request, cut from current main

Status: proposed. Nothing here is built. Nothing in `crates/`, `xtask/`, `athena.toml`, `defaults.toml` or
`.github/workflows/` changes with this note. Written for the Athena maintainers and for the next throughput review,
which scores the prediction in section 5.

Source: throughput review `20261108-183648-5eya` (record
`records/throughput-reviews/2026-11/20261108-183648-5eya.json`). It looked at 414 landings and named landing and
rebuild as the biggest wait. Its proposal text is the origin of the figures below.

Labels used below: **measured** (a figure from the review record or `THROUGHPUT.md`, not re-measured here),
**prediction** (the review's expected saving), **estimate** (reasoned here, not measured), **existing** (built; file
named), **new** (not built).

## 1. Problem

**Measured** (review `20261108-183648-5eya`, `THROUGHPUT.md`): the landing and rebuild stage is 1537.1 of the
1582.1-minute median per landed job. The other stages together are about 40 minutes. The stage is defined in
`crates/athena-core/src/status/throughput.rs` (`StageMinutes`): from the end of the job's last invocation to the later of the
landing outcome and the landing record (`landings/`).

The review's reading, which this note adopts as a hypothesis and not a measurement: most of the 1537.1 minutes is not
real waiting. Landing commits stack on one long-lived local branch. The pull request of a job is cut from that
branch, so it carries every landing commit since the last fork from main, not only its own. A job's commit is
therefore seen again in each later pull request until main catches up, and the span of each later job includes the
work of dealing with commits that are not its own: checks that run on a larger diff, rebases of the whole stack,
conflicts in commits the job did not write, and waits for earlier commits to reach main.

What the code does today (read in `crates/athena-core/src/gh/landing.rs`, `crates/athena-core/src/land.rs` and
`docs/design/native-pr-landing.md` section 5):

1. `plan_rebase` replays every first-parent commit between the fork with main and the branch head, oldest first, with
   one three-way merge each. A commit that conflicts, and is not the job's own, is kept as records only (its code is
   dropped and a `Athena-Records-Only` marker is added). So one job's landing depends on the state of all the
   unlanded jobs under it.
2. After a squash or rebase merge, `resync_onto_merge` rebuilds the local branch on the merged commit with
   `Git::replay`, because the squash breaks ancestry (`native-pr-landing.md` section 5, the resync paragraph).
   The PR #40 case in that section is the same stacking fault seen once by hand.
3. The pull request head must be the job's landing commit. `land_pr` refuses any other head and names the extra
   commits (tests `land_pr_refuses_any_other_head_naming_each_extra_commit_and_why`), which is the gate's way of
   saying the head carries more than one job.

The size of the stage is inferred from these, not shown by them: no record in the snapshot splits the 1537.1
minutes by phase (section 9, build job 1 produces that split).

## 2. Design

One job, one commit, one pull request, cut from the current main.

1. **Cut at landing time, from main.** When a job's checks and verifier pass, Athena fetches the host's current main
   and builds the job's landing commit directly on it: a three-way merge with base = the job's recorded base commit,
   ours = current main, theirs = the job's candidate content plus its closing records (`Git::merge_tree`, then
   `Git::commit_tree` with one parent, current main). The result is a single commit with the job's message and
   `Athena-*` trailers. It is not built on top of any other unlanded job.
2. **A branch per job.** That commit is pushed as the job's work branch (`work_branch(job)`) and the pull request
   opens from it. The pull request therefore holds exactly one commit and the diff of one job.
3. **The local branch stops being a staging area.** Landing commits no longer accumulate on it. After the merge is
   confirmed the local branch is moved to the merged commit (a fast-forward in the common case), so there is no
   stack to replay. `resync_onto_merge` shrinks to that move; its replay path stays for hand work on the local
   branch.
4. **Conflicts are decided per job.** A conflict between the job and current main holds back that job alone
   (`Rebase::Conflict` today). It no longer drops other jobs' code to records only, because no other job's commit
   is under it.
5. **Re-check after the cut.** The cut can produce governed content different from what the job verified, because
   main moved. The existing rule decides whether the job is re-checked (`rebase_needs_recheck`,
   `recheck_after_rebase`). A cut that merges cleanly into the same governed fingerprint needs no new check.
6. **Queue and merge are unchanged.** The ordered landing queue, `land_pr` with the merge gate, the landing-intent
   record, the merged-content check and the landing record are used as they are
   (`docs/design/continuous-landing.md`). If a job is cut and main moves again before it merges, the cut is repeated
   on the new main (one merge, no stack) and the pull request head is replaced, as `land_pr` already does for an
   early head.

## 3. Existing code

Nothing in this table is created by this note. Each row says whether the design **keeps** the item or **changes**
it.

| What exists | Where | This design |
| --- | --- | --- |
| `Git::merge_tree`, `Git::commit_tree`, `Git::commit_with_records`, `Git::push_branch`, `Git::fetch_remote_branch`, `Git::is_ancestor` | `crates/athena-git/src/lib.rs` | **Kept** as the building blocks of the cut. No new primitive is needed. |
| `Git::replay(onto, from, to, sign)`: carries a first-parent range onto a tip, one commit per change | `crates/athena-git/src/lib.rs` (line 828) | **Kept**, used only for the one-commit range of the job and for hand work after a merge. It stops being the way every job is carried over a stack. |
| `plan_rebase`, `Rebase`, `rebase_job_branch`: replay the whole stack, records-only fallback for earlier conflicting commits | `crates/athena-core/src/gh/landing.rs` (lines 484 to 707) | **Changed**: the cut replaces the stack replay for landing. The records-only path (`records_only_message`, `Athena-Records-Only`) is kept for existing branches until none remain, then removed (build job 4). |
| `land_pr`, the merge gate, the head check, the early-head replacement, `wait_for_checks` | `crates/athena-core/src/gh/landing.rs`, `docs/design/native-pr-landing.md` section 5 | **Kept**. With one commit per pull request the head check passes in the normal case. |
| `resync_onto_merge`, `resync_tip` | `crates/athena-core/src/land.rs` | **Changed**: a move to the merged commit in the common case; replay only for later hand work. |
| One ordered landing queue, landing lock, `Landing::Waiting`, batched rebuild | `docs/design/continuous-landing.md` (D155) | **Kept** as written. This note acts on a different cause of the same stage: pull requests that carry other jobs' commits. The two changes do not depend on each other; see the interaction in section 7. |
| Squash-chain landing: one squash commit adds the records of several chained jobs, linked by `previous_job` and `base_fingerprint` | `docs/design/squash-chain-landing.md`, `crates/athena-gate/src/invariant.rs` (`landing`) | **Kept**, not used by this design. A pull request holds one job, so the gate's single-job rule applies, which is the path with no chain check. The chain rule stays for adopters who squash a pull request with several jobs. |
| Stage definition and the median over the newest 20 landings (`THROUGHPUT_WINDOW`, `StageMinutes::median_of`) | `crates/athena-core/src/status/throughput.rs`, `crates/athena-data/src/records/throughput.rs` | **Kept** unchanged, so the next review compares like with like. |

## 4. How it removes the bottleneck

**Measured:** landing and rebuild is 1537.1 of 1582.1 minutes; the rest is about 40.

Cutting from main removes the stacked state that the 1537.1 minutes are inferred to come from:

- **No re-carrying.** A later pull request contains one commit. Its diff, checks and review cover one job, and it no
  longer depends on whether earlier jobs have reached main.
- **No stack replay.** `plan_rebase` cost grows with the stack. The cut is one three-way merge.
- **No cross-job conflict holds.** Today an earlier job's conflict turns its commit into records only, and the
  later job's landing waits on that. After the change a conflict belongs to the job that has it.
- **No resync of a stack after a squash.** The local branch moves to the merged commit.

It does not shorten a CI run or change the merge queue. Whatever part of the 1537.1 minutes is not caused by stacking
(for example time behind the queue or in `Waiting`, see `continuous-landing.md`) stays.

## 5. Predicted saving and how it is measured

**Prediction** (review `20261108-183648-5eya`, an estimate and not a measurement): 1530.0 minutes saved per landed
job. The median was 1582.1 minutes per landed job when the review ran.

The implied remainder is 1582.1 - 1530.0 = 52.1 minutes (estimate): about 40 for the other stages and about 12 for
landing and rebuild, which is a bit more than one CI window (the median CI run is 583 seconds, about 10 minutes,
`docs/design/warm-baseline-build.md` section 2). This is only reachable if nearly all of the stage comes from
stacking. It is the upper end of what the design can do.

**How the next throughput review measures it.** It uses the same function as the review that made the prediction:
the median over the newest 20 landings (`THROUGHPUT_WINDOW`) with `StageMinutes::median_of`. It reports:

- the total median minutes per landed job from the end of the verifier to the landing record, compared with 1582.1;
- the landing and rebuild stage compared with 1537.1;
- the saving as 1582.1 minus the new total, set against 1530.0.

It counts only landings made after the change landed, so the window must be full of post-change landings. It also
reports the number of commits in each landed pull request (build job 1 records it); a post-change window where
every pull request holds one commit shows the design was in force. That count only shows the design was in force.
It does not separate this saving from one made by `continuous-landing.md` if that ships in the same window.

The attribution therefore uses the per-phase split from build job 1 (queue wait, cut or rebase, checks, merge). The
review credits this change with the fall in the cut-or-rebase and stacked-carry phases, and credits the continuous
landing change with the fall in the queue-wait and rebuild-batch phases. A drop in a phase is not credited to a
change that does not act on that phase. If the stage falls less than predicted and the commit count is one, the
remainder was not stacking, and the review says so.

## 6. Alternatives considered

| Option | Why not |
| --- | --- |
| Keep stacking, squash on merge | The squash already happens (`native-pr-landing.md` section 5). It collapses the stack on main but not the pull request, which still carries every earlier commit, and the squash breaks ancestry and forces `resync_onto_merge`. It keeps the cause. |
| Rebase the whole stack more often | `plan_rebase` already does this at every landing. It makes each landing dearer as the stack grows and turns earlier conflicts into records-only commits. It keeps the cause. |
| Chain the pull requests: each job's pull request is based on the previous job's branch | The host then merges them in order, and each still carries the earlier jobs until they merge. A conflict or failure early in the chain blocks every later one. This is the stack under another name. |
| Land the stack as one squash pull request (`squash-chain-landing.md`) | Lands several jobs in one step, but one failing job holds all, and the gate's chain rule is weaker than the single-job rule (it catches accidental mismatches, not forgery). The jobs still wait for each other. |
| Cherry-pick a job's commit onto main at landing | Close to the design here. A three-way merge from the job's recorded base is the same operation with a defined base, and `merge_tree` and `commit_tree` already exist. Chosen in that form. |
| Raise `[parallel] max_concurrency` only | Does not touch what a pull request carries. |
| Do nothing | Leaves the stage at 1537.1 of 1582.1 minutes. |

## 7. Risks

- **Conflicts with main.** Cutting from current main means a job's change may conflict where it did not on the stack
  (the stack had the earlier job's code under it). Mitigation: a conflict holds back that job only and goes through
  the existing conflict path (`max_conflict_returns`). Build job 2 adds a test for a job that conflicts with main and
  one that conflicts only with a still-unlanded earlier job.
- **Landing order.** A job that depends on an earlier unlanded job (it edits lines the earlier job added) cannot be
  cut from main until that job lands. It must wait. Mitigation: the ordered landing queue keeps first in, first out
  by verifier pass; a cut that fails because the base content is missing returns the job to the queue behind its
  predecessor, not to the user. This dependency is the one real source of waiting that stays.
- **Verified content drifts.** The cut tree can differ from the tree the job verified. Mitigation: the existing
  re-check rule and the merged-content check after the merge stay; build job 2 tests that a cut with a different
  governed fingerprint is re-checked.
- **Main moves while the pull request waits.** The cut is repeated on the new main and the head is replaced (section 2,
  step 6). The cost is one extra merge, not a stack.
- **Interaction with continuous landing.** If both changes ship, a job may wait in `Waiting` while main moves. The two
  are independent in code. In measurement, the review attributes a drop to the phase it came from (section 5).
- **Existing stacked branches.** Branches built before the change hold stacked commits. They are drained by the
  old path. Build job 4 removes it once none remain.
- **The saving is misread.** The 1537.1 figure may hold waiting unrelated to stacking. Mitigation: section 5 labels
  1530.0 an estimate at the upper bound and asks the review to report the pull request commit count.

## 8. Out of scope

- Any change to `crates/`, `xtask/`, `athena.toml`, `defaults.toml` or the workflows: this note is a design only.
- Editing `THROUGHPUT.md`, the records, or throughput review `20261108-183648-5eya`.
- Changing the stage definition in `crates/athena-core/src/status/throughput.rs`.
- A change to which merge method the repository uses (`native-pr-landing.md` section 5 stays).

## 9. Ordered build jobs

Each is one focused change with its own tests.

1. **Measure.** Record the number of commits in each landed pull request and the time per phase of the stage (queue
   wait, cut or rebase, checks, merge). No behavior change.
2. **Cut a job's commit from current main.** Depends on: 1. A function built on `merge_tree` and `commit_tree` that
   returns the one-commit landing branch; tests for a clean cut, a conflict with main, a dependency on an unlanded
   job, and a re-check after a changed governed fingerprint.
3. **Use the cut in `land_pr` and `resync_onto_merge`.** Depends on: 2. The pull request opens from the cut; the
   local branch moves to the merged commit; tests that the pull request holds one commit.
4. **Remove the stack replay.** Depends on: 3. Drop the `plan_rebase` stack path and records-only fallback once no
   stacked branch remains.

## 10. Citation duty

A change that implements this design must cite throughput review `20261108-183648-5eya` in its request text. The
next throughput review then sets the minutes it measured after that change landed against the 1530.0-minute
prediction, by the rule in section 5.

## Questions for the user

- Does any job today depend on an earlier unlanded job's code (so that it cannot be cut from main)? The records in
  this job's snapshot do not show how often. Build job 1 would count it.
