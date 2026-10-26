# Jev code-health checks: typed questions about changed files and functions, run first in shadow mode

Status: design only. No code, no configuration and no workflow changes. The decision is proposed D130 in
[`docs/decisions.md`](../decisions.md); the build jobs in section 10 are not queued.

Every number below is an **estimate** unless it names a source. The one price used is the request's: about
$0.00001 for a Jev call. Nothing here was measured from `records/`.

## 1. What this is

**Jev** is a small model that answers typed choice questions. Athena sends it a text (the *state*) and a
question with a description of each option; it returns the chosen option, a probability per option, and a
confidence. It costs about $0.00001 a call (the request's figure; `docs/design/defaults-loop.md` also quotes about $0.000012 a
call from one probe, not re-measured). That is cheap enough to ask about every changed file and function at
every landing, where a line count cannot say what a function does.

This design asks five questions about the code a landing changes:

1. Does this function do one thing, or several?
2. Does this change add coupling between modules?
3. Which module does a new item belong in?
4. Is this file drifting toward a god object (a file that owns too many unrelated jobs)?
5. Does this change duplicate logic that exists elsewhere?

The **shadow period** is the first phase: Jev is called and every answer is recorded, but nothing acts on any
answer. The answers are scored later against what actually happened to the code. Only a question that beats a
plain deterministic baseline leaves shadow (sections 4 and 5).

Jev never acts alone. The checks are advisory: a confident answer can raise a finding in status, or ask for a
structural pass; it can never block a landing. The verifier and the scope guard still gate work.

## 2. How Jev is called

The calling convention is in `docs/design/defaults-loop.md` section 6 and is **reported there, not reproduced**.
This design uses it as follows:

- `POST https://ai-gateway.vercel.sh/v1/evaluate`, model `typesafe-ai/jev`. The chat route refuses Jev; the client
  treats that as a build error (section 6).
- Request fields: `state` (the text the question is about) and `questions`, a map from a key to
  `{type, instructions, criteria}` where `criteria` maps each option to its description.
- Every question here is a `choice` question with a neutral option (`none`, `fine`, `unclear`). For a `choice`
  question the threshold input is `p_top`, the probability of the chosen option (section 6.1). Boolean
  questions were rejected: `p_top` is not their threshold input, and the placement question has several options.
- Confidence `c` controls escalation only: below `t_conf`, an explicit second call goes to a stronger model, and
  the stored answer names the model that gave it (section 6.2). In shadow mode escalation is off (section 7).
- Open point from section 6: whether one call answers several questions is unprobed. This design assumes one
  call per question and notes the saving if batching works.
- Only stored fields go to Jev. The state is built from the diff and the repository's own files, never from logs,
  outcomes or records of other jobs (the section 5 rule of the same document).

## 3. The five questions

Common to all five:

- **Unit.** Questions 1 and 5 are asked per changed *function*; questions 2, 3 and 4 per changed *file*. One
  call per unit and question keeps each verdict scoreable on its own.
- **Excerpt bound.** The state is the changed hunk plus up to 40 lines of context above and below, capped at
  6,000 bytes (estimates). A longer excerpt is cut at a line boundary around the hunk and says it was cut. Whole
  files are never sent.
- **Thresholds** are starting values (estimates). Each is set from shadow calibration (section 5) before it
  triggers anything. Two thresholds per question: `t_find` (a status finding) and `t_pass` (a structural-pass
  request). `t_pass` is always higher than `t_find`.
- **Effects** during shadow: *record only*. The effects below apply after a question exits shadow (section 4).
- **Structural pass** means the request in job 20261026-042558-5p65, the deterministic code-health check (planned,
  not built; section 6). Jev asks for one only when that check's own measure also trips on the same file, unless
  the question's precision has been measured high enough after shadow (section 4).

### 3.1 Q1 `one-thing`: does this function do one thing or several?

| Field | Content |
|---|---|
| State | The function's text (up to the bound), its signature, the file path, and the names of the functions it calls in the same file. |
| Options | `one` (a single job, any length); `several` (two or more separable jobs); `unclear` (cannot tell from the excerpt). |
| Criteria | `one`: a reader can state what it does in one sentence without "and". `several`: it parses, decides and writes in the same body, or has sections that could be named and called separately. `unclear`: the excerpt is cut or lacks context. Length alone is not a criterion. |
| Threshold | `t_find` 0.70 on `several`; `t_pass` 0.85 on `several`. |
| Effect | Below `t_find`: nothing. At or above: a status finding naming the function. At or above `t_pass` and the deterministic function-length or nesting measure also trips: a structural-pass request to split it. |

