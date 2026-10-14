# Socratic plan review

Status: design, for review. Nothing here is built. No code, `athena.toml` or `defaults.toml` change is part of it.

Marks used below: **(read)** means I read it in this repository in this job; **(reported)** means it comes from the
job request and its records were not available to me (the `records/` directory is absent from the working copy);
**(design)** means a proposal; **(estimate)** means a number I did not measure.

## 1. What this is for

`BRIEF.md` has an addendum, "2026-10-14: Socratic questions": "Can we add a Socratic question generator or
something?" **(read: `BRIEF.md:515-517`)**. It builds on the addendum "2026-10-13: Reconstructing job state, and
retros over the record" (`BRIEF.md:488-492`), which says: "it's good questions that generate good defaults or
better assumptions that we need to bake in as world understanding."

This design adds one step between the reader's reading and dispatch: **plan review**. Athena asks the plan a fixed
set of questions, answers as many as it can cheaply, and asks the user at most two. It serves these requirements:

| Requirement | Where | What the plan review does for it |
|---|---|---|
| A1, pushes back, open questions (`BRIEF.md:154-159`) | line 158 | A question whose answer contradicts the plan becomes a pushback with evidence, and the exchange is recorded |
| A2-4, mispredictions update defaults, estimates and routing (`BRIEF.md:171`) | line 171 | A failed plan traces to a question; repeated answers become defaults proposals; question weights move |
| A3, lessons from corrections turned into context, checks and examples (`BRIEF.md:180`) | line 180 | A question's answer is context for the briefs; a recurring question becomes a code check or an example |
| A6-4, Jev for rubric checks and gates (`BRIEF.md:209-217`) | line 209 | Each question is a typed question Jev can answer with a probability |
| 2026-10-13 addendum | `BRIEF.md:491` | The questions are the "good questions that generate good defaults" |

Today the reader may set `question` or `decline_reason` in a reading (`docs/records.md`, `athena.reading/1`
**(read)**), and the map says only that "a reading can ask or decline" (`docs/design/requirements-map.md`, A1-3
**(read)**). Nothing makes the reader ask the same questions each time, so what gets asked depends on the reader's
mood on the day.

## 2. Where it sits

```
request -> reading (reader model) -> PLAN REVIEW -> user accepts -> dispatch (athena go)
                                        |
                       six fixed questions, answered by the ladder (§4)
                                        |
                  question + answer + source recorded (§5), then the plan is
                  amended, or a pushback is raised, or up to two questions go to the user
```

Plan review runs on the reading after the reader has produced it and before the user is shown it. The user then
sees one reading that already carries the answers (as `assumptions` the reading already has) and at most two
open questions. Review changes the reading; it never dispatches and never edits a file in the repository.

## 3. The six fixed question kinds

The kinds are fixed so that the review is the same every time and each failure below can be compared with a kind.
Adding a seventh kind is a design change, not a prompt tweak. Each question is a typed question: a `boolean` or a
`choice` over described options, the two shapes the Jev client carries (`crates/athena-core/src/jev.rs`
**(read)**).

