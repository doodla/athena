# Job 20261110-191432-ysr1: Hold a line after two non-landed attempts instead of retrying it

## Request (verbatim)

> The owner-approved 'stop and diagnose' and 'stop after three' loop changes both failed in planning on 7 Nov and have not run since, so today's thrash had no stop
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261110-180505-dbcm (key owner-approved-thrash-stops-failed-planning-never-retried), queued at 2026-11-10T18:05:05.019272274Z by the steward role.
> Rung of the requirements ladder: implementation.
> 
> ## Source
> 
> path records/interventions/2026-11/20261107-031955-bm2p.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261107-031955-bm2p",
>   "at": "2026-11-07T03:19:55.134538569Z",
>   "what": "Gave detailed instructions: with Sri's approval, queued four changes to Athena's improvement loop (a guaranteed turn for its own fixes, stop and diagnose repeated failures, allow relaxing a rule, measure wasted work)",
>   "why": "A two-reviewer design review found Loop B sees and diagnoses thrash but cannot act on it",
>   "class_of_work": "design",
>   "category": "gave-detailed-instructions",
>   "job": null,
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ## Evidence
> 
> ### 1. job 20261107-031917-r96w
> Request:
> ```
> Notice when one piece of wanted work keeps failing, stop retrying it, and diagnose the cause.
> 
> Each failed attempt is reviewed on its own today, so nothing notices the third try at the same thing. On 4 to 6 Nov, ten pieces of work took three or more attempts and spent about $73 of the $94 that went on attempts that did not land.
> 
> Change:
> - every job records which line of work it belongs to: a retry or a re-ask of a request joins the line of the request it repeats (the scope guard already groups retries and re-asks into lines; reuse that)
> - when a line has a second attempt that does not land, the drain does not retry it again; instead it queues one diagnosis job for the line
> - the diagnosis job reads every attempt on the line and sorts each failure into one of: defect in the work, requirement too strict or impossible under the rules, flaky test or environment, bad request, or collision with another landing; it then queues the one fix that matches (a corrected request, a change to the rule, a flaky-test fix, or a retry on the new code) and records its reasoning
> 
> Acceptance:
> - a test: a line with two attempts that did not land is not retried and gets exactly one diagnosis job
> - a test: the diagnosis job's output names a category for each attempt and queues exactly one follow-up
> - a test: a line with one failed attempt is retried as today
> - athena status shows each open line with its attempt count and state
> - all tests pass; CHANGELOG states only what the diff does
> 
> Files changed: the drain, the scope guard's line grouping, a new diagnosis role brief and its schema, status display, tests, CHANGELOG.md.
> ```
> Read as: Stop retrying a failing line and queue one diagnosis job (code). When a line of work (as grouped by the scope guard) has two attempts that did not land, the drain stops retrying it and queues exactly one diagnosis job. That job reads every attempt, assigns each failure one of five categories, and queues one matching follow-up. Jobs record their line, and `athena status` lists open lines with attempt count and state. The audience is the the person running Athena, who wants fewer wasted retries.
> Outcome: failed. planning stopped before any code was written: the critic judged that the reading misread the request: The request counts "attempts that do not land" and lists "collision with another landing" as a failure category. The reading narrowed "did not land" to records that are Rejected or Failed. An attempt that passed verification but failed to land (the drain's LandStep::NotLanded) counts only if its job closed with a non-landed outcome; if it is still blocked or open, it does not count. So a line can have two collision attempts and never be diagnosed. (round 1 of 5). The plan reuses group_lines and line_attempts properly. It keeps prompt budgets unchanged by checking the new frame against the verifier budget set, as the critic's frame is, and it bounds diagnosis spend with a create_new claim. Four design problems need fixing before code is written. (1) Filing the diagnosis invocation under the line's last job, which is already closed, breaks the landing gate's check that a closed job's totals match its invocation records (athena-gate reconcile.rs:105-131). It also adds the diagnosis cost to that job's spend and score. The steward's pattern (a fresh run id routed in store.rs) should be the design, not a fallback. (2) The retry follow-up joins the original line. When the scope guard has paused that line, which the plan's risks say is the usual case at two failures, the core refuses both the reading (reading.rs:1112) and the dispatch (run/mod.rs:1012). The drain's own-follow-up exception does not get past this, and the fake cannot show it. (3) Counting only Rejected or Failed records leaves out verified-but-not-landed attempts (collision), which the request names as a category, whenever their job is still blocked or open. (4) Every drain demonstration uses a fake whose line standing it computes itself. No test checks the real drain step reading real records, and the drain part of criterion 9 cannot fail. Athena's findings about files outside the deliverables are mostly files the change must touch to compile; I weighed them as minor. Findings: minor: crates/athena-core/src/diagnosis.rs is not among the reading's deliverables (files, crates/athena-core/src/diagnosis.rs); minor: crates/athena-core/src/run_pin_tests.rs is not among the reading's deliverables (files, crates/athena-core/src/run_pin_tests.rs); minor: crates/athena-core/src/store.rs is not among the reading's deliverables (files, crates/athena-core/src/store.rs); minor: crates/athena-core/src/reading.rs is not among the reading's deliverables (files, crates/athena-core/src/reading.rs); minor: crates/athena-agent/src/fake.rs is not among the reading's deliverables (files, crates/athena-agent/src/fake.rs); minor: crates/athena-cli/src/main.rs is not among the reading's deliverables (files, crates/athena-cli/src/main.rs); minor: crates/athena-cli/tests/drain.rs is not among the reading's deliverables (files, crates/athena-cli/tests/drain.rs); major: Filing the diagnosis invocation under the line's last job, already closed, breaks the gate's check that totals match the invocation records, and it inflates that job's spend and score. (Decision 6; Risks 'Records validation may refuse an invocation record...'; diagnosis.rs step 4); major: The retry-on-new-code follow-up joins the original line, so the core refuses it whenever the scope guard has paused that line; the drain-side exception does not help. (Decision 8 and Risk 'The retry follow-up joins the same line...'; Demonstrations criterion 1/2); major: Attempts that passed verification but failed to land (the collision category) are counted by the drain but may never reach the threshold, so such a line can end 'failed twice' with no diagnosis. (Decision 1 (failed_attempts counts Rejected and Failed only); Approach (1)); major: Every drain demonstration takes its line standing from a fake that computes the failure count itself. The real drain step, which reads real records, is never exercised. (Demonstrations criteria 1, 3, 9 (Test seam: Stub.line_standing)); minor: The exhaustive Role match in the CLI test fake is not listed. (Files / Dependents (Role::Diagnosis variant)); minor: The request says to 'queue one diagnosis job'; the plan runs it synchronously inside the drain with no job record. (Decision 5 (diagnosis runs inside the drain)); minor: A failed diagnosis stays claimed until the the person running Athena deletes a local file by hand, and only status reveals this. (Decision 4 / Risk 'A role failure leaves a claimed line'); minor: The extra files are mostly needed to compile; diagnosis.rs is a real scope addition. (Athena findings 1-7 (files outside deliverables))
> 
> 
> ### 2. job 20261107-212343-z26k
> Request:
> ```
> try again on 20261107-045801-vfa6
> ```
> Read as: Hold a line after two non-landed attempts instead of retrying it (code). Retry of job 20261107-045801-vfa6 (itself a retry of 20261107-031917-r96w), narrowed to the first piece as the user accepted: the drain stops retrying a line, grouped by the scope guard's group_lines, once it has two attempts that did not land, and holds it as awaiting diagnosis. The diagnosis role, schema, line field and status display stay follow-up requests. Audience: the the person running Athena who pays for repeated failed attempts.
> Outcome: failed. planning stopped before any code was written: the plan still needed revising in the last round (round 5 of 5). The plan matches the request and the accepted reading. The functions it names exist where it says, with small line drift: group_lines at 122, line_attempts at 161, verdict_of at 544, request_records at 616, line_pause at 784, Steps::paused/flagged at 703/711, Drain::prepare at 2084 with the paused map at 2126, and the attempts loop at 1855. The grouping change (a job that has not landed on the host does not close its text line) is necessary and correctly limited to the hold's grouping; the signal callers keep the wrapper. The demonstrations use real seams: production drain::drain and drain::plan, production hold_reason inside the Stub, RealSteps.lander (drain.rs:4033) for CollidingLander, and the helpers it cites (all exist). One soundness problem needs revising. The plan decides whether the host took a job from the existence of a pull-request link plus the absence of a landing record. It ignores the link's own `state` field, which records whether the host merged the job (pr.rs:26-35, set at pr.rs:360). Landing records come from the local records branch and read as empty when that branch is missing (store.rs:260-262), so jobs that did land can be counted as unlanded. Two smaller issues: the check at the top of each loop iteration would also refuse the free landing retry of the item's own blocked job, and the dependents list leaves out the production RealSteps callers in main.rs. Findings: major: The 'not taken by the host' test is guessed from whether a link file exists and a landing record is missing. The link file already stores whether the host merged the job. (Decision 1 / Step 3 (`Athena::unlanded_jobs`)); minor: The in-loop hold also refuses the free landing retry of the item's own blocked job. (Step 6 (check at the top of each `Drain::attempts` iteration)); minor: The list leaves out the production callers of RealSteps in main.rs. (Dependents: '`RealSteps::awaiting` now runs in every RealSteps drain or plan'); nit: Some cited line numbers are off. (Steps 5-6 line citations)
> 
> 
> ### 3. path records/jobs/2026-11/20261107-031917-r96w/outcome.json
> ```
> {
>   "schema": "athena.outcome/1",
>   "job": "20261107-031917-r96w",
>   "state": "failed",
>   "reason": "planning stopped before any code was written: the critic judged that the reading misread the request: The request counts \"attempts that do not land\" and lists \"collision with another landing\" as a failure category. The reading narrowed \"did not land\" to records that are Rejected or Failed. An attempt that passed verification but failed to land (the drain's LandStep::NotLanded) counts only if its job closed with a non-landed outcome; if it is still blocked or open, it does not count. So a line can have two collision attempts and never be diagnosed. (round 1 of 5). The plan reuses group_lines and line_attempts properly. It keeps prompt budgets unchanged by checking the new frame against the verifier budget set, as the critic's frame is, and it bounds diagnosis spend with a create_new claim. Four design problems need fixing before code is written. (1) Filing the diagnosis invocation under the line's last job, which is already closed, breaks the landing gate's check that a closed job's totals match its invocation records (athena-gate reconcile.rs:105-131). It also adds the diagnosis cost to that job's spend and score. The steward's pattern (a fresh run id routed in store.rs) should be the design, not a fallback. (2) The retry follow-up joins the original line. When the scope guard has paused that line, which the plan's risks say is the usual case at two failures, the core refuses both the reading (reading.rs:1112) and the dispatch (run/mod.rs:1012). The drain's own-follow-up exception does not get past this, and the fake cannot show it. (3) Counting only Rejected or Failed records leaves out verified-but-not-landed attempts (collision), which the request names as a category, whenever their job is still blocked or open. (4) Every drain demonstration uses a fake whose line standing it computes itself. No test checks the real drain step reading real records, and the drain part of criterion 9 cannot fail. Athena's findings about files outside the deliverables are mostly files the change must touch to compile; I weighed them as minor. Findings: minor: crates/athena-core/src/diagnosis.rs is not among the reading's deliverables (files, crates/athena-core/src/diagnosis.rs); minor: crates/athena-core/src/run_pin_tests.rs is not among the reading's deliverables (files, crates/athena-core/src/run_pin_tests.rs); minor: crates/athena-core/src/store.rs is not among the reading's deliverables (files, crates/athena-core/src/store.rs); minor: crates/athena-core/src/reading.rs is not among the reading's deliverables (files, crates/athena-core/src/reading.rs); minor: crates/athena-agent/src/fake.rs is not among the reading's deliverables (files, crates/athena-agent/src/fake.rs); minor: crates/athena-cli/src/main.rs is not among the reading's deliverables (files, crates/athena-cli/src/main.rs); minor: crates/athena-cli/tests/drain.rs is not among the reading's deliverables (files, crates/athena-cli/tests/drain.rs); major: Filing the diagnosis invocation under the line's last job, already closed, breaks the gate's check that totals match the invocation records, and it inflates that job's spend and score. (Decision 6; Risks 'Records validation may refuse an invocation record...'; diagnosis.rs step 4); major: The retry-on-new-code follow-up joins the original line, so the core refuses it whenever the scope guard has paused that line; the drain-side exception does not help. (Decision 8 and Risk 'The retry follow-up joins the same line...'; Demonstrations criterion 1/2); major: Attempts that passed verification but failed to land (the collision category) are counted by the drain but may never reach the threshold, so such a line can end 'failed twice' with no diagnosis. (Decision 1 (failed_attempts counts Rejected and Failed only); Approach (1)); major: Every drain demonstration takes its line standing from a fake that computes the failure count itself. The real drain step, which reads real records, is never exercised. (Demonstrations criteria 1, 3, 9 (Test seam: Stub.line_standing)); minor: The exhaustive Role match in the CLI test fake is not listed. (Files / Dependents (Role::Diagnosis variant)); minor: The request says to 'queue one diagnosis job'; the plan runs it synchronously inside the drain with no job record. (Decision 5 (diagnosis runs inside the drain)); minor: A failed diagnosis stays claimed until the the person running Athena deletes a local file by hand, and only status reveals this. (Decision 4 / Risk 'A role failure leaves a claimed line'); minor: The extra files are mostly needed to compile; diagnosis.rs is a real scope addition. (Athena findings 1-7 (files outside deliverables))",
>   "at": "2026-11-07T04:57:30.059950308Z",
>   "totals": {
>     "cost_all": {
>       "value": 2.7738581000000004,
>       "unit": "usd",
>       "provenance": "reported",
>       "basis": "sum of the reported costs of every invocation of the job"
>     },
>     "cost_execution": {
>       "value": 2.6474837000000004,
>       "unit": "usd",
>       "provenance": "reported",
>       "basis": "sum of the reported costs of the producer and verifier invocations"
>     },
>     "wall_time_execution": {
>       "value": 569.530618639,
>       "unit": "seconds",
>       "provenance": "measured",
>       "basis": "athena clock, from dispatch to the final verdict"
>     },
>     "tokens": {
>       "claude-opus-5-5": {
>         "input": 48,
>         "output": 15817,
>         "cache_read": 1221349,
>         "cache_write": 76775,
>         "cost_usd": 1.1750018000000002
>       },
>       "claude-sonnet-5-5": {
>         "input": 82,
>         "output": 54044,
>         "cache_read": 3237723,
>         "cache_write": 183620,
>         "cost_usd": 1.5988563000000002
>       }
>     },
>     "invocations": 3
>   },
>   "branch": "the person running Athena/next5",
>   "base_commit": "83bd84c95d5f37bec089257821d8addae7c11c4d",
>   "candidate_commit": null,
>   "after_blocked": false,
>   "through_commit": null
> }
> ```
> 
> ### 4. path records/jobs/2026-11/20261107-212343-z26k/outcome.json
> ```
> {
>   "schema": "athena.outcome/1",
>   "job": "20261107-212343-z26k",
>   "state": "failed",
>   "reason": "planning stopped before any code was written: the plan still needed revising in the last round (round 5 of 5). The plan matches the request and the accepted reading. The functions it names exist where it says, with small line drift: group_lines at 122, line_attempts at 161, verdict_of at 544, request_records at 616, line_pause at 784, Steps::paused/flagged at 703/711, Drain::prepare at 2084 with the paused map at 2126, and the attempts loop at 1855. The grouping change (a job that has not landed on the host does not close its text line) is necessary and correctly limited to the hold's grouping; the signal callers keep the wrapper. The demonstrations use real seams: production drain::drain and drain::plan, production hold_reason inside the Stub, RealSteps.lander (drain.rs:4033) for CollidingLander, and the helpers it cites (all exist). One soundness problem needs revising. The plan decides whether the host took a job from the existence of a pull-request link plus the absence of a landing record. It ignores the link's own `state` field, which records whether the host merged the job (pr.rs:26-35, set at pr.rs:360). Landing records come from the local records branch and read as empty when that branch is missing (store.rs:260-262), so jobs that did land can be counted as unlanded. Two smaller issues: the check at the top of each loop iteration would also refuse the free landing retry of the item's own blocked job, and the dependents list leaves out the production RealSteps callers in main.rs. Findings: major: The 'not taken by the host' test is guessed from whether a link file exists and a landing record is missing. The link file already stores whether the host merged the job. (Decision 1 / Step 3 (`Athena::unlanded_jobs`)); minor: The in-loop hold also refuses the free landing retry of the item's own blocked job. (Step 6 (check at the top of each `Drain::attempts` iteration)); minor: The list leaves out the production callers of RealSteps in main.rs. (Dependents: '`RealSteps::awaiting` now runs in every RealSteps drain or plan'); nit: Some cited line numbers are off. (Steps 5-6 line citations)",
>   "at": "2026-11-07T21:59:47.543672751Z",
>   "totals": {
>     "cost_all": {
>       "value": 9.622521,
>       "unit": "usd",
>       "provenance": "reported",
>       "basis": "sum of the reported costs of every invocation of the job"
>     },
>     "cost_execution": {
>       "value": 9.4737838,
>       "unit": "usd",
>       "provenance": "reported",
>       "basis": "sum of the reported costs of the producer and verifier invocations"
>     },
>     "wall_time_execution": {
>       "value": 2120.547459108,
>       "unit": "seconds",
>       "provenance": "measured",
>       "basis": "athena clock, from dispatch to the final verdict"
>     },
>     "tokens": {
>       "claude-opus-5-5": {
>         "input": 246,
>         "output": 80037,
>         "cache_read": 6016421,
>         "cache_write": 347788,
>         "cost_usd": 5.5873121999999995
>       },
>       "claude-sonnet-5-5": {
>         "input": 196,
>         "output": 166863,
>         "cache_read": 6050388,
>         "cache_write": 440287,
>         "cost_usd": 4.0352087999999995
>       }
>     },
>     "invocations": 11
>   },
>   "branch": "the person running Athena/next5",
>   "base_commit": "04e4505408ffcccce6772024d1af2dccf1f447dd",
>   "candidate_commit": null,
>   "after_blocked": false,
>   "through_commit": null
> }
> ```
> 
> ### 5. path records/interventions/2026-11/20261110-165614-yn1h.json
> ```
> {
>   "schema": "athena.intervention/1",
>   "id": "20261110-165614-yn1h",
>   "at": "2026-11-10T16:56:14.465682684Z",
>   "what": "Observation: a chain of wrong-cause fixes. Since 12:00 UTC five jobs were rejected because the cause they fixed is not in the records: the acceptance-point check (twice), 'show cited jobs' files to the reader' (already done), 'resolve every cited job to its reading' (cause contradicted), and 'print deliverable counts for carried jobs' (false fact, /bin/bash.86). The last two were queued by the error review of the job before, so each rejection breeds the next request on the same false premise.",
>   "why": "This is the thrash pattern from 6 Nov: Athena keeps reworking one topic from a misdiagnosis. The stop-and-diagnose and stop-after-three lines are both paused, so nothing breaks the chain. Reporting for the steward; not closing the requests by hand.",
>   "class_of_work": "other",
>   "category": "other",
>   "job": "20261110-163344-3gaa",
>   "source": "user",
>   "with_staged_changes": false
> }
> ```
> 
> ### 6. job 20261110-181101-x2ek
> Request:
> ```
> Queue each of the two owner-approved loop changes again as a fresh request: stop and diagnose repeated failures (from job 20261107-031917-r96w) and hold a line after repeated failures (from job 20261107-212343-z26k). Keep each original goal and acceptance. Add the critic's major findings from the job's last outcome as extra acceptance bullets. For r96w: do not file the diagnosis invocation under an already-closed job, count verified-but-not-landed attempts, and test the real drain step on real records. For z26k: decide 'taken by the host' from the pull-request link's state field, not from whether a landing record exists. Source both from the steward so they get the improvement turn.
> 
> Acceptance:
> - The change is made as described: The owner-approved 'stop and diagnose' and 'stop after three' loop changes both failed in planning on 7 Nov and have not run since, so today's thrash had no stop
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/interventions/2026-11/20261107-031955-bm2p.json
> ```
> Outcome: none recorded yet.

## Reading (accepted by the user)

Carry out the approved stop-after-repeated-failure loop change as its first piece (retry of jobs 20261107-212343-z26k, 20261107-045801-vfa6 and 20261107-031917-r96w): the drain stops retrying a scope-guard line once it has two attempts that did not land, and holds it as awaiting diagnosis. Whether the host took a job is decided from the pull-request link's state field. The diagnosis role, line field and status display stay follow-ups.

Assumptions:
- Goal and audience: the person running Athena, who pays for repeated failed attempts at one piece of work and saw no stop in the 10 Nov thrash.
- Job 20261107-031917-r96w (failed in planning, no candidate): its diagnosis role, schema, line field and status display are dropped from this change as follow-ups; its findings on diagnosis cost filing, paused-line retry and the status display no longer apply because no diagnosis runs here. Its finding that verified-but-not-landed (collision) attempts must count IS addressed: they count toward the two attempts.
- Job 20261107-212343-z26k (failed in planning round 5, no candidate): each finding is fixed. 'Taken by the host' is decided from the pull-request link's state (LinkState::Landed means landed) together with landing records, and either source saying landed removes the job from the unlanded set. The in-loop hold skips the item's own blocked-job landing retry. The main.rs RealSteps callers are covered by the drain tests. Line citations are corrected by reading the code.
- Job 20261107-045801-vfa6 (dispatched, no candidate, nothing kept): superseded by this reading.
- Code survey: crates/athena-cli/src/drain.rs (Drain::attempts loop, Drain::prepare, RealSteps), crates/athena-core/src/scope_guard.rs (group_lines, line_attempts, verdict_of, line_pause) and crates/athena-core/src/pr.rs (PrLink, LinkState). No new record or data file is added, so no check_record_file or AnyRecord change is needed. The hold is measured by running the real drain::drain on real records in crates/athena-cli/tests/drain.rs.
- Environment: none. No CI, push, PR or records-branch write; the pull-request link is read only from local job files.
- No governance file (athena.toml, defaults.toml, workflows) changes. The hold adds no refusal to readings; it only stops drain retries, so no reading-refusal rate applies.
- Consumers: none found, because no config deliverable exists.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`: a test through the real drain::drain on real records shows a line with two attempts that did not land is not retried a third time and ends held as awaiting diagnosis.
2. [behavior] Run `cargo test -p athena-cli --test drain`: a test shows a line with one failed attempt is still retried as today.
3. [behavior] Run `cargo test -p athena-core scope_guard`: a test shows two verified-but-not-landed (collision) attempts on one line count as two attempts that did not land, and a job whose pull-request link state is Landed with no landing record is not counted as unlanded.
4. [behavior] Run `cargo test -p athena-cli --test drain`: a test shows an item whose own first attempt is blocked on landing still gets its free landing retry rather than being held.
5. [content] crates/athena-cli/src/drain.rs and crates/athena-core/src/scope_guard.rs read the link's `state` field (LinkState::Landed) in the diff, not only whether a link file exists.
6. [content] CHANGELOG.md lists only what the diff does: the hold after two non-landed attempts, with the diagnosis job named as a follow-up not done here.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no unrelated behavior change.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Diagnosis role brief (crates/athena-context/prompts/diagnosis.md), its output schema, the five-category sort and the queued follow-up
- Recording a line value on every job and the athena status display of open lines
- Running diagnosis under its own run id, a paused-line retry exception, releasing a failed diagnosis by command
- Changing scope guard signals T1 to T5 or thresholds; editing athena.toml, defaults.toml or workflows
- Backfilling old records; editing shared contract, reader or verifier prompts
- Editing records/ files, including records/interventions/2026-11/20261107-031955-bm2p.json

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the drain holding a failing line through the real drain on real records.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion included.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Narrowed to the hold, the first piece, as earlier accepted.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 17 cases in 17 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) against it. A mismatch refuses the reading and prints the re...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

34 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-191432-ysr1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-191432-ysr1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-cli --test drain`: a test through the real drain::drain on real records shows a line with two attempts that did not land is not retried a third time and ends held as awaiting diagnosis.
Run `cargo test -p athena-cli --test drain`: a test shows a line with one failed attempt is still retried as today.
Run `cargo test -p athena-core scope_guard`: a test shows two verified-but-not-landed (collision) attempts on one line count as two attempts that did not land, and a job whose pull-request link state is Landed with no landing record is not counted as unlanded.
Run `cargo test -p athena-cli --test drain`: a test shows an item whose own first attempt is blocked on landing still gets its free landing retry rather than being held.
crates/athena-cli/src/drain.rs and crates/athena-core/src/scope_guard.rs read the link's `state` field (LinkState::Landed) in the diff, not only whether a link file exists.
CHANGELOG.md lists only what the diff does: the hold after two non-landed attempts, with the diagnosis job named as a follow-up not done here.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no unrelated behavior change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
