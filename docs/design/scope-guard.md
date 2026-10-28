# The scope guard: Artemis, with Demeter's view of cost

Status: accepted (D127 in `docs/decisions.md`), decided by the brain under Sri's delegation (BRIEF.md addenda of
2026-10-23 and 2026-10-25), with two changes: section 8 escalations go to whoever holds Sri's delegation, and
the build order (section 13) starts with jobs 1 and 5 (rules only). Job 1 is built; this document writes no code. Audience: the owner
and design reviewers.

## 1. Purpose

The BRIEF.md addendum of 2026-10-25 says learnings must go back into the system, and that some role must be
responsible for ensuring Athena does not rabbit-hole into unnecessary work. Today nothing weighs a piece of
improvement work against what it buys before it is queued, and nothing watches a running line of work for
thrash. The scope guard does two things:

1. **Before queueing or dispatch:** weigh what the work buys (which failure class, goal or product need it
   serves, how often that bites) against its expected cost.
2. **While work runs:** watch a line of work (one request and its retries, or a chain of requests that chase the
   same aim) for repeated retries, growing plans, cost far past forecast, and repeated rejection for the same
   unmet criterion.

It must cut unnecessary work without slowing learnings work (section 7).

## 2. Mapping, confirmed

The BRIEF.md pantheon table has these rows (BRIEF.md lines 603 to 607):

| God | Domain | Role in the table |
|---|---|---|
| Artemis | Boundaries | Controls scope, unintended side effects, and impact beyond the requested change |
| Hestia | Continuity | Maintains knowledge, documentation, operational health, and long-term quality |
| Demeter | Sustainability | Evaluates resource consumption and the long-term cost of operating the system |

The relay note at BRIEF.md line 1067 proposes the role that stops rabbit holes as Artemis with Demeter's view
of cost. **Confirmed, with one refinement:** Artemis owns the role and its decisions; Demeter is not a second
gate but the cost input to Artemis's test (section 3.2). Hestia stays knowledge and quality and does not gate
work.

- Why Artemis owns it: a rabbit hole is scope growth or repeated effort on the same aim, which is a boundary
  question, not only a money question.
- Why Demeter is an input and not the owner: cost alone would cut cheap but pointless work, and would miss
  thrash that is about repeated rejection rather than dollars (the speedup case, section 4).
- Why not Hestia: gating on her would blur capturing learnings with throttling them.
- Why one role: two gates double the latency and give two places to appeal.

## 3. Value versus cost

### 3.1 Value side

Every piece of improvement work that is about to be queued or dispatched gets a value record. A field with no
source counts as unknown, not as zero.

| Input | What it names | Source it must cite |
|---|---|---|
| Failure class | One of the classes in `job-planning.md` section 2, or a named new class | A job record, decision or review finding; or "none" |
| Goal or product need | A BRIEF.md requirement, a `requirements-map.md` row, or a stated user need | The row or line cited |
| How often it bites | Count of occurrences per 100 jobs, or per month | Records, or "unknown" with a label |
| Already done or superseded | Whether a landed change or open request already covers it | The decision, commit or request id |

### 3.2 Cost side (Demeter's view)

| Input | Form | Label |
|---|---|---|
| Expected dollars | Point and p90, from the forecast the reader already gives per job kind, times expected attempts | **measured** if from forecast history, otherwise **estimate** |
| Expected wall time | Same | **measured** or **estimate** |
| Standing cost | Added tokens or minutes in every later job (for example a longer brief, a slower check) | **estimate** unless measured |
| Reversal cost | Low, medium, high, as in the `decisions.md` entries | stated |

Expected attempts default to 1.5 (estimate, to be tuned) until the first measurement.

### 3.3 Decision rule

Value is scored 0 to 3 from the table above: one point each for a cited failure class, a cited goal or need,
and a cited frequency of at least 2 per 100 jobs (estimate, to be tuned); the fourth input can only subtract.

| Outcome | When (rules, in order) |
|---|---|
| **Drop (proposal to Sri)** | Already done or superseded with a cited id, or value 0 with expected cost over $1 |
| **Merge (proposal; exact duplicates may be merged by the guard)** | Another open request has the same files and the same unmet aim; the older one survives, the other's text is attached |
| **Defer** | Value 1 and expected cost over $3, or expected cost over p90 of the job kind with value under 3. A deferred request stays queued with the reason and a review date 14 days out (estimate) |
| **Queue** | Value 2 or 3, or value 1 and cost at most $3 |
| **Queue (doubt)** | The value record cannot be completed and the work is learnings work: it proceeds (section 7) |

