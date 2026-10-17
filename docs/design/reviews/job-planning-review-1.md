# Review 1 of `docs/design/job-planning.md`

**Reviewed:** the design at status "proposed", as it stands in the job's base commit (short id `de62c08`). The
snapshot holds that one commit and no history, so the commit `4adaf98` that the design says its facts were read
at could not be compared with it; every line number below is from the base commit.
**Method:** reading, plus searches of the source. No command was run to reproduce behavior and no model was
called. The `records/` directory is absent from the snapshot, so every figure the design takes from records
(85 introspections and their classes, 237 closed jobs, 155 of 216 verifications, about $1.67 per landed job,
the job ids named in sections 2 and 8) is **unchecked**. Only their arithmetic was checked: 152 + 61 + 24 = 237,
and 155 / 216 = 71.8%. Numbers that are not measured are labeled **estimate**.

## Verdict: build after the listed fixes

The shape is sound. The stage sits at a point where the job is already dispatched, it reuses the existing
invocation, budget, failure and scoring paths, the model pair keeps the verifier independent of the producer
model, and the build order puts the schema before any `athena.toml` edit. Nothing found blocks the build
(no High finding). Eleven Medium findings should be settled in the design or in the named build job before it
starts, and the keep rule in section 9 needs a rewrite before it can decide anything.

Counts: 0 High, 11 Medium (F1, F5, F6, F10, F11, F14, F15, F16, F18, F19, F20), 10 Low (F2, F3, F4, F7, F8, F9,
F12, F13, F17, F21).

Severity: **High** blocks the build; **Medium** must be settled in the design or the first build job; **Low**
is a correction or a later job.

## 1. Code claims marked (read), checked against the source

