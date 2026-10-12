# Native pull-request landing, including repositories Athena does not own

**Status:** proposed (D72). Design only: no code is built by this document.

Claims are marked **(read)** when taken from the file named beside them, and **(design)** when they are
proposals. No command was run to reproduce any behavior.

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
  (`design/reviews/athena-v0-review-1.md`, near its line 626) **(read)**.
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
   record file.
4. **Wait.** Athena reads the pull request's check status through the client. It does not merge on its own
   authority: the merge is the user's, or the host's auto-merge setting the user chose.
5. **Merge observed.** Athena notices the merge (merge commit, squash or rebase) by asking the host for the
   merged commit. It then reconciles: in owned mode it checks the landing rule of M7 or M30 against the new
   base-branch tip; in stealth mode it records in its state directory that the job landed, with the merged
   commit id.
6. **Stale.** If the target branch moved before the merge, the job is stale under the existing rule of D10
   and Athena asks again instead of rebasing silently. Re-verification stays as D10 (d) left it: not
   automatic.

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
  recreates the commits with new ids, but the trailers and the content survive, and the rule does not depend
  on commit ids (squash-chain design §2) **(read)**; a squash is judged by the chain rule (M30). A host
  merge the rule cannot link is detected by the history check in CI, not prevented, as the v0 design says of
  any push (`design/athena-v0.md`, near its line 1447) **(read)**.
- **M23 (the prediction precedes the outcome).** M23 is now checked from the records' recorded times and
  invocation order, with no trailer or ancestry (`docs/enforcement.md`, row M23) **(read)**. Pushing and
  merging do not move those times, so the check is unaffected by which merge method the host uses.
- **Trailers.** The job trailers on the branch commits are unchanged in owned mode.

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
  instead: the commit id (or, after a squash, the merged commit id read from the host) next to the job id.
- **Commit identity.** The commit is authored as the user, with the user's own signing settings (D10),
  and the message is the user's, drafted by Athena.
- **What is still checked.** Everything that does not need the repository to hold records: the verified
  fingerprint equals the pushed content (R2, checked before the push, M7 first half), the secret scan of the
  candidate (M1, M2, M22), and the append-only and consistency checks over the local store.
- **What is given up, said plainly.** The history audit in CI (M3's sibling checks, M6/M7 over history) has
  nothing to read in the third-party repository, so it cannot run there. Stealth trades the repository-side
  audit for cleanliness. The proof that the landed content was verified lives only in the local store.
- **Leak checks.** Before opening the pull request in stealth mode, Athena compares the pushed tree to the
  base tree and refuses if any path or message line would add an Athena-specific file, attribute or trailer
  **(design)**. This is a mechanism; its tests are listed in section 8.

### 5.5 Brownfield

- No history walk blocks adoption. Stealth mode starts from the repository's current tip with no records
  anywhere. Owned mode on an existing repository records the adoption point, and the history check applies
  only to commits after it. This is the same shape as the bootstrap range and the exempt list (D60, D61)
  **(read)** and is designed separately; this document needs only that stealth mode does not depend on it.
- The first job in a repository has no `previous_job` and no landed predecessor. The chain rule already
  treats that as a head (D69) **(read)**.

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
