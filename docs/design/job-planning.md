# Job planning: a planner and a critic before the producer

**Status:** proposed, revision 2. Review 1 (`docs/design/reviews/job-planning-review-1.md`) found no High and
11 Medium findings; section 12 says where each is settled. Job 1 of section 8 (the records, roles and config)
is built; nothing else here is.

**Author:** written by hand at Sri's direction ("You are the brain until Athena's brain starts working
properly", 2026-10-17, `BRIEF.md` addenda), and committed as a declared intervention, so the work Athena's
planner should have done is on the record.

**Reader:** Sri, the design reviewer, and the producers of the build jobs in section 8. Paths are relative to
the repository root. Code facts were read at commit 4adaf98 by a read-only review and are marked **(read)**.
Proposals are marked **(design)**, and numbers that were not measured are marked **estimate**.

## 1. Source

Sri, 2026-10-17 (`BRIEF.md`, addenda after the Project Olympus brief):

- "I was surprised to read there's no planner and we move directly to coding. No wonder we've been thrashing"
- "The producer verifier can be a planner critic loop."
- "I'm not sold the API surface should be considered settled"

The Olympus brief puts planning in Loop A ("Intent → Planning → Decomposition → Execution → Verification") and
asks that improvements favor "a test, tool, invariant, interface, or deterministic mechanism" over prompt text
(section 5.1).

## 2. The problem

- `Athena::dispatch` in `crates/athena-core/src/run.rs` checks the budget and pins, makes the prediction commit
  (`job.json`, `brief-producer.md`), then runs `produce_and_verify`: producer (with one send-back), admission,
  checks, verifier and `verify::judge`, then `close_dispatched` **(read)**. No step decides how the work will be
  done before the producer starts coding.
- The reader leaves design choices to the producer. The reading of job 20261017-155259-7mjc says "Exact ref
  choice is left to the producer" **(read)**. That job was rejected.
- `athena plan` (`crates/athena-core/src/plan.rs`) orders the queue. It is deterministic code and does not plan a
  job's work **(read)**. The 2026-10-16 addendum's "planning first" was built as that queue ordering (D105).
- Of Athena's 85 recorded introspections, the largest class is criterion-unmet-at-final-verification (20). Others
  are criteria-check-presence-not-behavior (5), governance-file-needed-but-not-in-reading (4),
  acceptance-method-unspecified (3) and reading-or-provenance-mismatch-caught-only-at-final-verdict (3)
  **(read)**. Each is a problem a review of the plan, before any code, could catch.

## 3. The stage

**Placement (design).** `dispatch` calls a new `job_plan::plan_job` after the prediction commit and before
`produce_and_verify`. The job is already dispatched there, so a crash is recovered by `recover_interrupted`
(`manage.rs`) from the live records that `invoke_and_record` writes **(read)**. Planning is skipped when
`[job_plan]` is absent or `enabled = false`, and for capabilities that pin a repository (study jobs) in the
first build.

**The planner (design)** reads the request, the accepted reading, the checks Athena will run, the commands the
verifier may run (`describe_commands`), and for each deliverable whether it exists and its size. It works in a
read-only snapshot of the prediction commit with the reader's grant (Read, Grep, Glob). It writes a plan:

- the approach;
- decisions, each with the choice, the reason and the options rejected with why;
- files, each with its action and purpose; Athena marks each as governance or not, and whether it exists;
- ordered steps;
- risks, each with a mitigation;
- one demonstration per acceptance criterion: the method, the command or test, what it observes, and
  `fails_without` (what would fail if the behavior were missing);
- open questions.

**Athena's own checks run first, at no model cost (design).** Each problem is a finding with source `athena`:
every criterion has exactly one demonstration; every behavior demonstration has `fails_without` and a command
the verifier's grant allows (`athena_agent::policy::evaluate`); files to modify exist and files to create do
not; a governance path (`admission::is_governance`) must pass `governance_change_is_listed`, else a major
finding; any other path outside the deliverables is a minor finding. A major finding sends the plan back to the
planner with no model call.

**The critic (design)** works in the same snapshot with the same grant. It checks the plan against the request
and the reading, that the code the plan names exists, that the change is made where the fault starts, that each
demonstration would fail without the behavior, that a new `athena.toml` field ships with its schema field, and
that the work fits one producer run. It returns accept, revise or reject, findings (severity, location,
evidence, fix), and whether the reading misread the request. Athena decides the verdict as `verify::judge` does:
accept only when neither source has a blocker or major finding. Reject, or a misread reading, stops the job.

