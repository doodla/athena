# The context and knowledge engineer: Hermes, with Hestia

Status: proposed design, no code. Decision: D130 in [`docs/decisions.md`](../decisions.md), status proposed.
Readers: Sri, who judges the design, and the producers who will build the jobs in section 11.

## 0. Question to file

This is the doc's own map, because the role it describes asks every doc to have one.

| Question | Section |
|---|---|
| Who owns this, and is the Hermes and Hestia mapping right? | 2 |
| What exists today and what is only proposed? | 3 |
| How small should files, functions, docs and prompts be? | 4 |
| What do agents read, and how is that kept good? | 5 |
| What happens after a job fails, is rejected or stops in planning? | 6 |
| What does the role measure, and how often? | 7 |
| How are prompt budgets kept? | 8 |
| How is restructuring queued? | 9 |
| How does it relate to Hestia, the code-health check, Jev, the introspector and the requirements engineer? | 10 |
| What are the build jobs? | 11 |
| What was rejected? | 12 |
| What was not measured? | 13 |

## 1. What this is

BRIEF.md has two addenda that ask for one thing. The first (2026-10-25) asks to "make sure that the context
provided to agents is sufficient and good" and what could have been planned better. The second (2026-10-26)
says everything is for agents, so Athena should architect to be context efficient in what it writes, and some
role should own the craft. Both are cited as quoted in the request of job 20261026-043431-kgrm; the relay note
at `BRIEF.md:1067` maps the role to Hermes with Hestia.

In plain terms, the role answers two questions and keeps the answers from drifting:

1. **Part 1, what Athena writes.** How little context does an agent need to find, understand and safely change
   a piece of code, a doc, a record or a prompt? Fewer files to open, fewer lines to read, one place for each fact.
2. **Part 2, what agents are given.** Briefs, the system-facts file, recurring failure classes, the vocabulary
   and the spec. After a job goes wrong, what context would have prevented it, and what changes so the next
   agent reads it?

The role is a craft owner and a sweeper. It sets conventions, measures against them, and queues work. It edits
nothing itself and gates nothing.

### Why now: the evidence

Each figure carries its source. "Stated by the request" means the job's request gave it and `records/` was not
available here, so it is not measured in this design.

| Evidence | Figure | Source |
|---|---|---|
| `gh.rs` size | 5,276 lines | Measured at the base commit, `crates/athena-core/src/gh.rs`; the request says 5,276 |
| `produce_and_verify` | 737 lines, `run.rs` lines 1527 to 2263 | Measured at base from the `fn` line to its closing brace; the request says 724, so the function grew or was counted differently |
| Decision log | 3,261 lines at base, 3,231 in the request | Measured at base, `docs/decisions.md`; D129 and later entries explain the difference |
| Reader prompt | 1997 of 2000 (99.85%) | `docs/system-facts.md` line `budget reader: 1997 of 2000`, checked by `system_facts_budgets_match_computed` |
| Other budgets | contract 326 of 350, producer 1257 of 1300, verifier 1298 of 1300, planner 788 of 1300, critic 742 of 1300 | Same file |
| Critics finding missed callers in `gh.rs` and `produce_and_verify` | not counted | Stated by the request, not measured here |
| 7 retries that lost an earlier verifier's findings | 7 jobs | Stated by the request, not measured here |
| Planning stops from facts no file stated | not counted | Stated by the request, not measured here |
| A planning stop is recorded as a failed job with the reason "planning stopped before any code was written:" | behavior | `docs/design/job-planning.md` lines 121 to 124 |

## 2. The mapping call: Hermes owns the craft, Hestia keeps it

**Confirmed, with a split.** The relay note proposes Hermes (context) working with Hestia (knowledge). I read
the pantheon rows (`BRIEF.md:597` and `:604`), `hestia.md` section 8, D127 and D129, and
`docs/design/roles-and-ladder.md`. Nothing in them conflicts. The call:

