# Direct main landing with merge_tree: land each job on fetched host main, not through the local branch

Status: proposed. Nothing here is built. Nothing in `crates/`, `athena.toml`, `defaults.toml`,
`.github/workflows/`, `THROUGHPUT.md` or `records/` changes with this note. Written for the Athena maintainers who
will implement and judge the change, and for the next throughput review, which scores the prediction in section 6.

Source: throughput review `20261111-055051-tjym`. It named the landing-and-rebuild stage as the biggest wait in the
pipeline and proposed this change.

Labels: **measured** (a figure from a named source), **estimate** (reasoned here, not measured), **prediction** (a
number a later review must check), **existing** (built; path named), **new** (not built).

## 1. The bottleneck

**Measured** (review `20261111-055051-tjym`, quoting `THROUGHPUT.md`): landing and rebuild is 2984.3 of the 2990.6
median minutes per landed job. The review gives reading, planning, producer, checks and verifier together as about
28.5 minutes. (Stage medians do not add up to the median of the total, which is why 2984.3 plus 28.5 is not 2990.6.
Either way, one stage is almost the whole figure.) `THROUGHPUT.md` is not in this job's working copy, so the figures
are copied from the review's request text, not re-read from the file.

**Measured** (earlier notes, each quoting the review that wrote it): the same stage was 440.1
(`continuous-landing.md`), 1069.2 (`agent-and-build-slots.md`), 1537.1 (`per-job-landing-branches.md`), 1842.1
(`per-job-main-landing.md`), 2203.6 (`direct-main-landing.md`) and is now 2984.3. It has grown in every review.

**Cause**, as `per-job-main-landing.md` section 1 read it from `crates/athena-core/src/status/throughput.rs`: the
stage ends at the later of the job's closing outcome and the newest landing record in `landings/` that lists the
job's commit. Athena keeps one long-lived local landing branch. Every pull request is cut from it, so each pull
request carries every earlier unlanded job's commit again, and each one writes a new landing record that lists those
jobs again. The "newest record that lists the job" keeps moving, so the stage grows with the number of pull requests
cut after the job, not with any wait the job had itself.

## 2. The change

**New.** Stop using the local branch as a staging area. When a job's verifier passes:

1. Fetch host main (`Git::fetch_remote_branch`) and take its tip as `ours`.
2. Take the job's recorded base (the commit it was dispatched from, as `dispatched_from` reads it) as `base`, and the
   job's single landing commit (its candidate plus its closing records) as `theirs`.
3. Compute the merged tree with `Git::merge_tree(base, ours, theirs)`. A conflict returns `None`; see section 7.
4. Create the commit with `Git::commit_tree`: parent is the fetched tip, tree is the merged tree, message and
   trailers are those of the job's landing commit.
5. Push it as the job's own pull request branch (`Git::push_branch`). The pull request holds exactly one commit on
   top of the main tip it was built on.
6. Write the landing record once, listing only this job's commit.
7. The local landing branch is not written. After the pull request merges, the local branch is moved to host main
   (fast-forward only), so it follows main and holds nothing of its own.

## 3. Why this removes the bottleneck

The cause-to-fix link, step by step:

- Cause: a job's commit sits on the local branch until that branch is merged, and every pull request cut from the
  branch in the meantime includes it. Fix: the commit is built from `base`, `ours` and `theirs` as objects, with
  host main as parent. The job's commit is never on the local branch, so no other job's pull request can include it.
- Cause: each such pull request writes a landing record listing the job again. Fix: a job's commit is built into
  exactly one pull request and exactly one record. "Newest record that lists the job" equals "first record".
- Cause: the stage end keeps moving with every later pull request. Fix: it stops at the first record, which is
  written shortly after verifier pass. The stage becomes the real time from verifier pass to that record.
- Cause: when an earlier job is squash merged, the later jobs' carried commits go stale and need a replay
  (`resync_onto_merge`, `plan_rebase`). Fix: the commit is cut from current main, so a squash merge of another job
  changes nothing about it. A job whose pull request main moved past is rebuilt from a fresh fetch (section 7).

A reader can check this in `landings/`: after the change, a job's commit appears in one file, not many. Before it,
`per-job-main-landing.md` section 1 found one job's commit in 60 of 215 landing files.

