# Review 2 of `docs/design/native-pr-landing.md`

**Reviewed:** the design at status "proposed (D72)", plus the source and documents it cites.
**Why "review 2":** the design's first review was rejected, and no first review file for this design exists in
the tree. This review is independent: it was written without reading the rejected one. The name `-2` keeps the
numbering free of a clash if the first one is ever added.
**Method:** reading only. Claims are labeled **(read)** when taken from a file and lines named beside them,
**(design)** when they are what the design proposes, **(reported)** when they come from evidence given to this
review and not reproduced by it, **unchecked** when not read, and **estimate** when not measured. No command
was run to reproduce any behavior. Line numbers are those of the repository at the time of review.

## Verdict: reject for revision

The recommended shape (alternative B: Athena drives the user's own host client, never forces a push, never
merges on its own) is sound and matches the requirements it cites. It should not be built from this text,
for four reasons:

1. Its account of what survives a host squash is wrong for the Athena-Job trailer (F1).
2. It silently assumes one job per pull request and gives no rule for a squash that lands several (F2).
3. It does not say whether it supplies what the parallel-jobs review (F1 there) asks to come first, and the
   roadmap lists the two in the opposite order (F3, F4).
4. Its stealth rule "no trailers" is contradicted by what Athena's landing commit writes today (F5).

The design is not revised by this review. Fixing it is a separate job.

## Severity scale

**High** blocks the build or makes a claim in the design false. **Medium** must be settled in the design or in
the first build job. **Low** is a correction or a later job.

## 1. Evidence used

| Evidence | Source | Status |
|---|---|---|
| E1. On 2026-10-13 a GitHub squash merge appended a `Co-authored-by` paragraph after the commit message's `Athena-Job` trailer block, so the `Athena-Job` line is no longer in the last paragraph. | Given to this review by the user as that day's observation of a real squash merge. | **(reported)**, observed 2026-10-13 by the user on the host's merge button; **not reproduced here**. How it was observed beyond that (which pull request, which message setting) was not given, so the exact squash message layout is **unchecked**. |
| E2. A squash message may name only one landed job. | Same source, same date. | **(reported)**, not reproduced. Whether the limit is the host's message building or the way Athena writes branch commits was not given. |
| E3. Git's trailer parser reads the trailers of the last paragraph of a message. | `Git::trailers` runs `git interpret-trailers --parse` on the message, `crates/athena-git/src/lib.rs` 353-365 **(read)**. That the parser takes only the last paragraph is git's documented behavior, which I did not run here. | **unchecked** in this repository. |

E1 is the reason F1 exists. Nothing below treats E1 or E2 as measured.

## 2. Findings

### F1 (High): the design says trailers survive a merge; a squash with a `Co-authored-by` paragraph hides them

**Concerns:** §5.3 "M7", line 134-136 ("the trailers and the content survive"); §5.3 "Trailers", line 142;
§5.1 step 5, lines 105-108.

The design says that after a rebase merge "the trailers and the content survive", and for a squash it defers to
the chain rule (M30). It never says how a trailer is *found* in the merged commit. Reading the code, there are
three places that look at trailers, and E1 treats them differently:

1. **The gate's landing check does not use trailers to find jobs.** `landing` documents that "the jobs are
   identified by the closing records the commit adds, never by trailers" and that trailers "are hints that
   must agree with the records" (`crates/athena-gate/src/invariant.rs` 267-272 **(read)**). Its trailer
   checks (297-303 **(read)**) only refuse when a trailer is *present and wrong*. Hidden trailers therefore do
   not make a valid landing fail, and also do not make a wrong one pass. This is the safe direction. **(read)**
2. **The core's retry path** (`run.rs` 1370-1383 **(read)**) looks for the blocked closing commit with
   `log_grep("HEAD", "^Athena-Job: <id>$")` and then takes the trailers of that commit's message, requiring
   `Athena-Prediction`. A line-anchored grep matches in any paragraph, but `Git::trailers` is then called on the
   same message. Under E1 the `Athena-Job` line would be found by the grep and the `Athena-Prediction` line
   would not be parsed, so the commit is found and then skipped, ending in "was not found" (1381-1383). This
   only matters if a *squashed* commit can be a blocked closing commit. In today's flow it cannot (the blocked
   closing commit is Athena's own, made by plumbing), so this is a latent risk for the design, not a bug today.
   **(read; the squash case is a reading of E1 and E3, not run)**
