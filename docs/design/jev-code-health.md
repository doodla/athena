# Jev code-health questions, in shadow

Status: proposed design (decision D131). Design only: no Rust is written and the build jobs are not queued.

Audience: Sri, judging whether the checks below are specified well enough to build.

## 1. Purpose

Ask Jev a few typed questions about the code a landing changes (is a file doing one thing or several, did a
change add coupling, is a function in the right module, is a type becoming a god object, is logic duplicated),
and record the answers in **shadow**: stored, scored later, never read to decide anything. This is a retry of an
earlier design that was rejected for rebuilding a client, a fake and a scorer. Those exist; section 2 lists them
and section 3 marks every build job as reuse, extend or new against that list.

Two requests are named here and are **not in this repository**, so what this document says about them is assumed:

- `20261026-042558-5p65`, the deterministic code-health measures and structural pass. Assumed: it yields
  per-file measures (size, function length, import count) and can accept a request for a structural pass on a
  named file. Nothing here depends on its exact fields.
- `20261026-043137-zc0v`, the earlier version of this design. Not read; its changes were discarded.

## 2. Existing code

Read at the base commit. Every path below exists.

| What | Where | Purpose |
|---|---|---|
| Jev client for `POST /v1/evaluate` | `crates/athena-core/src/jev.rs`: `JevClient`, `Evaluate` trait, `request_body`, `parse_response`, `Question` (`Choice` with `criteria`, or `Boolean`), `Answer` (`Choice` with `probabilities` and `confidence`, or `Boolean` with `probability`), `Evaluation` (answers, usage, `cost_usd`), `JevError` | Sends one text `state` and a map of named questions in one call; retries only transient failures through `gh.rs`; never holds the key in an error; `shadow_from_env` returns a client only when `ATHENA_JEV_SHADOW=1`. The module header says Jev only runs in shadow. |
| The fake | `crates/athena-core/src/jev.rs`: `FakeJev` | Answers in the real shape without a network: first option at probability 1 and confidence 0.9, booleans at 0.5; can replay a script of results and errors; logs every `(state, questions)` call. |
| Opt-in live test | `crates/athena-core/src/jev.rs`: `live_jev_answers_both_question_kinds` (`#[ignore]`) | Calls the real gateway with one choice and one boolean question. Run with `cargo test -p athena-core -- --ignored jev`. Costs about $0.00002 (the test's own estimate). |
| Labeled set and scorer | `crates/athena-core/src/labeled.rs`: `read_inputs`, `build`, `brier`, `split`, `report`, `MIN_TEST_CASES` (10), `MAX_FALSE_MATCH_RATE`, `Athena::score_jev` | Reads introspections, class labels and failure classes under `records/`; builds labeled rows; scores Jev's stored probabilities by Brier score against two plain references on a time split; refuses a score and verdict under `MIN_TEST_CASES` test rows. It reads records and never writes. |
| The command | `athena score-jev` (`crates/athena-cli/src/main.rs`, `display.rs`); test `crates/athena-cli/tests/labeled.rs` | Prints the labeled set, what was left out, and the score or a "too few cases" notice. |

What the survey shows: the client already takes several questions per call, the fake already supports scripts,
and the scorer already knows how to refuse a verdict on too little data. The scorer's current rows are failure
classes (`introspections`, `class-labels`); it has no notion of a code-health question. That is the one gap
the design fills, by extension.

## 3. Build plan

Each job is sized for one producer run: one module or one doc, named files, and one test or demonstration.
"Reuse" means call it unchanged; "extend" means add to an existing file; "new" means a new file or type. The plan
adds only code-health questions, state builders, thresholds, shadow records and evidence scoring. It adds no
client, no fake and no scorer.

| Job | Adds | Mark | Files | Test or demonstration |
|---|---|---|---|---|
| J1 | The five typed questions (section 5) as `Question` values with their thresholds as constants, and a function that builds the one batched question map | new (uses `Question` from `jev.rs`, unchanged) | `crates/athena-core/src/code_health_jev.rs` (new), `lib.rs` (module line) | Unit test: the map serializes through `jev::request_body` with five keys of the right kinds. |
| J2 | State builders: from a landing's diff, make a bounded excerpt plus context (section 5.1) | new | same new module, or `code_health_state.rs` | Unit test on a fixture diff: output is under the byte cap, names the changed files, and truncation is marked. |
| J3 | Ask in shadow after a landing: call `Evaluate::evaluate` once with the batched map and store the result as a shadow record | extend (`jev.rs` untouched; `shadow_from_env`, `FakeJev` and `Evaluation` reused; new record type next to existing records) | `crates/athena-data/src/records.rs` (new `athena.code-health-shadow/1` record), the landing path in `crates/athena-core` | Test with `FakeJev`: one landing makes exactly one call and one record; with no `ATHENA_JEV_SHADOW`, no call; a `JevError` makes a record that holds the error and no answers. |
| J4 | Evidence scoring: from shadow records and later landing records, compute the labels of section 6.2 and turn them into rows `labeled.rs` can score | extend (`labeled.rs`: a second row source beside the failure-class one; `build`, `brier`, `split`, `report` and `MIN_TEST_CASES` reused) | `crates/athena-core/src/labeled.rs`, `crates/athena-cli/tests/labeled.rs` | Test with fixture records: `athena score-jev` reports a code-health section; under 10 test rows it says "too few labeled cases" and no verdict. |
| J5 | Effects, still shadow: a would-be status finding or structural-pass request is written into the shadow record as a field, never acted on | extend (the J3 record) | the J3 record and writer | Test: a high-confidence answer sets `would_flag`; a test asserts no other code path reads it before the exit rule is met. |

The live test `live_jev_answers_both_question_kinds` is reused as is; a second live check is not needed because
the questions use only the two kinds it covers.

## 4. Batching and cost

**Batching.** All five questions about one landing go in **one call**. `jev::request_body(state, questions)`
takes `questions: &BTreeMap<String, Question>`, so several named questions share one `state`, and the reply has
one answer per key (`parse_response` rejects a reply missing a key or with a wrong kind). Asking one call per
question would multiply the calls five-fold for the same state text.

**Price.** The per-call price in `docs/design/defaults-loop.md` is about $0.000012 (line 259, from one probe,
route not recorded, not re-measured) and about $0.00002 (line 667). Use the range **$0.000012 to about
$0.00002 per call**. A larger state than those probes may cost more; that is the first thing the shadow period
measures, from `Evaluation::cost_usd` and `usage`, which the client already returns.

**Per landing (estimates, not measured).**

| Quantity | Estimate |
|---|---|
| Calls per landing | 1 batched call; 2 if the excerpt is split by size (estimate) |
| Cost per landing | $0.000012 to $0.00004 (1 or 2 calls at the range above) |
| Landings per week | 20 to 60 (estimate; to be replaced by a count from `records/`) |
| Cost per week | about $0.0003 to $0.0024 (estimate: calls times price) |

At these figures cost is not the constraint; the constraint is the number of scored cases (section 6).

## 5. The questions

All five are asked in the one call. Each is `Choice` unless noted. Every threshold is an **estimate** set before
any measurement; the shadow period exists to revise them. A threshold is applied to the answer's
`confidence` (choice) or `probability` (boolean), as returned by the existing `Answer` type.

### 5.1 State (shared)

The `state` text holds: the landing's request text (one paragraph), the list of changed files with line counts
before and after, for each changed file its first 40 lines and the changed hunks with 5 lines of context, capped
at a total of 24,000 bytes (estimate; about 6,000 tokens). When the cap cuts content, the state ends with a line
saying which files were cut. Context for module questions adds the module's file list and each file's `//!`
header. Where the assumed deterministic measures exist (5p65), their per-file numbers for the changed files are
added as plain text; where they do not, that line is omitted.

### 5.2 One thing or several (`one_or_several`)

- **State:** the changed file's excerpt and header, plus its function list with line counts.
- **Options:** `one` (the file has one responsibility), `several` (two or more separable responsibilities),
  `unclear`.
- **Criteria:** `one`: everything in the file serves the purpose its header states. `several`: the file holds
  groups of items that share no types or state with each other and could be named separately. `unclear`: the
  excerpt is too cut to tell.
- **Confidence threshold:** `several` with confidence at least 0.8 (estimate).
- **Effect:** at or above the threshold on a file over 500 lines (estimate): a **structural-pass request** for
  that file via 5p65 (assumed interface). On a smaller file: nothing. Below the threshold: nothing. In shadow,
  the would-be effect is only recorded.

### 5.3 Added coupling (`added_coupling`, boolean)

- **State:** the diff's added `use` and import lines and calls across module lines, with the module list.
- **Options:** true or false (the probability of true).
- **Criteria:** true: the change makes a module depend on one it did not use before, or makes two modules
  depend on each other. False: new dependencies stay within the module or follow existing arrows.
- **Confidence threshold:** probability at least 0.75 (estimate).
- **Effect:** at or above: a **status finding** (a line in the landing's status, non-gating) naming the new
  dependency. Below: nothing.

### 5.4 Module placement (`module_placement`)

- **State:** each added or moved function or type, its signature, and the module headers of the file it is in
  and of the two most similar modules by name.
- **Options:** `here` (right place), `elsewhere` (belongs in another listed module), `new_module`.
- **Criteria:** `here`: it uses the file's own types or serves its header. `elsewhere`: it mostly touches the
  types of a listed module. `new_module`: no listed module fits and it is large enough (over 100 lines) to stand
  alone.
- **Confidence threshold:** `elsewhere` or `new_module` at confidence at least 0.8 (estimate).
- **Effect:** at or above: a **status finding** naming the suggested home. For `new_module` on a file already
  over 500 lines: a **structural-pass request** instead. Below: nothing.

### 5.5 God-object drift (`god_object_drift`)

- **State:** for the type the change touches: its field count and method count before and after, the list of
  its method names, and the callers' module names.
- **Options:** `fine`, `drifting` (growing in unrelated directions), `god_object` (already central to unrelated
  work).
- **Criteria:** `fine`: methods group around one purpose. `drifting`: this change adds a method unrelated to the
  existing groups. `god_object`: three or more unrelated groups and callers in four or more modules (the counts
  are estimates).
- **Confidence threshold:** `drifting` or `god_object` at confidence at least 0.8 (estimate).
- **Effect:** `god_object` at or above: a **structural-pass request** for the file. `drifting`: a **status
  finding**. Below: nothing.

### 5.6 Duplicated logic (`duplicated_logic`, boolean)

- **State:** each added function body (first 40 lines) and, from the assumed deterministic measures if present,
  the names of up to three existing functions with similar names or signatures, each with its first 40 lines.
  Without those candidates the question is still asked but the effect is capped at nothing (see below).
- **Options:** true or false.
- **Criteria:** true: an added body does the same steps as a listed function, differing only in names or
  constants. False: shared words but different steps.
- **Confidence threshold:** probability at least 0.85 (estimate), higher because a wrong "duplicate" sends
  someone looking for nothing.
- **Effect:** at or above with candidates given: a **status finding** naming both functions. Without candidates,
  or below the threshold: nothing.

## 6. Shadow period, exit rule and evidence

### 6.1 Shadow period

Jev answers are recorded and nothing reads them to decide anything (the rule already stated in the `jev.rs`
header and consistent with D127's non-gating stance). The period starts when J3 lands and runs until the exit
rule below is met, with a floor of **4 weeks** (estimate) so that later breakage and churn have time to appear.
Each landing makes one `athena.code-health-shadow/1` record: landing id, time, the base commit, the answers
(choice or probability, confidence), tokens, cost, and the J5 would-be effects.

### 6.2 Evidence scoring

An answer is scored by what happened to the file later, never by a person agreeing with it. The landing records
that already exist (job, candidate, verification and outcome records, `athena.*/1` in
`crates/athena-data/src/records.rs`) tell which files each landing changed. The exact fields are **assumed**
until J4 reads them; if the changed-file list is not in them, J4's first step is to take it from the candidate's
diff, and that is stated in J4's test.

- **Flagged file later breaks.** For a question that flagged a file (`several`, `god_object`, and so on, at or
  above its threshold), the label is *true* if within **30 days** (estimate) of the landing a later job whose
  failure or rejection record names that file is recorded; otherwise *false*. Measured from the later jobs'
  records only.
- **Flagged file later churns.** The label is *true* if the file is changed by **3 or more** later landings in
  30 days (estimate), counted against the file's median over unflagged files in the same period, so a hot file is
  not blamed for being hot.
- **Recommended split lands and reduces churn.** For a structural-pass request recorded as would-be: if a later
  landing actually splits that file (the file's line count falls by at least a third, estimate, and a new file
  appears in the same module), compare the changes per 30 days before and after the split. The label is *true*
  when churn fell. This is the evidence for the effect, not for the answer, and can be scored only for the
  few splits that happen; those few are reported by count, not by a skill figure.

### 6.3 How it feeds the scorer

`labeled.rs` scores a stored probability against a label by `brier`, on a time `split`, with two plain
references, and `report` refuses a verdict under `MIN_TEST_CASES`. J4 adds a second row source: each
scored answer becomes a row with the probability Jev gave for the option that was flagged and the 0 or 1 label
from section 6.2, so the existing `brier`, `split` and `report` run on it. The references are *always false*
(never flag) and, per question, the deterministic measure where one exists (for example file size over 500 lines
for `one_or_several`; assumed from 5p65). Skill is the improvement over the better reference, as for failure
classes. No new scorer is written.

### 6.4 Exit rule

Shadow ends, and only then may any effect be proposed for switching on by a new decision, for a question when
**all** of these hold (thresholds are estimates):

1. At least 4 weeks have passed.
2. The question has at least `MIN_TEST_CASES` scored test rows, and at least 30 (estimate) for any lower bound
   to be trusted; `report` already states when no bound was computed.
3. Skill over the better reference is above zero with the interval `report` gives, or, when no interval is
   computed, the raw per-record table is shown and a person decides.
4. The false-flag rate on rows labeled false is under `MAX_FALSE_MATCH_RATE` (10%).

A question that fails at 12 weeks (estimate) is dropped. Passing is a recommendation, not a switch: turning an
effect on is its own decision.

## 7. Fit with other work

- **Deterministic code-health measures (5p65, assumed).** Deterministic numbers (size, length, import counts)
  are the base; Jev is asked only about what numbers cannot say (is this one responsibility, is it the right
  home). The deterministic measure is also the score's reference in 6.3, so Jev earns its place only by beating
  it. A Jev "structural-pass request" would call the interface 5p65 provides; if that interface does not exist
  the effect stays a status finding. Nothing here duplicates a measure.
- **Hestia (20261025-033743-npr5, `docs/design/hestia.md`, D129).** Hestia's sweep reports, non-gating. Shadow
  records are an input she may read, to say whether the shadow is running and whether its cost is within the
  estimate. She does not read Jev's findings as facts and does not act on them. Her proposed sweep is not built.
- **Scope guard (`docs/design/scope-guard.md`, D127).** Jev's findings never gate a landing. A would-be
  structural-pass request is improvement work and so goes through the guard's value-against-cost rule like any
  other; its thrash signals (repeated restructuring of one file) also count against Jev's own recommendations
  when scoring in 6.2.

## 8. Risks and open questions

- The landing-record fields for changed files are assumed (6.2); J4 confirms them first.
- Few splits will happen in 4 weeks, so the split-reduces-churn evidence may stay below any count that supports
  a conclusion. It is reported as a count.
- The reply key for answers (`answers`) in `jev.rs` is itself marked an assumption until the live test has run;
  run `live_jev_answers_both_question_kinds` before J3 is trusted.
- A state of 24,000 bytes is larger than the probes behind the price range; cost may exceed it.
- Whether 5p65 provides a structural-pass interface is unknown here.
