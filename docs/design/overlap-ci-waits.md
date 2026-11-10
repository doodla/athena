# Overlapping CI waits with model work: start the producer at dispatch, push the candidate early

Status: design only. No code, workflow or configuration changes in this note.

Source: throughput review `20261110-191047-6szm`. It named the wait for CI before each job's verifier as the biggest
real wait (stage `checks`) and proposed running the CI waits alongside the model work. This note designs that change.
Audience: Athena maintainers, and the next throughput review, which scores the prediction in section 5.

## 1. The bottleneck

From the review (figures quoted, not re-measured here):

- About 15 minutes pass from dispatch to the producer's start, likely while CI runs on the base commit.
- About 10 minutes more pass for the candidate's CI run before the verifier can run.
- The review's predicted saving is **8.0 minutes per landed job (estimate)**, against a median of **2708.1 minutes
  per landed job** when the review ran.

Both waits are idle time for the model: nothing in the base commit's CI result is needed to write the change, and the
candidate's CI result is needed only by the checks and the verifier, not by the producer that has already finished.

The same review puts landing and rebuild at 2687.2 of the 2708.1 median minutes. `agent-and-build-slots.md` section 6.1
and `per-job-main-landing.md` section 1 show from the records that most of that is repeated landing listings of one
commit, which is a measurement effect and not wait. So the CI waits are the real wait that is left, and the 8.0 minutes
is a small share of the headline median. This note does not claim the landing stage.

## 2. Existing code

This design builds on three earlier notes. Each row says what is **kept** and what **changes**. The rule text is read
from those notes in this snapshot; the exact function names that wait for CI are not read here and are found by build
job 1 (section 7).

| What exists | Where | This design |
| --- | --- | --- |
| Agent slots and build slots: many agents at once, few builds at once; a build takes a build slot | `docs/design/agent-and-build-slots.md` sections 3 and 4 | **Kept**. The producer takes an agent slot at dispatch as before. **Changed**: the rule "a job is dispatched only after the base's CI result is known" no longer holds; the base CI fetch is a background wait that holds no slot. |
| Landing wait and rebuild measured to the newest landing record; first-landing stage as the like-for-like measure | `docs/design/agent-and-build-slots.md` section 6; `docs/design/per-job-main-landing.md` sections 1 and 5 | **Kept**. The `checks` stage is reported next to it and no headline minutes are claimed from landing. |
| One job, one commit, one pull request, cut from current main at landing; `land_pr` refuses a head other than the job's landing commit and replaces an early head | `docs/design/per-job-main-landing.md` sections 3 and 6 | **Extended**. An early push of the candidate branch is a candidate-only push for CI. It is not the landing commit and no pull request merges from it. The landing cut is still made from current main after the verifier passes. **Changed**: the rule "the branch is pushed once, at landing" becomes "pushed at the producer's end for CI, replaced by the cut at landing". |
| Ordered landing queue, `Landing::Waiting`, batched rebuild | `docs/design/continuous-landing.md` sections 3 and 4 | **Kept**. This design acts before the queue. **Changed**: nothing in the queue; a job that reaches it earlier only waits there longer if the queue was already the limit (section 6). |
| Required-check gate and merge | `docs/design/native-pr-landing.md` section 5 (via the notes above) | **Kept**. CI results used at the checks are the same required checks. |

## 3. Design

Three changes, each independent enough to build and measure on its own.

1. **Start the producer at dispatch.** When a job is dispatched, its producer starts at once in a working copy of the
   base commit. It does not wait for the base's CI result.
2. **Fetch the base CI result in the background, use it only at the checks.** At dispatch, start a background
   watcher for the base commit's CI run. Its result is stored in the job's record and read only when the checks decide
   whether a failing candidate check was already failing on the base. If the result is not ready when the checks start,
   the checks wait for it then, which is no worse than today.
3. **Push the candidate when the producer ends.** As soon as the producer reports, push the candidate to the job's
   work branch so CI starts, and run Athena's own checks and the verifier alongside that CI run. The checks read the CI
   result when it is ready. The verifier does not wait for the candidate's CI unless a criterion needs it.

Resulting order for one job:

```
dispatch ──► producer ───────────────► push candidate ──► checks ──► verifier ──► cut and land
   └──► base CI (background)  ─────────────────────────────┘ used here only
                                       └──► candidate CI ───┘ read here
```

### How this removes the bottleneck

- The roughly 15 minutes before the producer starts becomes zero model-idle time: the base CI run overlaps the
  producer's work. The producer takes longer than that CI run in most jobs, so the result is ready by the checks.
- The roughly 10 minutes of candidate CI overlaps Athena's own checks and the verifier rather than preceding them.
  The saving is bounded by the length of the checks and the verifier run, so it is smaller than 10 minutes.
- Why the review's saving is 8.0 minutes and not 25: part of the base wait is longer than the producer's work on short
  jobs, and the candidate overlap is bounded as above. The 8.0 is the review's figure and is used as stated.

## 4. Alternatives considered