- **Hermes owns the craft.** Hermes is the orchestration god, whose row covers context and the passing of work
  between agents. Hermes sets the conventions in section 4, decides what agents read (section 5), and owns the
  outcome of each post-mortem (section 6).
- **Hestia keeps and sweeps.** Her row is knowledge, documentation and long-term quality, and `hestia.md`
  already gives her a periodic sweep that never gates. Her sweep runs the recurring checks for these
  conventions (section 10).

Reason for the split: the craft (what the rules are, what agents should read) changes rarely and needs
judgment; the upkeep (is this doc still indexed, is this file over the line) recurs and is mostly a count.
That is the same split `hestia.md` uses between rules and a model that only reports.

One naming caution, from `docs/ontology.toml` (entry "Hermes"): the name also means Hermes Agent, the second
studied system. Here it means only the pantheon role. Neither is part of the other.

Alternatives for the mapping are in section 12.

## 3. What exists today and what is only proposed

This design assumes nothing that is not in the first column's source. "Not built" means not in this snapshot.

| Thing | State | Source |
|---|---|---|
| Introspection after a `rejected` or `failed` job, a correction, or another intervention; record schema `athena.introspection/1` | Built | `crates/athena-core/src/introspect.rs` (module comment, lines 1 to 34); `defaults-loop.md` sections 3.1 to 3.4 |
| Jev asked the failure-class and is-repeat questions in shadow, answers stored in the record's `jev` field | Built | `introspect.rs` line 14 onward |
| A planning stop recorded as `failed` with a fixed reason prefix | Built | `job-planning.md` lines 121 to 124 |
| Retry brief section with the earlier findings | Built in part (D117, D128, and the retry section in the decision at `decisions.md` line 2800) | `decisions.md`; the 7 lost-findings retries are stated by the request, so how each lost them is not checked here |
| Prompt budgets and the exact-match test | Built | `docs/system-facts.md`; `crates/athena-cli/tests/budgets.rs`. The test fails on a stale line; it does not check headroom |
| Shared vocabulary | Built, 95 lines | `docs/ontology.toml` |
| Spec | Built, 859 lines | `docs/spec.toml` |
| Hestia's sweep | Proposed, not built | `hestia.md`, D129 |
| The code-health check (job 20261026-042558-5p65) | Proposed or in flight; no file for it in this snapshot | The request; not found in `docs/` or `crates/` |
| The Jev code-health checks (job 20261026-043137-zc0v) | Proposed or in flight; no `jev-code-health.md` in `docs/design/` | The request; `ls docs/design` |
| The requirements engineer (job 20261017-173506-k67x) | Proposed, not built | `BRIEF.md:1010` (item 4A); `requirements-map.md` has no row that covers it |
| A question-to-file map for docs | Partial: `docs/architecture.md` "Where things live" (line 61) maps code locations; no file maps questions | `docs/architecture.md` |
| A decision-log index | Not found at base | `docs/decisions.md` is read top to bottom; `docs/decision-links.toml` (168 lines) links decisions, and was not read as an index |

Where this design needs the code-health check or the Jev checks it says "needs 5p65" or "needs zc0v". It states
the interface it assumes (a per-file and per-function size report; a model-read judgment about structure) and
does not specify their output.

## 4. Part 1: what Athena writes

Each convention has a measure, a threshold, and an enforcement label. A label is `mechanism: <named measure>`
(code can fail on it) or `prose-only` (a person or model must judge). Every threshold is an **estimate** until
the first measurement (build job 1) and is then reset from data, with the reset recorded in a decision.

### 4.1 Code structure

The test of a good structure: a change touches few files, and a caller is easy to find. The evidence is that
critics kept finding missed callers in the two largest units (stated by the request).

