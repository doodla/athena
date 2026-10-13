# Review 2 of `docs/design/parallel-jobs.md`

**Reviewed:** the design as revised after review 1 (status line: "revised after review 1 ... findings F1-F8"),
against the source and `docs/design/native-pr-landing.md` (the "native design" below).
**Independence:** I formed the verdict from the revised design and the code. I used review 1
(`docs/design/reviews/parallel-jobs-review-1.md`) only as the list of findings F1-F8 to check, not as evidence.
**Method:** reading only. No command was run and no behavior was reproduced. Claims are labelled **(read)**
when taken from the file and lines named beside them, **(design)** when they are what the design proposes,
**unchecked** when I did not read the code, and **estimate** when not measured. Line numbers are those of the
repository at the time of review. This review does not edit the design; fixing it is a separate job.

## Verdict: accept with changes

All eight findings of review 1 are answered in the text: seven resolved, none unresolved, F3 resolved with a
caveat that I list as N2. The revised shape (one child process per job, per-job pull requests, a docs job that
runs alone) matches the code I read. The revision introduces one High problem and several smaller ones, all in
the new landing and launcher text. They can be fixed in the text; none needs a change of direction. The changes
are listed in section 4. No build should start before N1 is settled.

## Severity scale

**High** blocks the build or makes a central claim false. **Medium** must be settled in the design or the first
build job. **Low** is a correction or a later job.

## 1. Review 1's findings, checked against the revision

Status values: resolved, partly resolved, unresolved.

### F1 (High): disjoint jobs go stale after the first lands. Status: **resolved**

- **Design's answer:** section 7, "What this revision drops", says `plan` replays only over records-only
  changes (`land.rs` 55-64) and returns `Stale` otherwise (65-74), "even when the two jobs' files are
  disjoint". Section 1 narrows the promise to overlap of the slow phases. Question 2 (section 11) and the build
  order (section 10 item 3) put the landing mechanism before the pool.
- **Source check (read):** `crates/athena-core/src/land.rs` `plan`, lines 53-74: `head == p` takes the
  candidate's tree; `fingerprint(head; p) == fingerprint(p; p)` calls `merge_tree`; the `else` returns `Stale`
  with `first_governed_change` naming the commit. The design's line citations match. The module header (lines
  3-11) states the same rule.
- **Native design (read):** §5.7 says per-job pull requests are "option (b)", that automatic re-verification is
  not supplied, and that "most such merges from a moved base will not be reported as landed" (lines 410-426).
  §5.4 gives `merged-unverified` (line 342). Section 7 of the design uses both correctly.
- **Remaining gap:** the re-verify step the design leans on ("on the user's request", §5.7 points 1-3) is
  design text only. A search of `crates/` for re-verify found no implementation, only a comment at
  `crates/athena-core/src/pr.rs` line 33. The design says so itself (section 7, "Whether the build of that
  request exists was not checked here") and makes it a prerequisite (section 10 item 3). That is honest, so F1
  is resolved; the new landing text has its own problem (N1).

### F2 (High): threads are not shown to be safe. Status: **resolved**

- **Design's answer:** section 4.1 recommends one child process per job; threads return only if a spike shows
  four things (a to d). Section 5.1 and 5.3 isolate failure per child.
- **Source check (read):** `AgentDriver` has no `Send`/`Sync` bound (`crates/athena-agent/src/lib.rs` line 139);
  `Ui` has none (`crates/athena-core/src/lib.rs` line 56); `Athena` is at line 69. `Capability` and `Pinner`
  do (`capability.rs` line 86, `pin.rs` line 20). `run.rs` line 1128 uses
  `std::cell::RefCell` for the running score. All as the design states.
- **Process model check (read):** `Terminal::line` writes progress to standard error (`main.rs` lines
  152-158), so a child's standard output holds only the result. `go_and_report` prompts only when both standard
  input and output are terminals (line 225), so piped children never prompt. The design's launcher step 2
  (section 4.2) relies on exactly this and is right.
- The spike conditions are specific and testable. I found nothing to add.

### F3 (High): shared append-only files collide. Status: **resolved**, with N2

- **Design's answer:** section 9 adds four rules (docs job alone writes the three files; numbers allocated by
  the docs job at write time; serialization by declared path; the landing check as authority), a table of what
  is enforced and what is a hint, and a four-part collision test. The roadmap's done-when for parallel jobs
  (`docs/design/roadmap.md` lines 47-49) is cited and the test matches it.
