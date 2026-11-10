# Hestia: the continuity role, with Hades's cleanup folded in

Status: accepted design. Job 1 of section 9 (the sweep skeleton) is built in `crates/athena-core/src/hestia.rs`;
the checks and later jobs are not. Section 13 (recurring duties) is built too: the drain start and the landing hook
now run the duties that are due, as well as the explicit command. Decision: D129 in
[`docs/decisions.md`](../decisions.md), status accepted.
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

- The trigger is settled in D129: an explicit command (`athena hestia sweep`, `athena hestia last`). Section 13
  later added runs of the due duties at a drain start and after a landing, on the same function; a timer is not
  built.
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
builder (`steward::StewardRequest`). The request is at `Priority: high` (Hestia never queues urgent; that level is for a person), asks for the root cause to be fixed
where it originates, cites every introspection id of the group with the job and the prevention each proposed,
and has acceptance bullets that are checked by the builder's own parse. A request cites a cause when its text
contains one of the group's introspection ids, so a third introspection of a cause finds the open request and
queues nothing. A request counts while it is open (not closed) or landed. A request that was rejected, failed or
closed by Hestia itself (to queue it again) does not, so a cause whose fix did not land can get a new request.
A request a person closed with `athena close` counts until an introspection of the cause is recorded after the
close, so the review does not queue again what a person turned down. The review does not fix the cause
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

## 12. Throughput review: what the pipeline waits on

Built in `crates/athena-core/src/status.rs` (`Throughput`) and `crates/athena-core/src/hestia.rs`
(`Athena::throughput_review`, `throughput_due`, `priority_for_minutes`, `throughput_followups`). Enforced by code.

**Why.** The error review covers failures, the code-health review covers file and function size and structure.
Nothing measured speed or asked what change to the shape of the pipeline would remove the biggest wait. The
velocity analysis and the asynchronous-drain idea (`async-drain.md`) came from people. This review lets Athena
find such changes itself.