3. **`status.rs` 332 greps `^Athena-Job: <id>$`** **(read)**: also paragraph-independent, so it still finds the
   line under E1. Whether it then also depends on trailer parsing was not read in full (**unchecked**).

What the design needs, then:

- Delete the claim that trailers "survive" a host merge. A rebase merge keeps messages, but a squash builds a
  new message from the host's template, and E1 shows it can append a paragraph after the trailer block.
- State the rule: **Athena identifies a landed job from its records, never from a trailer**, which is already
  true in the gate (`invariant.rs` 267-268). Say it again in §5.1 step 5 and §5.3, because step 5 says Athena
  asks the host for the merged commit and then "checks the landing rule", and a reader will assume the
  trailer drives the lookup.
- Any code the build adds to find the merged commit for a job must not parse trailers from the *merged*
  commit. Use the pull request's own record of its head commit and the host's merged commit id, then check the
  content with the records (design §5.1 step 5 already says "asking the host for the merged commit"; keep it
  that way, and say so).
- Add a test with a hand-built commit whose message ends in a `Co-authored-by` paragraph after the Athena
  trailers, and assert that the landing check still decides from records. This test is cheap and would fail
  if someone later moved the gate to trailer-first lookup.

### F2 (High): one pull request is assumed to hold one job; a squash may name only one

**Concerns:** §5.1 step 1 (work branch "derived from the job id", line 95-96), §5.1 steps 2-5, §5.3 "M7"
(line 136, "a squash is judged by the chain rule (M30)"), §3 owned mode line 45-46.

The flow is written for one job: one branch name from one job id, one pull request body "generated from the
job" (line 100-102), one "merged commit" found for it (line 105). The design never asks what happens when a
pull request holds more than one job. E2 says a squash can name only one landed job. Consequences:

- **Chain rule (M30).** The chain rule is built for exactly this shape: several jobs folded into one squash.
  Its gate finds every job from the closing records the commit adds, so a squash that names one job still
  *lands* all of them if the chain is valid (`invariant.rs` 225-252, 277-: jobs come from records **(read)**).
  So E2 does not break the *check*. It breaks two other things:
  1. **Naming.** The message's trailer names one job. The trailer rule refuses when `Athena-Job` names a
     request job the commit does not land (`invariant.rs` 297-303 **(read)**), so a squash that names a job
     outside the chain is refused. If the host names the *first* or *last* job, both are inside the chain, so
     it passes; this relies on the host naming a member, which nothing in the design guarantees.
  2. **Bookkeeping in stealth mode.** §5.4 keeps "the merged commit id next to the job id" in the state
     directory (lines 156-158). A squash of three jobs needs one commit id next to **three** job ids. The
     design describes one-to-one.
- **The chain's own open problem.** The squash-chain design's second review (R2-M1,
  `docs/design/reviews/squash-chain-landing-review-2.md`, §"R2-M1") found that nothing says where
  `previous_job` comes from **(read)**. A pull request that holds jobs 1 to 3 is the case where the core has
  commits but no landing to read a predecessor from. This design inherits that gap, and should not claim M30
  makes its squash case safe until the gap is closed. I did not check whether it has been closed since (the
  squash-chain design file was not re-read in this review: **unchecked**).
- **Pull requests of several jobs, other merge methods.** A merge commit keeps each job's commit, so the
  single-job rule sees each job (design line 134). A rebase merge also keeps one commit per job. Only a squash
  collapses them. The design says none of this, and the answer for the user is "pick merge or rebase for
  multi-job pull requests, or accept a chain squash". That choice belongs in the text.

