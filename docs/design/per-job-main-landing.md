# Per-job main landing: one job, one commit, one pull request, cut from current main

Status: proposed. Nothing here is built. Nothing in `crates/`, `xtask/`, `athena.toml`, `defaults.toml`,
`.github/workflows/`, `THROUGHPUT.md` or `records/` changes with this note. Written for the Athena maintainers and for
the next throughput review, which scores the prediction in section 5.

Source: throughput review `20261109-014617-5cg0` (record
`records/throughput-reviews/2026-11/20261109-014617-5cg0.json`). It looked at 420 landings, found the median at 1903.5
minutes per landed job, and named the landing-and-rebuild stage, 1842.1 of those minutes, as the biggest. Its proposal
expected 1830.0 minutes saved. This note checks that expectation first (section 1) and does not adopt it.

Labels used below: **measured** (a figure read from a named file or command output), **sample** (measured on a few
jobs, not the whole window), **estimate** (reasoned here, not measured), **existing** (built; file named), **new** (not
built).

Terms: the **first-landing stage** is the minutes from the end of the job's verifier to the `created_at` of the first
file in `landings/` that lists the job's commit. The **headline stage** is the "landing and rebuild" figure of
`athena status`, which ends at the *newest* such file.

## 1. Premise check: what the 1842.1 minutes measure

**Measured** (`cargo run -p athena-cli -- status --snapshot <evidence>`, this job's evidence snapshot): median per
landed job over the last 20 measured is 1903.5 minutes: reading 0.4, planning 11.0, producer 22.5, checks 1.0,
verifier 3.3, landing and rebuild 1842.1. The other five stages are 61.4 minutes together (1903.5 - 1842.1).

**What the stage is.** `crates/athena-core/src/status/throughput.rs`, `landing_end`: the end of the stage is the later
of the job's closing outcome and `landing_record_time`. `landing_record_time` takes `.max()` of `created_at` over every
record in `landings/` whose commit list holds a `job_landing` commit of the job. So the stage ends at the **newest**
landing record that lists the job, not the first.
`docs/design/agent-and-build-slots.md` section 6.1 reads this and shows it on the records: a job's commit is listed in
many landing records, because each later pull request cut from the one long-lived local branch carries the earlier
jobs' commits again.

**Re-measured here** (evidence snapshot, `landings/`): the text `"job": "20261104-054754-b4f5"` is in 60 of the 215
landing files. The first is `20261104-062243-y5nm`; the newest is `20261109-014430-574k`. From the file names that is
about 4 days 19 hours 22 minutes, about 6,920 minutes (estimate: file-name times, not `created_at`). Section 6.1 gives
the same job as 4.3 minutes from its verifier to the first record (`created_at`, sample of five jobs) and so about
6,925 minutes to the newest. The same job's commit, with the same tree, is listed 60 times over five days.

**Real elapsed wait versus the measurement effect.**

| Quantity | Value | Source |
| --- | --- | --- |
| Headline stage, median of newest 20 | 1842.1 min | measured, `status --snapshot` |
| Verifier end to first landing record, median of five jobs | 4.6 min (range 4.3 to 11.0) | sample, `agent-and-build-slots.md` section 6.1 table (jobs landed 2026-11-04, an older window) |
| Same five jobs to the newest record | median 1337.0 min | sample, same table |
| Real wait of the newest 20 jobs (first-landing stage) | not measured here; estimate in the single digits to low tens of minutes | estimate, from the 4.6 median and 11.0 maximum above |
| The rest of the headline stage | about 1840 min or more of it is later listings of the same commit | estimate: 1842.1 minus the estimate above |

So the 1842.1 minutes are **mostly a measurement effect**: the clock keeps running because later pull requests list
the job's commit again. The wait that a job really experienced between verifier pass and its first landing record is
small by comparison (sample 4.6 minutes). The 1830.0 minutes the review expected saved is **not** a real wait and this
note does not claim it. Whether the first-landing stage of the newest 20 jobs is larger than the sample is not known;
section 5 has the next review measure it.

What follows from that: the real problem the stacked branch causes is smaller than the headline and of a different
kind. It is the cost of stack replay, cross-job conflicts and the resync after a squash (section 2), which show up
as failed or held landings and as work, not as 1842 minutes of waiting. The note proposes the change for those causes
and says plainly that the change removes the repeated listings as a side effect, which changes the headline number
without changing any job's real wait.

## 2. How landing works today (existing code)

Read in this snapshot; each row says what the design **keeps** or **changes**.

| What exists | Where | This design |
| --- | --- | --- |
| The landing decision: the job lands on its dispatch branch; the head `H` must descend from the base `P`; the verified fingerprint must equal the landed one. Merge by `merge-tree --merge-base P H C` | `crates/athena-core/src/land.rs` (module comment, `plan`) | **Kept**. The cut in section 3 is the same three-way merge applied to current main. |
| `resync_tip`, `resync_onto_merge`: move the local branch after a merge, replaying commits that the squash left out | `crates/athena-core/src/land.rs` (`resync_tip` line 218, `resync_onto_merge` line 311) | **Changed**: in the common case a move to the merged commit; the replay stays for hand work on the local branch. |
| `pr_commits`, `classify_commit`: classify each commit a pull request carries over the remote base | `crates/athena-core/src/land.rs` (lines 267, 284) | **Kept**. With one commit per pull request the list has one entry. |
| `land_pr`: opens or replaces the head, refuses any head other than the job's landing commit, waits for the required check, merges | `crates/athena-core/src/gh/landing.rs` (`land_pr`, line 179) and `docs/design/native-pr-landing.md` section 5 | **Kept**. The head check, early-head replacement and merge gate are used as they are. |
| `plan_rebase`, `rebase_job_branch`: replay every first-parent commit between the fork with main and the head; an earlier conflicting commit is kept as records only | `crates/athena-core/src/gh/landing.rs` (lines 665, 761) | **Changed**: replaced by the one-job cut for landing. The records-only path stays for branches built earlier, then goes (section 8). |
| `rebase_needs_recheck`, `recheck_after_rebase`: decide whether a rebased job is checked again | `crates/athena-cli/src/drain.rs` (lines 745, 1737) | **Kept**, applied to the cut. |
| Squash-chain landing: one squash commit adds the records of several chained jobs, linked by `previous_job` and `base_fingerprint` | `docs/design/squash-chain-landing.md`, `crates/athena-gate/src/invariant.rs` (`landing`) | **Kept**, not used by this design. A pull request with one job takes the gate's single-job rule. The chain rule stays for adopters who squash several jobs. |
| Native pull-request landing, merge method follows the repository, resync after a squash | `docs/design/native-pr-landing.md` sections 5 and 6 | **Kept**; only the resync shrinks (above). |
| Pull-request-level stacking noted as the fault, with a records-only fallback | `docs/design/per-job-landing-branches.md` (earlier note, review `20261108-183648-5eya`) | **Extended**: same cut. This note replaces its prediction with the premise check of section 1 and gets the pointer that says which review an implementing change cites (section 7). |
| Ordered landing queue, `Landing::Waiting`, batched rebuild | `docs/design/continuous-landing.md` | **Kept**. Acts on queueing, a different cause; independent in code. |
| Stage definition and median over the newest 20 landings (`THROUGHPUT_WINDOW`, `StageMinutes::median_of`) | `crates/athena-core/src/status/throughput.rs`, `crates/athena-data/src/records/throughput.rs` | **Kept** unchanged, so the next review compares like with like. The first-landing stage is reported next to it (section 5). |
| Git primitives: `Git::merge_tree`, `Git::commit_tree`, `Git::push_branch`, `Git::fetch_remote_branch`, `Git::is_ancestor`, `Git::replay` | `crates/athena-git/src/lib.rs` | **Kept**; no new primitive. |

## 3. Design

One job, one commit, one pull request, cut from the current main.

1. **Cut at landing time, from main.** When a job's checks and verifier pass, fetch the host's main and build the
   job's single landing commit directly on it: a three-way merge with base = the job's recorded base commit, ours =
   current main, theirs = the job's candidate plus its closing records (`Git::merge_tree`, then `Git::commit_tree`
   with one parent). It is not built on top of any other unlanded job.
