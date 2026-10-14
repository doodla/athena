# The defaults-learning loop: introspection on every correction, and repeats become proposed, versioned defaults

**Status:** proposed (D90), revised after review 1 (`docs/design/reviews/defaults-loop-review-1.md`), extended
with introspection as a first-class step (`BRIEF.md`, addendum "2026-10-13: Introspection on every correction"),
and revised again after the joint review of this document with `docs/design/decision-sets.md`
(`docs/design/reviews/decision-sets-review-1.md`, accept with changes; its section 6 changes are applied in both
documents). Awaiting the confirming look at that revision. This document builds nothing. It changes no Rust, no tests and no `defaults.toml`. A later job reviews it;
nothing is built before that review (the `designed` default). Paths are relative to the repository root. The
reader is someone who knows what `defaults.toml` is (D26) and what an intervention record is
(`docs/records.md`) but has not seen the jobs that motivated this.

Claims are marked **(read)** when taken from a file named beside them, **(design)** when they are proposals,
**(reported)** when they come from the job request, the user's probes or the review and were not checked here,
and **(estimate)** for numbers that nobody measured. Jobs are cited by job id and pull requests by number; no
person is named.

## 1. The problem

`defaults.toml` is Athena's model of what its user expects without saying so. It has 12 entries today
**(read: `defaults.toml`, counted)**. D26 says "eleven"; that text was written before `root-cause` was added and
is corrected in the same change as this revision **(read: `docs/decisions.md` D26)**. The seed entries each cite
`BRIEF.md`. Nothing yet turns what the user repeatedly has to say back into an entry. The roadmap names the gap
in RM-4, "feed scored mispredictions back into `defaults.toml`" **(read: `docs/design/roadmap.md`)**. RM-3,
"Close the intervention loop", is the nearby item: it is about a trend per class of work, an audit step and a
link to an improvement job, not about defaults **(read: same file)**. This design supplies the part of RM-3 that
turns a correction into a diagnosis (§3) and the part of RM-4 that turns repeated diagnoses into defaults.

When the user corrects a reading, answers a question, or fixes something by hand, Athena records an
intervention (D27) **(read)**. The record carries `what`, `why`, `class_of_work`, a `category` (one of eight,
such as `corrected-output` or `overrode-decision`), the job, and a source
**(read: `crates/athena-data/src/records.rs`, `Intervention`, `InterventionCategory`)**. A correction record
carries the intervention recorded for it **(read: `docs/records.md`)**. Today a person has to notice that the
same thing happened again and then write a defaults change, and each retry after a failure starts from the user
rewriting the request. The change request asks to "close the defaults loop: use Jev to match interventions and
corrections against the defaults model, so repeats become new, versioned defaults" **(reported)**, and the
brief's addendum asks that every correction be followed by introspection by Athena itself **(read: `BRIEF.md`)**.

## 2. Evidence: four repeats, and what they do and do not prove

**Access caveat.** The `records/` tree is not part of the snapshot this document was written from. The
records behind the repeats could not be opened, so their record ids are not cited, and the counts come from the
job request **(reported)**. Where the repository itself shows the repeat, that is cited **(read)**.

| # | Repeat | Count | What it is |
|---|--------|-------|------------|
| R1 | A change reading's scope criteria excluded `CHANGELOG.md`, so the verifier saw a changed file outside scope | x4 **(reported)** | **Closed in code by D89.** Change readings now always list `CHANGELOG.md`, and the verifier brief says the deliverables list is the one statement of scope **(read: D89)**. R1 cannot recur as it did. It is a test case for matching (a correction that should land on "none" or at most `small`), not proof that the loop is needed. |
| R2 | The reader's always-loaded set went over its token budget and blocked the job | x4 **(reported)** | **A counter-example, not a defaults repeat.** The reader budget is 1,800 tokens, and an over-budget static set fails a **test**; it is not a check at job time **(read: D25; `crates/athena-context/src/lib.rs`, `check_budget`, called only from tests and `crates/athena-cli/tests/budgets.rs`; `docs/enforcement.md` row M18)**. What actually blocked the four jobs is reported and was not verified here. D89 records one instance **(read)**. The fault is caught by a test, so a default such as "keep prompts under budget" would restate a rule the tests enforce and cost reader tokens. R2 is labelled "none" in the labeled set (§10.2). Its other lesson, that four retries each patched the last finding, belongs to introspection (§3, case C2). |
| R3 | One job bundled design, review and build | x1 visible **(reported for the rest)** | The nearest written rules are `small` and `designed` **(read: `defaults.toml`)**. No instance is visible here: **unverified**. Test case for "close to `small`, not covered by it" (an amend, §7.2). |
| R4 | A new config field and its first use landed in one commit | not visible **(reported)** | Matches no entry. No instance is visible here: **unverified**. Test case for a new id. |

**The case for the loop is unproved.** Of the four, one is closed in code (R1), one is not a defaults repeat
(R2), and two have no visible instance (R3, R4). The review makes the same point **(read: review 1, F2)**.

**Gate before any build.** The building job's first task is to read `records/` and list the actual record ids
for each repeat. If fewer than **N distinct jobs** (N as in §7.2) show a repeat that a default would cover, the
loop is not built, and this design is closed with that finding. The step that needs no such evidence is
introspection (§3): it has value when a single failure is diagnosed, and it produces the evidence the gate
reads.

## 3. Introspection: the first-class step