The dollar figures are estimates to be tuned. They are anchored on the mean cost per landed job of about $1.67
and the $2.50 ceiling in `job-planning.md` section 9 (both read there).

## 4. Grounding in the record

Figures marked **(request)** were supplied by the request of 2026-10-25; `records/` was not visible while this
was written and none was measured here. Figures marked with a path were read in this repository.

- **Trailer job:** three rejected attempts and about $11 before it was split **(request)**.
- **Prompt-budget trap:** about 80 minutes went to it **(request)**.
- **Open requests:** 49, many of them done or superseded **(request)**. The count and the done or superseded
  split are not checked here.
- **Test-suite speedup:** three attempts, jobs 20261025-190606-eebk, 20261025-202447-xgk9 and
  20261025-214509-y179, about $9 and 4 hours in all **(request)**. Each was a real gain: the test check went
  from about 12 minutes to 7.4 to 7.8 minutes **(request)**; `docs/decisions.md` D126 records about 651 s before
  and about 392 s after as summed per-binary test time, and test-check times of 444 to 466 s against about 720 s
  from the earlier attempts (read there). Each was rejected for missing a fixed time target that was set without
  a measurement showing it reachable **(request)**. The work landed through narrowed request 20261025-230352-ph40
  **(request)**; D126 names the three rejected jobs (read there).
- **Cut A:** four tries: a timing race, a wait on the wrong check, and duplicate push-triggered runs **(request)**.
  D125 in `docs/decisions.md` covers the push-trigger change (read there).

The common pattern: the same line of work was retried with the same unmet criterion or the same unmeasured
assumption, and the cost was only seen afterwards.

## 5. Thrash signals

A line of work is a request plus its retries ("try again on <job>") and any re-ask of the same request text. A
request that only cites another line's job ids as evidence starts its own line. Every number is an **estimate, to
be tuned** unless a basis is given.

| # | Signal | Threshold | Basis | Effect |
|---|---|---|---|---|
| T1 | Repeated retries | 3rd attempt on one line is about to be dispatched | Trailer job (3 rejections) and cut A (4 tries), both request-supplied | Flag; Sri sees it in the steward report |
| T2 | Growing plan | Accepted plan lists more than 1.5 times the files or steps of the first plan, or a 3rd planning round | `job-planning.md` plans record files and steps; 1.5 is an estimate | Flag; pause if T1 or T4 also holds |
| T3 | Cost against forecast | Spend on the line over the forecast p90 for its job kind: flag at 1.0 times, pause at 1.5 times; and over $5 on one line regardless of forecast | The $2.50 per landed job ceiling in `job-planning.md` section 9 (read); the rest is estimate | Flag, then pause |
| T4 | Same criterion rejected twice | 2 rejections of one line whose unmet acceptance criterion text (or its subject) is the same | The speedup case: three rejections for one time target (request) | Pause before the 3rd attempt |
| T5 | Wall time on one line | 60 minutes of agent time with no accepted candidate | Prompt-budget trap, about 80 minutes (request) | Flag; pause at 120 minutes |

T4 compares criteria by the verifier's recorded unmet criterion, normalised to lower case, with numbers
removed. Where the match is unclear the code reports "maybe" and the line is flagged, not paused.

### 5.1 Replay on the named cases

Even splits of cost across attempts are an estimate; the per-attempt records were not visible.

| Case | T1 | T2 | T3 | T4 | T5 |
|---|---|---|---|---|---|
| Trailer job (3 rejections, about $11) | Fires before attempt 3 | Fires if the split plan was larger than the first (not known) | Over $5 on the line after about two attempts (about $7 at an even split); fires before attempt 3 | Fires before attempt 3 only if the unmet criterion repeated (not known) | Not known |
| Test-suite speedup (3 rejections, about $9, 4 hours) | Fires before attempt 3 | Not known | $5 line limit not crossed until after attempt 2 (about $6 at an even split), so it fires before attempt 3 | **Fires after the 2nd rejection (xgk9), before attempt 3 (y179)**: the unmet criterion was the same fixed time target both times | Fires at 60 minutes of the 4 hours, and again at 120 |
| Prompt-budget trap (about 80 minutes) | Not applicable (no retries known) | Not applicable | Not known | Not applicable | **Fires at 60 minutes**, 20 minutes before the 80 spent |
| Cut A (4 tries) | Fires before attempt 3 | Not known | Not known | Not known (three different causes, so it should not fire) | Not known |

