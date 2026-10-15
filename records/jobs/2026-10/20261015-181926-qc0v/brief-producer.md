# Job 20261015-181926-qc0v: Reconcile invocation tokens with modelUsage, with a real negative test

## Request (verbatim)

> Retries of rejected work fail again: the token reconciliation work was rejected three times and the drain source check twice
> 
> A person approved this proposal from the steward role. Carry the work out as an ordinary change.
> Everything the proposal cites is copied below; nothing here points at a file you cannot open.
> 
> Proposal: 20261015-173135-37jx (key retry-repeats-same-rejection-finding), queued at 2026-10-15T17:31:35.006790924Z by the steward role.
> Rung of the requirements ladder: design-decisions.
> 
> ## Source
> 
> path records/introspections/2026-10/20261015-163343-435x.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261015-163343-435x",
>   "at": "2026-10-15T16:33:43.920567241Z",
>   "trigger": "rejection",
>   "job": "20261015-160432-g1tq",
>   "intervention": null,
>   "class_of_work": "code",
>   "category": "rejected",
>   "what_happened": "The job asked to reconcile invocation token totals with modelUsage, and to add an end-to-end test. It was rejected. The verifier failed 1 of 5 acceptance criteria. The unmet criterion required a named negative cargo test. The test must build the record from a result event whose totals really disagree with modelUsage, through the same path as the positive test, and assert that check_invocation returns a Reconciliation violation. The one major finding was that the test a_result_whose_totals_disagree_with_model_usage_is_flagged_end_to_end (crates/athena-core/src/invocation.rs) never builds a record from a truly disagreeing event. So it does not show the violation is caught. The event lists no failed checks. The reconciliation code may be right. The negative test does not prove it.",
>   "failure_class": "20261014-184842-whyd",
>   "new_class": false,
>   "earlier_cases": [
>     "20261015-081905-xw6c"
>   ],
>   "decisions_matched": [],
>   "cause": "A test that has the right name and the right assertion can still pass without exercising the failing condition. Nothing required a negative test to show it can fail: for example, that its input really violates the invariant, or that it fails when the check is removed. This is the same job family as the earlier token-reconciliation case (10ms), which was rejected for a missing end-to-end test. The retry added a test that looked like the criterion but the input was not truly disagreeing. The build only checked that the test exists and passes. Only the final verifier looked at what the input contained. This is an inference from one finding, since the stored fields do not show the test body.",
>   "prevention_kind": "code-check",
>   "prevention": "A mutation-style check on negative tests that a criterion names. The check would run the named test against the code with the guard disabled or the invariant inverted, and require the test to fail there. It would also require the test to assert, before calling check_invocation, that the built record's totals differ from the modelUsage sum. A negative test whose input does not break the invariant would then be caught during the build, not at the final verdict. A default of writing the disagreeing fixture first and watching the test fail would support it.",
>   "proposed_request": "Fix the negative end-to-end test in crates/athena-core/src/invocation.rs, a_result_whose_totals_disagree_with_model_usage_is_flagged_end_to_end. It must build the invocation record from a result event whose top-level usage totals really differ from the sum of modelUsage. It must use the same parsing and record-building path as the positive reconciliation test, and then assert that check_invocation returns a Reconciliation violation. First assert in the test that the record's totals differ from the modelUsage sum, so the input is known to be bad. Show that the test fails when the reconciliation check is temporarily disabled, then restore the check. Run cargo test and report the results. Change no other behavior.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.0302141,
>     "input_tokens": 2,
>     "output_tokens": 1097
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [
>     "20261015-152122-10ms"
>   ],
>   "from_jev": false
> }
> ```
> 
> ## Evidence
> 
> ### 1. job 20261015-124217-6xhe
> Request:
> ```
> try again on 20261015-104515-8wqq
> ```
> Read as: Make the explainer site introduce Athena and fix newcomer findings (code). Rework the generated explainer page so a newcomer at doodla.github.io/athena first learns what Athena is and that the page is an example study of firstmate. Add a next-step section, linked claim sources, and a plain-language f2 limit. The producer must run the gates and map each criterion to evidence before reporting.
> Outcome: rejected. the final verdict is fail: the verifier's verdict is fail; 1 of 7 criteria unmet
> 
> 
> ### 2. job 20261015-145809-7565
> Request:
> ```
> The gate refused Athena's own landing of verified job 20261015-131540-2fsa: 'usage records reconcile' says the producer used 90/52656/3634696/134508 tokens but the invocation records 6/3136/275219/3821. Athena's invocation records and the gate must agree on what an invocation used.
> ```
> Read as: Make invocation token totals agree with per-model usage and the gate (code). The gate's reconciliation rule compares the requested model's modelUsage tokens with the invocation's tokens, which come from the result event's top-level usage. In job 2fsa those disagreed (90/52656/3634696/134508 vs 6/3136/275219/3821). I read the request as: fix the root cause so records Athena writes always reconcile, and have the gate check the same quantity. The audience is the Athena owner, who needs verified jobs to land.
> Outcome: rejected. the final verdict is fail: the verifier's verdict is fail; 1 of 5 criteria unmet; 1 blocker or major finding(s)
> 
> 
> ### 3. job 20261015-160432-g1tq
> Request:
> ```
> try again on 20261015-145809-7565
> ```
> Read as: Reconcile invocation tokens with modelUsage, with end-to-end test (code). Retry of job 145809-7565: make the invocation's measured tokens and the gate's reconciliation rule use one definition (modelUsage), starting from the base because the earlier changes were discarded. This time add a test that runs a result event through parsing, the invocation record and check_invocation, plus a negative case. The audience is the Athena owner, who needs verified jobs to land.
> Outcome: rejected. the final verdict is fail: the verifier's verdict is fail; 1 of 5 criteria unmet; 1 blocker or major finding(s)
> 
> 
> ### 4. job 20261015-064153-wekz
> Request:
> ```
> Athena's steward and audience queue work, but every queued item waits for me to run it, land it and rate the result again. Work through your own queue yourself, within a budget I set, until the outcome you are working toward passes its check or the budget is spent, and tell me where it stopped.
> ```
> Read as: Add a drain command that works the queue within a budget (code). Add a CLI command (`drain`) that runs queued steward- and audience-queued requests itself: run, land and rate each item. It stops when the work's own check passes (for the explainer, the audience rates the live page 4 or more), when the user-set budget is spent, or when the same item fails twice (thrashing). It then reports where it stopped. The audience is the repository owner, who now runs go, land and rate by hand.
> Outcome: rejected. the final verdict is fail: the verifier's verdict is fail; 1 blocker or major finding(s)
> 
> 
> ### 5. job 20261015-152305-6h8k
> Request:
> ```
> Drain reported 'target met' after landing 20261015-141831-hyb4, a defaults change that did not touch the explainer, and its check against the pinned source passed a page that an independent reader found wrong on three facts (teardown can reopen instead of close, yolo-on self-merge, which handoff frees a place). Only a landing that changed the outcome may count, and the source check must catch statements the source contradicts or leaves out.
> ```
> Read as: Count only outcome-changing landings; make drain's source check semantic (code). Two faults in `athena drain` (crates/athena-cli/src/drain.rs). (1) A landing that did not change the explainer's content still allows "target met"; only a landing whose diff touches what the page is built from may count. (2) The pinned-source check passes pages with wrong facts because it only tests the study models for self-consistency; it must compare the page's statements with the pinned source and fail on any statement the source contradicts or omits. The audience is the person running drain who trusts "target met".
> Outcome: rejected. the final verdict is fail: the verifier's verdict is fail; 1 blocker or major finding(s)
> 
> 
> ### 6. path records/introspections/2026-10/20261015-152122-10ms.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261015-152122-10ms",
>   "at": "2026-10-15T15:21:22.931027458Z",
>   "trigger": "rejection",
>   "job": "20261015-145809-7565",
>   "intervention": null,
>   "class_of_work": "code",
>   "category": "rejected",
>   "what_happened": "The job asked to make invocation token totals agree with per-model usage and the gate. It was rejected. The verifier failed 1 of 5 acceptance criteria. The unmet criterion required a new cargo test. The test would feed a result event whose top-level usage differs from modelUsage (main thread plus subagent usage on the requested model). It would show the resulting invocation record passes check_invocation with no Reconciliation violation. The one major finding was that no test runs a result event through to the invocation record and check_invocation (stream.rs and reconcile.rs tests). The event lists no failed checks. The reconciliation change may be correct. What was missing was the end-to-end test that proves it.",
>   "failure_class": "20261014-172333-6fy0",
>   "new_class": false,
>   "earlier_cases": [
>     "20261015-131120-cjyk",
>     "20261015-013952-zyzm",
>     "20261014-213402-g1b6"
>   ],
>   "decisions_matched": [],
>   "cause": "Nothing in the build compared the work to each acceptance criterion before handover. The criterion named a specific test shape: a result event through parsing, the invocation record, and the gate. The build likely tested the pieces separately (stream parsing, and reconcile on hand-built records). Each piece passed, so the build seemed done while the joined path stayed untested. No step turned each criterion that asks for a test into a named test that must exist and pass. So the gap surfaced only at the final verdict, when rejection was the only option. This is an inference, because the stored fields show only the criterion and the finding.",
>   "prevention_kind": "code-check",
>   "prevention": "Add a pre-handover criterion check to the build. Before a job can finish, the builder must produce a table mapping each acceptance criterion to its evidence. For a criterion that asks for a test, the evidence is a named test that exists and passes, and a check confirms that name appears in the cargo test output. If any criterion has no evidence, the job returns to the build instead of going to the final verifier. This would have caught the missing end-to-end test, and it covers the whole class of criteria unmet at final verification.",
>   "proposed_request": "Add the missing end-to-end test for invocation token reconciliation. In the athena-agent and athena-gate crates, add a cargo test that feeds a stream result event whose top-level usage differs from its modelUsage. The event should include main-thread usage plus subagent usage on the requested model. The test must parse the event into the invocation record and run check_invocation on it. It must assert that there is no Reconciliation violation. Also add a negative case: a result event whose totals really disagree with modelUsage must produce a Reconciliation violation, which shows the test can fail. Do not change reconciliation behavior unless the new test exposes a bug. In the final report, include a table mapping each acceptance criterion to the named test or command output that shows it is met. Run cargo test and the repo gate commands, and record their results in the report.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.03327,
>     "input_tokens": 2,
>     "output_tokens": 1117
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 7. path records/introspections/2026-10/20261015-070659-2kqw.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261015-070659-2kqw",
>   "at": "2026-10-15T07:06:59.16592877Z",
>   "trigger": "rejection",
>   "job": "20261015-064153-wekz",
>   "intervention": null,
>   "class_of_work": "code",
>   "category": "rejected",
>   "what_happened": "The job asked for a drain command that works the queue within a budget. It was rejected. The verifier gave a fail verdict with one blocker-or-major finding. The finding was that drain never lands its work through `athena land --pr`. The work therefore never reaches the live page that drain rates (Drain::pass and Drain::rate in crates/athena-cli/src/drain.rs). The owner would still have to land each item by hand. \"Target met\" could come from a rating of a page the loop never changed. The event lists no unmet acceptance criteria and no failed checks, so the finding is the only evidence of what failed.",
>   "failure_class": "20261014-184842-whyd",
>   "new_class": false,
>   "earlier_cases": [],
>   "decisions_matched": [],
>   "cause": "The request described a command (\"works the queue within a budget\") without stating the end-to-end effect: queued items must be landed, and the rating must be of the page after those changes. Nothing required the build to prove that the rated state is the changed state. A build can therefore have a pass step, a rate step and a budget, all present and wired, and still never connect the work to the thing it measures. Neither the reading's criteria nor any check exercised the full loop of work, land, then rate. The gap was found only by the final verifier. This is an inference from one finding. The stored fields do not show which criteria were written. The class fit is partial, because the earlier final-verification class also applies.",
>   "prevention_kind": "design-change",
>   "prevention": "Require that a loop-style command's acceptance criteria state the observable outcome, not the presence of parts. For drain, a test must run one pass with a stubbed queue item. It must assert that `athena land --pr` is invoked for the item. It must also assert that the rating is taken only after landing, against the changed page. A rating of an unchanged page must not be able to report \"target met\". Admission should reject a reading for such a command if it lacks a criterion tying the measured artifact to the work's output.",
>   "proposed_request": "Fix the drain command so that it completes the loop without the owner's help. Each queue item it works must be landed through `athena land --pr`, and only then may the live page be rated. \"Target met\" must be reported only when the rating was taken after at least one item's change had landed on that page, or when the page was rated unchanged and the output says so. Add a test with a stubbed queue and a stubbed land step. It must show that (1) each worked item is landed, (2) the rating happens after landing, and (3) a pass that landed nothing cannot report \"target met\" from a stale rating. Update README.md 'What it does not decide' so it matches the new behavior. Keep the budget limit working: drain stops when the budget is spent, and the output says whether it stopped for budget or because the target was met.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.0236661,
>     "input_tokens": 2,
>     "output_tokens": 1051
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```
> 
> ### 8. path records/introspections/2026-10/20261015-160328-7077.json
> ```
> {
>   "schema": "athena.introspection/1",
>   "id": "20261015-160328-7077",
>   "at": "2026-10-15T16:03:28.903754991Z",
>   "trigger": "rejection",
>   "job": "20261015-152305-6h8k",
>   "intervention": null,
>   "class_of_work": "code",
>   "category": "rejected",
>   "what_happened": "The job asked for two changes to the drain command: count only landings that change the outcome, and make drain's source check semantic. It was rejected. The verifier's verdict was fail, with one blocker-or-major finding. The finding was that the source check (RealSteps::check_page / study_source in crates/athena-cli/src/drain.rs) reads only claims.json. The page's intro, transition and glossary text (intro.json) is never compared with the source. Wrong prose could therefore pass and end the run as \"target met\". This also conflicts with the tradeoff recorded in docs/decisions.md D100. The event lists no unmet acceptance criteria and no failed checks, so the finding is the only evidence. The semantic check may work for claims. It covers only part of the page.",
>   "failure_class": "20261014-184842-whyd",
>   "new_class": false,
>   "earlier_cases": [
>     "20261015-070659-2kqw"
>   ],
>   "decisions_matched": [],
>   "cause": "The request said to make the source check \"semantic\" without saying what it must cover: every piece of page text that makes a claim about the source. Nothing in the reading or the build listed the page's content inputs (claims.json and intro.json) and required the check to be exercised against each. A check that works on one input looks complete, and the criteria could pass on that basis. No test fed wrong prose in the intro, transition or glossary and expected the run not to end as \"target met\". Coverage was judged only by the final verifier. This is an inference from one finding, since the stored fields do not show the criteria.",
>   "prevention_kind": "code-check",
>   "prevention": "Add a code check, and a matching test, that the source check covers every content file of the page. The check should enumerate the page's text-bearing inputs (claims.json, intro.json and any others) and fail if any of them is not compared with the source. For each input, a test should inject a known-wrong statement and assert that drain does not report \"target met\". The reading should state this as a behavior criterion: a wrong statement in any page text blocks the finish. This would prevent the whole class, in which a check is present and wired but its scope is narrower than the outcome.",
>   "proposed_request": "Make drain's source check cover all page text, not only claims. Today RealSteps::check_page / study_source in crates/athena-cli/src/drain.rs compares only claims.json with the source. Extend it so that the intro, transition and glossary text in intro.json is also compared with the source, and any other text-bearing page file is included. Outcome: a page with a wrong statement anywhere in its text can never end a drain run as \"target met\". Acceptance: (1) add tests that inject one wrong statement into each of claims.json, the intro, the transition and the glossary, and show the source check fails each time and drain does not report \"target met\"; (2) add a code check that fails if a page text file is not covered by the source check; (3) a page with correct text in all files still passes; (4) keep the existing rule that only outcome-changing landings count, with its tests passing; (5) update the D100 Tradeoff note in docs/decisions.md to match the new coverage; (6) the final report includes a table mapping each criterion to a named test and the gate command results.",
>   "evidence": {
>     "model": "claude-sonnet-5-5",
>     "cost_usd": 0.0302561,
>     "input_tokens": 2,
>     "output_tokens": 1186
>   },
>   "status": "recorded",
>   "failure": null,
>   "flags": [],
>   "dropped_ids": [],
>   "from_jev": false
> }
> ```

