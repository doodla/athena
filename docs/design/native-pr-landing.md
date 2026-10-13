# Native pull-request landing, including repositories Athena does not own

**Status:** proposed (D72); revised after review 2 (`design/reviews/native-pr-landing-review-2.md`, findings
F1-F8; revision recorded in D85). Design only: no code is built by this document. Not yet re-reviewed.

Claims are marked **(read)** when taken from the file named beside them, **(design)** when they are
proposals, and **(reported)** when they come from a user's observation that was not reproduced. No command
was run to reproduce any behavior.

**Revision note (after review 2).** What changed, by finding: F1 trailers are no longer said to survive a host
merge; a landed job is identified from records (§5.1 step 5, §5.3, test in §8). F2 one pull request per job
is the stated choice (§5.6). F3 the design says what it supplies for the parallel pool and how a stale job
proceeds (§5.1 step 6, §5.7). F4 the unverified-merge risk is recorded (§5.4) with a pre-report content check
(§8). F5 stealth writes no model `Co-Authored-By` unless asked (§5.4). F6 the pull request number is stored
in both modes (§5.1 step 3). F7 line references re-checked (§10). F8 §9 gains the four questions of review 2
§4, each with the assumption taken.

**Evidence from the user, not reproduced.** (E1, reported 2026-10-13) A GitHub squash merge appended a
`Co-authored-by` paragraph after the `Athena-Job` trailer block, so that line was no longer in the last
paragraph; the exact message layout and settings are unchecked. (E2, reported) A squash message may name only
one landed job; whether the limit is the host's or Athena's branch commits is unchecked. Neither was
reproduced here; the design is built to hold whether or not they are exact.

## 1. The problem

Today Athena lands a verified job by advancing a local branch with `merge --ff-only` after an in-process gate
check (D10, `docs/decisions.md`) **(read)**. Getting that commit into a shared branch is then done by hand:
push, open a pull request, wait for checks, merge. D10 lists "Pull requests" as option (c) and left it out
as "unavailable here and adds a step" **(read)**. The prose-only list in `docs/enforcement.md` says "Pushes
are regular, and published history is never rewritten: Athena does not push in v0" **(read)**.

The brief's 2026-10-12 addenda change the weighting **(read, `BRIEF.md` lines 459-466)**:

1. *Athena does the work its user should not have to.* "I feel like Athena should natively handle PRs."
2. *Brownfield integration is first-class.* Adding Athena to an existing codebase must not be blocked by the
   need to track the whole history.
3. *Stealth.* "If I'm contributing to a third-party repo and I'm making a PR, I can't dump a bunch of Athena
   stuff in there."

Requirement 3 collides with how landing works now. The gate's rules (M4 to M7, M23, M30) read Athena's
records from `records/` inside the repository, and a landing commit carries the job's closing records
**(read, `docs/enforcement.md`)**. A pull request made that way adds `records/` files to someone else's
project.

## 2. Requirements

| # | Requirement | Source |
|---|---|---|
| R1 | One command takes a verified job to an open pull request, and later to a merge, with no manual git. | addendum "does the work its user should not have to" |
| R2 | Nothing reaches a shared branch that was not verified: the pushed content is the verified content. | M7 |
| R3 | The prediction still precedes the outcome. | M23 |
| R4 | Published history is never rewritten by Athena. | `docs/enforcement.md`, prose-only list |
| R5 | In a repository the user does not own, the pull request holds only the user's change: no Athena records, attributes, hooks or configuration. | stealth addendum |
| R6 | Works on an existing repository with no Athena history and no prior bootstrap. | brownfield addendum |
| R7 | Squash, merge and rebase merges chosen by the host do not break Athena's own accounting. | `design/squash-chain-landing.md` |

## 3. Two modes

The repository's relationship to Athena decides what a pull request may contain.

