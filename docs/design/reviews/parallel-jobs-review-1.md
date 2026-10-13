# Review 1 of `docs/design/parallel-jobs.md`

**Reviewed:** the design at status "proposed, independent review pending", plus the source it cites.
**Method:** reading only. No command was run and no behavior was reproduced. Every claim below cites the file
and lines read, or is labeled **unchecked** (not read) or **estimate** (not measured). Line numbers are those of
the repository at the time of review.

## Verdict: accept with changes

The overall shape (a bounded pool, the existing job lock and repository lock, per-job results, no silent merge)
is sound. Two things must change before any build starts:

1. The design's central premise, that disjoint parallel jobs mostly land and only conflicting ones end stale,
   does not hold in the current `plan`. Almost every parallel job after the first ends stale (F1).
2. "Threads in one process" cannot be chosen yet: `Athena` is not shown to be shareable, and one of its fields
   is not (F2).

The rest are corrections and smaller gaps.

## 1. Claims checked against the source

| Design claim | What the source says | Holds? |
|---|---|---|
| `try_lock_job` is an exclusive, non-waiting lock on `jobs/<id>/job.lock`; `None` if held (§3) | `state.rs` 74-82: `lock_file(job_dir/job.lock)`, `file.try_lock()`, `WouldBlock` gives `Ok(None)` | Yes |
| `lock_repository` is a blocking exclusive lock on `repository.lock` (§3) | `state.rs` 67-72: `file.lock()` | Yes |
| Each lock call opens its own file handle, so locks work between threads (§4.1) | `state.rs` 55-65 opens a new `File` per call. That the OS lock then conflicts between two handles in one process is standard `flock` behavior; I did not run it (**unchecked**) | Yes by reading |
| `go` fails with "already running in another athena process" at `run.rs` line 116 (§3, §5.3) | `run.rs` 116-118 | Yes |
| `refuse_if_stopped` is called by `go` at `run.rs` 111 and "689" (§3) | `run.rs` 111 is right. The second call is at line 698, not 689. Also called in `reading.rs` 469, 534, 602 | Minor line error |
| `land_or_block` holds the repository lock, makes up to three attempts, re-reads `HEAD`, runs `plan`, and retries on `Refusal::Moved` (§3) | `run.rs` 1178 (lock), 1181 (`for _attempt in 0..3`), 1182 (`head()`), 1189 (`plan`), 1212/1253/1270/1298 (`Moved` then `continue`) | Yes. The design cites "1156-1210"; the function is lines 1165-1307, and `retry_landing` has the same loop at 1424-1433 |
| `plan` returns `Stale` when the base is no longer an ancestor of `HEAD` or when governed base content changed (§3, §7) | `land.rs` 35-39 (ancestor), 45-51 (exclusion rules changed), 64-72 (governed content changed) | Yes, but see F1 for what it implies |
| "Disjoint jobs whose base is an older ancestor of `HEAD` are handled by `plan` as today" (§7) | See F1 | **Misleading** |
| Lock order: job lock first, then repository lock, never the reverse (§4.3) | `try_lock_job` is called only at `run.rs` 116 and `manage.rs` 80 (grep over `crates/`). `lock_repository` is called only at `commits.rs` 138, `manage.rs` 370, `run.rs` 1178 and 1424 | Yes |
| The review should confirm lock order for `manage.rs` and `commits.rs` (§4.3) | `manage.rs` 80 takes the job lock in `close`, then reaches `commit_records` (`manage.rs` 113) and so `commits.rs` 138. `manage.rs` 370 (`commit_with_staged`) takes only the repository lock. `commits.rs` 138 takes only the repository lock. No path takes the repository lock and then a job lock | Confirmed |
| The repository lock is not held across a nested call that takes it again | The lock is a blocking, non-reentrant file lock (`state.rs` 70). Inside `land_or_block` (1165-1307) and the retry loop (1424-1433) I found only `head`, `score_on`, `plan`, `build_commit` and `advance`; none of the grep hits for `lock_repository` or `commit_records` are inside those ranges. I did not read `build_commit` and `advance` in full (**unchecked**) | Likely yes |
| `Athena`/drivers may be shared across threads (open, §4.1) | See F2 | **No, not as written** |

