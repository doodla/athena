# Decision sets: typed decisions, applied by scope, and put under review when their signals fire

**Status:** proposed (D91). Design only: no Rust, no schema file, no change to `defaults.toml` or
`athena.toml`. A review comes before any build (the `designed` default). Paths are relative to the repository root.
The reader knows `defaults.toml` (D26) and `docs/decisions.md` but has not seen the jobs behind this.

Claims are marked **(read)** when taken from a file named beside them, **(design)** for proposals,
**(reported)** when taken from the job request and not checked, **(estimate)** for unmeasured numbers.

## 1. The problem

The addendum asks how to "classify, organize, and apply decision sets", and for "indications that the current thing
isn't working. Can't confidently be wrong" **(read: `BRIEF.md`, addendum "2026-10-13: Decision sets, and knowing
when a decision is not working")**. Today decisions live in four places, and only one of them reaches a brief:

| Place | Shape | Reaches a brief? |
|---|---|---|
| `BRIEF.md` addenda | Free prose bullets, dated | No; read by people and by the reader's prompt |
| `defaults.toml` | 12 entries: `id`, `statement`, `applies_to`, `source` | **Yes**: the reader names entries, `get()` puts the chosen ones in the producer and verifier briefs **(read: `defaults.toml`; `docs/design/defaults-loop.md` §8.3)** |
| `docs/decisions.md` | D01-D90, one flat log, prose fields (Decision, Alternatives, Tradeoff, Reversal cost, Provenance) | No |
| Corrections inside job records | Intervention and correction records | No; read by the introspection step only (defaults-loop §3) |

Two gaps follow. (1) A decision cannot be found by what it governs: D89's rule about `CHANGELOG.md` and the
`changelog` default say the same thing in two places and neither points to the other. Supersession lives in prose:
D87 says "Superseded by D88" in a bullet, and the log's own header says entries are never rewritten, so a
reader must read every later entry to know what stands **(read: `docs/decisions.md` header, D87, D88)**.
(2) A decision never states what it assumed or how to tell it failed. D87 chose merge commits for pull requests;
the repository allows only squash merges, and the host refused the merge with HTTP 405 within the hour
**(reported)**; D88 records the repository's flags (`allow_merge_commit` false) and a hand rebuild of PR #40 as the
cost **(read: D88)**. The decision stayed applied until a person noticed.

## 2. The typed decision

Every decision gets these fields **(design)**. The first five are the request's; the last four are new.

| Field | Values | Notes |
|---|---|---|
| `id` | `D91`, or a default's kebab id (`small`) | Existing ids are kept; no renumbering |
| `kind` | `requirement`, `expectation`, `design`, `process` | Requirement: the user stated it as a must (BRIEF.md). Expectation: Athena's model of what the user wants unsaid (today's defaults). Design: a choice about how Athena is built. Process: how work is done or landed |
| `scope` | `classes`, `paths`, `phase` | `classes`: classes of work or `*` (same vocabulary as `applies_to`). `paths`: globs, optional (e.g. `defaults.toml`, `crates/athena-core/src/pr.rs`). `phase`: one or more of `reading`, `producing`, `verifying`, `landing` |
| `status` | `active`, `proposed`, `superseded` | `superseded` carries `by = "D88"`, a required, existing id. Also `amended_by` (list) for partial changes |
| `source` | Free text, as now | A BRIEF.md section, a request, a review finding |
| `enforcement` | `code-check`, `brief`, `prose-only` | `code-check` names the test or check; `brief` means only the text of a brief carries it; `prose-only` is declared, with a reason (the `mechanism` default) |
| `statement` | One to two sentences; first sentence stands alone | The first sentence is what the reader sees (defaults-loop §8.3) |
| `assumptions` | List of plain sentences | What must be true for the decision to be right (§4) |
| `signals` | List of watched conditions | Observable signs of failure (§4) |
| `reviewed` | Date, outcome | Last review; empty until the first |

`statement` and the log's prose fields stay in `docs/decisions.md`; the typed fields are a header on each entry
(§6). A default is a decision with `kind = expectation`, `phase` set from its `applies_to`, and `source` as today.
It stays in `defaults.toml`, which remains a governance file (D26); only the typed view over it is new.

**Why these four kinds and not more.** They differ in how they are treated: requirements are changed only by the
user; expectations are proposed by the defaults loop; design and process decisions are changed by a job with a
review. A fifth kind would need its own rule.

## 3. Sets, and how a set reaches a brief

A **set** is the group of decisions that share a scope. Sets are computed, not stored **(design)**: a set is a query
`(class, phase, paths)`, so there is no second list to keep in step.