| Convention | Measure | Threshold (estimate) | Today | Enforcement |
|---|---|---|---|---|
| **File size.** A source file stays small enough to read in part. | Lines per `.rs` file, test module counted separately | Warn above 1,500 lines of non-test code; queue a restructure above 3,000 | `gh.rs` 5,276 total; `run.rs` 3,090; `reading.rs` 2,779 (all measured at base, whole file) | mechanism: `size-report` (build job 1); needs 5p65 if that check lands first |
| **Function length.** A function fits one screen of reasoning. | Lines from `fn` to closing brace | Warn above 100; queue a restructure above 300 | `produce_and_verify` 737 | mechanism: `size-report` (build job 1); needs 5p65 if that check lands first |
| **One concern per module.** A reader can say what the module is for in one line, and a change to that concern stays in it. | (a) the module has a first-line doc comment naming one concern; (b) median files touched per landed code job over the last 20 jobs | (a) required; (b) at most 6 | (a) `introspect.rs` has one; not counted for the other files. (b) not measured | (a) mechanism: `size-report` flags a module with no doc comment (build job 1). (b) mechanism: `touch-report` from git history (build job 1). Whether the concern is truly one: prose-only, judged by the Jev checks (needs zc0v) |
| **Callers are findable.** A public item has a short, greppable name and few call sites in other crates. | Call sites in other files per public function | Flag above 15 | not measured | Needs zc0v for a model-read judgment; the count itself is mechanism: `size-report` (build job 1) |

### 4.2 Docs

| Convention | Measure | Threshold (estimate) | Today | Enforcement |
|---|---|---|---|---|
| **Short docs.** A doc is read whole in one pass. | Lines per `.md` and `.toml` under `docs/`, excluding `docs/design/reviews/` | Warn above 500; queue a split above 1,000 | `athena-v0.md` 1,964; `decisions.md` 3,261; `spec.toml` 859; `defaults-loop.md` 716 (line counts at base) | mechanism: `doc-size` check (build job 2) |
| **Indexed.** Every doc is reachable from one index. | Docs under `docs/` not named in `docs/index.md` | 0 | No `docs/index.md` at base | mechanism: `doc-index` check fails on an unlisted doc (build job 2) |
| **Question to file.** The index maps a question to the one file that answers it. | Questions in the map; files with no question | Every doc has at least one; each question names exactly one file | none at base | Coverage: mechanism: `doc-index` (build job 2). Whether the question is the one an agent asks: prose-only, tested by the post-mortem (section 6) |
| **One fact in one place.** A figure, rule or definition lives in one file; others link to it. | Normalized lines of 80 or more characters that appear in two or more docs under `docs/`, excluding `reviews/` and `decisions.md` | 0 new per change | not measured | mechanism: `dup-facts` check (build job 2). The budget lines already follow this rule: they live only in `system-facts.md` |
| **Decision log is indexed and has an active part.** A reader finds a decision without scanning the file. | Lines of `decisions.md`; entries without an index line | Active file at most 1,000 lines; 0 unindexed | 3,261 lines | mechanism: `decision-index` check (build job 3). Moving old entries to an archive is a later job, not this design |
| **Headings carry the answer.** A heading says what the section settles. | none | none | n/a | prose-only |

### 4.3 Records and prompts

| Convention | Measure | Threshold (estimate) | Today | Enforcement |
|---|---|---|---|---|
| **A record is readable alone.** The stored text a reader needs fits in the caps. | Stored detail against `DETAIL_CAP`, `TEXT_CAP`, `TAIL_CAP` | Caps as set in `introspect.rs` | Built | mechanism: the caps in `introspect.rs`, with tests there. This design adds no new cap |
| **A prompt set stays within budget.** | Size of each role's always-loaded set against its budget | Exact match today; headroom rule in section 8 | reader 99.85%; verifier 99.85% (1298 of 1300); producer 96.7%; contract 93.1% (computed from the budget lines) | Exact match: mechanism: `system_facts_budgets_match_computed`. Headroom and offset: mechanism: `budget-guard` (build job 5) |
| **A prompt line earns its place.** | Per-line: does a failure class or post-mortem outcome cite it? | Every line added after this design cites one | n/a | prose-only until `budget-guard` also reads the citation |

Every row in 4.1 to 4.3 is either enforced by a named measure that build jobs 1 to 5 create, or marked
prose-only. None is enforced today except the budget test and the record caps. The honest summary: this is a
design that adds enforcement, not a description of enforcement that exists.