The brief's addendum: the user corrected a body of work and the question is how Athena does what that
correction did, itself **(read: `BRIEF.md`, 2026-10-13 addendum)**. This section is the design for that.

### 3.1 Trigger

Athena runs an introspection after each of these, with no request from the user **(design)**:

| Event | Source in today's records |
|---|---|
| **Correction** of a reading or an answer to a reader question | intervention `corrected-output` or `answered-question` (D27) **(read)** |
| **Intervention** of any other category (a hand change, an overridden decision) | intervention record, other categories **(read)** |
| **Rejection** of a result, a proposal (§8) or a plan by the user | a rejected result is `OutcomeState::Rejected`, which is recorded today **(read: `crates/athena-core/src/run.rs` lines 925, 1042; `crates/athena-data/src/records.rs` line 96)**; a rejected proposal or plan has no record, and a new kind is added when built |
| **Failure**: a job fails, a check fails or never finishes, a budget test fails | job outcome and check results |
| **Host error**: the host (GitHub) refuses a call, whether or not a job fails | `athena.host-error/1`, a new record written by the client: `status_code`, `operation`, `job`, `phase`, `at` (decision-sets §4). Today the client keeps only error text **(read: `crates/athena-core/src/gh.rs` lines 47-53)**, so the status is lost |
| **Landing**: `athena land` runs, after the job is closed | the landing run, its host-error records and check results; a merge refusal (`gh.rs` line 485) happens here and fits none of the rows above unless it also fails a job |

An introspection is recorded even if the user never asks for a retry. A failed introspection (spend cap, model
error) is recorded as failed and never blocks or delays the job; it is itself a failure and is not
introspected again. **No recursion on firings:** the introspection recorded for a decision's signal firing
(decision-sets §4) is not itself watched, so a firing cannot cause a firing. The signal watcher of decision-sets §4
runs after job close, after each introspection and at landing, and the introspection triggers above include the
last two rows so that a host refusal is diagnosed even when no job fails.

### 3.2 The record

`athena.introspection/1` **(design)**, one per triggering event, linked to the intervention record of that event
by id (the intervention remains the single counted unit, §7.1):

| Field | Meaning | Written by |
|---|---|---|
| `what_happened` | What the event was, in plain words, with job ids | language model, from stored fields |
| `failure_class` | One id from the class registry (§3.4) or `new` with a proposed name | typed decision (Jev, §3.4) |
| `earlier_cases` | Ids of earlier events in the same class, with job ids | **code** (§3.3); the model may not add ids |
| `decisions_matched` | Ids of typed decisions whose signals the event matched (decision-sets §4, §5); empty if none | **code**; the model may not add ids |
| `cause` | The cause beyond this instance: why the system allowed it, not what was wrong this time | language model |
| `prevention_kind` | `default`, `code-check` or `design-change` | typed decision |
| `prevention` | What exactly would have prevented it: the default text, the check and where it runs, or the design change | language model |
| `proposed_request` | A complete next request, ready to submit | language model |
| `evidence` | Models that answered, probabilities, confidence (§6), cost and tokens | code |
| `status` | `recorded`, `failed`, or `acted-on` with the job that did | code |

The triggering event is the earliest valid citation of whichever signal fired. For an intervention signal that is
an intervention; for a derived host-error record it is the record's source intervention, so the introspection
links to an intervention in every case. A decision put under review with no introspection record is a finding
in the backtest (condition `d`), not a silent gap **(design)**.

The rule behind `cause`: a retry that patches only the last finding is the failure this step exists to stop
(case C2). The model is asked "what would have stopped this class, not this instance", and a record whose
`cause` restates `what_happened` is flagged by code (equal text after normalising) and rated as failed.

### 3.3 Who reads past records, and what is passed on

The introspection needs earlier cases, which live in `records/`. The agent snapshot has no `records/` by design.
Two ways to give it that history:

1. **A filtered view inside the agent snapshot.** A copy of selected past records is placed in the agent's
   working directory.
2. **Athena code outside the agent snapshot reads `records/` and passes on a filtered summary.**

**Chosen: 2 (design).** Reasons: the snapshot rule that records stay out is kept whole (a view is a hole in
it, and a rule with exceptions is no longer checkable); the agent cannot be made to quote a record it never
received; code can verify every cited id; and what crosses the boundary is a small, typed thing that can be
tested. Cost: the code that selects earlier cases must be right, since the agent cannot look further.

The reading code is a function of Athena itself, run before the agent is invoked. Exactly what it does:

- **Reads** the stored fields of intervention, correction and introspection records: `what`, `why`,
  `class_of_work`, `category`, job id, date, and for introspections the `failure_class` and `cause`; for host-error
  records the `status_code`, `operation`, `phase` and `message_class`; for check records the check name, its result
  and the job id (§5 lists the whole set).
- **Never reads** model transcripts, diffs, standard-error text, or files from other jobs.
- **Selects** earlier cases in two steps: code narrows by `class_of_work` and `category`, then a Jev choice
  question assigns the event to a class (§3.4); the earlier cases are the records already in that class.
- **Passes to the introspecting agent** (only this): the event's stored fields; the class registry entry chosen
  (id, one-line description); and for each earlier case up to a cap of **5 (estimate)**, its record id, job id,
  date and the one-line `what`. No other record text.
- **Passes onward** from the finished record, to the retry's reading: `proposed_request`, the `failure_class`
  and `cause`, and the earlier-case ids. The retry's agents do not get `records/` either; they get the
  reading's text, which already carries those fields. To the defaults loop (§4): the class, `cause`,
  `prevention_kind`, `prevention` and the event's intervention id, not `what_happened`.
