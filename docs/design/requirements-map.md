# The requirements map, the strategy view, and the roadmap

**Status:** proposed. Design only: no code is built by this document, and no generator or checker exists. The
independent review is pending; nothing here is accepted until it is done.

Claims are marked **(read)** when taken from the file named beside them, and **(design)** when they are
proposals. No command was run to reproduce any behavior; every **(read)** claim comes from reading files in
this tree. Numbers that were not measured are labeled **(estimate)**.

**Requirements source:** [`BRIEF.md`](../../BRIEF.md). Line numbers below refer to the `BRIEF.md` in the same
tree as this document, not to a commit. If `BRIEF.md` has since changed, find the passage by the quoted
heading or id.

## 1. The problem

Requests arrive as one-off jobs. Each job is traceable to its own request, prediction and verification
(A5), but nothing says which of the brief's requirements those jobs are serving, which requirements have no
design yet, or what should come next. A reader who wants "how far along is requirement A7?" must search the
changelog, the decision log and the design documents by hand **(read: `CHANGELOG.md`, `docs/decisions.md`,
`docs/design/`)**.

The brief asks for this view in several places:

- A1 asks for "a living strategy view (goals, bets, risks, open questions, and what it would do next), current
  enough that I could read it at any time" (`BRIEF.md` lines 154-159).
- The addendum "The brief as a structured contract" (2026-10-12, lines 468-472) asks to "track changes ... back
  to the requirements, very, very cheaply".
- The addendum "A tired user, and a home for Athena's artifacts" (lines 474-476) asks for a place that
  "give[s] that context" to a user who may have none.
- O4 (lines 78-82) asks that every milestone leave the repository presentable, "what's done is published and
  verified, and what isn't is listed".

This document designs three linked things: a **requirements map** (§4), the **living strategy view** for A1
(§5), and a **roadmap** that sequences the queued work (§6). It builds none of them.

## 2. Terms

- **Requirement.** One row of the map: a section or addendum of `BRIEF.md` with an id. Ids are the brief's own
  (O1, A7, S4, D2) where it has them, and the ones defined in §3.1 where it does not.
- **Owning design.** The one document under `docs/design/` that is the place to change when the requirement's
  design changes. A requirement with none says `none yet`.
- **Serving work.** The landed changes that advanced a requirement, cited by the evidence this tree holds:
  a decision-log entry (`Dnn` in [`docs/decisions.md`](../decisions.md)), a `CHANGELOG.md` entry
  (`CL: <first words>`), or a design document. Job ids are not cited, because job records live under
  `records/`, which this tree does not contain **(read: the tree has no `records/` directory)**. §7 says how
  job ids join the map when they are available.
- **Queued work.** Requirements whose status is `designed`, `not-started` or `partial`, plus any named queue in
  the repository. Open requests recorded with `athena ask --later` are also queued work (D58), but their
  content is in `records/` and is not visible here **(read: D58)**.

## 3. The map

### 3.1 Status values

Every status below is defined here, and no other value may be used.

| Status | Meaning | Evidence needed to claim it |
|---|---|---|
| `built` | The requirement is met by code or a standing practice that exists in the tree. | A decision, changelog entry or enforcement row naming it, and for a mechanism a named test (`docs/enforcement.md`). |
| `partial` | Some of the requirement is built and some is not. The notes column says which part is missing. | Evidence for the built part, and a named gap. |
| `designed` | A design document covers it and nothing, or little, is built. The document may cover only part of the requirement; the notes column says which part. | A design document under `docs/design/` with a status line. |
| `not-started` | Neither a design nor an implementation exists. | No evidence found by reading `docs/`, `CHANGELOG.md` and the crate list. |
| `outside` | The requirement governs the user or the session that builds Athena, not Athena's code. It is deliberately not in Athena's files (D34), so this tree cannot show whether it is met. | A statement that the evidence lives elsewhere. Never counted as met. |

A status is a claim about the tree at the moment it was written. A status of `built` that the notes column
cannot support is an error in the map.

Ids not taken from the brief: `G` is the "Goal" section; `ENV` is the section about the run's environment (its
heading names the machine, so it is not quoted here); `STD` is "Standards"; `BND` is "Boundaries"; `ORD` is
"Order I require", with its scope and stopping rules; `JDG` is "How I'll judge it"; `AD1` to `AD11` are the
addenda in the order they appear.

### 3.2 The table