2. **A branch per job.** The commit is pushed as the job's work branch; the pull request holds one commit.
3. **The local branch stops being a staging area.** After the merge is confirmed it moves to the merged commit.
4. **Conflicts belong to the job that has them.** A conflict with current main holds back that job alone. No other
   job's code is dropped to records only.
5. **Re-check after the cut** by the existing rule (`rebase_needs_recheck`): a cut that merges into the same governed
   fingerprint needs no new check.
6. **Queue and merge are unchanged.** If main moves before the merge, the cut is repeated on the new main and the head
   replaced, as `land_pr` already does for an early head.

### Why this helps, and what it does not

- **Real effects (all unmeasured here):** no stack replay (one merge instead of one per stacked commit), no
  cross-job conflict that turns another job's commit into records only, no resync of a stack after a squash. These
  reduce real work and failures in landing. Their size in minutes is not in any record in the snapshot, so no
  minutes are claimed for them.
- **Measurement effect:** each job's commit would be listed in one landing record, not in dozens, so the headline
  stage would end at the first record and fall toward the first-landing stage. That fall is the removal of the
  repeated listings. It is not time saved for any job, and the next review must not score it as saved wait.
- **What it does not change:** CI run length, the queue, `Waiting`, the rebuild.

## 4. Problem size, in one quantity

The quantity used throughout is **minutes from verifier pass to first landing record**.

- Today (sample, section 1): median 4.6 minutes, maximum 11.0 in five jobs. The newest-20 value is not measured here.
- Ceiling on real saving from this quantity: the whole first-landing stage, so about 5 minutes per job on the sample
  (estimate). Anything above that in the headline is the measurement effect.
