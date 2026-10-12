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
link must therefore be a fingerprint, which survives a squash, and not a commit id.

## 3. Alternatives

**A. Per-job patch composition proof.** For each job, treat its candidate as a patch against its base and
require the union of the patches to equal the landed tree.
- For: no new record field if every candidate commit were still reachable.
- Against: the squash removes the intermediate commits, so the patches are gone. The records carry a change
  list (`changes`) but the proof would need the blobs, which are not in the landed tree. It also needs a merge
  of overlapping patches, a new algorithm in the most sensitive part of the gate. Weakens "one verified
  candidate is the landed content" (D66 (b)).
- Rejected: needs data the squash discards.

**B. Chain-aware landing rule (recommended).** Each job records the fingerprint of its base. The gate
accepts one commit that adds the closing records of jobs J1..Jn when they form one linear chain from the
parent's fingerprint to the landed tree's fingerprint, and every job passes every per-job check except
"fingerprint equals the landed tree", which applies only to the final job.
- For: uses only what a squash keeps (records and the final tree). The landed content is exactly content that
  a verifier judged in full: the final job's verifier saw the whole tree. Works for squash, rebase, and merge.
  No exemption. Strictness for a single job is unchanged (a chain of one is today's rule).
- Against: a new field in `candidate.json` (a schema change, since unknown fields are rejected, so
  `athena.candidate/2` or an optional field with a version bump). Intermediate states are not present as trees, so
  they are proved only by the records' agreement (the same unsigned-records limit as §7.3's timestamp proof).
  Jobs landed before the field exists cannot form a chain.

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
- Against: earlier jobs' records could be fabricated or unrelated, and nothing says they were part of the
  tree's history. Rejected: it drops the link that makes the earlier jobs mean something.

## 4. Recommendation: B

B is the only option that removes both the restriction and the exemption without requiring data the squash
destroys. It keeps the final tree checked against a verification of exactly that tree, and it makes the
earlier jobs' contribution checkable by a hash chain of fingerprints.

### 4.1 Records that link a chain

- New field in `candidate.json`: `base_fingerprint`, the fingerprint of the base commit's tree under the
  base commit's own rules, computed by the same `fingerprint` function (`athena_git::attrs`) the gate uses.
  It is written by the core when it builds the candidate, in the same place that already writes
  `base_commit` and `fingerprint` **(inferred: not yet located in the core)**.
- The chain link is: for consecutive jobs Ji and Ji+1, `Ji+1.base_fingerprint == Ji.fingerprint`. The first
  job's `base_fingerprint` equals the fingerprint of the squash commit's first parent. The last job's
  `fingerprint` equals the fingerprint of the squash commit's tree.
- `verification.json` already holds `verified_fingerprint` and `candidate_commit`; no change. Per job it must
  equal that job's own `candidate.json` `fingerprint`.

### 4.2 The check, exactly

Let X be the commit, P its first parent, R the rules of P, and F = fingerprint(X; R). Let `jobs` be the
closing jobs X adds (`closing_jobs`). If `jobs` is empty, nothing changes. Otherwise:

1. If `jobs` has one member, run today's rule unchanged (this includes the fingerprint equality at lines
   349-359). A chain of one is not special.
2. If it has more than one, load each job's files from X's tree and require each `candidate.json` to hold a
   `base_fingerprint`.
3. Order the jobs by the links. Exactly one job must have `base_fingerprint == fingerprint(P; R)` (the head).
   Repeatedly pick the unique job whose `base_fingerprint` equals the previous job's `fingerprint`. The order
   must use every job exactly once, with no fork, no cycle, and no job left over.
4. The last job's `fingerprint` and `verified_fingerprint` must equal F.
5. For every job, run every per-job check `landing` runs today, using that job's records, except the
   fingerprint equality of step 4, which for earlier jobs is replaced by: `candidate.fingerprint ==
   verified_fingerprint` and both equal the fingerprint the next job names as its base.
   These are: an accepted reading and a prediction that precedes that job's own work (`prediction_problem`),
   a passing final verdict, matching candidate commits, a successful verifier invocation with the job's
   verifier model, reconciled usage and outcome totals, a fresh landed outcome (or `outcome-landed.json`
   after blocked), and no earlier landing of the job in the parent or in the history walk.
