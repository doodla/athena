# Roadmap

Ordered next steps for every row of the [requirements map](requirements-map.md) that is partial or not
covered, grouped into steps. The [strategy view](strategy-view.md) shows the goals the steps serve.

**How the order was chosen.** Acceleration comes first. The brief's addendum of 2026-10-13, "Athena owns
acceleration" (`BRIEF.md:494-496`), gives Athena the mandate to accelerate and to track timings and feed them
back into the system, so every later step is cheaper and quicker to do once RM-0 lands. RM-0 gathers the speed
work: the build cache, concurrent jobs, native pull-request landing and timings. The firstmate study (RM-6)
runs alongside RM-0, not after it: the brief requires the study to reach full quality (OR-1), it needs no
change to the core, and RM-0 does not wait on it. Next comes the rest of RM-4, the user-model loop, because
the addenda of 2026-10-13 (root causes, introspection, decision sets) are already partly built and their
first run over real records is the nearest evidence. After that, cheap steps that make state and evidence
visible (RM-1 to RM-3), because the brief says improvement work is pulled by evidence (OM-9) and later steps
need that evidence. Then enforcement, then the site, then the optional pieces. The order is a
proposal for the user to correct, not a commitment: Athena owns the method, and each step would be designed,
reviewed and verified like any other change.

Each step lists the row IDs it closes and a "done when" test that a reader can run or read. All 108 partial
and uncovered rows (67 partial and 41 not covered in the map of 2026-10-14) appear in exactly one step (13
steps: RM-0 to RM-7 and RM-9 to RM-13). The earlier RM-8 no longer exists as a step: its eight rows and AD-11
moved into RM-0, and the other numbers did not change. The rows AD-12 to AD-15 were added to the map on
2026-10-14: AD-12 to AD-14 sit in RM-4 and AD-15 in RM-13. Nothing here is implemented by this change.

Continuity and tidiness work (stale queue entries, leftover branches and caches, doc drift) is designed in
[Hestia](hestia.md), accepted in D129; it adds no row here.

## RM-0 Accelerate Athena: build cache, concurrent jobs, pull-request landing, timings

- **Rows:** A4-3, A5-3, A5-4, A5-5, SD-6, AD-4, AD-5, AD-9, AD-11.
- **Source:** `BRIEF.md:494-496`, the 2026-10-13 addendum "Athena owns acceleration". This step runs alongside
  RM-6 (the firstmate study), which neither waits for it nor blocks it.