Brief lines are in `BRIEF.md`. "Serving work" cites `Dnn` entries and changelog entries; "notes" says what is
missing.

| Id | Requirement (brief lines) | Status | Owning design | Serving work | Notes |
|---|---|---|---|---|---|
| G | Goal: a strategic partner that learns, predicts, verifies, compresses (32-47) | partial | [`athena-v0.md`](athena-v0.md) | D01, D26, D14; CL: 0.1.0 | The aggregate of A1-A13. Compression and learning curves are not built. |
| O1 | Bootstrap small, then everything through Athena (51-57) | built | [`athena-v0.md`](athena-v0.md) | D41; CL: 0.1.0; acceptance reviews 1-3 under `reviews/` | v0 accepted; bootstrap is a closed range (D39). |
| O2 | The builder is a lazy user (59-62) | outside | none yet | none | Governs the user's conduct. |
| O3 | Interventions are the signal (64-76) | partial | [`athena-v0.md`](athena-v0.md) | D27; D39; D71 | Capture is built. The trend per class, the independent audit for missed interventions and the "act on each one" loop are not. |
| O4 | Keep the study moving (78-82) | partial | none yet | D58 (open work visible); CL: first study | Effort split between study and Athena is not tracked. |
| O5 | Restarts (84-89) | partial | [`athena-v0.md`](athena-v0.md) | CL: 0.1.0 (`restart` command and record) | The record and command exist. Resumption cost is not measured (see A11). The instructions for the session that continues the work are outside Athena (D34). |
| ENV | Facts about the run's environment | outside | none yet | D34; D23; D79; D80 (setup is committed and pinned) | Not part of Athena's design. Setup pieces Athena needs are under A4. |
| STD | Standards: designed, buy before build, evidence, strangers (113-124) | partial | [`athena-v0.md`](athena-v0.md) | D36; D16; five designs, each with reviews under `reviews/` | A practice, partly enforced (Quantity type, M17). The site's design standard has no subject yet. |
| BND | Boundaries (126-132) | partial | [`athena-v0.md`](athena-v0.md) | D17; D29; D34; D54 | Secret and personal-data gates are built. "Never push upstream" is prose-only until the native pull-request design is built. |
| A1 | Strategic partner (154-159) | partial | this document (the strategy view, §5) | D-log records choices (all entries carry rationale) | Rationale recording is built. The strategy view is designed here only. Pushback and the next-task proposal have no record type. |
| A2 | It learns its user (161-171) | partial | [`athena-v0.md`](athena-v0.md) | D26 (defaults model); D07; D14; D15; D56 | Defaults file, predictions before dispatch and proper scoring are built. Defaults are learned from interventions only by hand today. |
| A3 | Self-improving and versioned (173-182) | partial | [`athena-v0.md`](athena-v0.md) | CL: `[0.1.0]`; decision log "Bundle" fields; first study | Changes go through jobs and are logged. Rollback and "adopted deliberately" versions have no mechanism. What was learned from firstmate and Hermes is recorded only for M1 on firstmate. |
| A4 | Shape (184-189) | built | [`athena-v0.md`](athena-v0.md) | D20; D21; D33; D23; D79; D80 | Core separate from capabilities, modular Rust, one-command setup. Setup was not re-verified from clean for this document. |
| A5 | Work and history (191-202) | partial | [`squash-chain-landing.md`](squash-chain-landing.md) | D06; D10; D39; D46; D55; D69; D74; D76 | Traceable jobs, gated landings and secret scans are built. Parallel runs and pull requests are designed only (see AD4, AD9). |
| A6 | Efficiency (204-217) | partial | [`athena-v0.md`](athena-v0.md) | D16; D40; D28; D31; D79; D80 | Invocation records and cost reconcile. Jev is left out of v0 (D31), so the typed-decision use is not started. |
| A7 | Adversarial verification (219-225) | partial | [`athena-v0.md`](athena-v0.md) | D12; D13; D43; D59; D62; D63 | Code and docs are verified by an independent agent. User-facing verification (site, live deploy) has no subject yet. |
| A8 | Enforcement (227-231) | partial | none yet (the map is [`docs/enforcement.md`](../enforcement.md)) | D38; D39; D43; D73; enforcement rows M1-M31 | The map exists and names real tests (M24). Its measured overhead and the "load-bearing prose trends to zero" measure are not built. |
| A9 | Automation engineer (233-238) | partial | none yet | CL: 0.1.0 (the bootstrap built v0); D79; D80 | No registry of automations, their savings or retirement. |
| A10 | Specialists (240-243) | not-started | none yet | none | Capability crates exist (change, study), but they are capabilities, not recorded roles. |
| A11 | Resumption (245-248) | partial | none yet | D35 (state derived from records); D58 | A fresh context can read state with `athena status`. Resumption cost is not measured. |
| A12 | Memory and skills (250-254) | not-started | none yet | none | No skills directory. The acceptance test (a fresh agent works from the repository only) has not been run. |
| A13 | Compression and learning (256-269) | partial | none yet | D25 (fixed token budgets); D26 | Budgets are enforced (M18). Learning curves and the trained-contract loop are not built. |
| S1 | Instances and independence (273-279) | partial | [`study-capability-m1.md`](study-capability-m1.md) | D75; D77; D64; D65 | Firstmate is pinned and mounted read-only. Hermes and Athena are not yet studied. |
| S2 | Prompt analysis (281-292) | not-started | none yet | none | The first study is S4/M1, not a prompt inventory. |
| S3 | Operational lab (294-300) | not-started | [`study-capability-m1.md`](study-capability-m1.md) (run step, unbuilt) | none | The design covers containment and the run step for M1 only. |
| S4 | Mechanisms M1-M9 (302-316) | partial | [`study-capability-m1.md`](study-capability-m1.md) | D64; D65; D78; CL: first study | M1 (work handling) on firstmate is recorded and validated. M2-M9, and M1 on the other two instances, are not. |
| S5 | Evidence and rationale (318-321) | partial | [`study-capability-m1.md`](study-capability-m1.md) | D78; the decision log's Provenance field | Claims in the study carry pin, path, lines and a quote. Rationale cards for studied systems do not exist. |
| S6 | Critique ledger (323-325) | not-started | none yet | none | No ledger. |
| S7 | Comparison (327-331) | not-started | none yet | none | Depends on S1-S4 for all three instances. |
| D1 | Athena as a deliverable (335-339) | partial | none yet | CL: 0.1.0; `README.md`; `docs/architecture.md` | Changelog and docs exist. Tagged releases beyond 0.1.0, the capability-adding guide and the carried strategy view are missing. |
| D2 | Explainer site (341-361) | not-started | none yet | none | No site crate or Pages workflow. |
| D3 | Final report (363-389) | not-started | none yet | none | Depends on the site and on most of the data it reports. |
| ORD | Order, scope and stopping (391-402) | partial | none yet | O1; first study | Steps 1 and 2 are done. Step 3 (M1 on firstmate live on Pages) waits for the site. |
| JDG | How I'll judge it (404-441) | not-started | none yet | none | The acceptance conditions are not yet assembled into a checklist; this map is an input to one. |
| AD1 | 2026-10-10 Athena as a plugin (445-448) | not-started | none yet | none | D02 chose a command-line tool and kept `--json` as the seam for wrappers; this addendum reverses the weighting, not the seam. |
| AD2 | 2026-10-12 Taste probes, formerly "dumb context questions" (450-453) | not-started | none yet | none | Needs the expiry model the addendum names ("some expectations are forever"). |
| AD3 | 2026-10-12 A persistent project tracker (455-457) | designed | this document | none | Only the strategy view and the map are designed (§5). The browsable tracker is not. |
| AD4 | 2026-10-12 Athena does the work its user should not have to (459-461) | partial | [`native-pr-landing.md`](native-pr-landing.md) | D72 (proposal); D58; CL: "try again on" entries | Retry and deferral are built. Native pull-request handling is a proposal; its review has not been done. |
| AD5 | 2026-10-12 Brownfield integration, stealth (463-466) | designed | [`native-pr-landing.md`](native-pr-landing.md) | D72 | Requirements R5 and R6 of that design. Nothing is built. |
| AD6 | 2026-10-12 The brief as a structured contract (468-472) | designed | this document | none | This map is the first slice (requirement to design to work). Cheap automatic tagging with Jev and change tracking from reviews are not designed. |
| AD7 | 2026-10-12 A tired user, and a home for artifacts (474-476) | not-started | none yet | none | The map and the strategy view are candidate artifacts. |
| AD8 | 2026-10-12 The dashboard (478-481) | not-started | none yet | none | Depends on AD3 data and on Jev (A6). |
| AD9 | 2026-10-12 Chunking and parallel work (483-486) | designed | [`parallel-jobs.md`](parallel-jobs.md) | none | The design covers running independent jobs in parallel. Chunking one request into jobs is not designed. Its independent review is pending. |
| AD10 | 2026-10-13 Reconstructing job state, retros over the record (488-492) | partial | none yet | D35; D56; D06 | Records give per-job state at HEAD. State "at different points" and structured retros are not designed. |
| AD11 | 2026-10-13 Athena owns acceleration (494-496) | partial | none yet | D79; D80; D46 | Warm builds exist. Timings are recorded in `Quantity` values (D16); the feedback loop that acts on them is not designed. |

