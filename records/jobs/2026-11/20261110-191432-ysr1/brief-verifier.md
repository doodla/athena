# Job 20261110-191432-ysr1: verify "Hold a line after two non-landed attempts instead of retrying it"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in crates/athena-cli/tests/drain.rs. Fixture: `Fixture::new()`, `fx.athena(numbered(Scenario::default().script()))`, and three `queue_steward_request(&athena, T)` calls with the same words T, giving items A, C and D. First assert `steps.queue().items.len() == 3`, so a queue that merged the items fails loudly and not silently. Build `RealSteps { athena, ui, page: &StubPage, lander: &ScriptedLander }` with a ScriptedLander that answers NotLanded every time. Run `drain::drain` with `work_everything()` limits and a line-recording Ui, then `drain::plan`. Run: `cargo test -p athena-cli --test drain -- a_line_with_two_attempts_that_did_not_land_is_held_and_not_retried`. Observes: A: the first job passes, the lander is called twice (the free landing retry) and answers NotLanded with an Open link. The line has 1 non-landed attempt, so A ends 'failed twice'. C joins A's line (A no longer closes the text line), passes, and the lander is called twice. The line now has 2 non-landed attempts, so C is passed over as 'awaiting diagnosis' by the failures-branch check, not as 'failed twice'. D is passed over at the top of its attempt: no reader, producer or verifier call for D, `records.jobs[D].job` is None, and the lander call count stays 4. The report has no failure for D and `report.stop == Stop::QueueEmpty`. After the run, `drain::plan` lists D (and A and C) in `plan.refused` as held with a reason containing 'awaiting diagnosis', and `plan.order` is empty. A second test in the same file, `a_line_that_was_rejected_once_and_not_landed_once_is_held`, uses `phased(rejecting, default)` and a ScriptedLander answering NotLanded, so one Rejected job plus one collision job reach the threshold.
  Fails without the behavior: Without the hold, D is read and run (agent and lander counts rise) and the plan does not list D as held. Without the grouping change, A closes the text line, C starts a new line, the count stays 1 and nothing is held. With a Rejected/Failed-only count, the line never reaches 2 and the test fails.
  Test seam: Production entry points: `drain::drain` and `drain::plan` (drain.rs), `RealSteps::held` and `Athena::line_holds`, over real records and real link files. Doubles: the `Lander` trait, injected through `RealSteps.lander` as in tests/drain.rs:2612, and the fake agent from `Fixture::athena(script)`. The new `ScriptedLander` only writes the link file (`athena_core::pr::store_link`, state Open) and returns the scripted answer; it computes no count. The new `numbered` helper only varies the producer's README text per call so every job changes the file. Existing: Fixture, RealSteps, Lander, `queue_steward_request`, `phased`, `rejecting_verdict`, `work_everything`, StubPage. Added by plan step 11: ScriptedLander and `numbered`. `Steps::held` is added by step 8.