| Design claim | What the source says | Holds? |
|---|---|---|
| `Athena::dispatch` (`run.rs`) checks the budget and pins, makes the prediction commit (`job.json`, `brief-producer.md`), then runs `produce_and_verify`, then `close_dispatched` (§2) | `run.rs` 184 (`dispatch`); 236-241 budget check; 246 `pin_repo`; 311-323 the two files in the prediction commit; 342-343 `produce_and_verify` then `close_dispatched` | Confirmed |
| `produce_and_verify`: producer with one send-back, admission, checks, verifier, `verify::judge` (§2) | `run.rs` 653 (definition); 602 `PRODUCER_SEND_BACKS = 1`; 789-800 the send-back limit. I read the producer loop (653-828) and the start of the candidate step (830-852). I did not read the admission, checks and verifier part, where `judge` is called | Confirmed for the producer half; the rest **unchecked** |
| `athena plan` (`plan.rs`) orders the queue and does not plan a job's work (§2) | `crates/athena-core/src/plan.rs` exists, with `pub struct Plan` at 115. I did not read its logic | Exists; behavior **unchecked** |
| `recover_interrupted` (`manage.rs`) recovers a crash from the live records `invoke_and_record` writes (§3) | `manage.rs` 236 (definition), 245-256 reads `live_records` and keeps invocation records not yet committed; 257-275 turns partial usage files into lower-bound records; `invocation.rs` 226-238 (`invoke_and_record` with a `FileSink` and `PartialUsage`) | Confirmed. See F4 for what it also does |
| `describe_commands` lists the commands a grant allows (§3) | `athena-agent/src/policy.rs` 905; used at `run.rs` 267 | Confirmed |
| The planner works with the reader's grant (Read, Grep, Glob) (§3) | `reading.rs` 300-305: `tools: Read, Grep, Glob`, no commands, `writes: false` | Confirmed |
| `athena_agent::policy::evaluate` can say whether the verifier's grant allows a command (§3) | `policy.rs` 76: `evaluate(policy, tool, input, cwd)`; the `Bash` arm at 128-134 calls `check_bash` and returns `Deny` or `NoObjection`. It needs `policy.root` to exist (77-80 deny when it is missing) | Confirmed, with a limit (F3) |
| `admission::is_governance` identifies governance paths (§3) | `admission.rs` 59-64: `athena.toml`, `defaults.toml`, `setup.sh`, `rust-toolchain.toml` (32-37), `.gitattributes`, workflow files, tool-config files | Confirmed. Manifest lint tables are separate (F2) |
| `governance_change_is_listed` (§3) | `admission.rs` 117-124: the path is a listed deliverable and a criterion's text contains the path or its file name | Confirmed |
| `verify::judge` decides the verdict from findings (§3) | `verify.rs` 246 (`judge`); findings carry `Severity` and the source `athena`; the function builds reasons from failed checks, a verifier failure, covered requirement rows and unassessed criteria (255-298) | Confirmed. I did not read the rest of the function. The "accept only when neither source has a blocker or major finding" rule is the design's own proposal |
| `Config::validate` keeps the verifier out of `producer_allowed` (§3) | `config.rs` 329-333: `VerifierCanProduce` | Confirmed |
| The reader's p90 cost is $0.14 and the verifier's p50 $0.33 (§3) | Needs `records/` | **Unchecked** |
| The "Sent back" section is appended to the brief today (§4) | `run.rs` 803: `format!("{brief}\n\n## Sent back\n\n")`, and 827 `attempt_brief = sent_back`. It is appended to the original `brief` and replaces the prompt for the second run | Confirmed. See F6 |
| `store.rs` `job_of` "panics on a type it does not know" (§5) | `store.rs` 208-221: `job_of` returns `Option<&JobId>` and ends in `_ => None`. It does not panic. The panic is the `.expect("every other record names its job")` in `Records::load` at 273-276 | **Contradicted** for `job_of` itself; the consequence is real (F9) |
| Reused unchanged: `agent_spec`, `invoke_and_record`, `check_budget`, `failure`, `close_dispatched` (§5) | Exist: `reading.rs` 142, `invocation.rs` 226, `reading.rs` 210, `run.rs` 1357 and 1429. `produce_and_verify` and `close_dispatched` do not take planning invocations as they are (F1) | Exist; "unchanged" is **contradicted** for the run path |
| `Finding` and `Severity` reused "with `FindingSource::Critic`" (§5) | `records.rs` 215-218: `FindingSource` has `Verifier` and `Athena` only. `Critic` does not exist | **Contradicted** (F10) |
| Scoring counts every non-reader invocation as execution (§5) | `scoring.rs` 116-119 (`cost_execution` filters `role != Role::Reader`); `athena-score/src/lib.rs` 446 does the same | Confirmed |
| A failed job's reason reaches the next reader through `earlier_attempt` (§5) | `reading.rs` 811-877: `earlier_attempt` accepts a failed job and carries its request, verification and outcome reason (852-858, 875) | Confirmed. Findings in critique records would not be carried (F13) |
| `Config::validate` already keeps the verifier out of `producer_allowed`, so no model reviews its own plan (§3) | Same lines as above. The critic's model is the reading's verifier model and the planner's is the reading's producer model (`run.rs` 682 shows the producer model is taken from the reading) | Confirmed |
| `go` introspects a failed job (§5) | Not read | **Unchecked** |

## 2. Findings by section

### Section 3, The stage

**F1 (Medium): planning invocations have no path into the closing records or the budget.**
`dispatch` builds `ctx.job` from the records loaded before dispatch (`run.rs` 329-341). `produce_and_verify`
starts its own `Closing::new()` and takes the next sequence number from `ctx.job.next_seq()` (`run.rs` 659, 664).
`close_dispatched` adds the invocations in `closing.new_invocations` to the committed ones and writes only
invocation, candidate and verification files (`run.rs` 1431-1475). `dispatch` computes `spent` once, before any
agent runs (`run.rs` 235). So the design's "invocation numbers show the plan came first", "the totals include all
six planning invocations", "before each planning call, that cap plus the producer and verifier caps must
still fit", and "committed in the closing or landing commit" each need a value to carry planning invocations,
plan files and spend into `produce_and_verify` and `close_dispatched`. The design calls those functions reused
unchanged. Without this the producer would reuse sequence number 02 and collide with the planner's record.
**Fix:** state in section 3 that `plan_job` returns a struct of its invocations, plan and critique files and
spend; that `produce_and_verify` starts from that struct (sequence number, `closing.new_invocations`, extra
files); and that `close_dispatched` writes the extra files. Put it in the Done-when of job 3, with a test that
the producer's sequence number follows the critic's.