### 3.2 Q2 `adds-coupling`: does this change add coupling between modules?

| Field | Content |
|---|---|
| State | The file's path, its changed hunk, the file's import or `use` lines before and after the change, and the module list (directory names, one line each, from the repository tree). |
| Options | `none` (no new dependency between modules); `along-existing` (new use of a module already depended on); `new-edge` (a module now depends on one it did not before); `cycle` (the new dependency runs back to a module that depends on this one). |
| Criteria | Decided from the import lines and the module list only. `cycle` needs both directions visible in the state; if the reverse edge is not in the state, Jev must not choose it. |
| Threshold | `t_find` 0.70 on `new-edge` or `cycle`; `t_pass` 0.85 on `cycle` only. |
| Effect | Below `t_find`: nothing. At or above: a status finding naming the new edge. At or above `t_pass` with a deterministic import-cycle or fan-in measure also tripping: a structural-pass request. `along-existing` and `none` do nothing. |

The deterministic part (the import graph) can state most of this exactly. Jev's value here is the judgement of a
dependency that is not an import (a shared constant, a type reached through a re-export). The question is
therefore kept only if shadow scoring shows lift over the import-graph baseline (section 5).

### 3.3 Q3 `module-placement`: which module does a new item belong in?

Asked when a landing adds a new function, type or file, not for edits.

| Field | Content |
|---|---|
| State | The new item (signature and up to the bound of its body), the file it was added to, and the module list with a one-line purpose for each (the first doc-comment line of each module). |
| Options | `here` (this file is right); one option per candidate module, at most six, chosen deterministically as the modules whose names or imports are closest to the item's; `new-module` (none fits). |
| Criteria | `here`: the item uses the file's existing types and is used mostly from it. A candidate module: the item mainly operates on that module's types or is mainly called from it. `new-module` only when no purpose line fits. |
| Threshold | `t_find` 0.75 on any option other than `here`; there is no `t_pass`. |
| Effect | Below `t_find`: nothing. At or above: a status finding "item X may belong in Y". It never requests a structural pass, because moving one item is not a structural pass. |

### 3.4 Q4 `god-object-drift`: is this file drifting toward a god object?

| Field | Content |
|---|---|
| State | The file's path; its outline (the item names with one line each, no bodies); its size now and 10 landings ago (lines and item count); and the changed hunk. |
| Options | `focused` (items serve one purpose); `broadening` (the change adds a purpose the file did not have); `sprawling` (the file already owns three or more unrelated purposes). |
| Criteria | Judged from the outline: group the item names by purpose. `focused`: one group. `broadening`: this change starts a new group. `sprawling`: three or more groups, whatever the size. Size trend is context, not a criterion. |
| Threshold | `t_find` 0.70 on `broadening` or `sprawling`; `t_pass` 0.85 on `sprawling`. |
| Effect | Below `t_find`: nothing. At or above: a status finding naming the file and groups. At or above `t_pass` with the deterministic file-size measure also tripping: a structural-pass request to split the file. |

### 3.5 Q5 `duplicates-existing-logic`: does this change duplicate logic that exists elsewhere?

| Field | Content |
|---|---|
| State | The new or changed function (up to the bound), and up to three candidate functions found deterministically elsewhere in the repository (same or similar name, or the most shared identifiers), each cut to the bound with its path. |
| Options | `new` (no candidate does the same job); `duplicate` (a candidate does the same job); `near-duplicate` (the same job with a difference that may be deliberate). Each option names a candidate by number. |
| Criteria | `duplicate`: the candidate could replace the new code with a call and no change in behavior. `near-duplicate`: it could with a parameter. When the deterministic search finds no candidate, no call is made: the answer is `new` and costs nothing. |
| Threshold | `t_find` 0.75 on `duplicate` or `near-duplicate`; `t_pass` 0.90 on `duplicate`. |
| Effect | Below `t_find`: nothing. At or above: a status finding naming both functions. At or above `t_pass` with the deterministic similarity measure also tripping: a structural-pass request to reuse the existing function. |

## 4. The shadow period

**Definition.** For a stated period, every landing runs the checks and records each answer. The record holds the
question key, the unit (path and function), the state hash, the options with their probabilities, `p_top`,
confidence, the model, tokens, the cost read from the response, and the landing's job id and base. **No answer
produces a finding, a status line or a request.** Escalation is off. The record is a new kind with a version
(`athena.jev-answer/1`), written next to the job's other records; the exact fields are fixed by job J1 in
section 10, which also confirms the existing landing-record field names (this design did not check them).