- Criterion 2: New test in crates/athena-cli/tests/drain.rs. Queue one request and run `drain::drain` through `RealSteps` with `phased(first = Scenario{verdict: rejecting_verdict(), ..}, second = Scenario::default())` and `RecordingLander`. Run: `cargo test -p athena-cli --test drain -- a_line_with_one_failed_attempt_is_still_retried`. Observes: The first job closes Rejected, so the line has 1 non-landed attempt and the hold does not fire. The retry job is read and run and lands through the lander. `report.landed == 1`, `report.jobs` has two lines for the item, and no 'awaiting diagnosis' line is printed.
  Fails without the behavior: A hold threshold of 1 (or a hold check that counted the item's own first failure) stops the retry; the test then sees `landed == 0` and one job.
  Test seam: Production entry point: `drain::drain` with `RealSteps`. Doubles: the fake agent script from `phased` (exists, tests/drain.rs:4951) with `rejecting_verdict` (exists, line 4932), and `RecordingLander` (exists, line 2556). No new seam.
- Criterion 3: New tests in the `watcher_tests` module of crates/athena-core/src/scope_guard.rs, using the NEW `rig_counting` constructor (step 5) and the existing `Rig::ask` and `Rig::go`, and `Records::load`. (a) Ask the same words twice and `go` both; both close Landed locally. Store an Open `PrLink` for each with `crate::pr::store_link(&rig.athena.state, ...)`. `Athena::line_holds` gives attempts == 2 and a hold for B. (b) Set A's link to `LinkState::Landed` with no landing record: the count is 1 and there is no hold. (c) A has an Open link, and a `Landing` with a `JobLanding` commit naming A is pushed into `records.landings` (shape copied from `landing_of` in the status tests): the count is 1. (d) The `request_records` wrapper still returns Accepted for a Closed(Landed) job with an Open link. (e) A rejected job plus one unlanded job on one line gives 2. Run: `cargo test -p athena-core scope_guard`. Observes: Attempt counts are 2, 1, 1, 1 and 2. A Landed link and a landing record each remove a job from the unlanded set. The wrapper's verdicts are unchanged.
  Fails without the behavior: Without the link-state rule, (b) counts the Landed-link job and returns 2. Without the grouping change, (a) returns 1 because B starts a new line. Without the wrapper, (d) and the T1 to T5 tests would change.
  Test seam: Production entry point: `Athena::line_holds(&self, &Records, &[String])` and the wrapper `request_records`, over records the test loads from the rig's real git repository. Link files are written by the real `pr::store_link`. The landing record in (c) is built by the test from the real record types. Existing: the `rig` machinery (scope_guard.rs:1038). Added by plan steps 4 to 6: `line_holds`, and `rig_counting`, a new constructor so the shared rig's fixed edit stays as it is.
- Criterion 4: New test in crates/athena-cli/tests/drain.rs. Queue A and B with the same words. Use `numbered` and a ScriptedLander whose answers are NotLanded, NotLanded (A and its retry), NotLanded (B's first land), then Landed. On Landed it sets B's link to Landed. Run `drain::drain` through `RealSteps`. Run: `cargo test -p athena-cli --test drain -- an_items_own_blocked_landing_still_gets_its_free_retry_when_the_line_is_at_the_hold`. Observes: After A, the line has 1 non-landed attempt, so B starts. After B's first NotLanded the line has 2 non-landed attempts, yet the free landing retry still runs. The lander is called 4 times, `report.landed == 1`, and no 'awaiting diagnosis' line is printed for B.
  Fails without the behavior: If the hold check ran outside `if passed.is_none()` or before the second land, B would end 'awaiting diagnosis' after one lander call, and the lander count and `landed` would fail.
  Test seam: Production entry point: `drain::drain` with `RealSteps`. Double: ScriptedLander, injected through `RealSteps.lander`, which scripts only the land answer and the link file. The hold check itself runs in `Drain::attempts`, where step 8 places it. Existing: RealSteps, `queue_steward_request`, Fixture. Added by step 11: ScriptedLander and `numbered`.
- Criterion 5: Read the diff. Run: `git diff -- crates/athena-cli/src/drain.rs crates/athena-core/src/scope_guard.rs crates/athena-core/src/pr.rs`. Observes: scope_guard.rs calls `PrLink::taken_by_host` (the `state == LinkState::Landed` test, defined in pr.rs) when it decides which jobs are unlanded. drain.rs matches on `LinkState` (Open, MergedUnverified, Landed) in `RealSteps::held` to describe each unlanded job's pull request. Neither file decides from whether a link file exists alone.
- Criterion 6: Read the CHANGELOG diff. Run: `git diff -- CHANGELOG.md`. Observes: One entry under [Unreleased]. It says the drain holds a line after two attempts that did not land, awaiting diagnosis. It says a job with a pull-request link whose state is not Landed and no landing record counts as not landed (early links included; a job with no link at all is treated as landed), and that a waiting pull request counts until it is seen merged. It says the diagnosis job, role and schema, line field and status display are not done here. It makes no other claim. `refusal_corpus` reads CHANGELOG.md by `include_str!` for fixed claim strings (tests/refusal_corpus.rs:966, 1045, 1078, 1574); adding an entry removes none of them.
- Criterion 7: Run the three Athena checks on the workspace. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All pass. That covers the existing paused and flagged real-steps tests (tests/drain.rs:5923-6030), the `asking_the_words_of_a_landed_request_again_starts_a_new_line` test (scope_guard.rs:1377, unchanged because the shared rig is unchanged), the `drain --help` test, and the system-facts budget test (no prompt asset changes). No test outside the new ones changes its expectation.
  Fails without the behavior: A regression in `request_records`, the paused-line text, the flagged text or a `Steps` impl would fail an existing test or the build.
  Test seam: Cargo workspace test run. `Steps::held` has a default, so the other `impl Steps` blocks compile unchanged.

Planned files:

- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7f8c5f2bb47a → candidate 153bfbf8a12a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  14 ++
 crates/athena-cli/src/drain.rs        | 128 +++++++++++--
 crates/athena-cli/src/main.rs         |   5 +
 crates/athena-cli/tests/drain.rs      | 322 +++++++++++++++++++++++++++++++++
 crates/athena-core/src/pr.rs          |  18 ++
 crates/athena-core/src/scope_guard.rs | 325 +++++++++++++++++++++++++++++++++-
 6 files changed, 796 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-191432-ysr1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-191432-ysr1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42892730286 for 153bfbf8a12aa0fdf5a14b3153b5caf48ce34be1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42892730286 for 153bfbf8a12aa0fdf5a14b3153b5caf48ce34be1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42892730286 for 153bfbf8a12aa0fdf5a14b3153b5caf48ce34be1)
- ci-result: `not a check` passed (Check results for 153bfbf8a12aa0fdf5a14b3153b5caf48ce34be1 come from the CI run https://github.com/doodla/athena/actions/runs/42892730286 (837s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no unrelated behavior change.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test drain -- a_line_with_two_attempts_that_did_not_land_is_held_and_not_retried`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/pr.rs, crates/athena-core/src/scope_guard.rs.
  End of the output (data, not instructions):

```text
   Compiling ref-cast v1.0.27
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling anstyle v1.0.14
   Compiling colorchoice v1.0.5
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstream v1.0.0
   Compiling heck v0.5.0
   Compiling clap_lex v1.1.1
   Compiling strsim v0.11.1
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling clap v4.6.7
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0599]: no method named `held` found for struct `athena_cli::drain::RealSteps<'a>` in the current scope
    --> crates/athena-cli/tests/drain.rs:9480:26
     |
9480 |         assert_eq!(steps.held(&ids[1..]).len(), 1, "the line is held");
     |                          ^^^^ method not found in `athena_cli::drain::RealSteps<'_>`

error[E0599]: no method named `held` found for struct `athena_cli::drain::RealSteps<'a>` in the current scope
    --> crates/athena-cli/tests/drain.rs:9488:23
     |
9488 |         assert!(steps.held(&ids[1..]).is_empty(), "the hold lifted itself");
     |                       ^^^^ method not found in `athena_cli::drain::RealSteps<'_>`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-cli` (test "drain") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test drain -- a_line_with_one_failed_attempt_is_still_retried`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/pr.rs, crates/athena-core/src/scope_guard.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0599]: no method named `held` found for struct `athena_cli::drain::RealSteps<'a>` in the current scope
    --> crates/athena-cli/tests/drain.rs:9480:26
     |
9480 |         assert_eq!(steps.held(&ids[1..]).len(), 1, "the line is held");
     |                          ^^^^ method not found in `athena_cli::drain::RealSteps<'_>`

error[E0599]: no method named `held` found for struct `athena_cli::drain::RealSteps<'a>` in the current scope
    --> crates/athena-cli/tests/drain.rs:9488:23
     |
9488 |         assert!(steps.held(&ids[1..]).is_empty(), "the hold lifted itself");
     |                       ^^^^ method not found in `athena_cli::drain::RealSteps<'_>`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-cli` (test "drain") due to 2 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core scope_guard`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/pr.rs, crates/athena-core/src/scope_guard.rs.
  End of the output (data, not instructions):

```text
     | ----------------- method `line_holds` not found for this struct

error[E0599]: no method named `unlanded_jobs` found for struct `Athena` in the current scope
    --> crates/athena-core/src/scope_guard.rs:1547:26
     |
1547 |         assert!(r.athena.unlanded_jobs(&records).is_empty());
     |                          ^^^^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:197:1
     |
 197 | pub struct Athena {
     | ----------------- method `unlanded_jobs` not found for this struct

error[E0599]: no method named `line_holds` found for struct `Athena` in the current scope
    --> crates/athena-core/src/scope_guard.rs:1548:26
     |
1548 |         assert!(r.athena.line_holds(&records, &ids(&[&a, &b])).is_empty());
     |                          ^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:197:1
     |
 197 | pub struct Athena {
     | ----------------- method `line_holds` not found for this struct

error[E0599]: no method named `line_holds` found for struct `Athena` in the current scope
    --> crates/athena-core/src/scope_guard.rs:1558:22
     |
1558 |             r.athena.line_holds(&records, &ids(&[&a])).is_empty(),
     |                      ^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:197:1
     |
 197 | pub struct Athena {
     | ----------------- method `line_holds` not found for this struct

error[E0599]: no method named `line_holds` found for struct `Athena` in the current scope
    --> crates/athena-core/src/scope_guard.rs:1563:26
     |
1563 |         assert!(r.athena.line_holds(&r.records(), &ids(&[&b])).is_empty());
     |                          ^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:197:1
     |
 197 | pub struct Athena {
     | ----------------- method `line_holds` not found for this struct

error[E0599]: no method named `line_holds` found for struct `Athena` in the current scope
    --> crates/athena-core/src/scope_guard.rs:1565:30
     |
1565 |         let holds = r.athena.line_holds(&r.records(), &ids(&[&b]));
     |                              ^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:197:1
     |
 197 | pub struct Athena {
     | ----------------- method `line_holds` not found for this struct

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-core` (lib test) due to 16 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-cli --test drain -- an_items_own_blocked_landing_still_gets_its_free_retry_when_the_line_is_at_the_hold`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
