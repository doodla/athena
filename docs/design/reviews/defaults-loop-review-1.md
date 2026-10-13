# Review 1 of `docs/design/defaults-loop.md`

**Reviewed:** the design as it stands in this repository (status line: "proposed (D90)"), against the records it
cites, the source, `docs/decisions.md`, `BRIEF.md`, `docs/design/athena-v0.md`, `docs/records.md`, and the way the
gateway reaches Jev as the user probed it.
**Independence:** I formed the verdict from the design, the repository and the user's probe report. No earlier
review of this design exists; D90 says none is recorded (`docs/decisions.md`, D90 Provenance).
**Method:** reading only. I ran no command against the gateway and reproduced no behavior. Claims are labelled
**(read)** when taken from the file named beside them, **(user-reported)** when they come from the user's probe
of the gateway today and cannot be reproduced here, **(design)** when they are what the design proposes, and
**(estimate)** when nobody measured them. This review does not edit the design; fixing it is a separate job.

## Verdict: accept with changes

The shape is sound: code makes every decision that has a rule and every write, proposals are review-only, Jev
starts in shadow, and `defaults.toml` stays the user's file. Its claims about the repository are almost all
right. Two things are not. First, it asks Jev to do work that the gateway's evaluate call cannot do (clustering
and drafting prose). Second, its evidence for the problem is thinner than its text suggests, and one of the four
repeats is not a repeat of a user expectation. Both can be fixed in the text; neither needs a change of
direction. The changes are listed in section 5. No Jev client should be built before F1 and F2 are settled.

## Severity scale

**High** makes a central claim false or blocks the build. **Medium** must be settled in the design or the first
build job. **Low** is a correction.

## 1. Evidence against the records it cites

**The records are absent.** `records/` is not in the snapshot I was given, as it was not in the design's
(design section 2, "Access caveat"). The design cites no record id at all. So I could not check any of the
"x4" counts against records, and neither could the design. What I could check is what the repository itself
shows.

| Repeat | What the design says | What I found | Supports the claim? |
|---|---|---|---|
| R1: change reading's scope excluded `CHANGELOG.md` | x4 (reported); D89 is the repair | D89 exists and says what the design says: change readings always list `CHANGELOG.md` and a `changelog` entry in `defaults_applied` is added by code; adding a `changelog` entry to `defaults.toml` was rejected because "the defaults model is the user's, not changed here" (read: `docs/decisions.md` D89). `defaults.toml` has no `changelog` id (read: `defaults.toml`, ids `rust` to `root-cause`) | The instance, yes. The count of four, unverifiable. |
| R2: reader set over its token budget | x4 (reported); D89 and D25 | D89 says a `reader.md` line "put the reader prompt over its 1800-token budget" (read). The reader budget is 1,800 (read: `crates/athena-context/src/lib.rs` `BudgetSet::Reader`, line 157) and `check_budget` refuses an over-budget set with "over its budget" (read: same file, lines 173-180) | The instance, yes. The count of four, unverifiable. But see F3: this is not a repeat the loop can turn into a default. |
| R3: one job bundled design, review and build | x1 visible | No record visible. `small` and `designed` exist in `defaults.toml` (read) | Nothing to check. The design says so. |
| R4: new config field and first use in one commit | not visible | No record visible; no default covers it (read: `defaults.toml`) | Nothing to check. The design says so. |

The design is candid about all of this and marks the counts **(reported)**. That is the right label. It means
the section is a plan for evidence, not evidence, and the design's own text ("A proposal would have named the
gap sooner", section 2) is a counterfactual that nothing shows.

**Other checks of cited facts (read):**

- "12 entries today; D26 said eleven": `defaults.toml` has 12 `[[default]]` entries (`rust`, `strangers`,
  `evidence`, `buy-first`, `tested`, `public`, `small`, `verified`, `designed`, `lazy-requests`, `mechanism`,
  `root-cause`), and D26 says "eleven" (`docs/decisions.md` D26). Right. D26 has not been updated; that is a
  stale decision text, not a design fault (Low).