**F2 (Low): the governance check does not cover manifest lint tables.**
`is_governance` is whole-file; `Cargo.toml` `[lints]` tables are handled by separate admission rules
(`admission.rs` 58, 66-81). A plan that edits those tables passes the proposed Athena check and is refused later
at admission. **Fix:** name `is_manifest` and `lints_tables` in the check, or say the case is left to admission.

**F3 (Low): the command check judges permission, not usefulness.**
`evaluate` needs an existing root directory (`policy.rs` 77-80) and a `Policy` for the verifier role, which
`agent_spec` builds from a grant and a snapshot (`reading.rs` 162-167). The check therefore runs after the
snapshot exists, and it shows only that a command would not be denied, not that it exists in the repository.
**Fix:** say so, and keep "the code the plan names exists" as the critic's check.

**F4 (Low): recovery needs a change that the design assigns to the last job.**
`recover_interrupted` records a `Missing` producer invocation when no producer invocation exists
(`manage.rs` 276-298). A job killed during planning would get that record. Job 5 fixes this
("closing a job killed during planning adds no producer invocation"), but jobs 3 and 4 add the stage first.
It is harmless while `[job_plan]` is off, but the fix belongs with the code that creates the case.
**Fix:** move the `recover_interrupted` change and its test into job 3.

Planning for study jobs is skipped by `capability.pins_repo()` (`run.rs` 370, 685). No finding.

### Section 4, How the producer and verifier use the plan

**F5 (Medium): the verifier is given reasoning from the producer's model, against the verifier's stated inputs.**
`VerifierInputs` says no field can carry the producer's report, commands, reasoning or cost
(`briefs.rs` 325-327), and `VerifierInputs::names` lists the inputs recorded in `verification.json`
(`briefs.rs` 343-356). The plan is written by the model the reading chose as producer, and it holds decisions,
reasons and rejected options. Passing it to the verifier reintroduces the anchoring the design itself lists as a
risk (section 10), and `names()` would need a conditional entry. **Fix:** decide in section 4 either to give the
verifier only the critic-accepted demonstrations and the file list (no approach, reasons or rejected options), or
to record the exception and add the plan to `names()` only when a plan exists. Put the choice in the Done-when
of job 2.