**Decision needed (revision):** either (a) state that Athena opens **one pull request per job** and that a
user who combines jobs in one pull request and squashes relies on M30, with the limits above; or (b) design the
multi-job pull request (branch naming, body listing every job, the merged-commit-to-jobs mapping as one-to-many).
Option (a) is the smaller change and agrees with the per-job pull requests that the parallel-jobs review
recommends (see F3).

### F3 (High): the roadmap runs the parallel pool before this design, against parallel-jobs review F1

**Concerns:** whole design; §5.1 step 6 (line 109-111); `docs/design/roadmap.md` 33-38.

The parallel-jobs review's finding F1 says, for the local-landing pool, that almost every parallel change job
after the first ends stale (`docs/design/reviews/parallel-jobs-review-1.md`, F1 **(read)**). Its decision is
that the pool must not be the first build with the promise that independent jobs land, and that either (a) a
budgeted "re-verify stale job on the new `HEAD`" option, or (b) "per-job branches and pull requests ... come
first". Its table in §5 repeats it: "reorder §10 so re-verification or per-job pull requests precede or
accompany the pool".

The roadmap, at the base commit, lists *Parallel jobs* as item 2 and *Native pull-request landing* as item 3
of RM-0 (`docs/design/roadmap.md` 33-38 **(read)**). That is the order F1 asks to avoid. The request to this job
says the same: this design "must come before the parallel pool". This review agrees, and records that **the
roadmap's text does not yet reflect it**; changing the roadmap is not in this job.

Placement is not enough by itself. F1 offers two ways to make parallel jobs land, and this design supplies
neither completely:

- It is (b) only in part: one pull request per job is the shape F1 names, but this document is written around a
  single job at a time and never says it is the answer to the pool's staleness (F2).
- It is not (a): §5.1 step 6 says a job made stale by a moved target is *asked again* and that "re-verification
  stays as D10 (d) left it: not automatic" (line 109-111 **(read)**). That is the very outcome F1 calls the
  common case for parallel jobs. Per-job pull requests do not remove it; they move it to the host.

So the ordering is a **condition with a gap**: the pool may follow this design only if the revised design says
how a second job whose base is no longer the target's tip proceeds. I do not know which answer the user wants
(re-verify on the new tip, or a pull request that the host merges and Athena then re-checks), so that is a
question in §4, not a decision made here.

### F4 (Medium): two pull requests merged by the host produce content nobody verified

**Concerns:** §5.1 step 5 (merge observed); §5.3 "M7"; R2 in §2.

R2 says "the pushed content is the verified content" and checks it before the push (lines 33, 160-162). That
holds for one job at one base. The host, however, can merge two pull requests from the same base cleanly when
their files are disjoint. The merged tree then equals neither job's verified fingerprint. The gate's landing
check compares the landed content with the last job's fingerprint (`invariant.rs` 277-: "only the last has to
hold the landed content" **(read)**), and `plan` bails when landed content differs from verified
(`land.rs` 74-80, as described in the parallel-jobs review F3 **(read, second hand)**). So in owned mode the
second merge would be *detected*, not prevented: the design's own phrase for the host (line 137-138, "detected
by the history check in CI, not prevented"). In stealth mode there is no CI audit at all (line 163-165), so
nothing detects it. Say so in §5.4 under "What is given up", and decide whether Athena should check, before it
reports "landed", that the merged commit's content equals the verified fingerprint (a cheap mechanism the
design does not currently list in §8).

### F5 (Medium): stealth says "no trailers", but today's landing commit adds a `Co-Authored-By` trailer

**Concerns:** §5.4 "Trailers" and "Commit identity" (lines 155-160); §5.4 "Leak checks" (166-168).

The landing message built today carries a `Co-Authored-By` trailer for the model (`run.rs` 1107-1115 and
1398-1403 **(read)**; `lifecycle.rs` 124 asserts a model name in it **(read)**). §5.4 says stealth commits
carry "none by default" and are "the user's" message. The leak check in §5.4 compares "any ... trailer" the
design calls *Athena-specific*, and does not say whether `Co-Authored-By` is one. A `Co-Authored-By` line naming
a model announces the tool exactly as an `Athena-Job` line does. Separately, E1 shows the **host** can also add
a `Co-authored-by` paragraph on squash, for the pull request's other authors; that one is the host's and
cannot be a leak check failure. Fix: state that stealth mode writes no `Co-Authored-By` for the model unless
the user asks, and that the leak check must tell Athena's own such line from one the host adds on merge.