- **What:**
  1. *Build cache.* The user reports that sccache shows 0 cache hits (a user report, not measured in this
     repository). D80 turned sccache on for the change and study capabilities, and the speedup was never
     measured. Outcome (D84): the cause could not be reproduced, because no job could run
     `sccache --show-stats`, so no hit count was measured and the wiring was reverted. To retry, measure
     hits and misses first, then look at per-job path differences in keys.
  2. *Parallel jobs.* The independent review of the parallel-jobs design (`docs/design/parallel-jobs.md`) is
     done: accept with changes (`docs/design/reviews/parallel-jobs-review-1.md`). The design is revised to
     answer findings F1-F8 (one process per job, landing through per-job pull requests, rules for shared
     append-only files) and awaits a second review; then build concurrent jobs. Jobs that both edit the
     shared `CHANGELOG.md` and `docs/decisions.md` collide; the revised design's section 9 says how that is
     avoided, and the build must test it.
  3. *Native pull-request landing,* from `docs/design/native-pr-landing.md`. Owned mode is built
     (`athena land --pr`, D86 to D88); what remains is the mode that leaves no Athena files in a third-party
     repository (stealth).
  3a. *CI routing.* Per-job routing of checks to CI, warm caches keyed on `Cargo.lock` and the toolchain, one
     check run per commit SHA reused by the verifier, and wall time per verified result measured before and
     after. Designed in `docs/design/ci-routing.md` (the brief's 2026-10-16 addendum); review 1 is requested,
     not done, and nothing is built.
  4. *Timings fed back.* Report invocation and job wall time per class of work, and use them to change
     routing, ordering or the cache, so a recorded timing leads to a recorded change (AD-11).
  Mechanism rows are added to `docs/enforcement.md` when the tests exist.
- **Done when:**
  1. *Build cache:* a second build of an unchanged tree in a new job reports a nonzero sccache hit count, and
     a test or recorded measurement shows the count.
  2. *Parallel jobs:* two independent jobs run at once without touching each other's records, and two jobs
     that both add an entry to `CHANGELOG.md` and `docs/decisions.md` both land, or the second is refused
     with a named message and no lost text.
  3. *Pull-request landing:* a verified job opens its own pull request on the user's command and never pushes
     non-fast-forward.
  4. *Timings:* `athena status` or a record shows wall time per class of work, and one change that cites a
     timing is recorded with its before and after numbers.

## RM-1 Make the project's state readable from the records

- **Rows:** AD-3, AD-10, A1-2, AD-6, OM-11.
- **What:** extend the derived view (`athena status`) with a requirement-to-job trace: a record that links a
  landed job to the map rows it answered, and a view that shows each row's status from those links. Add
  state reconstruction of a job at a past point from its records.
- **Done when:** a tired reader can see, from the repository alone, which rows each job moved and what stays
  open, and the strategy view's counts can be regenerated rather than counted by hand.

## RM-2 Record the operating facts that have no record yet

- **Rows:** OM-1, OM-10, OM-12, OM-15, EN-1, EN-2, EN-3, EN-4, EN-6, A11-2.
- **What:** a preflight record; a record of the active session; a record for a failed session start; a
  from-clean setup verification at each restart; the effort split between the study and Athena; restart
  cadence and resumption cost as measured values. Invocation timings per class of work moved to RM-0.
- **Done when:** each of these facts has a record type or view, a test that it is refused when malformed, and
  a row in `docs/enforcement.md` or an entry in its *Prose-only in v0* list.

## RM-3 Close the intervention loop

- **Rows:** OM-5, OM-6, OM-7, OM-8, OM-9.
- **What:** an intervention trend per class of work; an audit step that looks for interventions the log
  missed; a link from each intervention to the improvement job it caused.
- **Done when:** `athena status` shows the trend per class, and an audit job's record lists what it checked
  and found.

## RM-4 Close the user-model loop

- **Rows:** A1-1, A1-3, A1-4, A2-1, A2-4, AD-2, AD-7, AD-12, AD-13, AD-14.
- **What:** feed scored mispredictions back into `defaults.toml`, the estimates and the routing (the
  repeat-to-proposal part is designed in `docs/design/defaults-loop.md`, D90: Jev matches records to
  defaults in shadow, code validates, and proposals are review-only; introspection, `athena propose` and the
  Jev client are built, and the first run over real records, labeling and the live shadow remain); organize
  decisions into typed sets applied to briefs, each with assumptions and failure signals that put it
  under review (designed in `docs/design/decision-sets.md`, D91; the decision view and the D87 watcher are
  built, set selection is not); check that a change names and fixes its root cause (AD-12); record
  evidence-backed pushback exchanges; add taste probes with expiry; add decision-sheet artifacts for a tired
  user; propose the next big task when the study ends. The Socratic plan review (six fixed question kinds
  answered facts first, then Jev, then at most two user questions; designed in
  `docs/design/socratic-review.md`, not built) serves the pushback, defaults and root-cause parts of this step.
- **Done when:** a misprediction changes a defaults entry through a verified job that cites the score, and
  a probe answer carries an expiry that the model respects.

## RM-5 Complete enforcement and verification

- **Rows:** A3-1, A5-6, A7-1, A7-2, A7-4, A7-5, A8-1, A8-3, SD-4, SD-7, SD-8, JG-4.
- **What:** add enforcement-map rows for the addenda's rules; measure mechanism overhead; track verification
  false positives and escapes; add a highest-tier rule for high-stakes conclusions; check licenses of studied
  systems; test that an injected instruction in untrusted text is ignored; record version adoption and
  rollback; check total spend against caps over a whole run.
- **Done when:** each rule in the operating model and Part 1 is a row in `docs/enforcement.md` or is listed
  as prose-only with a reason, and every new row names tests that exist.

## RM-6 Finish the firstmate work-handling study at full quality

- **Rows:** A3-2, EN-7, S1-1, S1-3, S4-1, S5-1, S5-2.
- **Order:** runs alongside RM-0, not after it.
- **What:** register the validation rules for use by the user, finish the mechanism end to end with
  rationale cards and observed evidence, record any downgrade explicitly, and record what Athena borrowed,
  adapted or rejected.
- **Done when:** the study passes its validation and an independent verifier, with every claim marked read or
  observed and every rationale card carrying provenance.

## RM-7 Ship the first slice of the site and publish it

- **Rows:** D1-1, D1-2, D1-3, D2-1, D2-2, D2-3, D2-4, D2-5, D2-6, A4-4, EN-8, OR-1.
- **What:** a site that publishes the firstmate mechanism from data, builds from committed pinned setup, runs
  locally with one command, works on a phone, and keeps an Iris spend ledger that is checked before every
  generation. Add a second release and an adopter guide that uses the study as the worked example.
- **Done when:** the published site passes the user-facing checks at phone and desktop sizes, and the
  Iris ledger refuses a generation that would pass the $15 cap.

## RM-9 Efficiency, learning and delegation

- **Rows:** A6-3, A6-4, A9-1, A9-2, A10-1, A12-1, A12-2, A12-3, A13-1, A13-2, A13-3.
- **What:** revise routing from recorded cost per verified result and record the revision; calibrate Jev, whose
  client and scoring are built (D92, `athena score-jev`), on real records and add escalation on low
  confidence (its first use is the defaults loop, `docs/design/defaults-loop.md` §8); register automations with measured savings; define
  specialist roles with cost and yield; version skills in the repo; measure tokens per class of work; track
  learning curves; train the always-loaded contract from verbatim evidence within its budget; test a cold
  agent against the repository.
- **Done when:** one routing change is recorded with its before and after numbers, and a cold agent extends
  the work using only the repository.

## RM-10 Study Hermes and Athena, then compare

- **Rows:** S1-2, S1-4, S1-5, S2-1, S2-2, S2-3, S3-1, S3-2, S3-3, S3-4, S3-5, S4-2, S4-3, S4-4, S6-1, S7-1,
  S7-2.
- **What:** build the lab (caps, kill switch, observability, repeatable runs), run Hermes and then Athena
  with fresh contexts, tag Athena before studying it, log and fix every gap in the capability, keep the
  critique ledger, and compare the three mechanism by mechanism.
- **Done when:** each instance has the mechanisms M1 to M7 with cited, marked claims, and the comparison and
  its closing essay exist.

## RM-11 Final report and the brief's own tests

- **Rows:** D3-1, JG-1, JG-2, JG-3.
- **What:** the final report on the site; the cold-reader quiz for each instance; re-execution of two runs per
  instance from recorded configurations; the live-site suite.
- **Done when:** each of these has a passing verification record.

## RM-12 Plugin and dashboard

- **Rows:** AD-1, AD-8.
- **What:** package Athena as a Claude Code plugin first, and build the dashboard with anatomy and Jev.
- **Done when:** a user in Claude Code reaches `ask`, `go` and `status` through the plugin with no change to
  the core.

## RM-13 Keep the standards honest

- **Rows:** SD-1, SD-2, SD-5, AD-15.
- **What:** these are judgments; AD-15 asks for common-sense checks in place of rigid rules, and M20 is the
  first rule revised that way. Add a verifier severity check that each significant design has a recorded
  independent review, and that each new doc names its reader; revisit each milestone.
- **Done when:** a design without a review is a finding the verifier reports.
