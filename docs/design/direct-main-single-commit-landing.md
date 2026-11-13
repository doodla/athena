# Single-commit landing on fetched main: retire the long-lived local landing branch

Status: proposed. Nothing here is built. Nothing in `crates/`, `athena.toml`, `defaults.toml`,
`.github/workflows/`, `THROUGHPUT.md` or `records/` changes with this note. Written for Athena's owner, who will
decide whether to build it, and for the next throughput review, which scores the prediction in section 6.

Source: throughput review `20261113-190159-mapn`. It named the landing-and-rebuild stage as the biggest wait in the
pipeline and proposed this change.

Labels: **measured** (a figure from a named source), **estimate** (reasoned here, not measured), **prediction** (a
number a later review must check), **existing** (built; path named), **new** (not built).

## 1. The bottleneck

**Measured** (review `20261113-190159-mapn`, quoting `THROUGHPUT.md`): landing and rebuild is 2310.0 of the 2342.1
median minutes per landed job. The other five stages are small: producer 24.0, verifier 2.8, planning 2.7, checks 0.8,
reading 0.6. Stage medians do not add up to the median of the total, but one stage is almost the whole figure.
`THROUGHPUT.md` is not in this job's working copy, so the figures are copied from the review's request text and were
not re-read from the file.

**Cause**, as `docs/design/per-job-main-landing.md` section 1 read it from
`crates/athena-core/src/status/throughput.rs`: the stage ends at the later of the job's closing outcome and the newest
landing record in `landings/` that lists the job's commit. Athena keeps one long-lived local landing branch. Every
pull request is cut from it, so each pull request carries every earlier unlanded job's commit again, and each writes
a landing record that lists those jobs again. The "newest record that lists the job" keeps moving, so the stage
grows with the number of pull requests cut after the job, not with a wait the job had itself.

## 2. The change

**New.** The local branch stops being a staging area. When a job's verifier passes:

1. Fetch host main with `Git::fetch_remote_branch`; its tip is `ours`.
2. Read the job's recorded base with `dispatched_from` (`base`) and take the job's single landing commit, the
   candidate plus its closing records (`theirs`).
3. Compute the merged tree with `Git::merge_tree(base, ours, theirs)`. `None` means a conflict (section 7).
4. Build one commit with `Git::commit_tree`: parent is the fetched tip, tree is the merged tree, message and trailers
   are those of the job's landing commit.
5. Push it as the job's own new pull request branch with `Git::push_branch`. The pull request holds exactly one
   commit on top of the main tip it was built on.
6. Write the landing record once, listing only this job's commit.
7. Do not write the local landing branch. After a pull request merges, move the local branch to host main with
   `Git::merge_ff_only`, so it follows main and holds nothing of its own.

## 3. Why this removes the bottleneck

- Cause: a job's commit sits on the local branch until it merges, and every pull request cut from the branch in the
  meantime includes it. Fix: the commit is built as an object from `base`, `ours` and `theirs` with host main as
  parent. It is never on the local branch, so no other job's pull request can include it.
- Cause: each such pull request writes a landing record listing the job again. Fix: a job's commit is in exactly one
  pull request and one record, so "newest record that lists the job" is the first record.
- Cause: the stage end keeps moving with each later pull request. Fix: it stops at the first record, written soon
  after verifier pass. The stage becomes the real time from verifier pass to that record.
- Cause: after an earlier job is squash merged, carried commits go stale and need a replay (`resync_onto_merge`).
  Fix: the commit is cut from current main, so another job's squash changes nothing about it.

A reader can check this in `landings/`: after the change a job's commit appears in one file, not many.

**Honest limit.** Per `per-job-main-landing.md` section 1, most of the headline is repeated listings, not a real wait;
the real wait from verifier pass to first record was about 4.6 minutes in that note's sample. Removing the repeated
listings should move the headline far more than any job's real wait. The 5.0-minute prediction is therefore scored
against the first-record stage, not the 2342.1 headline (section 6).

## 4. Existing code

- `crates/athena-git/src/lib.rs` (**existing**, reused unchanged): `Git::fetch_remote_branch`, `Git::merge_tree`,
  `Git::merge_tree_report`, `Git::commit_tree`, `Git::push_branch`, `Git::is_ancestor`, `Git::merge_ff_only`. No new
  primitive is needed.
- `crates/athena-core/src/land.rs` (**existing**, extended): `dispatched_from` and `refuse_detached` give and check
  the recorded base and stay as they are. `plan` already calls `merge_tree` against a `head` argument; the change
  passes the fetched main tip as `head` instead of the local branch head. `resync_tip`, `pr_commits`,
  `classify_commit` and `resync_onto_merge` exist to rebuild the stacked branch after a squash; they leave the normal
  path and stay only for the rollback in section 8.
