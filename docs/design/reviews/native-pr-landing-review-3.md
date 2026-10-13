# Review 3 of `docs/design/native-pr-landing.md`

**Reviewed:** the design as revised after review 2 (status line: "revised after review 2 ... D85", line 3-4),
plus the source and documents it cites.
**Independence:** this review was written independently. I formed the verdict from the revised design and the
code, and used review 2 only as the list of findings to check (F1-F8). I did not read any other review of this
design, and I did not take review 2's verdict as given.
**Method:** reading only. Claims are labeled **(read)** when taken from a file and lines named beside them,
**(design)** when they are what the design proposes, **(reported)** when they come from evidence given to the
design and not reproduced, **unchecked** when not read, and **estimate** when not measured. No command was run
to reproduce any behavior. Line numbers are those of the repository at the time of review.

## Verdict: accept with changes

Review 2's eight findings are resolved in the text: seven fully, F7 in part (§1 below). The recommended shape
(alternative B: Athena drives the user's own host client, never forces a push, never merges on its own) is
still sound. The revision introduced no contradiction with the gate code I read. I found no High finding.
The design can go to a build job once the five new findings are settled in the text or the first build job:
four Medium (N1-N4) and one Low (N5). N1 and N2 are the ones most likely to change the build.

The design is not edited by this review. Fixing it is a separate job.

## Severity scale

**High** blocks the build or makes a claim in the design false. **Medium** must be settled in the design or in
the first build job. **Low** is a correction or a later job. (Same scale as review 2.)

## 1. Evidence used

| Evidence | Source | Status |
|---|---|---|
| E1. A host squash appended a `Co-authored-by` paragraph after the `Athena-Job` trailer block. | Design lines 18-22, attributed to the user. | **(reported)**, not reproduced here; exact layout **unchecked**. |
| E2. A squash message may name only one landed job. | Design lines 20-22. | **(reported)**, not reproduced. |
| E3. The gate's landing check finds jobs from closing records, and treats trailers as hints that must agree. | `crates/athena-gate/src/invariant.rs` 267-272, 286-314 **(read)**; closing records collected at 225-252 **(read)**. | **(read)** |
| E4. The core's retry path greps `^Athena-Job: <id>$` and then needs `Athena-Prediction` in the parsed trailers. | `crates/athena-core/src/run.rs` 1371-1383 **(read)**. | **(read)** |
| E5. The landing message carries a model `Co-Authored-By` today. | `run.rs` 1113 and 1401 **(read)**; `crates/athena-cli/tests/lifecycle.rs` 124 **(read)**. | **(read)** |
| E6. `previous_job` is derived by the core as the latest landed job in the base's history, absent when none. | `docs/design/squash-chain-landing.md` 135-149 and the R2-M1 row at 377 **(read)**. Whether the build does this was **not checked**. | **(read)** for the text; build **unchecked** |
| E7. Roadmap order. | `docs/design/roadmap.md` 33-38: parallel jobs is item 2, native pull-request landing item 3 **(read)**. | **(read)** |

## 2. Review 2's findings, checked against the revision

| Review 2 | Resolved? | Where in the revised design | What I read |
|---|---|---|---|
| F1 (High): trailers said to survive a merge | **Resolved** | §5.1 step 5, lines 123-131; §5.3 "M7", 156-165; §5.3 "Trailers", 169-171; §5.3 "Retry path", 173-176; §8 row at 303 | The word "survive" now appears only in the negative ("No part of this relies on the job trailers surviving", 159-160). The rule "identified from records, never from trailers" is stated at 125-128 and 169-171. The gate citations match the code (E3). The test is named at line 303 with a hand-built `Co-authored-by` tail. The retry-path caveat at 173-176 matches `run.rs` (E4). |
| F2 (High): one job per pull request assumed | **Resolved** | §5.6, 230-256; §9 item 6, 325-326 | One pull request per job is chosen and listed as an assumption. The squash-of-several case relies on M30 with stated limits. The trailer-naming rule is cited at `invariant.rs` 297-303, which matches the code (read). The many-to-one stealth bookkeeping is stated (245-247). |
| F3 (High): roadmap order and the parallel pool | **Resolved in the design; roadmap still open** | §5.7, 258-275; §5.1 step 6, 132-134; §9 item 5, 322-324 | The design says it supplies option (b) only and not automatic re-verification (a), and says how a stale job proceeds. It says the roadmap order "is to be changed in a separate job" (273-275). The roadmap still lists the pool first (E7), so the ordering condition is not yet met. That is outside this design's text, and the design says so. |
| F4 (Medium): host-merged content nobody verified | **Resolved** | §5.4, 204-212; §8 row at 304 | The risk is stated for owned mode (detected, not prevented) and stealth mode (nothing detects it), a pre-report check is added, and a test is named (two disjoint-file pull requests). See N1 for how well the check is defined. |
| F5 (Medium): stealth and `Co-Authored-By` | **Resolved** | §5.4, 193-195 and 213-219; §8 row at 302; §9 item 8 | Stealth writes no model `Co-Authored-By` unless asked. The leak check separates Athena's own line from one the host adds, and a test for the host-only case is named. Matches E5. |
| F6 (Medium): merged-commit lookup | **Resolved in the text; see N2** | §5.1 step 3, 116-120; §8 row at 305 | The pull request number is stored in both modes, with host, repository and pushed head commit id. N2 asks where owned mode can store it. |
| F7 (Low): references | **Partly resolved** | §10, 332-341 | I re-checked the ones I could. `BRIEF.md` 459-461 and 463-466 match (read; the stealth quote is at 466). `design/athena-v0.md` 1446-1449 matches (read). `design/reviews/athena-v0-review-1.md` 626-627 matches (read). `docs/enforcement.md` row M23 says "no trailer or ancestry" (read). The `run.rs` and `invariant.rs` line numbers are marked unchecked in §10 (line 341); I spot-checked `invariant.rs` 225-252, 267-272, 297-303 and `run.rs` 1113, 1373, 1401, and they are right (read). The design's own marker is honest; it is simply not closed by the author. |
| F8 (Low): open questions | **Resolved** | §9 items 5-8, 320-330 | All four of review 2's §4 questions are there, each with the assumption taken. |

Two smaller notes on the revision as a document:

- Line 4 still says "Not yet re-reviewed". This review is that re-review; whoever edits the design next should
  change the line.
- D85 in `docs/decisions.md` (1861-1877) describes the revision in the same terms as the design's revision note
  (read). I found no mismatch.

## 3. New findings

### N1 (Medium): the merged-content check is not defined well enough to build

**Concerns:** §5.4 "Merged-content check", lines 209-212; §5.6 lines 245-247; §5.7 point 2, lines 267-269;
§8 row at 304.

The check "compares its tree (or, for a pull request whose base moved, the diff the pull request brought) with
the verified fingerprint". Three gaps, all **(design)** readings:

1. A tree can be compared with a fingerprint of content. A *diff* cannot, without a stated way to turn it into
   one. The text does not say which fingerprint the diff is compared with, or how it is computed. I did not read
   how the fingerprint is defined in this review (**unchecked**), so I only claim the design does not say.
2. §5.7 point 2 says that a clean merge from a moved tip "gives content nobody verified, so that check will
   often refuse". Read together with point 1, the check as written refuses *every* squash or merge from a
   moved base whose tree differs from the verified one, and the diff branch is the only exception. The design
   should say which of the two it means, because "often refuse" is a description of the common parallel case,
   not an edge case.
3. When the check refuses, the content is already on the shared branch (the host merged it). The design says
   "the job is not recorded as landed, and the user is told" (211-212). It does not say what Athena does next
   with a job that is merged but not "landed", for example whether it stays stale forever or can be
   re-verified on the new tip. §5.7 point 3 covers re-verification of an *unmerged* stale job only.

Fix in the design: state the comparison exactly (tree against fingerprint, or diff against what), say that a
refusal leaves a merged job in a named state, and say how the user moves it out of that state.

### N2 (Medium): owned mode cannot store the pull request number in the job's records at the time the design says

**Concerns:** §5.1 step 3, lines 116-120 ("in owned mode in the job's records"); §5.3 "M6", 153-155; §5.1 step 5.

In owned mode the records of a job are committed on the work branch and pushed (step 2). The pull request
number and the pushed head commit id exist only after the push and the open (step 3). A commit cannot hold its
own id, and the number is unknown when the closing records are written, so these two values cannot be in the
records that the pull request itself carries. They would need a later record, committed after the pull request
exists. The design does not say where that later record goes: a new commit on the pull request branch changes
the head the number was stored against; a commit after the merge needs a path to the base branch that is not
a pull request. Records are append-only and checked by M4 and M5 (design 183-184, 306), so a new file must
also fit the gate. I did not read the records schema for a field that already holds this (**unchecked**).
Stealth mode has no such problem, because the state directory is outside the repository.

Fix in the design: say where the number lives in owned mode (for example, the state directory in both modes,
with the repository records unchanged), or describe the extra record and its path through the gate.

### N3 (Medium): the stealth pull request body and branch name can name the job

**Concerns:** §5.1 steps 1-3, lines 109-120; §5.2, 140-141; §5.4 "Trailers" and "Leak checks", 189-192 and
213-219; §6 "Stealth leaks", 283-285; §8 rows at 301 and 302.

The design reasons that a trailer should be absent in stealth because it "names a job whose records a reader
cannot find in a third-party repository, and it would announce the tool" (189-191). Two other places carry the
same information, and the design handles them differently:

- **The body.** Step 3 says the body contains "the request, the criteria, the verifier's findings, and the job
  id", and adds only that in stealth mode it has "no link to a record file" (116-118). It does not say the job id
  is dropped in stealth. The leak check (213-219) is written over "any path or message line" and "the message
  it drafts"; it does not say that it also covers the pull request body, the criteria text and the verifier's
  findings, which are Athena vocabulary even without the id.
- **The branch name.** Step 1 derives the work branch name from the job id (110-111), §5.2 restricts pushes to
  that branch (140-141), and §6 says the stealth branch name should instead be neutral, "a short name from the
  change, not the job id" (283-285). §8 leaves the neutrality of the name as prose-only (301). So the
  mechanism the design requires (push only the job's work branch, line 298) and the stealth rule (neutral name)
  are not reconciled: it does not say that in stealth the work branch itself has the neutral name, or that
  Athena pushes a differently named ref.

Fix in the design: say what the stealth body contains and drops, extend the leak check to the body, and say
that in stealth the pushed branch name is the neutral one (and how the restriction to "the job's own work
branch" is then checked).

### N4 (Medium): the many-to-one check is judged on the last job only, with no word on the earlier ones

**Concerns:** §5.6 lines 245-247; §5.4 "Merged-content check"; §8 row at 304.

For a squash of several jobs the design says the content check "applies to the last job's fingerprint, as the
gate's does" (246-247). The gate's rule is the same **(read, `invariant.rs` 274-276: "only the last has to
hold the landed content")**. In owned mode the gate also runs each earlier job through its per-job checks
(same lines), so the earlier jobs are not unchecked there. In stealth mode the design has no gate over history
(201-203). So in stealth, for a squash of three jobs, the only guard is one comparison against the last job's
fingerprint, and the earlier two jobs are recorded as landed with no check at all. The design should either say
that this is accepted (it fits "more safety than M30 gives" at 253-254, but that sentence is about the chain
inheriting a gap, not about stealth), or say which per-job checks Athena runs over the local store before it
records several jobs as landed. This is a gap in the written promise, not a code defect: the stealth chain
check is not built.

### N5 (Low): wording that is easy to misread

- Line 3-4 says "findings F1-F8" in a parenthesis that points at `design/reviews/native-pr-landing-review-2.md`
  with a path relative to `docs/`; the other paths in §10 use the same style, but §1 uses `docs/...` style for
  `docs/enforcement.md`. A reader following the paths has to guess the base. A short sentence on path style
  would remove the guess.
- §5.1 step 5 says Athena "does not search the base branch's log for the job id" (126-127). The retry path in
  `run.rs` 1371-1383 does exactly that for the blocked closing commit (read), and §5.3 explains why that is
  safe today (173-176). The two statements are consistent, but step 5 should say "to find the merge" once
  more, or refer to §5.3, so a reader does not take it for a ban that the existing core code breaks.
- §9 item 3 and §8's "Merge is the user's decision" row (307) both stay prose-only. That is coherent, but
  §6 still says nothing about what Athena does if the user has turned on host auto-merge while a stale check
  is in progress. This is a later job, not a defect here.

## 4. Placement against the parallel pool

| Check | Result | Evidence |
|---|---|---|
| Does the revised design say what it supplies for the pool? | Yes: per-job pull requests (option b), no automatic re-verification | Design 258-271 **(design)** |
| Does the roadmap now place this design before the pool? | No: pool is item 2, this design item 3 | `docs/design/roadmap.md` 33-38 **(read)** |
| Is the roadmap change a condition for building the pool? | Yes, and the design says it is a separate job | Design 273-275 **(design)** |

N1 matters here: if the merged-content check refuses most merges from a moved base (N1 point 2), then
per-job pull requests do not make independent jobs "land" in Athena's accounting, only in the host's. The design
admits this in §5.7 ("does not claim to make every parallel job land"); it should say it plainly in the
summary of §5.7 point 2.

## 5. Questions for the user

None new. The assumptions for §9 items 5-8 are the design's; this review does not change them. N1 point 3
(what state a merged-but-refused job is in) and N2 (where the number lives in owned mode) are design choices
that the user may want to make; the smaller option for each is stated in the finding.

## 6. What can be fixed in the design, and what belongs to later jobs

| Finding | Fix in the design | Later job |
|---|---|---|
| N1 | State the comparison, the refused-job state and its exit | The merged-content check |
| N2 | State where the pull request number lives in owned mode | None, or a record schema change |
| N3 | Define the stealth body, extend the leak check, reconcile the branch name | Leak-check tests |
| N4 | Say which per-job checks run in stealth for a multi-job squash | Stealth consistency checks |
| N5 | Wording | None |
| Roadmap order (F3, still open) | None in this design | Roadmap edit |

## 7. Limits of this review

I did not run code or the host, and did not reproduce E1 or E2. I read the gate only at the cited lines
(`invariant.rs` 222-316) and `run.rs` at 1368-1385; I did not re-read `status.rs` 332 or the fingerprint
definition. I did not check whether the build matches `squash-chain-landing.md` on `previous_job` (E6). I did
not read `design/parallel-jobs.md` or the parallel-jobs review. Counts here (eight findings checked, five new)
were counted by reading, not by a tool. The N-findings are readings of the design text, not tests.