- **Verifies** on return: every id in `earlier_cases` exists and is in the class; any id not on the list passed
  in is dropped and logged, not repaired.

### 3.4 Class matching is a typed decision for Jev

Matching a new case to an earlier class is "which of these, and how likely", the shape Jev is for. Code builds one
`choice` question per event for `POST /v1/evaluate` (§6): `state` is the event's stored fields; options are the
ids in the **class registry** plus `new`; each criterion is the class's one-line description. Code reads the
top probability and the confidence as in §6.2. When the answer is `new` (or below threshold), a language model
proposes a name and description; code assigns the id and adds the class to the registry as `proposed`; the user
may rename or merge classes at review. The registry is a record store, not a file Athena edits in the
repository. Choosing `prevention_kind` among its three values is also a `choice` question for Jev; writing the
prose is not.

### 3.5 Retries start from the record

When the user asks for a retry of a failed or rejected job, the new reading is seeded with the
introspection's `proposed_request`, shown to the user as a draft with the cause and earlier cases beside it. The
user can accept, edit or discard it; the original request is not rewritten by hand. Athena never submits the
retry itself **(design)**. If the previous introspection for the same class names a prevention already tried,
the reading shows that too, so a retry that would repeat a patch is visible as one.

### 3.6 Cases reported by the user today

These come from the job request and were not checked against records here, so each is **(reported)**. The class
names are the proposed starting registry; the prevention is what this design says would have stopped the class.

| Case | Jobs | Class (proposed) | Cause beyond the instance | Would have prevented it |
|---|---|---|---|---|
| C1. `CHANGELOG.md` excluded by the scope criteria | 8qkc, m1bs | scope omits a file the repository requires | the scope was derived from criteria wording, not from repository rules | **code check**: done, D89 **(read)** |
| C2. Reader budget failed four jobs in a row; each retry patched the last finding | 69gf, t1y6, 3p3y, jk8v | repeated budget overrun / retry patches the symptom | the always-loaded set has no headroom, and the retry started from the last finding, not from a cause | **code check**: measure the set against the live counter before the job runs (the counter exists, D25 **(read)**), plus **introspection**: the retry starts from the cause record (§3.5) |
| C3. "Checks not finished" on #47, with no diagnosis of a hung CI | #47 | failure reported without diagnosis | an unfinished check is recorded as a state, with no step that asks why | **code check**: when checks are unfinished past a deadline, capture the run's state and record a diagnosis; the cause of that hang is not known here |
| C4. A Haiku producer handed in untested work | 69gf | untested work from a weak producer tier | the `tested` default exists **(read)** but nothing in code enforces it for a tier | **code check** (the verifier requires a test for code changes) and a **design change** (tier choice by class of work) |
| C5. A plan to edit the owner-only `BRIEF.md` | 16zf | plan touches a file the user owns | the plan was not checked against the list of owner-only files | **code check**: refuse a plan that names an owner-only path as a deliverable; I did not check whether a guard exists today |
| C6. Plans bundling design and build | not named | job bundles design and build (R3) | `small` and `designed` exist **(read)** but a plan is not tested against them | **default** amended (`small`/`designed`) plus a **code check** that a plan lists only one of design or build |
| C7. Jev's interface assumed instead of checked | this design | external interface assumed | no step requires the interface to be read or probed before it is designed against | **default** ("check an interface you did not write before designing against it") and **review**; here review 1 caught it |

Reading the table: C1, C2, C4, C5 are prevented by code, not defaults; only C6 and C7 point to defaults. That is
the answer to the question the loop asks, and it is why `prevention_kind` is a field: the defaults loop (§4)
consumes the `default` ones, and the others are listed to the user as proposed code or design changes (§9). The
root-cause default covers all seven: a fix at the place the fault arises, not the instance.

## 4. The loop in one view

```
 event: correction / intervention / rejection / failure / host error / landing
        |
        v
 [0] Athena: introspection record (§3). Code reads past records and passes a filtered summary.
        |
        v
 [1] Jev: P(introspection or intervention matches default d), per d plus none          <- typed
        |
        v
 [2] code: threshold on top probability. At or above: "matched". Below: unmatched pool. <- deterministic
        |
        v
 [3] Jev: assign each record to a candidate cluster, or "new" (or a pairwise yes/no)    <- typed
        |
        v
 [4] code: a cluster of at least N distinct jobs becomes a candidate                    <- deterministic
        |
        v
 [5] language model: draft statement, applies_to and title for the candidate            <- not Jev
        |
        v
 [6] code: id, uniqueness, version bump, candidate file text, DefaultsModel::parse,
     write the proposal                                                                 <- deterministic
        |
        v
 [7] user reviews the proposal; Athena never edits defaults.toml itself                 <- user
```

The rule behind the split: Jev answers questions with the shape "which of these, and how likely" (a choice
among supplied options, or yes/no). A language model writes prose. Code makes every decision that has a rule
and every write.

## 5. What Jev, a language model and code decide

