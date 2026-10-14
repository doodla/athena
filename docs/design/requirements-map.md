# Requirements map

This map lists every requirement in [`BRIEF.md`](../../BRIEF.md) as a row with an ID, says where in this
repository it is answered, and marks it covered, partial, or not covered. It is a reading aid, not a test: a
status is a judgment from reading the cited files, and a row says where to look.

Two views are built on the rows:

- [`strategy-view.md`](strategy-view.md) groups the rows by goal and shows the coverage of each goal.
- [`roadmap.md`](roadmap.md) orders the next steps for every partial and uncovered row, linked to row IDs. It
  puts acceleration (row AD-11 and the pull-request and parallel-jobs rows) first, with the firstmate study
  alongside.

## Wording

Each row's "Requirement" cell either **quotes** the brief (text inside quotation marks, copied verbatim) or
**paraphrases** it (no quotation marks). A paraphrase is shorter than the brief and may group a bullet list;
the cited lines are the authority. Where a row groups several bullets, its status is the weakest status of
what it groups.

## How to cite

**By line, only into files this change does not edit.** `BRIEF.md` is never edited by work on this map (it
changes only by dated addenda from its owner), so the "BRIEF" column cites it as `BRIEF.md:<line>` or
`BRIEF.md:<first>-<last>`. Those numbers were checked against the file when the map was written. An addendum
is appended at the end of the file, so it does not move earlier lines; an edit in the middle would, and the
numbers would then need re-checking against the quoted topic.

**By heading or quote, everywhere else.** The map, the strategy view, the roadmap, `CHANGELOG.md` and
`README.md` are all edited together, so a line number into any of them would drift as soon as the next edit
lands. The "Evidence" column therefore cites those files, and every other documentation file, by heading
(for example "`docs/records.md`, *Following a job from a commit*") or by a short quote. For
`docs/enforcement.md` the stable handle is the row number (for example "row M7"). A reader finds a cited
heading with a text search.

## Statuses

- **Covered.** The cited material states or enforces the requirement as the brief words it, and a test, a
  record type, or a written design shows how.
- **Partial.** One part is answered, or the requirement is only prose-only, proposed in a design that is not
  built, or depends on behavior that the repository cannot show. The cell says which part is missing.
- **Not covered.** Nothing in the repository answers it yet. A proposed design is named when one exists.

The evidence used: the files named in each row, read at the tree the map is written in. No command was run
to measure coverage, and no number here is a measurement.

**Re-assessment of 2026-10-14.** Every row was re-read against the work that landed after the map was first
written. A row whose status changed names, in its Evidence cell, the `CHANGELOG.md` entry (by its opening
words) and the file that justify the new status; the Status cell begins with the new status. Commit hashes
are not cited: the history was not available to this update, so the changelog entry and the file stand in for
them. The rows added for the addenda of 2026-10-13 and 2026-10-14 (AD-12 to AD-15) were judged the same way.
Rows whose status did not change were re-read and their cells updated only where the evidence moved.

## Operating model

| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| OM-1 | Run a preflight first and record each capability; anything missing becomes a blocker with a workaround plan | `BRIEF.md:53`, `BRIEF.md:95-101` | `README.md`, *Quick start* (`./setup.sh` reports whether Claude Code can authenticate) | Partial: setup checks prerequisites and credentials; no preflight record covers push access, Pages, starting a new session, studied-system model access, or Iris |
| OM-2 | Build v0 as a walking skeleton: take a request, dispatch one job, verify independently, record cost and prediction, land a traceable commit, secret scanning on | `BRIEF.md:54` | `README.md`, *What happens when you ask*; `docs/design/athena-v0.md`, *1.2 How v0 will be accepted* | Covered |
| OM-3 | The bootstrap is the only work done outside Athena, and is recorded as such | `BRIEF.md:55` | `docs/records.md`, *Closing the bootstrap* and the `jobs/bootstrap/` entry in *Layout* | Covered |
| OM-4 | After v0 is accepted, all work goes through Athena, including work on Athena | `BRIEF.md:57` | `docs/enforcement.md`, row M6 (governed content changes only through a verified landing or a declared intervention) | Covered |
| OM-5 | Ask for outcomes in a line or two, without method or restated requirements | `BRIEF.md:61-62` | `README.md`, *What happens when you ask*; `docs/design/athena-v0.md`, *3.3 Correcting in one line* | Partial: the command surface supports one-line requests; the behavior itself is not checkable from the repository |
| OM-6 | Every intervention is logged with what, why, class of work and category, and each becomes improvement work | `BRIEF.md:74` | `docs/records.md`, `athena.intervention/1`; `docs/decisions.md`, *D27* | Partial: the record and capture are built; after a rejected or failed job, a correction or an intervention Athena now writes an introspection with a proposed next request (see AD-13), but turning each into improvement work is still the user's step |
| OM-7 | Interventions are never suppressed to improve the numbers; an independent audit looks for missed ones | `BRIEF.md:75` | `docs/enforcement.md`, *Prose-only in v0* ("Interventions are never suppressed") | Partial: content-changing interventions are captured by mechanism; others depend on the user; no audit exists |
| OM-8 | The intervention rate trends down for each class of work | `BRIEF.md:76` | `docs/records.md`, `athena.score/1` (the `user_intervenes` event) | Partial: intervention and class are recorded; no per-class trend report |
| OM-9 | Improvement work on Athena is pulled by evidence, never by anticipation | `BRIEF.md:80` | `docs/decisions.md`, *D41 v0 is cut to what its ten capabilities need* | Partial: a recorded practice, not a mechanism |
| OM-10 | Track how effort splits between the study and Athena itself, and correct course if Athena work crowds out the study | `BRIEF.md:81` | none | Not covered |
| OM-11 | Every milestone leaves the repo presentable: what is done is published and verified, what is not is listed | `BRIEF.md:82` | `CHANGELOG.md`, *Unreleased*; `docs/design/athena-v0.md`, *14. What v0 leaves out* | Partial: v0 lists what it leaves out; no per-milestone check |
| OM-12 | Restart the session right after v0 is accepted, at least once per milestone, and when efficiency drops | `BRIEF.md:86` | `docs/records.md`, `athena.restart/1` | Partial: restarts are recorded; the cadence is behavior |
| OM-13 | The restart notes stay minimal; project state comes from Athena | `BRIEF.md:87` | `README.md`, *Picking up the work*; `docs/decisions.md`, *D35 The state of the work is derived from records* | Covered |
| OM-14 | Athena never sees the restart notes and knows nothing about the host it runs on | `BRIEF.md:88` | `docs/enforcement.md`, row M19 (boundary scan over tracked files) | Covered |
| OM-15 | After each restart, log its cost and what was lost, then fix the causes | `BRIEF.md:89` | `docs/records.md`, `athena.restart/1` (`cost`, `minutes`, `tokens`, `lost`) | Partial: logging is built; fixing causes is manual |

## Running environment

| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| EN-1 | Restart by starting the next session with the prompt `continue` and nothing more | `BRIEF.md:102` | `README.md`, *Commands* table (`athena restart` records a restart) | Partial: the record exists; starting a session is outside Athena by design |
| EN-2 | Every environment change is committed, idempotent and version-pinned; a fresh environment reaches a working state with one command; setup is verified from clean at every restart | `BRIEF.md:103-105` | `README.md`, *Quick start*; `docs/decisions.md`, *D23 Setup is a minimal POSIX bootstrap, then Rust* | Partial: one command and pins exist; no record of a from-clean verification at each restart |
| EN-3 | Host-only pieces stay outside Athena; everything Athena needs to build, test and run its lab lives with Athena | `BRIEF.md:106` | `docs/enforcement.md`, row M19 | Partial: build and test are covered; the lab does not exist yet |
| EN-4 | Keep a record of which session is active and since when, so two sessions never push at once | `BRIEF.md:107` | none | Not covered |
| EN-5 | A session that starts with only `continue` finds the restart notes; Athena's always-loaded contract holds nothing about who runs it or where | `BRIEF.md:108` | `docs/enforcement.md`, rows M18 and M19; `docs/design/athena-v0.md`, *12. The always-loaded contract and the role budgets* | Covered |
| EN-6 | If starting a session fails, record the blocker, keep working and retry; never skip a required restart silently | `BRIEF.md:109` | none | Not covered |
| EN-7 | If a studied system cannot run, downgrade explicitly, record the blocker and mark affected findings as having no observed evidence | `BRIEF.md:110` | `crates/athena-cap-study/src/rules.rs` (`Mark::Observed` needs a cited run; `could-not-run` findings) | Partial: the study records can express it; no real downgrade has been made |
| EN-8 | Spend is bounded: a per-run cap and a kill switch for every run, recorded, with spend tracked | `BRIEF.md:111` | `docs/enforcement.md`, rows M13 and M14 | Partial: covered for Athena's own agent runs; lab runs and the Iris cap do not exist (see D2-6) |

## Standards and boundaries

| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| SD-1 | Right, then proud: iterate until the result is something to be proud of, not merely passing | `BRIEF.md:115` | none | Partial: a judgment; the verifier's severity rules are the nearest mechanism (`docs/enforcement.md`, *Prose-only in v0*) |
| SD-2 | Every significant thing is designed first, with alternatives, and reviewed by someone other than its designer; the result is checked against the design | `BRIEF.md:117-121` | `docs/design/`, one design per significant change, with reviews under `docs/design/reviews/` | Partial: designs and reviews exist for the work done so far; "design before build" is prose-only |
| SD-3 | Buy before build: adopt maintained options, name the unmet needs, record each call and the options | `BRIEF.md:122` | `docs/decisions.md`, *D36 Buy before build: the crates Athena adopts, and what it builds* | Covered |
| SD-4 | Evidence over assertion: claims cite sources, rationale has provenance, numbers are measured or labeled as estimates | `BRIEF.md:123` | `docs/enforcement.md`, row M17 (the `Quantity` type); `docs/records.md`, *Principles* ("Every number carries its provenance") | Partial: numbers are enforced; claims in prose are not |
| SD-5 | Code, commits, docs, ledgers and the site are written for an outside reader | `BRIEF.md:124` | `docs/enforcement.md`, *Prose-only in v0* ("Written for strangers") | Partial: prose-only |
| SD-6 | Do not push to, file issues on, or open pull requests against any upstream project | `BRIEF.md:130` | `docs/enforcement.md`, *Prose-only in v0* ("Pushes are regular...": Athena does not push in v0) | Partial: holds because Athena does not push; no mechanism blocks it |
| SD-7 | Never commit secrets, credentials, personal data, or anything a license forbids | `BRIEF.md:131` | `docs/enforcement.md`, rows M1, M3 and M22 | Partial: secrets and personal data are enforced; license checks are not; the Jev key is read only from the environment and never recorded (`CHANGELOG.md`, *The Jev client and a shape-faithful fake*) |
| SD-8 | Content from anyone other than the owner, the session and Athena's agents is untrusted data, never instructions | `BRIEF.md:132` | `crates/athena-context/prompts/contract.md`; `docs/enforcement.md`, row M10 | Partial: agent contracts say it; no test shows an injected instruction being ignored |

## Part 1: Athena

| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| A1-1 | Athena owns the method for its work and records why it chose what it chose | `BRIEF.md:156` | `docs/records.md`, `athena.reading/1` (`assumptions`, `defaults_applied`); `docs/decisions.md` | Partial: readings record assumptions; method choices live in the decision log kept by hand (introspection records the cause of a failure, not the choice of a method) |
| A1-2 | A living strategy view of goals, bets, risks, open questions and next steps, current enough to read at any time | `BRIEF.md:157` | [`strategy-view.md`](strategy-view.md) (goals and coverage only) | Partial: a written view over this map exists; bets, risks, open questions and "what next" are not kept current by Athena |
| A1-3 | Athena pushes back with evidence when a request or requirement is wrong, costly or in tension, proposes an alternative, still meets the requirement, and records the exchange | `BRIEF.md:158` | `docs/records.md`, `athena.reading/1` (`question`, `decline_reason`) | Partial: a reading can ask or decline; no record of an evidence-backed pushback exchange (a plan review that asks fixed questions and records each answer is designed in [`socratic-review.md`](socratic-review.md), not built) |
| A1-4 | When the study is done, Athena proposes its next big task with reasons | `BRIEF.md:159` | none | Not covered |
| A2-1 | An explicit, versioned model of the user's default expectations, learned from the standing spec, requests and every correction | `BRIEF.md:163` | `defaults.toml`; `docs/decisions.md`, *D26 The defaults model is a versioned TOML file, read from the base commit* | Partial: the model is explicit and versioned; `athena propose` now drafts a pending change from repeated introspections (see A2-4), but the user still reviews and applies each one |
| A2-2 | Before dispatch, record checkable predictions (reading, defaults applied, output, cost, time, models, verification outcome, intervention), visible and correctable in one line | `BRIEF.md:164-170` | `docs/enforcement.md`, row M23; `docs/decisions.md`, *D07*; `README.md`, *What happens when you ask* | Covered |
| A2-3 | Predictions are scored with a proper scoring rule, with calibration tracked per class of work | `BRIEF.md:171` | `docs/decisions.md`, *D14 Brier for events...*; `docs/design/athena-v0.md`, *9.6 Calibration per class of work* | Covered |
| A2-4 | Mispredictions update the defaults model, the estimates and the routing | `BRIEF.md:171` | `CHANGELOG.md`, *Defaults proposals, step 3 of the build order*; `docs/records.md`, `athena.defaults-proposal/1`; `docs/design/defaults-loop.md`, *12. Build order, if accepted* | Partial: the defaults part has a built path, from repeated failure classes in introspections to a pending, versioned proposal that the user accepts; it starts from corrections and failures, not from prediction scores, and the estimates and the routing are not updated |
| A3-1 | Changes to Athena go through Athena; versions are adopted deliberately, can be rolled back, and carry a reason | `BRIEF.md:175` | `docs/enforcement.md`, row M6; `CHANGELOG.md`, *Unreleased* | Partial: changes are gated and logged; there is no version adoption or rollback mechanism |
| A3-2 | What Athena learns from firstmate and Hermes (borrowed, adapted, rejected) is recorded with rationale | `BRIEF.md:176` | `studies/firstmate-work-handling/rationales.json` | Partial: firstmate work handling only |
| A3-3 | Use the harness-engineering bundle as a reference; record where it shaped a decision and where Athena departs; attribute it; check its license file | `BRIEF.md:177-182` | `README.md`, *Attribution*; `docs/design/athena-v0.md`, *15. The harness-engineering bundle* | Covered |
| A4-1 | The core is separate from capabilities; a new kind of work is a new capability | `BRIEF.md:186` | `docs/architecture.md`, *Seams* ("Capabilities") | Covered |
| A4-2 | All code is Rust, modular, with documented dependency direction and no cycles; other-language code is flagged, and the few exceptions are listed with reasons | `BRIEF.md:187` | `docs/enforcement.md`, rows M20 and M21; `docs/non-rust-files.md` (*Non-Rust files*: code is flagged by extension or `#!` line, data and documents are allowed, two exceptions are listed with reasons); `CHANGELOG.md`, *The non-Rust file check (M20) now aims at its intent* | Covered: re-confirmed; the check now matches the brief's wording (code, not data) |
| A4-3 | Athena is standalone and environment-agnostic, liftable with its history intact | `BRIEF.md:188` | `docs/enforcement.md`, row M19 | Partial: nothing host-specific is referenced; adoption into an existing repository is not built (see AD-5, still partial) |
| A4-4 | Everything Athena needs, including the site build and the lab prerequisites, comes from committed, pinned, idempotent setup; one command on a clean machine | `BRIEF.md:189` | `README.md`, *Quick start*; `docs/decisions.md`, *D23* | Partial: toolchain and tools are covered; no site or lab exists |
| A5-1 | Jobs are traceable from request through prediction, owner, model, cost, verification and outcome, in machine-readable records in the repo | `BRIEF.md:193` | `docs/records.md`, *Following a job from a commit* | Covered |
| A5-2 | Every commit traces to the work that produced it | `BRIEF.md:194` | `docs/enforcement.md`, row M6 | Covered |
| A5-3 | "Nothing reaches the main branch, and nothing deploys, without passing verification." | `BRIEF.md:195` | `docs/enforcement.md`, row M7; `CHANGELOG.md`, *`athena land <job> --pr`* | Partial: landing through Athena is gated, and `athena land --pr` now merges only when the checks are green; nothing stops a push to the main branch around Athena, and nothing deploys yet |
| A5-4 | Parallel work never corrupts other work | `BRIEF.md:196` | `docs/design/athena-v0.md`, *5.5 What parallel work can and cannot do*; `docs/design/parallel-jobs.md` | Partial: records are per-job; concurrent job running is a reviewed design (`docs/design/reviews/parallel-jobs-review-1.md` and `parallel-jobs-review-2.md`, each accept with changes, the design revised after both), not built |
| A5-5 | Pushes are regular, and published history on main is never rewritten | `BRIEF.md:197` | `CHANGELOG.md`, *`athena-core::pr`, slice 1 of native pull-request landing* (push guard: the job's own branch, fast-forward only, no force option) and *`athena land --pr` works on repositories that allow only squash merges*; `docs/design/native-pr-landing.md`, *5.2 What changes in the pushes rule* | Partial: Athena now pushes, through a guard that allows only its own `athena/<job>` branch (a conditional replace of that branch only, never main), and merging is left to the host, which squashes; nothing blocks a push to main around Athena, and the guard's tests use a fake host, not a real remote |
| A5-6 | Everything committed is safe to publish: secrets scanned per commit and across history, redaction, studied-system licenses respected, repo size healthy | `BRIEF.md:198-202` | `docs/enforcement.md`, rows M1, M3, M5 and M22 | Partial: secrets, redaction and size are enforced; licenses of studied systems are not |
| A6-1 | Orchestrating contexts are the scarcest resource; they hold pointers, not dumps | `BRIEF.md:206` | `docs/enforcement.md`, row M18; `docs/decisions.md`, *D25* | Covered |
| A6-2 | Every model invocation is recorded with job, model, tokens, time and outcome | `BRIEF.md:207` | `docs/enforcement.md`, rows M14 and M15; `docs/records.md`, `athena.invocation/1` | Covered |
| A6-3 | The governing metric is cost per verified result; routing is revised from evidence at least once, recorded | `BRIEF.md:208` | `docs/records.md`, `athena.outcome/1` (`totals`); `docs/decisions.md`, *D31* | Partial: totals are recorded; no routing revision has been made |
| A6-4 | Use Jev for any decision that can be a typed question, calibrated in shadow against a stronger model, escalating on low confidence | `BRIEF.md:209-217` | `docs/decisions.md`, *D92 The Jev client rides the existing outbound wrapper*; `crates/athena-core/src/jev.rs`; `CHANGELOG.md`, *The Jev client and a shape-faithful fake* and *The labeled set and Jev scoring* (`athena score-jev`) | Partial: a typed client and a fake exist, Jev answers the failure-class question in shadow (only with `ATHENA_JEV_SHADOW=1` on the command line), and `athena score-jev` scores stored answers against labels from `athena review`; no live run or calibration on real records has been made, no other decision is a typed question, and escalation on low confidence is not built (D31 left Jev out of v0) |
| A7-1 | Everything substantive is verified before it is accepted | `BRIEF.md:221` | `docs/enforcement.md`, row M7; `crates/athena-cap-study/src/rules.rs` | Partial: changes and study validation are verified; designs, skills and claims in prose are not |
| A7-2 | Verification is independent; high-stakes conclusions are verified at the highest tier by agents that had no part in producing them | `BRIEF.md:222` | `docs/enforcement.md`, rows M8 and M9 | Partial: independence is enforced; a highest-tier rule for high stakes is not |
| A7-3 | Verification is proportional and converges: severities, and a bounded number of rounds | `BRIEF.md:223` | `docs/decisions.md`, *D13 One verification round, with severities and a verdict computed by code* | Covered |
| A7-4 | User-facing behavior is verified by exercising it, never by reading code | `BRIEF.md:224` | `docs/enforcement.md`, row M16 | Partial: behavior criteria must cite commands that ran; no site exists to exercise |
| A7-5 | Verification is tracked: cost, what it caught, false positives, escapes | `BRIEF.md:225` | `docs/records.md`, `athena.verification/1` | Partial: cost and findings are recorded; no false-positive or escape tracking |
| A8-1 | Every rule in the operating model and Part 1 is enforced, reminded at the moment it matters, or marked prose-only | `BRIEF.md:229` | `docs/enforcement.md` (the rows and *Prose-only in v0*) | Partial: v0's rules and later mechanisms through row M31 (for example the skip rule and the history check) are mapped; the rules from the addenda (root cause, introspection, decision sets) have no row, and the introspection and watcher code that now enforces some of them is not in the map |
| A8-2 | Athena keeps its own enforcement map | `BRIEF.md:230` | `docs/enforcement.md` | Covered |
| A8-3 | Every mechanism has tests showing what it blocks and allows, and its overhead is measured | `BRIEF.md:231` | `docs/enforcement.md`, intro and the Blocks and Allows columns | Partial: tests are named and checked to exist; overhead is not measured |
| A9-1 | The automation engineer builds v0, then stays on call and gets used for toil and repeat causes of interventions | `BRIEF.md:235-236` | `docs/records.md`, *Closing the bootstrap* | Partial: the v0 build is recorded; no standing role exists |
| A9-2 | Every automation is registered with what it replaced, its tests and measured savings; costly ones are retired | `BRIEF.md:237-238` | none | Not covered |
| A10-1 | Specialist roles are recorded role definitions, tracked by cost and yield, reshaped or retired from the numbers | `BRIEF.md:242-243` | none | Not covered |
| A11-1 | After any context reset, Athena resumes from the repo alone | `BRIEF.md:247` | `README.md`, *Picking up the work*; `docs/decisions.md`, *D35* | Covered |
| A11-2 | Resumption cost is measured, and each rough resumption leads to a fix | `BRIEF.md:248` | `docs/records.md`, `athena.restart/1` | Partial: cost is recorded as written by the user; no measurement or fix loop |
| A12-1 | Skills are protocols and memory packages, versioned in the repo and installed from it | `BRIEF.md:252` | none | Not covered |
| A12-2 | Skills cover how Athena operates and every capability, including studying a new system, refreshing one, changing how the site feels, and adding a capability | `BRIEF.md:253` | none | Not covered |
| A12-3 | A fresh agent with zero context can pick up, modify and extend the work using only the repo | `BRIEF.md:254` | `README.md`, *Picking up the work*; `docs/architecture.md` | Partial: the pickup path exists; it has not been tested with a cold agent |
| A13-1 | Measure tokens loaded per class of work and whether outcomes held; compression never trades away correctness | `BRIEF.md:258`, `BRIEF.md:269` | `docs/enforcement.md`, row M18 | Partial: per-role budgets are enforced; no per-class measure of outcomes |
| A13-2 | Track learning curves per class (tokens, time, error rate, interventions) and fix regressions | `BRIEF.md:259` | none | Not covered |
| A13-3 | The always-loaded contract is trained from verbatim transcript evidence, in small steps, within a fixed budget, with independent verification and memory of rejected edits | `BRIEF.md:260-268` | `docs/enforcement.md`, row M18 (the budget only) | Partial: the fixed budget exists; the training procedure does not |

## Part 2: the study

| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| S1-1 | Study firstmate, then Hermes, then Athena at a tagged release; pin instances A and B to recorded commits | `BRIEF.md:275` | `studies/firstmate-work-handling/pin.json` | Partial: firstmate is pinned; Hermes and Athena are not studied |
| S1-2 | Understand how Hermes decides what becomes a skill, models its user, and keeps skills and memory from going stale | `BRIEF.md:276` | none | Not covered |
| S1-3 | Everything reusable lives in the studying capability; everything specific to one system stays with that instance | `BRIEF.md:277` | `docs/architecture.md`, crate table (`athena-cap-study`); `studies/firstmate-work-handling/` | Partial: the split exists for the first mechanism |
| S1-4 | Hermes and Athena are each studied with a fresh context and no access to other instances' notes beyond labeled examples | `BRIEF.md:278` | none | Not covered |
| S1-5 | Gaps found are logged, fixed in the capability and re-checked against earlier instances | `BRIEF.md:279` | none | Not covered |
| S2-1 | For each instance: inventory every prompt surface and the load path with measured token costs | `BRIEF.md:285-286` | none | Not covered |
| S2-2 | For each instance: instruction design, enforcement of each rule, evolution across history, and quality | `BRIEF.md:287-290` | none | Not covered |
| S2-3 | The prompt analysis is reproducible in code | `BRIEF.md:292` | none | Not covered |
| S3-1 | Run all three systems for real within spend caps, on tasks exercising every mechanism including failure and recovery | `BRIEF.md:296` | none | Not covered |
| S3-2 | Firstmate and Hermes get identical tasks where possible; Athena gets the same mechanisms as its own kind of work | `BRIEF.md:297` | none | Not covered |
| S3-3 | Every run is observable: prompts as sent, tool calls, tokens by agent and model, timing, topology, state changes, git activity, human contact | `BRIEF.md:298` | `docs/records.md`, `athena.invocation/1` (Athena's own runs only) | Not covered: no lab; Athena's own invocation records are the nearest part |
| S3-4 | Each run is reproducible from a recorded configuration with one command, with a scripted human side, redacted and replayable | `BRIEF.md:299` | none | Not covered |
| S3-5 | Observed behavior is reconciled against what the prompts and docs say; each divergence is a finding | `BRIEF.md:300` | none | Not covered |
| S4-1 | Mechanism M1, work handling, explained for each instance and modeled formally as data | `BRIEF.md:306` | `studies/firstmate-work-handling/model.json`; `docs/design/study-capability-m1.md` | Partial: firstmate only |
| S4-2 | Mechanisms M2 to M7 (long-running sessions, interaction, memory, protocols, context and tokens, verification) for all three | `BRIEF.md:307-312` | none | Not covered |
| S4-3 | Mechanisms M8 and M9 (trust and security; observability and testing), strongly wanted | `BRIEF.md:313-314` | none | Not covered |
| S4-4 | For Athena, its own records are primary evidence | `BRIEF.md:316` | `docs/records.md` | Partial: the records exist; the Athena study has not run |
| S5-1 | Every claim cites its source and is marked read, observed or both | `BRIEF.md:320` | `crates/athena-cap-study/src/rules.rs` (evidence rules); `studies/firstmate-work-handling/claims.json` | Partial: validation core and one study; not wired into the CLI for use as a gate by the user |
| S5-2 | Every significant design decision gets a rationale card with alternatives, tradeoff, reversal cost and provenance | `BRIEF.md:321` | `studies/firstmate-work-handling/rationales.json`; `docs/decisions.md` | Partial: Athena's decision log has the shape; study rationale cards exist for one mechanism |
| S6-1 | A living, machine-readable critique ledger with severity, impact, effort, evidence and the strongest defense | `BRIEF.md:325` | none | Not covered |
| S7-1 | Map all three systems onto shared concepts and compare mechanism by mechanism, naming convergent and divergent patterns | `BRIEF.md:329-330` | none | Not covered |
| S7-2 | Close the comparison with a short essay on what the three teach | `BRIEF.md:331` | none | Not covered |

## Part 3: deliverables and order

| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| D1-1 | Athena is standalone, with tagged releases and a changelog | `BRIEF.md:337` | `CHANGELOG.md`, the `[0.1.0]` heading | Partial: one release so far |
| D1-2 | Documented for adoption: what it is, architecture, how to run work through it, how to add a capability with studying as the worked example | `BRIEF.md:338` | `README.md`; `docs/architecture.md`, *Seams* | Partial: no adopter guide for adding a capability using the study as the example |
| D1-3 | Athena carries its own evidence: jobs, predictions, interventions, ledgers, defaults, versions, restarts, specialists, enforcement map, strategy view | `BRIEF.md:339` | `docs/records.md`; `docs/enforcement.md`; `defaults.toml` | Partial: no ledgers of critiques, versions or specialists |
| D2-1 | The explainer site is published to GitHub Pages from this repo and runs locally with one command | `BRIEF.md:343` | none | Not covered |
| D2-2 | Mobile is first-class for every page, graph and animation, and stays fast | `BRIEF.md:344` | none | Not covered |
| D2-3 | Two reading layers (core and expert), a clear voice, a learning path, themes, reduced motion and accessibility basics | `BRIEF.md:345-349` | none | Not covered |
| D2-4 | Content is separate from presentation: findings as data under a shared schema, identity in design tokens plus a written feel spec | `BRIEF.md:350` | none | Not covered |
| D2-5 | A reader can see each agent's context, model and cost, enforced and requested rules, prompt evolution, work flow, a replay, memory over time, agent talk, decision provenance, and how Athena was built | `BRIEF.md:351-360` | none | Not covered |
| D2-6 | Assets come from Iris within a $15 lifetime budget, tracked in a committed ledger checked before every generation; credentials never enter the repo | `BRIEF.md:361` | none | Not covered |
| D3-1 | A final report, published on the site and verified, covering Athena, each studied system, findings, the experiment, operating numbers, what Athena learned, and its proposal for the next task | `BRIEF.md:363-389` | none | Not covered |
| OR-1 | Order: preflight and bootstrap, then v0, then the first restart, then M1 on firstmate end to end and live on Pages, then Hermes and Athena (tagged before it is studied) | `BRIEF.md:393-398` | `CHANGELOG.md`, the `[0.1.0]` heading; `studies/firstmate-work-handling/` | Partial: v0 is accepted and M1 on firstmate has study records; nothing is live on Pages |

## How the brief will judge it

These bullets test the requirements above. Rows are added here only where the judging bullet asks for
something no row above covers.

| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| JG-1 | Cold-reader agents, one per instance, take a fixed quiz using only the site, and scores are reported | `BRIEF.md:428` | none | Not covered |
| JG-2 | Two randomly chosen runs per instance re-execute from recorded configs, with nondeterminism characterized | `BRIEF.md:427` | none | Not covered |
| JG-3 | The live site passes the full user-facing suite at phone and desktop sizes, in both themes and with reduced motion | `BRIEF.md:419` | none | Not covered |
| JG-4 | The full history passes a secret scan, and spend stayed within caps | `BRIEF.md:438`, `BRIEF.md:441` | `docs/enforcement.md`, rows M3 and M13 | Partial: history scan is enforced in CI; spend is bounded per run, with no record that the whole run stayed within its caps |
| JG-5 | Everything is Rust and modular, every module has passing tests, and every exception is listed | `BRIEF.md:436` | `docs/enforcement.md`, rows M20 and M21; `docs/non-rust-files.md`, *Allow-list* | Covered: re-confirmed (see A4-2) |

## Addenda

| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| AD-1 | At the end, the user works in Claude Code or their own harness with Athena as a plugin; the first version works with Claude Code | `BRIEF.md:445-448` | `docs/decisions.md`, *D02* (a plugin was considered and not chosen for v0) | Not covered |
| AD-2 | Taste probes: easy, self-contained questions with examples that capture the user's taste and expectations, with memory decay or expiry | `BRIEF.md:450-453` | none | Not covered |
| AD-3 | A persistent place to track the project and Athena's reasoning, built on structured data | `BRIEF.md:455-457` | `README.md`, *Picking up the work* (`athena status` over the records) | Partial: a derived view of jobs exists; reasoning and project-level tracking are not kept |
| AD-4 | Athena natively handles pull requests, doing the work its user should not have to | `BRIEF.md:459-461` | `docs/design/native-pr-landing.md`, *Slice 2 as built*; `docs/decisions.md`, *D88 The merge method follows the repository; a squash is followed by a resync of the local branch*; `CHANGELOG.md`, *`athena land <job> --pr`*, *`athena land --pr` works on repositories that allow only squash merges* and *Fix: `athena land --pr` run again after the merge* | Partial: `athena land --pr` pushes the job's branch, opens the pull request, waits for green checks, merges through REST where `athena.toml` sets `merge_via_rest` (this repository does), picks the allowed merge method, and rebuilds the local branch after a squash; it is built for a repository Athena owns, covers one job per pull request, and does not review, respond to review comments or handle other people's pull requests |
| AD-5 | Brownfield integration is first-class, designed for stealth so that no Athena files land in a third-party repository | `BRIEF.md:463-466` | `docs/design/native-pr-landing.md`, *5.4 Stealth mode* and *5.5 Brownfield*; `CHANGELOG.md`, *`athena-core::pr`, slice 1 of native pull-request landing* (link kept in the job's state directory, not the repository) | Partial: the pull-request path that stealth mode shares with owned mode is built and keeps its link store outside the repository; stealth mode itself (a body and branch name free of Athena text, a leak check, no model `Co-Authored-By`, records kept outside the repository) is a later slice and is not built |
| AD-6 | The brief becomes a structured contract, with changes and reviews traceable to requirements cheaply, surfacing lossy communication | `BRIEF.md:468-472` | this map (hand-written rows only) | Not covered |
| AD-7 | The user is lazy and tired, so artifacts (for example a decision sheet) give context, with a home for them | `BRIEF.md:474-476` | none | Not covered |
| AD-8 | A dashboard with anatomy, using Jev | `BRIEF.md:478-481` | none | Not covered |
| AD-9 | Chunk and parallelize work, and consider moving the study up or running it in parallel | `BRIEF.md:483-486` | `docs/design/parallel-jobs.md` | Not covered: the design is reviewed twice (accept with changes, `docs/design/reviews/parallel-jobs-review-1.md` and `parallel-jobs-review-2.md`) and revised after each, not built; re-confirmed |
| AD-10 | Reconstruct the state of any job from the repository, and run structured retros over the data to learn what to ask the user earlier | `BRIEF.md:488-491` | `docs/records.md`, *Principles* ("State is derived") and `athena.introspection/1`, `athena.class-label/1`; `CHANGELOG.md`, *Introspection, step 1 of the build order* and *`athena retry <job>`* | Partial: current state is derived from records; `athena retry` rebuilds a failed job's brief from its introspection record and the earlier cases of the same class, and `athena status <job>` prints the introspection; reconstruction of a job at a past point and structured retros over whole jobs are not built |
| AD-11 | Athena owns acceleration: track timings and feed them back | `BRIEF.md:494-496` | `docs/records.md`, `athena.invocation/1` (`wall_time`); `CHANGELOG.md`, *sccache is removed from `athena.toml` (D84, reversing D80)* | Partial: timings are recorded; nothing feeds them back, and the build cache wiring was removed because its cause could not be reproduced |
| AD-12 | Athena favors fixing root causes | `BRIEF.md:498-500` | `defaults.toml`, the `root-cause` entry; `CHANGELOG.md`, *A `root-cause` default in `defaults.toml`* and *Introspection, step 1* (a cause that restates what happened is flagged and the record rated failed) | Partial: the default is in the model, the producer and verifier briefs carry its full text and the reader a one-sentence rendering, and introspection asks for a cause beyond the instance; whether a given change fixes the root cause is a judgment, and no mechanism checks it |
| AD-13 | Every time the user corrects Athena, it does some introspection as well | `BRIEF.md:502-504` | `docs/design/defaults-loop.md`, *12. Build order, if accepted*; `docs/records.md`, `athena.introspection/1` (named in `CHANGELOG.md`, *Introspection, step 1 of the build order*), `athena.failure-class/1`, `athena.class-label/1`; `CHANGELOG.md`, *`athena retry <job>`* and *`athena review <introspection-id>`* | Partial: after a rejected or failed job, a correction of a reading and any other intervention, Athena writes one introspection (what happened, class, earlier cases, cause, prevention, proposed next request) without being asked, and a person can label its class; the run over the real `records/` is said to follow landing, so no result on real data is shown here, and the host-error and landing triggers are later steps |
| AD-14 | Classify, organize and apply decision sets, with indications that the current thing is not working, so Athena cannot be confidently wrong | `BRIEF.md:506-509` | `docs/design/decision-sets.md`, *8. Build order, if accepted*; `docs/decision-links.toml`; `docs/records.md`, `athena.host-error/1` and `athena.decision-state/1`; `CHANGELOG.md`, *A typed, read-only decision view*, *Decision-sets design §8 step 4* and *`athena backtest`* | Partial: a typed decision view over `docs/decisions.md`, host-error records, a watcher that puts D87 under review on its first refused merge, and the backtest are built; set selection for briefs, the brief component, clearing a review, and failure signals for decisions other than D87 are not |
| AD-15 | Prefer common-sense checks with simpler handling over rigid rules (for example the rule that non-Rust files must be listed in a markdown file) | `BRIEF.md:511-513` | `docs/non-rust-files.md`; `CHANGELOG.md`, *The non-Rust file check (M20) now aims at its intent*; `docs/design/decision-sets.md`, *9* ("a rule that rejects work meeting its intent is a failure signal") | Partial: the one rule the addendum names now checks its intent (code, not data, by extension or `#!` line); other rules are unchanged, and no general check for common-sense issues exists |