- RM-3 and RM-4 as the roadmap's names for the gap. RM-4 "feed scored mispredictions back into `defaults.toml`"
  is quoted correctly, and the roadmap now points at this design (`docs/design/roadmap.md` lines 81-88). RM-3
  has the title "Close the intervention loop" but its content is a trend per class, an audit step and a link to
  an improvement job (lines 73-79); it is not about defaults. Citing it as naming this gap is loose (Low).
- E12 "HTTP 200 in about 0.75 s ... about $0.000012 per call": matches `docs/design/athena-v0.md` line 1733. It
  was one probe on a forecast question, and the design says so. It says nothing about which endpoint was used
  (see section 3).
- "the reference is what makes a vague forecaster show zero skill": present at `athena-v0.md` line 1208.
- `BRIEF.md` lines 25, 144 and 217 say what the design says (32K context, reach Jev through the gateway's HTTP
  API, shadow against a stronger model and escalate low-confidence calls). Right.
- D31 and the "narrows D31" claim: D31 leaves Jev out of v0 and names "the first typed decision in the flow" as
  the pull-in point (`docs/decisions.md` D31, Reversal cost). D90 is consistent with that.

## 2. Claims against the code

Confirmed (read):

- `DefaultsModel::parse` rejects duplicate ids (`DefaultsError::Duplicate`), empty statement or source
  (`DefaultsError::Empty`) and unknown fields (`deny_unknown_fields`): `crates/athena-data/src/defaults.rs`
  lines 7-47. The design's claim in section 4 is exact.
- The model has one integer `version` (`defaults.rs` line 9), and `defaults.toml` has `version = 1` (line 7).
  The design's example is consistent.
- `render()` prints one line per default with the first sentence of the statement (`defaults.rs` lines 55-72).
  The design's description of it is right.
- `InterventionCategory` has eight values (`crates/athena-data/src/records.rs` lines 121-141), and
  `athena.intervention/1` carries `what`, `why`, `class_of_work`, `category`, `job`, `source`
  (`docs/records.md` line 70). Right.
- The reader budget is 1,800 tokens and the checker refuses an over-budget set (see section 1).
- No Jev code exists: a search of `crates/` and `athena.toml` found no gateway client, no `evaluate` call and
  no Jev configuration. The design's build order (section 11) starts from nothing, which is accurate.

Refuted or not supported:

- **Statement-length budget protects the wrong thing (Medium).** Section 6 caps a proposed statement at the
  longest statement now in the model, "because the model is loaded inside the reader's token budget". The
  reader loads `render()`, which carries only the first sentence of each statement plus the id and scope
  (`defaults.rs` lines 55-72). The reader cost of a new entry is its id, its scope list and its first sentence;
  the rest of the statement costs reader tokens not at all. A long statement is a cost to the producer and
  verifier briefs, which use the full text through `get()`, and only for the defaults the reader selected. I did
  not read how those two sets are budgeted (they have budgets of 1,100 and 1,300: `athena-context/src/lib.rs`
  lines 158-159), so I cannot say whether defaults count against them. The design should budget the first
  sentence and the entry count for the reader, and say which set the full statement lands in.
- **Corrections are double counted (Medium).** A correction record carries "the `intervention` recorded for
  it" (`docs/records.md` line 62). The design treats "intervention / correction record" as separate inputs
  (section 3, diagram) and the labeled set as "all intervention and correction records ... plus reading
  corrections" (section 8.2). Each correction would be counted twice, once as itself and once as its
  intervention. Counting distinct jobs (section 5.2) hides this from N, but it still inflates the labeled set,
  the match counts in `athena status` and the cluster inputs. Also, a correction of kind `answer` (to a
  question the reader asked) is not a repeated expectation in the same way as kind `correction`
  (`crates/athena-core/src/reading.rs` lines 694-701 chooses the kind). The design should say which records
  count, and use the intervention record as the single unit.