**Rounds and cost (design).** At most three rounds (two revisions). A revision brief quotes the previous plan and
every finding. Caps: `planner_usd` 0.75 and `critic_usd` 0.75, 600 s timeouts (**estimate**: the reader's p90 is
$0.14 and the verifier's p50 $0.33 **(read)**). At dispatch, one round plus the producer and verifier caps must
fit `job_usd`; before each planning call, that call's cap plus the producer and verifier caps must still fit.

**Models (design).** The planner is the reading's producer model and the critic is its verifier model: the
producer–verifier pair moved earlier, as Sri put it. `Config::validate` already keeps the verifier out of
`producer_allowed`, so no model reviews its own plan **(read)**.

## 4. How the producer and verifier use the plan

- **Producer (design):** its prompt is `brief-producer.md` plus a section "Job plan (accepted, round r)", rendered
  by code from the accepted `athena.job-plan/1` record. The rendering is deterministic, and the job-plan record
  stores the sha256 of the rendered section (`producer_section_sha256`), so the producer's exact prompt can be
  rebuilt from the committed `brief-producer.md` and the plan record without a new markdown file (review F6). The
  send-back builds from the plan-augmented prompt, not the original brief, so a second producer run keeps the
  plan (F6). The producer follows the plan and may depart only with a reason in `open_issues`.
- **Planned files left unchanged (code):** before the candidate is built, `produce_and_verify` compares the
  snapshot's change list with the accepted plan's files (when the job was planned) and the accepted reading's
  deliverables (`planned_paths`, `unchanged_planned_files` in `crates/athena-core/src/run/mod.rs`). A listed path
  that is not in the change list and has no entry with a reason in the report's `unchanged_planned_files` is
  sent back with the unmet criteria, in the one send-back (`send_back_text` names each path). If it is still
  unexplained afterwards, it joins the `producer-report-gaps` entry the verifier is given, so a forgotten file is
  found before verification and not by the verifier alone. It is a flag, not a failure, like an unmet
  criterion. A run with no change at all is not checked this way; it fails as an idle producer. The producer
  brief says the same ("Planned files left unchanged"). The reasons are read from the report and are not kept in
  the invocation record.
- **Verifier (design):** it is not given the planner's approach, reasons or rejected options, which come from the
  producer's model and would anchor it (review F5; `VerifierInputs` in `briefs.rs` promises no producer
  reasoning). It gets only the critic-accepted demonstrations (one per criterion) and the planned file list.
  `VerifierInputs::names` gains a conditional entry for them. The verifier judges the request and the criteria
  first and runs each demonstration; a changed file that was not planned, or a planned file that did not change,
  is a finding only when it leaves a criterion unshown or adds scope.
- The guidance goes in the briefs, not in the always-loaded prompt frames, whose budgets are nearly full
  (**estimate**).

## 5. Records, status and failure

- `athena.job-plan/1` at `<job>/job-plan-<round>.json` and `athena.critique/1` at `<job>/critique-<round>.json`.
  A critique that accepts while holding a blocker or major finding is invalid; round 0 is invalid. The critique
  reuses `Finding`, so `FindingSource` gains `Critic`, a change to a committed schema that lands in job 1, before
  anything writes it (review F10).
- **Plumbing (review F1, design):** `plan_job` returns a `Closing` seeded with the planning invocations, plans
  and critiques, and `produce_and_verify` continues from that `Closing` instead of starting a new one, taking
  its next invocation number after the planning invocations. `dispatch` recomputes `spent` after planning, so
  the producer's budget check counts what planning cost. `close_dispatched` writes the plan and critique
  records with the invocations. The invocation numbers (`NN-planner.json`, `NN-critic.json`, before
  `NN-producer.json`) then show the plan came first.
- `records::Role` gains `Planner` and `Critic`. `store.rs` `job_of` needs arms for the new record types: it
  panics on a type it does not know **(read)**. Reused unchanged: `agent_spec`, `invoke_and_record`,
  `check_budget`, `failure`, and the scoring functions, which already count every non-reader invocation as
  execution **(read)**.
- `athena status` shows "plan accepted r2" or "plan stopped r3" per job, and the first-round accept and stop
  counts. `athena status <job>` lists each round with its verdict and findings.
