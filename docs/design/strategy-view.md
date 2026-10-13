# Strategy view

This page groups the rows of the [requirements map](requirements-map.md) by goal and shows how much of each
goal the repository covers. It is a snapshot written by hand from the map; when a row's status changes in
the map, update the counts here. The ordered next steps for every goal are in the [roadmap](roadmap.md).

Rows are cited by ID. Each ID appears under exactly one goal. Statuses are the map's: covered, partial, not
covered. Counts were made by counting rows in the map's tables, and they are exact for the map as written.

## Coverage at a glance

| Goal | Rows | Covered | Partial | Not covered |
| --- | --- | --- | --- | --- |
| G1 Run the work through Athena, restartably | 24 | 6 | 15 | 3 |
| G2 Know the user, and be a strategic partner | 11 | 2 | 4 | 5 |
| G3 Safe, traceable, verified work | 19 | 4 | 14 | 1 |
| G4 Sound shape, standards and resumption | 13 | 6 | 7 | 0 |
| G5 Efficiency, learning and delegation | 14 | 2 | 6 | 6 |
| G6 The study of three systems | 23 | 0 | 7 | 16 |
| G7 Explainer site, report and adoption | 15 | 0 | 3 | 12 |
| G8 Work Athena does so the user does not have to | 5 | 0 | 2 | 3 |
| **Total** | **124** | **20** | **58** | **46** |

Reading the table: the core of v0 is built and gated (G3, G4), the study and the site are mostly ahead (G6,
G7), and the addenda that ask Athena to do more for its user (G8, and the taste and tracker rows in G2) are
mostly designs or ideas.

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
- **Partial:** A1-1, A1-2, A1-3, A2-1.
- **Not covered:** A1-4, A2-4, AD-2, AD-6, AD-7.
- **Bet:** predictions recorded before work, scored with a proper rule, are the signal that will update the
  defaults model (`docs/decisions.md`, *D14*, *D26*). The scoring half is built; the update half is not.
- **Open question:** how taste probes (AD-2) and the structured brief (AD-6) should expire or be revised
  without making the user answer what learned defaults already settle.

## G3 Safe, traceable, verified work

The aim: nothing lands unverified, every commit traces to a job, and nothing unsafe to publish is committed.

- **Covered:** A5-1, A5-2, A7-3, A8-2.
- **Partial:** SD-6, SD-7, SD-8, A3-1, A5-3, A5-4, A5-6, A7-1, A7-2, A7-4, A7-5, A8-1, A8-3, JG-4.
- **Not covered:** A5-5.
- **Bet:** a gate that runs in the hook, in the core's commit path and over history (`docs/enforcement.md`,
  rows M1 to M7) is stronger than a rule in a prompt.
- **Risk:** records are not signed, so deliberate fabrication of consistent records still passes
  (`docs/enforcement.md`, *Prose-only in v0*, "Records are Athena's own").

## G4 Sound shape, standards and resumption

The aim: a core separate from capabilities, Rust and modular, designed before built, resumable from the
repository alone.

- **Covered:** SD-3, A3-3, A4-1, A4-2, A11-1, JG-5.
- **Partial:** SD-1, SD-2, SD-4, SD-5, A4-3, A4-4, A11-2.
- **Not covered:** none.
- **Bet:** a crate per concern, with the dependency direction checked against `docs/architecture.md`,
  keeps the capability seam honest (`docs/enforcement.md`, row M21).
- **Open question:** how to measure a cold agent's ability to extend the work (A12-3 in G5).

## G5 Efficiency, learning and delegation

The aim: cost per verified result falls for repeated work, and Athena delegates toil to automations and
specialists.

- **Covered:** A6-1, A6-2.
- **Partial:** A6-3, A9-1, A12-3, A13-1, A13-3, AD-11.
- **Not covered:** A6-4, A9-2, A10-1, A12-1, A12-2, A13-2.
- **Bet:** a typed-decision model (Jev) can take over many small decisions cheaply. It was left out of v0 on
  purpose (`docs/decisions.md`, *D31*), so no calibration record exists yet.
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
- **Partial:** AD-3, AD-10.
- **Not covered:** AD-4, AD-5, AD-9.
- **Bet:** the landing and parallel-jobs designs (`docs/design/native-pr-landing.md`,
  `docs/design/parallel-jobs.md`) are the plan. Both are marked proposed, and the second says its
  independent review is pending.

## What to do next

See the [roadmap](roadmap.md). Its first step, RM-0, follows the brief's 2026-10-13 addendum that Athena owns
acceleration (`BRIEF.md:494-496`): the build cache, concurrent jobs, native pull-request landing and timings.
The firstmate study (RM-6) runs alongside it. The next three steps, RM-1 to RM-3, need no new subsystem: they
make the project's state and the intervention loop visible from the records Athena already writes.
