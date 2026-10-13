# The defaults-learning loop: repeats become proposed, versioned defaults

**Status:** proposed (D90). This document builds nothing. It changes no Rust, no tests and no `defaults.toml`.
A later job reviews it; nothing is built before that review (the `designed` default). Paths are relative to
the repository root. The reader is someone who knows what `defaults.toml` is (D26) and what an intervention
record is (`docs/records.md`) but has not seen the jobs that motivated this.

Claims are marked **(read)** when taken from a file named beside them, **(design)** when they are proposals,
**(reported)** when they come from the job request and were not checked here, and **(estimate)** for numbers
that nobody measured.

## 1. The problem

`defaults.toml` is Athena's model of what its user expects without saying so. It has 12 entries today
**(read: `defaults.toml`, counted)**; D26 said eleven when it was written, and `root-cause` was added since
**(read: `docs/decisions.md` D26)**. The seed entries each cite `BRIEF.md`. Nothing yet turns what the user
repeatedly has to say back into an entry. The roadmap names the gap: RM-4 "feed scored mispredictions back into
`defaults.toml`" and RM-3 "close the intervention loop" (`docs/design/roadmap.md`) **(read)**.

When the user corrects a reading, answers a question, or fixes something by hand, Athena records an
intervention (D27) **(read)**. The record carries `what`, `why`, `class_of_work`, a `category` (one of eight,
such as `corrected-output` or `overrode-decision`), the job, and a source
**(read: `crates/athena-data/src/records.rs`, `Intervention`, `InterventionCategory`)**. Today a person has to
notice that the same thing happened again and then write a defaults change. The change request asks to "close
the defaults loop: use Jev to match interventions and corrections against the defaults model, so repeats
become new, versioned defaults" **(reported: the job request)**.

## 2. Evidence: four repeats

**Access caveat.** The `records/` tree is not part of the snapshot this document was written from. The
intervention and correction records behind the four repeats could not be opened, so their record ids are not
cited below, and the "x4" counts come from the job request **(reported)**. Where the repository itself shows
the repeat, that is cited **(read)**. The job that builds this design must list the actual record ids for each
repeat and replace every **(reported)** mark below with them, or say which it could not find.

| # | Repeat | Count | Evidence |
|---|--------|-------|----------|
| R1 | A change reading's scope criteria excluded `CHANGELOG.md`, so the verifier saw a changed file outside scope | x4 **(reported)** | D89 is the repair made in code: change readings now always list `CHANGELOG.md`, and the verifier brief says the deliverables list is the one statement of scope **(read: `docs/decisions.md` D89)**. D89 rejected adding a `changelog` entry to `defaults.toml` because "the defaults model is the user's, not changed here" **(read)**. That is the loop this design supplies. |
| R2 | The reader's always-loaded set went over its token budget and blocked the job | x4 **(reported)** | The reader budget is 1,800 tokens, measured live, and over-budget sets are refused **(read: D25; `crates/athena-context/src/lib.rs` message "over its budget")**. D89 records that adding a changelog line to `reader.md` "put the reader prompt over its 1800-token budget" **(read)**. Only that one instance is visible here. |
| R3 | One job bundled design, review and build | x1 visible **(reported for the rest)** | The request names the repeat. The nearest written rules are `small` ("prefer one focused change") and `designed` ("reviewed before they are built") **(read: `defaults.toml`)**. No record of an instance is visible here, so the count of this repeat is **unverified**. |
| R4 | A new config field and its first use landed in one commit | not visible **(reported)** | The request names the repeat. No written default covers it. No record of an instance is visible here, so the count is **unverified**. |

What the four show, in terms of the loop:

- R1 and R2 are repeats the existing defaults did not cover. Each was fixed in code or prose, once, after
  several corrections. A proposal would have named the gap sooner.
- R3 partly matches an existing entry (`small`). It is the test case for the matching step: Jev should say
  "close to `small`, not covered by it", and the proposal should amend rather than duplicate (§5.2).