**Honest limit.** `per-job-main-landing.md` section 1 re-measured that most of the headline is the repeated listings
rather than a real wait: the real wait from verifier to first record was about 4.6 minutes in its sample. Removing
the repeated listings will therefore move the headline by far more than it moves any job's real wait. The 5.0-minute
prediction in section 6 is scored against that first-record stage (verifier pass to first landing record), whose
bound is about 4.6 minutes. It is not scored against the headline. The headline is expected to fall much further,
toward the first-record time, and that fall is reported but is not the prediction.

## 4. Existing code:

- `crates/athena-git/src/lib.rs` (**existing**, reused): `Git::merge_tree`, `Git::commit_tree`, `Git::push_branch`,
  `Git::fetch_remote_branch`, `Git::is_ancestor`. No new primitive is needed.
- `crates/athena-core/src/land.rs` (**existing**, extended): `dispatched_from` and `refuse_detached` give and check
  the recorded base and stay as they are. `plan` already calls `merge_tree` against the local branch head
  (`merge_tree(p, head, c)`); the change passes the fetched main tip as `head`. `resync_tip`, `pr_commits`,
  `classify_commit` and `resync_onto_merge` exist to rebuild the stacked branch after a squash; they leave the normal
  path and are kept only for the fallback in section 8.
- `crates/athena-core/src/gh/landing.rs` (**existing**, extended): `land_pr` opens the pull request, `publish` pushes
  the branch, `plan_rebase` and `rebase_job_branch` rebase a job branch onto moved main, and `close_carried_prs`
  closes pull requests that carry other jobs' commits. It also already calls `merge_tree`. The change makes
  `land_pr` publish the single commit built in section 2; `close_carried_prs` is not needed once nothing is carried.
- `docs/design/direct-main-landing.md` (**existing** note, review `20261110-024838-mch9`): proposes the same
  mechanism, with the same 5.0-minute prediction against a median of 2252.0.
- `docs/design/per-job-main-landing.md` (**existing** note, review `20261109-014617-5cg0`): proposes one job, one
  commit, one pull request cut from current main, and re-measures that the headline is mostly repeated listings.
- `docs/design/per-job-landing-branches.md`, `docs/design/continuous-landing.md`: earlier landing designs.

**How this note differs, and why the earlier designs did not remove the wait.** Those notes carry the status
"proposed; nothing here is built", and the stage kept growing after each (440.1, 1069.2, 1537.1, 1842.1, 2203.6,
2984.3, all measured in the notes above). The headline is still the repeated-listing figure, so the earlier designs
did not remove it, because the long-lived branch is still how landings are made. This note adds nothing new to the
mechanism of `direct-main-landing.md` and `per-job-main-landing.md`. It re-states it against the new measurement
(2984.3 of 2990.6), makes each step name the function it uses (section 2), pins down conflict and race handling
(section 7), fixes the migration and rollback (section 8), and sets the prediction and measurement (section 6). If
the maintainers prefer, an implementing change may fold this note into `direct-main-landing.md`; the request text
must still cite this review (section 9).

## 5. Alternatives considered and rejected

- **Keep the long-lived branch and dedupe landing records.** Rejected: it hides the headline but keeps stack replay,
  cross-job conflicts and carried pull requests.
- **Change only the metric** (end the stage at the first record). Rejected here as a separate decision: it changes the
  figure, not any job's behavior, and the review asked for a change to the pipeline.
- **Reset the local branch to main after each merge.** Rejected: the reset races with jobs that verify meanwhile and
  would drop their commits.
- **Rebase each job onto main in a working copy.** Rejected: slower, needs a checkout, and `merge_tree` and
  `commit_tree` work on objects only.
- **Cherry-pick each job's commit onto main.** Rejected as a separate path: a three-way merge from the job's recorded
  base is the same operation with a defined base, and the primitives exist.
- **Batch several jobs into one pull request.** Rejected: it brings back a shared unit, so one conflict holds every job
  in the batch, and the landing record lists several jobs again.

## 6. Prediction and how it is measured

**Prediction:** this change should save **5.0 minutes per landed job** of the real wait, the stage from verifier pass
to the job's first landing record. This is a prediction, not a measurement. The note's own figure for that stage is
about 4.6 minutes (section 3), so 5.0 is at the bound of what that stage holds, and the first-record stage after the
change is expected to be near zero to a few minutes; a saving above about 4.6 is not expected. The 2990.6-minute
median is the headline and is not what is saved here.

