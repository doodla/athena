# Strategy view

This page groups the rows of the [requirements map](requirements-map.md) by goal and shows how much of each
goal the repository covers. The counts are computed: `athena strategy` reads the map's tables and the goal lists
below and prints rows, covered, partial and not covered per goal with each goal's not-covered rows. A test
(`strategy_view_table_matches_the_map` in `crates/athena-cli/src/strategy.rs`) fails when the table below
differs from that computation, so when a row's status changes in the map, update the table here. The ordered
next steps for every goal are in the [roadmap](roadmap.md).

Rows are cited by ID. Each ID appears under exactly one goal; the command reports an ID that is missing from
the map or from every goal. Statuses are the map's: covered, partial, not covered.

## Coverage at a glance

| Goal | Rows | Covered | Partial | Not covered |
| --- | --- | --- | --- | --- |
| G1 Run the work through Athena, restartably | 24 | 6 | 15 | 3 |
| G2 Know the user, and be a strategic partner | 14 | 2 | 8 | 4 |
| G3 Safe, traceable, verified work | 19 | 4 | 15 | 0 |
| G4 Sound shape, standards and resumption | 14 | 6 | 8 | 0 |
| G5 Efficiency, learning and delegation | 14 | 2 | 8 | 4 |
| G6 The study of three systems | 23 | 0 | 7 | 16 |
| G7 Explainer site, report and adoption | 15 | 0 | 3 | 12 |
| G8 Work Athena does so the user does not have to | 5 | 0 | 4 | 1 |
| **Total** | **128** | **20** | **68** | **40** |

Reading the table: the core of v0 is built and gated (G3, G4), and the addenda of 2026-10-12 to 2026-10-14
have moved the most since the map was first written: pull-request landing, introspection, defaults proposals,
the decision view and the Jev client are built in part (G2, G3, G5, G8). The study and the site are still
mostly ahead (G6, G7), and the taste, tracker and structured-brief rows in G2 are still ideas. The table
changed from the first version (124 rows: 20 covered, 58 partial, 46 not covered) by four new rows (AD-12 to
AD-15, all partial) and five rows that moved from not covered to partial (A2-4, A5-5, A6-4, AD-4, AD-5).
Both views were recounted from the map's tables, not patched.

## G1 Run the work through Athena, restartably

The aim: after v0 every change goes through Athena, the session that drives it can restart at any time, and
the state of the work comes from the repository.

- **Covered:** OM-2, OM-3, OM-4, OM-13, OM-14, EN-5.
- **Partial:** OM-1, OM-5, OM-6, OM-7, OM-8, OM-9, OM-11, OM-12, OM-15, EN-1, EN-2, EN-3, EN-7, EN-8, OR-1.
- **Not covered:** OM-10, EN-4, EN-6.
- **Bet:** records and a gate are enough to keep the restart boundary clean (`docs/decisions.md`, *D35 The
  state of the work is derived from records*).
- **Risk:** interventions that change no content depend on the user recording them (`docs/enforcement.md`,
  *Prose-only in v0*), so the intervention trend can look better than the work was.

## G2 Know the user, and be a strategic partner

The aim: a lazy or tired request still gives the right result, because Athena holds a model of what the
user expects and says so when a request looks wrong.

- **Covered:** A2-2, A2-3.
- **Partial:** A1-1, A1-2, A1-3, A2-1, A2-4, AD-12, AD-13, AD-14.
- **Not covered:** A1-4, AD-2, AD-6, AD-7.
- **Bet:** predictions recorded before work, scored with a proper rule, and introspection after each
  correction are the signals that update the defaults model (`docs/decisions.md`, *D14*, *D26*, *D90*). The
  scoring half is built, and a proposal path from repeated introspections is built (`athena propose`); the
  update from prediction scores is not.
- **Open question:** how taste probes (AD-2) and the structured brief (AD-6) should expire or be revised
  without making the user answer what learned defaults already settle.

## G3 Safe, traceable, verified work

The aim: nothing lands unverified, every commit traces to a job, and nothing unsafe to publish is committed.