**Applying.** When a job is read, Athena builds the sets for each role from the reading's class of work and
deliverable paths **(design)**:

1. Code selects decisions with `status = active` whose `classes` contain the job's class or `*`, whose `paths`
   (if any) match a deliverable, and whose `phase` contains the role's phase (reader: `reading`; producer:
   `producing`; verifier: `verifying`; landing checks: `landing`). `proposed` and `superseded` decisions are never
   selected: a proposal cannot reach a brief (defaults-loop §9; D26).
2. For defaults, the existing path stays: the reader names the entries that apply and `get()` renders them
   **(read: defaults-loop §8.3)**. Set membership for the other kinds uses the same call (§5).
3. Decisions whose `enforcement` is `code-check` are **not** put in a brief: the check enforces them, and brief
   text would cost tokens to say what code already does. This is the lesson of R2 in defaults-loop §2, where a
   rule the code enforces is not worth reader tokens **(read)**. They appear in the brief only as a one-line
   pointer if they have an open review (§4).

**Token budget.** The budgets are in code: contract 350, reader 1,800, producer 1,100, verifier 1,300 tokens
**(read: `crates/athena-context/src/lib.rs` lines 156-159)**, and an over-budget set is refused (`check_budget`).
The set must fit, so selection is ordered and truncating **(design)**:

- A set is rendered one line per decision: id, kind, and the first sentence of the statement. The full statement is
  added only for the decisions the reader names, as for defaults today.
- Order: decisions under review first, then `requirement`, then `expectation`, then `process`, then `design`;
  ties by narrower scope. Lines are added until the live counter (D25) would exceed the role's budget; the
  remainder is counted and the brief says "N more decisions apply; not shown". The cut is recorded in the job's
  records, so a decision left out is visible, not silent.
- The reader sees the one-line rendering of the whole set; it cannot load full text for every decision. Whether
  defaults already count against the producer's and verifier's budgets was not read here; the building job reads it
  first (defaults-loop §8.3 leaves the same question open).
- The rendered size is measured with the live counter in a test before relying on any estimate (D25, defaults-loop
  §8.3).

## 4. Assumptions and failure signals

**Each decision states its assumptions and its signals, or says it has none.** A decision with an empty
`signals` list must say why (a pure preference with no observable consequence); `athena status` lists those, so
unwatchable decisions are visible. This is `prose-only` applied to the watching itself.

A **signal** is a typed, observable condition over records Athena already writes **(design)**:

| Field | Meaning |
|---|---|
| `source` | One of `rejection`, `intervention`, `correction`, `check-failure`, `host-error` |
| `match` | Narrowing by code: record kind, `class_of_work`, `category`, check name, host error status, job phase |
| `text` | Optional one-line description, for a typed question to Jev (§5) when code cannot match exactly |
| `threshold` | Distinct jobs and window, e.g. 1 job within 7 days for a host refusal; code decides |

The five sources are the ones in the request and map to records that exist or are named in defaults-loop §3.1:
interventions and corrections (`docs/records.md`), rejected outcomes, failed checks and job outcomes, and host
errors from the GitHub client **(read: defaults-loop §3.1)**. Rejections and host errors have no dedicated record
kind today; the building job adds them, as defaults-loop §3.1 already requires for rejections **(design)**.

**Watching.** After each job closes, and after each introspection (defaults-loop §3), code evaluates every active
decision's signals against records written since the last evaluation. Exact `match` fields are decided by code,
deterministically. Only a signal with `text` and no exact match goes to Jev. The evaluation reads stored fields only,
as D31 requires of Jev's inputs **(read: defaults-loop §5)**.

**Firing.** A signal that crosses its threshold sets the decision's state to `under-review` **(design)**, a
state beside `status`, with the firing records' ids. From then on:

- The decision is **no longer applied silently**. If it was `brief`-enforced, the next briefs carry it with the
  marker "under review: <signal>, <record ids>", so the producer and verifier see that it may be wrong; the
  one-line pointer applies to `code-check` decisions as well (§3).
- `athena status` lists it with the firing records.
- An introspection is recorded for the event, as for any failure (defaults-loop §3.1), and its `prevention_kind`
  can be `design-change` against the decision.
- Athena does not edit or supersede the decision itself. The user's options: **confirm** (signal was noise; add an
  assumption or raise the threshold), **amend** (a new decision with `amended_by`), or **supersede** (a new
  decision; the old one becomes `superseded` with `by`). Each is an ordinary job that edits `docs/decisions.md`
  and, for expectations, `defaults.toml` (D26). Governance stays with the user (defaults-loop §9).