### 3.3 How to read the table honestly

- The `Serving work` column is **evidence available in this tree**, not a complete list. A job whose records
  were not carried here is invisible, and the map says nothing about it.
- `built` is rare on purpose. Where a mechanism exists but the requirement names more (A5, A7, A8), the status
  is `partial` and the notes column names the gap.
- Statuses were assigned by reading the changelog, decision log, designs and the crate list **(read)**. They
  were not checked against `athena check` or by running tests.

## 4. How the map is kept up to date

The map is a hand-maintained Markdown table (§8 gives the alternatives).

**Update rule (design).** A change that moves a requirement edits that requirement's row in the same change:

1. Landing a design document sets the `Owning design` cell (one design per row; a second design is linked in
   notes).
2. Landing code or a decision adds its `Dnn` or changelog entry to `Serving work` and re-states `Status`.
3. A new addendum from the user adds a row with the next `ADn` id, in the user's order.
4. A status may only move to a value whose evidence column in §3.1 is satisfied.

**Enforcement.** This rule is **prose-only** until a mechanism is built (the project's rule for any rule
without one is to say so; see A8). The mechanism this design proposes, for a later job, is a policy test in
`xtask` with three checks: every `###` heading or addendum of `BRIEF.md` has a row; every cited design path
exists; every status is one of the five defined values. Those checks are cheap and need no model. The
following parts cannot be checked by a test and stay prose-only: whether a status is true, and whether a
serving entry actually advanced the requirement. A verifier reading the diff is the only check on those.