- **Covered:** A5-1, A5-2, A7-3, A8-2.
- **Partial:** SD-6, SD-7, SD-8, A3-1, A5-3, A5-4, A5-5, A5-6, A7-1, A7-2, A7-4, A7-5, A8-1, A8-3, JG-4.
- **Not covered:** none.
- **Bet:** a gate that runs in the hook, in the core's commit path and over history (`docs/enforcement.md`,
  rows M1 to M7) is stronger than a rule in a prompt.
- **Risk:** records are not signed, so deliberate fabrication of consistent records still passes
  (`docs/enforcement.md`, *Prose-only in v0*, "Records are Athena's own").

## G4 Sound shape, standards and resumption

The aim: a core separate from capabilities, Rust and modular, designed before built, resumable from the
repository alone.

- **Covered:** SD-3, A3-3, A4-1, A4-2, A11-1, JG-5.
- **Partial:** SD-1, SD-2, SD-4, SD-5, A4-3, A4-4, A11-2, AD-15.
- **Not covered:** none.
- **Bet:** a crate per concern, with the dependency direction checked against `docs/architecture.md`,
  keeps the capability seam honest (`docs/enforcement.md`, row M21).
- **Open question:** how to measure a cold agent's ability to extend the work (A12-3 in G5).

## G5 Efficiency, learning and delegation

The aim: cost per verified result falls for repeated work, and Athena delegates toil to automations and
specialists.

- **Covered:** A6-1, A6-2.
- **Partial:** A6-3, A6-4, A9-1, A10-1, A12-3, A13-1, A13-3, AD-11.
- **Not covered:** A9-2, A12-1, A12-2, A13-2.
- **Bet:** a typed-decision model (Jev) can take over many small decisions cheaply. It was left out of v0 on
  purpose (`docs/decisions.md`, *D31*); a client and scoring now exist (*D92*, `athena score-jev`), but no
  calibration on real records has been made.
- **Risk:** without learning curves (A13-2) there is no evidence that repeated work improves.

## G6 The study of three systems

The aim: firstmate, Hermes Agent and Athena, understood through their prompts, run for real, compared, and
cited at every claim.

- **Covered:** none.
- **Partial:** A3-2, S1-1, S1-3, S4-1, S4-4, S5-1, S5-2.
- **Not covered:** S1-2, S1-4, S1-5, S2-1, S2-2, S2-3, S3-1, S3-2, S3-3, S3-4, S3-5, S4-2, S4-3, S6-1, S7-1,
  S7-2.
- **Bet:** validation rules that fail a study with unmarked or unsourced claims
  (`crates/athena-cap-study/src/rules.rs`) keep findings honest before the breadth arrives.
- **Risk:** the lab (S3) is the largest unbuilt piece, and every observed-evidence claim depends on it.
- **Open question:** the user's addendum "Chunking and parallel work" suggests running the study in parallel
  with Athena work; the parallel-jobs design is proposed, not built (AD-9 in G8).

## G7 Explainer site, report and adoption

The aim: a published, mobile-first site and a final report that let an outsider adopt Athena and follow what
was learned.

- **Covered:** none.
- **Partial:** D1-1, D1-2, D1-3.
- **Not covered:** D2-1, D2-2, D2-3, D2-4, D2-5, D2-6, D3-1, JG-1, JG-2, JG-3, AD-1, AD-8.
- **Risk:** the order in the brief puts a live site before the study broadens (OR-1 in G1), so the missing
  site blocks the next milestone.

## G8 Work Athena does so the user does not have to

The aim: Athena handles pull requests, joins an existing repository without leaving files in it, runs
independent jobs in parallel, and keeps the project's state readable.

- **Covered:** none.
- **Partial:** AD-3, AD-4, AD-5, AD-10.
- **Not covered:** AD-9.
- **Bet:** the landing and parallel-jobs designs (`docs/design/native-pr-landing.md`,
  `docs/design/parallel-jobs.md`) are the plan. Landing is built for a repository Athena owns (`athena land
  --pr`, D86 to D88); stealth mode is not. Parallel jobs have had two reviews (accept with changes) and are
  not built.

## What to do next

The steps below run in this order and use the [roadmap](roadmap.md)'s step names. Each says what blocks it.
The order is a proposal for the user to correct. "Blocked by" names the thing that has to exist or be decided
first, as far as the repository shows.