**How the next throughput review measures it.** After a change implementing this design has landed, the review
measures one figure: the median minutes from verifier pass to the first landing record listing the job, over the jobs
landed after that change (jobs landed before it are left out). It sets that against the same stage measured before
(about 4.6 minutes in `per-job-main-landing.md`, to be re-measured on the same sample method), and compares the
difference with the 5.0-minute prediction. The headline 2984.3 median is not the figure compared with 5.0. The review
reports the headline separately, as expected to fall toward the first-record time, and counts the landing files that
list one job's commit, which should be one.

## 7. Conflict handling

- **`merge_tree` conflict.** `Git::merge_tree` returns `None`. The job is held, not landed, and the conflicted paths
  are reported (`Git::merge_tree_report` already returns them). The landing is retried from a fresh fetch up to a
  bounded number of times (a setting; the default is chosen by the implementing change). Nothing is pushed, and the
  local branch is untouched, so a hold leaves no stale state.
- **Host main moves while landing: the push does not fail.** `Git::push_branch` (`crates/athena-git/src/lib.rs`,
  lines 430-436) is a plain push of a commit as a new branch. Git refuses it only when the branch already exists and
  the commit does not descend from it. The job's pull request branch is new, so a move of host main never makes the
  push fail. This design therefore does not rely on a push refusal. A pull request built on an old tip is pushed
  without complaint.
- **Where the move is detected: just before the host merge.** The race is at the host merge, which uses a
  `MergeMethod` (`crates/athena-core/src/gh/client.rs`; squash, merge or rebase) and would combine a pull request
  built on an old tip with no check of its own. So, immediately before asking the host to merge, the job fetches host
  main again and checks that the pull request's parent is still the fetched main tip: `Git::is_ancestor(fetched_tip,
  parent)` is true exactly when the tip is the parent or older. As a second guard the repository may require branches to be up to date before merging
  (a host setting, not code in this repository; stated here as an option, not as a measured behavior).
  - **Signal when the parent is still the tip:** the fetched tip equals the parent, so the merge goes ahead.
  - **Signal when main has moved:** `Git::is_ancestor(fetched_tip, parent)` is false, or the tip differs from the
    parent (the parent is an ancestor of the fetched tip but not equal to it). The job does not
    merge. It refetches, recomputes `Git::merge_tree(base, new_tip, theirs)`, builds a new commit with
    `Git::commit_tree`, and pushes it under a fresh branch name, because the new commit does not descend from the old
    branch commit and a push to the old name would be refused. It then closes the old pull request. After the bounded
    retries the job is held.
  - **Signal when the host refuses the merge anyway** (main moved between the check and the merge call): the host
    returns an error to the merge request; the job treats it like the moved-main case and rebuilds.
- **Two jobs land at once.** Both build on the tip they fetched and both push their own new branches; neither push
  fails. The first to merge moves main. For the second, the pre-merge check finds its parent is now an ancestor of,
  but not equal to, the fetched tip, so it rebuilds from a fresh fetch as above. If both reach the merge call at the
  same moment, the host's refusal (or the up-to-date requirement) catches the second. No job's commit is ever
  listed in the other's pull request.
- **Base missing or wrong.** `refuse_detached` and `dispatched_from` in `land.rs` already refuse a missing or
  detached base; keep them.
- **Content differs from what was verified.** If the merged tree differs from the verified fingerprint because main
  changed governed files, the existing stale checks in `land.rs` apply and the job goes back for re-verification.

## 8. Migration and rollback

- **Migration.** Jobs already on the local landing branch when the change ships are landed once by the existing path
  (pull requests cut from the branch, `resync_onto_merge` after each squash). New jobs, verified after the change,
  use the new path. When the branch holds no unlanded job commit, move it to host main and keep it a follower.
- **Rollback.** Keep the old path behind a setting for one release. To roll back, turn the setting off: landing returns
  to the local branch and `resync_onto_merge`. Records do not change shape (a landing record still lists commits), so
  no data migration is needed in either direction. If the next review shows the stage grew or conflicts rose,
  revert the change.

## 9. Requirement on the implementing change

A change that implements this design must cite throughput review `20261111-055051-tjym` in its request text. The next
throughput review then sets the minutes it measured after that change landed against this prediction.

## 10. Risks

- The measured saving may be below 5.0 minutes, or the headline may fall without any real saving. Section 6 scores
  both.
- More conflicts when main moves fast, since each job meets main alone. Mitigation: bounded retry and the held state.
- A hidden dependence on the local branch holding landed commits (for example a reader of `landings/` or a tool that
  expects the branch to lead main). The implementing change must search for readers of the local branch before it
  stops writing to it.
