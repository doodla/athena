# A history of its own for Athena's records

Status: accepted, design only (D111), under Sri's delegation in the `BRIEF.md` addenda of 2026-10-23. Nothing
here is built. Audience: Sri, who decides, and anyone checking the design.

Three changes were decided in those addenda after the first draft. (1) The records history is a branch named
`athena/records` with its own root, not the custom ref `refs/athena/records` the first draft proposed (§3).
(2) §13 lists the readers of `records/` and how each reads from the branch. (3) The pre-merge status depends on
the host requiring branches to be up to date before merging (§5.1). The open points are filled: pending window
15 minutes, cutover after build job 6, two weeks between job 7 and job 8.

Sri's words (`BRIEF.md`, addendum of 2026-10-23): "Wouldn't a db or a shadow repo solve this entire class of
issues?" and "Yeah. Also copy over whatever. We shouldn't depend on trailers in commit messages".

Sources read for this document: `BRIEF.md` (A4, A5), `athena.toml` `[history]`, `docs/design/athena-v0.md`
sections 7.3 and 7.4, `docs/design/native-pr-landing.md` and D72 in `docs/decisions.md`, `docs/enforcement.md`.
Figures marked "reported" come from the request and were not re-run here. Figures marked "estimate" are mine.
No command that times the check was run.

## 1. The problem

Records ride in the same commits as code, and that keeps breaking. As reported and as `athena.toml` records:

- Six `[history] exempt` entries: 49c6ac1 (predates Athena), 64ecdc9 (a second root that joined main through
  the bootstrap merge), and four squash merges: ee4b0dd (PR #8), f2a8453 (PR #9), 5a73478 (PR #14),
  39962ed (PR #130). The comments in `athena.toml` give each reason; for the squashes it is that one commit
  carried several jobs' landings, or records with no named job, so the landing check could not tie content to a job.
- Main's history check is red again on the squash 02837e0 (reported).
- A revert of code fails the check, because a revert deletes records, and records are append-only.
- A job's landed outcome is written before the squash exists, so it cannot name the commit that lands. The
  workaround is the `Athena-Verified-Content` trailer and the fingerprint rules of §7.3.
- Every landing forces `land` to rebuild the work branch, so records and code agree.
- Most of the check's time (about 65 s locally, reported) is spent validating about 4,000 records (reported)
  at the tip.

The common cause: the record of what happened has to be in the same commit as the thing it describes, and a
commit cannot contain its own hash. Records written after the merge could name it. They must live somewhere
main's commits do not.

## 2. Requirements the design must meet

- A4: Athena "can be lifted out and adopted by someone else, with its history intact", and nothing in it
  depends on this environment.
- A5: job records are "machine-readable and live in the repo"; "every commit traces to the work that produced
  it"; nothing reaches main without verification; "published history on main is never rewritten"; secrets are
  "scanned on every commit and across the full history".
- D72: in a repository Athena does not own (stealth mode) its records stay outside it, in Athena's state
  directory, and the pull request holds only the change.
- Sri: no dependence on commit-message trailers.

## 3. Options

Each option is weighed against A4, A5 and D72.

| | A records branch in this repository, `athena/records` (own root, never merged into main) | A second repository | A database |
|---|---|---|---|
| A4: lift out with history intact | Good. One repository. `athena/records` is an ordinary branch, so a plain clone carries both histories with no special refspec. Lifting out means pushing two branches. | Weaker. Two repositories to move, each with its own access and secrets. The link between a code commit and its record spans repositories, so it can break in transit. | Weaker. History is in a file or server, not in git. Lift-out needs a dump format and a loader. Intact history means the database's own audit log, which Athena would have to build. |
| A5: machine-readable, in the repo | Good. The same JSON files under `records/`, diffable, in the repository's object store. | Partly. Machine-readable, but not "in the repo". | Fails as written unless exported back into the repo, which recreates the problem. |
| A5: every commit traces to its work | Good. A landing record on the same repository's branch names the commit; reachability is by parent link (§6). | Cross-repository names only. No shared garbage-collection or reachability. | Names only. |
| A5: append-only | Git's fast-forward-only branch update, enforced by the host's rule forbidding force-pushes, plus the existing check on the records history. | Same, in the second repository. | Must be rebuilt: triggers or permissions, and a story for the administrator. |
| A5: secrets across full history | The existing gitleaks scan, pointed at two branches. | Two scans, two configurations. | A new scanner over rows, or an export. No history to scan. |
| A5: published history never rewritten | Same rule on both branches. | Same. | Not applicable; needs its own equivalent. |
| D72 stealth mode | The same branch in a local bare repository in Athena's state directory (§5.4). Same code path, different location. | Natural fit: the second repository is local. But it is the same thing as the bare repository, with a second remote added for the owned case. | Natural fit: a local file. |
| New moving parts | One branch, a writer, a checkpoint file. | A second repository, a second credential, cross-repository atomicity. | A dependency, a schema migration story, a backup story, a failure class git does not have. |

### Recommendation

**A records branch in this repository: `athena/records`, with its own root, never merged into main.**
In D72 stealth mode the same branch lives in a local bare repository in Athena's state directory.

Why a branch and not the custom ref `refs/athena/records` the first draft proposed: the host's branch rules can
forbid force-pushes and deletion on a branch, which protects append-only at the host, and a plain clone carries a
branch, where a custom ref needs a special refspec or a mirror clone and a host may refuse to accept it.

Reasons: it is the only option that keeps A5's "live in the repo" and full-history scan with no new service,
it keeps A4's single-repository lift-out, and git already gives fast-forward-only refs for append-only and
parallel-writer safety. A second repository's one advantage, D72's separation, is met by the local bare
repository. A database fails A5 as written and replaces mechanisms git gives for free.

Requirement on the host, replacing the first draft's assumption that the host accepts `refs/athena/*` pushes:
branch rules on `athena/records` that (a) forbid force-pushes, (b) forbid deletion, and, on main, (c) require
branches to be up to date before merging (§5.1). These are host settings, configured by the user; this design
does not create them. If the host cannot enforce (a) and (b), append-only rests on the records check alone and
the records branch is not protected against a force-push until the rules exist.

Rejected: keep records on main and add more exempt entries. That is the status quo that produced six
exemptions and a red check on 02837e0.

## 4. Migration and the cutover

**Copy, do not rewrite.** Published history on main is never rewritten (A5), so main keeps every commit and
every record it has.

1. Freeze: pick the cutover commit `C0` on main, after build job 6 has passed (decided in the 2026-10-23
   addenda), and record its full sha. Job 7 is the cutover job.
2. Import: one commit on the new root writes every file of `records/` at `C0` to the same paths. Alongside
   them it writes `import-manifest.json`: each source path, its git blob id at `C0`, and the SHA-256 of its
   contents. The importer then re-reads the new commit and fails if any path or hash differs from `C0`, or if
   the count of files differs. The "reported ~4,000" records are counted by that step, not assumed.
3. The manifest is itself a record, so completeness is checkable at any time by anyone.
4. The per-job history of how each record reached main stays in main's commits; the records branch holds the
   result, not a replay of every commit. A reader who wants the original commit uses the manifest's `C0`.

**What happens to `records/` on main afterwards.** In three stages, each its own job (§10):

- Stage 1, from the import: `records/` on main is frozen and unchanged; new records go only to the records
  branch. The check on main stops requiring records for new commits only after the rule of §5.3 is in force.
- Stage 2, after the new check passes on main for two weeks between job 7 and job 8 (decided in the
  2026-10-23 addenda): a late job removes `records/` from main's
  tree. This is an ordinary forward commit; the old files remain in main's history. Because the history check
  judges pre-cutover commits by the frozen legacy rule below, the removal does not trip the append-only rule.
- A revert of code no longer deletes records, because records are not in main's tree. This ends the revert
  failure.

**How the check treats commits before the cutover.** A frozen legacy rule, applied to commits that are
ancestors of `C0` (and `C0` itself):

- The existing history-check logic runs unchanged for them, as a frozen module, with the exempt list moved into
  the records history as `legacy-exempt.json`. It contains the six entries (49c6ac1, 64ecdc9, ee4b0dd,
  f2a8453, 5a73478, 39962ed) and also 02837e0, with the reason for each taken from the `athena.toml` comments.
  It is a record, so it is append-only; adding to it after the cutover is refused.
- Any pre-cutover commit not named by the legacy list is judged exactly as today.
- Post-cutover commits are judged only by the records-based rule of §6. No `athena.toml` exemption is ever
  needed for them. The `[history] exempt` table stays in `athena.toml` until stage 2, then is emptied by a
  governance job (a change to a governance file, so the accepted reading must list it).
- Because `C0` is chosen after job 6, 02837e0 is expected to sit before it, so the legacy list covers it. If
  02837e0 is somehow after `C0`, the design needs a landing record for it written by hand as a declared
  intervention (§6), which is what that record type is for.

## 5. How main is protected without trailers

### 5.1 Two gates, in order

Content cannot be judged only after the merge, because by then main is changed. Attribution of the host's
commit cannot be known before the merge. So there are two gates:

1. **Before the merge: a required status on the pull request's head.** It verifies the head's content, that
   each commit of the pull request is a verified landing or a declared intervention from the squash's parent
   (§6), and that the prediction record precedes the outcome (§7). The host's branch protection makes it
   required. This is where "nothing unverified reaches main" is enforced.

   **Dependency on up-to-date branches.** The status judges each commit "from the squash's parent". That parent
   equals the head's base only if the host requires branches to be up to date before merging, so main cannot
   move between the status and the merge. This design therefore requires that host rule (§3). If the host does
   not require it, the squash's parent can differ from the head's base: the status verified the head against a
   base the merge did not use, the §6 squash rule cannot be proved from the status, and the landing is treated
   as pending (§5.2) and then fails as unattributed. Recovery: Athena re-verifies the head against the actual
   parent after the merge and writes the landing record, or the user declares an intervention.
2. **After the merge: the attribution check.** It runs on the push to the records branch and on the push to
   main. It requires every commit on main's first-parent line after `C0` to be named by a landing record or a
   declared-intervention record.

### 5.2 The gap and the pending state

Between the host's merge and the landing record there is a gap. Without a rule main would go red on every
landing, which is the class of problem this design ends. The rule:

- Before merging, Athena writes a **landing-intent** record: PR number, head sha, head tree, the expected
  base. This is the only thing the check needs to recognize the incoming commit.
- A main tip with no landing record but a matching landing-intent record is **pending reconciliation**. The
  check passes with a notice for a bounded window of 15 minutes (decided in the 2026-10-23 addenda).
  After the window it fails as unattributed. Failing closed stops this from being an unbounded hole.
- A tip with no matching intent fails at once.

Rejected: failing immediately (red on every landing); skipping the newest commit (an unbounded hole).

### 5.3 Records the design adds

All are per-record JSON files on the records branch, never edited:

- `landing-intent`: PR number, head sha, head tree sha, expected base (main's tip when the intent was written),
  job ids.
- `landing`: PR number, head sha, **the commit the host made on main**, the merge method, the list of the pull
  request's commits (sha, tree) in order, each with the job and verification record that covers it, and the
  intent it fulfills. Written after the merge.
- `intervention` (declared): a human commit on main or in a pull request, with who declared it, the reason and
  the commit sha. This is today's idea of a declared intervention made a record of its own.
- `checkpoint`: see §8.

A landing record names commits by sha, not by trailer. The check never reads a commit message for attribution.
The `Athena-Verified-Content` trailer and the fingerprint-in-message rules are dropped for post-cutover commits;
the content equality they were protecting is now a tree comparison against the recorded head (§6).

## 6. Attribution of every commit on main, from records alone

The check walks main's first-parent commits after `C0`, oldest first. For each commit `M` with parent `P`:

- **Direct commit** (a merge commit or fast-forward of a verified commit): `M` is named by a landing or
  intervention record, and every commit it brings is a verified landing or declared intervention from `P`.
- **Squash:** the commit is valid when `tree(M)` equals `tree(head)` of the pull request named by the landing
  record, and each commit of that pull request is a verified landing or a declared intervention, each applied
  from the squash's parent `P`. This is the pull-request-level rule: the unit judged is the whole pull request
  head, not the squash's message or a single job. It is what ee4b0dd, f2a8453, 5a73478 and 39962ed needed and
  the old per-job rule could not give: several jobs in one squash are fine so long as each is recorded and the
  whole equals the head that was verified.
- **Unnamed commit:** unattributed, so a failure (or pending, §5.2).

The host's merge commit or squash commit is named by the landing record's "commit the host made" field, which
is why the record is written after the merge.

### 6.1 Reachability without the host

The pull request's commits must stay reachable after the host forgets them, or the old per-commit rule cannot
be rechecked. One mechanism: **each landing record's commit on the records branch takes the pull request's head
as an additional parent.** The records commit's tree is its first parent's tree plus the new files; the
second parent only carries reachability.

- Garbage collection keeps everything reachable from the branch `athena/records`. The pull request's commits
  survive with no host ref.
- A plain clone or fetch of that branch carries them, which keeps A4's lift-out intact.
- In stealth mode the local bare repository holds the same branch and objects.
- Reading the records history uses first-parent, so the extra parents do not change record order.

Size: the pull request's objects are already in the repository's object store, so the second parent adds a
commit pointer, not new bytes. Estimate: a few hundred bytes per landing; to be measured in job 4.

Rejected: one keep-ref per pull request in a custom namespace (must be mirrored separately and can be forgotten or pruned); commit
ids stored as tree entries or gitlinks (not reachable for garbage collection); the host's `refs/pull/N/head`
(host-dependent and absent in stealth mode).

## 7. Guarantees, and where each is enforced after the change

| Guarantee today | Where it is enforced now | Where it is enforced after |
|---|---|---|
| Records are append-only (D06, D07; M4): no existing record path modified, deleted or renamed | The gate in three places and `athena check --history`, on main's tree | Two rules on `athena/records`: the update must be a fast-forward (the host's branch rules forbid force-pushes and deletion), and the check rejects any commit on that ref that modifies, deletes or renames an existing path. The legacy list is a record, so append-only covers it. Main no longer holds records after stage 2, so a revert cannot violate it. |
| Nothing unverified reaches main (A5) | Landing algorithm under the repository lock, plus the history check's fingerprint match on main | The required status on the pull request's head (§5.1, step 1) verifies content and each commit's verification before the merge. The attribution check after the merge (§5.1, step 2) proves every first-parent main commit is named. Both read records from the records branch. |
| The prediction precedes the outcome (A2; M23) | The `Athena-Prediction` trailer and a CI check that the prediction commit is an ancestor | Records history order: the pre-merge status requires the accepted-reading (prediction) record to be an ancestor of the outcome record on the records branch. This is by commit order on the records branch, not a trailer. |
| Secrets never enter any commit (M1, M3) | Pre-commit gitleaks run; `athena check --history` over every commit | The same configuration, run at commit time on the records branch writer, and in the history check over both `main` and `athena/records` (which reach the pull request commits through the second parent). Full history scan is part of the periodic replay (§8). |
| Personal data never reaches committed content (M22) | Redaction before recording; personal-data gate on added lines | Unchanged in the writer: redaction before the record is written, and the personal-data gate on added lines for both branches. |

Everything else in §7.4 of the v0 design (valid records, size, reconciliation of costs) keeps applying; it moves
from "added files in a main commit" to "added files in a records-branch commit".

## 8. Reconciliation and parallel writers

### 8.1 A crash between the host's merge and the landing record

The crash window: the host merged, the process stopped, no landing record exists. The landing-intent record was
written first, so the facts for repair are local.

A reconcile step runs at the start of any Athena command and inside the check:

1. List main's first-parent commits after the last checkpoint without a landing record.
2. **Local first.** For each, look for a landing-intent record whose head tree equals the commit's tree and whose
   expected base is the commit's parent. A match writes the landing record (with the host-made commit sha).
   Nothing is asked of the host.
3. **Host only as a fallback.** If no intent matches, ask the host which pull request merged. If the host is
   unavailable, report "unattributed" and write nothing. Athena never guesses.
4. A squash where the tree matches no intent is a human commit to main: it is reported, and a user declares it
   as an intervention, or it fails.

Pending reconciliation (§5.2) bounds how long main can be red-free without a landing record.

### 8.2 Parallel writers

Several jobs may write to the records branch at once.

- Each writer builds its commit on the tip it last fetched and pushes with a **compare-and-swap**: the update
  succeeds only if the ref still points at that tip (fast-forward-only).
- On rejection the writer fetches, **replays its new job-keyed files on the new tip, and retries**. A bounded
  retry count (an example: 5) then reports failure rather than loop.
- Files are per-record and keyed by job id (and record id). A shared append-only file, if any, becomes
  one-file-per-record so two writers never touch the same path and replay cannot conflict.
- Landing records are the one place two writers could contend on meaning (two pull requests merged in quick
  succession). The landing record's expected base makes the order explicit: the second is checked against the
  first's host-made commit.

Local repositories use the same ref update with a lock file git already provides.

## 9. Keeping the check's cost independent of history size

Today the history check validates about 4,000 records at the tip (reported), about 65 s locally (reported).
Cost grows with the number of records. New design:

- A `checkpoint` record stores: the last passing main sha, the records-branch sha at that point, and the
  verifier version. It lives on the records branch.
- Each check judges only main's commits and records **new since the checkpoint**. Cost depends on the number
  of new commits, not total history. After a pass the check writes a new checkpoint.
- A scheduled **full replay** (weekly and whenever the verifier version changes) judges everything from the
  root and must reproduce the checkpoint (same pass, same sha). A mismatch fails the replay and invalidates the
  checkpoint until repaired. This bounds trust in the checkpoint.
- The check is sound against a forged checkpoint because it is a record on an append-only ref and the replay
  recomputes it.

Measuring: record, per run, seconds and the number of records and commits judged. Plan, to be run in job 6:

1. Baseline: time a cold full check on the current main, and count records judged (compare with the reported
   65 s / 4,000).
2. Incremental: time a check with the checkpoint one landing behind; the records judged must equal the records
   that landing added (a handful, not 4,000).
3. Time it again after adding, say, 100 synthetic landings to a scratch copy; incremental time must not scale
   with the 4,000 old records. Estimate: seconds, to be replaced by the measurement.
4. The full replay is expected to remain at roughly the baseline, because it re-reads all records.

## 10. Build plan

Each job is sized for one producer run (one seam and its tests). Order keeps main attributed at all times: the
new records and the attribution rule exist and pass before `records/` leaves main's check, and the old check
keeps running until the new one has passed alongside it.

1. **Schema and writer.** New record types (`landing-intent`, `landing`, `intervention`, `checkpoint`,
   `legacy-exempt`, `import-manifest`) in `athena-data`; a writer that creates the records ref with
   compare-and-swap and retry (§8.2), writing the branch `athena/records` (own root). Test: two simultaneous
   writers both land; a rejected push retries. Also confirm the host rules of §3 are in force on
   `athena/records` (no force-push, no deletion) and on main (branches up to date before merging).
2. **Importer.** Copy `records/` at `C0` to the new root with the manifest and the self-check of §4. Test:
   a missing, changed or extra file fails the import. Run once into a scratch repository first; main is
   unchanged.
3. **Landing intent, landing record and reconcile.** Write the intent before the merge, the landing record
   after, with the PR head as extra parent (§6.1); the reconcile step (§8.1). Old trailers still written.
   Test: crash between merge and record is repaired locally; host unavailable reports unattributed.
4. **Pre-merge status.** The required status on the PR head (§5.1 step 1), and the stealth-mode local bare
   repository. Test: a head with an unverified commit fails the status; PR commits are reachable after the
   host ref is deleted (garbage collection run).
5. **Attribution check, alongside the old one.** First-parent walk, squash rule, pending state, legacy rule
   (§4, §5, §6). It runs in CI as a second, non-blocking check beside the existing one, to compare verdicts
   on real history. Test: each of the seven named commits (six exempt entries and 02837e0) is judged as §4 says.
6. **Incremental check and replay.** Checkpoint record, new-since-checkpoint judging, weekly full replay and
   the measurement of §9. Test: the incremental count equals new records; a tampered checkpoint fails replay.
7. **Cutover.** `C0` is chosen after job 6 passes. Make the new check the required one and the old one retire
   for post-cutover commits; stop writing trailers; `records/` on main frozen. Also move the readers of §13:
   keep a worktree of `athena/records` at a fixed path in Athena's state directory, point the two reader seams
   at it (a branch-tip treeish for `Records::load` and `RecordSet::load`; the worktree path as the root for
   `read_records` and the walltime reader). Test: a revert of code passes; every reader of §13 returns the same
   result from the branch as from `records/` at `C0`. If this is too large for one producer run, split it inside
   job 7 (cutover, then readers) without renumbering.
8. **Removal.** Two weeks after job 7, remove `records/` from main's tree and empty `[history] exempt`
   (a governance job). Test: the check on a new main passes with no `records/`; legacy commits still judged.

Main is never unattributed: until job 7 the old rules remain required and the new records are written in
addition; job 7 changes the required check only after job 5's comparison showed the same verdicts. Readers keep
reading `records/` from the working tree until job 7 moves them, and `records/` stays on main until job 8, so
no reader is left without records.

## 11. Reversal cost

- Before job 7: low. Delete the branch and the new writers; main and its records are untouched because nothing was
  moved or rewritten (migration is a copy).
- After job 7, before job 8: medium. Main still holds all records up to `C0`; records written since live only on
  the records branch and would need a copy back onto main as a normal commit, then the old check re-enabled.
- After job 8: higher. Re-adding `records/` to main from the records branch is a forward commit, but the old
  check's trailer rules would need to be restored for the commits made in between. The records branch remains,
  so no data is lost.

## 12. Points still for the user

Decided in the 2026-10-23 addenda and no longer open: pending window 15 minutes (§5.2); `C0` chosen after build
job 6 (§4, §10); two weeks between job 7 and job 8 (§4, §10, §11). The host-accepts-`refs/athena/*` question is
gone with the custom ref; in its place the user configures the host rules of §3 (no force-push or deletion on
`athena/records`; branches up to date before merging on main). Whether the host offers those rules is not
visible from the code and is checked in job 1.

## 13. Readers of `records/` today, and how each reads after the cutover

Found by searching the code for reads of `records/` (function names and files; no line numbers, which drift).
Nearly all readers go through one of two seams, so the move is two edits plus the worktree:

- **Seam A, git tree reads.** `Records::load(git, treeish)` (`crates/athena-core/src/store.rs`) and
  `RecordSet::load(git, treeish)` (`crates/athena-gate/src/records.rs`) read `records/` from a git tree, today
  at the repository's HEAD. After the cutover the treeish is the tip of `athena/records`.
- **Seam B, filesystem reads.** `read_records(root)` (`crates/athena-core/src/introspect.rs`) and the walltime
  reader read `<root>/records/...` as files. After the cutover `root` is a worktree of `athena/records` kept at
  a fixed path in Athena's state directory (for example `<state>/records-worktree`), updated to the branch tip
  before a read.

| Reader | Reads now | Reads after the cutover | Job |
|---|---|---|---|
| Status | `status.rs`: `Records::load` and `RecordSet::load` at HEAD (also `records/resolutions/...` paths it formats) | Seam A: branch tip as treeish | 7 |
| Forecasting and the reference forecaster | `scoring.rs`: `history(&Records)` and `reference(priors, class, &Records)`, fed by `Records::load`; the arithmetic is in `athena-score` (`reference_forecast`) | Seam A: unchanged code given records loaded from the branch tip | 7 |
| Scoring | `athena-score`: `score_records`, fed by `Records` loaded as above | Seam A | 7 |
| Retry carry | `reading.rs`: `carry(job, redactor)` over the `JobRecords` of a `Records::load` | Seam A | 7 |
| Audit | `auditor.rs`: `Records::load(&athena.git, &head)` | Seam A | 7 |
| Introspection | `introspect.rs`: `read_records(root)` reads `records/jobs`, `interventions`, `host-errors` as files; its `GitRecords::load` callers use seam A | Seam B for `read_records` (worktree path as `root`); seam A for the rest | 7 |
| Study site generator | `xtask/src/site.rs` reads `STUDY_DIR` = `studies/firstmate-work-handling` (the `load` calls); the search found no read of `records/` in it | No change: it does not read `records/`, so no job moves it | none |
| Wall time (not in the request's list) | `xtask/src/walltime.rs`: reads `records/jobs` under a directory | Seam B | 7 |
| Other `Records::load` callers | `review.rs`, `run.rs`, `manage.rs`, `proposals.rs`, `work_decisions.rs`, `watcher.rs`, `audience.rs`, `gh.rs`, `steward.rs` | Seam A | 7 |
| Gate history and reconcile | `athena-gate`: `history.rs` and `reconcile.rs` call `RecordSet::load` on a tip or parent | Seam A, with the legacy rule of §4 for pre-cutover commits | 5 (alongside), 7 |
| Others reading files under `records/` | `labeled.rs` (`root/records/`) and `steward.rs` (copies `records/` into a read-only workspace) | Seam B: root is the worktree; the steward copies from it | 7 |

The study site generator row differs from the request, which listed it as a reader; the code read shows
otherwise, and the table says what the code does. Until job 7, all rows keep reading `records/` on main, which
is frozen from the cutover and removed in job 8, after the two-week period.