**Exit rule (per question; estimates).** A question leaves shadow when both hold on two consecutive windows
(a window is 30 days or 40 landings, whichever is longer):

1. At least 100 answers at or above `t_find` have been scored (so the precision has a meaning).
2. Its precision, as defined in section 5, beats the deterministic baseline by at least 15 percentage points.

A question that has not met this after four windows is dropped, and the drop is recorded with its numbers. After
exit, the first effect enabled is the status finding. The structural-pass request is enabled only after a
further window where the finding's measured precision stays above the exit margin. Sri accepts each exit; the
rule does not move a question out of shadow by itself.

## 5. Scoring by evidence

Scoring uses later landing records only, as Sri asked for Jev's class answers (BRIEF.md addendum of 2026-10-25):
no human labels and no stronger-model opinion. A stored answer is scored at the end of a *follow window* of the
next 20 landings after it (estimate). The fields used are the job's changed-file list, its diff statistics,
its outcome (`landed`, `rejected`, `blocked`) and the findings of its verifier, all from the landing records of
`docs/records.md`; job J1 names the exact fields.

**Event A: a flagged file later breaks or churns.** Let a *flag* be an answer at or above `t_find` on a
non-neutral option. The flag is a **hit** if, in the follow window, any of these happens:

- a later landing whose job reason is a fix or revert changes that file (a break);
- a later job's verifier finding or failed check names that file (a break);
- the file's **churn** (lines added plus deleted, summed over landings in the window) is above the median churn
  of all files changed in the window (churn).

Precision is hits divided by flags. For function-level questions the same events are matched at the file, since
function identity is not stable across edits.

**Event B: a recommended split lands and churn drops.** Let a *recommendation* be an answer at or above `t_pass`
(scored in shadow even though it does nothing) for Q1 or Q4. It is a **hit** if (i) within the window a landing
splits the file or function, shown by a new file or function that takes at least 30% of the removed lines, and
(ii) the file's churn over the 20 landings after the split is lower than over the 20 landings before it. Both
numbers come from the diff statistics. The *rate* of recommendations that land is also reported, since a good
recommendation nobody acts on is not scoreable by B.

**Baseline.** Each question is compared with a deterministic baseline picking the same number of units: the most
churned or largest files for Q1, Q4; the import graph for Q2; the closest-name rule for Q3 and Q5. **Lift** is
Jev's precision minus the baseline's. A question with no lift does not exit shadow, however plausible it reads.

**Limits.** Churn is high in a file being actively developed whatever its health, so Event A's churn clause can
reward flags on busy files. The baseline of most-churned files has the same bias, which is why lift, not raw
precision, is the exit test. Event B has few samples. Both are reported with counts, and a result built on fewer
than 20 samples is marked inconclusive.

## 6. How the checks fit with what exists

- **Deterministic code-health check (request 20261026-042558-5p65, planned, not built).** That request defines
  measures and thresholds (as that request states them; this design does not restate or change them). They are
  the *floor*: cheap, exact, reproducible. Jev adds what a measure cannot: whether a function does one thing,
  whether a dependency is new. The two meet at the effect: Jev's structural-pass request goes through the
  deterministic check and needs its measure to trip too, until Jev's precision is measured. Jev never opens a
  pass the deterministic check would not allow.
- **Hestia (D129, [`hestia.md`](hestia.md)).** Hestia sweeps repository and system health, rules first, and
  fixes only through small jobs. Her "docs match the code" check and Jev's checks do not overlap: Hestia asks
  whether things were left behind; Jev asks about the shape of newly changed code. Jev findings may appear in
  the same status area as Hestia's report, but are separate and labelled with the question key. Hestia's budget
  (hestia.md section 4) is not drawn on.
- **Scope guard (D127, [`scope-guard.md`](scope-guard.md)).** The scope guard weighs improvement work by value
  and cost and watches for thrash. A structural-pass request from Jev is an improvement request like any other:
  it goes through the guard, which may refuse it, and a request that repeats on a file that churns is the guard's
  thrash signal. Jev's Event B (split lands, churn drops) is the measured value the guard's cost view needs.
  Jev never bypasses the guard.
- **Verifier.** Unchanged. Jev findings are not acceptance criteria and never reject a candidate.

## 7. Cost per landing