**Measurement** (`athena status`, section "throughput"). Computed from the job records alone, with no git history:
landed jobs per UTC day; the median minutes per landed job with a median per stage over the newest 20 landed jobs
(`THROUGHPUT_WINDOW`); idle time between jobs; and the minutes spent on jobs that did not land. A stage is
reading (reader invocations), planning (planner and critic), producer, checks (the durations Athena measured for
the job's checks), verifier, and landing and rebuild. Landing and rebuild is one stage, because no record
separates the rebuild: it is the time from the end of the job's last invocation to the landing outcome. Idle time
is the sum of the gaps between the merged working spans of all request jobs, so overlapping jobs leave none. Time
on jobs that did not land counts each closed, non-landed job, and each job stopped in planning, from its first
invocation to its closing record.

**Wasted work** (`athena status`, the same section; `status::waste_windows`). The minutes per landed job cannot
see attempts that never landed: on 4 to 6 November 58% of attempts did not land, and the median of the landed ones
did not move. Three measures are computed from the job and invocation records over the newest 20 landings (fewer
while fewer have landed, and the count is printed) and over the 20 landings before them:

- **Attempts per landed change**: landings plus attempts that did not land, divided by landings. An attempt that
  did not land is a request job that ran a model and was closed unlanded (failed, rejected, abandoned) or stopped in
  planning. A job that never ran a model is not an attempt, and a job still open is neither landed nor wasted yet.
  Each attempt is placed by when it closed, in the window of the landings it closed among.
- **Money spent on attempts that did not land**: the recorded cost of those attempts' invocations, in US dollars.
- **Share landed on the first attempt**: landings whose request is not "try again on" a job that ran a model, over
  all landings.

The figures are a reconstruction from the records; a landing record written later can move a landing end and so a
window cut.

**Cadence.** `Athena::review_passes` runs the review at the start of every drain and in each sweep. It is due
**every 10 landings** (`code_health::REVIEW_EVERY`, the cadence of the design review) after the previous
throughput review, or the first 10 landings when there is none, **or when the median minutes per landed job is
worse by more than 20%** than the median the previous review recorded (`WORSEN_RATIO`), once at least one job
has landed since. Exactly 20% worse is not more than 20%. **It is also due on a rise in wasted work** (trigger
`wasted`, `waste_rises`): attempts per landed change or the money spent on attempts that did not land more than
20% above the figures the previous review stored on its record, or the first-attempt share more than 20% below
it, once a job has landed since. When the previous review stored no spend (nothing wasted), a spend above zero
counts only if attempts per landed change also rose by more than 20%, so one failed run after a clean window starts
no review. A previous review written before these figures existed gives no such trigger. When it is not due,
nothing runs and no model is called. A review that runs always makes the comparison described under "Slowdown",
whether or not a design request is open, and always asks the model.

**The question.** A strong model (the verifier's model, as for the steward, the audience and the auditor) is
given the status section (`THROUGHPUT.md`) and a copy of `docs/design/` in a read-only workspace. It names the
biggest bottleneck, the architectural change that removes it (a change to the shape of the pipeline, not a tweak
inside one stage), and the minutes per landed job it expects the change to save.

**Request and priority.** The top proposal is queued as a design request through the shared steward request
builder (`steward::StewardRequest`), so `athena plan` accepts it. Its title starts `Throughput design: `, its body
carries the bottleneck, the proposed change, the predicted saving and the review's id, and it asks that a change
which implements the design cites that id. The priority follows the predicted saving, speed first: at least 5
minutes per landed job is high (never urgent), at least 2 normal, and less is low
(`priority_for_minutes`).

**An open design request does not stop the review.** While a request with that title is open (not closed), a
later review still runs and asks the model: the record's note says "throughput design request <id> is still open;
this review ran anyway", and `THROUGHPUT.md` names the open request (id and title) so the proposal differs from
it. The proposal is recorded either way. Whether a design request is queued is decided separately, by the noise
rule below.

**The noise rule** (`request_gate`). A design request is queued only when a measure got worse by more than the
noise between recent windows. For each of four measures (median minutes per landed job, attempts per landed
change, money spent on attempts that did not land, and the first-attempt share) the review takes the change from
the window of 20 landings before the newest to the newest 20 landings, in the bad direction. The noise is the
largest difference between two adjacent windows older than the newest: the second and third newest, and the third
and fourth newest when the fourth is full. That needs 60 landings, so it rests on one difference at exactly 60 and
on two at 80; the note states how many. The change must also exceed the smallest change one job can make (the
floor), so a flat older history, whose noise is zero, does not turn one retried landing into a request: 1/20 for
attempts per landed change and for the share, the median cost of one attempt that did not land for spend, and one
minute for the median. A change equal to the larger of the two is within the noise. The record's note starts that
sentence with "design request:" and says which measures were beyond (with change, noise, floor and the number of
differences), or that the change was within noise, or that no measure worsened. A measure whose windows are not
all full is left out and the note says so.

Under 60 landings the noise cannot be estimated, so the review queues no design request and its note says "not
enough history: N of 60 landings, none queued"; the proposal is still recorded. The very first review is the
exception: with no earlier review to compare with, it queues the proposal when no design request is open. Because
the noise is taken from adjacent differences, it absorbs a steady trend: a worsening of equal steps (for example
spend of 2, 4, 6, then 8 USD) is within the noise and queues no design request. The 20% trigger and the cadence
still run the review, and the model's proposal is still recorded. When a request is open and a change is beyond
the noise, a second design request is queued; both are linked by their reviews' proposals.

**Slowdown.** Every review that runs compares the latest median minutes per landed job with the previous review's,
even while a design request is open (a rise from 25.4 to about 60 minutes once went unflagged behind an open
request). A review stores only the landings it counted and the median of the totals, so the previous per-stage
medians are rebuilt from the job records: the first `landings` request jobs to land, ranked over every landed job
so that jobs that ran no model do not shift the cut, then the newest 20 measured ones, with the same per-stage
median as the status section (`previous_stage_medians`, `StageMinutes::median_of`). The rebuild is a
reconstruction; a landing record written after the review can have moved a job's landing time. The stages whose
median minutes grew are named, the stage that gained most first (`stage_growth`), in the review's note, in the line
`athena drain` prints and in a line to the person running the review. When the median grew by **more than 25%**
(`SLOWDOWN_RATIO`; exactly 25% is not more), the review also queues, without asking the model, one request at
**high** priority (`slowdown_request`). Its title starts `Throughput slowdown: ` and names the stage that grew most
with its measured median minutes; the body adds the previous minutes, the median change and the other stages that
grew. It cites no review id, so it is not mistaken for the change that implements a design proposal. While a
request with that title is open, a later review names the stages again but queues no second one. A rise between
20% and 25% runs a review and names the stages but queues nothing. When the median grew by more than 25% but no
stage's median did (medians of stages are not additive), or the previous window cannot be rebuilt, nothing is
queued and the note says so. The slowdown request follows the noise rule too: with 60 or more landings it is
queued only if the rise over the previous review's median is also more than the noise between recent windows
and one minute (`slowdown_beyond_noise`), and otherwise the note says "slowdown request: ... within noise"; with
fewer landings the noise cannot be estimated, the 25% rule alone decides, and the note says "noise could not be
estimated".

**Prediction record.** Each review is an `athena.throughput-review/1` record at
`records/throughput-reviews/<YYYY-MM>/<id>.json`. It holds the landings counted and the window median (the
baseline the next review compares with), the trigger (`cadence`, `worsened` or `wasted`), the proposal (bottleneck,
stage, change, expected minutes saved, priority, request job, empty when the noise rule queued none), the
wasted-work figures over the newest window (`waste`, optional, so older records still parse) and the follow-ups
it closes. A model that fails or answers
badly still leaves a record, with no proposal and a note. The model's invocation is recorded either way.

**Prediction against measurement.** A change is cited when a landed request job, other than the design request,
has the review's id in its text and landed after the review. When one has landed, the next review records a
follow-up for it once: the predicted minutes saved, the baseline median, the median minutes per landed job over
the (at most 10) newest jobs that landed after the change, and the difference, which is the measured saving. The
review prints a line for each follow-up. A change that has not landed, or that no job has followed, records none.

**Not covered.** The review does not change the error review or the code-health review, and it does not
implement any change. A crash between queuing the request and committing the review record leaves an open
request with no record: the next review names it as open, and that prediction gets no follow-up.

## 13. Recurring duties

Athena has duties that must happen again and again, and none may depend on a person remembering. Hestia's sweep
owns them: this table is the list (`DUTIES` in `crates/athena-core/src/hestia.rs`; a test compares the ids and
cadences below with it), and the sweep runs each duty on its cadence and records when it last ran.

| id | duty | owner | cadence | where it runs |
|---|---|---|---|---|
| `error-review` | Error-review backlog pass | Hestia's sweep | every drain start | `Athena::error_review_backfill`, from the drain start and the sweep |
| `evidence-labels` | Evidence-label derivation | Hestia's sweep | every drain start | `Athena::derive_labels`, from the drain start and the sweep |
| `code-health` | Code-health check | Hestia's sweep | every landing | `Athena::code_health_check`, from the landing hook and the sweep |
| `design-review` | Periodic design review | Hestia's sweep | every 10 landings | the code-health check, which queues the review |
| `keep-rule` | Keep-rule threshold check | Hestia's sweep | every 10 landings | the sweep and the landing hook, over the records |
| `reading-rule-refusals` | Reading-rule refusal-rate check | Hestia's sweep | every 5 landings | the sweep and the landing hook, over the records |

The cadence kinds are "every drain start", "every landing" and "every N landings". The values of N (10 and 5) are
a choice made here; the design review's 10 is the code-health cadence (`code_health::REVIEW_EVERY`), and the
throughput review and the spec review are not duties: they keep their own triggers (sections 9 and 12).

**Occasions.** A pass over the duties starts from one of three places, and the place decides which duties may run.
At a drain start (`Athena::review_passes`) the drain-start duties run, every time. After a landing
(`Athena::code_health_after_landing`, called on both landing paths) the landing duties that are due run. An explicit
`athena hestia sweep` runs every duty that is due. Whether a duty that may run is due is decided from the stored
runs, never from the place alone.

**Last run.** A pass that ran at least one duty commits one `athena.hestia-sweep/1` record. Each duty that ran is
a check in it, named by the duty's id; the record's `duty_pass` field holds the occasion (`drain_start`, `landing`
or `sweep`) and the landing total at that moment. "Every N landings" is the landing total now minus the total stored
with the duty's last run, so it does not depend on commit times, and a host squash that replaces the landed commit
and keeps its trailers leaves the total unchanged: the second landing hook for one landing runs nothing. Commits
that carry no job trailers (a duty pass, a code-health baseline, a person's fix) are not landings. A duty that
failed to run is recorded as an errored check with the usual finding and does not count as a run, so it is due at
the next pass that may run it. Sweeps written before this field existed have no `duty_pass`; a duty then reads as
never run and runs once. `athena hestia last` skips drain-start and landing passes, which list only the duties
that ran, so they never hide the findings of the last full sweep.

**Due and overdue.** A duty that never ran, or whose newest attempt failed, is due at every pass that may run it. A
drain-start duty that ran is due at the next drain start and, at a sweep, not before: a sweep within the cadence
runs none. A landing duty is due once its landings since the last run reach its N. `athena status` shows each duty
with its last run, its next due point and an OVERDUE mark, and a duty is overdue when the next sweep would run it.
A drain-start duty has no time limit, so it is overdue only when it never ran or its last attempt failed. The
design review is not run by itself: the code-health check queues it when ten landings stand beyond the latest code-health
baseline, so its last run is the newest baseline that queued a review and its next due point counts from the latest
baseline, the same counter that queues the review. A repository whose first baseline queued no review shows "never
reviewed" and is not overdue until ten landings have passed.

**Cost.** Every drain start and every landing that runs a duty adds one small records-only commit. Nothing is
committed when no duty was due.