- When planning stops (reject, misread, revise in the last round, a planner or critic failure, or no budget), the
  job fails through `failure()` with the reason "planning stopped before any code was written:" and the findings.
  `go` introspects it. A retry's carry includes the last critique's findings, not only that reason (review F11).
  A job stopped in planning records `verification_passed` as unknown, not false, so it is not scored as a
  verification that failed (F11).

## 6. Naming

The stage is **job planning**: records `athena.job-plan/1` and `athena.critique/1`, roles `planner` and
`critic`, module `athena_core::job_plan`, config `[job_plan]`. This avoids `athena plan` and `crate::plan::Plan`
(queue order), `crate::land::Plan`, and the steward's work items. Sri does not consider the command set settled;
renaming `athena plan` to `athena queue` belongs to the design review Sri asked for, not to this build.

## 7. Alternatives

| Alternative | Why not now |
|---|---|
| Athena's own checks on the reading only, with no planner or critic | Free, and by the introspection counts it reaches 12 of 85 (governance file missing from the reading, criteria that check presence, acceptance method unstated; **unchecked** against records). The largest class, a criterion found unmet only at the final verdict (20), needs a plan to review. These checks are still built first inside the stage (job 4), so their reach is measured on their own (review F14). |
| A stronger reader that also designs | The reader works without reading the code in depth, and one model would plan and review itself. |
| A plan section inside the producer's run | The producer already decides the design mid-coding; that is the status quo that thrashes. |
| A critic alone, reviewing the reading | It can catch vague criteria but not a bad design, because no design exists yet to review. |
| Plans written by hand for every job (today's stopgap) | Works now, but it is the gap itself: the plans should come from Athena. |
| A different model family as critic | Worth testing later; the producer–verifier pair keeps scoring and model rules unchanged. |

## 8. Build plan

Order matters: the config section needs its schema before any `athena.toml` turns it on (the
config-edit-ahead-of-schema class, jobs 20261016-071202-q0xy and 20261016-185517-qcrz). Each job is sized for
one producer run; review 1 split jobs 3 and 5.

1. **Records, roles and config; no behavior change.** `crates/athena-data/src/{records.rs, config.rs}` (with
   `FindingSource::Critic`, F10), `crates/athena-core/src/{store.rs, run_pin_tests.rs}`, the fake driver's role
   arms (`crates/athena-agent/src/fake.rs`: a deterministic planner that covers every criterion, a critic that
   accepts), the exhaustive role matches in `crates/athena-cli/tests/retry.rs` and
   `crates/athena-cli/tests/common/mod.rs` (F15), `docs/records.md`, `CHANGELOG.md`. `Config.job_plan:
   Option<JobPlanConfig>` with `deny_unknown_fields`, `skip_serializing_if = "Option::is_none"`, `enabled` false
   by default, and reset to `None` in `reading_inputs` (F16). No `athena.toml` edit. Done when: `cargo test
   --locked` passes; round-2 records pass `check_record_file` at their paths and are refused as misplaced at
   round-1 paths; a round-0 record and an accepting critique with a major finding are refused; `reading_inputs()`
   for this repository's `athena.toml` is byte-identical to a digest recorded in the test before the change, and
   adding an enabled section leaves it unchanged; `max_rounds = 0`, a zero cap and an unknown key are refused; a
   fake-driver `go` still runs only reader, producer and verifier.
2. **Prompts, outputs and briefs, as pure functions.** `crates/athena-context/prompts/{planner.md, critic.md}`,
   `crates/athena-context/src/{lib.rs, outputs.rs, briefs.rs}`, `crates/athena-cli/tests/budgets.rs`. Done when:
   the planner brief numbers each criterion, flags a listed `athena.toml` as governance and lists the verifier's
   commands; a revision brief quotes the previous plan and its findings; the critic brief carries the
   demonstrations and Athena's findings and nothing about the planner's cost or commands; both output schemas
   forbid extra properties and allow exactly accept, revise and reject; the rendered producer section is
   deterministic (same record, same bytes); the verifier brief with a plan holds only the demonstrations and the
   planned files (F5), and without a plan is byte-identical to today's.
3. **3a: the plumbing in `run.rs`, with a fixed plan.** `crates/athena-core/src/run.rs`, `lib.rs`, a test with the
   fake driver and a plan supplied by the test. Done when: planning invocations are numbered before the
   producer's and are in the landing commit with the plan and critique records (F1); `spent` after planning
   counts the planning invocations; the send-back prompt keeps the plan section (F6); `check_history()` is clean;
   with planning off, roles and records are today's.
   *Built as:* `plan_round` in `run.rs` runs the planner and critic (accept only; any other verdict fails the job
   with the "planning stopped" reason) and returns the producer section; the fake planner and critic stand in
   for a plan supplied by the test, and `crates/athena-cli/tests/job_plan.rs` holds the tests that 3b extends.
4. **3b: the stage.** New `crates/athena-core/src/job_plan.rs`, `crates/athena-cli/tests/job_plan.rs` (planning
   on through `Fixture::with_config`). Done when: with an accepting critic the roles run reader, planner, critic,
   producer, verifier; the producer's prompt holds the planned files and demonstration commands and its section
   hash matches the plan record; for a study, roles and records are today's.
   *Built as:* `produce_and_verify` builds the verifier's `VerifierPlan` from the accepted plan and uses
   `verifier_brief_with_plan` for both the stored brief and the live one; `verification.json` records the input
   through `names_with_plan`. `crates/athena-cli/tests/job_plan.rs` runs the accept path through the real
   lifecycle. No new `job_plan.rs` module in `athena-core` was needed for this step.
5. **4: revise and stop paths, and Athena's own checks.** `job_plan.rs` and its tests. Done when: after revise then
   accept, the second planner prompt quotes the finding; when every round says revise, no producer or verifier
   runs, the outcome is failed with the "planning stopped" reason, three plans and three critiques are
   committed, the totals include all six planning invocations, `verification_passed` is unknown (F11), and
   `check_history()` is clean; a retry's carry quotes the last critique's findings (F11); a plan that edits an
   unlisted `athena.toml`, or cites a command outside the verifier's grant, gets an Athena critique with no
   critic call.
   *Built as:* `plan_round` in `run.rs` loops over `planning_round` up to `max_rounds`; `admission::plan_findings`
   holds Athena's checks; `end_round` files each plan and critique and decides accept, revise or stop. A job
   stopped in planning scores `verification_passes` as unknown in `athena-score` (a job with planner or critic
   invocations and no producer or verifier), and `reading::carry` quotes the stopping critique's findings.
   Tests are in `crates/athena-cli/tests/job_plan.rs`.
6. **5a: status and recovery.** `crates/athena-core/src/{status.rs, manage.rs}`, `crates/athena-cli/src/display.rs`.
   Done when: plan state appears only for planned jobs and `status <job> --json` includes plans and critiques;
   closing a job killed during planning adds no producer invocation.
   *Built as:* a job's plan state is that of its last critique (accepted unless it is not an accept or it names a
   misread request), shown on the job's line in `athena status`; the summary's first-round counts are the planned
   jobs whose planning ended accepted or stopped in round 1. `status <job>` prints each round's verdict and
   findings, and the trace's `plan`, `plans` and `critiques` keys are skipped when empty, so an unplanned job's
   text and JSON are unchanged. `recover_interrupted` records the planner and critic invocations it finds and
   adds the missing-producer record only when no planner, critic or producer invocation exists. Tests are in
   `crates/athena-cli/tests/job_plan.rs`.