- **Source check (read):** the landing check works as the design says. `plan` bails when the landed
  fingerprint differs from the verified one (`land.rs` lines 75-81), and a job whose shared file moved is stale
  by lines 65-74. So rule 4 and test items 2 and 4 are backed by code.
- **Declared path list:** the design says "a job's brief names the shared files". A structured field already
  exists: `Reading.deliverables` (each with a `path`), built in `crates/athena-core/src/reading.rs` lines
  391-455 and used in `admission.rs` line 109. The design should name it (N6). Admission checks the listing
  only for governance changes (`governance_change_is_listed`, `admission.rs` lines 107-115, the part I read), so
  an undeclared edit to `CHANGELOG.md` is not stopped before landing. The design's "hint" label is therefore
  accurate.
- The design is honest that rules 1 and 2 are prose, not code. That is acceptable for a design; see N2 for the
  practical conflict.

### F4 (Medium): duplicate job refs. Status: **resolved**

- **Design's answer:** section 6.1 resolves each ref to an id (`Records::resolve`), compares ids, and exits 1
  naming the duplicate before starting anything. Section 4.4 says `locked` now means a process outside the run.
- **Source check (read):** `run.rs` lines 114-118: `go` resolves the ref with `records.resolve`, then calls
  `try_lock_job`, which returns `None` on `WouldBlock` (`state.rs` lines 75-82). The launcher's pre-check
  removes the race review 1 described. Two refs to one job are caught because the comparison is on ids.

### F5 (Medium): kill switch before the job lock. Status: **resolved**

- **Design's answer:** section 5.1: the launcher checks before resolving and before each spawn; the child keeps
  the existing order.
- **Source check (read):** `run.rs` line 111 (`refuse_if_stopped`) comes before line 116 (`try_lock_job`).
  `lock_file` creates the file and its parent directories even when the caller then refuses (`state.rs` lines
  55-65). The same order holds in `land_pr`: `gh.rs` line 607 (refuse) before line 611 (job lock). The design's
  rule is correct and already true in both entry points.

### F6 (Medium): worst-case spend. Status: **resolved**

- **Design's answer:** section 5.2: budgets stay per job; the launcher prints N times the per-job cap and the
  number of queued jobs; no run-level cap in the first build (question 3).
- **Source check (read):** `check_budget` compares a step's cap with `job_usd - spent` for that job alone
  (`crates/athena-core/src/reading.rs` lines 159-176); `job_usd` is in `crates/athena-data/src/config.rs` line
  64. So per-job caps hold in each child without shared state, and the printed worst case is arithmetic on
  config the launcher can read. The design correctly gives no dollar figure.

### F7 (Low): line references. Status: **resolved**

- **Design's answer:** section 3 corrects the references and says to cite function names first.
- **Source check (read):** every reference I checked is right at this commit: `refuse_if_stopped` calls at
  `run.rs` 111 and 698; `land_or_block` at 1165 (lock 1178, loop 1181); `retry_landing` at 1344 (lock 1424,
  loop 1425); `try_lock_job` at `state.rs` 75; `lock_repository` at 68; `commits.rs` 138; `manage.rs` 370;
  `gh.rs` 611; `Ui` at `lib.rs` 56; `Athena` at 69; `land.rs` 33-83. I did not check "about 1307" for the end
  of `land_or_block`.