6. Trailers stay hints. `Athena-Verified-Content` must equal F. An `Athena-Job` naming a request job must
   name a member of `jobs`.

### 4.3 Refusal messages

Each is returned through the existing `refuse`, which reports a violation on the commit:

- `a chain landing needs base_fingerprint in <dir>/candidate.json` (step 2)
- `no job in the commit starts from the parent's content <P-fingerprint>` (step 3, no head)
- `jobs <a> and <b> both start from the same content <fp>, so they do not form one chain` (a fork)
- `job <x> is not linked to the others: its base content <fp> is not any job's result` (a leftover or cycle)
- `the last job <x> holds <fp>, not the landed content <F>` (step 4; the existing per-record messages for
  `candidate.json` and `verification.json` apply the same way)
- the existing messages for each per-job failure (steps 5-6), which already name the job's directory.

### 4.4 Why "nothing reaches main unverified" still holds

The landed tree's governed content is exactly the final job's verified fingerprint (step 4). That verifier
judged the whole candidate, including everything earlier jobs had added, because those were part of its
base. Each earlier job is tied in by the fingerprint chain from the parent to the final state, so the landed
content is reached from the parent only through states that a recorded, passing verification names.
**(inferred from the design of the fingerprint: equal fingerprints mean byte-identical governed content,
§7.3.)** Governed content that matches no job's chain makes the final fingerprint differ and the commit is
refused.

What it does not prove: that the intermediate trees existed as such, or that a verifier looked at each
intermediate state, beyond what its unsigned record says. The records are written by Athena and not signed;
someone who forges every record consistently forges the chain. This is the limit §7.3 states for the
timestamp proof, and it is no weaker here **(read, §7.3)**.

### 4.5 Why M23 (prediction precedes the outcome) still holds

Step 5 runs `prediction_problem` for every job against that job's own records, so each job's accepted
reading is recorded before its own dispatch and before every one of its producer and verifier invocations.
The chain adds an order between jobs (a job's base is another's result) but changes nothing about
per-job order, so a reading written after its own work is still refused. A reading is not required to
precede the *previous* job's work: each job predicts its own work.

### 4.6 Other cases

- *Parent moved.* If other governed changes reached the base branch after the chain began, the head's
  `base_fingerprint` no longer equals the parent's fingerprint and the squash is refused ("no job starts from
  the parent's content"). That is correct: the squash would not equal the chain's final content anyway. The
  adopter rebases the chain, which makes the jobs stale (design §7.3 landing algorithm).
- *Records-only commits in the pull request* do not change the governed fingerprint and do not break links.
- *A job that blocked then landed* (`outcome-landed.json`) is handled by step 5, as today.
- *Rebase merge and merge commit* keep today's single-job commits and never reach the chain path.

## 5. Follow-up work (not done here)

1. `crates/athena-data`: add `base_fingerprint` to `Candidate` with a schema version, and `docs/records.md`.
2. `crates/athena-core`: write it when the candidate is built.
3. `crates/athena-gate/src/invariant.rs`: replace the single-job refusal in `landing` with the chain check
   of §4.2; factor the per-job checks so they can run per job. `landing_job` and the history walk
   (`HistoryState::landed`) must record every job of the chain as landed.
4. `crates/athena-gate/src/landing_tests.rs`: tests that a valid chain squash passes, and that each refusal
   of §4.3 fires: missing `base_fingerprint`, wrong head, a fork, an unlinked job, a last job not matching
   the tree, and a reading written after one job's work.
5. Update design §7.3 (merge, squash, and rebase merges) and `docs/enforcement.md` (the M23 and landing rows)
   once the check exists, and settle D69 as accepted or changed.

## 6. This change

This change adds this document, the decision D69, a changelog line, and pointers in `docs/design/athena-v0.md`,
`docs/enforcement.md`, and `docs/architecture.md`. It modifies no Rust code, `athena.toml`, workflow, or other
governance file, and nothing under `records/`. D60-D68 stay in force; D69 supersedes none of them until it is
accepted.
