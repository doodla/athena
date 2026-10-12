# The studying capability, milestone M1: design

> **Status:** proposed. **Design only.** This document describes a capability that does not exist. No crate,
> code, test, or configuration has been written for it, and nothing will be built until an independent
> review of this document is complete ([§9](#9-review-gate)).
> **Requirements source:** [`BRIEF.md`](../../BRIEF.md). Line numbers below refer to it as of this change.
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
| 1. Pin | Resolve the studied repository's reference to a full commit SHA and record it with the date. All later steps read that SHA only, from a detached snapshot, never the live default branch. | `pin` record |
| 2. Read | Agents read the pinned tree and write **read claims**: each one cites path, line range, and SHA. The reader's role is read-only. | claims marked `read` |
| 3. Run | Code runs the studied system, within spend caps, on a scripted scenario that exercises success, failure, retry, and concurrency, and captures what happened. Each run has a recorded configuration. | claims marked `observed`, plus run records |
| 4. Model | Claims are assembled into the work-handling model ([§5](#5-the-work-handling-model-and-its-validation)). Where a read claim and an observed claim disagree, the divergence is a recorded finding, not a silent choice (`BRIEF.md:300`). | `model` record |
| 5. Publish | Code validates the whole study; only a study that passes validation and an independent verification is rendered as the explainer page. | validated study, then page |

Rules for the steps:

- **Pin before read.** A step 2 claim whose SHA differs from the pin is invalid.
- **Run is separate from read.** A claim can be both `read` and `observed` when both sources agree. A claim
  that was only read says so, so the reader can tell what has been exercised (`BRIEF.md:320`).
- **Spend and safety.** Runs inherit v0's invocation caps and kill switch. Running a third-party system
  runs its code with the user's privileges, the residual stated in `athena-v0.md` §16. This capability does
  not add containment; the run step must say so in each run record.
- **Publishing is the last step**, and what it publishes is decided in a later design ([§8](#8-out-of-scope)).

## 3. Evidence rules

These rules are the capability's core, and each is enforced by validation ([§5.4](#54-validation-rules)),
not requested in prose.

### 3.1 Claims

Every claim carries a **source**:

- for a file: path, start line, end line, and the pinned SHA;
- for history: a commit SHA or a pull request identifier;
- for a run: a run id and a timestamp.

Every claim is marked **`read`** (learned from reading the source), **`observed`** (learned from a run), or
**`both`**. A claim with no source, or with a mark that does not match its source kind (a `read` claim
citing a run, an `observed` claim citing only a file), is invalid.

### 3.2 Rationale cards

A rationale card records why a significant design decision was made. It carries:

- the decision;
- the alternatives considered;
- the tradeoff;
- the reversal cost;
- a **provenance**: `stated`, `evidenced`, or `inferred`.

The provenance means:

| Provenance | Meaning | Required support |
|---|---|---|
| `stated` | The authors say why, in a source Athena can cite. | A cited source containing the statement. |
| `evidenced` | Not stated, but the sources and runs show it. | At least one cited claim that supports each part of the rationale. |
| `inferred` | Athena's own reconstruction. | An explicit note that it is inferred. |

Only `stated` cards may be presented in text as what the authors said. A card that is `inferred` or
`evidenced` is displayed with its label, in the same view as the claim, and the rendered text must not use
wording reserved for statements (such as "the authors chose X because"). Validation checks this
([§5.4](#54-validation-rules)).

### 3.3 Numbers

A number is either measured (with the run or tool that measured it) or labeled as an estimate, as in v0's
`Quantity` ([`athena-v0.md`](athena-v0.md) §5.1, item 5). A study may reuse that type.

## 4. Data model

The capability would be a crate, `athena-cap-study`, implementing the core's `Capability` seam
([`athena-v0.md`](athena-v0.md) §6.2). **The crate is not created by this change.** The model below is
prose; field names are proposals for review. Records follow v0's rules: one fact per file, written once,
typed at the boundary, with a `schema` field.

| Record | Key fields |
|---|---|
| `study.pin/1` | study id; system name; repository URL; full commit SHA; resolved-at; how the reference was resolved |
| `study.source/1` | a citation: kind (`file`, `commit`, `pull-request`, `run`); path; start line; end line; SHA or id; timestamp for runs |
| `study.claim/1` | id; text; mark (`read`, `observed`, `both`); one or more sources; the mechanism it belongs to (for M1, `work-handling`) |
| `study.run/1` | id; scenario id; recorded configuration; command line; start and end; cost as quantities; exit state; location of the captured output |
| `study.scenario/1` | id; purpose; which model elements it is meant to exercise; scripted human side |
| `study.model/1` | the work-handling model ([§5.1](#51-elements)); each element lists the claims that support it |
| `study.rationale/1` | the rationale card of [§3.2](#32-rationale-cards), plus the claims it cites |
| `study.finding/1` | a divergence between read and observed claims, or a validation waiver; severity; both claims |
| `study.validation/1` | the result of running every rule in [§5.4](#54-validation-rules): rule id, pass or fail, offending record ids |

Storage reuses v0: records are append-only under `records/`, and a correction is a new record that
references the old one. Records specific to one studied system stay together under that study's own
directory, and the reusable parts (schemas, validation, prompts) live in the capability. That is the
separation the brief asks for (`BRIEF.md:277`).

**Synthetic example** (an invented system, "Example Runner", no relation to any real one):

```text
claim c1  "A task moves from queued to running when a worker is free."
          mark: read   source: file src/queue.example:10-24 @ <pinned sha>
claim c2  "A task that times out is requeued up to three times."
          mark: both   sources: file src/retry.example:5-12 @ <pinned sha>; run r4 @ 2026-01-01T00:00Z
rationale "Retries are bounded." provenance: inferred   cites: c2
```

The values above are made up to show structure only.

## 5. The work-handling model and its validation

### 5.1 Elements

The model is a data description of how one system takes a unit of work from request to done. It has:

- **States.** Named conditions of a unit of work, each with a kind: `initial`, `active`, `waiting`,
  `terminal-success`, `terminal-failure`.
- **Transitions.** Each has a source state, a target state, a **trigger** (the event that causes it), and
  a **guard** (the condition that must hold, or `none`). A transition also lists the actor that performs
  it (a person, a process, an agent, or the clock).
- **Failure edges.** Transitions whose trigger is a failure (an error, a timeout, a refusal, a crash). Each
  names the failure it handles.
- **Retry edges.** Failure edges that return to an earlier state. Each records a **bound** (the maximum
  attempts) and a **backoff**, or says the bound is absent.
- **Concurrency.** Which units of work can be in which states at once: the shared resources, the locks or
  queues that serialize access, the limit on simultaneous units, and what happens when two units conflict.
- **Human touchpoints.** The states or transitions where a person is addressed or must act.

Every element lists its supporting claims. An element supported by no claim is invalid.

### 5.2 Why transitions carry trigger and guard

A bare arrow between two states hides what can make it fire and when it cannot. The trigger answers
"what makes this happen", the guard answers "what stops it", and the pair is what a reader and the
validator check against runs. A missing guard is written `none` explicitly, so absence is a claim, with its
own citation, not an omission.

### 5.3 Synthetic example

This model describes the invented "Example Runner" and nothing real:

```text
states      queued (initial) · running (active) · failed (terminal-failure) · done (terminal-success)
transition  queued -> running   trigger: worker-free        guard: slot < limit    actor: scheduler
transition  running -> done     trigger: exit-0             guard: none            actor: worker
transition  running -> queued   trigger: timeout            guard: attempts < 3    actor: scheduler   [retry, bound 3]
transition  running -> failed   trigger: timeout            guard: attempts == 3   actor: scheduler   [failure]
concurrency limit 4 units running; one lock per target file; conflicting units wait in queued
```

### 5.4 Validation rules

Code runs these rules over the study before publishing. A failure blocks publishing unless a `finding`
waiver exists, and a waiver is itself shown on the page.

| Rule | Fails when |
|---|---|
| Uncited claim | a claim has no source, or its mark does not match its source kind (`read` with a run, `observed` with only a file) |
| Wrong SHA | a file citation names a SHA other than the pin |
| Bad citation | a cited path or line range does not exist at the pinned SHA |
| Uncited element | a state, transition, failure edge, retry edge, or concurrency rule lists no supporting claim |
| Unreachable state | a state cannot be reached from any `initial` state by following transitions |
| Dead end | a non-terminal state has no outgoing transition |
| Dangling target | a transition names a source or target that is not a declared state, or a retry edge returns to an undeclared state |
| Missing guard | a transition has no guard field (the value `none` is allowed; absence is not) |
| Unbounded retry | a retry edge has no bound and no claim that says the bound is absent |
| Unexercised path | a failure or retry edge was never covered by a scenario and carries no note that it is read-only |
| Divergence unrecorded | a `read` claim and an `observed` claim contradict each other and no finding links them |
| Inferred shown as stated | a rationale card marked `inferred` or `evidenced` is displayed or worded as a statement by the authors |
| Stated without source | a `stated` rationale card cites no source containing the statement |
| Unsupported number | a number has neither a measurement nor an estimate label |

Each rule is checked by a test that holds one planted fault and one clean case, following the pattern in
`docs/enforcement.md`. Those tests are part of the later build, not this change.

## 6. Considered alternatives

### A1. Where the capability lives

| Option | Gives | Costs |
|---|---|---|
| **A. A new crate `athena-cap-study`, implementing the `Capability` trait (chosen for review)** | Matches the brief: studying is the first capability and the core stays unchanged (`BRIEF.md:186`). Reusable pieces and system-specific records separate cleanly. | A new crate means a new documented dependency edge and a new entry in `docs/architecture.md`. The trait may need to grow to fit a long, multi-step job that is not "a verified change". |
| B. Fold studying into `athena-cap-change` | No new crate. | Mixes two kinds of work. Breaks the brief's rule that a new kind of work is a capability, not a core change. |
| C. A standalone script collection outside the capability seam | Fastest to try. | Not reusable by the Hermes and Athena studies, no gate coverage, and unverified by the same machinery. |

### A2. How the work-handling model is represented

| Option | Gives | Costs |
|---|---|---|
| **A. Typed records: states, transitions with trigger and guard, retry bounds, concurrency (chosen for review)** | Validation is mechanical. The model can be rendered and compared across systems. | Some systems may not fit the shape well. The record types would need extending. |
| B. Prose with diagrams | Cheap to write and flexible. | The brief asks for "modeled formally as data" (`BRIEF.md:306`); prose cannot be validated for unreachable states or dangling targets. |
| C. An existing formal language such as TLA+ or a statechart format | Strong checking, existing tools. | A new toolchain in a repository whose setup is pinned and idempotent (`BRIEF.md:103`). The learning cost for a reader is high. Most checks needed here are graph checks that the typed records already allow. |

### A3. When claims are checked

| Option | Gives | Costs |
|---|---|---|
| **A. At the boundary, by code, before publishing (chosen for review)** | Matches v0: rules have teeth. Findings are reproducible. | Extra work for every record. |
| B. A verifier agent reads the finished study | Catches meaning errors that code cannot. | Alone it would be model judgment where a mechanism is possible. It is still needed on top of A, for content the rules cannot judge (for example, whether a cited line supports the claim). |

Choice: A3-A with an independent verifier on top (A3-B as a second layer, not a replacement).

### A4. Running the studied system

| Option | Gives | Costs |
|---|---|---|
| **A. Run it in scripted scenarios, within caps (chosen for review)** | Observed claims; reconciliation of docs against behavior (`BRIEF.md:296-300`). | Spend, and untrusted code running with the user's privileges. |
| B. Read only | No spend, no running of third-party code. | Everything stays `read`. It cannot find divergences. Contradicts the brief's lab requirement. |

Where the options are marked "chosen for review", the choice is the author's proposal and is exactly what
the review ([§9](#9-review-gate)) is asked to challenge.

## 7. Reversal cost

| Decision | Cost to reverse | Why |
|---|---|---|
| New crate (A1-A) | Low before build; moderate after. | Before build only this document changes. After build, moving the code into another crate touches the architecture table and its test. Records do not move, because they are append-only. |
| Typed model records (A2-A) | Moderate to high once studies are published. | Records are written once and never edited. Changing the shape means a new schema version, and old studies stay readable under the old one. |
| Evidence and mark rules | High. | Every published study depends on them, and loosening them undermines the claim that the study is evidenced. Tightening them later would invalidate earlier studies. |
| Validation gate before publish | Low. | It is a check in code; removing it needs only a change, but the change would itself be reviewed. |
| Running third-party systems | Low per run, ongoing in cost. | It can be turned off by configuration; observed claims then become `read`. |

These are estimates by the author of this document, not measurements.

## 8. Out of scope

- Creating the `athena-cap-study` crate, any Rust code, tests, or `Cargo.toml` changes.
- Reading, running, or modeling firstmate, and any statement about what firstmate does.
- Pinning a SHA: the pin is made at study time, not now.
- Publishing to Pages, or designing the explainer page.
- Studying Hermes, Athena itself, or milestones M2–M9 (`BRIEF.md:306-314`).
- Changing governance files, `docs/architecture.md`, `docs/decisions.md`, or `CHANGELOG.md`. They are updated
  when the design is reviewed and built.
- Cost or token figures for a study; none have been measured.

## 9. Review gate

This document is a design. **Nothing described here will be built until an independent review of it is
complete.** "Independent" means the reviewer shares no context with the author, as the v0 reviews did
([`reviews/`](reviews/)). The review should be written down in `docs/design/reviews/`, and its findings
answered before a build job is asked for.

The review must check:

1. **The brief is met.** Each of M1 (`BRIEF.md:306`), the evidence rules (`BRIEF.md:320-321`), and the
   separation of reusable from system-specific (`BRIEF.md:277`) is covered, and nothing the brief requires
   of M1 is missing.
2. **No claim about firstmate.** The document asserts nothing about firstmate, and every example is
   labeled synthetic.
3. **The evidence rules close the loop.** A claim cannot be published without a matching source and mark;
   an inferred rationale cannot be shown as stated. Look for a path that avoids [§5.4](#54-validation-rules).
4. **The model fits.** The elements of [§5.1](#51-elements) can express failures, retries, and concurrency
   for a system that is not a simple queue, for example one with several agents or a human in the loop. Name
   any case that does not fit.
5. **The validation rules are mechanical.** Each rule in [§5.4](#54-validation-rules) can be written as
   code with a planted-fault test; none secretly needs a judgment.
6. **The alternatives are fair.** Each rejected option in [§6](#6-considered-alternatives) is described
   without a straw man, and the reversal costs in [§7](#7-reversal-cost) are plausible.
7. **Risk.** The handling of running untrusted third-party code, spend, and secrets is adequate, or the
   gap is named.
8. **Scope of this change.** Only this document was added.

A review finding of blocker or major severity returns the design to its author. Only a design that passes
review is scheduled for building.