**Why review and not auto-retire.** A signal is evidence, not proof: one rejection may be about the work, not the
decision. Auto-retiring would make Athena edit its own rulebook, which defaults-loop §9 rejects. Review costs a
short mark in a brief; wrong retirement costs more.

### Worked example: D87

D87 as it should have been written (design; the facts come from D87 and D88, read; the 405 is reported):

```
id: D87            kind: process
scope: classes = [landing]; paths = [crates/athena-core/src/gh.rs]; phase = [landing]
status: active     enforcement: code-check (tests in pr.rs and gh.rs)
statement: Pull requests land as merge commits.
assumptions:
  - The repository allows merge commits (GET repos/{owner}/{repo} reports allow_merge_commit true).
  - A user who merges by hand follows the recommendation.
signals:
  - host-error: merge request refused, HTTP 405 or 422, class landing; threshold 1 job
  - check-failure: athena check --history violation after a PR merge; threshold 1 job
  - intervention: category hand-change on a landed branch, class landing; threshold 2 jobs
```

The first assumption was false for this repository **(read: D88 states `allow_merge_commit` false)**. The
HTTP 405 matches signal 1 on its first occurrence, so the evaluation sets D87 to `under-review` after that job,
before PR #40 needs a hand rebuild **(reported: #40 was rebuilt by hand; whether the signal would have fired
earlier than D88 is a claim about this design, not a measurement)**. The user's choice then is the one actually
made: supersede with D88, "the merge method follows the repository" **(read)**. D88's own assumptions would be: the
host reports the three allow flags accurately; the squash text is accepted as given (D88 itself says this was not
observed **(read)**), with a signal on a history violation after a squash.

D87 shows the pattern the field is for: the assumption "the repository allows X" is a precondition that a
code check could read before acting. Where an assumption can be checked up front, the design change is a code
check (as in defaults-loop §3.6), and the signal is the backstop.

## 5. Fit with the defaults loop and Jev

**Introspection.** Defaults-loop §3 runs an introspection after each correction, intervention, rejection or
failure, and records `failure_class`, `cause` and `prevention_kind` **(read)**. Two links are added **(design)**:

- An introspection's record lists the decision ids whose signals the event matched; a decision's firing records
  include that introspection. Repeats of one `failure_class` against one decision are the evidence for review.
- When `prevention_kind` is `default`, the loop's candidate becomes a decision with `status = proposed`,
  `kind = expectation`, and its own assumptions and signals drafted by the language model. Defaults-loop §7-§9 is
  unchanged: the proposal is review-only and Athena never edits `defaults.toml`.

Defaults-loop's matching question is "does this record match default d" **(read: §4 step 1)**. Signal matching is
the same question with the decision's signal in place of the default's statement, so one mechanism serves both.

**Jev.** Jev is reached with `POST /v1/evaluate` and takes `model`, `state` and a `questions` map of `boolean` or
`choice` questions with `criteria`, and returns the chosen option, a probability per option and a confidence
**(reported: defaults-loop §6; not re-checked here)**. Jev is "a decision model, not a language model": it
chooses among supplied options and does not write **(read: defaults-loop §5)**. Two typed questions fit:

| Question | Type | `state` | Options |
|---|---|---|---|
| Which decisions in a candidate set apply to this reading? | `choice` per decision (or `boolean`) | The reading's class, paths, request summary | apply / not apply, with the decision's first sentence as criterion |
| Does this record match this signal? | `boolean` | The record's stored fields | match / no match |

Rules carried from defaults-loop: code applies thresholds to `p_top`; confidence only triggers a second call to a
stronger model; every stored answer names the model that gave it; and Jev stays in shadow until its promotion
criteria hold (§10 there; D90 narrows D31 for the loop only). **This design widens that narrowing to two more
typed questions, and D91 says so**; until promotion, set membership uses class, phase and path matching by code
only, and the signal watcher uses `match` fields only. Both work without Jev; Jev adds paraphrase tolerance.
Set membership by Jev must not drop a `requirement`; those are selected by code alone.

## 6. Migration

Nothing is migrated by this job. A later job does it in this order, each step reviewable alone **(design)**.

1. **`defaults.toml` (12 entries).** Add typed fields in a derived view, not by editing the file: `kind =
   expectation`, `status = active`, `phase` from `applies_to`, `enforcement` by reading each default (`rust`,
   `tested`, `public` are partly checked by code and gate; the rest `brief`). Whether to add `assumptions` and
   `signals` as optional TOML keys is decided at review, since `DefaultsModel::parse` rejects unknown fields
   **(read: defaults-loop §5)** and any change is a governance job (D26). `changelog` is an entry written by code
   (D89) and is migrated as `enforcement = code-check`.