### F8 (Low): `reread` in the multi-job path. Status: **resolved**

- **Design's answer:** section 6.2: no prompts; a `reread` job prints the usual text and exits 2; `--all`
  excludes `queued`, so it arises only for an explicit name.
- **Source check (read):** `go` on a `Queued` job calls `read_queued` and returns `GoResult::Reread`
  (`run.rs` lines 121-123); `go_and_report` returns 2 for it (`main.rs` lines 222-233). One detail is wrong in
  the design's JSON text; see N3.

## 2. Lock-order claim and question 9 (checked, now closeable)

Design section 4.4 and question 9 leave open whether `athena land --pr` takes the repository lock only after its
job lock. I read it: `Athena::land_pr` calls `try_lock_job` at `gh.rs` line 611 and holds that guard (`_lock`)
through the function; the resync at line 696 calls `resync_onto_merge`, which takes `lock_repository`
(`land.rs` line 113). So the order is job lock, then repository lock, as the design needs. I did not find a path
that takes them the other way round (grep of `lock_repository` and `try_lock_job` over `crates/`: the callers
are `commits.rs` 138, `land.rs` 113, `manage.rs` 80 and 370, `run.rs` 116, 1178 and 1424, `gh.rs` 611). Question
9 can be closed with this evidence.

## 3. New problems

### N1 (High): the landing queue in the launcher does not match what the children do

- **Evidence:** design section 7 point 2 says landings go "through one at a time", held "in the launcher, not in
  a file lock", each landing through `land --pr` and then the rebuild, "before the next landing check reads
  `HEAD`". Section 4.2 says each child is `athena go <id> --json`.
- **What the code does (read):** `go` dispatches, produces, verifies and lands locally inside one call;
  `land_or_block` (`run.rs` 1165) is reached from that path and reads `HEAD` at line 1182 when the child's own
  verification ends, under the repository lock (1178). `athena land --pr` is a separate command
  (`main.rs` line 365, `gh.rs` line 600); `go` does not call it. The rebuild `resync_onto_merge` runs only when
  `land_pr` sees the pull request merged (`gh.rs` 693-697), which needs checks to pass and a merge, and with
  `merge_via_rest` off (section 8 point 3) that is after the run.
- **Consequences:** (1) The queue cannot order the local landing, because it happens inside each child, at
  whatever time that child finishes, in the order the repository lock is won. The statement that the rebuild is
  finished before the next landing check is false for the local step. (2) The design does not say who runs
  `land --pr` for each job, when, or in which process, nor how the launcher learns a pull request merged. (3)
  Section 8 point 2 is consistent with this (the local landing comes first and the pull request is cut from it),
  so sections 7 and 8 together say two different things about order.
- **Needed:** state that local landing stays inside the child and is ordered by the repository lock; that the
  launcher runs `land --pr` per landed job as a second phase (or that the user does); and what the result of a
  job is while its pull request waits. Drop or restate the "rebuild before the next check" sentence.

### N2 (Medium): rule 1 of section 9 conflicts with how change jobs are written today

- **Evidence:** section 9 rule 1 says code, test and design jobs do not edit `CHANGELOG.md`. In this
  repository, change jobs routinely carry a `CHANGELOG.md` entry in their acceptance criteria (this job's own
  brief and the `CHANGELOG.md` entries for earlier design revisions are examples), and the roadmap's done-when
  (`docs/design/roadmap.md` lines 47-49) expects two jobs that both add an entry to land or the second to be
  refused with a named message. Nothing in the part of `admission.rs` I read (lines 90-130) rejects a change to
  those files.
- **Impact:** the rule is a convention that the existing job-writing habit breaks. Until the docs job exists
  and the readers know the rule, parallel jobs will name `CHANGELOG.md` and be serialized by rule 3, which is the
  fallback, so nothing is unsafe; but the first runs will serialize most jobs.