- The other stages are 61.4 minutes (measured) and are not touched.

## 5. Predicted saving and how the next review scores it

**Prediction (estimate):** the real saving, in minutes from verifier pass to first landing record, is between 0 and
about 5 minutes per landed job (the sample median is 4.6). The change is chosen for the work and failure causes in
section 3, not for this number. **The 1830.0 minutes of review `20261109-014617-5cg0` is not claimed**, because the
records show it is later listings of one commit, not wait (section 1).

**What the next throughput review reports**, from the same newest-20 window and the same
`StageMinutes::median_of`:

1. the **unchanged landing stage** (the headline: verifier end to newest landing record), compared with 1842.1;
2. the **first-landing stage** (verifier end to first landing record), compared with the sample 4.6 and with its own
   value before the change, measured on the pre-change landings by the same method;
3. the number of commits in each landed pull request and the number of landing records that list each job (one per
   job after the change; this shows the design was in force);
4. the total median, compared with 1903.5.

**Scoring rule.** The saving is the fall in the first-landing stage, in minutes, between the pre-change and
post-change windows. A fall in the headline stage that is not matched by a fall in the first-landing stage is credited
as the end of repeated listings and is not counted as saved minutes. Only landings made after the change landed count.
If the first-landing stage does not fall, the review says the landing wait was not stacking and says so in its
record.

## 6. Alternatives and failure cases

| Option | Why not |
| --- | --- |
| Measure from the first landing record only, with no landing change (`agent-and-build-slots.md` section 6.2, build job 2: `.min()` in `landing_record_time`) | It removes the measurement effect but changes no landing behavior, so the stack-replay and conflict costs stay. It also changes the stage definition, so earlier reviews are not comparable. Kept as a **companion**: the review reports both stages (section 5) without changing `landing_end` here. |
| Keep stacking, squash on merge | The squash exists already; the pull request still carries earlier commits, and the squash forces `resync_onto_merge`. The cause stays. |
| Rebase the whole stack more often | `plan_rebase` already does it at each landing; cost grows with the stack. |
| Chain pull requests, each based on the previous job's branch | Each still carries earlier jobs until they merge, and an early failure blocks all later ones. The stack under another name. |
| Land the stack as one squash pull request (`squash-chain-landing.md`) | One failing job holds all, and the chain rule is weaker than the single-job rule. |
| Raise `[parallel] max_concurrency` only | Does not touch what a pull request carries. |
| Do nothing | Leaves the headline inflated and hides the real wait. |

**Failure cases.**

- **Merge conflict.** The cut conflicts with current main. That job alone is held through the existing conflict path
  (`max_conflict_returns`); nothing else changes. A test covers a conflict with main and a conflict only with an
  unlanded earlier job.
- **Main moved.** Main advances while the pull request waits. The cut is repeated on the new main and the head is
  replaced: one extra merge, not a stack. Real wait may rise by that merge.
- **Verified content differs.** The cut tree may differ from the verified tree. The existing re-check rule and the
  merged-content check apply; a cut with a different governed fingerprint is checked again before it lands.
- **Dependent job.** A job that edits lines an unlanded earlier job added cannot be cut from main yet. It waits behind
  its predecessor in the ordered queue. This is the one real wait that stays; how often it happens is not known
  (question below).
- **Existing stacked branches.** Branches built before the change are drained by the old path.

## 7. Citation duty

A change that implements this design must cite throughput review `20261109-014617-5cg0` in its request text. This
supersedes the citation of `20261108-183648-5eya` named in `per-job-landing-branches.md`; that file now points here.
The next review scores the change by section 5.

A later design, `overlap-ci-waits.md` (throughput review `20261110-191047-6szm`), pushes the candidate early for CI.
That push is a candidate-only work branch; the landing cut in section 3 is still the only commit `land_pr` accepts.

## 8. Out of scope and build jobs

Out of scope: any change to `crates/`, `xtask/`, `athena.toml`, `defaults.toml`, the workflows, `THROUGHPUT.md` or
`records/`; changing how `landing_end` measures the stage (only proposed above); the repository's merge method.

Ordered build jobs, each one focused change with tests:

1. **Measure.** Record the commits per landed pull request, the landing records listing each job, and the time per
   phase of the first-landing stage. No behavior change. Depends on: nothing.
2. **Cut a job's commit from current main.** Depends on: 1. Tests: clean cut, conflict with main, dependency on an
   unlanded job, re-check after a changed fingerprint.
3. **Use the cut in `land_pr` and `resync_onto_merge`.** Depends on: 2. Test: the pull request holds one commit.
4. **Remove the stack replay and records-only fallback** once no stacked branch remains. Depends on: 3.

## Questions for the user

- Do you want the stage reported from the first landing record (`.min()`), as `agent-and-build-slots.md` build job 2
  proposes, instead of only reporting it beside the headline?
- How often does a job depend on an earlier unlanded job's code? No record in the snapshot shows it; build job 1
  would count it.
