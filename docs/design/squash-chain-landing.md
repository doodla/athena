# Squash-merge landing for a chain of verified jobs

**Status:** proposed (D69). Nothing here is implemented. This change edits documentation only: no Rust code,
no `athena.toml`, no workflows, no other governance file, and nothing under `records/`. The gate behaves
exactly as before until the follow-up work below is built and accepted.

Claims are marked **(read)** when taken from the code or docs named beside them, and **(inferred)** when they
follow from reading but were not run. No command was run to reproduce any failure.

## 1. The problem

An adopter chooses how pull requests are merged. Many choose squash merge. A pull request may hold a *chain*
of jobs: job 1 is verified and lands on the branch, job 2 is based on that landed state, and so on. Each job
lands as its own commit and passes the gate there.

A squash merge then replaces the chain with **one commit** on the base branch. That commit adds the code of
every job and the closing records (`candidate.json`, `verification.json`, `outcome.json`, or
`outcome-landed.json`) of every job. The landing rule refuses it, for two independent reasons:

1. **More than one closing job.** `closing_jobs` collects every job whose closing records the commit adds,
   and `landing` in `crates/athena-gate/src/invariant.rs` refuses when there is more than one: "a commit adds
   closing records for more than one job" (lines 262-268) **(read)**.
2. **Whole-tree fingerprint mismatch.** Even without that refusal, `landing` requires the job's
   `candidate.json` fingerprint and its `verification.json` verified fingerprint to equal the fingerprint of
   the whole landed tree (lines 349-359) **(read)**. Each job verified only its own candidate, so at most the
   last job's fingerprint can equal the squashed tree; the earlier jobs' cannot.

Design §7.3 ("Merge, squash, and rebase merges", `docs/design/athena-v0.md`) describes a squash as one commit
"whose diff adds the code and every record of *the* job" **(read)**: the design chose one job per commit.
D66 recorded the resulting failure on main for the squash commit ee4b0dd, D67 excused that one commit with a
`[history] exempt` entry, and D68 extended the excuse to the job records the commit adds **(read,
`docs/decisions.md`)**. D66 left open how to stop it recurring: (b) teach the gate, or (c) stop squashing
multi-job pull requests. D66's option (c) is enforced by nothing unless the host restricts merge methods
**(read)**. This document designs (b).

The goal: a squash of a chain of sequentially verified jobs lands, and the history and landing checks pass
with no per-commit exemption, while "nothing reaches main unverified" (design §7.3) and "the prediction
precedes the outcome" (M23, `docs/enforcement.md`) still hold.

## 2. What a chain is

A chain is a list of jobs J1..Jn where each job was based on the state the previous job produced. In
Athena's terms **(read, `Candidate` in `crates/athena-data/src/records.rs`)**, a job's `candidate.json`
records `base_commit` and `fingerprint` (the fingerprint of the candidate tree). It does **not** record the
fingerprint of its base. After a squash, `base_commit` names a commit that no longer exists on the base
branch, so it cannot be resolved by the gate **(inferred from the squash description in §7.3)**. The chain
link must therefore be something a squash keeps: a fingerprint (the content link) and a job id (the order),
and not a commit id. Job ids survive because each job's records sit in a directory named by its id.

## 3. Alternatives

**A. Per-job patch composition proof.** For each job, treat its candidate as a patch against its base and
require the union of the patches to equal the landed tree.
- For: no new record field if every candidate commit were still reachable.
- Against: the squash removes the intermediate commits, so the patches are gone. The records carry a change
  list (`changes`) but the proof would need the blobs, which are not in the landed tree. It also needs a merge
  of overlapping patches, a new algorithm in the most sensitive part of the gate. Weakens "one verified
  candidate is the landed content" (D66 (b)).
- Rejected: needs data the squash discards.