**F6 (Medium): the plan-augmented producer prompt is not what the record shows, and a send-back could drop it.**
The prediction commit holds `brief-producer.md` and its hash before any work (`run.rs` 302, 313), and
`producer_brief` is "stored verbatim" (`briefs.rs` 201). The plan is added after that commit, so the committed
brief is not the producer's prompt. The only job markdown files that `check_record_file` accepts are
`brief-producer.md` and `brief-verifier.md` (`records.rs` 1998). Also, the send-back prompt is built from the
original `brief` (`run.rs` 803), so a plan appended to a copy would be lost on the second run.
**Fix:** say where the final producer prompt is recorded (for example a `brief-producer-2.md`, or a hash of the
plan in the producer's invocation record), add it to the markdown allow-list, and make the send-back use the
plan-augmented brief. Put these in jobs 1 and 3.

**F7 (Low): the verifier would have two scope lists.**
The verifier brief states that the deliverables list is the scope (test at `briefs.rs` 865). The design adds "planned
files that did not change, and changed files that were not planned". **Fix:** say which wins, and keep the
deliverables as the scope.

### Section 5, Records, status and failure

**F8 (Low): `Role::parse`, `as_str` and the storage split need arms for both roles.**
`records.rs` 29-47 (`as_str`, `parse`) and `store.rs` 264-272 (steward, audience and auditor runs are kept apart
from `jobs`) show the pattern. The design says planner and critic invocations belong to the job, which is the
default arm, so no change to the split is needed; say so. **Fix:** one sentence in section 5.

**F9 (Low): the `job_of` statement is wrong about where it panics.** See the table. The needed change is the
same: add arms to `job_of` (`store.rs` 208-221) for the new record types, or `Records::load` panics at 273-276;
and `JobRecords::add` must store them. **Fix:** reword the sentence.

**F10 (Medium): `FindingSource::Critic` is a schema change to a record that is already committed.**
`Finding` is a field of `athena.verification/1` (`records.rs` 666-695), and `FindingSource` has two variants
(`records.rs` 215-218). The design lists `FindingSource::Critic` under "reused unchanged". Adding a variant is
backward compatible for reading but an older binary refuses a new value, and the critique record must carry
the same `Finding`. **Fix:** list `FindingSource::Critic` in job 1's files and Done-when, and state whether the
critique record reuses `Finding` or defines its own type; if it reuses it, the variant is needed before job 2.

**F11 (Medium): a retry would lose the critic's findings, and the stopped job's score is wrong.**
`earlier_attempt` carries the request, verification and outcome reason (`reading.rs` 852-858, 875); it does not
read critique records. The design says "a retry passes the reason to the next reader", which is only the
`planning stopped before any code was written:` string. `failure()` sets `verification_passed: Some(false)`
(`run.rs` 1359-1365), so a job stopped in planning scores the reader's `p_verification_passes` forecast as a
miss for a verification that never ran. **Fix:** say whether `carry` includes the last critique's findings; if
yes, add it to job 4's Done-when. Say whether a stopped-in-planning job is excluded from the verification
Brier event, or accept the effect and note it in section 10 beside the forecast risk.

**F12 (Low): status needs a place for the plan state.**
`status.rs` (687 lines) has no planner concept; job 5 adds it. No finding beyond: the display is covered by job 5's
Done-when.

### Section 6, Naming

**Confirmed:** `crate::land::Plan` (`land.rs` 25) and `crate::plan::Plan` (`plan.rs` 115) both exist, so the
design's reason for the name `job_plan` holds. `docs/ontology.toml` 21-22 already defines "Job planning" as
proposed, which agrees with it.

**F13 (Low): two stale statements will outlive this review.** The header says the review is "not yet written"
(`job-planning.md` line 3), and `docs/ontology.toml` 21-22 says "proposed, not built". Job 5 must update the
ontology entry, which `crates/athena-cli/tests/ontology.rs` checks for structure only. **Fix:** change the header
when the design next changes, and add the ontology entry to job 5's file list.

### Section 7, Alternatives

**F14 (Medium): the alternatives omit the cheapest one, the deterministic checks alone.**
The checks that need no model (every criterion has a demonstration, a governance file is listed, the commands
are allowed) could run on the reader's output with no planner. By the design's own counts these address
governance-file-needed-but-not-in-reading (4), criteria-check-presence-not-behavior (5) and
acceptance-method-unspecified (3), 12 of the 85 introspections, at no model cost. The largest class, criterion
unmet at final verification (20), would need the planner. Without this row the full-cost design cannot be
compared with its partial. **Fix:** add the row ("Athena's own checks on the reading only") with its measured
reach and the reason for not stopping there. The counts are **unchecked** (no `records/`).

### Section 8, Build plan

Ordering: the schema (`config.rs`, job 1) comes before the only `athena.toml` edit (job 5), and `athena.toml`
is read by `include_str!` in `config.rs` 419 (the repository's own config test), so a config edit ahead of its
schema would fail that test at once. The order holds. Job 2 depends only on the records of job 1; job 3 uses
`Fixture::with_config` (`tests/common/mod.rs` 69), which exists. The text "Queue job 5 only after Athena has been
rebuilt with `./setup.sh`" is a note, not a mechanism, but the config test covers a missing schema.

**F15 (Medium): job 1 does not compile as listed.**
Adding `Role::Planner` and `Role::Critic` breaks the exhaustive matches on `Role` that job 1 does not list:
`crates/athena-cli/tests/retry.rs` 56-60 (a match with no wildcard, arms at 57-59) and
`crates/athena-cli/tests/common/mod.rs` 372-455 (arms for each role). Job 1 lists the fake driver
(`athena-agent/src/fake.rs` 183-268, also exhaustive) and `run_pin_tests.rs` 153-210 but not these two. An
unlisted file also becomes a minor admission finding. **Fix:** add both files to job 1's list with an arm for
each new role, and require `cargo test --locked` in its Done-when.

**F16 (Medium): `Config.job_plan` could change every reading's `config_sha256`.**
`reading_inputs` serializes the whole `Config` (`config.rs` 318-323) and the digest is taken of that text
(`capability.rs` 45). A new `Option` field serializes as `null` even when absent, so the digest of every
existing reading changes the first time the new binary dispatches, and `dispatch` rereads those jobs
(`run.rs` 195-209). The test at `config.rs` 436-449 compares two configs from the same binary, so it would not
notice. **Fix:** in job 1, skip the field when `None` (`skip_serializing_if`) and reset it to `None` in
`reading_inputs`; the Done-when should say that `reading_inputs()` for the repository's `athena.toml` is
byte-identical before and after the change (a recorded digest in the test).

**F17 (Low): job 5's test-config statement needs a placement rule.**
`test_config()` builds its text from `athena.toml` by keeping everything before the first
`[[capabilities.change.checks]]` and after `[classes]` (`tests/common/mod.rs` 20-37). A `[job_plan]` section
outside the dropped region would turn planning on in all CLI tests. **Fix:** say where in `athena.toml` the
section goes, or have `test_config` remove it by name.

**Sizing (one producer run).** The producer cap is $3.00 and 1800 s (`athena.toml` 26, 31). No per-job cost data
is available in the snapshot, so the sizes below are judged by the amount of code touched (measured lines) and
are **estimates**.

| Job | Touches | Fits one run? |
|---|---|---|
| 1 | `records.rs` (2841 lines), `config.rs`, `store.rs`, `fake.rs`, `run_pin_tests.rs`, docs, plus the two files in F15 and the `FindingSource` change in F10 | Yes, if F10 and F15 are counted. Two record types with validation and a config section are the bulk |
| 2 | Two prompts, `lib.rs`, `outputs.rs`, `briefs.rs` (1240 lines), `budgets.rs` | Yes; pure functions |
| 3 | New `job_plan.rs`, `run.rs` (1980 lines), `lib.rs`, a new integration test; F1, F4 and F6 add plumbing, recovery and the recorded brief | **No, likely too large.** It holds the stage, the plumbing for `produce_and_verify`, the producer and verifier prompts and an end-to-end test. **Fix:** split into 3a (plumbing in `run.rs` with the fake driver, plan fixed) and 3b (the stage and prompts) |
| 4 | `job_plan.rs` and its tests | Yes |
| 5 | `status.rs` (687 lines), `manage.rs` (654), `display.rs`, test config, `athena.toml`, `decisions.md`, `CHANGELOG.md` | **No.** It mixes display, recovery and a governance-file edit. **Fix:** 5a status and recovery; 5b only `athena.toml` and the decision entry, which lists `athena.toml` as a deliverable and names it in a criterion (`governance_change_is_listed`, `admission.rs` 117-124) |

### Section 9, Measuring it, and when to keep it

The keep rule can be computed only in part. Each measure against the records:

| Measure | Source in the records | Computable? |
|---|---|---|
| First-pass verification rate | `verification.json` `final_verdict` (`records.rs` 691), one per job; the send-back is inside the job (`run.rs` 789-800) | Yes, but "first-pass" is not a field. See F18 for the denominator |
| Cost per landed job | `outcome.json` `totals.cost_all` and `cost_execution` (`records.rs` 699-708) | Yes. The design must say which; they differ by the reader (`scoring.rs` 116-119). A cost that is a lower bound is marked `provisional` (`records.rs` 773) |
| Execution wall time | `totals.wall_time_execution`, measured from dispatch (`run.rs` 271, `scoring` 120) | Yes, and it includes planning because `clock` starts before the stage |
| Attempts per landed request | Retry links are the request text `try again on <job>` (`reading.rs` 811-828, 1024-1049) | Only for retries made that way. A request asked again in new words is a new chain |
| Five failure classes per 100 jobs | Introspection records and their classes (`store.rs` 256) | Partly: classes are assigned by a model, only failed or rejected jobs have them, and the class names are **unchecked** |
| First-round accept rate, planning's share of cost | `critique-1.json` and invocations with role `planner` or `critic` | Yes, once built |
| Which jobs are "planned" | A `job-plan-*.json` file | No, not alone: a job stopped by a planner failure has an invocation record but no plan file. Define planned as "has a `planner` invocation" |

**F18 (Medium): the pass-rate denominator makes the rule favor planning.**
A job stopped in planning has no verification (`failure()` at `run.rs` 1357-1366 leaves `closing.verification`
empty, and `close_dispatched` reports "failed before verification" at 1491). The baseline of 155 of 216 also
excludes the 21 closed jobs that never verified, so it is a rate among verified jobs. If planned jobs are scored
the same way, every job that planning stops leaves the denominator and the rate rises. **Fix:** define the first
measure per dispatched job (landed or verified-pass over all dispatched jobs), and recompute the baseline that way
before starting.

**F19 (Medium): at 30 jobs a 10-point rise cannot be told from noise.**
With a baseline near 72%, 30 jobs give a standard error of about 8 points (sqrt(0.72 × 0.28 / 30), an estimate),
so a 10-point rise is about one standard error. The rule would keep or drop planning on chance. **Fix:** either
raise the sample (the design's "about 30" to a number from a stated power, for example 100 jobs, an estimate) or
state that the result at 30 is a decision under uncertainty, and keep the rule's first step as "do not turn off
unless the rate falls".

**F20 (Medium): the cost rule and the design's own cost estimate nearly exclude each other.**
Section 10 estimates planning at 40 to 70% on the mean job; the rule keeps planning only if cost per landed job
rises by 25% or less. The share of jobs that land is 152 of 237 (64%, measured in the design). Cost per landed
job is cost per job divided by that share, so to stay within +25% the landed share must rise from 0.64 to at least
about 0.72 (at +40%) or 0.87 (at +70%) (estimate: 0.64 × 1.4 / 1.25 and 0.64 × 1.7 / 1.25). The pass-rate and cost
conditions are the same effect measured twice, and the cost condition is stricter. **Fix:** state the target
landed share and say which of the two conditions governs, or replace the cost condition with cost per landed job
against a stated ceiling in dollars.

**F21 (Low): the baseline is not like for like, and there is no concurrent control.**
The baseline counts all 237 closed jobs (change, study and others), the experiment tracks "planned change jobs",
and other design changes will land during the 30 jobs. **Fix:** recompute the baseline on change jobs only, and
record the Athena version on each job (the `job.json` `athena` field, `run.rs` 300) so shifts can be separated.

### What the design misses

- The recorded final producer prompt, planning inputs and the critic's inputs are not kept for later study (F6).
- The two caps (`planner_usd`, `critic_usd`) are fixed, but nothing says what happens if the reading's producer
  model is changed after the plan: a plan made for one model is followed by another.
- A study that later wants planning needs the pin mounted into the planner's snapshot; the design defers it,
  correctly, and the note in section 11 covers it.
- Worked example: no concrete plan record is shown. Job 1's Done-when depends on field names that exist only in
  prose. **Fix:** add one example `athena.job-plan/1` and `athena.critique/1` to section 5.

## 3. Summary of required fixes before building

1. F1: plan the invocation, file and spend plumbing through `produce_and_verify` and `close_dispatched`.
2. F5: decide what the verifier sees of the plan.
3. F6: record the producer's final prompt and keep the plan on the send-back.
4. F10: add `FindingSource::Critic` to job 1.
5. F11: decide the retry carry and the stopped job's score.
6. F14: add the deterministic-checks-only alternative.
7. F15, F16: list the unlisted files in job 1 and keep `reading_inputs` byte-identical.
8. Split job 3 and job 5 (sizing table).
9. F18 to F20: rewrite the keep rule's denominator, sample and cost condition.

## 4. What this review read

The design in full; `run.rs` 184-344 and 653-852, 1357-1500; `manage.rs` 220-301; `store.rs` 195-316;
`admission.rs` 30-145; `config.rs` 1-110, 300-470; `verify.rs` 200-310; `policy.rs` 60-140; `reading.rs` 205-330,
805-880, 1024-1049; `records.rs` 15-50, 210-220, 640-830, 1880-2040; `scoring.rs` 95-145; `briefs.rs` 325-360;
`athena-agent/src/fake.rs` 175-270; `athena-cli/tests/common/mod.rs` 20-50, 370-460 and `retry.rs` 40-65;
`athena-gate/src/invariant.rs` 595-671 (the landing check on invocation order, which planning invocations satisfy
because they come after the reading); `athena.toml` 24-33; `docs/ontology.toml` 21-23; and the format of
`parallel-jobs-review-1.md`. Not read: `status.rs`, `display.rs`, the rest of `verify.rs`, the introspection and
admission code beyond the lines above, and every record under `records/`.