Jev (`typesafe-ai/jev`, through the gateway) returns choices, scores and probabilities, with a 32K context
**(read: `BRIEF.md` lines 25 and 144)**. It is "a decision model, not a language model" **(reported: the
gateway's refusal on the chat route, §6)**. A probe answered a typed forecast question in about 0.75 s at about
$0.000012 per call **(read: `docs/design/athena-v0.md` E12, one probe, route not recorded; not re-measured)**.
The rendered defaults list carries the id, scope and first sentence of each entry (§8.3); its size is measured
before it is relied on. The earlier guess of "under 1,500 tokens for 12 entries" is withdrawn: the reader set as a
whole measures 1,798 of 1,800 tokens **(measured in review 1: `cargo test --locked -p athena-cli --test budgets --
--nocapture`)**, which says nothing about the share `render()` takes. That share is **unmeasured**; the build
measures `render()` alone with the live counter (D25, §8.3).

| Decision | Who | Why |
|---|---|---|
| Which class an event belongs to (§3.4); `prevention_kind` | **Jev** (`choice`) | Typed question over text. |
| Probability that a record matches default `d`, for each `d` and for "none" | **Jev** (`choice`) | The top probability is what the threshold reads. |
| Which candidate cluster a record joins, or "new" | **Jev** (`choice`), or a pairwise `boolean` | Assignment, not free clustering. Pairs grow with the square of the pool, so assignment is the practical form. Cluster membership then depends on arrival order; the order is stored and the shadow run (§10) tests the effect. |
| Introspection prose: `what_happened`, `cause`, `prevention`, `proposed_request` | **language model** | Writing. Jev cannot. |
| Draft statement, `applies_to` and title of a candidate | **language model** (the stronger shadow model or the producer tier) | Writing. Its output is text for review, not a decision. |
| Whether a signal matches a record, and which decisions in a candidate set apply (decision-sets §5) | **Jev** (`boolean`), in shadow only; exact `match` fields and class/phase/path narrowing are **code** | A choice between two fixed options. Different question and threshold from default matching (§6.1). |
| Match threshold, escalation threshold, N | **code** | Numbers in `athena.toml`, set from calibration (§10). |
| Earlier cases for an event, and the validity of every cited id | **code** | §3.3. |
| Whether the id is unique and well formed | **code** | Code derives a kebab-case id from the title and refuses any id already in the model or a pending proposal; `DefaultsModel::parse` also rejects duplicates **(read: `DefaultsError::Duplicate`)**. |
| Version bump | **code** | One integer `version` **(read: `DefaultsModel`)**; a proposal bumps it by one against the base commit's value. |
| Validity of the resulting file | **code** | Code builds the full candidate text and calls `DefaultsModel::parse`: unknown fields, duplicate ids, empty statement or source all fail **(read: `crates/athena-data/src/defaults.rs`)**. |
| Writing any record | **code** | Model output is parsed against a typed schema; anything that does not parse is dropped and logged, never repaired. |

Jev's input is built only from stored fields: `what`, `why`, `class_of_work`, `category`, the text of a reading
correction, and, for introspections, `failure_class` and `cause`. For the two questions of decision-sets §5 the list
also holds: for set membership, the reading's `class_of_work`, `deliverables` paths and `summary` (the field that
decision-sets §5 names for the "request summary"); for signal match, a host-error record's `status_code`,
`operation`, `phase` and `message_class`, and a check record's check name, result and job id. **No standard-error or
log text from any job is sent**, and nothing from another job's files. It never sees outcomes or scores (the rule D31
set for Jev's forecasts, which keeps every match computable again from records alone) **(read: D31)**.

## 6. How Jev is reached: `POST /v1/evaluate`

All of this section is **(reported)** from the user's probes of the gateway and the review; none of it was
reproduced here, and the building job checks each point against a live call before the client is written.

**Route.** `POST https://ai-gateway.vercel.sh/v1/evaluate`. The general chat route (`/v1/chat/completions`)
refuses Jev. A Jev client that meets this refusal treats it as a **build error** (the client was pointed at the
wrong route), never as a transient fault to retry.

**Request fields.**

| Field | Content |
|---|---|
| `model` | `typesafe-ai/jev` |
| `state` | The text the question is about: the record's stored fields |
| `questions` | A map from a key to one question: `type` (`boolean` or `choice`), `instructions`, and `criteria`, a map from each option to its description |

**Pre-build requirement: a multi-question call.** The reported response is for one question only. Decision-sets §5
asks one `boolean` per decision or per signal in one call, so before the client is written the building job probes a
call with several questions in `questions` and records here the response shape (one answer per key, its
probabilities and confidence, and whether the call is priced per question). If it does not answer several
questions, the fallback is one call per question. This job made no call to the gateway.

**Response fields.** The chosen option; a probability per option; a confidence. One probe returned `none` at
0.06, a top option at 0.90 and a confidence of 0.87 with 409 input and 59 output tokens, for five short options.
The probe offered a `changelog` option that is not in `defaults.toml`, so it shows that Jev can pick a supplied
label and nothing about discovery **(reported; read: review 1, §4)**.

**Cost and tokens** are read from the response and stored; they are never estimated by the client. (The cost
figure in the probe report was garbled, so no per-call cost is taken from it here.)

**Fake Jev.** The test double in the build order (§12) follows this shape exactly: the same route, the same
request fields, the same response fields, and a refusal on the chat route that produces the build error. A
fake that returns anything else would let a wrong client pass.

### 6.1 The two signals

- **Top probability** `p_top`: the probability of the chosen option. It controls **matching**: at or above
  `t_match`, the record is matched to that option (a default, a class, or a cluster); below it, to the unmatched
  pool or to "new". This reading is for `choice` questions.
- **Boolean questions (set membership, signal match).** For a `boolean`, `p_top` is not the threshold input: a
  chosen "false" at 0.9 has a high `p_top` and means no match. Code reads the probability of the `true` option
  (apply, or match) and compares it with `t_apply` or `t_signal`, numbers set by calibration (§10.3) and separate from
  `t_match` (decision-sets §5).