## 2. Findings

Severity: **High** blocks the build; **Medium** must be settled in the design or the first build job; **Low**
is a correction or a later job.

### F1 (High): parallel jobs that change governed content go stale after the first one lands

`plan` computes the fingerprint of governed content, which is "everything except `records/` and owner paths"
(`crates/athena-git/src/attrs.rs` 135-139). `README.md`, `CHANGELOG.md`, `docs/` and all code are governed.
When `HEAD` is not the job's base, `plan` goes down one of two ways (`land.rs` 52-73):

- `fingerprint(head; p) == fingerprint(p; p)` (only records moved): replay the candidate with
  `merge_tree(p, head, c)` (lines 54-63).
- Otherwise: `Stale("governed content changed on the branch since dispatch ...")` (lines 64-72).

So `merge_tree` is used only when the branch gained **records-only** commits. If job A lands any governed
change, job B (same base) sees a different governed fingerprint and ends stale, **even if its files are
disjoint from A's**. `plan` has no case that merges two jobs' governed changes. The design's §7 sentence
("disjoint jobs ... are handled by `plan` as today, whatever it already decides") hides this, and its open
question 2 is answered here: **it always goes stale.** With N jobs from one base that each change governed
content, at best one lands (an estimate from reading; not measured). The design's own warning in question 2
applies: the feature "needs either a rebase-and-reverify option or the per-job pull request mode first."

Records-only commits do not cause this: job dispatch and closing commits add only records
(`commit_records`, `commits.rs` 127-149), so a job whose siblings only wrote records still replays.

**Decision:** the local-landing pool (section 10 item 3) must not be built as the first step with the promise
that independent jobs land. Either (a) the first build adds an explicit, budgeted, user-chosen
"re-verify stale job on the new `HEAD`" option (design alternative E, today rejected), or (b) per-job branches
and pull requests (section 8) come first. Until then the pool still helps for jobs that change **no governed
content** (read-only and study-like jobs, if their capability produces none; **unchecked**) and for the slow
produce/verify phases overlapping, but the section 1 claim of a gain for ordinary change jobs is overstated.
This needs a design change (fixable now): rewrite §7, §11 question 2 and §10.

### F2 (High): `Athena` is not shown to be shareable across threads

`Athena` (`crates/athena-core/src/lib.rs` 64-77) holds `Box<dyn AgentDriver>`. The `AgentDriver` trait
(`crates/athena-agent/src/lib.rs` 139) has no `Send` or `Sync` bound in the line I read, and `Ui`
(`lib.rs` 51) has none either; `go` takes `ui: &dyn Ui` (`run.rs` 110). By contrast `Capability` and `Pinner`
require `Send + Sync` (`capability.rs` 86, `pin.rs` 20). Rust will therefore not accept `&Athena` shared
between threads until `AgentDriver` is bounded (and every implementer, including the fake driver, complies).
`run.rs` also uses `RefCell` locally (around line 1140, `last_score.borrow_mut()`), which is fine inside one
thread but shows the code was written assuming one. I did not read every driver implementation or look for
global state (**unchecked**).

**Decision (question 1):** do not pick threads on faith. Run the spike (item 2) first, but plan on **separate
child processes (alternative B)** as the default, because it needs no change to `AgentDriver`, keeps P4 true by
construction, and isolates panics fully. A caught panic is *not* enough isolation for threads: it leaves shared
state (the registry directory, the repository lock file, partial usage files) in whatever condition the panic
left it, and a poisoned or abandoned lock is released only on drop. If the spike shows the bounds are cheap to
add, threads remain acceptable. This is a design change (fixable now): make B the recommendation and A the
option the spike may restore.

### F3 (High): shared append-only files collide, and the design does not say how it avoids this