## 5. Part 2: what agents are given

| Input | What is wrong or missing today | Owner of content | Hermes's concern |
|---|---|---|---|
| **Briefs** (producer, verifier, planner, critic, reader) | Briefs sit outside the prompt budgets (`system-facts.md` line 17), so nothing bounds their size; retries lost earlier findings in 7 jobs (stated by the request) | The brief builders in the code | Size per brief; whether a retry brief carries every earlier verifier finding |
| **The system-facts file** (`docs/system-facts.md`, 50 lines) | It is the place for facts no other file states; planning stops came from facts no file stated (stated by the request) | Hermes decides what goes in; the facts themselves are written by whoever found them | One fact in one place; each line cites a source; size within budget |
| **Recurring failure classes** (class registry, `defaults-loop.md` section 3.4) | Classes are recorded; whether agents read them before they start is not stated in any file I read | The introspection step and Jev | Which classes reach which prompt, and at what cost in bytes |
| **The vocabulary** (`docs/ontology.toml`) | A term is added when it causes a misread (`BRIEF.md:1011`) | The requirements engineer, once built (section 10) | Size and shape; whether agents read it within budget |
| **The spec** (`docs/spec.toml`) | 859 lines, one `statement` per entry with a quote | The requirements engineer, once built | Size; whether a job's brief names only the spec entries it needs |

### 5.1 Rule for what goes in a brief

A brief carries a fact when one of three things is true: the job cannot proceed without it; a post-mortem
outcome (section 6) names it; or it is the earlier verifier's finding on this same job. It carries a pointer
(a file and a heading) when the fact is long and only some jobs need it. This keeps briefs small and still
traceable. The rule is prose-only; its measure is the post-mortem outcome count in section 7.

### 5.2 Retries keep earlier findings

Seven retries lost an earlier verifier's findings (stated by the request). The record of what is carried today
is D117 and D128 and the retry section in the decision at `decisions.md` line 2800. The gap for build job 4 is
a check, not a guess: a retry brief must list every blocker and major finding of the previous attempt that was
not resolved, and a test fails when one is dropped.

## 6. The post-mortem step

The post-mortem is **an extension of the existing introspection record**, not a second step. The introspection
already fires on a job that ends `rejected` or `failed` (the `Rejection` and `Failure` triggers in
`crates/athena-data/src/records.rs`, lines 1206 to 1215) and already asks Jev for the failure class and whether
it repeats. A planning stop already ends as `failed` (`job-planning.md` lines 121 to 124), so it already fires
the `Failure` trigger. A second record would duplicate triggers, classes and storage.

**Triggers (three):** rejected, failed, and planning-stopped. The third is a failed job whose reason starts
with "planning stopped before any code was written:". The build adds a marker so the planning-stopped case is
counted on its own; it does not add a new event.

**The question,** added to the introspection step: *"What one piece of context, if the agent had read it, would
have prevented this?"* Jev answers with one outcome class and a one-line quote of the missing fact or
convention, and the answer is stored beside the existing `failure_class`. Like the rest of Jev's output, it is
stored in shadow and changes no outcome.

**Outcome classes (fixed list):**

| Outcome | What changes | Who changes it |
|---|---|---|
| `system-facts-line` | A line is added to `docs/system-facts.md`, citing the source | A small job queued by Hermes |
| `failure-class` | The class entry gains a sentence, or a class is added | The introspection step proposes; a small job lands it |
| `brief-assembly` | A brief builder includes or points to a fact it left out (for example the earlier findings in a retry) | A code job |
| `vocabulary-term` | A term is added to `docs/ontology.toml` | The requirements engineer, once built; before that, a small job |
| `spec-fix` | A spec entry is corrected or added | The requirements engineer, once built; before that, a small job |
| `structure` | A convention in section 4 was the cause (a missed caller in a huge file) | A restructure request (section 9) |
| `none` | No context would have helped; the cause was elsewhere | Nothing; counted |