- **Confidence** `c`: the number the gateway returns beside the probabilities. It controls **escalation** only:
  below `t_conf`, the question is asked again of a stronger model (§6.2). The probe returned 0.90 and 0.87, so
  they differ; what `c` measures is not known, so `t_conf` is set from calibration (§10), not guessed.
- Both are stored with every answer.

### 6.2 Escalation: an explicit second call (design)

The gateway can run a fallback model below a confidence threshold (`providerOptions.gateway.models` with
`when.confidenceBelow`, **reported**). Two options:

| Option | For | Against |
|---|---|---|
| Gateway fallback | Small client; one call | It runs only below the threshold, so the shadow comparison on everything still needs a second call. Whether the response says the fallback ran and which model answered was not reported, so the stored answer might be mislabelled as Jev's. |
| **Explicit second call by Athena (chosen)** | One mechanism serves escalation and shadow comparison; the answering model is known and stored | A second code path and a second client |

Chosen: the explicit second call. Every answer stored in `evidence` names the model that gave it, so a stronger
model's answer is never recorded as Jev's. If the building job finds that the response labels the fallback
model reliably, this choice may be revisited; the cost is a client change.

## 7. Matching, clustering and candidates

### 7.1 What is counted

The **intervention record** (`athena.intervention/1`) is the single counted unit. A correction record carries
its intervention **(read: `docs/records.md`)**, so counting both would count a correction twice; the correction's
text is read as part of its intervention and is not a separate input. An introspection links to its event's
intervention and is read as an extension of it. An intervention of category `answered-question` is an answer to
the reader's question, not a repeated expectation; it stays in the labeled set as its own stratum and does not
count toward N **(design)**. Events with no intervention record (a failure, a rejection, a host error, a check
failure) get one when the introspection is recorded, so the unit is uniform, with `source = observed` (a value
`docs/records.md` line 70 lists). **Only user-originated units count toward N.** Observed interventions made for host
errors and check failures do not: a system fault would otherwise count as the user repeating an expectation.

**One evidence rule across both designs.** One event is enough to put a decision under review (decision-sets §4);
N = 3 distinct jobs, user-originated, is needed before a default is added (§7.2). Review costs a short mark in a
brief and is cleared by the user; adding a default changes every later brief.

### 7.2 Matching and clustering

For each new unit, Jev gets its stored fields and the rendered defaults (id, scope, first sentence) and returns
a probability for each id and for "none". Code applies `t_match`. A matched record counts against an existing
default; a default matched often is one the briefs are not getting across, and `athena status` lists the counts.

Unmatched records, and records matched repeatedly to one default, are assigned to clusters by Jev (assignment or
pairwise, §5). Code counts distinct jobs, not records. A cluster of at least N jobs becomes a candidate. **N = 3
(estimate)**, a design choice, movable by calibration.

A candidate is **add** (no good match; a new id) or **amend** (a moderate match to an existing default; the
draft widens or sharpens it and keeps the id). Introspections whose `prevention_kind` is `code-check` or
`design-change` do not become candidates; they are listed (§9).

### 7.3 Drafting

A language model drafts the statement in the voice of the existing entries (one or two sentences), `applies_to`
from the repository's classes of work, and a short title. Code checks `applies_to` against the classes in the
cluster's records plus `*`, and refuses a statement that is empty or over the length cap (§8.3).

## 8. The proposal format

A proposal is a record, not an edit to `defaults.toml` (`docs/records.md`). Name: `athena.defaults-proposal/1`
**(design)**.

### 8.1 Fields

