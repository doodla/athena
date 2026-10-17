# Job planning: a planner and a critic before the producer

**Status:** proposed. Independent review requested (`docs/design/reviews/job-planning-review-1.md`, not yet
written). Nothing here is built.

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

- **Producer (design):** its prompt is `brief-producer.md` plus a section "Job plan (accepted, round r)",
  rendered from the plan record and appended the way the "Sent back" section is today **(read)**. It follows the
  plan and may depart only with a reason in `open_issues`.
- **Verifier (design):** `VerifierInputs` gains the plan. The brief lists the demonstrations, planned files that
  did not change, and changed files that were not planned (from `candidate.changes`). The verifier judges the
  request and the criteria first and runs each demonstration. A departure from the plan is a finding only when
  it leaves a criterion unshown or adds scope.
- The guidance goes in the briefs, not in the always-loaded prompt frames, whose budgets are nearly full
  (**estimate**).

## 5. Records, status and failure

- `athena.job-plan/1` at `<job>/job-plan-<round>.json` and `athena.critique/1` at `<job>/critique-<round>.json`.
  A critique that accepts while holding a blocker or major finding is invalid. Both are committed in the closing
  or landing commit, and the invocation numbers (`NN-planner.json`, `NN-critic.json`, before `NN-producer.json`)
  show the plan came first.
- `records::Role` gains `Planner` and `Critic`. `store.rs` `job_of` needs arms for the new record types: it
  panics on a type it does not know **(read)**. Reused unchanged: `agent_spec`, `invoke_and_record`,
  `check_budget`, `failure`, `close_dispatched`, `Finding` and `Severity` (with `FindingSource::Critic`), and the
  scoring functions, which already count every non-reader invocation as execution **(read)**.
- `athena status` shows "plan accepted r2" or "plan stopped r3" per job, and the first-round accept and stop
  counts. `athena status <job>` lists each round with its verdict and findings.
- When planning stops (reject, misread, revise in the last round, a planner or critic failure, or no budget), the
  job fails through `failure()` with the reason "planning stopped before any code was written:" and the findings.
  `go` introspects it, and a retry passes the reason to the next reader through `earlier_attempt` **(read)**.

## 6. Naming

The stage is **job planning**: records `athena.job-plan/1` and `athena.critique/1`, roles `planner` and
`critic`, module `athena_core::job_plan`, config `[job_plan]`. This avoids `athena plan` and `crate::plan::Plan`
(queue order), `crate::land::Plan`, and the steward's work items. Sri does not consider the command set settled;
renaming `athena plan` to `athena queue` belongs to the design review Sri asked for, not to this build.

## 7. Alternatives

| Alternative | Why not now |
|---|---|
| A stronger reader that also designs | The reader works without reading the code in depth, and one model would plan and review itself. |
| A plan section inside the producer's run | The producer already decides the design mid-coding; that is the status quo that thrashes. |
| A critic alone, reviewing the reading | It can catch vague criteria but not a bad design, because no design exists yet to review. |
| Plans written by hand for every job (today's stopgap) | Works now, but it is the gap itself: the plans should come from Athena. |
| A different model family as critic | Worth testing later; the producer–verifier pair keeps scoring and model rules unchanged. |

## 8. Build plan

Order matters: the config section needs its schema before any `athena.toml` turns it on (the
config-edit-ahead-of-schema class, jobs 20261016-071202-q0xy and 20261016-185517-qcrz). Each job is sized for
one producer run.

1. **Records, roles and config; no behavior change.** `crates/athena-data/src/{records.rs, config.rs}`,
   `crates/athena-core/src/{store.rs, run_pin_tests.rs}`, the fake driver's role arms
   (`crates/athena-agent/src/fake.rs`: a deterministic planner that covers every criterion, a critic that
   accepts), tests, `docs/records.md`, `CHANGELOG.md`. `Config.job_plan: Option<JobPlanConfig>` with
   `deny_unknown_fields`, `enabled` false by default, and left out of `reading_inputs` so existing readings stay
   valid. No `athena.toml` edit. Done when: round-2 records pass `check_record_file` at their paths and are
   refused as misplaced at round-1 paths; a round-0 record and an accepting critique with a major finding are
   refused; this repository's `athena.toml` parses to `job_plan == None` and adding an enabled section leaves
   `reading_inputs()` unchanged; `max_rounds = 0`, a zero cap and an unknown key are refused; a fake-driver `go`
   still runs only reader, producer and verifier.
2. **Prompts, outputs and briefs, as pure functions.** `crates/athena-context/prompts/{planner.md, critic.md}`,
   `crates/athena-context/src/{lib.rs, outputs.rs, briefs.rs}`, `crates/athena-cli/tests/budgets.rs`. Done when:
   the planner brief numbers each criterion, flags a listed `athena.toml` as governance and lists the verifier's
   commands; a revision brief quotes the previous plan and its findings; the critic brief carries the
   demonstrations and Athena's findings and nothing about the planner's cost or commands; both output schemas
   forbid extra properties and allow exactly accept, revise and reject; the verifier brief without a plan is
   byte-identical to today's.
3. **The accept path in `go`.** New `crates/athena-core/src/job_plan.rs`, `run.rs`, `lib.rs`,
   `crates/athena-cli/tests/job_plan.rs` (planning on through `Fixture::with_config`). Done when: with an
   accepting critic the roles run reader, planner, critic, producer, verifier; the landing commit holds
   `job-plan-1`, `critique-1` and invocations 02 to 05, and `check_history()` is clean; the producer's prompt holds
   the planned files and demonstration commands and the verifier's brief holds the plan; with the section absent
   or off, or for a study, roles and records are today's.
4. **Revise and stop paths, and Athena's own checks.** `job_plan.rs` and its tests. Done when: after revise then
   accept, the second planner prompt quotes the finding; when every round says revise, no producer or verifier
   runs, the outcome is failed with the "planning stopped" reason, three plans and three critiques are committed,
   the totals include all six planning invocations, and `check_history()` is clean; a plan that edits an
   unlisted `athena.toml`, or cites a command outside the verifier's grant, gets an Athena critique with no critic
   call.
5. **Status, recovery, and turning it on.** `crates/athena-core/src/{status.rs, manage.rs}`,
   `crates/athena-cli/src/display.rs`, `crates/athena-cli/tests/common/mod.rs` (the test config drops
   `[job_plan]`), `athena.toml`, `docs/decisions.md`, `CHANGELOG.md`. Queue it only after Athena has been rebuilt
   with `./setup.sh` from a main that holds jobs 1 to 4. Done when: plan state appears only for planned jobs and
   `status <job> --json` includes plans and critiques; closing a job killed during planning adds no producer
   invocation; the repository's `athena.toml` parses with planning on; the decision entry names the alternatives
   above and the reversal (set `enabled = false`).

## 9. Measuring it, and when to keep it

This is a measured experiment in the sense of the Olympus brief's R6.

- **Baseline at 4adaf98 (read):** 237 jobs closed after dispatch, 152 landed, 61 rejected, 24 failed; 155 of 216
  verifications passed (about 72%); about $1.67 per landed job.
- **Track, for planned change jobs:** first-pass verification rate; cost and execution wall time per landed job
  (counting every job, landed or not); attempts per landed request; the five failure classes in section 2 per
  100 jobs; the first-round accept rate; planning's share of cost.
- **Keep rule (design):** after about 30 planned jobs, keep planning if the first-pass rate rises by at least 10
  points and cost per landed job rises by no more than 25%. Otherwise record the result as unsupported or
  inconclusive, keep the records, and turn it off.

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