- R4 matches no entry. It is the test case for a new id.

## 3. The loop in one view

```
 intervention / correction record  (athena.intervention/1, plus reading corrections)
        |
        v
 [1] Jev: P(record matches default d) for each d in defaults.toml, plus P(none)       <- model
        |
        v
 [2] code: threshold. At or above: "matched" (counts as a violation of d).
           Below: goes to the unmatched pool.                                          <- deterministic
        |
        v
 [3] Jev: cluster the unmatched pool and the matched-but-repeated records; say which
     records say the same thing                                                         <- model
        |
        v
 [4] code: a cluster of at least N distinct jobs becomes a candidate                    <- deterministic
        |
        v
 [5] Jev: draft a statement, scope and a short title for the candidate                  <- model
        |
        v
 [6] code: choose the id, check it is unique, compute the version bump, build the
     candidate file text, validate with DefaultsModel::parse, write the proposal        <- deterministic
        |
        v
 [7] user reviews the proposal; Athena never edits defaults.toml itself                 <- user
```

The rule behind the split: Jev answers questions that have the shape "which of these, and how likely". Code
makes every decision that has a rule, and every write.

## 4. What Jev decides and what code decides

Jev (`typesafe-ai/jev`, through the gateway) returns choices, scores and probabilities, with a 32K context
**(read: `BRIEF.md` lines 25 and 144)**. A probe answered a typed forecast question in about 0.75 s at about
$0.000012 per call **(read: `docs/design/athena-v0.md` E12, one probe; not re-measured)**. The whole defaults
list rendered by `DefaultsModel::render()` is one line per default and fits in that context with room to spare
(**estimate:** under 1,500 tokens for 12 entries, from the reader budget of 1,800 that holds the whole model
plus the reader prompt; D25).

| Decision | Who | Why |
|---|---|---|
| Probability that a record matches default `d`, for each `d`, and for "none" | **Jev** | A typed question over text; the probability is what the threshold reads. |
| Which records say the same thing (clustering, paraphrase) | **Jev** | Two corrections worded differently are one repeat. A string match would miss R1 worded four ways. |
| Draft statement, scope (`applies_to`) and title for a new or amended default | **Jev** | Drafting prose from several records. The draft is text for review, not a decision. |
| The match threshold | **code** | A number in `athena.toml`, set from calibration (§8). |
| How many distinct jobs make a candidate (N) | **code** | A number in `athena.toml`. Repeats inside one job count once. |
| Whether the id is unique and well formed | **code** | Jev's suggested id is only a suggestion. Code derives a kebab-case id from the title and refuses any id already in the model or in a pending proposal; `DefaultsModel::parse` also rejects duplicates **(read: `DefaultsError::Duplicate`)**. |
| Version bump | **code** | The model has one integer `version` **(read: `DefaultsModel`)**. A proposal bumps it by one, computed against the base commit's value. Jev never writes a version. |
| Whether the source records exist | **code** | Every cited record id must exist at the base commit, belong to the cluster, and be an intervention or correction. |
| Validity of the resulting file | **code** | Code builds the full candidate `defaults.toml` text and calls `DefaultsModel::parse`: unknown fields, duplicate ids, and empty statement or source all fail **(read: `crates/athena-data/src/defaults.rs`)**. |
| Writing the proposal record | **code** | Only code writes. Jev's output is parsed against a typed schema; anything that does not parse is dropped and logged, never repaired. |

Two safeguards follow from this split:

- **Jev never sees a decision it could make unaccountable.** Its probabilities are recorded beside the
  threshold, so a wrong proposal can be traced to one number.
