# Hestia: the continuity role, with Hades's cleanup folded in

Status: accepted design. Job 1 of section 9 (the sweep skeleton) is built in `crates/athena-core/src/hestia.rs`;
the checks and later jobs are not. Decision: D129 in [`docs/decisions.md`](../decisions.md), status accepted.
Readers: Sri, the brain, and the producers who will build the jobs in section 9.

## 1. What this is

BRIEF.md names Hestia in its pantheon table (`BRIEF.md:604`): "Continuity: maintains knowledge, documentation,
operational health, and long-term quality". It names Hades on `BRIEF.md:606`: "Failure and retirement: manages
rollback, abandonment, cleanup, deprecation, and safe termination". The addendum of 2026-10-25
(`BRIEF.md:1058-1061`) asks for "something that sanity checks the system and keeps everything neat and clean".
Its relay note points to Hestia, "with Hades's cleanup folded in until evidence calls for a separate role".

In plain terms: Hestia is a periodic sweep. It looks for things that have gone stale, piled up or been
forgotten, and either opens a small fix job or reports a work item. It is a sweep, not a gate: D127 says Hestia
does not gate work ([`scope-guard.md`](scope-guard.md) section 2 and line 35), and nothing in this design lets a
finding block a job.

Hades here means only the cleanup half: removing what Athena itself made and no longer needs (branches,
worktrees, caches). Rollback and deprecation have no workload yet and are out of scope.

## 2. The eight failures

Each went unnoticed until someone saw it by chance. The facts below are cited as stated in the request of job
20261025-033743-npr5; `records/` was not available to check them, so every figure is "as stated, not
re-measured". "Code" means a deterministic check with no model; "model" means a model reads and judges, and its
output is only a report.

| # | Failure (plain terms) | Check | Code or model | Cadence | Fix or report |
|---|---|---|---|---|---|
| 1 | Queued requests already done or superseded (the request cites 49 open requests, many done or superseded) | Code lists open requests older than 7 days (estimate) and the jobs and commits that name the same files or request text; a model then reads each candidate pair and says done, superseded or still open, with the quote it relied on | Code finds candidates; model judges | Weekly (estimate) and on each queue drain | Report. A work item per request, naming the evidence. Closing a request is Sri's or the brain's call, never Hestia's |
| 2 | Leftover branches, worktrees and build caches. On 2026-10-23 a cache filled the disk (as stated in the request) | Code: list branches and worktrees under Athena's own prefixes and paths; mark each merged (its tip is in the main branch) or not; sum build-cache and job-directory bytes against the free space on the volume; flag a cache above a size limit or a disk above a usage limit | Code | Every drain, and a cheap free-space check before each job starts (estimate: limit 80% disk used, cache limit 20 GB; both to be tuned) | Fix when provably Athena's own and merged or unreferenced: a small job deletes it. Otherwise report. A disk above the limit is reported at once as a work item with the largest directories listed |
| 3 | Checkpoints never pushed | Code: for each Athena-owned local branch or tag that holds a checkpoint, compare with the remote and list those with no remote copy and an age above 1 day (estimate) | Code | Every drain | Fix: push is not done by Hestia directly; she opens a small job that pushes, since pushing changes shared state. If the push is refused, report |
| 4 | Installed binary older than its source | Code: compare the build time or embedded commit of the installed `athena` with the commit at the repository's main branch; flag when the source is newer by any code change under `crates/` | Code | Every drain and at the start of each sweep | Report with the one command to rebuild and install. Replacing the running binary is not done by a sweep |
| 5 | Prompt budgets nearly full | Code: compute each role's prompt size against its budget (the same computation the budget test uses, `system_facts_budgets_match_computed`) and flag any role above 90% (estimate) | Code | Every drain, and in CI-time tests as now | Report a work item naming the role, bytes used and bytes left. Trimming a prompt changes governance-adjacent text, so it is never a silent fix |
| 6 | A keep rule nobody evaluated (the job-planning keep rule, `job-planning.md` section 9) | Code: read the rule's metric and the records it names, and report whether it was evaluated in the last 4 weeks (estimate) and what the data says now. The rule's own numbers are computed, not judged | Code | Weekly (estimate) | Report the evaluation result as a work item: keep, or the rule's own stated action. Acting on it follows the rule's text and goes through the normal job pipeline |
| 7 | `athena propose` never run | Code: read the time of the last proposal record; flag when none exists or the last is older than 14 days (estimate) | Code | Weekly (estimate) | Fix: open a small job that runs `athena propose` and files its output. The command and its output stay as defined elsewhere; Hestia only notices and triggers it |
| 8 | Docs that no longer match the code | Code first: broken links, documented commands or flags that the CLI no longer lists, paths in docs that do not exist, and counts in docs that differ from computed values. A model then reads a bounded sample of design docs against the code they describe and reports mismatches with a quote from each side | Code for the mechanical part; model for meaning | Code part every drain; model part weekly on a rotating sample (estimate) | Mechanical mismatches (a renamed path, a changed count): small fix job. Meaning mismatches: report as a work item with both quotes, since the right side may be the doc or the code |