**Displacement rule.** An addition to a prompt set above 95% of its budget must name what it displaces (a line
to remove or shorten) in the same change. The reader is at 99.85%, so any reader addition today needs an offset.
See section 8.

**What stops it growing.** A line added by a post-mortem is kept only if, within 8 weeks (estimate), a later
introspection cites it or a failure class of the same kind does not recur. A line with neither is queued for
removal. This is the prune half of the post-mortem, and is how the 95% rule stays livable.

## 7. What the role measures, and when

All thresholds are estimates until the first four weeks of data.

| Measure | How | Cadence | Threshold |
|---|---|---|---|
| Size report: files and functions over the section 4.1 lines | `size-report` (build job 1) | Every drain, and in CI-time tests | Any item over its queue line files a restructure request |
| Doc report: size, unindexed docs, duplicated facts, decision-log lines | `doc-size`, `doc-index`, `dup-facts`, `decision-index` (build jobs 2 and 3) | Every drain | Zero unindexed docs; zero new duplicates |
| Prompt headroom per role | `budget-guard` (build job 5) | Every drain | Above 95% needs a named offset |
| Median files touched per landed code job | `touch-report` (build job 1) | Weekly | At most 6 |
| Context-preventable share: post-mortems whose outcome is not `none`, as a share of all post-mortems | Introspection records with the new field | Weekly, over a rolling 4 weeks | Reported, not gated; a share falling over 8 weeks suggests the briefs improved |
| Recurrence: the same failure class returning after its context was added | Introspection `failure_class` and the `is-repeat` answer | Weekly | Any recurrence within 4 weeks reports that the added context was not read or not enough |
| Planning stops from unstated facts | Planning-stopped introspections whose outcome is `system-facts-line` | Weekly | Reported |

The cadence matches Hestia's: code checks every drain, the judgment-based ones weekly. Hermes reviews the weekly
report and decides what to queue. The measures count outcomes (a restructure landed, a recurrence avoided), not
activity (a report written), as `hestia.md` section 5 does.

## 8. Keeping context within budgets

- **Budgets are not raised to make room.** `system-facts.md` says: update the line, never the budget. This
  design keeps that. Raising a budget needs a decision of its own.
- **95% rule.** A change that raises a prompt set above 95% of its budget, or adds to one already above it,
  must name what it removes or shortens. Today the reader (99.85%), the verifier (99.85%) and the producer
  (96.7%) are above 95%; the contract (93.1%), planner (60.6%) and critic (57.1%) are not. All percentages are
  computed from the budget lines in `system-facts.md` and rounded to one place.
- **Relation to Hestia's flag.** `hestia.md` row 5 reports any role above 90% (estimate). The two numbers are
  deliberate: Hestia reports at 90% so someone looks early; Hermes's rule binds at 95% so a change is a
  trade-off. If the 90% flag is never raised before 95%, the numbers are reset together.
- **Briefs are outside the budgets.** Their size is bounded only by what the builders put in. Build job 5 adds a
  measured per-role brief size so growth is visible; a cap, if any, is set after one month of data.
- **Check.** An outside reader can test this section: compute each budget line's ratio, compare with 95%, and
  look for the offset in the change that moved it.

## 9. Restructuring: queued as ordinary requests

The role never edits a file. When a measure slips, Hermes queues a restructure request through the normal
queue and pipeline, as `hestia.md` does for fixes, so every change is verified.