Reading of the speedup row: without T4, every signal that fires does so at the same point as T1, and by then
two attempts, about $6 and most of the 4 hours were already spent. T4 is the one that names the cause: a target
with no measurement behind it. On a T4 pause the guard asks for a measurement of the target or a narrowed
request, which is what request 20261025-230352-ph40 did after the third rejection **(request)**.

Cut A is a useful negative: four tries with three different causes is T1 only, and the guard should flag it
without a pause, because each try was learning something new.

## 6. Rules versus model

**Code decides (rules):**

- All five thrash signals and their pause effect; they are counts, dollars, minutes and string matches over
  records, so they are cheap, repeatable and checkable.
- The decision table in 3.3, given a value record.
- Exact-duplicate detection (same files, same text hash).
- The protected quota and the default-to-proceed rule (section 7).
- Which actions are Sri-only (section 8).

**A model scores and writes (advisory):**

- Fills value inputs from text: which failure class, which requirement, whether a request is already done or
  superseded, and a frequency lookup, each with a cited source.
- Writes a short recommendation to Sri.
- Judges whether two unmet criteria are "the same" when the T4 match is "maybe".

Why: thresholds on counts and dollars need no reading, and a model that gated them would cost per call and
could itself rabbit-hole. Reading whether a request serves a failure class or is already done needs text. So
the model supplies inputs and the code applies the rules.

**A model verdict may not:** lift or shorten a rule-based pause; cancel a flag; pass work the table drops or
defers; raise a value score above what its cited sources support (a score with an uncheckable source counts as
unknown); touch the protected quota; skip a Sri-only item; or edit `athena.toml`, `defaults.toml`, workflows,
`BRIEF.md` or the owner's goals. It may only move a decision toward "ask Sri", never away.

## 7. Keeping learnings work flowing

The addendum says learnings must go back into the system. The guard must therefore cut waste, not learning.

- **Protected quota:** learnings work (an improvement that cites a failure class from records, a review finding
  or a decision follow-up) has a reserved share of dispatch slots and budget: at least 25% of improvement
  dispatches in any 20 (estimate, to be tuned). The value test cannot defer learnings work while the quota is
  unmet.
- **Default to proceed on doubt:** if the value record is incomplete, or the model and the rules disagree,
  learnings work is queued. Only the thrash signals can pause it.
- **Appeal:** any defer, drop or pause can be appealed to Sri through the steward's channel (section 8); an
  appeal is a request that cites the held item and goes to the front of the next steward report. If Sri has not
  answered within 7 days (estimate), a deferred item is queued, and a paused line stays paused only if the
  pause was from T4.
- **Thrash is not exempt:** learnings work is watched like any other. A T4 pause on a learnings line asks for a
  measurement or a narrowed request; it does not close the line.

## 8. Powers

**On its own authority (reversible):**

1. Hold a new request in the queue as **deferred**, with the reason and a review date.
2. **Merge exact duplicates** (same files and same text hash), keeping the older request.
3. **Pause** (never kill) a running line after T3 at 1.5 times, T4, or T5 at 120 minutes; a pause stops new
   attempts and leaves finished work and records alone.
4. Flag lines on T1, T2, T3 at 1.0 times and T5 at 60 minutes.

**Advisory (a recommendation attached to the item):**

- Merge of similar but not identical requests; marking a request done or superseded; the value score; a
  suggested narrowing of a request.

**Goes to Sri (D127, accepted):** these escalations go to whoever holds Sri's delegation (the brain, under the
BRIEF.md addenda of 2026-10-23 and 2026-10-25). They go to Sri himself only for what the brief reserves to Sri.

- Dropping or closing any request (the 49 open requests are not touched by this design).
- Re-scoping a request, or changing an acceptance criterion.
- Anything touching governance files, the quota, or the thresholds themselves.
- Any appeal.
- Channel: the steward's report and items file (`roles-and-ladder.md`, the `steward` channel and `ask --later`
  pending items), so no new channel is built.

## 9. Relations

- **Steward (`roles-and-ladder.md`):** the steward is the periodic reviewer that runs after every five finished
  tasks, calls a model and may queue requests. The guard is a rules-first check the steward and the
  queue and dispatch code both call. The steward still reads the world and proposes; the guard decides whether a
  proposal is queued, and reports through the steward. The steward cannot override a guard pause; it can raise
  an appeal. The guard does not run on its own thread, which avoids a second model loop.