## Reading (accepted by the user)

Retry of the rejected token reconciliation work. The base has no fix: the gate compares the requested model's modelUsage with top-level usage tokens, so they disagree. Make the record and the gate use one definition, and prove it with end-to-end tests whose negative case builds a truly disagreeing event through the same parse-and-record path. The audience is the Athena owner, who needs verified jobs to land.

Assumptions:
- The base commit does not contain the earlier discarded reconciliation change, so the full change is redone. The proposal's 'change no other behavior' applies to the rest of the system.
- Chosen definition: invocation tokens are the requested model's modelUsage sums, as the earlier retry intended. The verifier's suggestion of a cost disagreement is an alternative for the negative case.
- The mutation check (disabling the guard and watching the test fail) is done by the producer and reported. It is not a new mechanism in the repository.
- No governance files change.

## Acceptance criteria

1. [behavior] Run cargo test. A positive end-to-end test passes: a result event whose top-level usage differs from modelUsage (main thread plus subagent on the requested model) goes through parsing, the invocation record and check_invocation with no Reconciliation violation.
2. [behavior] Run cargo test. A named negative end-to-end test, a_result_whose_totals_disagree_with_model_usage_is_flagged_end_to_end, builds the record from a result event that has modelUsage for the requested model and really disagrees with it. It uses the same parse-and-record path as the positive test and does not edit the record by hand. It asserts check_invocation returns a Reconciliation violation.
3. [content] Before calling check_invocation, the negative test asserts that the built record's totals differ from the modelUsage sum. Its event is not one with no modelUsage, so it does not reach the 'no usage for the requested model' branch.
4. [content] The invocation's measured tokens and the gate's reconciliation rule use one definition of what an invocation used (modelUsage). The code or the CHANGELOG states it. The legacy exception is either removed or limited to older records.
5. [behavior] Run cargo clippy --all-targets and cargo fmt --check. Both are clean, and the full cargo test passes.
6. [content] The final report maps each criterion to a named test or command output.

## Expected deliverables

- crates/athena-core/src/invocation.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-agent/src/stream.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Building a general mutation-testing check for named negative tests
- The drain command and the explainer site
- Changes to records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus passing cargo test and clean clippy.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require the negative test to build a truly disagreeing record through the real path.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The record and the gate share one definition instead of patching the gate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to reconciliation and its tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