- Each request names the **slipped measure** and the figure ("`gh.rs` is 5,276 lines against a queue line of
  3,000").
- A split is **behavior-preserving**: existing tests pass unchanged, and callers are found by the compiler.
  A job that changes a test to make a split pass is rejected.
- **Cap:** at most 3 restructure requests open at once (estimate), so a restructure backlog cannot crowd out
  ordinary work. When the cap is full, new slips are listed in the weekly report and not queued.
- **First candidates, named only:** `gh.rs` (5,276 lines) and `produce_and_verify` (737 lines). This design does
  not split either; that is out of scope here.
- **No blocking.** A job that touches an oversized file is not stopped or delayed. Gating belongs to another
  role (D127), and an urgent job would stall. A touched oversized file is noted in the weekly report.
- **Order:** the largest overshoot of its queue line first; a function before the file that holds it.

## 10. Relations

- **Hestia (proposed, `hestia.md`).** Hestia's sweep runs the recurring doc checks of section 4.2, as her row 8
  runs mechanical doc-drift checks, and reports prompt headroom as her row 5. Hermes sets the rules; Hestia runs
  and reports them. Hestia stays non-gating (D127). If her sweep is not built when build jobs 1 to 5 land, the
  checks run on their own and are folded into her sweep later. `hestia.md` section 8 carries a cross-reference.
- **The code-health check (5p65).** Proposed, not in this snapshot. Hermes consumes its file and function size
  report as the data for section 4.1 and does not rebuild it. If it lands first, build job 1 shrinks to the
  thresholds and the restructure filing. If it does not, job 1 builds a minimal `size-report`.
- **The Jev code-health checks (zc0v).** Proposed, not in this snapshot. They would supply the model-read
  judgments (is this one concern; is a caller easy to find). Hermes treats their output as a report with
  quotes, as with every Jev answer: shadow first, no gate.
- **The introspector (built).** The post-mortem is a field and a trigger marker on the existing introspection
  record (section 6). Hermes reads its output; the step stays the sole writer of the record.
- **The requirements engineer (proposed, `BRIEF.md:1010`, job 20261017-173506-k67x).** Not built. It owns the
  *content* of the spec and the vocabulary: what a term means, whether a spec entry says what Sri meant. Hermes
  owns their *size, shape, index and whether agents read them within budget*. A `vocabulary-term` or `spec-fix`
  outcome goes to the requirements engineer once it exists, and to a small job before then. The two roles
  share no file edits: one changes meaning, the other changes shape.
- **Artemis and Demeter.** Unchanged. The scope guard (D127) weighs work; this role does not gate. Demeter's
  cost view includes the post-mortem's Jev call like any other.

## 11. Build plan

Six jobs, each one producer run (a focused change of a few files, with tests). None is queued by this design;
queueing needs D130 accepted. Each job's acceptance bullets show a failing case first, then the passing one.

**Job 1. Size and touch report.**
Delivers: `size-report` (lines per file with test code separate, lines per function, doc-comment presence,
cross-file call-site counts) and `touch-report` (files touched per landed code job from git history), with the
thresholds of section 4.1 as constants. Depends on: none (or reuse of 5p65 if it has landed).
Acceptance:
- On a fixture repository holding one 1,600-line file, one 120-line function and one module with no doc comment,
  the report lists exactly those three, each with its measured number.
- On the same fixture with all three fixed, the report is empty and says how many files and functions it read.
- Run on the real tree, it reports `gh.rs` and `produce_and_verify` with numbers equal to `wc`-style counts of
  the files (a test compares the two).
- A run on a tree it cannot read fails with an error, and does not report "no findings".

**Job 2. Docs index, question map, size and duplicate checks.**
Delivers: `docs/index.md` mapping questions to files; `doc-index`, `doc-size` and `dup-facts` checks.
Depends on: none.
Acceptance:
- Adding an unindexed `docs/x.md` makes `doc-index` fail naming `docs/x.md`; indexing it passes.
- A doc of 501 lines warns and one of 1,001 lines files a restructure; one of 400 does neither.
- Pasting an 80-character line from `system-facts.md` into a second doc makes `dup-facts` fail naming both files.
- Every existing doc outside `reviews/` is indexed after the job, shown by the check passing on the real tree.

**Job 3. Decision-log index and link check.**
Delivers: a one-line-per-decision index for `docs/decisions.md` and a `decision-index` check; a design for the
archive, without moving any entry. Depends on: none.
Acceptance:
- Deleting an index line for D129 makes the check fail naming D129; restoring it passes.
- An index link to a heading that does not exist fails the check.
- `docs/decisions.md` itself is byte-identical before and after the job (a test compares it to the base copy).

**Job 4. Post-mortem field and retry findings.**
Delivers: the `context that would have prevented it` field and fixed outcome list on the introspection record,
the planning-stopped marker, the question added to Jev's call, and a retry brief that lists every unresolved
blocker and major finding of the previous attempt. Depends on: none.
Acceptance:
- Through the existing introspection path, a fake job that ends `failed` with the planning-stop reason yields a
  record with the marker and an outcome from the fixed list; an outcome outside the list is stored as an error
  and blocks nothing.
- A fake job that ends `rejected` and one that ends `failed` for another reason each yield the field; a job that
  lands yields no record.
- A retry brief built from a previous attempt with two blocker findings contains both; a test that removes one
  from the builder's input fails (shown before the fix on the same fixture).
- A failed Jev call stores an error in the field and leaves the job's outcome unchanged.

**Job 5. Budget guard and brief sizes.**
Delivers: `budget-guard` (95% rule with a named offset), per-role brief size recorded in the job record.
Depends on: none.
Acceptance:
- A fixture change that takes a set from 94% to 96% without an offset fails; the same change with an offset line
  naming what it removes passes; a change that lowers a set above 95% passes without one.
- The guard run on the real tree reports the reader at 99.85% and the verifier at 99.85%, as recomputed from
  `system-facts.md`.
- The existing exact-match test still fails on a stale budget line (shown by editing a line in a fixture).

**Job 6. Restructure-request filer with cap.**
Delivers: turn a slipped measure into a queued request that names the measure, with the cap of 3. Depends on 1.
Acceptance:
- Four slips with a cap of 3 file three requests and list the fourth in the report; none edits a source file.
- A request's text names the slipped measure and its figure; a request without one is refused.
- When an open request already covers the same file, no duplicate is filed.

Not in the plan: splitting `gh.rs` or `produce_and_verify`, rewriting or archiving the decision log, and any
edit of prompts or `docs/system-facts.md`. Those follow as ordinary requests once the measures exist.

## 12. Alternatives considered

1. **Hestia owns everything.** Rejected: her design is already a ten-job sweep and keeps her out of gating
   (D127); a craft owner would blur it. Upkeep fits her; the rules and the post-mortem outcome need an owner of
   context.
2. **A new named role.** Rejected: no evidence needs one, and Hermes's row covers context.
3. **Hermes owns part 2 only.** Rejected: part 1 (file size, doc shape) is what makes part 2 cheap. Splitting
   loses the link between a big file and a missed caller.
4. **A separate post-mortem record.** Rejected: it duplicates triggers, failure classes and storage that
   `introspect.rs` and `records.md` already hold, and two records would drift.
5. **A free-form lessons file.** Rejected: it grows without bound, nobody reads it, and it breaks one fact in
   one place.
6. **Post-mortem only on failures.** Rejected: the request names rejected and planning-stopped jobs too.
7. **The role edits files directly.** Rejected: skips verification.
8. **Block jobs that touch oversized files.** Rejected: gating belongs to another role and would stall urgent work.
9. **Specify the code-health check output as if it exists.** Rejected: it is not in the snapshot.
10. **Omit thresholds until measured.** Rejected: a convention without a number cannot be checked; estimates are
    labeled and reset in job 1.

## 13. Not measured, and open points

- The 7 retries that lost an earlier verifier's findings, the planning-stop counts and the missed-caller finds
  come from the request. `records/` was not available, so none is re-measured. Job 4's first output is the
  count.
- Every threshold (1,500 and 3,000 lines, 100 and 300 lines, 500 and 1,000 doc lines, 6 files, 95%, 8 weeks,
  the cap of 3) is an estimate. Job 1 and the first four weeks of reports reset them.
- Whether 5p65 and zc0v landed after the base commit is unknown; section 10 states the interface assumed.
- `produce_and_verify` measured 737 lines here against 724 in the request; the difference is not explained.
- Percentages in sections 4.3 and 8 are computed from the `system-facts.md` budget lines and rounded.