### F6 (Medium): the merged-commit lookup has no stated method, and the host is the only source

**Concerns:** §5.1 step 5; §6 "Host differences"; §8 last row ("Remote host behaves as expected" is prose-only).

Step 5 says Athena asks the host for the merged commit, but not how it ties a pull request to a job without a
trailer (F1) or, in stealth mode, without any marker (§5.4). The natural answer is that Athena records the
pull request number when it opens it. The design does not say so. Add it to §5.1 step 3 as part of what is
stored, in both modes. Without it, the "merge observed" step has nothing to look up.

### F7 (Low): line references and cross-document claims need a re-check at revision

- §1 cites `BRIEF.md` lines 459-466 (line 16). That file is not in this review's working tree, so I could not
  check it (**unchecked**).
- §4 D cites `design/reviews/athena-v0-review-1.md` "near its line 626"; §5.3 cites `design/athena-v0.md` "near
  its line 1447". I did not open either (**unchecked**).
- §5.3 says M23 is checked from recorded times with no trailer or ancestry, from `docs/enforcement.md` row
  M23. I did not re-read the row, but `invariant.rs` 267-272 says the same of prediction proofs **(read)**.

### F8 (Low): "Open questions" omit the ones this review raises

§9 lists four questions. None covers one-job-per-pull-request, the host-merged content check, or the
re-verification gap. See §4.

## 3. Placement against the parallel pool

| Check | Result | Evidence |
|---|---|---|
| Does `parallel-jobs-review-1.md` F1 say per-job pull requests (or re-verification) come before the pool? | Yes | F1 "Decision": options (a) and (b); §5 table row F1 **(read)** |
| Does the roadmap place this design before the pool? | No: pool is item 2, this design is item 3 | `docs/design/roadmap.md` 33-38 **(read)** |
| Does this design, as written, give the pool what F1 asks? | Not yet | F3 above: single-job; re-verification stays manual (§5.1 step 6) |
| Does this review agree it must come first? | Yes, as a condition | F3 |

## 4. Questions for the user (decisions this review cannot make)

1. For a job whose target moved, should Athena re-verify on the new tip when the user asks (parallel-jobs
   review F1 option a), or leave it stale and re-propose, as D10 (d) and §5.1 step 6 say?
2. One pull request per job only, or also several jobs in one pull request (F2)?
3. Should Athena check that a host-merged commit holds the verified content before it reports "landed" (F4)?
4. In stealth mode, is a `Co-Authored-By` line for the model ever wanted (F5)?

The four open questions in §9 of the design are not answered here; they need the user.

## 5. What can be fixed in the design, and what belongs to later jobs

| Finding | Fix in the design | Later job |
|---|---|---|
| F1 | Correct "trailers survive"; state records-not-trailers; name the test | Squash test with a `Co-authored-by` tail |
| F2 | Choose (a) or (b) | Multi-job pull request, if (b) |
| F3 | State whether the design supplies (a) or (b) for the pool; the roadmap order is changed separately | Roadmap edit |
| F4 | Add to "What is given up" and §8 | Merged-content check |
| F5 | One rule for `Co-Authored-By` in stealth | Leak check |
| F6 | Store the pull request number | Lookup |
| F7, F8 | Re-check references; extend §9 | None |

## 6. Limits of this review

I did not run code or the host, did not reproduce E1 or E2, and read the gate only where cited. I did not read
the rejected first review (none is in the tree), `design/parallel-jobs.md` itself, `BRIEF.md`, or
`squash-chain-landing.md` in full. The squash message layout of E1 comes from the user's report. Counts in this
review (three trailer-reading places, four questions) were counted by reading, not measured by a tool. The list
of trailer-reading places in F1 came from a search for `Athena-Job` and `trailers(` in `crates/` and may be
incomplete.