| Field | Meaning |
|---|---|
| `id` | The default's id: new for `add`, existing for `amend`. Unique among the model and pending proposals (code-checked). |
| `kind` | `add` or `amend`. |
| `statement` | The full statement text that would appear in `defaults.toml`. For `amend`, the whole new statement; the prior one is shown beside it. |
| `applies_to` | The scope list; `*` means every class. |
| `version` | `{ from, to }`: the base commit's model version and `from + 1`. |
| `sources` | The intervention ids of the cluster (at least N distinct jobs), each with its job id, and the introspection ids that reached the same `cause`. These become the `source` text of the entry when accepted. |
| `assumptions` | Plain sentences: what must be true for the default to be right. Drafted by the language model, reviewed by the user (decision-sets §2, §5). |
| `signals` | Typed watched conditions in the form of decision-sets §4 (`source`, `match`, optional `text`, `threshold`), or an explicit "none" with the reason. |
| `evidence` | For each source: the probabilities and confidence of each Jev answer, the thresholds used, and **the model that gave each answer** (Jev, the stronger model, or a drafting model). |
| `status` | `pending`, `accepted` (with the job that applied it), `rejected` (with the user's reason), or `withdrawn`. |
| `candidate_sha256` | Hash of the full candidate `defaults.toml` that passed `DefaultsModel::parse`. |

**Where the two new fields live once accepted (F2).** `defaults.toml` cannot hold them (`DefaultsModel::parse`
rejects unknown fields; **read** `defaults.rs` lines 7, 16). On acceptance, code writes them to an
`athena.default-expectations/1` record keyed by the default's `id` (decision-sets §2). The `defaults.toml` entry is
unchanged in shape. The alternative, new optional keys in `defaults.toml`, is a governance job and is not chosen now.

### 8.2 Example

Illustrative draft for R4; not accepted, not in `defaults.toml`, source ids are placeholders because the real
records were not readable:

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

The example's version follows the base file: `defaults.toml` has `version = 1` **(read)**. The version counts
accepted changes, not entries. If several proposals are accepted together they land in one job and bump the
version once; `to` is recomputed by code at acceptance, never trusted from the proposal.

### 8.3 The reader budget

The earlier draft capped a statement's length by the reader budget. That protected the wrong thing: the reader
loads `DefaultsModel::render()`, which carries one line per default with the **id, the scope and the first
sentence** of the statement **(read: `defaults.rs`, `render`)**. So:

- **For the reader, the cost of an entry is its id, scope and first sentence, and the entry count.** The rest of
  the statement costs the reader nothing. The proposal states its rendered size (**estimate:** ⌈bytes/3.0⌉, the
  tripwire estimate in D25) and the **test** that composes the reader set fails if it would go over 1,800 tokens.
  That is a test, not a job-time refusal (§2, R2). The reader set is 2 tokens under budget today (measured, review
  1), so a proposal that adds an entry's line needs headroom made first.
- **The full statement lands in the producer and verifier briefs**, through `get()`, for the defaults the
  reader selected **(read: `defaults.rs`)**, in the per-job section "User defaults that apply" **(read:
  `crates/athena-context/src/briefs.rs` line 158)**. **Defaults do not count against the producer and verifier
  budgets of 1,100 and 1,300 today:** the budget test composes only the contract, the frame and the capability
  section **(read: `budgets.rs` lines 35-44, `lib.rs` lines 158-159)**. A statement-length cap stays and protects
  the per-job section, which decision-sets §3 puts under a cap, `brief_set_tokens`, enforced at run time by a new
  check with its own test.
- **Gate before relying on any figure.** Before any entry is added, a test calls `render()` alone and measures the
  result with the live counter (D25: the live measurement is the authority). The earlier 1,500-token estimate is
  withdrawn (§5). The build does not proceed on an estimate.

## 9. How a proposal reaches the user, and why Athena never edits `defaults.toml`

`defaults.toml` is a governance file: changing it is a job that lists it as a deliverable, and each change cites
the evidence that pulled it in (D26; header of `defaults.toml`) **(read)**. This design keeps that rule whole.

1. **Generation.** After a job closes, or on a command, code runs the loop (§4) over records not yet seen.
   Output is records and proposals, nothing else. No path in the loop opens `defaults.toml` for writing; a test
   when built asserts the loop's code has no such call **(design)**.
2. **Surfacing.** `athena status` lists pending proposals: id, kind, statement, source count and version bump,
   with source records and probabilities. It also lists, apart, introspections whose prevention is a code check
   or design change, with their classes and counts, as proposed work for the user to request. A proposal is
   never injected into a brief: briefs see only the accepted model at the base commit (D26).
3. **Decision.** To accept, the user asks for a job in the ordinary way ("apply defaults proposal X"). That
   reading lists `defaults.toml` as a deliverable, the producer makes the edit, and the verifier checks: id and
   statement equal the proposal, the version is the base plus one, and `DefaultsModel::parse` accepts the file.
   To reject, the user records `rejected` with a reason (itself an event that triggers an introspection, §3.1);
   code does not propose the same cluster again unless it grows by N more jobs.
4. **Landing.** It lands like any job. The accepted entry's `source` cites the proposal's records.

Reasons: a wrong default quietly shapes every future brief; the model is the user's, not Athena's (D89 says so
in terms); and a loop that edits its own rulebook cannot be told apart from drift.

## 10. Calibration before trust

Jev is not part of v0 (D31) **(read)**. D31 left it out for an HTTPS client, a credential, a driver and its fake,
and refusal handling, and because its forecasts could be computed retroactively. Its path back was "the first
calibration review, or the first typed decision in the flow" (`docs/design/athena-v0.md` §14) **(read)**. This
design is that typed decision. D90 narrows D31: Jev may be added, in shadow only, for the typed decisions here
(class assignment, default matching, cluster assignment, `prevention_kind`), and nothing runs live until the
promotion criteria hold. `docs/design/decision-sets.md` (D91) widens the narrowing to two more typed questions,
set membership and signal match, which are covered by §10.2 to §10.4 below. Introspection prose, which a language model writes, is not gated on Jev; it is
reviewed as part of the introspection record and scored in §10.3.

### 10.1 Shadow mode

- Jev runs on every new unit. Its answers are recorded. They produce no proposal the user sees as live and
  change no count in `athena status` that the user relies on.
- A stronger model answers the same typed questions on the same inputs (`BRIEF.md` line 217) **(read)**.
- Shadow runs have a spend cap, like every invocation (D28) **(read)**.

### 10.2 The labeled set

- **Source:** all intervention and correction records now in `records/`, counted once each (§7.1), plus
  reading corrections. Labels follow D31's rule: Jev's input is built only from stored fields. The count is not
  known here **(unknown; the building job counts it and states it)**.
- **Labels:** for each unit, the user, or a stronger model whose labels the user spot-checks, names the default
  it matches or "none", the failure class, and groups repeats. Who labeled which is stored.
- **Must include** R1 to R4 and the cases in §3.6, with R2 labelled "none", and for each, records that look
  similar but are not repeats.
- **Set membership (J6; design):** a second labeled set, one row per (reading, decision in the code-narrowed
  candidate set): the user, or a stronger model spot-checked by the user, labels `apply` or `not apply` from
  past readings (class, deliverables, `summary`). Labels are written before Jev's output is read. Rows where the
  decision is a `requirement` are labeled too but are scored for information only, since requirements are selected
  by code alone.
- **Signal match (J6; design):** a third labeled set, one row per (record, signal): `match` or `no match` over
  stored intervention, correction, host-error and check records, including near-misses (same class, other
  category; same operation, other status). The first rows are the D87 cases of decision-sets §4 and §6 step 5;
  where `records/` has too few, the set says so.
- **Split:** by time. Records up to a cut date calibrate the thresholds and N; later records test. No tuning on
  the test part. The same split applies to the two sets above, and they calibrate `t_apply` and `t_signal`.
- **Label hygiene:** labels are written before Jev's output is read.

### 10.3 Scoring

- **Matching:** the Brier score of Jev's probability vector against the label, with two references: always
  predicting "none", and a plain word-overlap matcher. As in athena's forecasters, where "the reference is what
  makes a vague forecaster show zero skill" (`docs/design/athena-v0.md`, near line 1208) **(read)**. Skill is the
  Brier improvement over the better reference.
- **Top-1 accuracy** on matched-or-not, and the false-match rate on "none" labels, because a false match hides
  a gap.
- **Class and cluster assignment:** whether each labeled group lands in one class or cluster, and whether any
  mixes groups; the effect of arrival order.
- **Set membership and signal match:** the Brier score of the probability of the `true` option against the label,
  with two references (always `false`, and the code-only matcher: class, phase and path narrowing for membership,
  exact `match` fields for signals); skill is the improvement over the better reference. Also the **miss rate**
  (a label `apply` or `match` that Jev scored below the threshold), which is the costly error here because it hides a
  decision or a failure, and the false-apply and false-match rates. A `requirement` dropped by Jev is counted as a
  critical failure and must be zero.
- **Confidence:** whether `c` predicts error better than `p_top` does, which fixes `t_conf` (§6.1).
- **Prose:** a person (or the stronger model, spot-checked) rates each introspection's `cause` and each draft:
  pass if it says something beyond the instance (introspection) or says what the sources say (draft).
- **Cost and latency** per decision, read from responses (§6) **(read: `BRIEF.md` line 217)**.

### 10.4 Promotion criteria

Jev's proposals become visible in `athena status` as live (still review-only) when all hold. Numbers are
starting points to be set at review and are **estimates** unless computed.

1. **Matching skill is above zero on the test part, with the test part sized for it.** A lower bound on mean
   skill needs the test size *n* to satisfy *n* > (1.96·sd / mean)², where mean and sd are the per-record Brier
   improvement over the better reference. Assuming a mean improvement of 0.10 and sd of 0.30 (**both
   estimates**), *n* > (1.96·0.3/0.1)² ≈ 34.6, so **35 test records**; at sd 0.5 it is ≈ 96. With a time split
   that keeps about a third for testing (estimate), the labeled set would need about 100 records or more. Only
   four repeats are named today, so unless the building job finds a labeled set of that size, **the interval
   is dropped**: the user judges the skill from the raw Brier scores and the per-record table, and the report
   says that no bound was computed. If the test part reaches the computed size, the interval is used.
2. R1 to R4 and the cases of §3.6 each land in one class or cluster in shadow, with none mixed (R2 as "none").
   Records not found are scored on what exists, and the result says so.
3. The false-match rate on "none" labels is under **10% (estimate)**.
4. No proposal in shadow cites a record id that does not exist (a code check, so zero), and every draft passes
   `DefaultsModel::parse` in the candidate file.
5. At least **90% (estimate)** of drafts and introspection causes are rated faithful and beyond the instance.
6. **Set membership and signal match are promoted separately from default matching,** each when: skill is above
   zero on its own test part (the interval rule of item 1 applies, with the same fall-back to raw scores); the miss
   rate is under **5% (estimate)** for signal match and **no `requirement` is ever dropped** for membership; the
   false-apply and false-match rates are under **10% (estimate)**; and the multi-question call of §6 has been probed.
   Until then membership and the watcher run by code alone (decision-sets §5).
7. The user reviews the shadow report and agrees.

**Demotion:** if agreement with the stronger model drops, or the user rejects proposals for the reason "Jev
misread the records" twice in a row, Jev returns to shadow (counts are estimates).

Until promotion, the live parts are the deterministic path from a person-written proposal request to the same
record format (so the review path, §9, is built first) and introspection records written by a language model
with a person-visible review.

## 11. Alternatives considered

1. **A person notices repeats and writes the change (status quo).** No code. It is the loop that missed R1
   four times before D89 repaired it in code (reported count; D89 read).
2. **Rules instead of a model: keyword and category counts.** Cheap, but fails on paraphrase; kept as the
   reference in §10.3.
3. **A stronger model for everything, no Jev.** Likely the most accurate matcher; it costs more per record
   (**estimate:** one to two orders of magnitude over E12's per-call cost, not measured), and the brief asks for
   Jev on typed decisions.
4. **Jev decides everything, including clustering and drafting.** Rejected: an evaluate call can only choose
   among options or answer yes/no (review 1, F1). It cannot draft prose.
5. **Jev proposes, Athena applies accepted proposals itself.** Rejected: Athena would edit the user's model (§9).
6. **Auto-land low-risk defaults.** Out of scope, rejected for the same reason.
7. **A separate Athena-owned defaults file.** Splits the model in two; the version would have no single
   meaning. Rejected.
8. **Count records, not jobs, for N.** Rejected: one job with four corrections is one episode.
9. **Skip shadow mode.** Rejected: a wrong accepted default costs every later brief.
10. **Dashboard first (RM-12).** Later.
11. **Records reader for introspection: a filtered view in the agent snapshot.** Rejected for Athena code outside
    it (§3.3): a view breaks the "records stay out" rule, and the agent could cite what code cannot verify.
12. **Introspection only when the user asks.** Rejected: the user's correction is the costly moment, and asking
    makes the user do the work the step exists to remove. The cost of always running it is bounded by the spend
    cap and by the rule that it never blocks a job.
13. **Gateway fallback for escalation.** Weighed in §6.2; the explicit second call is chosen.
14. **Introspection as free prose with no class.** Rejected: without a class there are no earlier cases, and the
    defaults loop has nothing to count.

## 12. Build order, if accepted

0. **Gate (§2):** read `records/`, list record ids for R1 to R4 and the §3.6 cases, and stop if fewer than N
   distinct jobs show a repeat a default would cover. Introspection (steps 1 and 2) goes ahead on its own value.
1. Add the introspection record, the class registry, the records-reading code (§3.3) and a person-visible
   review of the output, with a language model for the prose and no Jev.
   **Built (step 1).** `crates/athena-core/src/introspect.rs` holds the reading code and the step itself; the
   record and the registry entry are `athena.introspection/1` and `athena.failure-class/1` in
   `crates/athena-data/src/records.rs`. The triggers built are rows 1 to 4 of §3.1 (a correction, any other
   intervention, a rejected job, a failed job); host errors and landings, and a rejected proposal or plan, are
   not. Without Jev the model names the class (a registry id, or `new` with a name) and the prevention kind, and
   the record says so (`from_jev` is false); code assigns the id of a new class and adds it as `proposed`. The
   reading code takes the shared reader, the typed headers, the evidence-after-decision rule and the citation
   validation from the backtest, which now re-exports them. The review is the introspection block of
   `athena status <job>` and its `--json`. Tests use fixture records in a temporary directory. **The run over the
   real `records/` follows landing**: the agents' copies leave `records/` out, so it cannot be done before.
2. Add retry-from-record (§3.5).
   **Built (step 2).** `athena retry <job>` asks "try again on" the job and shows the reading as `ask` does;
   the reader's brief gains the job's cause, prevention, proposed next request and the earlier cases stored on its
   introspection, with each earlier prevention shown as already tried. A job without a usable introspection falls
   back to plain "try again on". Nothing is submitted without the user's acceptance.
3. Add the proposal record and validation, `athena status` surfacing and the apply-as-a-job check (§9).
   **Built (step 3).** `athena.defaults-proposal/1` is in `crates/athena-data/src/records.rs`; generation,
   validation and the apply check are `crates/athena-core/src/proposals.rs`. `athena propose` groups recorded
   introspections by `failure_class` and, at `REPEAT_THRESHOLD` (3) distinct jobs, writes a pending proposal
   whose statement code drafts from the class entry and the cases' `cause` (no model, no Jev; `assumptions`,
   `signals` and `evidence` are not fields yet and the record's `notes` say so). Each proposal is validated by
   building the candidate `defaults.toml` in memory and parsing it with `DefaultsModel::parse`, a unique id, the
   version base plus one, and the evidence-after-decision rule over its sources. `athena status` lists open
   proposals with their sources. `athena accept-proposal` submits the ordinary job of §9 item 3; `athena
   check-proposal` is its verifier check; `athena reject-proposal` records the reason and suppresses the class
   until `REPEAT_THRESHOLD` newer distinct jobs arrive. A test asserts the module's code never opens a file for
   writing. Not built: the reject-triggered introspection, cases of kind `code-check` or `design-change` listed
   apart, and the check running by itself inside the apply job.
4. Measure `render()` alone with the live counter (§8.3). Defaults are known not to count against the producer and
   verifier budgets (§8.3); the per-job runtime set-size check is built under decision-sets §8.
5. Probe a multi-question `evaluate` call (§6), write the fake Jev to the shape found, then the Jev client; check
   each reported fact in §6 against a live call.
6. Build the labeled set and scoring (§10.2, §10.3) with the fake, then run in shadow; review; promote under
   §10.4.

## 13. What is left open

- The real record ids and counts for R1 to R4 and the §3.6 cases (the gate, §2).
- Where the loop is triggered: after each job close, or on a command. A command first, close trigger after
  promotion. Introspection is triggered by the event itself (§3.1).
- Whether an `amend` may shrink a statement. Starting rule: no.
- Whether the draft model should see accepted and rejected proposals as examples; it changes what shadow must
  hold out.
- Whether classes of work (`applies_to` values) need a registry so code can check them exactly.
- What the gateway's confidence measures, and whether a fallback answer is labelled (§6).
- How a rejection is recorded when it is not a proposal or a rejected outcome (a plan): a new record kind, not
  designed here. A rejected outcome is already recorded (`OutcomeState::Rejected`, §3.1).
- Whether the class registry should be merged by Jev pairwise questions; not decided.
- The class vocabulary is closed in decision-sets §2 to the values `defaults.toml` uses; a registry that code can
  check exactly remains open, as the item above says.
- The correction to `docs/decisions.md` D90 ("Alternatives ... section 9" should read §11 of this document) is made
  by a dated note on the entry, not a rewrite of it.
- A review of this revision is still to be done; review 1 of the pair is `docs/design/reviews/decision-sets-review-1.md`.