- **The "under 1,500 tokens" figure for the rendered model (estimate, weakly based).** The design derives it
  from the 1,800 budget that holds the model "plus the reader prompt". That bounds the model's share from
  above only if the prompt is at least 300 tokens; I did not measure either. It is labelled an estimate, which
  is honest, but the building job should measure `render()` with the live counter (D25: the live measurement
  is the authority) before relying on it.

## 3. The design against how Jev is reached

All items in this section are **(user-reported)** from today's gateway probes, unless a repository file is named.

**What the user found:**

- `/v1/chat/completions` refuses Jev ("a decision model, not a language model").
- Jev is called with `POST https://ai-gateway.vercel.sh/v1/evaluate`, body `{model: 'typesafe-ai/jev', state:
  <text>, questions: {<key>: {type: 'boolean' or 'choice', instructions, criteria: {<option>: <description>}}}}`.
- The answer gives a choice, a probability per option and a confidence.
- An optional fallback model can run when confidence is below a threshold, set with
  `providerOptions.gateway.models` and `when.confidenceBelow`.

**Where the design stands against each:**

| Gateway fact | Design | Result |
|---|---|---|
| Chat completions refuses Jev | The design never names an endpoint. `BRIEF.md` line 144 says "reach it from Rust through the gateway's HTTP API" (read), and D31 lists "an HTTPS client, a credential, a driver and its fake" (read) | **Gap (F4).** A client written to the general model route would fail. The design must name the route and carry the request and response shapes. |
| Questions are `boolean` or `choice`, each with instructions and criteria per option | Step 1 and section 5.1: probability that a record matches each default and "none" | **Fits.** This is one `choice` question: options are the default ids plus `none`, each criterion is the default's statement or scope; `state` is the record's stored fields. This is the part Jev is built for. |
| The answer is a choice, probabilities and a confidence | Section 4 and 5.1 use only "the highest-probability id" and `t_match` | **Gap (F5).** The design has one number, the probability. The gateway returns a second, confidence. The user's probe shows they differ (0.90 against 0.87). The design must say which one `t_match` reads, and record both. |
| The output is a selection, not prose | Steps 3 and 5: Jev clusters the pool and drafts a statement, `applies_to` and a title | **Refuted (F1).** An evaluate call can choose among given options or say yes or no. It cannot produce a statement or a title, and it cannot cluster an unlabeled pool. See F1. |
| Fallback runs through `providerOptions.gateway.models` with `when.confidenceBelow` | Section 4: a record "between a lower and the upper threshold goes to a stronger model"; shadow mode runs the stronger model "on everything" | **Partly fits (F5).** The gateway can do the escalation, so the design's own escalation need not be built as a second client path. But the design names a lower and an upper threshold and defines only `t_match`; and the fallback only runs below its threshold, so shadow comparison on everything is still a separate call. |

### F1 (High): steps 3 and 5 ask Jev for work an evaluate call cannot do

- **Evidence:** the design table (section 4) gives Jev "Which records say the same thing (clustering,
  paraphrase)" and "Draft statement, scope (`applies_to`) and title". The gateway shape (user-reported) is
  questions with typed answers: boolean, or a choice among supplied options. Jev's description from the gateway
  is "a decision model, not a language model". `BRIEF.md` line 25 says the same in the user's own words: it
  "returns choices, scores, and probabilities" (read).
- **Root cause:** the design chose Jev for every model step in the loop, on the rule "Jev answers questions that
  have the shape 'which of these, and how likely'" (section 3), and then listed two steps that are not of that
  shape. The rule is right; the table breaks it.
- **What can be done with Jev:** clustering can be turned into typed questions. For each new record, one
  `choice` question over the existing candidate clusters plus "new" (assignment), or a `boolean` question per
  pair ("do these two say the same thing"). The pair form grows with the square of the pool, so assignment is
  the practical form. That changes step 3 from "cluster the pool" to "assign each record", which also changes
  determinism: cluster membership then depends on arrival order.