**Why not drive it from jobs.** The brief says a requirement that seems to imply a method does not imply it
(lines 16-18), and the map is a view over records, not a new source of truth. The truth for "what happened"
stays in jobs and the decision log. The map stores only the join.

## 5. The living strategy view (A1)

### 5.1 What it must show

From A1 (`BRIEF.md` lines 154-159) and "Strategy: the strategy view is current, and the record shows real
pushback where the evidence warranted it" (line 414):

- goals;
- bets (choices made under uncertainty, each with the evidence that would change it);
- risks;
- open questions;
- what Athena would do next;
- pushback, with evidence and the alternative proposed, and the user's answer;
- when the study is done, the next big task with reasons.

### 5.2 Source of truth (design)

The source of truth is **typed records in the repository**, written like every other Athena record: one fact
per file, immutable once written, parsed with unknown fields denied, and a change expressed as a new record
that names the one it supersedes (D06 pattern, `docs/records.md`) **(read)**. A proposed schema family,
`athena.strategy/1`, with a `kind` of `goal`, `bet`, `risk`, `question`, `next`, `pushback`, or
`proposal`, each carrying:

- an id and a short statement;
- `requirements`: the map ids it concerns (for example `A7`, `AD5`);
- `evidence`: citations as in the decision log (a `Dnn`, a changelog entry, a job id, a record path);
- for a bet, a `would_change_if` condition;
- for a pushback, the request or requirement questioned, the evidence, the proposed alternative, and a
  `response` that is `pending`, `accepted`, `declined`, or `deferred` (the requirement still holds until a
  change is agreed, as A1 says);
- `supersedes`, optional.

The requirements map (§3) is the other input: it is the only place that holds each requirement's status, so
the view reads it rather than copying it. Decision-log entries are the existing record of "why Athena chose
what it chose" and are cited, not duplicated.

### 5.3 The view is derived, not edited (design)

The view is **rendered**, never written by hand: a command in the family of `athena status` (D35) reads the
strategy records and the map and prints goals, active bets, open risks, open questions, pending pushback, and
the next steps. The command shape is not fixed here. It follows D35's reasoning: a hand-kept status file
drifts, and a derived one cannot.