Today's evidence (reported by the user, not reproduced by me: **unchecked**): hand-run parallel branches both
edited `CHANGELOG.md` and both took the next number in `docs/decisions.md` (both `D81`), and a stale candidate
could not land because the landing check refuses a hand-merged squash, so it needed re-verification.

What the design does about it:

- §7 treats a conflicting job as stale and leaves re-verification to the user, as D10 did. That matches the
  last point of the evidence: a stale candidate is re-verified, not merged. Reading of `plan` confirms it: a
  hand-merged content would not match the verified fingerprint, and `plan` bails with "refusing to land"
  when `landed != verified.fingerprint()` (`land.rs` 74-80). That is the check working, not a defect.
- The first point of the evidence is **not** covered. Section 10 item 5 (the docs job) edits `README`,
  `CHANGELOG` and `docs/decisions.md`; the roadmap says the design "must say how those collisions are avoided
  or resolved, and test it" (`docs/design/roadmap.md` 33-35). The design has no such text, and its test list
  (item 4) has no collision test.
- By F1, two parallel jobs that both touch `CHANGELOG.md` already end stale after the first, so the "same D81"
  collision is not silent: it becomes a stale job. The cost is a wasted run, not a corrupt file.
  With a future rebase-and-reverify option or per-job pull requests, the same edits become a real textual
  conflict in the merge (append at the same line) and a number clash that no merge tool can detect semantically
  (two different texts, both "D81").

**Recommendation:** add a section "Shared append-only files" to the design with these rules, and a test:

1. A job's brief states which shared files it will edit. Jobs whose briefs name the same append-only file
   (`CHANGELOG.md`, `docs/decisions.md`, `README.md`) are **not run in parallel**: the pool serializes them
   (an `--all` partition by file, using the declared path list; the true diff is unknown before the producer
   runs, so this is a hint, and the landing check stays the authority).
2. Decision numbers and changelog entries are written by the **docs job, run last and alone**, after the code
   jobs have landed; code jobs do not number decisions. This matches section 10 item 5 and makes it a rule.
3. Add the test: two jobs that both append to a shared file, run with `--jobs 2`, end with the second stale
   (and, once re-verification exists, land with distinct numbers).

This is partly fixable in the design now (rules 1-2) and partly a later job (rule 3, with the build).

### F4 (Medium): `--all` and `locked` results interact with the duplicate case