- **Hestia:** Hestia keeps knowledge and quality. She records what the guard stopped and why (decisions,
  reviews). She does not gate work. The guard reads her records as sources for failure class and frequency.
- **Planning keep rule (`job-planning.md` section 9):** planning is kept or turned off by the landed share of
  dispatched change jobs, cost per landed job, and guards. The guard reuses that frame (section 10). The two
  interact: a T2 plan-growth flag is a planning outcome, and the guard's pauses must not remove jobs from
  the landed-share denominator, for the same reason planning's stopped jobs count as not landed.

## 10. Measurement and the keep rule

This is a measured experiment. Primary measures: unnecessary spend avoided and wrongly held work.

- **Wasted share:** dollars spent on lines that ended with no landed change, divided by all dollars. Baseline to
  be taken from records before the build lands (not measured here).
- **Wrong holds:** deferred, dropped or paused items that Sri later releases or appeals successfully, per 100
  holds.
- **Learnings flow:** share of improvement dispatches that were learnings work; must stay at or above the quota.
- **Cost of the guard:** model dollars per decision.

Rule (estimates, to be tuned): through the first 30 guard decisions keep it on unless wrong holds exceed 30%.
At 100 decisions keep it if the wasted share has fallen by at least a quarter against baseline and wrong holds
are 20% or less; turn it off, or turn the model scoring off and keep the rules, if wrong holds exceed 30%;
otherwise record the result as inconclusive and keep measuring.

## 11. Alternatives

- **Demeter owns it:** rejected, see section 2.
- **Hestia owns it:** rejected, see section 2.
- **Model-only judgment:** rejected: costs per call, is not reproducible, and could rabbit-hole itself.
- **Rules only:** rejected: it cannot tell whether a request serves a failure class or is already done.
- **A fixed dollar cap per request and nothing else:** rejected: the speedup attempts were each cheap, about $3
  at an even split, yet repeated the same unmeasured target.
- **Exempt all learnings work:** rejected: it can rabbit-hole too; it is protected from value-blocking, not from
  thrash watching.
- **Let the guard close requests:** rejected: closing loses the user's intent and cannot be undone.

## 12. Risks and open questions

- The guard becomes the rabbit hole. Mitigation: the build is small jobs, rules first, and the keep rule can
  turn it off.
- Wrong holds slow real fixes. Mitigation: default to proceed, the quota, appeal, and 14 and 7 day timers.
- T4 matching is crude. Mitigation: "maybe" flags rather than pauses.
- Open: whether per-line cost can be read from existing job records without a new field (build job 1 settles
  it). Open: the true wasted share, which needs the records. Open: all thresholds are estimates.

## 13. Build plan

Each job is sized for one producer run and ordered by dependency. D127 (accepted) changes the order: build
jobs 1 and 5 (rules only) first, before the value scoring (jobs 2 to 4) and the model scorer (job 7). Job
numbers are kept so other text still refers to them. Job 5 in its first form depends on job 1 only; its
dependency on job 4 applies when queue-time decisions exist. Job 1 is built (`scope_guard.rs`). Job 5 is built
in its first form, described in 13.1.

1. **Line and signal types, pure functions.** Scope: a module with the "line of work" grouping (request plus
   retries and re-asks of the same text) and functions computing T1 to T5 from job records, with no I/O. Files: a new
   module in `crates/athena-core/src/` (for example `scope_guard.rs`) and its export line. Acceptance: unit
   tests build records for the trailer, speedup, prompt-budget and cut A cases and assert which signals fire
   and at which attempt, as in section 5.1; the T4 test fails if the same-criterion rule is removed.
2. **Value record and decision table.** Scope: the value and cost record types and the table of 3.3 as a pure
   function. Files: the same module, plus tests. Depends on 1. Acceptance: tests cover queue, merge, defer,
   drop and the learnings default-to-proceed row, including that an unknown source scores zero.
3. **Quota and appeal rules.** Scope: the 25% in 20 quota check, the 14 and 7 day timers and the appeal item,
   as pure functions. Files: same module, tests. Depends on 2. Acceptance: tests show learnings work is not
   deferred while the quota is unmet, and a lapsed appeal timer queues a deferred item.
