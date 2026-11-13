# Single-commit landing on fetched main: retire the long-lived local landing branch

Status: proposed. Nothing here is built. Nothing in `crates/`, `athena.toml`, `defaults.toml`,
`.github/workflows/`, `THROUGHPUT.md` or `records/` changes with this note. Written for Athena's owner and for the
next throughput review, which scores the prediction in section 5.

Source: throughput review `20261113-190159-mapn` (`records/throughput-reviews/2026-11/20261113-190159-mapn.json`).
It names the landing-and-rebuild stage as 2310.0 of the 2342.1 median minutes per landed job (`THROUGHPUT.md`), and
proposes this change with an expected saving of 5.0 minutes. The figures are copied from that record's text;
`THROUGHPUT.md` is not in this job's working copy.

Labels: **measured** (a figure from a named source), **estimate** (reasoned here, not measured), **existing** (built;
path named), **new** (not built).

## 1. The bottleneck and the change

**Measured** (review `20261113-190159-mapn`): landing and rebuild is 2310.0 of 2342.1 median minutes per landed job.
**Cause** (stated by the review, and by `docs/design/per-job-main-landing.md` section 1): Athena keeps one long-lived
local landing branch. Every pull request is cut from it, so each one carries every earlier unlanded job's commit and
writes a landing record that lists those jobs again. The stage ends at the newest landing record that lists the job,
so it keeps moving while later pull requests are cut.

**New.** Stop using the local branch as a staging area. When a job's verifier passes:

1. Fetch host main (`Git::fetch_remote_branch`); its tip is `ours`.
2. Take the job's recorded base `P` from `Verified::base()` and the job's candidate `C` from `Verified::candidate()`.
3. Compute the merged tree with `Git::merge_tree(P, ours, C)`. A conflict returns `None`; see section 6.
4. Create one commit with `Git::commit_tree`: parent is the fetched tip, tree is the merged tree.
5. Push it as the job's own pull request branch (`Git::push_branch`). The pull request holds one commit on top of
   the main tip it was built on.
6. Write the landing record once, listing only this job's commit.
7. The local landing branch holds nothing of its own: after a pull request merges, it is fast-forwarded to host main.

## 2. How this removes the wait

- A job's commit never sits on a shared branch, so no other job's pull request can carry it.
- A job's commit is built into one pull request and one landing record, so "newest record that lists the job" is the
  first record and no longer moves with later pull requests.
- A squash merge of another job changes nothing about this job's commit, since it is cut from current main. The replay
  after a squash (`resync_onto_merge`, `resync_tip`, `plan_rebase`) leaves the normal path.

## 3. Existing code

- `crates/athena-git/src/lib.rs` (**existing**, reused, no new primitive): `Git::fetch_remote_branch`,
  `Git::merge_tree`, `Git::merge_tree_report` (conflicted paths), `Git::commit_tree`, `Git::push_branch`,
  `Git::is_ancestor`.
- `crates/athena-core/src/land.rs` (**existing**, extended). `plan(git, verified, head, dispatched)` takes the base
  from `Verified::base()` (`let p = verified.base()`). It already computes `git.merge_tree(p, head, c)` when `head`
  differs from `p`, and checks that the landed fingerprint equals the verified one. The change is to pass the fetched
  main tip, on a follower branch, as `head`. `dispatched_from` and `refuse_detached` are only branch checks: they
  read or require the branch the job was dispatched from and a checked-out HEAD, and they do not find or check the
  base commit.
- `crates/athena-core/src/land.rs` `resync_tip`, `pr_commits`, `classify_commit`, `resync_onto_merge`
  (**existing**, left in place): they rebuild a stacked branch after a squash. They are kept for the rollback path in
  section 7.
- `crates/athena-core/src/gh/landing.rs` (**existing**, extended): `land_pr` publishes the single commit from
  section 1; `close_carried_prs` is not needed once nothing is carried.
- `docs/design/direct-main-merge-tree-landing.md` (**existing** note, for an earlier review): proposes the same
  mechanism. This note re-states it against review `20261113-190159-mapn`, names `Verified::base()` as the base
  source, states the preconditions of `plan` (section 4) and sets the measurement (section 5). An implementing change
  may fold the two notes together.