- **Owned mode** (Athena's records live in the repository, as today). A pull request carries code and
  records. The existing gate applies on the branch and, after a squash, by the chain rule of D69 (M30).
- **Stealth mode** **(design)**. Athena's records for the job live outside the repository, in Athena's own
  state directory, and the pull request carries only the change. The records are the same files with the same
  schema; only where they are stored differs.

The mode is a per-repository setting that the user chooses; the default for a repository with no `records/`
directory is stealth.

## 4. Alternatives

**A. Keep landing local; document the manual push (status quo).**
- For: no new trust boundary; Athena holds no credentials.
- Against: fails R1. Leaves a manual step on every job. For stealth work it leaves the user to strip
  `records/` by hand, which is the failure the addendum describes.
- Rejected.

**B. Athena pushes the landed branch and opens the pull request through the host's command-line client
(recommended).**
- Athena runs the user's already-authenticated host client (for GitHub, `gh`) as a child process. It never
  reads, stores or records a credential, the same posture as for signing keys in D10.
- For: smallest new surface; uses the user's existing permissions; works with forks, which a third-party
  contribution needs.
- Against: depends on a client being installed. The host's merge button is outside Athena's control.
- Recommended, with the details in section 5.

**C. Athena calls the host's web API directly with its own token.**
- For: no dependency on a client.
- Against: Athena would hold a long-lived credential, a new secret class for M1 and M22 to cover; one API per
  host. Not needed to meet any requirement.
- Rejected.

**D. Commit the records on a separate branch or side ref and keep the pull request clean.**
- For: keeps stealth without a separate store.
- Against: the records would still be pushed to the third-party repository (a side ref is visible), which
  breaks R5. The v0 design already decided that side refs are not fetched or pushed
  (`design/reviews/athena-v0-review-1.md`, lines 626-627) **(read)**.
- Rejected for stealth. It remains a possible variant of owned mode.

**E. Store the records in a second repository the user owns.**
- For: durable, shareable, keeps the third-party repository clean.
- Against: a second remote to configure and keep in sync; more moving parts than a local state directory.
- Deferred: a later option for people who want records to survive the machine.

## 5. Recommended design (B)

### 5.1 The flow, from verified job to merge

1. **Verified job.** As today: the job is dispatched, produced, verified, and the candidate passes the
   landing checks of D10 (ancestry, classification stability, fingerprint). The candidate is built by
   plumbing on a local work branch whose name is derived from the job id **(design)**.
2. **Publish.** `athena land --pr` **(design; the name is not fixed)** pushes that branch to the user's
   remote, or to the user's fork when the user cannot push to the target, with a regular push. Athena never
   forces a push. If the remote branch already exists and is not an ancestor of the local one, the command
   stops and says so (R4).
3. **Open.** Athena opens a pull request with the host client. The body is generated from the job: the
   request, the criteria, the verifier's findings, and the job id. In stealth mode it contains no link to a
   record file. Athena stores the **pull request number** (with the host, the repository and the pushed head
   commit id) next to the job id: in owned mode in the job's records, in stealth mode in the state directory.
   Both modes store it, because step 5 looks the merge up by it.
4. **Wait.** Athena reads the pull request's check status through the client. It does not merge on its own
   authority: the merge is the user's, or the host's auto-merge setting the user chose.
5. **Merge observed.** Athena asks the host for the pull request by its stored number and reads the merged
   commit id from the answer (merge commit, squash or rebase). **A landed job is identified from records,
   never from the trailers of the merged commit**: Athena does not parse trailers, and does not search the
   base branch's log for the job id, to find the merge. The pull request number gives the merged commit; the
   records give the job. It then reconciles: in owned mode it checks the landing rule of M7 or M30 against the
   new base-branch tip (the gate already decides from the closing records the commit adds, and treats
   trailers only as hints that must agree **(read, `crates/athena-gate/src/invariant.rs` 267-272)**); in
   stealth mode it records in its state directory that the job landed, with the merged commit id, after the
   content check of §5.4.
6. **Stale.** If the target branch moved before the merge, the job is stale under the existing rule of D10.
   Athena does not rebase silently. What the design supplies is in §5.7: the job is shown as stale, and on
   the user's request Athena re-verifies it on the new tip as a new, budgeted job.

### 5.2 What changes in the pushes rule

The prose-only rule says Athena does not push. Under this design:

- Athena **may push**, only on the user's explicit command (`athena land --pr`), only to the work branch it
  created, only as a regular push.
- **Unchanged:** published history is never rewritten. A push that is not a fast-forward of the remote branch
  is refused; Athena has no force option.
- **Mechanism versus prose.** The refusal of a non-fast-forward push, and the restriction to the job's own
  work branch, are checked in code before the client is called and need tests (new row, section 8). "Athena
  does not push" is withdrawn. "Published history is never rewritten" stays a rule Athena obeys by
  mechanism for its own pushes, but cannot enforce against the user or the host: a user can still force-push
  by hand, and the host's branch protection (when the user has it) is what stops that.
- Until the build exists, the prose-only list keeps its current text. This document changes no rule.

### 5.3 Records, trailers and checks in a pull request (owned mode)

- **M6 (governed content changes only through a verified landing).** The work branch is built by the same
  plumbing path, so its commits carry the job trailers and pass the same gate. Nothing about M6 changes
  on the branch.
- **M7 (nothing lands unverified).** The pushed commit holds the verified fingerprint. After a merge:
  a merge commit keeps the branch commits, so the landing rule sees each job as today; a rebase merge
  recreates the commits with new ids, and the rule does not depend on commit ids (squash-chain design §2)
  **(read)**; a squash is judged by the chain rule (M30). **No part of this relies on the job trailers
  surviving the host merge.** A squash builds a new message from the host's template, and the host may add
  text after the trailer block (E1, reported), which moves the trailers out of the last paragraph where git
  reads them. The landing rule finds jobs from the closing records the commit adds; a trailer that is hidden
  neither fails a valid landing nor lets a wrong one pass. A host merge the rule cannot link is detected by
  the history check in CI, not prevented, as the v0 design says of any push (`design/athena-v0.md` 1446-1449)
  **(read)**.
- **M23 (the prediction precedes the outcome).** M23 is now checked from the records' recorded times and
  invocation order, with no trailer or ancestry (`docs/enforcement.md`, row M23) **(read)**. Pushing and
  merging do not move those times, so the check is unaffected by which merge method the host uses.
- **Trailers.** The job trailers on the branch commits are unchanged in owned mode. They are a convenience
  for a reader of the branch, not a lookup key after a host merge. Any code the build adds to find a merged
  commit uses the stored pull request number (§5.1 step 3) and never parses trailers from the merged commit.
  A test for this rule is named in §8.
- **Retry path.** The core's retry path greps for `^Athena-Job: <id>$` and then parses that commit's trailers
  for `Athena-Prediction` **(read, review 2 F1, `run.rs` 1370-1383; the squash case is a reading, not run)**.
  That is safe while the blocked closing commit is Athena's own. If a build ever lets a host-squashed commit
  be a blocked closing commit, this path must read the records instead.

### 5.4 Stealth mode: no Athena files in a third-party repository

In stealth mode **(design)**:

- **Where records live.** In Athena's state directory for that repository, under the same job directories and
  schemas. The gate's append-only and validity checks (M4, M5) run over that store instead of over
  `records/`.
- **What the commit contains.** Only the user's change. No `records/`, no `.gitattributes` lines for
  Athena's attributes, no `athena.toml`, no hooks installed in the repository's hooks directory, no CI
  workflow. Athena's classification of owner-private paths comes from its own configuration in the state
  directory, not from a committed attribute file.
- **Trailers.** None by default. A trailer names a job whose records a reader cannot find in a third-party
  repository, and it would announce the tool. The link from commit to job is kept in the state directory
  instead: the pull request number (§5.1 step 3) and the merged commit id read from the host, next to the job
  id. After a squash of several jobs one merged commit id sits next to several job ids (§5.6).
- **No model `Co-Authored-By`.** The landing message Athena builds today carries a `Co-Authored-By` trailer
  for the model **(read, review 2 F5: `run.rs` 1107-1115 and 1398-1403)**. In stealth mode Athena writes none
  unless the user asks for one: a line naming a model announces the tool as an `Athena-Job` line does.
- **Commit identity.** The commit is authored as the user, with the user's own signing settings (D10),
  and the message is the user's, drafted by Athena.
- **What is still checked.** Everything that does not need the repository to hold records: the verified
  fingerprint equals the pushed content (R2, checked before the push, M7 first half), the secret scan of the
  candidate (M1, M2, M22), and the append-only and consistency checks over the local store.
- **What is given up, said plainly.** The history audit in CI (M3's sibling checks, M6/M7 over history) has
  nothing to read in the third-party repository, so it cannot run there. Stealth trades the repository-side
  audit for cleanliness. The proof that the landed content was verified lives only in the local store.
  **Two host-merged pull requests can produce unverified content.** The host can merge two pull requests
  made from the same base cleanly when their files are disjoint; the merged tree then equals neither job's
  verified fingerprint (review 2 F4). In owned mode the history check detects this after the fact, not
  before. In stealth mode **nothing detects it**: there is no audit. The only guard is the pre-report check
  below, which runs on the user's machine when Athena observes the merge, and so cannot stop the merge.
- **Merged-content check (design).** Before Athena reports a job as landed, it fetches the merged commit
  and compares its tree (or, for a pull request whose base moved, the diff the pull request brought) with
  the verified fingerprint. A mismatch is reported as "merged content is not the verified content", the
  job is not recorded as landed, and the user is told. This check applies in both modes.
- **Leak checks.** Before opening the pull request in stealth mode, Athena compares the pushed tree to the
  base tree and refuses if any path or message line would add an Athena-specific file, attribute or trailer
  **(design)**. A `Co-Authored-By` line that Athena itself wrote (for the model) counts as a leak. A
  `Co-authored-by` paragraph that the **host** adds when it builds a squash message is the host's, appears
  only after the pull request is merged, and is never a leak failure; the check runs on what Athena pushes
  and on the message it drafts, not on the merged commit. This is a mechanism; its tests are listed in
  section 8.

### 5.5 Brownfield

- No history walk blocks adoption. Stealth mode starts from the repository's current tip with no records
  anywhere. Owned mode on an existing repository records the adoption point, and the history check applies
  only to commits after it. This is the same shape as the bootstrap range and the exempt list (D60, D61)
  **(read)** and is designed separately; this document needs only that stealth mode does not depend on it.
- The first job in a repository has no `previous_job` and no landed predecessor. The chain rule already
  treats that as a head (D69) **(read)**.

### 5.6 One pull request per job, and a user who squashes several

**Choice (assumption, listed in §9):** Athena opens **one pull request per job**. Option (b), a pull request
that holds several jobs (branch naming, a body listing every job, a one-to-many merged-commit mapping), is not
designed here and would be a separate design.

A user may still combine jobs by hand, for example by landing three job branches in one pull request and
squashing it. That case relies on the chain rule (M30) with its limits:

- **The check still works.** The gate finds every job from the closing records the commit adds, so a squash
  that names one job still lands all of them if the chain is valid **(read, review 2 F2,
  `invariant.rs` 225-252)**.
- **Naming.** The trailer rule refuses a squash whose `Athena-Job` names a request job the commit does not
  land **(read, `invariant.rs` 297-303)**. If the host names a member of the chain it passes; nothing here
  guarantees that, and E2 (reported) says a squash message names only one job.
- **Stealth bookkeeping.** One merged commit id is stored next to several job ids (§5.4): the state
  directory's link is many-to-one, not one-to-one. Athena's content check (§5.4) applies to the last job's
  fingerprint, as the gate's does.
- **Inherited gap.** The squash-chain design's second review (R2-M1) found that nothing said where
  `previous_job` comes from. That design now says the core derives it as the latest landed job in the base
  commit's history, and leaves it absent when none is found, so an unlanded predecessor makes the chain be
  refused **(read, `design/squash-chain-landing.md` §4.1 and the R2-M1 row)**. Whether the build matches that
  text was not checked here. A chain of jobs held in several pull requests that were never landed in turn
  still has no landed predecessor to read, so it is refused rather than guessed. This design does not claim
  more safety than M30 gives.
- A user who wants each job visible in history picks a merge commit or a rebase merge, which keep one commit
  per job (§5.3).

### 5.7 Parallel jobs, and a job whose base is no longer the target's tip

This design supplies **per-job pull requests** as the parallel-jobs review's F1 option (b): each job has its
own branch and pull request, so independent jobs can be merged by the host in any order. It does **not**
supply automatic re-verification (option a), and it does not claim to make every parallel job land.

A job whose base is no longer the target's tip proceeds as follows **(design; assumption, §9)**:

1. Athena marks it stale when it observes the moved target (D10), and does not rebase silently.
2. If the host reports the pull request still merges cleanly, the user may merge it; the merged-content check
   (§5.4) then decides whether Athena reports it as landed. A clean merge of disjoint files from a moved tip
   gives content nobody verified, so that check will often refuse; the user is told why.
3. Otherwise, **on the user's request only**, Athena re-verifies: it starts a new job on the new tip, carrying
   the stale job's request, with its own budget and its own prediction. Nothing is re-verified automatically.

The roadmap lists the parallel pool before this design (`docs/design/roadmap.md`, RM-0). Review 2 and the
parallel-jobs review ask for the opposite order. **The roadmap order is to be changed in a separate job;
this document does not edit it.**

## 6. Risks

- **Host differences.** Merge methods, auto-merge and fork permissions vary. Athena should support one host
  first (GitHub, through `gh`) and refuse others with a clear message rather than guess.
- **Credentials.** The host client holds the credential. Athena must keep the client's output out of
  records, since it can contain tokens or addresses (M1, M22). Redaction applies as for agent text.
- **Stealth leaks.** A branch name, a pull request body or a commit message can reveal the tool. Section 5.4
  covers messages and files; the branch name should be neutral (design: a short name from the change, not
  the job id).
- **Local store loss.** In stealth mode the only proof of verification is local. Losing the machine loses
  it. Alternative E is the answer if that matters.

## 7. Out of scope

Implementing any of this; the contribution-etiquette questions of third-party projects (their templates,
sign-off rules, review norms), which the user's instructions govern; designing the other 2026-10-12 addenda.

## 8. Enforcement plan

| Rule | By mechanism | Prose-only |
|---|---|---|
| Push only on the user's command, only the job's work branch | yes: code refuses otherwise; test needed | |
| No force push, no rewrite of published history by Athena | yes: refuse a non-fast-forward; test needed | what the user or host does by hand |
| Pushed content equals verified content (R2) | yes: existing fingerprint check run before the push (M7) | |
| Stealth pull request adds no Athena file, attribute or trailer (R5) | yes: tree-and-message comparison; test needed | the branch name's neutrality, until a check exists |
| Stealth pull request writes no Athena `Co-Authored-By` for the model | yes: part of the leak check; test needed, including a case where only the host's merge adds a `Co-authored-by` paragraph (not a failure) | |
| A landed job is identified from records, never from trailers (F1) | yes: test needed with a hand-built commit whose message ends in a `Co-authored-by` paragraph after the Athena trailers; the landing check must still decide from the records and give the same answer as without that paragraph | |
| Merged content equals the verified fingerprint before Athena reports "landed" (F4) | yes: pre-report check, both modes; test needed (two disjoint-file pull requests from one base: the second is not reported as landed) | the merge itself, which the host performs |
| The merge is found by stored pull request number (F6) | yes: lookup test needed, in both modes, with no trailer on the merged commit | host answers correctly |
| Records in stealth mode stay append-only and valid | yes: M4, M5 over the local store | |
| Merge is the user's decision | | prose-only: Athena does not call a merge |
| Remote host behaves as expected | | prose-only |

New rows would be numbered in `docs/enforcement.md` when the tests exist (M24 requires every named test to
exist, so none are named now).

## 9. Open questions for review

1. Is stealth the right default for a repository with no `records/`, or should the user always be asked?
2. Should a stealth store be exportable to a second repository (alternative E) in the first build?
3. Should Athena ever merge, when the user turns on auto-merge, or stay strictly out of the merge?
4. Is GitHub through `gh` the right first host?

Added after review 2 (§4 there). The user has not answered these; each shows the assumption this revision took.

5. For a job whose target moved, should Athena re-verify on the new tip when asked, or leave it stale and
   re-propose? **Assumption:** re-verify on the user's request only, as a new budgeted job; never
   automatic (§5.7).
6. One pull request per job only, or also several jobs in one pull request? **Assumption:** one per job;
   a user who squashes several relies on M30 with its limits (§5.6). The multi-job design is not built.
7. Should Athena check that a host-merged commit holds the verified content before it reports "landed"?
   **Assumption:** yes, a pre-report check against the verified fingerprint (§5.4, §8).
8. In stealth mode, is a `Co-Authored-By` line for the model ever wanted? **Assumption:** no, unless the
   user asks (§5.4).

## 10. References re-checked at this revision

| Cited in | Reference | Result |
|---|---|---|
| §1 | `BRIEF.md` lines 459-466 | Checked: 459-461 are the "does the work its user should not have to" addendum, 463-466 the brownfield and stealth addendum. Correct. |
| §4 D | `design/reviews/athena-v0-review-1.md` near line 626 | Checked: lines 626-627 say notes and side refs are not fetched or pushed by default. Correct. |
| §5.3 M7 | `design/athena-v0.md` near line 1447 | Checked: 1446-1449 say a violating push is detected after the fact rather than prevented. Correct; cited as 1446-1449. |
| §5.3 M23 | `docs/enforcement.md`, row M23 | Checked: the row says "no trailer or ancestry". Correct. |
| §5.6 | `design/squash-chain-landing.md` `previous_job` source | Read at this revision: §4.1 and the R2-M1 row say the core derives it. Whether the build matches was not checked. |
| §1, §5.3, §5.4, §5.6 | `run.rs`, `invariant.rs` line numbers | Taken from review 2 (read there), **unchecked** at this revision. |