4. **Queue-time check.** Scope: call the guard when a request is queued or dispatched, and record the decision
   with its reason. Files: the queue and dispatch code in `crates/athena-core` and the record schema. Depends on
   2 and 3. Acceptance: an integration test shows a value-0 costly request is deferred with a reason and a
   learnings request with an empty value record is queued.
5. **Running-line watcher.** Scope: evaluate T1 to T5 after each attempt and apply flags and pauses, never a
   kill. Files: the job-finish path in `crates/athena-core` and the CLI tests. Depends on 1 and 4. Acceptance:
   a test with two same-criterion rejections shows the third attempt is not dispatched and a pause record
   exists; a test with differing criteria shows a flag only.
6. **Sri channel.** Scope: report flags, pauses, deferrals and drop or merge proposals in the steward report and
   pending items; support appeals. Files: steward code and `docs/design/roles-and-ladder.md` if its text
   changes. Depends on 4 and 5. Acceptance: a test shows a paused line and an appeal appear in the next steward
   items, and an appeal lifts a defer only once Sri answers or the timer lapses.
7. **Model scorer.** Scope: a model call that fills value inputs with cited sources and writes the
   recommendation, with the rule that its output can only move toward "ask Sri". Files: a prompt under
   `crates/athena-core` prompts, scorer code, tests with a fake model. Depends on 4. Acceptance: a test with a
   fake model that tries to lift a rule pause, or cites nothing, shows the rule result stands.
8. **Duplicate and done or superseded finder.** Scope: exact-duplicate merge, and proposals (not closures) for
   done or superseded open requests. Files: queue code, tests. Depends on 4 and 7. Acceptance: a test shows an
   exact duplicate is merged automatically and a done request only produces a proposal.
9. **Measurement.** Scope: the wasted share, wrong holds, learnings flow and guard cost in the totals, and the
   keep rule as a status line. Files: the totals code and reader tests. Depends on 5 and 6. Acceptance: tests
   compute each measure on a fixture; the keep rule line reads inconclusive below 30 decisions.
10. **Baseline and first review.** Scope: record the baseline wasted share from records and review the
    thresholds after 30 decisions. Files: `docs/decisions.md` follow-up entry. Depends on 9. Acceptance: the
    entry states measured figures and either keeps, tunes or turns off the guard by the rule in section 10.

### 13.1 Job 5 as built: the running-line watcher

- **When.** `Athena::go` calls `Athena::watch_line` after a job finishes (next to the introspection and the
  steward count). The watcher reads the committed records, rebuilds the `RequestRecord`s
  (`request_records`: the retry link from "try again on <job>", the re-ask link to an earlier request with the same text, and each job's finished
  attempt from its outcome and verification), groups them with `group_lines`, and evaluates T1 to T5 with
  `evaluate_before_attempt`. A failure of the watcher is shown and ignored; it cannot change what `go` returns.
- **p90.** The p90 passed to `t3_cost` is `forecasts.reader.cost_usd.p90()` of the latest reading of the line's
  latest job. That forecast is the job's producer and verifier cost, which is the same quantity as the
  attempt spend the watcher sums (`Totals.cost_execution`). A missing or non-positive p90 is passed as 0, which
  `t3_cost` ignores. It is not aggregated over past jobs of the class, and the reference forecaster is not used.
- **Marks.** A flag is written to `<state root>/scope/<first id>.flag.json` and a pause to
  `<first id>.pause.json`, where the first id is the first request id of the line. A mark holds the signals with
  their level and numbers, the numbers they came from (attempts, rejections, spend, p90, agent minutes, the
  rejected criterion), the reason, and what would lift it: a narrowed request or a measurement for T4, a
  measurement of cost or progress for the others. A flag is removed when the line no longer raises a signal.
- **Refusal.** `dispatch` (`athena go`) and `Athena::retry` look the line up the same way (`line_pause`:
  rebuild the `RequestRecord`s, `group_lines`, then the pause file of the line's first id) and refuse a paused
  line with the signal, reason, lift and file name. `ask` is not blocked, so a request can still be read and
  proposed; only the attempt is refused.
- **Lifting.** Manual: the owner deletes the pause file. Nothing lifts a pause automatically.
- **Never a kill.** The watcher does not touch the process registry. A running job on a line that becomes
  paused keeps running; the pause only stops the next dispatch.
- **Status.** `athena status` lists flagged and paused lines with their signals, and a next step for each pause.