7. **5b: turning it on.** Only `athena.toml`, `crates/athena-cli/tests/common/mod.rs` (the test config drops
   `[job_plan]` so existing tests keep today's path), `docs/decisions.md` and `CHANGELOG.md`. The reading lists
   `athena.toml` as a deliverable and names it in a criterion (`governance_change_is_listed`). Queue it only after
   Athena has been rebuilt with `./setup.sh` from a main that holds jobs 1 to 5a. Done when: the repository's
   `athena.toml` parses with planning on; the decision entry names the alternatives above and the reversal (set
   `enabled = false`).
   *Built as:* `athena.toml` ends with `[job_plan]` set to the design's caps (`enabled = true`, `planner_usd` and
   `critic_usd` 0.75, both timeouts 600 s, `max_rounds` 3), and decision D110 records the choice. Test-only edits
   keep the unplanned path where tests need it, each stripping the table wherever it sits (from its header to
   the next table header); the helper is repeated in three places because the crates cannot share test code
   without non-test code. (1) `crates/athena-cli/tests/common/mod.rs`: new `without_job_plan`, which
   `test_config` now applies, so tests that append their own `[job_plan]` (`job_plan.rs`) do not get it twice.
   (2) `crates/athena-data/src/config.rs` tests: a private `unplanned` helper, used by the digest, defaults and
   bad-settings tests, which append their own section; and a new test that the repository's `athena.toml` parses
   with planning on and the design's caps. (3) `crates/athena-core/src/run_pin_tests.rs`: the `config` helper
   drops the `[job_plan]` lines, so a study stays unplanned and the test that enables planning appends its own.
   No non-test code changed.