### 5.4 How it is kept current instead of regenerated by hand (design)

1. **Written at the moment of the event.** The records that feed the view are produced by the work itself:
   the reader may emit `risk`, `question` and `pushback` records with a reading; a verifier finding of
   severity `major` or higher about a requirement may open a `risk`; closing a job that changes a map row
   may supersede a `next` record.
2. **Landing touches or declares.** A proposed gate rule: a landing that changes a requirement's status in
   the map must add or supersede at least one strategy record, or the job's record must declare "no
   strategy change". This is a mechanism only after it is built. **Until then it is prose-only.**
3. **Staleness is visible.** The rendered view prints, at its top, the commit it was rendered at and the
   number of landed jobs since the newest strategy record. A reader sees "stale" without trusting anyone.
   A threshold of **10 landed jobs (estimate, not measured)** is the proposed starting point to flag
   staleness; it is to be revised once jobs-per-day is known.
4. **No standing regeneration job.** The view is rendered on demand from records, so there is nothing to
   regenerate and nothing to forget. The cost of reading is the cost of the render, paid by the reader.
5. **Pushback is not optional bookkeeping.** A pushback record is created when the reading's evidence and the
   request disagree, whether or not the user changes the request (A1: "the exchange is recorded").
   The mechanism for detecting disagreement is not designed here; a judgment by the reader is the only
   source until one exists. That part is **prose-only**.

### 5.5 What this design does not do

It does not decide the view's visual form (AD7 and AD8 own that), does not name the command, and does not
build the record type. The records it needs are small; the first build can be one record family and one
rendering.

## 6. The roadmap

### 6.1 The queue

The queue is every requirement in §3.2 with status `partial`, `designed` or `not-started`, plus the named
queues in the repository: the pending review of `parallel-jobs.md`, the proposed state of
`native-pr-landing.md`, and the unbuilt remainder of `study-capability-m1.md` (containment, the run step,
replay, Pages publishing) **(read: the status lines of those documents)**.

### 6.2 The order and why

Each step names what it depends on. Sizes are **(estimate)** in jobs, not measured.

| Step | Work | Requirements | Depends on, and why | Size (estimate) |
|---|---|---|---|---|
| 1 | Review this document. | A1, AD3, AD6 | Nothing is built from an unreviewed design (`BRIEF.md` line 121). | 1 review |
| 2 | Add the map check as an `xtask` policy test (the three checks in §4). | AD6, A8 | Step 1, so the table and status words are settled. Cheap: no model. | 1 job |
| 3 | Review `parallel-jobs.md`, then build it. | AD9, A5, A6 | The review is pending. Parallel runs shorten every later step, and the brief's chunking question (AD9) asks to run the study in parallel. | 1 review, 2-3 jobs |
| 4 | Build the strategy record family and its rendering. | A1, AD3, AD7 | Step 1 for the design. Step 2 so row ids exist to cite. Later sequencing then reads from the view. | 2 jobs |
| 5 | Review `native-pr-landing.md`, then build the owned-repository mode before stealth. | AD4, AD5, A5 | Squash-chain landing is built (D69), which that design needs (its requirement R7). Stealth needs records outside the repository, a larger change, so it follows. | 1 review, 3-4 jobs |
| 6 | Study M1 remainder: containment first, then the run step, then replay. | S1, S3, S4, ORD | Containment before the run step, because the run step executes a studied system and the design's response to review 1 makes containment part of that step (CL: the revision entry, D64). Replay needs recorded runs. | 4-6 jobs |
| 7 | Typed decisions with Jev, shadowed by a stronger model. | A6, A2 | D31 left Jev out with a plan to pull it in at the first calibration review or the first typed decision in the flow. Pushback detection (§5.4) is the candidate first decision. | 2-3 jobs |
| 8 | Taste probes with an expiry model. | AD2, A2 | Needs the defaults file (D26) and a place to store answers; benefits from step 7. | 2 jobs |
| 9 | The site and the first Pages publish. | D2, ORD step 3, AD8 | Needs study findings as data (step 6) and the strategy data (step 4) to show. | many; not sized |
| 10 | The plugin for Claude Code. | AD1 | Needs a stable `--json` interface (D02) and the native pull-request commands (step 5) to wrap. | not sized |