Rules that apply to every row:

- **Provably Athena's own.** Hestia deletes only branches, worktrees and caches whose name or path matches
  Athena's own pattern and which are merged into the main branch or have no job record still open. Anything
  else, including anything ambiguous, is reported. Mechanism: the delete job's code takes an allow-list
  pattern and refuses a path outside it (build job 2). Until that exists this rule is prose only.
- **Small jobs, not direct edits.** A fix is a normal job through the change pipeline, so it is verified and
  reviewed like any other change. Hestia never edits a file or deletes a ref itself.
- **Empty findings.** A sweep that finds nothing writes the list of checks that ran, what each looked at (counts
  of branches, requests, docs examined) and any check that was skipped. "No findings" alone is never counted as
  success. A check that could not run (no network for the push comparison, say) is itself a finding.
- **Never gates.** A finding is a work item or a fix job. No finding stops, delays or fails a job in flight.

## 3. Where each check runs

Six checks are plain facts (rows 2 to 7), checkable from git, the filesystem, file times, byte counts and
configuration. Row 1 and the meaning half of row 8 need reading and judgment, so a model is used there, and only
there. The model's output is a report with quotes; it deletes and edits nothing. This follows the rules-first
shape of D127 ([`scope-guard.md`](scope-guard.md)): code decides what can be decided, a model scores the rest.

## 4. Budget

All figures in this section are estimates. None is measured; the first month of sweeps is the measurement.

- **Per sweep, code checks only:** under 1 minute of wall time and no model cost (estimate).
- **Per sweep, model checks:** at most $0.50 and 10 minutes (estimate), for rows 1 and 8 together, over a
  bounded sample (at most 10 request pairs and 5 docs per sweep, estimate).
- **Per week, in total:** at most $2 and 1 hour of wall time, including any fix jobs' sweep overhead (estimate).
  Fix jobs are charged to their own job budgets by the normal pipeline and are not part of this figure.
- **Stop rule:** a sweep stops its model checks as soon as its spend reaches the per-sweep cap or its wall time
  reaches the limit, writes what it covered and what it skipped, and reports the skipped items as a finding.
  If three weeks in a row hit the cap, the sample size is reported as a work item rather than raised silently.
  Code checks are never skipped for budget reasons.
- **Cap on fix jobs:** at most 5 open Hestia fix jobs at once (estimate), so a bad check cannot flood the queue.

## 5. How findings are measured

Sweeps run is activity, not outcome. The measures are:

- **Resolved problems:** findings that ended in a closed request, a deleted branch, a pushed checkpoint, a
  rebuilt binary, a trimmed prompt or a corrected doc, counted per week.
- **Time from appearance to resolution:** the age of each finding when it was resolved, from the first sweep
  that saw it. Open findings are shown with their age, so a pile does not hide.
- **Findings per run, by check:** to see which checks never find anything.
- **True-positive rate on a spot check:** each week a person or a fresh model reader that took no part in the
  sweep checks a sample of 10 findings (estimate) and marks each real or false. A rate below 80% (estimate) for
  a check turns that check into a work item to fix or remove it. The model checks, rows 1 and 8, are the ones
  expected to need this.
- **Recurrence:** a finding of the same kind returning within 4 weeks (estimate) means the fix did not address
  the cause; it is reported as such.

All thresholds start as estimates and are reset from the first four weeks of data.

## 6. How Hades's cleanup is folded in, and when it splits out

Cleanup (row 2, and the deletion half of row 3) is a group of checks inside Hestia called "retire". It has its
own allow-list and its own line in the measures, so it can be lifted out whole.

It splits into a separate Hades role when either holds:

1. Rollback, abandonment or deprecation work exists that is more than cleanup (for example a failed landing that
   needs undoing, or a feature to retire with notice); or
2. Cleanup findings exceed half of all findings over 4 weeks (estimate), which would mean the tidying has
   outgrown the continuity work and needs its own budget and prompt.

Until then a second role would add a prompt set and a budget for one group of mostly deterministic checks.

## 7. Alternatives considered

1. **Model-only sweeps.** One model reads the repository and the host and lists problems. Rejected: costly, not
   reproducible, and weak at plain facts such as a full cache directory. Rules-first is the shape D127 chose.