§4.3 says a job named twice yields a `locked` result for the second occurrence. Because both workers run in one
launcher, which one gets the lock is a race, and the "locked" message ("already running in another athena
process", `run.rs` 117) would be wrong. See the decision for question 6.

### F5 (Medium): the kill switch is checked at start only; running jobs still check it inside agents

The design says running jobs are ended by `athena stop` itself (§5.1). The sources agree: `claude.rs` 255 and
318 test `registry.stopped()` inside the agent loop, and `go` re-checks at `run.rs` 698. Nothing is wrong, but
the design should say the *per-worker* check belongs *before* `try_lock_job` is taken, so a refused job leaves
no lock file side effect beyond the file `lock_file` creates (`state.rs` 55-65 creates the file and parent
directories even when it then refuses). Low impact; mention in the build job.

### F6 (Medium): per-job budget, spend rate and a run cap

See question 3. The design is right that the worst case is N times the per-job cap, but gives no number. With
the default of 2 workers that is twice `job_usd` at once (**estimate** from §5.2; I did not read the configured
value).

### F7 (Low): line references drift

`run.rs` 689 should be 698; `land_or_block` spans 1165-1307 rather than 1156-1210. The `athena go` summary
should also cite `retry_landing` (1424-1433), which has its own loop. Fix in the design.

### F8 (Low): `reread` jobs in the multi-job path

§6.2 prints a follow-up for a `reread` job and ends it with exit code 2. Reading is paid work and may be
interactive for the user later; since question 4 excludes `queued`, `reread` can only arise for a job named
explicitly. Acceptable; no change.

## 3. The seven open questions

1. **Threads or processes? Is a caught panic enough?** Decision: **processes by default (alternative B)**,
   subject to the spike. Evidence: `lib.rs` 64-77, `AgentDriver` at `agent/src/lib.rs` 139 with no `Send +
   Sync` bound, `Ui` at `lib.rs` 51 with none, contrasted with `capability.rs` 86 and `pin.rs` 20. A caught
   panic is not sufficient (F2).
2. **How does `plan` treat a base that is an ancestor of `HEAD` but not `HEAD`?** Decision: it replays only if
   governed content is unchanged; otherwise **stale** (`land.rs` 52-73; `attrs.rs` 135-139). Parallel change
   jobs mostly end stale after the first. The feature needs the rebase-and-reverify option or per-job pull
   requests first, or must be limited to jobs that change no governed content (F1).
3. **Run-level spend cap in the first build?** Decision: **no.** Per-job caps stay (`run.rs` 182-240 as the
   design cites, which I did not re-read: **unchecked**), and `--jobs` with its ceiling bounds the rate. A
   global counter needs shared mutable state, which F2 argues against. Revisit after the first real run, with
   a measured cost. The design should print the worst case (N times `job_usd`) when a multi-job run starts.
4. **Which states does `--all` include? Queued?** Decision: **`proposed` and `blocked` only; not `queued`.**
   Evidence: the `match job.state()` in `run.rs` 120-157: `Queued` only re-reads (121-123), `Blocked`
   retries landing (124-126), `Proposed` dispatches (127-135), the others bail. Add the partition rule of F3 so
   that jobs naming the same shared file are serialized.
5. **Default of 2 workers, and a ceiling?** Decision: **2 is acceptable as the default, and a ceiling of 4 is
   enough for a first build** (both are estimates; neither is measured, and the design's 8 is also an
   estimate). Given F1, a low ceiling limits wasted spend on jobs that will end stale.
6. **Accept a job twice, or reject duplicates?** Decision: **reject duplicates up front** with a clear error
   before starting anything, since the second worker's `try_lock_job` would otherwise race the first
   (`state.rs` 74-82; F4). The same applies to two refs that resolve to the same id: resolve first
   (`Records::resolve`, `run.rs` 115), then compare ids.
7. **Exit-code precedence (error before did-not-land)?** Decision: **yes, keep it as proposed.** It matches
   the single-job rule documented in the header of `main.rs` (design §3, which I did not re-read:
   **unchecked**) and lets a script detect that Athena itself failed.

## 4. Today's evidence, summarized

| Observation | Does the design handle it? |
|---|---|
| Two parallel branches edited `CHANGELOG.md` and both took `D81` | Not covered: no rule for shared append-only files. Recommended rules in F3. |
| The landing check refuses a hand-merged squash, so a stale candidate needs re-verification | Covered in principle by §7 (stale, re-verify by the user), confirmed by `land.rs` 74-80. The gap is that this will be the *common* outcome (F1), not the exception. |
| The docs job (item 5) is the one shared-file hotspot | Yes: it is correctly the last, separate job; the design should state it must run alone (F3 rule 2). |

## 5. What can be fixed in the design, and what belongs to later jobs

| Finding | Fix now in the design | Later job |
|---|---|---|
| F1 | Rewrite §7 and question 2; reorder §10 so re-verification or per-job pull requests precede or accompany the pool | Re-verification option; per-job pull requests (item 6) |
| F2 | Make processes the recommendation; keep the spike | Spike (item 2) |
| F3 | Add the shared-file rules | Collision test and `--all` partition (items 3-4) |
| F4, F5, F6 | Add one sentence each | Item 3 |
| F7 | Correct the line references | None |
| F8 | None | None |

## 6. Limits of this review

I did not run the code, build anything, or read the agent driver implementations, `build_commit`, `advance`,
the budget code, or `main.rs` in full. Items marked **unchecked** above rest on the design's own statements.
The user's report of the `CHANGELOG.md` and `D81` collisions is evidence I was given and did not reproduce.