- **Low Jev confidence escalates, it does not guess.** A record whose best match is between a lower and the
  upper threshold goes to a stronger model, as the brief asks ("use Jev's probabilities to escalate
  low-confidence calls", `BRIEF.md` line 217) **(read)**. In shadow mode (§8) that stronger model also runs on
  everything, to be the comparison.

Jev's input is built only from stored fields: `what`, `why`, `class_of_work`, `category`, and the text of a
reading correction. It never sees outcomes or scores. This is the rule D31 set for Jev's forecasts, and it
keeps every match computable again from records alone **(read: D31)**.

## 5. Matching, clustering and candidates

### 5.1 Matching

For each new record, Jev gets the record's stored fields and the rendered defaults (id, scope, first
sentence), and returns a probability for each id and for "none". Code applies the threshold `t_match`. A
record at or above it is *matched* to the highest-probability id. A match is not a proposal: it is a count
against an existing default.

Matching also serves a second purpose that needs no new default: a default matched often is one the briefs are
not getting across. The report (`athena status`, §6) lists match counts per default.

### 5.2 Clustering

Unmatched records, plus records matched to the same default more than a few times, are clustered by Jev. Code
reads the clusters and counts distinct jobs, not records. A cluster of at least N jobs becomes a candidate.
N is a design choice, not a measurement. The proposed start is **N = 3 (estimate)**, below the four-fold
repeats in §2; calibration (§8) can move it.

A candidate is one of two kinds:

- **add:** no existing default is a good match. A new id.
- **amend:** the cluster matches an existing default at moderate probability, and the draft widens or
  sharpens that statement. The id is kept. R3 is expected to take this form against `small`.

### 5.3 Drafting

For each candidate Jev drafts the statement in the voice of the existing entries (one or two sentences), and
`applies_to` from the repo's classes of work. Code checks `applies_to` values against the classes seen in the
cluster's records plus `*`, and refuses a statement that is empty or longer than the length budget (§6).

## 6. The proposal format

A proposal is a record, not an edit to `defaults.toml`. It is written like other records (`docs/records.md`)
and its schema is added when it is built. Name: `athena.defaults-proposal/1` **(design)**.