- **What cannot:** drafting. Writing a statement and a title needs a language model, and the brief's own
  routing is Jev for typed decisions and a stronger model otherwise (`BRIEF.md` lines 139 and 209-217). The
  `evidence` field in section 6 ("which model produced the cluster and the draft") already allows a
  different model.
- **Needed:** move step 5 to a language model (the same stronger model used for shadow, or the producer tier),
  restate step 3 as assignment or pairwise questions, and update D90's decision text, which also gives Jev the
  clustering and drafting.

### F2 (High): the evidence does not yet justify the loop, and R2 is not a defaults repeat

- **Evidence:** section 1 of this review. Only two repeats are visible in the repository, each as one instance.
  R3 and R4 have none. R2 is a prompt edit that went over a token budget (D89); the budget is already enforced
  by code (`crates/athena-context/src/lib.rs` lines 173-180) and refuses the change.
- **Root cause:** the design counts "the user had to say it again" and "something broke the same way again" as the
  same thing. A defaults entry is a statement of what the user expects. A budget overrun is caught by a check;
  a statement such as "keep prompts under budget" would restate a rule the code enforces, add reader tokens,
  and be matched by Jev against records that have nothing to do with expectations. If R2 is in the labeled set
  as a repeat, it will teach the calibration the wrong thing.
- **R1 is closed in code (D89).** A change reading now lists `CHANGELOG.md` whatever the criteria say, so R1
  cannot recur as it did. It is useful as a test case for matching (a correction that matches no default) and
  not as proof the loop is needed now.
- **Needed:** say in section 2 that R2 is a counter-example (a repeat that belongs in code, not in defaults) and
  use it as a "none" label in the labeled set, and mark the case for the loop as unproved until the records are
  read. The building job's first task (section 10) should become a gate: if fewer than N distinct jobs show a
  repeat that a default would cover, the loop is not built.

### F3 (Medium): the promotion numbers cannot be met with the data the design describes

- **Evidence:** promotion criterion 1 asks for a skill lower bound above zero on a test part, from at least 30
  labeled records with at least 5 distinct outcomes (estimate). The data is four repeats, of which two are
  visible, split by time into calibration and test (section 8.2). A lower bound on skill from a test part of a
  handful of records would be too wide to read (an estimate, not computed).
- **Needed:** state the test-part size the lower bound needs and compute it, or say the criterion is a
  judgment by the user and drop the interval. This is a number the design should measure, not guess.

### F4 (Medium): name the route, request and response; do not leave "the gateway" implied

- **Evidence:** the user's finding that the general chat route refuses Jev (user-reported); the design's
  section 4 "through the gateway" and E12 (`athena-v0.md` line 1733), which records a probe but not its route.
- **Needed:** a short section giving the route (`POST /v1/evaluate`), the request fields (`model`, `state`,
  `questions` with `type`, `instructions`, `criteria`), the response fields (choice, probabilities,
  confidence), and the refusal on the chat route as a thing the Jev client must treat as a build error, not a
  retry. The fake Jev in build step 3 should be written to that shape. Cost and token counts per call should be
  read from the response, not estimated.

### F5 (Medium): two thresholds and one confidence, and who escalates

- **Evidence:** section 4 describes "a lower and the upper threshold"; section 5.1 defines only `t_match`; the
  gateway returns a confidence beside the probabilities and can run a fallback below a confidence threshold
  (user-reported).
- **Root cause:** the design was written before the gateway's shape was known, so it treats probability as
  the one signal and escalation as something Athena builds.
- **Needed:** define the signals (the top probability and the confidence), say which decision each one
  controls (the match threshold on one, the escalation on the other), and say whether escalation uses the
  gateway fallback or an explicit second call. Using the gateway fallback keeps the client small, but then the
  record must store which model answered, since the `evidence` field (section 6) promises that and a fallback
  answer is not Jev's. What the response says about whether the fallback ran was not part of what the user
  reported, so the building job should check it.