2. **`docs/decisions.md` (D01-D90).** Add a typed header per entry; leave the prose as written. An entry is never
   rewritten after acceptance (log header), so the header is a separate block under the title. A script is not
   needed; a job reads each entry. Order by usefulness: first the entries that govern behavior
   Athena still applies (count unknown until the job runs), then the rest default to `kind = design`, `status = active`,
   `signals = none (not yet written)` and are listed in `athena status` as unwatched.
3. **Prose-only supersessions.** Parse the log for the phrases it uses: "Superseded by Dnn", "Supersedes Dnn",
   "(Superseded by Dnn: ...)", "Amends Dnn" **(read: grep of `docs/decisions.md`: D87 line 1901, D88 line 1922, D46 note
   line 1020, D79 line 1752, and the "Amends" entries from D48 on)**. Each becomes `superseded by` or `amended_by`
   on the older entry. Code lists every hit for the user to confirm; no hit is applied unconfirmed, because
   "Amends" may be partial (D87 is superseded for its landing shape only, and its two rerun fixes stand, so it
   is `amended_by = D88`, not `superseded`). An "Amends" with unclear extent is left `active` with `amended_by`.
4. **`BRIEF.md` addenda.** The brief is owner-only; it is not edited. Each addendum bullet that states a rule
   becomes a decision with `kind = requirement`, `source` set to the addendum heading, `status = active`. The
   decisions are stored in `docs/decisions.md` and the default's `source` fields already cite addenda
   (`root-cause`). Addenda that state a wish with no testable form (this one: "can't confidently be wrong")
   become one `requirement` whose enforcement is this design.
5. **Corrections in job records.** A correction that changed a decision's meaning ("D89 narrows D26's count") is
   found by the introspection step and linked as an `amended_by` or `signals` firing record, not rewritten. Past
   corrections are used as **a test set** for signals: for each written signal, check that the past records it
   should have caught actually match it (D87's 405 is the first case). Records are read by Athena code outside the
   agent snapshot (defaults-loop §3.3).
6. **Signals backfill.** For decisions with open questions (assumed but unmeasured: the `(reported)` entries),
   `assumptions` are written when a decision first has a signal fire, not for all 90 up front. This keeps the work
   proportional to evidence.

**Verification.** A check (when built) that every `superseded` has an existing `by`, no active decision is
superseded by an inactive one, and every `code-check` decision names a test that exists, as
`docs/enforcement.md` does for rules **(read: it lists rows with tests)**.

## 7. Alternatives considered

1. **A single flat log with a status column (status quo plus a field).** Cheapest, but nothing reaches a brief and
   no signals are watched.
2. **Stored set files (one TOML per set).** Duplicates scope in two places and drifts. Rejected for computed sets.
3. **Put every decision in `defaults.toml`.** Makes the user's model carry design choices and burns reader tokens
   (1,800 budget). Rejected.
4. **Auto-supersede on a firing signal.** Rejected: Athena would edit its own rulebook (defaults-loop §9).
5. **Jev for all membership.** Rejected: a requirement dropped by a wrong answer is worse than a token cost; Jev
   is in shadow only (D90).
6. **Signals as free prose watched by a language model.** Rejected: unmeasurable and costly; typed `match` fields
   are checked by code.
7. **Only record assumptions, no signals.** Rejected: assumptions with nothing watching them are what D87 had
   in effect.
8. **Rewrite D01-D90 now.** Rejected: log header forbids rewriting accepted entries; a header block avoids it.

## 8. Build order, if accepted

1. Read how defaults count against the producer and verifier budgets; measure rendered sets with the live counter.
2. Typed decision view and the supersession parse (§6, steps 2-3), with a user-confirmed list. No briefs change.
3. Set selection by code for the reader, producer, verifier and truncation record (§3).
4. Signal records for rejections and host errors; the watcher with exact matching; `under-review` marker in briefs.
5. Backtest the signals on past records (§6 step 5), D87 first.
6. Only then add Jev for the two questions (§5), in shadow.

## 9. What is left open

- The number of distinct jobs and window per signal (set from backtest, not guessed).
- Whether `under-review` clears automatically after a quiet period, or only by a user decision. Starting rule: only
  by a user decision.
- Whether set membership by Jev ever replaces code matching. Not before the §5 promotion criteria hold.
- Whether to give `amended_by` a typed extent (`landing shape`, `count`) so partial supersession is queryable.
- How many entries of D01-D90 are still applied; unknown until step 2 of §6 is run.
- No independent review of this design is recorded yet.