| Field | Meaning |
|---|---|
| `id` | The default's id: new for `add`, existing for `amend`. Unique among the model and pending proposals (code-checked). |
| `kind` | `add` or `amend`. |
| `statement` | The full statement text that would appear in `defaults.toml`. For `amend`, the whole new statement; the prior one is shown beside it. |
| `applies_to` | The scope list, as in `defaults.toml`; `*` means every class. |
| `version` | `{ from, to }`: the base commit's model version and `from + 1`. |
| `sources` | The record ids of the cluster (at least N distinct jobs), each with its job id. These become the `source` text of the entry when accepted. |
| `evidence` | Jev's match probabilities for each source record, the threshold used, and which model produced the cluster and the draft. |
| `status` | `pending`, `accepted` (with the job that applied it), `rejected` (with the user's reason), or `withdrawn`. |
| `candidate_sha256` | Hash of the full candidate `defaults.toml` that passed `DefaultsModel::parse`. |

Code enforces a statement budget: a proposed statement may not be longer than the longest statement now in the
model. This is because the model is loaded inside the reader's token budget, and R2 shows what an overrun
costs (D25) **(read)**. Each added entry costs reader tokens, so a proposal states its rendered size
(**estimate:** ⌈bytes/3.0⌉, the CI tripwire estimate in D25) and refuses to be written if the reader set would
go over 1,800 tokens.

**Example (illustrative draft for R4; not accepted, not in `defaults.toml`, source ids are placeholders because
the real records were not readable):**

```toml
# proposal: defaults-proposal/1
id = "field-with-use"
kind = "add"
statement = "A new configuration field lands in the same commit as its first use, with a test that reads it. A field with no reader is not added."
applies_to = ["code", "config"]
version = { from = 1, to = 2 }
sources = [
  "intervention:<id-1> (job <job-1>)",
  "intervention:<id-2> (job <job-2>)",
  "intervention:<id-3> (job <job-3>)",
]
status = "pending"
```

The example's version follows the base file: `defaults.toml` has `version = 1` today **(read)**. The model's
version is a count of accepted changes, not of entries (the file has 12 entries at version 1).

If several proposals are accepted together, they land in one job and bump the version once for that job;
`to` is recomputed by code at acceptance, never trusted from the proposal.

## 7. How a proposal reaches the user, and why Athena never edits `defaults.toml`

`defaults.toml` is a governance file: changing it is a job that lists it as a deliverable, and each change
cites the evidence that pulled it in (D26; header of `defaults.toml`) **(read)**. This design keeps that rule
whole. Proposals are review-only.

1. **Generation.** After a job closes, or on a command, code runs the loop (§3) over records not yet seen.
   Output is proposal records, nothing else. No path in the loop opens `defaults.toml` for writing; a test
   when built should assert the loop's code has no such call **(design)**.
2. **Surfacing.** `athena status` lists pending proposals: id, kind, statement, source count, and the version
   bump. It shows, for each, the source records and Jev's probabilities. A proposal is never injected into a
   brief: the reader and the briefs see only the accepted model at the base commit (D26).
3. **Decision.** The user reads the proposal and chooses. To accept, the user asks for a job in the ordinary
   way ("apply defaults proposal X"). That reading lists `defaults.toml` as a deliverable, the producer makes
   the edit, and the verifier checks: the id and statement equal the proposal, the version is the base
   version plus one, and `DefaultsModel::parse` accepts the file. To reject, the user records `rejected` with a
   reason; code does not propose the same cluster again unless it grows by N more jobs.
4. **Landing.** It lands like any job. The accepted entry's `source` cites the proposal's records, so every
   learned default is as traceable as the seed ones are to `BRIEF.md`.

Reasons for the rule: a wrong default quietly shapes every future brief; the model is the user's, not
Athena's (D89 says so in terms); and a loop that edits its own rulebook cannot be told apart from drift.

## 8. Calibration before trust

Jev is not part of v0 (D31) **(read)**. D31 left it out because it would add an HTTPS client, a credential, a
driver and its fake, and refusal handling, and because its forecasts could be computed retroactively from the
reading records. Its path back was named: "the first calibration review, or the first typed decision in the
flow" (`docs/design/athena-v0.md` §14) **(read)**. This design is that typed decision. It does not reverse D31
for any other use. D90 narrows D31 to: Jev may be added, in shadow only, for this loop, and nothing
below runs live until the promotion criteria hold.

### 8.1 Shadow mode

- Jev runs on every new intervention and correction. Its match probabilities, clusters and drafts are
  recorded. They produce no proposal the user sees as live, and change no count in `athena status` that the
  user relies on.
- A stronger model runs the same typed questions on the same inputs (the brief: "shadow new decision types
  against a stronger model", `BRIEF.md` line 217) **(read)**.
- Shadow runs have a spend cap, like every invocation (D28) **(read)**.

### 8.2 The labeled set

- **Source:** all intervention and correction records now in `records/`, plus reading corrections, labeled
  retroactively under D31's rule: Jev's input is built only from stored fields. The number of such records is
  not known here because `records/` is not visible **(unknown; the building job counts it and states it)**.
- **Labels:** for each record, the user, or a stronger model whose labels the user spot-checks, names the
  default it matches or "none", and groups repeats. Who labeled which record is stored.
- **Must include** the four repeats in §2 and, for each, records that look similar but are not repeats.
- **Split:** by time. Records up to a cut date calibrate the threshold and N; later records test. No tuning on
  the test part.
- **Label hygiene:** labels are written before Jev's output is read, so Jev is not anchoring them.

### 8.3 Scoring

- **Matching:** the Brier score of Jev's probability vector against the label, with two references: always
  predicting "none", and a plain word-overlap matcher that needs no model. The same shape as athena's
  existing forecasters, where "the reference is what makes a vague forecaster show zero skill"
  (`docs/design/athena-v0.md`, near line 1208) **(read)**. Skill is the Brier improvement over the better
  reference.
- **Top-1 accuracy** on the matched-or-not decision, and the share of "none" labels that Jev calls matched
  (the false-match rate), because a false match hides a gap.
- **Clustering:** whether each labeled repeat group lands in one cluster, and whether any cluster mixes
  groups.
- **Drafts:** a person (or the stronger model, spot-checked) rates whether the draft statement says what the
  cluster's records say. Scored as pass or fail per draft.
- **Cost and latency** per decision, tracked like any other tier (`BRIEF.md` line 217) **(read)**.

### 8.4 Promotion criteria

Jev's proposals become visible in `athena status` as live (still review-only) when all hold. The numbers are
starting points to be set at review and are **estimates**, not measurements:

1. Matching skill is above zero against both references on the test part, and the lower bound of the skill is
   above zero. The labeled set has at least **30 records (estimate)** with at least 5 distinct defaults or
   "none" outcomes represented.
2. All four repeats in §2 form one cluster each in shadow, with no cluster that mixes them. If the real
   records cannot be found for R3 or R4, they are scored on what exists and the result says so.
3. The false-match rate on "none" labels is under **10% (estimate)**.
4. No proposal in shadow cites a record id that does not exist (this is already a code check, so it must be
   zero), and every draft passes `DefaultsModel::parse` in the candidate file.
5. At least **90% (estimate)** of drafts are rated as faithful to their sources.
6. The user reviews the shadow report and agrees.

**Demotion:** if a live window's agreement with the stronger model drops, or the user rejects proposals for
the reason "Jev misread the records" twice in a row, Jev returns to shadow. These counts are estimates.

Until promotion, the loop's only live part is the deterministic path from a person-written proposal request
to the same record format, so the review path (§7) can be built and used first.

## 9. Alternatives considered

1. **A person notices repeats and writes the change (status quo).** Costs no code. It is the loop that
   missed R1 four times before D89 repaired it in code (reported count; D89 read).
2. **Rules instead of a model: keyword and category counts.** Cheap and deterministic. It fails on
   paraphrase and needs a person to write the keywords; it is kept as the reference in §8.3, not as the
   decision-maker.
3. **A stronger model for everything, no Jev.** Likely the most accurate matcher. It costs more per record
   (**estimate:** one to two orders of magnitude over Jev's cost per call in E12, not measured), and the brief
   asks for Jev on typed decisions.
4. **Jev decides everything, including the threshold and writing the file.** Rejected: it removes the review,
   and Jev does not verify its own output. The split in §4 is the reply.
5. **Jev proposes, and Athena applies accepted proposals itself.** Rejected: this makes Athena edit the user's
   model (§7). The accept step is a job instead.
6. **Auto-land low-risk defaults.** Out of scope for this change, and rejected for the same reason.
7. **Put learned defaults in a separate file Athena owns.** This avoids editing `defaults.toml`, but splits
   the model in two; briefs and the reader would load two sources, and the version would have no single
   meaning. Rejected.
8. **Count records, not jobs, for N.** Rejected: one job with four corrections is one episode, not four.
9. **Skip shadow mode, since Jev's output is only review text.** Rejected: a plausible but wrong proposal
   costs the user's attention, and a plausible but wrong accepted default costs every later brief.
10. **Do this inside the dashboard (RM-12).** Later; the dashboard needs the records and the proposal format
    first.

## 10. What is left open

- The real record ids and counts for R1 to R4 (§2). This is the first task of the building job.
- Where the loop is triggered: after each job close, or on a command. This design prefers a command first,
  with the close trigger after promotion.
- Whether an `amend` should be allowed to shrink a statement. Starting rule: no; shrinking is a hand change.
- Whether the draft model should see accepted and rejected proposals as examples. Not decided; it changes
  what shadow mode must hold out.
- Whether classes of work (`applies_to` values) need a registry so code can check them exactly. Today they
  are free text in `defaults.toml`.
- A separate independent review of this design is still to be done; none is recorded.

## 11. Build order, if accepted

1. Add the proposal record and its validation, and a person-written proposal command (no Jev).
2. Add `athena status` surfacing and the apply-as-a-job check (§7).
3. Build the labeled set and the scoring (§8.2, §8.3) with a fake Jev, then the Jev client.
4. Run in shadow; review; promote under §8.4.