- **Needed:** say in section 9 that rules 1 and 2 apply from the build job onward, and that until then rule 3
  (serialize on shared files) is the working rule.

### N3 (Medium): the launcher classifies child results by message text, and `reread` JSON has no `state`

- **Evidence:** section 4.2 step 4 and the kill-switch paragraph map exit 1 to `error`, but then report
  `refused (kill switch)` "when its message says so"; section 5.3 lists `locked` separately. In the code,
  `refuse_if_stopped` and the lock refusal are `bail!`/`?` errors (`run.rs` 111, 116-118), which `main` turns
  into `error: ...` on standard error and exit 1 (`main.rs` lines 176-182). There is no structured code
  separating them from other errors.
- **Also:** section 6.2 says the JSON array uses the single-job fields (`job`, `state`, `reason`, ...). A
  `reread` child prints a different object (`job`, `commit`, `reading`, `reader_error`, `spent_usd`:
  `proposal_json`, `main.rs` lines 196-204), with no `state`, and exits 2 like a finished non-landed job. The
  launcher cannot tell the two apart by exit code alone.
- **Needed:** either match on message text and say that is brittle and tested, or give the child a way to
  report the kind (a field in `--json`, or an exit code); and say how a `reread` object is recognised.

### N4 (Low): exit code of an open pull request differs from the single-job command

- **Evidence:** section 6.3 counts `pr-open` as "did not land" (exit 2). `land_pr_code` returns 0 for an open
  pull request with green checks (`main.rs` lines 210-218). A user who runs `athena land --pr` alone gets 0 for
  the same state.
- **Needed:** say that the multi-job code differs on purpose, or align them.

### N5 (Low): the default `--all` buys expensive work that the design expects to go stale

- **Evidence:** section 7 "Consequence for use" expects most siblings to be stale; `plan` runs only at landing
  (`run.rs` 1189 inside `land_or_block`), after the producer and verifier budgets are spent. Section 5.2 prints
  the worst case but not that likely-stale jobs are paid in full.
- **Needed:** the warning in section 7 should include the cost, and the first build should consider asking for
  confirmation, or limiting default parallelism to jobs whose `Reading.deliverables` do not overlap (the data
  exists; see N6). This is a choice for the build job, not a defect.

### N6 (Low): name the field that holds the declared paths

- **Evidence:** section 9 rule 3 says "a job's brief names the shared files". The readings already carry
  `deliverables` with paths (`reading.rs` 391-455; `admission.rs` 109). Only `proposed` and `blocked` jobs have
  a reading that `--all` can use; a job named explicitly in `queued` state has none until it is read.
- **Needed:** name `Reading.deliverables` as the source of the list and say what happens for a job with no
  reading (it is not serialized, and the landing check decides).

### Not checked

- I did not read the agent driver implementations, `build_commit`, `advance`, or `Git::replay`. I did not search
  `crates/` for global state (`static mut`, `thread_local`); the design defers this to the spike and I did not
  need it for the process default.
- `registry().reap()` runs at the start of every `go` (`run.rs` line 112). I did not read it, so I cannot say
  whether several children reaping at once is safe. The build job should check it, because the process model
  makes that call concurrent for the first time.
- I did not check `docs/decisions.md` numbering rules beyond what the design and the roadmap say.

## 4. Changes required for "accept"

1. **N1:** rewrite section 7 point 2 and section 8 so that local landing stays in the child, ordered by the
   repository lock, and say who runs `land --pr` and when; remove the claim that the rebuild precedes the next
   landing check.
2. **N2:** state that section 9 rules 1 and 2 take effect with the build and that rule 3 is the working rule
   before then.
3. **N3:** say how the launcher tells `locked`, `refused`, `error` and `reread` apart, and correct the `--json`
   field list for `reread`.
4. Low items N4, N5, N6 and the closure of question 9 (section 2 above) may be done in the text or in the first
   build job.

The design is not edited by this review.