## 4. The probe

**What the user reported (user-reported, one probe).** The correction "A CHANGELOG.md entry in the repository's
usual style is allowed alongside the design doc" was asked against five options: `changelog`, `small`,
`root-cause`, `designed`, `none`. Jev returned `changelog` at 0.90 (`none` 0.06, `designed` 0.04), confidence
0.87, with 409 input tokens and 59 output tokens. The user's cost figure reads `/bin/bash.000017`; that
looks garbled, I do not know the intended value, and I do not use it. (The design's E12 figure of about
$0.000012 per call is a different, earlier probe: `athena-v0.md` line 1733.)

**What it shows:**

- The evaluate route works for a `choice` question of this shape, and the answer carries probabilities and a
  confidence, as the user described.
- On a correction that is a clear R1 case, with the matching option offered, Jev put most of the probability on
  that option. The three figures given sum to 1.00, so `small` and `root-cause` got about nothing (an
  inference from the sum, not a reported figure).
- One call is on the order of 400 tokens in and 60 out with five short options (user-reported). That is an
  upper-bound feel for cost per record, not a measure of it.

**What it does not show:**

- **The `changelog` option is not an existing default.** The repository's `defaults.toml` has no `changelog` id
  (read; D89 keeps the entry in code only). The probe offered an option the real matching step would not have.
  In the real loop this correction should land on `none` (or, at most, `small`), and the interesting question
  is whether it clusters with its three siblings, which this probe did not test. So the probe shows that Jev
  can pick a supplied label, which is the easy half. It says nothing about discovery of a new default.
- **It is one item.** There is no label set, no negative cases, no paraphrases and no accuracy figure. A
  single 0.90 does not show that 0.90 means "right nine times in ten". Calibration is exactly what
  the design's section 8 sets out to measure, and one probe cannot stand in for it.
- **The option set was five, not thirteen.** The real question lists 12 defaults plus `none`. The input would be
  larger, and the effect on accuracy and confidence of 13 options against 5 is unmeasured. Whether Jev's
  confidence falls as options grow is open.
- **The confidence (0.87) is below the top probability (0.90).** The probe does not say what the confidence
  measures or how it relates to the probabilities, so a threshold cannot yet be chosen from either.
- **The fallback was not exercised**, as far as the report says: nothing here shows what a fallback answer
  looks like or whether it is labelled.

## 5. Changes required for "accept"

1. **F1:** restate steps 3 and 5 of the loop and the §4 table so that Jev only answers typed questions
   (assignment of a record to a cluster, or pairwise yes/no) and a language model drafts the statement,
   `applies_to` and title. Update D90 to match.
2. **F2:** re-label R2 as a counter-example, say that R1 is closed in code, and make "the records show at least
   N distinct jobs that a default would cover" a gate before any build.
3. **F4:** add the Jev request and response shapes and the route (`POST /v1/evaluate`), with the chat route's
   refusal named.
4. **F5:** define the probability and confidence signals, say what each controls, and decide between the
   gateway fallback and an explicit second call, with the model that answered stored in `evidence`.
5. **Section 2 of this review (Medium items):** budget the first sentence and entry count for the reader rather
   than the statement length; define the unit counted as the intervention record so a correction is not counted
   twice.
6. **F3:** compute the test-part size the promotion criterion needs, or drop the interval.
7. **Low:** update D26's "eleven"; correct the RM-3 citation; measure `render()` with the live counter before
   relying on the 1,500-token estimate.

## Not checked

- `records/` (absent), so no record id, count or wording was compared with the design.
- The gateway: I made no call. Everything in section 3 and 4 is the user's report.
- How the producer and verifier sets budget the defaults they load.
- The remaining sections of the design's cited lines in `athena-v0.md` beyond the lines named above.

## Verdict line

**accept with changes** — the seven changes in section 5; none needs a change of direction, and F1 and F2 must
be settled before any build starts.

The design is not edited by this review.