1. **RM-0, acceleration** (`BRIEF.md:494-496`; rows AD-4, AD-5, AD-9, AD-11, A5-3, A5-4, A5-5, A4-3, SD-6).
   Finish pull-request landing by building stealth mode, then build concurrent jobs, then feed timings back.
   Blocked by: stealth mode is a later slice of `docs/design/native-pr-landing.md` and has no test against a
   real remote (the host tests use a fake); concurrent jobs wait for the user's confirming look at the design
   as revised after review 2 and for a test of the shared-file collisions (`CHANGELOG.md`,
   `docs/decisions.md`); the build cache cannot be retried until a job can run `sccache --show-stats` (D84).
2. **RM-6, the firstmate study at full quality,** alongside step 1 (rows A3-2, EN-7, S1-1, S1-3, S4-1, S5-1,
   S5-2). Blocked by: the validation rules are not wired into the CLI for the user (S5-1), and a claim marked
   observed needs a real run, which needs the lab of step 10.
3. **RM-4, close the user-model loop** (rows AD-12, AD-13, AD-14, A2-4, A1-1, A1-3, A1-4, A2-1, AD-2, AD-7).
   The introspection, proposals, decision view, host-error watcher and Jev client are built in part; what is
   left is the first run over the real `records/`, labeling by `athena review`, the live Jev shadow, set
   selection with the brief component, and the update from prediction scores. Blocked by: the run over real
   records follows landing of the built steps; `athena score-jev` gives no verdict below 10 labeled cases (an
   estimate in the code), so the user must label; the live shadow needs the gateway key in the environment;
   set selection needs the user's acceptance of `docs/design/decision-sets.md` (D91).
4. **RM-1, make the project's state readable** (rows AD-3, AD-10, A1-2, AD-6, OM-11). A requirement-to-job
   trace and reconstruction of a job at a past point. Blocked by: no record links a landed job to the map
   rows it answered, and that record is not yet designed.
5. **RM-2, record the operating facts** (rows OM-1, OM-10, OM-12, OM-15, EN-1, EN-2, EN-3, EN-4, EN-6, A11-2).
   Blocked by: a preflight record needs a decision on what counts as a capability (push access, Pages, Iris);
   the session facts are outside Athena by design and need the user to report them.
6. **RM-3, close the intervention loop** (rows OM-5 to OM-9). A trend per class of work and an audit for missed
   interventions. Blocked by: enough interventions per class to trend (none measured here), and step 3, whose
   introspection is the link from an intervention to its improvement job.
7. **RM-5, complete enforcement** (rows A3-1, A5-6, A7-1, A7-2, A7-4, A7-5, A8-1, A8-3, SD-4, SD-7, SD-8,
   JG-4), including rows in `docs/enforcement.md` for the addenda's rules. Blocked by: the mechanisms of steps
   1 and 3 must exist before they are mapped.
8. **RM-7, the first slice of the site** (rows D1-1, D1-2, D1-3, D2-1 to D2-6, A4-4, EN-8, OR-1). Blocked by:
   Pages and Iris access are not confirmed (OM-1), and the Iris ledger does not exist.
9. **RM-9, efficiency, learning and delegation** (rows A6-3, A6-4, A9-1, A9-2, A10-1, A12-1 to A12-3, A13-1 to
   A13-3). Blocked by: routing revision needs the timings and costs of step 1, and Jev calibration needs step
   3.
10. **RM-10, study Hermes and Athena, then compare** (rows S1-2, S1-4, S1-5, S2 to S7 rows). Blocked by: the
    lab (S3) is not built, Athena must be tagged before it is studied, and step 8 should be live first
    (OR-1).
11. **RM-11, the final report and the brief's own tests** (rows D3-1, JG-1 to JG-3). Blocked by: steps 8 and
    10.
12. **RM-12, the plugin and the dashboard** (rows AD-1, AD-8). Blocked by: the core's commands must be stable,
    and the dashboard uses Jev, so step 3.
13. **RM-13, keep the standards honest** (rows SD-1, SD-2, SD-5, and AD-15). Blocked by: these are judgments;
    the verifier severity check needs a recorded review for each design, and AD-15 needs the user to say what
    a common-sense check should look for beyond the one rule already revised.