Price: **$0.00001 per call** (the request's figure; not measured here). Cost per landing = calls × $0.00001.

Call count (all **estimates**, to be replaced by measured counts from the shadow records):

| Question | Units per landing (estimate) | Calls |
|---|---|---|
| Q1 one-thing | 8 changed functions | 8 |
| Q2 adds-coupling | 3 changed files | 3 |
| Q3 module-placement | 1 new item | 1 |
| Q4 god-object-drift | 3 changed files | 3 |
| Q5 duplicates | 8 changed functions, about half with a candidate | 4 |
| **Total** | | **about 19** |

About 19 calls × $0.00001 = **about $0.0002 per landing**. A heavier landing (25 functions, 8 files) is about
60 calls, or $0.0006. Escalation (off in shadow) adds one stronger-model call per answer below `t_conf`; its price
is not known and is not included. **Cap:** at most 100 Jev calls per landing; units beyond the cap are skipped,
largest hunks first kept, and the skip is recorded. At the cap the cost is $0.001. If one call can answer several
questions for one unit, the count falls by up to a third; that is unprobed.

For scale: a producer run costs dollars (the job budget is $3 for this job), so the checks add well under 0.1%
of a landing's cost on these estimates.

## 8. Alternatives considered

| Option | Why not |
|---|---|
| Line counts and function length only | Cannot say what a function does; they remain the deterministic floor. |
| Boolean per question | `p_top` is not the threshold input for booleans; placement has many options. |
| One call per file with all functions | Larger state, mixed verdicts, hard to score one function. |
| Whole-file state | A 5,000-line file exceeds any bounded cost. |
| Jev alone triggers structural passes | A false positive costs a producer run; precision is unproven. |
| Block landing on a confident answer | The check is advisory; the verifier and scope guard gate. |
| A model reviewer per landing | Orders of magnitude costlier than Jev. |
| Score by Sri's review or a stronger model | The addendum asks for evidence from records; opinion is not evidence. |
| Act at once, no shadow | No measured precision; the first false findings would teach Sri to ignore them. |

## 9. Risks and open points

- **Thresholds and counts are guesses.** Calibration in shadow replaces them. Until then no answer acts.
- **Self-fulfilling churn.** A finding that prompts a split changes the churn it is scored on. In shadow nothing
  acts, so scoring is clean; after exit, scored precision is tracked on a held-out share of flagged units that are
  not acted on (10%, estimate).
- **Data leaving.** The state holds source text. Only the bounded excerpt goes to Jev, never records or logs.
- **Batching unprobed.** See section 2.
- **Landing-record field names** for changed files, diff statistics and fix-or-revert reason were not checked
  for this design; J1 confirms them and adjusts section 5 if one does not exist.
- **The deterministic check does not exist yet.** Until it does, the `t_pass` effect has nothing to ask, and the
  structural-pass effect (J6) waits for it.

## 10. Build plan

Each job is sized for **one producer run**: one focused change with its tests. They are in order; none is queued.

| Job | Change | Files | Test or demonstration |
|---|---|---|---|
| J1 | Shadow answer record `athena.jev-answer/1`, its schema in `docs/records.md`, fixtures from a fake Jev, and a note of the real landing-record fields used by scoring. | `docs/records.md`, record type in the core crate, fixtures | Unit test: a record round-trips and rejects a missing `p_top`. A doc test that the schema is listed. |
| J2 | State builder: for a diff, the units, the bounded excerpt (40 lines, 6,000 bytes) and context (module list, outline, import lines). | new module in the core crate, test fixtures | Tests: a long function is cut at a line boundary and says so; a 5,000-line file yields a bounded state; no record content enters it. |
| J3 | Question definitions for the five questions and a client call in shadow mode, with the fake Jev (same route and fields as `defaults-loop.md` section 6) and the per-landing call cap. | core crate, fake Jev, tests | Tests: the fake refuses the chat route (build error); the cap skips and records; shadow produces records and no finding. |
| J4 | Scorer: Event A, Event B and the baseline from landing records, with the exit-rule report. | core crate, tests with synthetic landings | Tests: a flag followed by a fix landing on the file is a hit; a split followed by lower churn is a hit; no lift means no exit. |
| J5 | Status display: the shadow report (counts, precision, lift per question) and, after exit, findings. | status code, tests | Test: status prints the report from fixture records and no finding while a question is in shadow. |
| J6 | Effect wiring after exit: status finding, then structural-pass request through the deterministic check and the scope guard. Waits on 20261026-042558-5p65 and Sri's acceptance of an exit. | check and request code, tests | Test: a confident answer in shadow does nothing; the same after exit raises a finding; a pass request needs the deterministic measure to trip. |

Before J3, the building job probes a live multi-question call and records the result in `defaults-loop.md`
section 6, as that section requires.
