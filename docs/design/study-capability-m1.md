# The studying capability, milestone M1: design

> **Status:** revised after [review 1](reviews/study-capability-m1-review-1.md) (approve with changes). Every
> finding is answered in [Appendix A](#appendix-a-response-to-review-1), and the eight decisions the review put
> to the user are settled in the text and recorded as D64 in [`docs/decisions.md`](../decisions.md). **A re-check
> of this revised design is required before building** ([§9](#9-review-gate)); the revision has not been
> re-reviewed. **Design only.** This document describes a capability that does not exist. No crate, code, test,
> or configuration has been written for it, and nothing will be built until the re-check is complete.
> **Requirements source:** [`BRIEF.md`](../../BRIEF.md). Line numbers below refer to the `BRIEF.md`
> in the same tree as this document, not to a commit. To confirm them, open `BRIEF.md` at the cited line:
> line 306 is the bullet beginning "M1. Work handling", line 395 is the third item of the milestone list
> ("First milestone: one mechanism (M1 on firstmate)"), and lines 271–277 are the heading "Part 2" through the
> bullet "Everything reusable lives in Athena's studying capability". If `BRIEF.md` has since changed, find
> the passage by those quoted words.
> **Style:** follows [`athena-v0.md`](athena-v0.md). Alternatives, tradeoffs, and reversal cost are in
> [§6](#6-considered-alternatives) and [§7](#7-reversal-cost), in the shape of
> [`docs/decisions.md`](../decisions.md).
> **No claims about firstmate.** This document says nothing about how firstmate behaves, what it contains,
> or how it was built. Every example is labeled **synthetic**: it is invented to show a shape and describes
> no real system.

## Contents

1. [Goal](#1-goal)
2. [The M1 method](#2-the-m1-method)
3. [Evidence rules](#3-evidence-rules)
4. [Data model](#4-data-model)
5. [The work-handling model and its validation](#5-the-work-handling-model-and-its-validation)
6. [Considered alternatives](#6-considered-alternatives)
7. [Reversal cost](#7-reversal-cost)
8. [Out of scope](#8-out-of-scope)
9. [Review gate](#9-review-gate)
10. [Appendix A: response to review 1](#appendix-a-response-to-review-1)

---

## 1. Goal

The brief makes the study Athena's first big task and says that everything reusable lives in Athena's
studying capability, while everything specific to one studied system stays with that instance
(`BRIEF.md:271-277`). Its first milestone is one mechanism, M1, on firstmate, done end to end before the
study broadens (`BRIEF.md:395`). M1 is "work handling: from request to done, including failures, retries,
and concurrency, modeled formally as data" (`BRIEF.md:306`).

This document designs the **capability** that milestone needs: the reusable machinery that lets Athena
study the work handling of *any* orchestration system and publish the result. It is not the study. The
first use of the capability, on firstmate, is a separate job that happens after review and build.

The brief's first milestone is "designed, built, verified, and live on Pages" (`BRIEF.md:395`). This
capability covers the study up to a validated study. It is **not** M1 done: the Pages design in
[§8](#8-out-of-scope) is a required part of M1, and the study itself is another.

The capability must make four things true of every study it produces:

1. **Pinned.** The studied system is fixed at a recorded commit SHA before anything is read (`BRIEF.md:275`).
2. **Evidenced.** Every claim cites a source and says how it was learned (`BRIEF.md:320`, `BRIEF.md:123`).
3. **Honest about rationale.** A design decision's rationale says whether it was stated, evidenced, or only
   inferred, and inferred rationale is never shown as stated (`BRIEF.md:321`).
4. **Checkable as data.** The work-handling model is data that code can validate, not prose.

## 2. The M1 method

One study run goes through five steps. Code owns the sequencing, as in v0 ([`athena-v0.md`](athena-v0.md)
§0, commitment 1); agents own bounded judgments inside a step.

| Step | What happens | Output |
|---|---|---|
| 1. Pin | Resolve the studied repository's reference to a full commit SHA, and to a tag name when the reference is a tagged release, and record them with the date. All later steps read that SHA only, from a detached snapshot, never the live default branch. | `pin` record |
| 2. Read | Agents read the pinned tree and write **read claims**: each one cites path, line range, SHA, and a verbatim quote. The reader's role is read-only. | claims marked `read` |
| 3. Run | Code runs the studied system, inside the containment of [§2.1](#21-the-run-step-containment-redaction-and-caps), on a scripted scenario that exercises success, failure, retry, and concurrency, and captures the streams of [§4](#4-data-model). Each run has a recorded configuration and the pinned SHA. | claims marked `observed`, plus run records |
| 4. Model | Claims are assembled into the work-handling model ([§5](#5-the-work-handling-model-and-its-validation)). Where a read claim and an observed claim disagree, the claim author links them with `contradicts`, and the divergence is a recorded finding, not a silent choice (`BRIEF.md:300`). | `model` record |
| 5. Publish | Code validates the whole study; only a study that passes validation and an independent verification is rendered as the explainer page. | validated study, then page |

Rules for the steps:

- **Pin before read.** A step 2 claim whose SHA differs from the pin is invalid. A run of any other tree is
  invalid for the same reason ([§5.4](#54-validation-rules)).
- **Run is separate from read.** A claim can be `both` only when it carries at least one file source and at
  least one run source. A claim that was only read says so, so the reader can tell what has been exercised
  (`BRIEF.md:320`).
- **Replay.** Each run record holds the recorded configuration and the command that reproduces it. The
  replay step re-runs a recorded run from that record alone, with the scripted human side, inside the same
  containment, and records the replay as a new run that points at the original. Replay results are compared
  with the original capture; a difference is a `finding`. Rendering the replay on the site belongs to the
  Pages design (`BRIEF.md:299`).
- **Fresh context.** Each study's reader and scenario invocations receive the capability's prompts and that
  study's own records, and no other study's notes beyond labeled synthetic examples (`BRIEF.md:278`). Code
  composes the context, so the separation is a mechanism and not a request.
- **Gap log.** A gap found in the capability during a study is recorded as a `study.gap/1` record, fixed in
  the capability and never by hand-patching output, and re-checked against earlier studies (`BRIEF.md:279`)
  by running the validation of [§5.4](#54-validation-rules) again over them.
- **Scenarios are written apart from the reader.** A scenario is written by a separate invocation that sees
  the studied system's public documentation and the model's element kinds, but none of the reader's claims,
  so a scenario cannot be written to confirm what was read. The verifier checks this by the invocation
  records.
- **Publishing is the last step**, and what it publishes is decided in a later design that is a required
  part of M1 ([§8](#8-out-of-scope)).

### 2.1 The run step: containment, redaction, and caps

Running a third-party system runs code that Athena's agents did not write. `athena-v0.md` §16.2 says that
containment is pulled in by the first job that runs such code. This is that job, so **containment is part
of this capability**, not a residual:

1. **Containment.** The studied system runs in a boundary that is separate from the user's account: a
   throwaway operating-system user or container, a scratch directory as its home, no access to the user's
   home or to Athena's records, and an environment built from an allow-list (as v0 builds for agents), not
   inherited. The only credential in that environment is the model credential the run needs, and its value
   is never written to a record. The studied systems are limited to ones the user has chosen to study.
2. **If no boundary exists, the run does not start.** The step downgrades to read-only explicitly: it records
   the blocker, the affected claims stay `read`, and the study goes on (`BRIEF.md:110`). A run record names
   the boundary it ran in.
3. **Redaction before storage.** Captured output passes through redaction before anything is written under
   `records/`. The redactor removes known secret patterns, personal data, and host paths (replaced with
   placeholders, as v0 does for agent text). Raw output is never stored. The commit gate remains a second
   line, and a refusal at commit counts as a defect in the redactor, not as the method (`BRIEF.md:299`).
4. **Caps and the meter that enforces them.** Every run has a per-run cap and a kill switch, both recorded
   (`BRIEF.md:111`). Two meters apply, and the run record says which one governed:
   - *Wall-clock and process:* Athena enforces a time limit and kills the run's process group with v0's kill
     switch. This always applies.
   - *Model spend:* v0's `[budget]` caps meter Athena's own agent calls, not the studied system's. The
     studied system's model calls are bounded by a credential with a provider-side spend limit, and the cost
     is read back from the provider and recorded as a `Quantity` with its provenance.
   - *If the spend cap can be neither enforced nor observed, the run does not start* (the downgrade of
     item 2). Cost recorded only after the fact is never accepted as the cap.

## 3. Evidence rules

These rules are the capability's core, and each is enforced by validation ([§5.4](#54-validation-rules)),
not requested in prose. The **evidence rules are non-waivable**: no `finding` waives them
([§5.4](#54-validation-rules)).

### 3.1 Claims

Every claim carries one or more **sources**:

- for a file: path, start line, end line, the pinned SHA, and a verbatim `quote` from those lines;
- for history: a commit SHA, or a pull request identifier with the SHA of its head or merge commit; the
  commit must exist and be reachable from the pin;
- for a run: a run id and a timestamp, where the run is a `study.run` record of the pinned SHA and the
  timestamp lies within the run.

Every claim is marked **`read`** (learned from reading the source), **`observed`** (learned from a run), or
**`both`**. A claim with no source, or with a mark that does not match its source kinds, is invalid:
`read` cites no run, `observed` cites at least one run, and `both` cites at least one file source **and**
at least one run source. A claim that contradicts another claim carries a typed link, `contradicts: <claim
id>`.

### 3.2 Rationale cards

A rationale card records why a significant design decision was made. It carries:

- the decision;
- the alternatives considered;
- the tradeoff;
- the reversal cost;
- a **provenance**: `stated`, `evidenced`, or `inferred`.

A card with an empty field is invalid. The provenance means:

| Provenance | Meaning | Required support |
|---|---|---|
| `stated` | The authors say why, in a source Athena can cite. | A cited source whose `quote` contains the statement. |
| `evidenced` | Not stated, but the sources and runs show it. | At least one cited claim for each part of the rationale. |
| `inferred` | Athena's own reconstruction. | An explicit `inferred` label. |

Only `stated` cards may be presented as what the authors said. The card record has a field for the
authors' voice (`authors-say`) that is allowed only when the provenance is `stated`; the renderer takes the
provenance label from the record and has no other path to that voice ([§5.4](#54-validation-rules)). The
check is on the record and on the renderer's template, not on prose, because a word list is evaded by
paraphrase. Whether a quote means what the card says, and whether the cited claims of an `evidenced` card
support each part, are judgments that belong to the verifier ([§5.5](#55-what-the-verifier-owns)).

### 3.3 Numbers

A number is either measured (with the run or tool that measured it) or labeled as an estimate, as in v0's
`Quantity` ([`athena-v0.md`](athena-v0.md) §5.1, item 5). Numbers in a study are `Quantity` fields, and the
type refuses a bare number. A number written in prose has no rule; it is checked by the verifier.

## 4. Data model

The capability would be a crate, `athena-cap-study`, implementing the core's `Capability` seam
([`athena-v0.md`](athena-v0.md) §6.2). **The crate is not created by this change.** The model below is
prose; field names are proposals for review. Records follow v0's rules: one fact per file, written once,
typed at the boundary, with a `schema` field.

| Record | Key fields |
|---|---|
| `study.pin/1` | study id; system name; repository URL; full commit SHA; tag name (or none); resolved-at; how the reference was resolved |
| `study.source/1` | a citation: kind (`file`, `commit`, `pull-request`, `run`); path; start line; end line; SHA or id; verbatim `quote` (required for `file`); timestamp for runs |
| `study.claim/1` | id; text; mark (`read`, `observed`, `both`); one or more sources; `contradicts` links (zero or more claim ids); the mechanism it belongs to (for M1, `work-handling`) |
| `study.run/1` | id; **sha** of the tree run; scenario id; recorded configuration; command line; start and end; the boundary it ran in; the per-run cap, the kill switch, and the meter that governed each; cost as quantities; exit state; the blocker when the run did not start; the transitions observed (element ids); the capture streams below |
| `study.scenario/1` | id; purpose; which model elements it is meant to exercise; scripted human side; the invocation that wrote it |
| `study.model/1` | the work-handling model ([§5.1](#51-elements)); each element lists the claims that support it |
| `study.rationale/1` | the rationale card of [§3.2](#32-rationale-cards), plus the claims it cites |
| `study.finding/1` | kind: `divergence` (both claims), `waiver` (the rule, the user's reason text, and the user intervention that wrote it), `not-native` (a capture stream that had to be added), `no-observation` (a stream the system did not expose), `could-not-run` (the blocker), or `model-limit` (a case [§5.1](#51-elements) cannot express); severity |
| `study.gap/1` | a gap in the capability found during a study; the study; the fix; the earlier studies re-checked |
| `study.validation/1` | the result of running every rule in [§5.4](#54-validation-rules): rule id, pass or fail, offending record ids |

**Capture streams.** A run record lists each stream of `BRIEF.md:298` with one status: `native` (the
studied system exposes it), `added` (Athena had to add it, which is a `not-native` finding), or `absent`
(not captured, which is a `no-observation` finding). The streams are: prompts as sent; tool calls; token
usage by agent, role, and model; timing; topology; state and memory changes; git activity; every point where
the human is addressed. A stream may be `absent`; an `observed` claim about a failure or retry path must
cite a run in which the stream that shows that path is `native` or `added`. The captured content is the
redacted output of [§2.1](#21-the-run-step-containment-redaction-and-caps).

Storage reuses v0: records are append-only under `records/`, and a correction is a new record that
references the old one. Records specific to one studied system stay together under that study's own
directory, and the reusable parts (schemas, validation, prompts) live in the capability. That is the
separation the brief asks for (`BRIEF.md:277`).

**Synthetic example** (an invented system, "Example Runner", no relation to any real one):

```text
claim c1  "A task moves from queued to running when a worker is free."
          mark: read   source: file src/queue.example:10-24 @ <pinned sha>   quote: "<verbatim text>"
claim c2  "A task that times out is requeued after its first and second attempts, and fails after its third."
          mark: both   sources: file src/retry.example:5-12 @ <pinned sha>   quote: "<verbatim text>";
                       run r4 (sha: <pinned sha>) @ 2026-01-01T00:00Z
rationale "Retries are bounded." provenance: inferred   cites: c2
```

The values above are made up to show structure only.

## 5. The work-handling model and its validation

### 5.1 Elements

The model is a data description of how one system takes a unit of work from request to done. It has:

- **States.** Named conditions of a unit of work, each with a kind: `initial`, `active`, `waiting`,
  `terminal-success`, `terminal-failure`.
- **Counters.** Named values a unit carries, each with its start value and the transitions that change it.
  A guard may use only a declared counter.
- **Transitions.** Each has a source state, a target state, a **trigger** (the event that causes it), and
  a **guard** (the condition that must hold, or `none`). A transition also lists the actor that performs
  it (a person, a process, an agent, or the clock), and for the trigger and the guard an `evaluated-by`:
  `code`, `agent`, or `person`. A guard evaluated by an agent or a person is a model-decided transition: the
  validator cannot check it against a run, so it lists it and leaves the check to the verifier.
- **Failure edges.** Transitions whose trigger is a failure (an error, a timeout, a refusal, a crash). Each
  names the failure it handles.
- **Retry edges.** Failure edges that enter a state from which their own source state is reachable. Each
  records a **bound** (the maximum attempts, counted in attempts and not in requeues) and a **backoff**, or
  says the bound is absent. Each also records **what changes on the retry** (nothing, a different model, an
  added instruction, escalation to a person), as a list that may say `nothing`.
- **Unhandled failures.** A list of failures the system has no edge for (a unit dropped or lost without
  notice), each with its supporting claim, or an explicit empty list with a claim saying so. An unhandled
  failure is not a dead end; it is how the model says what it does not cover.
- **Concurrency.** Which units of work can be in which states at once: the shared resources, the locks or
  queues that serialize access, the limit on simultaneous units, and what happens when two units conflict.
- **Human touchpoints.** The states or transitions where a person is addressed or must act, each with the
  rule for a person who never answers (a timeout, an abandonment rule, or `none`, with a claim).

Every element lists its supporting claims. An element supported by no claim is invalid.

**Limits of M1 (named, not hidden).** The first schema does not express *parallel regions* (several agents
working one unit in different states at once) or *fan-out and join* (a unit that spawns sub-units and
completes when they do, with failure propagation). A study whose system needs either must say so in a
`finding` of kind `model-limit`, describe the case in claims, and leave the part unmodeled and listed; it
may not squeeze the case into the single-actor form. Adding
either is a new `study.model` schema version, which leaves published studies readable under the old one
([§7](#7-reversal-cost)). A system that is mostly one of these is not a good first subject for the M1
capability, and that is recorded as a limit and not as a pass.

### 5.2 Why transitions carry trigger and guard

A bare arrow between two states hides what can make it fire and when it cannot. The trigger answers
"what makes this happen", the guard answers "what stops it", and the pair is what a reader and the
validator check against runs, for the parts evaluated by `code`. A missing guard is written `none`
explicitly, so absence is a claim, with its own citation, not an omission.

### 5.3 Synthetic example

This model describes the invented "Example Runner" and nothing real:

```text
states      queued (initial) · running (active) · failed (terminal-failure) · done (terminal-success)
counter     attempts  start 0; +1 on each entry to running
transition  queued -> running   trigger: worker-free        guard: slot < limit    actor: scheduler  evaluated-by: code
transition  running -> done     trigger: exit-0             guard: none            actor: worker     evaluated-by: code
transition  running -> queued   trigger: timeout            guard: attempts < 3    actor: scheduler  evaluated-by: code
                                [retry, bound 3 attempts; changes on retry: nothing]
transition  running -> failed   trigger: timeout            guard: attempts == 3   actor: scheduler  evaluated-by: code   [failure]
unhandled   none (claim c9)
concurrency limit 4 units running; one lock per target file; conflicting units wait in queued
```

A unit enters `running` with `attempts` 1, 2, then 3. A timeout after the first or second attempt requeues
it (two requeues); a timeout after the third fails it. That is three attempts, the bound. The edge
`running -> queued` is a retry edge because it enters `queued`, from which `running` is reachable.

### 5.4 Validation rules

Code runs these rules over the study before publishing. A failure blocks publishing, with two classes:

- **Evidence rules (non-waivable).** Nothing waives them; a study that fails one is not published.
- **Model rules (waivable).** A `finding` of kind `waiver` clears one of these only when it names the rule
  and the offending ids, carries a reason written by the user, and cites the user intervention record that
  wrote it. A waiver by an agent, or one with no such record, is invalid. The page shows each waiver with
  the rule, the ids, and the user's reason text, in the same view as the element it covers.

| Rule | Class | Fails when |
|---|---|---|
| Uncited claim | evidence | a claim has no source; its mark does not match its source kinds (`read` with a run, `observed` with no run); or a `both` claim lacks a file source or a run source |
| Wrong SHA | evidence | a file citation names a SHA other than the pin, or a cited run's `sha` is not the pin |
| Bad citation | evidence | a cited path or line range does not exist at the pinned SHA; a commit or pull-request SHA does not exist or is not reachable from the pin; a run id has no `study.run` record; or a run timestamp lies outside the run |
| Quote mismatch | evidence | a file source's `quote` is not a substring of the cited lines at the pinned SHA |
| Stated without source | evidence | a `stated` card cites no source whose `quote` field is present |
| Inferred shown as stated | evidence | a card whose provenance is not `stated` has an `authors-say` field, or the renderer's template can reach that field for such a card |
| Incomplete card | evidence | a rationale card has an empty decision, alternatives, tradeoff, reversal cost, or provenance |
| Unsupported number | evidence | a `Quantity` field has neither a measurement nor an estimate label |
| Uncited element | model | a state, counter, transition, failure edge, retry edge, unhandled-failure list, or concurrency rule lists no supporting claim |
| Unreachable state | model | a state cannot be reached from any `initial` state by following transitions |
| Dead end | model | a non-terminal state has no outgoing transition |
| Dangling target | model | a transition names a source or target that is not a declared state, or a retry edge enters an undeclared state |
| Missing guard | model | a transition has no guard field (the value `none` is allowed; absence is not) |
| Undeclared counter | model | a guard uses a counter the model does not declare |
| Unbounded retry | model | a retry edge has no bound, or has no claim that says the bound is absent, or lacks its list of what changes |
| Unexercised path | model | a failure or retry edge appears in no run's observed transitions and has neither a `could-not-run` finding nor a `no-observation` finding that names it |
| Divergence unrecorded | model | a claim carries `contradicts` and no `divergence` finding links the two claims |
| Missing limit or list | model | the unhandled-failures list, the retry-change list, or a human touchpoint's no-answer rule is absent (an empty value is allowed; absence is not) |

Each rule is checked by a test that holds one planted fault and one clean case, following the pattern in
`docs/enforcement.md`. Those tests are part of the later build, not this change. Checks that need a git
lookup (Bad citation) use a small fixture repository.

### 5.5 What the verifier owns

The rules above are code. These are judgments that no rule decides, and they belong to the independent
verifier (A3-B in [§6](#6-considered-alternatives)), who must record a verdict on each in the study's
verification:

1. **Unlinked contradictions.** Two claims that contradict each other with no `contradicts` link.
2. **Meaning.** Whether a quote means what its claim or card says, and whether the cited lines support the
   claim.
3. **Support for `evidenced` cards.** Whether the cited claims support each part of the rationale.
4. **Prompt injection.** The reader works from a third-party tree that may contain text aimed at an agent.
   A read-only role limits the harm to wrong claims and does not remove it; the `quote` check shows that the
   cited words exist but not that the reader understood them as data. The verifier reads the cited lines
   with that in mind, and any instruction-like text in a cited source is recorded as a `finding`.
5. **Guards evaluated by an agent or a person.** Whether the model describes them faithfully.
6. **Scenario independence.** Whether a scenario was written without sight of the reader's claims.
7. **Numbers in prose.**

## 6. Considered alternatives

### A1. Where the capability lives

| Option | Gives | Costs |
|---|---|---|
| **A. A new crate `athena-cap-study`, implementing the `Capability` trait (chosen for review)** | Matches the brief: studying is the first capability and the core stays unchanged (`BRIEF.md:186`). Reusable pieces and system-specific records separate cleanly. | A new crate means a new documented dependency edge and a new entry in `docs/architecture.md`. The trait may need to grow to fit a long, multi-step job that is not "a verified change". |
| B. Fold studying into `athena-cap-change` | No new crate. | Mixes two kinds of work. Breaks the brief's rule that a new kind of work is a capability, not a core change. |
| C. A script collection outside the capability seam | The fastest route to a first result, and no new trait or crate to review. | Not reusable by the Hermes and Athena studies, no gate coverage, and unverified by the same machinery. A first result that cannot be re-checked or reused does not meet the brief's rule that everything reusable lives in the capability. |

### A2. How the work-handling model is represented

| Option | Gives | Costs |
|---|---|---|
| **A. Typed records: states, transitions with trigger and guard, retry bounds, concurrency (chosen for review)** | Validation is mechanical. The model can be rendered and compared across systems. | Some systems may not fit the shape well. The record types would need extending. |
| B. Prose with diagrams | Cheap to write and flexible. | The brief asks for "modeled formally as data" (`BRIEF.md:306`); prose cannot be validated for unreachable states or dangling targets. |
| C. An existing formal language such as TLA+ or a statechart format, as the model itself | Strong checking, existing tools. | Most checks needed here are graph checks that typed records already allow. The reader's learning cost is unmeasured. The brief's rule to buy before build (`BRIEF.md:122`) needs an unmet need named: the need is claim-level evidence on every element, which these formats do not carry, so the records stay the source of truth. A pinned tool can meet `BRIEF.md:103`, so setup is not the objection. |
| **D. Typed records now, with an export to a checker later (chosen as the extension of A)** | Keeps A's mechanical validation and evidence on every element, and leaves a path to a model checker for properties that graph checks cannot see. | The export is not built in M1 and its mapping is unspecified. Building it needs a named property that the graph rules miss; none is known yet. |

Choice: A2-A, with the export of A2-D left as a later option and no export built in M1.

### A3. When claims are checked

| Option | Gives | Costs |
|---|---|---|
| **A. At the boundary, by code, before publishing, with an independent verifier on top for the judgments of [§5.5](#55-what-the-verifier-owns) (chosen for review)** | Matches v0: rules have teeth. Findings are reproducible. | Extra work for every record. |
| B. A verifier agent reads the finished study, alone | Catches meaning errors that code cannot. | Alone it would be model judgment where a mechanism is possible. |

### A4. Running the studied system

| Option | Gives | Costs |
|---|---|---|
| **A. Run it in scripted scenarios, inside containment and within caps (chosen for review)** | Observed claims; reconciliation of docs against behavior (`BRIEF.md:296-300`). | Spend, and the work of building containment first. |
| B. Read only | No spend, no running of third-party code. | Everything stays `read`. It cannot find divergences. It is the explicit downgrade for a system that cannot run (`BRIEF.md:110`), and is the default to avoid, not forbidden. |

Where the options are marked "chosen for review", the choice is the author's proposal and is exactly what
the re-check ([§9](#9-review-gate)) is asked to challenge.

## 7. Reversal cost

| Decision | Cost to reverse | Why |
|---|---|---|
| New crate (A1-A) | Low before build; moderate after. | Before build only this document changes. After build, moving the code into another crate touches the architecture table and its test. Records do not move, because they are append-only. |
| Typed model records (A2-A) | Moderate to high once studies are published. | Records are written once and never edited. Changing the shape means a new schema version, and old studies stay readable under the old one. |
| Evidence and mark rules, non-waivable | High. | Every published study depends on them, and loosening them undermines the claim that the study is evidenced. Tightening them later would invalidate earlier studies. |
| Waivers for model rules | Low. | A rule class changes in code and the change is reviewed. |
| Validation gate before publish | Low. | It is a check in code; removing it needs only a change, but the change would itself be reviewed. |
| Containment in the run step | Moderate. | It adds a boundary and an environment allow-list to build; removing it would reopen v0's residual and would be reviewed. |
| Running third-party systems | Low per run, ongoing in cost. | It can be turned off by configuration; observed claims then become `read`. |
| Parallel regions and fan-out left out of the first schema | Moderate. | A new schema version; published studies stay readable under the old one. |

These are estimates by the author of this document, not measurements.

## 8. Out of scope

- Creating the `athena-cap-study` crate, any Rust code, tests, or `Cargo.toml` changes.
- Reading, running, or modeling firstmate, and any statement about what firstmate does.
- Pinning a SHA: the pin is made at study time, not now.
- Studying Hermes, Athena itself, or milestones M2–M9 (`BRIEF.md:306-314`).
- Changing governance files other than `docs/decisions.md` and `CHANGELOG.md`, which this revision updates
  with D64 and a changelog entry. `docs/architecture.md` is updated when the crate is built.
- Cost or token figures for a study; none have been measured.

**Required later design: publishing to Pages.** The design of the explainer page, the replay view, and the
publish step is **a required part of M1** (`BRIEF.md:395`), not an option. It is written and reviewed
**before the build job is asked for**, because two rules depend on it: the renderer's template for "Inferred
shown as stated" ([§5.4](#54-validation-rules)) and the replay view ([§2](#2-the-m1-method)). Until it
exists, this capability is not M1 done.

## 9. Review gate

This document is a design. **Nothing described here will be built until an independent review of it is
complete.** "Independent" means the reviewer shares no context with the author, as the v0 reviews did
([`reviews/`](reviews/)). The review is written down in `docs/design/reviews/`, and its findings are answered
before a build job is asked for.

Review 1 is complete and its findings are answered in
[Appendix A](#appendix-a-response-to-review-1). Because the answers changed the design's rules, model,
and run step, **a re-check of this revised design (review 2) is required before building.** The re-check
covers at least the changes marked in the appendix, and it follows the checks below.

The review must check:

1. **The brief is met.** Each of M1 (`BRIEF.md:306`), the evidence rules (`BRIEF.md:320-321`), and the
   separation of reusable from system-specific (`BRIEF.md:277`) is covered, and nothing the brief requires
   of M1 is missing.
2. **No claim about firstmate.** The document asserts nothing about firstmate, and every example is
   labeled synthetic.
3. **The evidence rules close the loop.** A claim cannot be published without a matching source and mark;
   an inferred rationale cannot be shown as stated. Look for a path that avoids [§5.4](#54-validation-rules).
4. **The model fits.** The elements of [§5.1](#51-elements) can express failures, retries, and concurrency
   for a system that is not a simple queue, or the limits are named. Name any case that does not fit.
5. **The validation rules are mechanical.** Each rule in [§5.4](#54-validation-rules) can be written as
   code with a planted-fault test; the judgments are listed in [§5.5](#55-what-the-verifier-owns).
6. **The alternatives are fair.** Each rejected option in [§6](#6-considered-alternatives) is described
   without a straw man, and the reversal costs in [§7](#7-reversal-cost) are plausible.
7. **Risk.** The handling of running untrusted third-party code, spend, and secrets
   ([§2.1](#21-the-run-step-containment-redaction-and-caps)) is adequate, or the gap is named.
8. **Scope of this change.** Only this document, the decision log, and the changelog changed.

A review finding of blocker or major severity returns the design to its author. Only a design that passes
review is scheduled for building.

## Appendix A: response to review 1

Every finding of [review 1](reviews/study-capability-m1-review-1.md) is listed with its disposition.
"Accepted" means the change the review proposed was made, in the section shown. Anything that differs from the
proposal is said. The review file is unchanged.

### Findings

| Finding | Disposition | Where |
|---|---|---|
| **M1** A waiver can switch off any rule | **Accepted.** The six evidence rules are non-waivable, and so are the quote, pin, and card-completeness rules added by other findings. Model rules can be waived only by a record that names the rule and ids, carries the user's reason, and cites a user intervention; the page shows the text. | [§3](#3-evidence-rules), [§5.4](#54-validation-rules), [§7](#7-reversal-cost) |
| **M2** Citations and runs can avoid the pin | **Accepted.** `study.run/1` gains `sha`; `study.pin/1` gains a tag; commit, pull-request, run-id and timestamp rules added; `both` needs a file and a run source. A pull-request citation must carry its head or merge SHA, since the identifier alone cannot be checked offline. | [§2](#2-the-m1-method), [§3.1](#31-claims), [§4](#4-data-model), [§5.4](#54-validation-rules) |
| **M3** Four rules need judgment | **Accepted** as structural checks for all four (`contradicts` link, `quote` field, provenance-only `authors-say` field, typed `Quantity`), plus a list of the remaining judgments as the verifier's duties. | [§3.2](#32-rationale-cards), [§3.3](#33-numbers), [§5.4](#54-validation-rules), [§5.5](#55-what-the-verifier-owns) |
| **M4** Model cannot express several agents, fan-out, or model-decided transitions | **Accepted in part.** `evaluated-by`, unhandled failures, what changes on a retry, the no-answer rule for a person, and declared counters are added. Parallel regions and fan-out are **declined for M1** and named as limits: the review offered extending now or limiting; limiting is the smaller change, and the first study's needs are not yet known. | [§5.1](#51-elements), [§5.2](#52-why-transitions-carry-trigger-and-guard), [§7](#7-reversal-cost) |
| **M5** The run step needs containment | **Accepted:** containment is built into this capability, with an environment allow-list, a scratch home, a boundary named in each run record, and no run without one. | [§2.1](#21-the-run-step-containment-redaction-and-caps), [§7](#7-reversal-cost) |
| **M6** Output redaction and uncovered spend | **Accepted.** Redaction runs before storage. Wall-clock and process limits come from Athena's kill switch; model spend is enforced by a provider-side limit on the credential and read back; if neither is possible, the run does not start. | [§2.1](#21-the-run-step-containment-redaction-and-caps), [§4](#4-data-model) |
| **M7** Run record not observable or replayable | **Accepted in part.** All streams of `BRIEF.md:298` are listed with a status (`native`, `added`, `absent`); missing ones are findings, not a requirement. A replay step is added. | [§2](#2-the-m1-method), [§4](#4-data-model) |
| **m1** Pages live is part of M1's done | **Accepted.** The Pages design is named a required part of M1 and is written and reviewed before the build job. | [§1](#1-goal), [§8](#8-out-of-scope) |
| **m2** Example disagrees with the model | **Accepted.** Claim c2 and §5.3 now agree: three attempts, two requeues; `attempts` is declared as a counter, counted at entry to `running`. A retry edge is defined by reachability, not order. A rule for undeclared counters is added. | [§4](#4-data-model), [§5.1](#51-elements), [§5.3](#53-synthetic-example), [§5.4](#54-validation-rules) |
| **m3** No completeness rule for cards | **Accepted.** An "Incomplete card" rule is added, as an evidence rule. | [§3.2](#32-rationale-cards), [§5.4](#54-validation-rules) |
| **m4** "Meant to exercise" is not "exercised" | **Accepted.** Each run lists the transitions it observed, and "Unexercised path" uses that list. Scenarios are written by a separate invocation that does not see the reader's claims; the verifier checks it. | [§2](#2-the-m1-method), [§4](#4-data-model), [§5.4](#54-validation-rules), [§5.5](#55-what-the-verifier-owns) |
| **m5** Alternatives argued from the wrong ground | **Accepted.** A2-C is rejected on the unmet need named under the buy-before-build rule, not on setup. A2-D, typed records now with an export later, is added and recorded as a later option. A4-B is the explicit downgrade, a default to avoid. A1-C's benefit is stated in full. | [§6](#6-considered-alternatives) |
| **m6** Run record omits cap, kill switch, blocker | **Accepted.** The run record holds them, and a `could-not-run` finding holds a blocker. "Unexercised path" requires such a finding or a `no-observation` finding, not free text. | [§2.1](#21-the-run-step-containment-redaction-and-caps), [§4](#4-data-model), [§5.4](#54-validation-rules) |
| **m7** Prompt injection through the studied repository | **Accepted.** The limit is stated: a read-only role bounds the harm to wrong claims. The `quote` check and the verifier's duty cover it; instruction-like text in a cited source is a finding. | [§5.5](#55-what-the-verifier-owns) |
| **m8** Fresh context and gap log | **Accepted.** Code composes each study's context from that study's records only; a `study.gap/1` record and re-check are added. | [§2](#2-the-m1-method), [§4](#4-data-model) |
| **n1** A3's choice sits outside the table | **Accepted.** The choice is in the bold row. | [§6](#6-considered-alternatives) |
| **n2** Name the commit when the BRIEF line numbers change | **Declined.** Athena's rule is that design text never pins a commit hash, since the hash cannot be confirmed in verification. The header already says how to re-find each passage by its quoted words. | header |

### The review's eight decisions

All eight were settled by the author on the user's instruction to decide, using `BRIEF.md` where it speaks
and the more conservative option where it does not. None is open. They are recorded as D64 in
[`docs/decisions.md`](../decisions.md).

| # | Decision | Settled as | Reason |
|---|---|---|---|
| 1 | M1 waivers | Evidence rules are non-waivable. Model rules may be waived only with a user-written reason. | The brief makes sourced, honestly labeled claims a standard (`BRIEF.md:320-321`, `BRIEF.md:123`); a user reason on a graph rule loses nothing the brief requires. |
| 2 | M5 containment | Built into this capability; no run without a boundary. | `BRIEF.md:110` permits an explicit downgrade, and v0 §16.2 ties containment to this job. It is the safer of the two options; the cost is build work. |
| 3 | M6 run caps | The cap must be enforced before the run starts (Athena's kill switch, and a provider-side limit on the credential). Cost recorded afterwards is never accepted as the cap. | `BRIEF.md:111` asks for a per-run cap, not a record of spend. |
| 4 | M3 judgment rules | Rewritten as structural checks wherever a mechanism exists; the rest relabeled as verifier duties ([§5.5](#55-what-the-verifier-owns)). | The review's own proposal, and both halves apply: the first is mechanism over judgment, and the rest cannot be code. |
| 5 | M4 model scope | Extend for model-decided guards, unhandled failures, retry changes, and unanswered people; limit parallel regions and fan-out and name the limit. | Smaller change; the first study's needs are unknown; a later schema version leaves old studies readable. |
| 6 | M7 observability | All streams listed per run; native ones required for the claims that use them; missing ones are findings. | `BRIEF.md:298` says the native-versus-added split is itself a finding, and a system may not expose every stream. |
| 7 | m1 Pages design timing | Written and reviewed before the build job. | Two rules depend on the renderer and the replay view; the brief's M1 is not done without it. |
| 8 | m5 alternatives | Record "typed records now, export to a checker later" as A2-D. | It costs a table row, and the review found the rejection of A2-C argued from the wrong ground. |