## 4. Preconditions of `plan`, and how migration meets them

`plan` has two preconditions that this design must keep true:

1. **The dispatch branch is checked out.** `plan` returns `Plan::OtherBranch` when the job's recorded dispatch branch
   is not the checked-out branch, and `refuse_detached` stops a detached HEAD.
2. **The base is an ancestor of the head.** `plan` returns `Plan::Stale` when `git.is_ancestor(p, head)` is false.

Migration meets them as follows:

- The local landing branch stays checked out and keeps its name, as a follower of host main, so the dispatch branch
  recorded in job records still exists and is checked out. Before `plan`, the follower is fast-forwarded to the
  fetched main tip, so `head` is that tip.
- A job's base is a commit on the old long-lived branch. It is an ancestor of the fetched tip only once the commits
  it sits on have reached main. So jobs already on the branch when the change ships are landed once by the old path,
  and the branch is moved to host main only when it holds no unlanded job commit. A job whose base is still not an
  ancestor is stale under the existing rule; `athena retry` (`Athena::stale_seed`) already re-verifies it as a new job
  on the current tip.

## 5. Prediction and how it is measured

**Prediction (estimate, not measured):** this change saves **5.0 minutes per landed job** against the 2342.1 median
(`THROUGHPUT.md`, as quoted by review `20261113-190159-mapn`). That is the review's own expected figure; this note
adds no measurement behind it.

**How the next review measures it:** it takes the `THROUGHPUT.md` median minutes per landed job for jobs landed after
the change, and compares it with 2337.1 (2342.1 minus 5.0). A median at or below 2337.1 meets the prediction; a
higher one does not. This is the only basis for scoring.

## 6. Conflicts and the failure path

- **Conflict at landing.** `Git::merge_tree` returns `None` when `C` and the main tip change the same lines. The job
  is held, not landed, and the conflicted paths are reported by `Git::merge_tree_report`. Nothing is pushed and the
  local branch is untouched. The landing is retried from a fresh fetch a bounded number of times (the bound is chosen
  by the implementing change); after that the job stays held, and `athena retry` applies as for a stale job.
- **Main moves while landing.** `Git::push_branch` pushes a new branch, so a move of main does not make the push
  fail. Just before asking the host to merge, the job fetches again and checks with `Git::is_ancestor` that the pull
  request's parent is still the tip. If not, it recomputes `merge_tree` from the new tip, builds a new commit, pushes
  it under a fresh branch name, and closes the old pull request. A host refusal at merge time is treated the same way.
- **Content differs from what was verified.** `plan` bails when the landed fingerprint differs from the verified one;
  that stays.
- **Failure path.** Every failure above leaves records and the follower branch unchanged, so a failed landing costs a
  retry and no cleanup.

## 7. Migration and rollback

Keep the old path behind a setting for one release. Turning it off returns landing to the local branch and
`resync_onto_merge`. Landing records keep their shape, so no data migration is needed either way. If the next review
shows the median above 2337.1 or conflicts rising, revert the change.

## 8. Alternatives considered and rejected

- **Keep the long-lived branch and dedupe landing records.** Rejected: it hides the repeated listings but keeps stack
  replay, cross-job conflicts and carried pull requests.
- **Reset the local branch to main after each merge.** Rejected: the reset races with jobs that verify meanwhile and
  would drop their commits.
- **Rebase each job onto main in a working copy.** Rejected: slower, needs a checkout, and `merge_tree` and
  `commit_tree` work on objects only.
- **Batch several jobs into one pull request.** Rejected: one conflict then holds every job in the batch, and the
  landing record lists several jobs again.

## 9. Risks

- The measured median may stay above 2337.1; section 5 scores that.
- More conflicts when main moves fast, since each job meets main alone. Mitigation: bounded retry and the held state.
- A reader of the local branch (a tool expecting it to lead main) may break. The implementing change must search for
  such readers before the branch stops being written.
- Jobs on the old branch at migration go stale if their base never reaches main; `athena retry` recovers them.

## 10. Requirement on the implementing change

A change that implements this design must cite throughput review `20261113-190159-mapn` in its request text, so the
next review can match the change to this prediction.