Dependencies that are not preferences: step 2 needs step 1; step 4 needs step 2; step 5's stealth needs the
owned mode first; step 6's run step needs containment; step 9 needs steps 4 and 6. Everything else is a
proposed order, and any step may be pulled earlier by evidence.

### 6.3 A tension to state

O4 says improvement to Athena is "pulled by evidence, never by anticipation" (line 80). This roadmap is an
anticipated order. It is a plan, not a commitment: **each step is run only when an intervention, a ledger
signal, a failed verification or a costly restart pulls it**, and the user can reorder it. Until a step is
pulled it stays in the table and nothing is built. If the evidence never arrives, the step stays unscheduled.
The map and strategy view (steps 2 and 4) are the exception the user asked for in AD3 and AD6, so they are
requested rather than anticipated.

### 6.4 Not yet scheduled

These have no step, and the reason is stated:

- A9 (registry of automations), A10 (specialists), A12 (skills), A13 (trained contract, learning curves): no
  design exists, and the brief says they are grown "pulled by real work" (line 57).
- S2, S6, S7 and the studies of Hermes and Athena (S1): they follow M1 on firstmate being live on Pages
  (`BRIEF.md` lines 395-396), which waits for step 9.
- AD10 (state at different points, retros) and AD11 (timing feedback): they need enough job records to
  analyze; the number of records is not visible from this tree.
- D1's tagged releases and D3, JDG: they depend on most of the above.
- Open `athena ask --later` requests: they are in `records/`, which this tree does not contain, so the
  roadmap cannot place them. The first job after this document that has `records/` should add them to the
  table.

## 7. Joining job ids (design)

Job ids are the best serving evidence, because a job traces to a request, prediction, verification and
landing (A5). They are absent here. When a later change can read `records/`, it should add job ids to
`Serving work`. A mechanism that derives the join (a `requirements` field on the reading, naming map ids)
is the cheap way, and is the place AD6's Jev tagging would plug in. It is not designed in detail here.

## 8. Alternatives considered

| Alternative | Why not |
|---|---|
| **Generate the map from records.** A tool joins jobs, decisions and designs into the table. | There is no `requirements` field on a job today, so the join would be guessed. A generator is also the build the request left out. Kept as the direction once step 7 gives jobs a field. |
| **Put the map in the `README` or `BRIEF.md`.** | `BRIEF.md` is the user's, never edited by Athena (its first lines say so). The `README` is for adopters, not for tracking. |
| **One status file per requirement under `records/`.** | Immutable records make status changes new files, so reading current status needs a derivation. A table is readable in one screen for 45 rows. Revisit when the row count or the update rate makes the table unwieldy. |
| **A hand-written strategy document.** | It drifts, which is why D35 rejected a hand-kept `STATUS.md`. Staleness would be invisible. |
| **Generate the strategy view by a scheduled job.** | It costs model calls even when nothing changed, and a stale file looks current. Rendering on demand has no such state. |
| **Strategy view as a section of the decision log.** | The log is append-only reasoning about decisions. Risks, questions and pushback are not decisions and have different lifetimes. |
| **Statuses as percentages.** | The numbers would be invented. Five defined words with named evidence are checkable. |
| **No roadmap; use the queue only (`athena ask --later`).** | D58 shows that the queue is for requests; it holds no dependencies or reasons. |

## 9. Risks and open questions

- The statuses depend on one reader's judgment of the evidence (§3.3). A reviewer should spot-check several
  rows, preferring `built` ones.
- A single table of 45 rows may become the bottleneck for parallel jobs (AD9): two jobs editing different rows
  can still conflict textually. Parallel landing treats that as a stale job (`parallel-jobs.md`, P5), which is
  correct but annoying. If it recurs, split by section.
- The 10-job staleness threshold is a guess **(estimate)**.
- Whether the strategy records belong in `records/` or in a separate directory is open. They are plain
  records, so `records/` is the default; the check that the layout keeps records append-only (`M4`) applies.

## 10. Reversal cost

Low. The document is Markdown with no dependents. Nothing reads it, and no rule is changed by it. Deleting it
loses the table and the design of the view; the decisions and designs it points to are unchanged.

## 11. Sources

All sources were read in this tree: `BRIEF.md` (headings and ids as cited in §3.2), `CHANGELOG.md`,
`docs/decisions.md` (D01-D80), `docs/architecture.md`, `docs/enforcement.md`, and the status lines of the five
designs in `docs/design/`. Nothing external was consulted.