2. **Rules only, no model.** Rejected: code cannot tell whether a queued request is already done or whether a
   doc still means what the code does (rows 1 and 8).
3. **Separate Hades now.** Rejected for now: no rollback or deprecation workload, so it would be a role with a
   prompt budget for a few deterministic checks. Split triggers are in section 6. This matches the relay note
   in `BRIEF.md:1061`.
4. **Fold into the D127 scope guard (Artemis).** Rejected: D127 and its alternative (b) keep continuity apart
   from throttling work; a sweep that tidies and a guard that weighs and pauses work have different timing, owners
   and failure costs.
5. **Direct edits and deletions by the sweep.** Rejected: skips verification, and a wrong deletion cannot be
   undone. Fix jobs go through the pipeline.
6. **Report only, no fix jobs.** Rejected: obvious tidying (a merged branch, a stale cache) would pile up, as
   the 2026-10-23 disk fill shows.
7. **A check on each commit instead of a sweep.** Rejected for now: most checks look at drift over days, not at
   one change, and a per-commit run adds latency to every job.

## 8. Boundary with Artemis (and Demeter)

- Artemis ("Boundaries", `BRIEF.md:603`) owns the scope guard of D127: it weighs work before it is queued and
  watches running lines. It may defer, merge duplicates and pause.
- Demeter ("Sustainability", `BRIEF.md:607`) supplies the cost view to Artemis's test.
- Hestia ("Continuity", `BRIEF.md:604`) keeps knowledge, documentation, health and tidiness. D127 states she
  "stays knowledge and does not gate work".
- Where they touch: Hestia's queue check (row 1) reports done or superseded requests, which also helps
  Artemis's duplicate view; Hestia's report does not feed a gate, and Artemis does not run Hestia's checks.
  Hestia records what the guard stopped and why (`scope-guard.md` line 219). Hestia's own spend is reported to
  Demeter's view like any other.
- Hermes ("Orchestration", `BRIEF.md:597`) owns the craft of context efficiency; Hestia runs its recurring doc
  and prompt-headroom checks as part of her sweep and stays non-gating. See
  [`context-engineer.md`](context-engineer.md) section 10 and D130.
- This design changes nothing in the scope guard.

## 9. Build plan

Each job is sized for one producer run (a focused change of a few files, with tests). Deterministic checks come
first. A job lists what it delivers, how it is checked, and what it needs. The decision is accepted
(D129) and job 1 is built; jobs 2 to 10 are not queued.

Job 1. **Sweep skeleton and findings record.**
Deliverables: a `hestia` module in `athena-core` with a `Finding` type (check id, subject, evidence, age,
state), a sweep runner that records the checks that ran and what each looked at, and a command that prints the
last sweep. Also settles the trigger (drain hook, timer or command), which the code does not yet fix.
Check: a test where every check returns nothing yields a report listing all checks with counts, and a test where
a check errors yields a finding for it. Depends on: none.

Job 2. **Leftover branches, worktrees and build caches (row 2).**
Deliverables: the check with the Athena-owned allow-list, merged detection, byte sums and disk-usage limits.
Reports only; no deletion yet.
Check: tests on a throwaway git repository with merged, unmerged and foreign branches; a foreign path is never
marked deletable; a cache above the limit is reported. Depends on: 1.

Job 3. **Unpushed checkpoints (row 3) and installed binary age (row 4).**
Deliverables: both checks, reporting only.
Check: a throwaway repository with and without a remote copy; a binary whose embedded commit lags the source
commit is flagged and a current one is not. Depends on: 1.

Job 4. **Prompt headroom (row 5), keep-rule evaluation (row 6), `athena propose` age (row 7).**
Deliverables: three checks reusing the budget computation, the keep rule's own computation and the proposal
record time. Reporting only.
Check: a role at 91% of budget is flagged and one at 80% is not; a keep rule with no evaluation in 4 weeks is
flagged; no proposal record is flagged. Depends on: 1.

Job 5. **Mechanical doc drift (row 8, code part).**
Deliverables: link, path, command and count checks over `docs/`.
Check: a doc naming a missing path or a removed flag is flagged; a correct one is not. Depends on: 1.

Job 6. **Fix-job emission with the delete allow-list (rows 2, 3, 7 and the mechanical part of 8).**
Deliverables: turn selected findings into small job requests (delete a merged owned branch, push a checkpoint,
run `athena propose`, fix a renamed path), capped at 5 open at once; a refusal for anything outside the
allow-list.
Check: a path outside the allow-list yields a report and no job; the cap holds at 6 findings; the request text
names the evidence. Depends on: 2, 3, 4, 5.

