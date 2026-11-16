# Job 20261116-020756-zx0r: verify "Show producer how each criterion is verified; record its pre-verdict"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Nothing in the stored event shows the unmet criterion being checked before the final verdict. The li
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: criterion-unmet-at-final-verification (20261014-172333-6fy0)
> Cause: Nothing in the stored event shows the unmet criterion being checked before the final verdict. The likely gap is that criteria are verified only at the end, so a miss costs the whole job instead of triggering a repair pass. The goal of the j...
> Seen 43 time(s) in 43 jobs. The introspections that record it:
> Introspection 20261014-172320-k3k0 (job 20261014-170031-2r9b); prevention it proposed: Have the implementer check each acceptance criterion against concrete evidence before submitting, and give the job a bounded repair loop: when the verifier reports an unmet criterion or a blocker or major finding, the job returns to the imp...
> Introspection 20261014-174804-mgn1 (job 20261014-172342-rqgb); prevention it proposed: Add a pre-verdict step where the worker maps each acceptance criterion to concrete evidence and runs a repair pass for any criterion without it. For "update every mention" criteria, require a repo-wide search with zero remaining stale hits,...
> Introspection 20261014-181337-4ns7 (job 20261014-174937-hrzn); prevention it proposed: Add a pre-submission self-check step: before a job goes to the verifier, the worker lists each acceptance criterion with the evidence that it is met (for sweep criteria, the search command and its remaining-hit result). For a redo, the earl...
> Introspection 20261014-213402-g1b6 (job 20261014-192610-xgke); prevention it proposed: Before a job can be declared complete, require a self-check step. It runs every accepted criterion as a concrete check, such as a run showing the steward reads the named sources and writes items to the queue. It must record pass or fail per...
> Introspection 20261014-222026-jm1z (job 20261014-221551-09t4); prevention it proposed: At admission, turn each outcome into acceptance criteria that name a runnable check. For docs, that means link and build checks plus a defined newcomer-readability check. Require the job to run those checks before it ends its work phase and...
> Introspection 20261015-013952-zyzm (job 20261015-012203-y8dx); prevention it proposed: Add a check during the build, before final verification. It runs each acceptance criterion as a concrete test and reports unmet ones to the builder while there is still time to fix them. Record the unmet criteria and findings in the rejecti...
> Introspection 20261015-060222-3dap (job 20261015-054026-k2ej); prevention it proposed: Add a pre-submission gate in code. The producer cannot send its final report until (1) it has a per-criterion evidence entry for every acceptance criterion (file and line, or test name), and (2) the system itself has run cargo fmt --check, ...
> Introspection 20261015-092454-x3d0 (job 20261015-085652-5qem); prevention it proposed: Add a pre-report gate in code. The producer cannot submit a final report until (1) the workspace checks (cargo fmt --check, cargo clippy --all-targets, cargo test) have been run by the harness on the current tree and the results recorded, a...
> Introspection 20261015-124204-qxbg (job 20261015-104515-8wqq); prevention it proposed: Add a code gate on the producer's final report. It must carry a per-criterion evidence table, with the command output or file excerpt for each acceptance criterion. It must also carry recorded results of the declared gate commands (test, cl...
> Introspection 20261015-131120-cjyk (job 20261015-124217-6xhe); prevention it proposed: Add a pre-transfer check in the job runner. It should refuse to finish a build until the final report has one row for every acceptance criterion, each with non-empty evidence (a test name or HTML excerpt), plus a section with the exit status...
> Introspection 20261015-152122-10ms (job 20261015-145809-7565); prevention it proposed: Add a pre-handover criterion check to the build. Before a job can finish, the builder must produce a table mapping each acceptance criterion to its evidence. For a criterion that asks for a test, the evidence is a named test that exists and...
> Introspection 20261015-215411-2zxz (job 20261015-214235-be8x); prevention it proposed: Add a completion gate in code that blocks the producer's "done" report until it has (a) submitted a per-criterion evidence table, each row naming a file, line or command output, and (b) run the workspace gate (cargo test, cargo clippy, carg...
> Introspection 20261015-225720-e0r8 (job 20261015-223418-qb41); prevention it proposed: Add a code gate that rejects the producer's final report before verification unless: (a) it contains an evidence entry for every acceptance criterion id, and (b) the gate commands (cargo test, clippy, fmt, the site build) have recorded exit...
> Introspection 20261016-005745-2wcm (job 20261016-003525-5wwj); prevention it proposed: Add a gate that blocks a producer's final report until it includes a table mapping every acceptance criterion to evidence, and rejects the report as incomplete if any criterion has no row. For structural criteria such as "not a special case...
> Introspection 20261016-011709-3n00 (job 20261016-005953-582g); prevention it proposed: A completion gate in code that refuses a producer's final report as "done" until it contains a table mapping every acceptance criterion to evidence, and until the gate commands (cargo test, cargo clippy --all-targets, cargo fmt --check) hav...
> Introspection 20261016-170323-hzhy (job 20261016-163644-txka); prevention it proposed: Add a pre-handover code check that runs before the job is sent to the verifier. For every acceptance criterion that names a command (cargo test, clippy, fmt --check, or a CLI behavior), the check requires an evidence entry that records the ...
> Introspection 20261017-011645-513m (job 20261017-004000-824m); prevention it proposed: Add a pre-verification check for criteria of the form 'term X is no longer used for Y'. The check greps all job-owned files (model.json, site.rs, glossary and card text) for the retired term. It lists every remaining occurrence, and the job...
> Introspection 20261017-014659-8e83 (job 20261017-011658-5ap5); prevention it proposed: Add a pre-handover gate that runs before the job can reach final verification. For each acceptance criterion, the reading names the evidence file or path that proves it. For each expected deliverable file, the gate checks that the diff touc...
> Introspection 20261017-114101-rsa3 (job 20261017-111119-m988); prevention it proposed: Add a pre-handover check in the run stage. For every criterion that names commands (such as the cargo test, clippy and fmt gate), Athena itself runs the commands and records the command lines, exit codes and output tail as evidence, or it r...
> Introspection 20261017-155222-5b78 (job 20261017-152733-efyd); prevention it proposed: Add a pre-handover gate that runs before the verifier. It would require (1) each criterion to map to a named test, or to a command that ran with its output, so a criterion asking for tests showing specific wiring fails if no such test exist...
> Introspection 20261023-214309-m5t6 (job 20261023-183334-aej6); prevention it proposed: Add a pre-handover check that blocks the producer from finishing until every acceptance criterion has a recorded evidence entry. For criteria that name a test, the entry must point to a test that exists in the diff and passed in a recorded ...
> Introspection 20261024-015841-5c6v (job 20261023-234313-h49d); prevention it proposed: Add a pre-handover check to the producer stage. When a criterion says that some wording or rule must no longer appear, the reading must name the phrase, or a pattern for it. The check then searches the listed docs (decisions.md, the design ...
> Introspection 20261024-045439-zj2c (job 20261024-033933-ta60); prevention it proposed: Add a pre-handover check in the pipeline. Athena itself runs the gate commands (`cargo test`, `cargo clippy --all-targets`, `cargo fmt --check`) in the producer's working copy. It stores their command lines, exit codes and output tails in t...
> Introspection 20261024-074821-hxps (job 20261024-033935-c4w4); prevention it proposed: Add a pre-handover code check: before a job moves from producer to verifier, require an evidence record for every criterion. For command-based criteria such as the clippy, fmt and test gate, it must hold the exact command, exit status and c...
> Introspection 20261024-170056-k1na (job 20261024-161559-rxjg); prevention it proposed: Add a pre-handover code check that runs before the job moves to verification. It would require the producer's change description to exist and to contain, for each acceptance criterion, a named test or a command with its recorded output. For...
> Introspection 20261025-022100-waxw (job 20261025-012657-hac3); prevention it proposed: Add a pre-handover check that runs after the producer finishes and before the verifier. For each acceptance criterion it requires a named test or command. It runs them and refuses handover if one fails or is missing. It also flags tests who...
> Introspection 20261025-031419-czhp (job 20261025-022144-emm8); prevention it proposed: Add a pre-handover check, run by code, that the producer must pass before it can hand over. For a change to landing or history code, the producer lists every writer and reader of the changed path (land_pr, retry_landing, reconcile, check_hi...
> Introspection 20261026-155302-zdnq (job 20261026-150607-85mv); prevention it proposed: Add a pre-handover check for any job that moves or changes a set of readers or writers. It lists every reader from a single registry, or from a grep of the loader call sites. It then requires each one to have a named test in the deliverable...
> Introspection 20261026-180059-a51n (job 20261026-170042-cgq9); prevention it proposed: Add a pre-handover check for reader-move jobs. It would enumerate the readers by grep and require, for each, a named test that calls the reader's real entry point. It would also run a mutation step: revert each reader to the in-tree path an...
> Introspection 20261027-011257-jxh3 (job 20261027-004517-gk9e); prevention it proposed: Before the verifier is dispatched, a code check confirms that the producer's report exists, is non-empty, and is included in the verifier's input. If any acceptance criterion refers to the report's content, the job is held at handover when ...
> Introspection 20261027-085315-n35f (job 20261027-082521-0sfz); prevention it proposed: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The pr...
> Introspection 20261028-232906-k1qr (job 20261027-160633-t5sh); prevention it proposed: Classify the usage-limit error once, at the single point where model-call errors are turned into results, as a typed "hold" outcome. Every caller (reading, planning, critic, producer, verifier) must receive that type, and the drain's failur...
> Introspection 20261029-023321-tn6t (job 20261026-232411-2zv3); prevention it proposed: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run wit...
> Introspection 20261030-004950-fqve (job 20261029-142853-x4nr); prevention it proposed: Add a test for any recorded-state feature that runs through the real implementation (RealSteps) instead of a fake. It would start a producer stage, read the in-flight table entry, and assert that the recorded path equals the directory the p...
> Introspection 20261030-013059-bf4h (job 20261030-005008-tf2s); prevention it proposed: Have the producer and checks code emit typed stage events (an enum with Producer, Checks and so on) at each transition, including the send-back transition, and have the in-flight table record only those. Do not parse display strings. Add a ...
> Introspection 20261030-065923-cp41 (job 20261029-233021-e5sg); prevention it proposed: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the fi...
> Introspection 20261031-111717-s4qq (job 20261031-110024-4qd3); prevention it proposed: Before the producer's work goes to the verifier, run the full check set (`cargo test`, clippy, fmt) automatically and send any failure back to the producer. At the same time, record a baseline of the same checks on the unchanged base so a f...
> Introspection 20261101-113931-j9at (job 20261101-102128-b1st); prevention it proposed: Add a pre-verdict step in code. Before the verifier runs, Athena runs the repo gate (fmt, clippy, test) itself and records the exact commands and exit codes. It also checks that every behavior criterion has a demonstration that names a comm...
> Introspection 20261101-163615-67fa (job 20261101-135902-9bxs); prevention it proposed: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If...
> Introspection 20261103-014831-3d1n (job 20261103-011406-wm2v); prevention it proposed: Add a pre-verdict gate in the job pipeline. After the producer stops, Athena itself runs fmt --check and clippy --all-targets -D warnings (and test) on the job's snapshot. If any fail, it sends the exact output back to the producer for one ...
> Introspection 20261103-101121-71yx (job 20261103-092929-s7ts); prevention it proposed: Add a producer default: before finishing, for every criterion that names several fields or inputs, list each one and point to the test line that varies it. For a "briefs differ" test, every fixture field named in the criterion must take a d...
> Introspection 20261103-202623-649w (job 20261016-005805-p806); prevention it proposed: Before the producer may finish, Athena runs the cheap mechanical gate on the working copy: the decision-link consistency test (every "amends/supersedes Dxx" link in docs/decisions.md is in docs/decision-links.toml), fmt, clippy, and a scan ...
> Introspection 20261103-222715-7xrj (job 20261103-193428-df2m); prevention it proposed: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-cha...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the producer's brief shows criteria but not how the verifier will check each one. A producer-reported "met" with no matching evidence (sweep criteria, line width, named tests) passes the send-back check, and the miss surfaces only at final verification. The fix gives the producer a per-criterion check hint in its brief and stores the per-criterion check on every attempt. It is a brief change, not a refusal rule. It also addresses jobs 20261014-170031-2r9b, 20261014-172342-rqgb and 20261014-174937-hrzn.

Assumptions:
- Goal and audience: the goal is fewer jobs failing final verification on a criterion the producer reported met. The audience is the producer, whose brief gains the information, and the verifier and introspection reader, who see the stored check.
- Code survey: crates/athena-core/src/run/producer.rs (send-back loop at about lines 560-715; it records the attempt's invocation detail only on failure), crates/athena-core/src/run/send_back.rs (unmet_criteria matches criteria_check entries by key and trusts a status of met; is_athena_check_criterion), crates/athena-context/src/briefs.rs (the producer brief and the AcceptedReading criteria), crates/athena-context/src/outputs.rs (CriterionCheckOutput) and crates/athena-context/prompts/producer.md. The entry point is the producer stage's run loop. No new record or data file is added, so check_record_file, AnyRecord and the round-trip test are untouched. The property is measured directly: the rendered producer brief text and the recorded invocation detail.
- Earlier jobs: 20261014-170031-2r9b, 20261014-172342-rqgb and 20261014-174937-hrzn each changed the role prompts, the reader budget, verify.rs and docs about request-goal survival, and each was rejected on one remaining unmet criterion, a stale 1,800 mention or an over-wide line. Those causes are not repeated here. Their changed files (CHANGELOG.md, the three prompts, context lib.rs, verify.rs, decisions.md, athena-v0.md, decision-sets.md, defaults-loop.md, socratic-review.md, records.md) are dropped, except CHANGELOG.md and docs/decisions.md, which this job appends to. Each earlier finding is covered by the new brief hint, which tells the producer to sweep the repo for 'every mention' criteria and to check line width for rewrapped prose.
- No refusal rule is added, so no refused-reading share applies. The hint is information in the brief, and the send-back loop is unchanged.
- Existing code: crates/athena-context/src/briefs.rs and crates/athena-core/src/run/producer.rs are extended, and the criteria_check report field is reused as is.
- Environment: none; local cargo only.

## Acceptance criteria

1. [content] The change description names the root cause: the producer is shown criteria but not how each will be verified, so a reported 'met' without evidence passes until final verification. The fix is in the producer brief builder in crates/athena-context/src/briefs.rs, recorded in a new appended decision in docs/decisions.md with a Provenance bullet and in CHANGELOG.md.
2. [content] The producer brief rendered by crates/athena-context/src/briefs.rs lists, for each accepted criterion, how the verifier checks it: a named file or diff line for a content criterion, and a command that ran for a behavior one. A criterion that says every mention, or no longer appears, gets a repo-wide search with zero remaining hits. A rewrapped-prose criterion gets a line-width check. No new check refuses or holds work.
3. [behavior] A new unit test in crates/athena-context/src/briefs.rs fails on the base tree and passes with the change. It renders the brief for a reading with a sweep criterion and a behavior criterion, and asserts that each hint appears. Run `cargo test -p athena-context`.
4. [behavior] A new test in crates/athena-core/src/run/ drives the producer stage with the fake agent and shows that the attempt's stored invocation detail carries the per-criterion check, status and evidence on a passing attempt, not only on failure. It fails on the base tree. Run `cargo test -p athena-core`.
5. [behavior] The recorded fmt, clippy and test checks on the final tree pass: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`.
6. [content] The changed prompt files stay within their token budgets, with the budgets test in crates/athena-cli/tests/budgets.rs passing, and no budget is raised.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/prompts/producer.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/budgets.rs (modify)

## Out of scope

- Adding a repair loop after the verifier or changing the job lifecycle
- Adding any refusal or hold rule
- Editing reader, verifier or contract prompts, or any budget
- Reverting or redoing the docs and budget edits of jobs 20261014-170031-2r9b, 20261014-172342-rqgb and 20261014-174937-hrzn
- Edits to records/ or owner files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests named in criteria; fmt, clippy, test recorded.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix placed in the brief builder, which feeds every job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered brief text and stored detail through tests that fail on the base tree.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff. docs/decisions.md ends with an appended `## D191 · ...` entry after D190 (line 5635). Its Case names the root cause: the producer brief shows criteria and not how the verifier checks each, so a 'met' with no matching evidence passes the send-back (send_back.rs:27-30) and the miss surfaces at final verification. The Case cites `Candidate.producer_report`, `pre_verdict_block` and commit 81b1374e, and says the stored check was not the gap. The Decision names `producer_brief_with_evidence` in crates/athena-context/src/briefs.rs. The entry has Alternatives, Tradeoff, Reversal cost and a Provenance bullet, and avoids the phrases the decision-link scan reads (decision_links.rs:255-257). CHANGELOG.md [Unreleased] 'Changed' has an entry with the same cause and fix. The mirrors unit tests (mirrors.rs:72, 354-396) judge the heading and Provenance bullet when the verifier runs `cargo test -p athena-core mirrors`. Run: `git diff -- docs/decisions.md CHANGELOG.md`. Observes: The diff holds the D191 entry and the CHANGELOG entry, both naming the brief gap and crates/athena-context/src/briefs.rs, and neither claims the stored check was missing.
- Criterion 2: Entry point: `producer_brief`, through `producer_brief_with_evidence`. The test renders the brief for a reading with six criteria (sweep, behavior with a command, rewrapped prose, plain content, fmt and clippy, and a 'wrapper' near-miss) and reads the new section. It asserts: a content criterion has 'named file or diff line'; a behavior criterion has 'a command that ran'; the sweep criterion adds 'repo-wide search' and 'zero remaining hits'; the rewrapped-prose criterion adds 'line-width check'; the fmt and clippy criterion gets the `left to Athena's checks` hint and not 'a command that ran'; the 'wrapper' criterion gets no width hint; the section sits before `## criteria_check: copy the line`. Then read the diff of producer.rs: `unmet_criteria`, the `problems` list (producer.rs:557-613) and the send-back text gain no entry, and no `Err`, `Flow::Stop` or hold path is added. The reading checks and refusal corpus are not touched; run `cargo test -p athena-core --test refusal_corpus` to see it pass unchanged (no case maps to these forms, none is added). Run: `cargo test -p athena-context producer_brief_says_how_the_verifier_checks_each_criterion`. Observes: One line per criterion with the stated hints, and a diff with no new refusal, send-back or hold condition.
- Criterion 3: New unit test `producer_brief_says_how_the_verifier_checks_each_criterion` in the briefs.rs test module, next to `producer_briefs_list_every_criterion_to_copy_and_ask_for_one_entry_each` (line 1640). It builds `AcceptedReading::new(&reading(), &defaults(), |_| false)`, sets the public `criteria` field (briefs.rs:80) to the six criteria, and calls `producer_brief(&ProducerInputs {..})` as the neighbouring tests do. It takes the text from the heading `## How the verifier checks each criterion` to the next `\n## `, and finds each `Criterion N: ` line. It asserts the hints named in criterion 2, that the section does not repeat any criterion's text, that the heading is before `## criteria_check: copy the line` and after `## Make your evidence checkable`, and that the brief holds none of '0.97', '0.8', 'p10' or 'cost'. A second test, `verifier_check_hint_matches_whole_phrases_only`, calls `verifier_check_hint` directly: 'Every Mention of X is updated' (mixed case) gets the sweep hint; 'line-width' gets the width hint; 'the cargo wrapper is installed' gets neither; 'nowhere else' gets the sweep hint. Run: `cargo test -p athena-context producer_brief_says_how_the_verifier_checks_each_criterion`. Observes: Both tests pass with the change.
  Fails without the behavior: On the base tree the brief has no such section, so `split_once` on the heading panics and the test fails. Removing only the sweep branch fails the 'repo-wide search' assertion. Removing only the behavior branch fails 'a command that ran'. Removing the `is_athena_check_criterion` branch makes the fmt criterion get 'a command that ran' and fails the assertion. Replacing the whole-phrase match with a substring match 'wrap' fails the 'wrapper' case.
  Test seam: Production entry point `producer_brief` (briefs.rs:676) into `producer_brief_with_evidence`, with the new `verifier_checks_section`. Fixtures are the `reading()` and `defaults()` helpers of the briefs.rs test module (defaults at line 1605), which exist. The criteria are set through the public `AcceptedReading.criteria` field, so there is no double. The hint logic runs in production code; plan steps 1-3 add it.
- Criterion 4: New test in crates/athena-core/src/run/tests.rs, next to `a_failing_check_is_sent_back_before_the_verifier_and_the_fix_reaches_it` (line 5148). It runs a whole job through `athena.go` with `run_checked_job` (line 5128) on `repo()`, with the check `PASSES` (line 103) and the script `editing(counts, prompts, Vec::new())` (line 3567). The base script's producer (tests.rs:250-262) reports the one criterion met with evidence 'read it' and edits README.md, so the first attempt passes with no send-back. The test loads `Records::load(&athena.git, &athena.head().unwrap())`, as at lines 5191-5198, and takes the producer invocations of the job. It asserts there is exactly one, and that `counts.all()` gives producer 1. It asserts its `detail` starts with 'self-check passed' and contains 'redacted report: '. It parses the JSON after that text into `athena_data::records::ProducerReport` (as run_pin_tests.rs:1799-1800 does) and asserts one `criteria_check` entry for the job's criterion (`CRITERION`, tests.rs:88), with status 'met' and evidence 'read it'. Finally it asserts that this `criteria_check` equals the job's `candidate.producer_report.criteria_check`, the per-attempt copy of the stored report. Run: `cargo test -p athena-core a_passing_attempt_stores_the_per_criterion_check_in_its_invocation_detail`. Observes: The stored detail of a passing producer attempt carries the criterion, its status and its evidence, and matches the candidate's report.
  Fails without the behavior: On the base tree the passing attempt's `detail` is None, since it is set only on the failing path (producer.rs:621). `unwrap_or_default()` gives an empty string and the 'self-check passed' assertion fails.
  Test seam: Production entry point `Athena::go` into `producer_loop` (producer.rs:298). The fake is the `FakeAgentDriver` script given to `athena_with` (tests.rs:192-221); the script, `run_checked_job`, `editing`, `PASSES` and `Records::load` all exist. The fake only supplies the producer's report and edit; the code that writes the detail is the production loop, so no test double holds the logic under test. Plan step 5 adds the production change.
- Criterion 5: Athena's recorded fmt, clippy and test checks are the evidence. The verifier may run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` itself. The new tests are in the workspace run. Moving `is_athena_check_criterion` leaves the send_back.rs tests (`only_unscoped_cargo_checks_count_as_athena_check_criteria` and the `left_to_athena` tests) in the run, so a mistake in the move shows there. Run: `cargo test --locked`. Observes: All three commands exit 0.
  Fails without the behavior: Not applicable: the recorded checks decide this criterion.
  Test seam: No new seam: the three checks are Athena's configured checks, as listed in the brief.
- Criterion 6: Run the budgets test (`every_role_set_is_within_its_budget`, `system_facts_budgets_match_computed`). Then `git diff --stat -- crates/athena-context/prompts crates/athena-cli/tests/budgets.rs docs/system-facts.md` is empty. No prompt file, no budget and no budget line changes, so the producer set stays at 1043 of 1200 and no budget is raised. producer.md and budgets.rs are expected deliverables the plan leaves unchanged. The producer lists both in `unchanged_planned_files` with the reason: the hint is in the brief, which sits outside the budgeted sets. Run: `cargo test -p athena-cli --test budgets`. Observes: The budgets tests pass and the diff stat for those files is empty.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5afacdb26a04 → candidate 9286a8a4fd7a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   1 +
 crates/athena-context/src/briefs.rs     | 274 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/producer.rs  |  11 ++
 crates/athena-core/src/run/send_back.rs |  30 +---
 crates/athena-core/src/run/tests.rs     |  51 ++++++
 docs/decisions.md                       |  45 ++++++
 6 files changed, 384 insertions(+), 28 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261116-020756-zx0r/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261116-020756-zx0r/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43975118145 for 9286a8a4fd7a3c09de9883eb9deb39abfcdc3d5d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43975118145 for 9286a8a4fd7a3c09de9883eb9deb39abfcdc3d5d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43975118145 for 9286a8a4fd7a3c09de9883eb9deb39abfcdc3d5d)
- ci-result: `not a check` passed (Check results for 9286a8a4fd7a3c09de9883eb9deb39abfcdc3d5d come from the CI run https://github.com/doodla/athena/actions/runs/43975118145 (784s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: The recorded fmt, clippy and test checks on the final tree pass: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context producer_brief_says_how_the_verifier_checks_each_criterion`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling thiserror-impl v2.0.21
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling digest v0.10.7
   Compiling winnow v0.7.15
   Compiling cpufeatures v0.2.17
   Compiling jiff-core v0.1.1
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling jiff v0.2.38
   Compiling toml v0.9.12+spec-1.1.0
   Compiling schemars_derive v1.2.2
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0425]: cannot find function `verifier_check_hint` in this scope
    --> crates/athena-context/src/briefs.rs:1777:21
     |
1777 |             assert!(verifier_check_hint(text, c).contains(sweep), "{text}");
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `verifier_check_hint` in this scope
    --> crates/athena-context/src/briefs.rs:1786:21
     |
1786 |             assert!(verifier_check_hint(text, c).contains(width), "{text}");
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `verifier_check_hint` in this scope
    --> crates/athena-context/src/briefs.rs:1793:24
     |
1793 |             let hint = verifier_check_hint(text, c);
     |                        ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `is_athena_check_criterion` in this scope
    --> crates/athena-context/src/briefs.rs:1800:17
     |
1800 |         assert!(is_athena_check_criterion("cargo fmt --check passes"));
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `is_athena_check_criterion` in this scope
    --> crates/athena-context/src/briefs.rs:1801:17
     |
1801 |         assert!(is_athena_check_criterion("`cargo test` passes"));
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `is_athena_check_criterion` in this scope
    --> crates/athena-context/src/briefs.rs:1802:18
     |
1802 |         assert!(!is_athena_check_criterion(
     |                  ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `is_athena_check_criterion` in this scope
    --> crates/athena-context/src/briefs.rs:1805:18
     |
1805 |         assert!(!is_athena_check_criterion("The README names the tool"));
     |                  ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 7 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_passing_attempt_stores_the_per_criterion_check_in_its_invocation_detail`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.00s

     Running tests/context_compression.rs (<job>/target/verify/debug/deps/context_compression-6112e9d39a3b55ee)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running tests/divergence.rs (<job>/target/verify/debug/deps/divergence-1ae4db8e62dcf162)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/lab_runs.rs (<job>/target/verify/debug/deps/lab_runs-6a6e3469b51eff19)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/learnings_recorded.rs (<job>/target/verify/debug/deps/learnings_recorded-ac4f2643c7b6c897)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

     Running tests/mechanism_explanation.rs (<job>/target/verify/debug/deps/mechanism_explanation-4037148411066e09)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.00s

     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling clap_derive v4.6.7
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling clap v4.6.7
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `verifier_check_hint` in this scope
    --> crates/athena-context/src/briefs.rs:1777:21
     |
1777 |             assert!(verifier_check_hint(text, c).contains(sweep), "{text}");
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `verifier_check_hint` in this scope
    --> crates/athena-context/src/briefs.rs:1786:21
     |
1786 |             assert!(verifier_check_hint(text, c).contains(width), "{text}");
     |                     ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `verifier_check_hint` in this scope
    --> crates/athena-context/src/briefs.rs:1793:24
     |
1793 |             let hint = verifier_check_hint(text, c);
     |                        ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `is_athena_check_criterion` in this scope
    --> crates/athena-context/src/briefs.rs:1800:17
     |
1800 |         assert!(is_athena_check_criterion("cargo fmt --check passes"));
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `is_athena_check_criterion` in this scope
    --> crates/athena-context/src/briefs.rs:1801:17
     |
1801 |         assert!(is_athena_check_criterion("`cargo test` passes"));
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `is_athena_check_criterion` in this scope
    --> crates/athena-context/src/briefs.rs:1802:18
     |
1802 |         assert!(!is_athena_check_criterion(
     |                  ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `is_athena_check_criterion` in this scope
    --> crates/athena-context/src/briefs.rs:1805:18
     |
1805 |         assert!(!is_athena_check_criterion("The README names the tool"));
     |                  ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
