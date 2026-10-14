# Decision sets: typed decisions, applied by scope, and put under review when their signals fire

**Status:** proposed (D91), revised after review 1 (`docs/design/reviews/decision-sets-review-1.md`, verdict accept
with changes; all eight changes of its section 6 are applied here and in `docs/design/defaults-loop.md`). Awaiting
the confirming look at the revision; no build starts before it. Design only: no Rust, no schema file, no change to
`defaults.toml` or `athena.toml`. A review comes before any build (the `designed` default). Paths are relative to the
repository root. The reader knows `defaults.toml` (D26) and `docs/decisions.md` but has not seen the jobs behind this.

**Revision summary.** F1 to F4 were settled first and the rest follows from them: (F1) the reader gets sets only
through code-side narrowing with a cap, and per-job set size gets a runtime check (§3); (F2) assumptions and signals
of a default live in a separate typed record keyed by the default's id (§2); (F3) host errors get a record with a
status field, and landing becomes a watcher trigger (§4); (F4) one evidence rule (§4) and a corrected D87 example.

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
| `docs/decisions.md` | D01-D91 (91 entries, **read**: counted `## D` headings; D91 is this design's own entry, so the log to migrate is D01-D90), one flat log, prose fields (Decision, Alternatives, Tradeoff, Reversal cost, Provenance) | No |
| Corrections inside job records | Intervention and correction records | No; read by the introspection step only (defaults-loop §3) |

Two gaps follow. (1) A decision cannot be found by what it governs: D89's rule about `CHANGELOG.md` and the code that
carries it (`ensure_changelog` in `crates/athena-core/src/reading.rs`, which also adds a `changelog` entry to the
reading's defaults when the reading is built) say the same thing in two places and neither points to the other
**(read: D89; `reading.rs` lines 93-115; `defaults.toml` has no `changelog` id)**. Supersession lives in prose:
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
| `scope` | `classes`, `paths`, `phase` | `classes`: a **closed** vocabulary, the values `defaults.toml` uses in `applies_to` today: `code`, `build`, `tests`, `docs`, `design`, `records`, `config` and `*` **(read: `defaults.toml`)**. The class of work in a record is a free string **(read: `crates/athena-data/src/records.rs` lines 386, 785)**, so code compares it to this list and a value outside it selects nothing; adding a class is a governance change (D26), tracked with the class registry that defaults-loop §13 lists as open. `landing` is **not** a class: it is a phase, and a landing decision sets `classes = [*]` and `phase = [landing]`, selected by phase and paths alone. `paths`: globs, optional (e.g. `defaults.toml`, `crates/athena-core/src/pr.rs`). `phase`: one or more of `reading`, `producing`, `verifying`, `landing` |
| `status` | `active`, `proposed`, `superseded` | `superseded` carries `by = "D88"`, a required, existing id. Also `amended_by` (list) for partial changes |
| `source` | Free text, as now | A BRIEF.md section, a request, a review finding |
| `enforcement` | `code-check`, `brief`, `prose-only` | `code-check` names the test or check, and it must be something that actually stops the thing; `brief` means only the text of a brief (or a recommendation in docs) carries it; `prose-only` is declared, with a reason (the `mechanism` default) |
| `statement` | One to two sentences; first sentence stands alone | The first sentence is what the reader sees (defaults-loop §8.3) |
| `assumptions` | List of plain sentences | What must be true for the decision to be right (§4) |
| `signals` | List of watched conditions | Observable signs of failure (§4) |
| `reviewed` | Date, outcome | Last review; empty until the first |

`statement` and the log's prose fields stay in `docs/decisions.md`; the typed fields are a header on each entry
(§6). A default is a decision with `kind = expectation`, `phase` set from its `applies_to`, and `source` as today.
It stays in `defaults.toml`, which remains a governance file (D26); only the typed view over it is new.

**Where a default's `assumptions` and `signals` live (F2; design).** `defaults.toml` cannot hold them:
`DefaultsModel::parse` rejects unknown fields **(read: `crates/athena-data/src/defaults.rs` lines 7, 16)**, and none
of the 12 entries has such keys. Decision: a **separate typed record, `athena.default-expectations/1`, keyed by the
default's id**. It holds `default_id`, `assumptions`, `signals`, the `defaults.toml` `version` it was written
against, and the proposal it came from. Code writes it when a defaults proposal is accepted and the accepting job
lands (defaults-loop §8.1 carries the same fields in the proposal), and the user can write one for an existing
default through an ordinary job. It is a record, so Athena code reads it outside the agent snapshot and briefs see
only what code renders from it. Typed decisions that are not defaults carry their fields in the header block of
their `docs/decisions.md` entry (§6 step 2). Alternatives: (a) new optional keys in `defaults.toml`, which needs a
governance job to change `DefaultsModel` and puts watched conditions in the file every reader loads; kept as the
later option if the record proves awkward. (b) Keep fields only in the proposal, rejected: they would be lost when
the proposal is accepted.

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
   **(read: defaults-loop §8.3)**. For the other kinds, membership is the code-side narrowing of step 1 and the
   cap of the token-budget text below; a reader that names decisions does so only among the K lines it was given.
   Jev's set-membership question (§5) stays in shadow.
3. Decisions whose `enforcement` is `code-check` are **not** put in a brief: the check enforces them, and brief
   text would cost tokens to say what code already does. This is the lesson of R2 in defaults-loop §2, where a
   rule the code enforces is not worth reader tokens **(read)**. They appear in the brief only as a one-line
   pointer if they have an open review (§4).

**Token budget (F1).** The budgets are in code: contract 350, reader 1,800, producer 1,100, verifier 1,300 tokens
**(read: `crates/athena-context/src/lib.rs` lines 156-159)**, estimated as ⌈bytes/3.0⌉. What those budgets do and do
not enforce, measured and read for review 1:

- **Measured (review 1):** the budgets test prints the reader set at 1,798 of 1,800 tokens, change producer 883 of
  1,100, change verifier 1,088 of 1,300, study producer 1,099 of 1,100, study verifier 1,238 of 1,300, contract 326
  of 350. The reader set has 2 tokens of headroom, and the log has 91 entries; at an assumed 25 tokens a line
  **(estimate)** one line each would be about 2,300 tokens. **The reader therefore does not see the whole set.**
- **Reader: narrowing and a cap, in code.** Code narrows the candidate set by class and phase (§3 step 1) before the
  reader is invoked. The reader is given at most **K lines** of that narrowed set (K is a number in `athena.toml`,
  starting at 0 until the reader set has headroom; the build measures what headroom exists, and a nonzero K needs a
  reader budget change or a smaller `render()` first). With K = 0 the reader sees no sets, and set selection for the
  producer and verifier is made by code alone (§5 describes the optional Jev step). The reader never loads full
  text for every decision.
- **Producer and verifier: defaults do not count against their budgets today.** The budget test composes only the
  contract, the role frame and the capability section **(read: `crates/athena-cli/tests/budgets.rs` lines 35-44)**;
  the defaults reach those roles in the per-job brief section "User defaults that apply" **(read:
  `crates/athena-context/src/briefs.rs` line 158)**, which no budget covers. The 1,100 and 1,300 budgets therefore do
  not protect the text sets would add.
- **The per-job brief section is budgeted by this design.** The section "User defaults that apply" and the new set
  section share one per-job cap, `brief_set_tokens` (in `athena.toml`; the value is set by the build from a
  measurement of the largest real section, not guessed here). It is measured with the same estimator and counted
  by the live counter where one exists (D25).
- **Refusal is a test today, not a job-time check.** `check_budget` is called only from tests **(read: grep of
  `crates/`: `lib.rs` tests and `budgets.rs`)**, and `docs/enforcement.md` row M18 describes "a per-set budget test
  with the byte estimator as a tripwire". An over-budget static set fails the build's tests; nothing refuses a job
  at run time. So this design does not say an over-budget set is "refused by code" at job time.
- **New build item: a runtime per-job set-size check, with its own test.** Selection is ordered and truncating
  **(design)**, and runs when the brief is composed:
  - A set is rendered one line per decision: id, kind, and the first sentence of the statement. The full statement
    is added only for the decisions the reader names, as for defaults today.
  - Order: decisions under review first, then `requirement`, then `expectation`, then `process`, then `design`;
    ties by narrower scope. Lines are added until the estimator would exceed `brief_set_tokens`; the remainder is
    counted and the brief says "N more decisions apply; not shown". The cut is recorded in the job's records, so
    a decision left out is visible, not silent. A `requirement` is never cut before a lower kind; if requirements
    alone exceed the cap, the job is stopped before the agent starts and the reason is recorded.
  - **Test (a build item, fails without the check):** a fixture set larger than the cap yields a brief whose set
    section is within the cap, shows the "N more" line, and writes the cut record; a fixture of requirements alone
    over the cap stops the job.
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
| `match` | Narrowing by code: record kind, `class_of_work`, `category`, check name, host error `status_code` and `operation`, job phase |
| `text` | Optional one-line description, for a typed question to Jev (§5) when code cannot match exactly |
| `threshold` | Distinct jobs and window, e.g. 1 job within 7 days for a host refusal; code decides |

The five sources are the ones in the request and map to records that exist or are added here:
interventions and corrections (`docs/records.md`), rejected outcomes, failed checks and job outcomes, and host
errors from the GitHub client **(read: defaults-loop §3.1)**.

- **Rejections.** A rejected outcome **is** recorded today: `OutcomeState::Rejected` **(read:
  `crates/athena-core/src/run.rs` lines 925 and 1042; `crates/athena-data/src/records.rs` line 96)**. The `rejection`
  source reads those. What has no record is a user rejection of a proposal or a plan (defaults-loop §13); the
  building job adds that kind, and until then the source covers rejected outcomes only.
- **Host errors (F3; design).** The client discards the status: `GhCli::gh` builds its error from the command's
  standard error text **(read: `crates/athena-core/src/gh.rs` lines 47-53)**, so "HTTP 405 or 422" would be a text
  match, not a field. The client therefore writes a record, `athena.host-error/1`, whenever a host call is
  refused: `status_code` (an integer, or absent when the call failed before an HTTP answer), `operation` (a closed
  list, e.g. `merge`, `create-pr`, `read-repo`), `job` (the job id, or the landing run's id when no job is open),
  `phase`, `at`, and a short `message_class` chosen by code from a closed list. The raw standard-error text is **not**
  stored in the record's matchable fields and is never passed to Jev. How the client gets the status (the `gh` exit
  text, or `gh api` with the response status) is a build detail the client's test pins; if the status cannot be had
  reliably, the record carries `status_code` absent and a signal on it matches `operation` only.
  The merge call is `job.host.merge(...)` **(read: `gh.rs` line 485)**, so the landing path writes this record.

**Watching.** Code evaluates every active decision's signals against records written since the last evaluation at
three points **(design)**: after each job closes, after each introspection (defaults-loop §3), and **at landing**,
because the merge refusal D87 met happens in `athena land --pr`, after the job is closed **(read: `gh.rs` line 485 is
in the landing path)**; a job-close trigger alone would never see it. Exact `match` fields are decided by code,
deterministically. Only a signal with `text` and no exact match goes to Jev. The evaluation reads stored fields only,
as D31 requires of Jev's inputs **(read: defaults-loop §5)**.

**One evidence rule (F4; design).** *A single event puts a decision under review; three distinct jobs are needed to
add a default.* The two numbers answer different costs. Review costs a short mark in a brief and is cleared only by
the user, so one event is enough to ask the question. Adding a default changes every later brief for everything the
default covers, so it needs the repeat evidence of defaults-loop §7.2 (N = 3 distinct jobs, an estimate). A signal's
own `threshold` may be higher than one job where one event is plausibly noise (the D87 hand-change signal uses 2),
but never below one event, and it does not change the rule for adding a default. Observed interventions made by
code (`source = observed`) for host errors and check failures do not count toward N (defaults-loop §7.1).

**Firing.** A signal that crosses its threshold sets the decision's state to `under-review` **(design)**, a
state beside `status`, with the firing records' ids. The state is **stored in a record, not in `docs/decisions.md`**
(F8): Athena never edits a decision (§4 "Firing"; defaults-loop §9), so an append-only
`athena.decision-state/1` record per change (decision id, state, firing record ids, date, and for clearing the user's
decision job) holds it. Code folds these records to the current state. A brief that shows the mark therefore has an
input that is not in the base commit, so it is recorded as a **brief component with a version and a hash, like the
defaults** **(read: `crates/athena-core/src/reading.rs` line 54 records the defaults as a component)**: the
fold's result (decision ids and states) is hashed and stored with the reading, and the brief can be rebuilt from
records. From then on:

- The decision is **no longer applied silently**. If it was `brief`-enforced, the next briefs carry it with the
  marker "under review: <signal>, <record ids>", so the producer and verifier see that it may be wrong; the
  one-line pointer applies to `code-check` decisions as well (§3).
- `athena status` lists it with the firing records.
- An introspection is recorded for the event, as for any failure (defaults-loop §3.1), and its `prevention_kind`
  can be `design-change` against the decision. **No recursion (F7):** the introspection recorded for a firing is
  **not itself watched**; the watcher skips records whose trigger is a firing. Defaults-loop §3.1 already stops
  the loop for failed introspections only, so this rule is the matching stop for firings. The same record is not
  evaluated twice, and a firing cannot cause a firing.
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
scope: classes = [*]; paths = [crates/athena-core/src/gh.rs]; phase = [landing]
status: active     enforcement: brief (a recommendation in docs; the merge request asking for `merge` is unchanged code)
statement: Pull requests land as merge commits.
assumptions:
  - The repository allows merge commits (GET repos/{owner}/{repo} reports allow_merge_commit true).
  - A user who merges by hand follows the recommendation.
signals:
  - host-error: operation merge, status_code 405 or 422, phase landing; threshold 1 job
  - check-failure: athena check --history violation after a PR merge; threshold 1 job
  - intervention: category did-work-themselves on a landed branch, class docs; threshold 2 jobs
```

**Why the enforcement is `brief`, not `code-check` (F4).** D87 states its landing rule as a recommendation in docs
("a user who merges by hand should pick a merge commit") and says the recommendation cannot stop a user from
squashing on the host **(read: `docs/decisions.md`, D87 from line 1899)**. The tests it cites are for its two rerun
fixes, which stand; they do not check the merge method. Under §3 step 3 a `code-check` decision is left out of
briefs, which would have hidden D87 from the roles that needed it. The first draft of this example said
`code-check`; that was wrong. (Class `landing` in the first draft was also wrong: landing is a phase, see §2.)

The first assumption was false for this repository **(read: D88 states `allow_merge_commit` false)**. The
HTTP 405 is recorded by the client as an `athena.host-error/1` record (`operation = merge`, `status_code = 405`) and
matches signal 1 on its first occurrence. The landing-time evaluation (§4 "Watching") sets D87 to `under-review`
right after the refused merge, before PR #40 needs a hand rebuild **(reported: #40 was rebuilt by hand; whether the signal would have fired
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

- An introspection's record lists the decision ids whose signals the event matched, in the field
  `decisions_matched` that this revision adds to `athena.introspection/1` (defaults-loop §3.2; written by code); a
  decision's firing records include that introspection. The evidence standard is the one rule of §4: one event
  puts a decision under review; a `failure_class` repeating against one decision strengthens the review (the user
  sees the count) but is not needed to start it.
- When `prevention_kind` is `default`, the loop's candidate becomes a decision with `status = proposed`,
  `kind = expectation`, and its own assumptions and signals drafted by the language model; the proposal carries them
  (defaults-loop §8.1) and, on acceptance, they are stored in `athena.default-expectations/1` (§2). Defaults-loop
  §7-§9 is otherwise unchanged: the proposal is review-only and Athena never edits `defaults.toml`.

**What is shared between default matching and signal matching, and what is not (F5).** Defaults-loop's matching
question is one `choice` over all default ids plus `none` **(read: defaults-loop §4 step 1, §7.2)**. Signal matching
is one `boolean` per signal. They are two different questions, not one mechanism. Shared: the gateway client and the
`POST /v1/evaluate` request and response shapes, the rule that `state` is built from stored fields only, the
recording of the answering model name with every answer, and the shadow and escalation handling. Not shared: the
question type (`choice` against `boolean`), the options, the threshold (below), the stored-field list (defaults-loop
§5) and the labeled set (defaults-loop §10).

**Jev.** Jev is reached with `POST /v1/evaluate` and takes `model`, `state` and a `questions` map of `boolean` or
`choice` questions with `criteria`, and returns the chosen option, a probability per option and a confidence
**(reported: defaults-loop §6; not re-checked here)**. Jev is "a decision model, not a language model": it
chooses among supplied options and does not write **(read: defaults-loop §5)**. Two typed questions fit:

| Question | Type | `state` | Options |
|---|---|---|---|
| Which decisions in a candidate set apply to this reading? | **One `boolean` question per decision** (a map of up to M questions in one call, M a cap in `athena.toml`) | The reading's `class_of_work`, its `deliverables` paths and its `summary` field, the one field named for "request summary" **(read: `Reading` in `crates/athena-data/src/records.rs` lines 375-399)** | `true` (apply) / `false` (does not apply), with the decision's first sentence as the criterion |
| Does this record match this signal? | `boolean`, one per signal | The record's stored fields (defaults-loop §5), including the host-error fields of §4 (`status_code`, `operation`, `phase`) but never stderr text from other jobs | `true` (match) / `false` (no match) |

**Question shape (J2).** Set membership is a choice between two fixed options, which is a `boolean`; a single
`choice` question with decision ids as options would fit about 91 options (roughly 7,000 input tokens by crude
scaling of the one reported probe, **estimate**, unmeasured). One `boolean` per decision is chosen. It needs a
multi-question call, whose response shape defaults-loop §6 does not report. **Pre-build requirement:** before the
client is written, a live call with several `boolean` questions in one `questions` map is probed and the response
shape (one entry per key, its probabilities and confidence) is recorded in defaults-loop §6. If the call does not
answer several questions, the fallback is one call per question, with its cost counted in the shadow budget. This
job did not call the gateway.

**Threshold for a boolean answer (J3).** For a `boolean`, `p_top` (probability of the chosen option) is not the
number to threshold: a chosen "no" at 0.9 has a high `p_top` and means no match. For both new questions code
thresholds **the probability of the `true` option** (apply, or match): `p_true ≥ t_apply` or `p_true ≥ t_signal`.
The thresholds are separate numbers set from calibration (defaults-loop §10.3), not `t_match`. `p_top` still
governs the one `choice` question of default matching.

Other rules carried from defaults-loop: confidence only triggers a second call to a stronger model; every stored
answer names the model that gave it; and Jev stays in shadow until its promotion criteria hold (§10 there; D90
narrows D31 for the loop only). **This design widens that narrowing to two more typed questions, and D91 says so.**
The labeled set, scoring and promotion criteria for them are in defaults-loop §10 (J6). Until promotion, set
membership uses class, phase and path matching by code only, and the signal watcher uses `match` fields only. Both
work without Jev; Jev adds paraphrase tolerance. Set membership by Jev must not drop a `requirement`; those are
selected by code alone.

## 6. Migration

Nothing is migrated by this job. A later job does it in this order, each step reviewable alone **(design)**.

1. **`defaults.toml` (12 entries).** Add typed fields in a derived view, not by editing the file: `kind =
   expectation`, `status = active`, `phase` from `applies_to`, `enforcement` by reading each default (`rust`,
   `tested`, `public` are partly checked by code and gate; the rest `brief`). `assumptions` and `signals` go in
   `athena.default-expectations/1` records keyed by default id (§2, decided), so `defaults.toml` and
   `DefaultsModel::parse` are not changed. The `changelog` entry is not migrated: it is not one of the 12, it is
   added to a reading in code by `ensure_changelog` (D89), and it has no `defaults.toml` id; the typed header of D89
   in `docs/decisions.md` (step 2) carries `enforcement = code-check`.
2. **`docs/decisions.md` (D01-D90).** Add a typed header per entry; leave the prose as written. The log's header says
   an accepted entry is never rewritten, yet D46 (line 1020) and D87 (line 1901) carry a note added after acceptance
   **(read)**; so the rule has had exceptions, and the header block is a separate block under the title that leaves
   the prose alone, not a claim that the rule always held. A script is not
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
   `assumptions` are written when a decision first has a signal fire, not for all 90 up front (D01-D90). This keeps the work
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
8. **Rewrite D01-D90 now.** Rejected: the log header says accepted entries are not rewritten (with the two notes
   added after acceptance, D46 and D87, as the only exceptions read); a header block leaves the prose alone.
9. **Store a default's assumptions and signals as new keys in `defaults.toml`.** Not chosen now (§2): it needs a
   governance job to change `DefaultsModel::parse`, and it puts watched conditions in the file every reader loads.
10. **Give the reader the whole set.** Rejected (§3): the reader set has 2 tokens of headroom (measured).

## 8. Build order, if accepted

0. **First build (review 1, section 5):** the read-only backtest of D87 and D88, with no Jev, no brief change and no
   new record kind other than the typed header. It starts once this revision is accepted.
   **Built** as `athena backtest` (`crates/athena-cli/src/backtest.rs`): typed headers for D87 and D88 only, the
   supersession parse, a records reader, D87's one exact-match signal (a `did-work-themselves` intervention of
   class `docs` on a landed job, two distinct jobs), and one introspection record (the code-written fields) for
   the earliest matching event, with the pass conditions of review 1 section 5 applied. A condition that does
   not hold is printed as a finding with its reason and the command exits 2; it is not retried. Its tests build
   their `records/` fixtures in a temporary directory and remove it, so no fixture file is committed. Read on
   `docs/decisions.md` (case-insensitive parse): the log has a fifth supersession hit the review's list does not
   name, "superseded by D39's invariant" in D11, so condition (a) reports it as an unexpected hit and the
   backtest does not pass on the log as it stands until the user confirms it as a known hit. Whether the real
   `records/` meet conditions (b) to (d) is not known here: the records are not part of the tests.
1. Measure rendered sets with the live counter and `render()` alone (defaults do not count against the producer and
   verifier budgets today, §3).
2. Typed decision view and the supersession parse (§6, steps 2-3), with a user-confirmed list. No briefs change.
3. Set selection by code for the reader (cap K), producer, verifier and truncation record (§3), **with the runtime
   per-job set-size check and its test**.
4. The host-error record written by the client (§4), landing as a watcher trigger, the watcher with exact matching,
   the `athena.decision-state/1` record and its brief component; the `athena.default-expectations/1` record.
5. Backtest the signals on past records (§6 step 5), D87 first.
6. Probe a multi-question `evaluate` call (§5), then add Jev for the two questions, in shadow, scored as in
   defaults-loop §10.

## 9. What is left open

- The number of distinct jobs and window per signal (set from backtest, not guessed).
- Whether `under-review` clears automatically after a quiet period, or only by a user decision. Starting rule: only
  by a user decision.
- Whether set membership by Jev ever replaces code matching. Not before the §5 promotion criteria hold.
- Whether to give `amended_by` a typed extent (`landing shape`, `count`) so partial supersession is queryable.
- How many entries of D01-D90 are still applied; unknown until step 2 of §6 is run.
- The value of K (reader cap) and `brief_set_tokens`: set from measurement in the first builds (§3).
- Whether `message_class` of a host-error record needs more values than the first backtest shows (§4).
- A correction needed outside these two designs: D90's "Alternatives" in `docs/decisions.md` cites "the design,
  section 9"; the alternatives are in defaults-loop §11. Corrected in this change by a dated note on D90 (see the
  log), since the log is not rewritten.
- Review 1 is recorded (`docs/design/reviews/decision-sets-review-1.md`); no review of this revision is recorded yet.