Job 7. **Model check: queued requests done or superseded (row 1).**
Deliverables: candidate finder (code), a prompt with a small budget, a report with quotes, per-sweep spend cap
and stop rule.
Check: on a fixture queue with one done and one open request the done one is reported with its evidence; a run
that hits the cap stops and reports what it skipped. Depends on: 1.

Job 8. **Model check: docs against code (row 8, meaning part).**
Deliverables: a rotating sample, a prompt, a report with a quote from each side.
Check: a fixture doc with a wrong sentence is reported with both quotes; a correct doc is not. Depends on: 5, 7.

Job 9. **Measures and spot check.**
Deliverables: resolved count, time to resolution, per-check findings, recurrence, and a weekly spot-check
sample of 10 findings with a place to mark each real or false.
Check: a finding that disappears is counted resolved with its age; a spot check below 80% creates a work item
for that check. Depends on: 6, 7, 8.

Job 10. **Budget enforcement and the split trigger.**
Deliverables: weekly and per-sweep caps, the stop rule, and a report of the cleanup share of findings over
4 weeks against the split trigger.
Check: a sweep at its cap stops model checks and still runs all code checks; a cleanup share above half shows
the split notice. Depends on: 7, 9.

## 10. Open points

- The trigger is settled in D129: an explicit command (`athena hestia sweep`, `athena hestia last`). A drain hook
  stays possible later on the same function; a timer is not built.
- Every figure in sections 2, 4 and 5 is an estimate; the failures' own figures come from the request and were
  not re-measured.
- The rules marked "prose only" above (the allow-list before job 6, the never-gates rule) have no code
  enforcement until the named job lands.

## 11. Error review: causes that repeat across jobs

Built in `crates/athena-core/src/hestia.rs` (`Athena::error_review`, `repeated_causes`). It is separate from the
sweep: it runs on an event, not on the `athena hestia sweep` command, and it is enforced by code.

**Why.** Every failed or rejected job already gets an introspection with a cause and a prevention
(`defaults-loop.md` section 3), but each one is written about one job. Nothing grouped them by cause or acted on
a repeat, so a person caught patterns by hand and late: four planning-cap stops before a fix, five attempts at
one request before a stop. The error review is the role that reads the introspections together.

**Trigger.** `Athena::introspect` calls `Athena::error_review` right after a recorded introspection is committed.
A failed introspection is not reviewed and is not a case. The review never fails the introspection: a problem is
a line to the user.

**Grouping key** (`cause_key`). Two introspections have the same cause when they resolve to the same failure
class: the class their record names when the class registry (`athena.failure-class/1`) holds it, otherwise the
class Jev chose in its stored answer when that is a registry class (`new` and unknown ids are not). An
introspection that resolves to no registry class is grouped by its cause text with case and punctuation ignored
(`introspect::normalise`). An introspection with no job, no cause, or a failed status is not a case. The
cause shown is the earliest introspection's wording.

**Repeat.** A cause is a repeat when its introspections are about two or more distinct jobs (`REPEAT_JOBS`).
Two introspections of one job are one job, so a single job that failed twice is not a repeat.

**Request and dedupe.** When the just-recorded introspection makes its cause a repeat, and no open or landed
request cites any introspection of that cause, the review queues one request through the shared steward request
builder (`steward::StewardRequest`). The request is at `Priority: urgent`, asks for the root cause to be fixed
where it originates, cites every introspection id of the group with the job and the prevention each proposed,
and has acceptance bullets that are checked by the builder's own parse. A request cites a cause when its text
contains one of the group's introspection ids, so a third introspection of a cause finds the open request and
queues nothing. A request counts while it is open (not closed) or landed. A request that was rejected, failed or
closed does not, so a cause whose fix did not land can get a new request. The review does not fix the cause
itself; the request runs as a job under the usual checks and the independent verifier.

**Status.** `athena status` lists each open repeated cause (`StatusView::repeated_causes`): the count of
introspections, the number of distinct jobs, the cause, its class, the introspection ids, and the request that
addresses it (or "no request yet", which also adds a line to the next steps). A cause drops off the list when its
request has landed.

**Scope guard counting.** A related change in `scope_guard.rs` makes the guard count attempts per line of work
instead of per job id. A line joins a request, the request it retries ("try again on <job>", the previous job of
the line), any request that quotes it, and any re-ask of the same request text (compared with
`introspect::normalise`) whose earlier twin was not accepted. The guard's thresholds are unchanged. A re-ask
after the earlier request landed starts a new line. The chain link `previous_job` of `athena.candidate/2` is not
used: it names the latest job landed in the base's history, which every job on that base shares.