| Option | Why not, or handling |
| --- | --- |
| Do nothing | Leaves the real wait in place; the model is idle for about 25 minutes per job (review figures). |
| Skip the base CI wait and never read the base result | Loses the ability to tell a candidate failure from a failure that was already on the base. Kept as the fallback in risk 1. |
| Start the producer at dispatch but still wait for base CI before the checks, with no background fetch | Same as the proposal for most jobs, but adds no information the checks lack; the background store of the result is a few lines of state. Kept as the first build step (section 7, job 1). |
| Push the candidate on every producer commit, not once at the end | More CI runs, more cost, and CI starts on code the producer may still change. Rejected for waste (risk 2). |
| Run the candidate's CI locally instead of on the host | Takes build slots (`agent-and-build-slots.md` section 4) and disk, which are the scarce resources; the host's CI is free of them. |
| Raise concurrency only | Does not shorten any one job's wait; adds load on the same slots. |

## 5. Predicted saving and how the next review scores it

**Prediction (estimate):** 8.0 minutes per landed job, taken from throughput review `20261110-191047-6szm`, against the
median of 2708.1 minutes per landed job when that review ran. It is an estimate, not a measurement made for this note.

**How the next throughput review measures it.** After the change lands, the next review reads the same newest-20
landing window and the same median method (`StageMinutes::median_of`, unchanged) and reports:

1. the **`checks` stage**: median minutes per landed job from dispatch to the end of the checks, compared with the same
   stage measured on the landings before the change by the same method. The saving is the fall in that median. The
   prediction holds if the fall is about 8.0 minutes or more per landed job;
2. the **total median minutes per landed job**, compared with 2708.1. Because landing dominates that total and is
   dominated by repeated listings (section 1), a total that moves for that reason is not credited to this change;
3. the **dispatch to producer start** gap and the **push to CI result** gap, taken from the job record timestamps, to
   show the design was in force (build job 1 records them).

Only landings dispatched after the change landed count. If the `checks` stage does not fall by about 8.0 minutes, the
review says the CI waits were not on the critical path and records that.

## 6. Risks and their handling

1. **Base CI fails after the producer ran.** The producer's work is not wasted by default: the producer worked against a
   base that is broken for reasons outside the job. Handling: the stored base result marks the failing checks as
   pre-existing, so the checks do not fail the candidate for them. If the base failure makes the job's own change
   impossible to verify (the failing check covers the changed code), the job is held, not failed, and the producer's
   work is kept for a retry once the base is repaired.
2. **Wasted work.** The producer may run on a base that is later found bad, or the candidate's early CI may be wasted if
   the checks reject the candidate first. Handling: one early push per job, only at the producer's end; a candidate
   that fails the checks stops its CI run if the host supports cancelling; the extra CI cost is one run per job
   against a model wait of about 25 minutes, so the trade is taken. The count of cancelled and wasted runs is added to
   the next review (section 5).
3. **Candidate pushed before checks pass.** An unverified branch exists on the host. Handling: it is a work branch with
   no pull request; the landing cut from section 2 is the only commit `land_pr` accepts, so nothing unverified merges.
4. **CI result and landing cut differ.** The early-pushed tree may not equal the cut tree after main moved. Handling:
   the existing re-check rule (`per-job-main-landing.md` section 6, verified content differs) applies; a different
   governed fingerprint is checked again before it lands.
5. **CI queue limits.** More jobs pushing early may lengthen the host's CI queue, so the 10 minute figure may rise.
   Handling: the review reports the push-to-result gap (section 5, item 3); if it rises, the saving falls and the
   review says so.
6. **Background watcher dies or the host does not answer.** Handling: the checks fall back to waiting for the result as
   they do today.

## 7. Out of scope and build jobs

Out of scope: any change to `crates/`, `xtask/`, `athena.toml`, `defaults.toml`, the workflows, `THROUGHPUT.md` or
`records/` in this job; the landing stage and its measurement (`per-job-main-landing.md`); the CI run length.

Ordered build jobs, each one focused change with tests:

1. **Measure.** Record dispatch, producer start, push and CI-result times per job, and find the code that waits for the
   base CI. No behavior change.
2. **Start the producer at dispatch** with the base CI fetch in the background and its result stored. Test: the producer
   starts before the base result exists; the checks read the stored result.
3. **Push the candidate at the producer's end** and read the candidate CI result at the checks. Test: the push happens
   before the checks start; no pull request is opened from it.
4. **Handle a failing base** (risk 1): pre-existing marking and the hold. Depends on: 2.

## 8. Citation duty

A change that implements this design must cite throughput review `20261110-191047-6szm` in its request text. The next
throughput review then sets the minutes it measured after that change landed against the prediction in section 5.

## Questions for the user

- Should a job whose base CI is red be held at dispatch instead of started? This note starts it, because the model work
  does not depend on the base result, but a long-red base would waste every producer run.
- Is a pushed work branch with no pull request acceptable for CI on this host, or does CI run only on pull requests?
  If the second, the early push must open a draft pull request, which changes risk 3.