**B. Chain-aware landing rule (recommended).** Each job records the fingerprint of its base and the id of its
predecessor job. The gate accepts one commit that adds the closing records of jobs J1..Jn when they form one
linear chain, in the recorded order, from the parent's fingerprint to the landed tree's fingerprint, and
every job passes every per-job check except "fingerprint equals the landed tree", which applies only to the
final job.
- For: uses only what a squash keeps (records and the final tree). The landed governed content equals the
  final job's verified fingerprint, and every earlier job's records are checked for consistency with it.
  Works for squash, rebase, and merge. No exemption. Strictness for a single job is unchanged (a chain of one
  is today's rule).
- What it does not give: the final job's verifier did **not** review the whole tree. The verifier receives a
  `CandidateView` of its own job's diff and stat (v0 §6.3 **(read)**); the checks ran on the whole tree, but
  the review of intent covered only that job. Earlier content was judged by earlier verifiers at their own
  states, and the new rule reaches it only through record agreement. So B's benefit over E is that it
  catches **accidental** mismatches (a leftover job from another pull request, a missing or reordered job, a
  stale base). It does not stop **forgery**: anyone who can write one consistent set of records can write
  the chain.
- Against: new fields in `candidate.json` (a schema change, settled in §4.1). Intermediate states are not
  present as trees, so they are proved only by the records' agreement (the same unsigned-records limit as
  §7.3's timestamp proof). Jobs landed before the fields exist cannot form a chain.

**C. Restrict merge methods (D66 (c)).** Adopters allow only merge commits or rebase merges, or one job per
pull request.
- For: no code; each commit is a single-job landing today (§7.3).
- Against: this is the restriction the request asks to remove. It is enforced only by a host setting, which
  is out of Athena's control; a mistaken squash puts main back into the ee4b0dd state.

**D. Exemption (D60, D67).** Add the squash commit to `[history] exempt`.
- For: one line, no code, reversible.
- Against: the whole commit skips the invariant (D68 extends this to its records), so nothing reaches main
  unverified is not checked for it. It needs a follow-up commit after every squash, because a full id must
  already be in history (D60). It does not scale to routine squashes.

**E. Last-job-only rule (B without the links).** Accept the commit if the last job's fingerprint equals the
tree and ignore the others.
- Against: earlier jobs' records could be unrelated (for example left over from another pull request), and
  nothing says they were part of the tree's history. Rejected, but on the accident-catching ground only: E
  and B are equally open to forged records (see B above). B checks that the records agree; E does not.

## 4. Recommendation: B

B is the only option that removes both the restriction and the exemption without requiring data the squash
destroys. It keeps the final tree checked against a verification of exactly that tree, and it makes the
earlier jobs' contribution checkable for consistency by a chain of fingerprints in a recorded order.

### 4.1 Records that link a chain

- **Schema choice (R1-m3): `athena.candidate/2`.** `Candidate` is `deny_unknown_fields` and is
  `athena.candidate/1` **(read, `records.rs`)**. An optional field under `/1` would make an old gate fail on
  a new record with an unhelpful parse error; a `/2` fails by a named schema. So the build adds
  `athena.candidate/2` with the two fields below, and `RecordSet::load` (in `athena-gate`) accepts `/1` and `/2`. A `/1`
  candidate is a complete job on its own and can land alone as today, but cannot be a member of a chain.
  D69's reversal text is amended to match.
- New fields in `candidate.json` `/2`:
  - `base_fingerprint`: the fingerprint of the base commit's tree.
  - `previous_job`: the id of the job this one was based on, or absent for the first job of a chain. The
    order of a chain is this field, **not** a search through fingerprints (R1-M1).
- **Which rules (R1-M3).** Both fingerprints in a job's record use the rules of that job's own base commit
  (`base_commit`'s committed `.gitattributes` files), computed with `fingerprint` (`athena_git::attrs`).
  Because `Judged` judges X by the rules of P **(read, `lib.rs`)**, a chain is only comparable when every
  job's base has the same rules as P. The gate therefore **refuses a chain in which any job changes a
  classification rules file** (see 4.2 step 3 and 4.3). Under that refusal all fingerprints in the chain
  are computed under identical rules and are comparable with F.
- Where the core writes them: the core holds the base commit `p` when it builds the `Candidate`
  (`crates/athena-core/src/run.rs` 456-465, with `base_commit: p.clone()` and `fingerprint: None`
  **(read, review 2)**); `fingerprint(base, base)` is available there.
- **Where `previous_job` comes from (R2-M1).** The core derives it; the user supplies nothing. The core
  looks at the base commit's own history (the commit and its ancestors) and takes the **latest landed job**:
  the job named by the most recent commit that added a landed outcome (`outcome.json` with a landed status,
  or `outcome-landed.json`), found with the same reading of records the history walk uses
  (`history.rs` 77-78). Consequences:
  - *After a hand change (an intervention) or a records-only commit:* these add no landed outcome, so they
    are skipped and the latest landed job before them is still the predecessor. The chain is not reset. If
    the hand change altered governed content, the next job's `base_fingerprint` will not equal that
    predecessor's `fingerprint`, and the gate refuses the chain at the link check, which is correct: the
    link is not unbroken.
  - *With an unlanded predecessor* (job 2 is based on job 1's candidate commit and job 1 has not landed): job
    1's records are not a landed outcome in the base's history, so the core does not name it. `previous_job`
    is then the earlier landed job, or absent. A squash holding both jobs would then show two heads or a
    broken link and be refused. Refusing is safe; the adopter lands job 1 first or the chain is rebuilt.
  - *When no landed job is found,* `previous_job` is **not written** (the field is absent) and the job is
    the first of a chain.
  This rule is a **stated assumption the user may change**, e.g. to "the user names the predecessor" or "the
  job whose landing commit is exactly the base commit". It is **(inferred)**; no command was run, and the
  build job confirms the history read before writing code.
- The chain link is: for a job Ji with `previous_job` = Ji-1, `Ji.base_fingerprint == Ji-1.fingerprint`. The
  first job has no `previous_job`, and its `base_fingerprint` equals the fingerprint of the squash commit's
  first parent. The last job's `fingerprint` equals the fingerprint of the squash commit's tree.
- `verification.json` already holds `verified_fingerprint` and `candidate_commit`; no change. Per job it must
  equal that job's own `candidate.json` `fingerprint`.

### 4.2 The check, exactly

Let X be the commit, P its first parent, R the rules of P, and F = fingerprint(X; R). Let `jobs` be the
closing jobs X adds (`closing_jobs`). If `jobs` is empty, nothing changes. Otherwise:

1. If `jobs` has one member, run today's rule unchanged (this includes the fingerprint equality at lines
   349-359). A chain of one is not special. This also covers a partly landed chain where only one job is
   left (R2-m2): the job is judged by the single-job rule and is not refused for a stale base, since the
   fingerprint equality with F already proves the content. The head rule is not applied to it.
2. If it has more than one: refuse if X has more than one parent (decision 3 in Appendix A). Otherwise load
   each job's files from X's tree and require each `candidate.json` to be `athena.candidate/2`.
3. Refuse if any job's `changes` list names a classification rules file (a `.gitattributes`), or if the
   diff from P to X touches one. (R1-M3; the `changes` field name is **(inferred)** from the records the
   design relied on above and the build job confirms it.)
4. Order the jobs by `previous_job`, not by fingerprint (R1-M1). Exactly one job is the head: it has no
   `previous_job`, or its `previous_job` is a job already landed on P (found as in 4.6, partly landed chains). Its
   `base_fingerprint` must equal fingerprint(P; R). Each other job names a `previous_job` that
   is a member of `jobs`, and no two jobs name the same one. Following the names from the head must use
   every job exactly once: no fork, no cycle, no job left over, no predecessor outside `jobs`. Then for each
   consecutive pair, `Ji.base_fingerprint == Ji-1.fingerprint`. Fingerprints may repeat: a job that reverts
   an earlier one, or one that changes no governed content (`base_fingerprint == fingerprint`), orders
   correctly because order never depends on their uniqueness.
5. The last job's `fingerprint` and `verified_fingerprint` must equal F.
6. For every job, run every check `landing` runs after the trailer checks, using that job's records (R1-n1).
   Any one job failing refuses the commit (R1-m5). The list: `job.json` is in the landed tree and names an
   accepted reading; a prediction that precedes that job's own work (`prediction_problem`); a passing final
   verdict; matching candidate commits; a successful verifier invocation with the job's verifier model;
   reconciled usage and outcome totals; the three-way choice between a fresh landed outcome, a retried
   outcome, and `outcome-landed.json` after blocked; and no earlier landing of the job in the parent or in the
   history walk. The fingerprint equality of step 5 is, for earlier jobs, replaced by
   `candidate.fingerprint == verified_fingerprint`, and both equal the next job's `base_fingerprint`.
7. Trailers stay hints. `Athena-Verified-Content` must equal F. An `Athena-Job` naming a request job must
   name a member of `jobs`.

### 4.3 Refusal messages

Each is returned through the existing `refuse`, which reports a violation on the commit:

- `a chain landing is refused on a commit with more than one parent` (step 2; `landing` produces it, but
  for a merge commit `check` shows the existing merge message instead, see §4.6)
- `a chain landing needs athena.candidate/2 in <dir>/candidate.json` (step 2)
- `job <x> changes a path classification rules file, so its fingerprints are not comparable; land it alone`
  (step 3)
- `no job in the commit starts from the parent's content <P-fingerprint>` (step 4, no head, or a head whose
  base differs)
- `jobs <a> and <b> both follow <c>, so they do not form one chain` (a fork)
- `job <x> is not linked to the others: its previous job <y> is not in this commit` (a leftover, a cycle, or
  a predecessor outside the commit)
- `job <x> starts from <fp> but its previous job <y> ended at <fp2>` (a broken fingerprint link)
- `the last job <x> holds <fp>, not the landed content <F>` (step 5; the existing per-record messages for
  `candidate.json` and `verification.json` apply the same way)
- the existing messages for each per-job failure (steps 6-7), which already name the job's directory.

### 4.4 Why "nothing reaches main unverified" holds, and how far

The landed tree's governed content equals the final job's verified fingerprint (step 5). The final job's
verifier reviewed **its own job's diff and stat**, not the whole tree **(read, v0 §6.3)**; the checks ran on
the whole tree, but the review of intent did not. Earlier content was judged by each earlier job's own
verifier at its own state. The claim that holds is therefore: each step from the parent to the landed content
names a passing verification of that step, and the records agree on the order and the content between steps.
It is not the claim that one verifier saw the final tree. **(inferred from the design of the fingerprint:
equal fingerprints mean byte-identical governed content, §7.3.)** Governed content that matches no job's
chain makes the final fingerprint differ and the commit is refused.

What the chain check adds over option E is that it catches **accidental** mismatches: a job left over from
another pull request, a missing, reordered or stale job. It does not prevent **forgery**. The records are
written by Athena and not signed; someone who forges every record consistently forges the chain. This is the
limit §7.3 states for the timestamp proof, and it is no weaker here **(read, §7.3)**. The argument above is
untested (R1-n2): the first thing the build job writes is a hand-built two-job chain test in
`landing_tests.rs`, before any gate change, to show the rule accepts it and refuses its variants.

### 4.5 Why M23 (prediction precedes the outcome) still holds

Step 6 runs `prediction_problem` for every job against that job's own records, so each job's accepted
reading is recorded before its own dispatch and before every one of its producer and verifier invocations.
The chain adds an order between jobs (a job's base is another's result) but changes nothing about
per-job order, so a reading written after its own work is still refused. A reading is not required to
precede the *previous* job's work: each job predicts its own work.

### 4.6 Other cases

- *Parent moved.* If other governed changes reached the base branch after the chain began, the head's
  `base_fingerprint` no longer equals the parent's fingerprint and the squash is refused ("no job starts from
  the parent's content"). That is correct: the squash would not equal the chain's final content anyway. The
  adopter rebases the chain, which makes the jobs stale (design §7.3 landing algorithm). A change to a
  classification rules file on the base branch between the head's base and P has the same effect (R2-n1):
  fingerprints use each base's own rules, so the head's `base_fingerprint` can differ from
  fingerprint(P; R) for the same content, and the chain is refused with the same message. Refusing is the
  safe direction. **(inferred)** from `fingerprint` in `attrs.rs`; the case was not constructed.
- *Records-only commits in the pull request* do not change the governed fingerprint and do not break links.
- *A job that blocked then landed* (`outcome-landed.json`) is handled by step 6, as today.
- *Rebase merge* keeps today's single-job commits and never reaches the chain path.
- *Merge commit (R1-m2).* `check` calls `landing` for a merge commit whose content equals no parent
  **(read, `invariant.rs` 126-134)**, so a merge commit whose first-parent diff adds several jobs' records
  would reach the chain path. Decided: chain landing is **refused on a commit with more than one parent**
  (step 2). Such a merge lands only if each job is its own commit, as today. The refusal's behaviour is
  right, but for a merge commit `check` uses only whether `landing` returned `Ok` and then reports "a merge
  introduces governed content that neither parent has" (`invariant.rs` 126-138 **(read, review 2)**). So the
  message of §4.3 for this case is **not shown**; the existing merge message is what the user sees (R2-m1).
  Passing `landing`'s reason through on the merge path is a possible change to `check` and is not made here.
- *Partly landed chain (R1-m4, R2-m2).* If J1 landed alone and J2, J3 arrive in a squash, `closing_jobs`
  counts status `A` only, so J1 is not in `jobs`; the squash adds J2 and J3. A squash that leaves one job
  is judged by step 1 (the single-job rule). With two or more left, J2 is the head: its `previous_job` is J1,
  which is not in `jobs`. The head rule is: a job whose `previous_job` is absent, or names a job already
  landed on P, is the head, and its `base_fingerprint` must equal fingerprint(P; R). **How the gate finds
  a landed predecessor at P:** it loads that id's `JobFiles` from P's tree, as `landing` loads `before` at
  line 294, and requires the closing records to be present there and the predecessor's `candidate.json`
  `fingerprint` to equal fingerprint(P; R). If the predecessor's records are not at P, the head is refused
  as "not linked". **A `/1` predecessor is accepted:** only its `fingerprint` is read, and the head's own
  `/2` record carries the `base_fingerprint`; the "a `/1` cannot be a member of a chain" rule applies to
  members of `jobs`, not to a predecessor already landed. The two orders in the earlier text are one case,
  so there are tests for (a) one job left, judged by the single-job rule, and (b) two or more left with a
  landed predecessor.

## 5. Follow-up work (not done here)

1. `crates/athena-data` `records.rs`: add `athena.candidate/2` with `base_fingerprint` and `previous_job`.
   The schema name is chosen by the tagged enum `AnyRecord` (`#[serde(tag = "schema")]`, lines 796-824),
   which has `athena.candidate/1` as its only candidate variant, so add a `/2` variant or a shared type,
   and add `/2` to the schema-name table (line 914). Update `docs/records.md`.
   `RecordSet::load` is in `crates/athena-gate/src/records.rs` (27-56), not `athena-data`; it parses through
   `check_record_file`, so it accepts `/1` and `/2` once `AnyRecord` does. Every consumer that matches
   `AnyRecord::Candidate` must handle `/2`: `crates/athena-core/src/run.rs` (950-956, where it is written),
   `crates/athena-core/src/store.rs` (145), and `crates/athena-gate/src/jobs.rs` (64). The list came from a
   search by review 2 and may be incomplete; the build job searches again.
2. `crates/athena-core` `run.rs` (456-465): write `base_fingerprint` and `previous_job` (derived as in §4.1)
   when the candidate is built, as `/2`; update `store.rs` (145) and the write at 950-956 for the new variant.
3. `crates/athena-gate/src/invariant.rs`: replace the single-job refusal in `landing` with the chain check
   of §4.2; factor the per-job checks so they can run per job. `landing_job` must return a set of jobs.
   `crates/athena-gate/src/jobs.rs` `jobs::check` (lines 330-344 in the base commit) calls `landing_job` and
   compares one job to each landed outcome in the diff; it must accept every job of the chain (R1-m1).
   `HistoryState::landed` and the history walk (`history.rs` 77-78) must record every job of the chain.
4. Tests, in `crates/athena-gate/src/landing_tests.rs` (and `jobs.rs` / `history.rs` for the last two):
   - first, a hand-built two-job chain that lands (R1-n2);
   - a valid chain squash passes both `invariant::check` and `jobs::check` (R1-m1);
   - the history walk: landing a chain, then repeating one of its jobs, is refused (R1-m1);
   - each refusal of §4.3: `/1` candidate in a chain, wrong head, a fork, an unlinked job, a broken
     fingerprint link, a last job not matching the tree;
   - fingerprints that repeat: a job that reverts an earlier one (A to B, B to A, A to C) lands; a job that
     changes no governed content lands (R1-M1);
   - a job that changes a `.gitattributes` file refuses the chain (R1-M3);
   - a chain on a commit with two parents is refused (R1-m2);
   - a partly landed chain (R1-m4, R2-m2): one job left (judged by the single-job rule), and two or more
     left with a landed predecessor, including one whose predecessor is a `/1` candidate;
   - `previous_job` derivation (R2-M1): after a hand change, after a records-only commit, with an unlanded
     predecessor, and with none found (field absent);
   - a merge commit whose first-parent diff adds several jobs is refused, and the test asserts the message
     `check` actually shows (R2-m1);
   - one refusal test per per-job check, each run against a **middle** job while the first and last are
     clean: no accepted reading or `job.json` missing, a reading written after that job's work, a failing
     verdict, candidate commits that differ, a failed or wrong-model verifier invocation, unreconciled usage,
     a bad outcome choice, and a job already landed in the parent or history (R1-m5);
   - a missing middle job whose neighbours' fingerprints still link: refused because `previous_job` names an
     absent job (R1-m5).
5. Update design §7.3 (merge, squash, and rebase merges) and `docs/enforcement.md` (the M23 and landing rows)
   once the check exists, and settle D69 as accepted or changed.

## 6. This change

The first version of this change added this document, the decision D69, a changelog line, and pointers in
`docs/design/athena-v0.md`, `docs/enforcement.md`, and `docs/architecture.md`. The revision answering review 1
rewrites this document, amends D69 (the schema choice and the tradeoff wording), adds a changelog line, and
updates the pointers in `docs/design/athena-v0.md` and `docs/enforcement.md`. It modifies no Rust code,
`athena.toml`, workflow, or other governance file, and nothing under `records/`. The review file is
unchanged. D60-D68 stay in force; D69 supersedes none of them until it is accepted.

The revision answering [review 2](reviews/squash-chain-landing-review-2.md) edits §4.1, §4.2 step 1, §4.3,
§4.6, §5 and this section, adds Appendix B, and amends D69, the changelog, and the pointers in
`docs/design/athena-v0.md` and `docs/enforcement.md`. It changes no code or test; the build is a separate
later job. The review file is unchanged.

## Appendix A: response to review 1

Every finding of [review 1](reviews/squash-chain-landing-review-1.md) is listed with its disposition.
"Accepted" means the change the review proposed was made, in the section shown. Anything that differs from the
proposal is said. None is declined. The review file is unchanged.

| Finding | Disposition | Where |
|---|---|---|
| **R1-M1** Ordering ambiguous when fingerprints repeat | **Accepted** by recording the order: `previous_job` in `candidate.json`. A reverting job and a no-op job are both ordered correctly and both are test cases. | [§4.1](#41-records-that-link-a-chain), [§4.2](#42-the-check-exactly) step 4, [§5](#5-follow-up-work-not-done-here) item 4 |
| **R1-M2** Security argument over-claims | **Accepted.** "The verifier saw the whole tree" is removed. B's benefit over E is catching accidental mismatches, not forgery; E's rejection is restated on that ground. | [§3](#3-alternatives) B and E, [§4.4](#44-why-nothing-reaches-main-unverified-holds-and-how-far) |
| **R1-M3** Rules under which fingerprints are computed | **Accepted** with the review's simpler rule. Each recorded fingerprint uses its own base commit's rules; a chain in which any job changes a rules file is refused, with a named message and a test. | [§4.1](#41-records-that-link-a-chain), [§4.2](#42-the-check-exactly) step 3, [§4.3](#43-refusal-messages) |
| **R1-m1** `jobs::check` missing from follow-up | **Accepted.** `jobs::check`, `landing_job` returning a set, and the history walk are listed; tests for `jobs::check` and the repeated-job history case added. | [§5](#5-follow-up-work-not-done-here) items 3-4 |
| **R1-m2** Merge-commit claim unchecked | **Accepted** (refuse option): chain landing is refused on a commit with more than one parent; the §4.6 claim is corrected; a test is added. | [§4.2](#42-the-check-exactly) step 2, [§4.6](#46-other-cases) |
| **R1-m3** Schema choice open | **Accepted:** `athena.candidate/2`, loaded alongside `/1`. D69's reversal text is amended. Where the core writes the fields is still inferred and the build job locates it first. | [§4.1](#41-records-that-link-a-chain), [§5](#5-follow-up-work-not-done-here) item 1 |
| **R1-m4** Partly landed chains | **Accepted.** Both orders are specified in §4.6, with the head rule extended to a predecessor already landed on the parent, and both are tests. | [§4.2](#42-the-check-exactly) step 4, [§4.6](#46-other-cases) |
| **R1-m5** Bad middle job; no per-check tests | **Accepted.** Step 6 says any one job failing refuses the commit; one refusal test per per-job check on a middle job and a missing-middle-job test are listed. | [§4.2](#42-the-check-exactly) step 6, [§5](#5-follow-up-work-not-done-here) item 4 |
| **R1-n1** Step 5 omits two checks | **Accepted.** The list now includes `job.json` with an accepted reading and the three-way outcome choice. | [§4.2](#42-the-check-exactly) step 6 |
| **R1-n2** Label the untested argument | **Accepted.** §4.4 states the argument is untested and that a hand-built two-job chain test is the build job's first task. | [§4.4](#44-why-nothing-reaches-main-unverified-holds-and-how-far), [§5](#5-follow-up-work-not-done-here) item 4 |

### The review's three decisions

The user did not answer these. The author chose the defaults below; each is a **stated assumption** the user
may change before the build job, and none is implemented.

| # | Decision | Assumed answer | Reason |
|---|---|---|---|
| 1 | Chain order: recorded or derived? (R1-M1) | **Recorded**: `previous_job` in `candidate.json`. | Derived order fails when fingerprints repeat; a recorded id has no such case. The cost is one more field in a schema change already needed. |
| 2 | Chain that changes path classification rules (R1-M3) | **Refused**, with a named message. | Smallest rule that keeps every fingerprint comparable. The cost is that such jobs land one at a time. |
| 3 | Chain landing on merge commits (R1-m2) | **Refused** on a commit with more than one parent. | Avoids an untested path in the gate's most sensitive rule; a merge commit still lands job by job. |

## Appendix B: response to review 2

Every finding of [review 2](reviews/squash-chain-landing-review-2.md) is listed with its disposition. Each
answer was checked against the code lines the review cites; the review's line numbers were read, not
re-measured by a tool. The review file is unchanged. Review 2 found no new decision for the user, but the
rule in R2-M1 is a stated assumption the user may change.

| Finding | Disposition | Where |
|---|---|---|
| **R2-M1** Nothing says where `previous_job` comes from | **Accepted.** The core derives it as the latest landed job in the base commit's history. A hand change or records-only commit does not reset it; an unlanded predecessor is not named, so such a chain is refused; when none is found the field is absent. Stated as an assumption the user may change. | [§4.1](#41-records-that-link-a-chain), [§5](#5-follow-up-work-not-done-here) items 2 and 4 |
| **R2-m1** Merge-commit refusal message never shown | **Accepted** (note option): for a merge commit the user sees the existing merge message; `landing`'s reason is not passed through. §4.3 and §4.6 say so, and the test asserts the message actually shown. | [§4.3](#43-refusal-messages), [§4.6](#46-other-cases), [§5](#5-follow-up-work-not-done-here) item 4 |
| **R2-m2** Partly landed cases are one; one path skips the head rule | **Accepted.** One remaining job is judged by the single-job rule (no stale-base refusal). With more left, the landed predecessor is found by loading its `JobFiles` at P and matching its fingerprint to fingerprint(P; R). A `/1` predecessor is accepted. The two cases are merged into two tests. | [§4.2](#42-the-check-exactly) step 1, [§4.6](#46-other-cases), [§5](#5-follow-up-work-not-done-here) item 4 |
| **R2-m3** Schema work in the wrong crate and understated | **Accepted.** `RecordSet::load` is placed in `athena-gate`; `AnyRecord`, the schema-name table, and the consumers (`run.rs`, `store.rs`, `jobs.rs`) are named. | [§4.1](#41-records-that-link-a-chain), [§5](#5-follow-up-work-not-done-here) items 1 and 2 |
| **R2-n1** Rules change on the base branch refused without naming it | **Accepted.** §4.6 now names a rules change as a cause of the "parent moved" refusal. | [§4.6](#46-other-cases) |
| **R2-n2** Uneven line wrapping | **Accepted.** The cited lines of D69 and of the v0 design are rewrapped to the surrounding width. | `docs/decisions.md` D69, `docs/design/athena-v0.md` |
