# Direct main landing: build each job's landing commit on host main, not on a local branch

Status: proposed. Nothing here is built. Nothing in `crates/`, `athena.toml`, `defaults.toml` or `.github/workflows/`
changes with this note. Written for the Athena maintainers and for the next throughput review, which scores the
prediction in section 5.

Source: throughput review `20261110-024838-mch9`. It named the landing-and-rebuild stage as the biggest wait.

Labels: **measured** (a figure from a named source), **estimate** (reasoned here, not measured), **existing** (built;
path named), **new** (not built).

## 1. The bottleneck

**Measured** (throughput review `20261110-024838-mch9`, quoting `THROUGHPUT.md`): the landing-and-rebuild stage is
2203.6 of the 2252.0 median minutes per landed job. The other five stages add up to about 35 minutes. The figure has
risen in each review: 1069.2, 1537.1, 1842.1, now 2203.6. (`THROUGHPUT.md` is not in this job's working copy, so the
figures are copied from the review's request text, not re-read from the file.)

The stage ends at the later of the job's closing outcome and the newest landing record that lists the job
(`crates/athena-core/src/status/throughput.rs`, `landing_end`, as described in `per-job-main-landing.md` section 1).

Cause: one long-lived local landing branch. Each pull request is cut from that branch, so it carries every earlier
job's commit again. Each later pull request therefore writes a new landing record that lists the same job, and the
"newest record" end of the stage keeps moving. The stage grows with the number of pull requests cut after the job, not
with any wait the job itself had.

## 2. The change

**New.** Stop using the local branch as a staging area. When a job's verifier passes:

1. Fetch host main and take its current tip as `ours`.
2. Take the job's recorded base commit (the commit it was dispatched from) as `base`, and the job's single landing
   commit as `theirs`.
3. Compute the merged tree with `Git::merge_tree(base, ours, theirs)` (three-way merge). A conflict returns `None`
   and the job is held, as today.
4. Create the commit with `Git::commit_tree`: parent is host main's tip, tree is the merged tree, message and trailers
   are those of the job's landing commit.
5. Push that commit as the job's pull request branch, so the pull request holds exactly one commit on top of current
   main.
6. Write the landing record once, listing only that job's commit.

The local landing branch is no longer written to, so no later pull request carries this job's commit again.

## 3. How this removes the bottleneck

- One job, one commit, one landing record. A job's commit is listed in exactly one record, so "newest record that
  lists the job" equals "first record". The stage's end stops moving after the first listing.
- The wait is the later of the outcome and that single record. Both happen soon after the verifier passes, so the
  stage shrinks from the cumulative listing time to the real time from verifier pass to the first record.
- No stack replay and no resync after a squash: the commit is cut from main as it is now, so a squash merge of an
  earlier job does not leave this job's commit stale.

Honest limit: `per-job-main-landing.md` section 1 re-measured that most of the headline is a measurement effect, not a
real wait. This note's saving below is the real-wait part only, and the headline will fall by more than the saving.

## 4. Existing code and what differs

- `docs/design/per-job-main-landing.md` (existing note): proposes one job, one commit, one pull request cut from
  current main. This note reuses that idea and adds the throughput review's measurement, the exact git steps
  (`merge_tree` from the recorded base, then `commit_tree`) and the prediction in section 5. It does not rewrite it.
- `docs/design/per-job-landing-branches.md` (existing note): per-job branches for landing. This note extends it: the
  branch is only a push target for the single commit, never a place where commits accumulate.
- `docs/design/continuous-landing.md`: the landing cadence. Unchanged; landing still happens as jobs verify.
- `crates/athena-git/src/lib.rs`: `Git::merge_tree` and `Git::commit_tree` already exist and are reused as they are.
  Difference: today's landing does not call them on host main's tip for each job.
- `crates/athena-core/src/land.rs`: `plan`, `resync_tip`, `pr_commits` and `resync_onto_merge` build and resync the
  stacked branch. Difference: `plan` would build the commit on host main as in section 2, and `resync_tip` and
  `resync_onto_merge` would no longer be needed on the normal path.

## 5. Prediction and how it is measured

Predicted saving: **5.0 minutes per landed job** (estimate; the median was 2252.0 minutes per landed job when the
review ran). It is small because the real wait is small (see the honest limit above); the rest of the headline falls
as a side effect of the single listing.

How the next throughput review measures it: after a change implementing this design has landed, it takes the
landing-and-rebuild minutes of the jobs landed after that change (median over its window, from `THROUGHPUT.md` and
`athena status`), and compares them with the 2203.6 minutes measured here, and the saving with the 5.0 minute
prediction. It reports the difference, and whether the stage's median now sits near the first-landing time. Jobs
landed before the change are left out of the "after" figure.

## 6. Requirement on the implementing change

A change that implements this design must cite throughput review `20261110-024838-mch9` in its request text. The next
throughput review then sets the minutes it measured after that change landed against this prediction.

## 7. Alternatives considered

- Keep the stacked branch and dedupe landing records. Rejected: it hides the number but keeps stack replay and
  conflicts.
- Rebase each job onto main with a worktree checkout. Rejected: slower and needs a working directory; `merge_tree`
  and `commit_tree` work on objects only.
- Cut the pull request from the local branch but reset the branch after each merge. Rejected: the reset races with
  jobs that verify meanwhile.
- Change only the metric (end the stage at the first record). Rejected here as a separate decision; it would not
  change any job's behavior.

## 8. Risks

- Merge conflicts against a moving main: the job is held and retried from the new tip. Mitigation: retry a bounded
  number of times.
- A race between two jobs pushing at once: the second push is refused as not a fast-forward of its own base; it
  refetches and rebuilds. The commit is cheap to rebuild.
- The base recorded for a job is wrong or missing: `refuse_detached` and `dispatched_from` in `land.rs` already
  refuse that case; keep them.
- The measured saving may be below 5.0 minutes: section 5 scores it either way.

## 9. Rollback

Keep the stacked-branch path behind a setting for one release. To roll back, turn the setting off: landing returns to
the local branch and `resync_onto_merge`. No records change shape, so no data migration is needed. If the next
throughput review shows the stage grew or conflicts rose, revert the change.