- `crates/athena-core/src/gh/landing.rs` (**existing**, extended): `land_pr` is changed to publish the single commit
  built in section 2. `close_carried_prs` is not needed once nothing is carried.
- `docs/design/direct-main-merge-tree-landing.md` (**existing** note, review `20261111-055051-tjym`): the same
  mechanism, written when the stage was 2984.3 of 2990.6. This note builds on it, including its conflict handling,
  the pre-merge check that fetched main is still the pull request's parent, and its rollback.
- Earlier notes with the same idea: `direct-main-landing.md` (review `20261110-024838-mch9`),
  `per-job-main-landing.md`, `per-job-landing-branches.md`, `continuous-landing.md`.

**How this note differs.** The mechanism is the same; nothing was built after those notes, and the stage kept
growing. This note adds the newest measurement (2310.0 of 2342.1), a fresh review id that an implementing change
must cite, and an explicit instruction to implement rather than re-propose. If the owner prefers, an implementing
change may fold this note into `direct-main-merge-tree-landing.md`.

## 5. Alternatives considered and rejected

- **Keep the long-lived branch and dedupe landing records.** Rejected: it hides the headline but keeps stack replay,
  cross-job conflicts and carried pull requests.
- **Change only the metric** (end the stage at the first record). Rejected as a separate decision: it changes the
  figure, not any job's behavior, and the review asked for a pipeline change.
- **Reset the local branch to main after each merge.** Rejected: the reset races with jobs that verify meanwhile and
  would drop their commits.
- **Rebase each job onto main in a working copy.** Rejected: slower, needs a checkout; `merge_tree` and `commit_tree`
  work on objects only.
- **Batch several jobs into one pull request.** Rejected: one conflict holds every job in the batch, and the record
  lists several jobs again.

## 6. Prediction and how it is measured

**Prediction (estimate):** the change should save **5.0 minutes per landed job** against the 2342.1 median measured
when review `20261113-190159-mapn` ran. This is a prediction, not a measurement. Because the sampled real wait was
about 4.6 minutes (section 3), 5.0 is at the bound of what the real wait holds; the headline is expected to fall
much further because repeated listings go away, and that fall is reported, not predicted.

**How the next throughput review measures it.** It reads the median minutes per landed job in `THROUGHPUT.md` for
jobs landed after the implementing change (earlier jobs left out), and compares it with the prediction: 2342.1 minus
5.0 is the figure the prediction implies (2337.1). It also reports the first-record stage (verifier pass to the first
landing record listing the job) before and after, and counts landing files that list one job's commit, which should
be one. If the median falls by less than 5.0, or the saving comes only from the headline, the review says so.

## 7. Risks and the failure path

- **Merge conflict at landing.** `Git::merge_tree` returns `None`. The job is held, not landed; conflicted paths come
  from `Git::merge_tree_report`. The landing is retried from a fresh fetch a bounded number of times (the default is
  chosen by the implementing change). Nothing is pushed and the local branch is untouched, so a hold leaves no stale
  state. Conflicts may rise when main moves fast, since each job meets main alone.
- **Host main moves during landing.** `Git::push_branch` pushes a commit as a new branch, so a move of main does not
  make the push fail; the race is at the host merge. Immediately before merging, fetch main again and compare it with
  the pull request's parent. If they differ, refetch, recompute `merge_tree(base, new_tip, theirs)`, build a new
  commit, push it under a fresh branch name, and close the old pull request. If the host refuses the merge anyway,
  treat it the same way. After the retries the job is held.
- **Two jobs land at once.** Both push their own branches; the first to merge moves main, and the second finds its
  parent is now behind the fetched tip and rebuilds. No job's commit is in the other's pull request.
- **Missing or wrong base.** `refuse_detached` and `dispatched_from` already refuse it.
- **Content differs from what was verified.** The stale checks in `land.rs` (`plan`) apply; the job goes back for
  re-verification.
- **Hidden readers of the local branch.** A tool that expects the branch to lead main would break. The implementing
  change must search for readers before it stops writing the branch.
- **The saving may be below 5.0 minutes**, or the headline may fall with no real saving. Section 6 scores both.

## 8. Migration and rollback

- **Migration.** Jobs already on the local branch when the change ships land once by the existing path. New jobs use
  the new path. When the branch holds no unlanded job commit, move it to host main and keep it a follower.
- **Rollback.** Keep the old path behind a setting for one release. Records keep their shape, so no data migration is
  needed in either direction. If the next review shows the stage grew or conflicts rose, revert.

## 9. Requirement on the implementing change

A change that implements this design must cite throughput review `20261113-190159-mapn` in its request text. The
next throughput review then sets the minutes it measured after that change landed against this prediction.