| Kind | Question put to the plan | Type | Cheapest answer source |
|---|---|---|---|
| K1 assumptions | "Which facts does this plan assume about the repository, the host or the user, and is each one true?" One question per assumption listed in the reading | boolean per assumption | Host or code first (for example, the host's merge flags) |
| K2 observable evidence | "For each acceptance criterion, which command or file can the verifier run or read in a fresh snapshot to see it?" | boolean per criterion: observable, or not | Code: a criterion marked `behavior` must name a command the verifier's allowed list can run (`verifier_commands` in `athena.toml`) |
| K3 file boundaries | "Does any deliverable touch a file the user owns, a governance file, or a file outside the stated scope? Is every file the repository requires for this kind of change listed?" | boolean per path | Code: owner-only and governance lists, D89's rule for `CHANGELOG.md` |
| K4 decomposition | "Is this one change? Does it mix design with build, or two independent deliverables?" | choice: one / design-and-build / several | Code first (deliverable classes, `design` against `code`), then Jev |
| K5 scope against deliverables | "Does every criterion's wording agree with the deliverables list and the out-of-scope list?" | boolean per pair | Code: compare the lists; Jev for wording |
| K6 root cause against symptom | "Does the plan change the place where the fault arises, or the instance that surfaced? If the place lies outside scope, is it named?" | choice: root / symptom / not a fix | Records (earlier introspections of the same class), then Jev |

K6 applies the `root-cause` default (`defaults.toml`, AD-12 in the map **(read)**) to the plan and not only to the
finished change. K4 applies `small` and `designed`.

## 4. The answer ladder

Every question is answered from the cheapest source that can answer it. The ladder stops at the first rung that
gives an answer at or above its confidence bar.

**Rung 1: facts (code, records, the host).** Free or near free, and exact. Examples: `GET repos/{owner}/{repo}`
returns `allow_merge_commit`, `allow_squash_merge` and `allow_rebase_merge` (the same read D88 uses, **read** in
`docs/decisions.md`); the owner-only path list; the verifier's allowed commands; the deliverables list; earlier
introspection records of the same failure class. A rung 1 answer is recorded as `fact` with the thing read.

**Rung 2: Jev, as a typed question with a probability.** What rung 1 cannot decide is framed as a `boolean` or
`choice` and sent to Jev with the plan as the `state`. `BRIEF.md:209` makes Jev the default for any decision that
can be a typed question, and rubric checks are named in it. Jev's answer carries a probability. Design: a
probability of at least 0.85 on the top option is accepted as the answer, marked `jev` and kept in shadow
scoring; a lower value goes on to rung 3. The 0.85 is a starting value to calibrate **(estimate)**. **Fact
about the present:** Jev runs only in shadow today and nothing in Athena reads its answers to decide anything
(`crates/athena-core/src/jev.rs` header **(read)**), and its reply key is still an assumption. So until Jev's
calibration on real records exists, rung 2 answers are recorded but treated as rung 3 candidates, and the user
confirms them inside the two-question cap. This is the honest default and it must be stated in the reading.

**Rung 3: the user, capped at two.** The residue is ranked by

```
score = P(the answer changes the plan) x cost of being wrong
```

`P` is Jev's probability that the answer differs from the plan's current assumption (or the reader's own
estimate, labeled as one); cost of being wrong is read from the plan: a deliverable in a governance file or an
irreversible host action scores high, a wording choice low. The top two by score go to the user, each with the
evidence so far and a default answer that is taken if the user replies "default". Questions below the line are not
asked; they are recorded as `unasked` with the assumption Athena proceeded on, so that a later failure can trace
to them (§6). This follows `BRIEF.md:163`: a lazy user corrects in one line. It also matches the user-default
that a tired user is asked little (`BRIEF.md:474`).

A question that rung 1 shows the plan has wrong is not asked of the user: it becomes a pushback (A1) with the
evidence, and the reading is amended or the pushback is shown beside it.

## 5. Recording

Every question and answer is recorded, including those answered by facts and those not asked. Records are the
repository's existing kind (`docs/records.md`): one new schema, `athena.plan-question/1` (design), one record per
question per reading, with:

| Field | Meaning |
|---|---|
| `reading`, `kind` (K1 to K6), `text` | which reading, which kind, the question as put |
| `source` | `fact`, `jev`, `user`, or `unasked` |
| `evidence` | what was read (path, host field, record id) |
| `answer`, `probability` | the answer; Jev's probability if rung 2 |
| `rank_score`, `asked` | the rung 3 score and whether the user saw it |
| `changed_plan` | whether the plan was amended because of the answer |
| `answer_key` | a normalized form of the answer, for counting repeats |

This is the data A2 wants and `BRIEF.md:490-491` asks for: what must be extracted from the user earlier.

**Repeated identical answers feed the defaults loop as proposals.** The defaults loop already counts repeats of
interventions: one event puts a decision under review, three distinct user-originated jobs are needed before a
default is added (`docs/design/defaults-loop.md` §7.2 and the N = 3 rule at its line 332, **read**). The same rule
applies here: when the user gives the same `answer_key` to the same kind of question in three distinct jobs, a
proposal for a `defaults.toml` entry is drafted through `athena propose`. It is review-only, as D90 requires. A
fact answer repeated does not make a default; it is repeated because the repository says so. Only `user` answers count.

**Question weights.** Each question has a weight that decides its rank on rung 3 and, if two questions tie, which
goes first. When introspection (`athena.introspection/1`, AD-13 **(read)**) traces a failure to a question that was
`unasked`, or to one answered wrongly by any source, that question's weight rises by one step; a question that
was asked and answered right in a job that failed for another reason does not move. A weight falls only when a
user-reviewed proposal lowers it, never silently. Weights are a record store, not a repository file Athena edits.
The starting weights are equal; this design does not pick numbers **(design)**.

## 6. Where a failure came from

Introspection today records a cause beyond the instance. Plan review adds one field to it: **origin**, one of

- `plan`: the plan was wrong or incomplete, and the review should have caught it;
- `execution`: the plan was right and the producer did not follow it;
- `verification`: the verifier passed what it should have failed, or failed what it should have passed;
- `environment`: the host, the tools or the network changed or failed (for example a refused merge).

Only `plan` failures can be charged to a question. An `environment` failure is charged to K1 only if the
environment fact could have been read beforehand. Attribution is by a typed question to Jev in shadow, reviewed by
the user the way class labels already are (`athena review`). Filing `plan` as the origin of everything would make
the weights meaningless, so the origin carries the evidence cited.

### Honest acceptance

A reading is accepted only by a reply from the user to a prompt that showed the reading and its open questions.
`athena.job/1` has `accepted_by` of `user` or `flag` for `--yes` (`docs/records.md` **(read)**). Design: add
`unprompted` for an `athena go` that ran a job with no prompt shown, and then define:

- `user`: the user was shown this reading with its recorded answers and replied. This is the only acceptance that
  counts as the user's.
- `flag` and `unprompted`: the user did not answer. A bare `athena go` on an existing proposal, with no
  prompt, is `unprompted`. It does not count as an answer to any open question, and each such question is
  recorded as `unasked` with the assumption proceeded on.
- A job accepted by `flag` or `unprompted` that fails is attributed with that fact beside the origin, so the
  loop does not read silence as confirmation.

## 7. Grounding in today's failures

For each failure, the question that would have caught it. Sources: `docs/decisions.md` and
`docs/design/defaults-loop.md` are **(read)**; job ids with no record available here are **(reported)** and are
cited through the document that mentions them.

| Failure | Source | Kind and question | Rung | Origin |
|---|---|---|---|---|
| D87: assumed merge commits were allowed; the host refused with HTTP 405, and D88 reversed the landing shape | `docs/decisions.md` D87, D88; `docs/design/decision-sets.md` §1, worked example (read) | K1: "Does this repository allow merge commits?" | 1: the host flags (`allow_merge_commit` false is the host's report, quoted in D88) | environment, but charged to plan: the fact was readable beforehand |
| 9gwv: a criterion the verifier could not observe | job request (reported); not in a document I could read | K2: "Which command can the verifier run in a fresh snapshot to see this criterion?" | 1: compare the criterion against `verifier_commands` and the snapshot | plan |
| 16zf: a plan to edit the owner-only `BRIEF.md` | `docs/design/defaults-loop.md` §3.6 row C5 (read) | K3: "Does any deliverable name an owner-only path?" | 1: the owner-only list. That document says it did not check whether a guard exists today | plan |
| nem8: design bundled with build | `docs/design/defaults-loop.md` §3.6 row C6 names the class without the job id (read); the id is from the request (reported) | K4: "Is this one change, or design and build together?" | 1: classes in the deliverables (`design` and `code`) then 2 | plan |
| 8qkc and m1bs: `CHANGELOG.md` left out because scope came from criteria wording | `docs/design/defaults-loop.md` §3.6 row C1; D89 (read) | K5, and K3: "Does the deliverables list match what the repository requires and what the criteria say?" | 1: D89 now adds `CHANGELOG.md` in code, so this question is already answered by a code check | plan |
| Reader-budget retries 69gf, t1y6, 3p3y, jk8v: each retry patched the last finding | `docs/design/defaults-loop.md` §3.6 row C2; D89 (read); the reader budget of 1,800 is D25 (read) | K6: "Does the retry change the place the fault arises, or only the last finding?" and K1: "Is the always-loaded set under its budget?" | 1: the live token counter against the budget, and the earlier introspection of the class | plan (and execution of the retries) |
| PR #59: the claim that the work was on a "real repo" | job request (reported); I found no record of it here | K1: "Which claim about the repository is the plan resting on, and where is it observed?" The claim is an assumption stated as fact | 1: read the repository or host; if only the user's word exists, mark it `reported`, as D87 and D88 do for their sources | plan |

Notes on honesty: for 9gwv and PR #59 I could not read a record, and what is above is the question that would have
applied to the failure as it was described to me. The four retries are counted as the defaults-loop document counts them;
the review of that document marks the count of four as unverifiable (`docs/design/reviews/defaults-loop-review-1.md`
risk row R2 **(read)**). Whether each question would in fact have caught its failure is the claim to test in the
backtest of §10.

## 8. Alternatives

1. **Ask the user everything the reader is unsure about.** Rejected: it is the cost the user is trying to avoid
   (`BRIEF.md:474`), and it spends the scarcest context (`BRIEF.md:206`).
2. **A free-form Socratic generator.** The addendum's wording points to this. Rejected as the base: nothing fixes
   which questions are asked, so a failure cannot be charged to one and weights have nothing to attach to. Free
   questions may be added later as a seventh, labeled kind.
3. **Only code checks, no questions.** Rejected as the whole answer: D89 and the owner-only check cover K3 and K5,
   but K1, K4 and K6 are judgments. Code checks are rung 1 of this design, so the idea is kept in the ladder.
4. **Jev decides alone.** Rejected today: Jev is in shadow with no calibration on real records (`jev.rs` and the A6-4
   row of the requirements map, **read**). It may take rung 2 once `athena score-jev` shows it calibrated.
5. **Review after dispatch, in the verifier.** Rejected: a wrong plan has then already cost a producer run (up to
   $3.00, `athena.toml` `producer_usd`, read).
6. **A longer reader prompt.** Rejected: D89 records that a prompt line put the reader over its 1,800-token
   budget, and a prompt line is not enforced.

## 9. Cost per job and the reader's token budget

**Measured caps (read, `athena.toml` `[budget]`):** reader $0.50, producer $3.00, verifier $2.00, job $8.00, at
most 5 readings.

**Cost of the review (estimate, not measured):**

| Part | Estimate | Basis |
|---|---|---|
| Rung 1 facts | $0, plus seconds of wall time | local reads and one host call |
| Rung 2 Jev | a few cents per reading, for about 6 to 12 short typed questions in one call | Jev takes several named questions in one `state` request (`jev.rs` header, read); the price per call was not measured by me |
| Rung 3 user | at most two questions, and a short reply each | the cap |
| Reader model | none extra if the review runs in code on the reading, so the reader's $0.50 cap is unchanged | design choice |

The review should repay itself: a plan that is wrong fails after a producer and verifier run, so up to
$3.00 + $2.00 (the caps) is at stake against a few cents (estimate). The measurement to make first is the
actual cost on the backtest of §10.

**Reader's token budget.** The reader's whole always-loaded set is 1,800 tokens (D25 and
`BudgetSet::Reader` in `crates/athena-context/src/lib.rs` per the defaults-loop review, **read**; the producer's
is 1,100 and the verifier's 1,300). The set is full enough that a line in `reader.md` already overran it once
(D89). So:

- The six kinds are **not** put in the reader prompt. They live in code and run after the reader.
- The reading carries the answers as `assumptions` and `question`, fields it already has. No prompt text grows.
- If a short rendering of the kinds must reach the producer or verifier, it is paid for under D25's rule that every
  addition is paid for: something comparable leaves, and the live counter, not the byte estimate, decides.
- Plan-review text sent to Jev does not count against any role's budget, since Jev is not one of those roles.

## 10. Build order, if accepted

1. **Backtest, read-only, no Jev.** Run rung 1 checks over the past readings in `records/` and report, for each
   failure of §7, whether its question would have fired. This tests the claim of §7. It needs `records/`, which
   I did not have.
2. Record `athena.plan-question/1`, `origin` and the `unprompted` acceptance value.
3. Rung 3 ranking and the cap, with the user's replies recorded.
4. Rung 2 in shadow, scored with `athena score-jev`; promoted only when calibrated.
5. Weights, and repeated-answer proposals through `athena propose`.

## 11. Open questions for the reviewer

1. Is 0.85 the right bar for accepting a Jev answer? It is an estimate.
2. Should `unprompted` be a new `accepted_by` value, or a separate field? A new value changes a record schema.
3. Are six kinds enough? The cited failures each map to one, which is a weak test, since the kinds were chosen
   with those failures in view.
4. Whether a failure traced to a question should raise the weight of the kind or of the specific question text.