## 9. Measuring it, and when to keep it

This is a measured experiment in the sense of the Olympus brief's R6. Review 1 found the first rule could not
decide anything (F18 to F20); this replaces it.

- **Primary measure (F18):** the landed share of dispatched change jobs, counting a job stopped in planning as
  not landed, so planning cannot raise the rate by removing jobs from the denominator. Baseline at 4adaf98: 152
  of 237 dispatched jobs landed, about 64% **(read)**.
- **Guards:** cost per landed job (`totals.cost_all`, every job's cost divided by landed jobs) at most $2.50
  against a baseline of about $1.67; attempts per landed request; the five failure classes in section 2 per 100
  jobs; the first-round accept rate; planning's share of cost.
- **Rule (F19, F20):** through the first 30 planned jobs, keep planning on unless the landed share falls below
  the baseline. At 100 planned jobs, keep it if the landed share is at least 72% and cost per landed job is at
  most $2.50; turn it off if the share is at or below 64%; otherwise record the result as inconclusive and keep
  measuring. At 30 jobs a 10-point change is about one standard error (**estimate**), so no firm decision is
  taken there. The landed-share target governs; the cost ceiling is a guard, not a second test of the same
  effect.

**Outcome.** The rule has decided: planning is turned off. `athena status` reported 58 of 105 planned jobs
landed (55%), at or below the 64% baseline at the 100-job point, and cost per landed job of $3.43 against the
$2.50 guard. The rule, not a person, made the call (D135); `athena.toml` now has `enabled = false` under
`[job_plan]`. Kept: the planning code, the resume-from-kept-plan path and the status measures, so planning can be
turned back on for a new measured trial. The earlier decision to keep planning on was taken at 51 jobs, below
the decision point.

## 10. Risks

- Cost: about $0.30 to $0.70 per round (**estimate**), 40 to 70% on today's mean job. It pays only if rejections
  and retries fall.
- Anchoring: the producer and verifier may follow a bad plan. Departures are allowed with a reason, and the
  verifier judges the criteria first.
- Over-strict critics stop jobs that would have landed. Watch for stopped jobs whose retry lands unchanged.
- Forecasts: the reader's forecasts do not include planning and will read low until they do.

## 11. Open questions

- Should study jobs be planned too? Not in the first build; decide after the measurement.
- Should the critic later become a different model family? Decide from the critique records.
- When do hand-written plans stop? When Athena's accepted plans agree with the hand-written ones on recent
  jobs.

## 12. Changes after review 1

| Finding | Settled in |
|---|---|
| F1: planning invocations had no path into the closing records or the budget | Section 5 (plumbing), job 3a |
| F5: the verifier would see the producer model's reasoning | Section 4: demonstrations and planned files only; job 2 |
| F6: the recorded brief was not the producer's prompt, and a send-back dropped the plan | Section 4: rendered section and its hash in the plan record; jobs 2 and 3a |
| F10: `FindingSource::Critic` changes a committed schema | Section 5; job 1 |
| F11: a retry lost the critic's findings, and a stopped job scored as a failed verification | Section 5; job 4 |
| F14: the cheapest alternative was missing | Section 7 |
| F15: job 1 did not compile as listed | Job 1 lists `retry.rs` and `common/mod.rs` and requires `cargo test --locked` |
| F16: the new config field could change every reading's digest | Job 1: skipped when absent, reset in `reading_inputs`, byte-identical digest test |
| Sizing: jobs 3 and 5 too large | Split into 3a and 3b, and 5a and 5b |
| F18 to F20: the keep rule could not decide | Section 9 rewritten |

The Low findings (F2, F3, F4, F7, F8, F9, F12, F13, F17, F21) are left to the build jobs that touch their code,
which should read the review.
