# Job 20261114-045647-w91y: verify "Tell the producer its deadline and pacing in its brief"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The producer runs against a fixed wall-clock deadline. Nothing in the job tracks progress, checks in
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: producer-deadline-exceeded-on-small-job (20261017-032151-9ymk)
> Cause: The producer runs against a fixed wall-clock deadline. Nothing in the job tracks progress, checks in partway, or tells a stalled producer from a slow one. A short prompt-edit job therefore burns the full 1800 seconds before anyone learns it...
> Seen 11 time(s) in 10 jobs. The introspections that record it:
> Introspection 20261017-032141-7xvh (job 20261010-154006-vc5k); prevention it proposed: Add a producer progress watchdog. It would require a visible step, such as a file written or a commit made, within a short window (about 10 minutes). If none appears, it would stop the producer early and record its last activity and partial...
> Introspection 20261017-044251-1804 (job 20261016-052605-qw38); prevention it proposed: Change the producer stage so a deadline does not discard its work. (1) The producer commits checkpoints and writes a progress note as it goes. (2) On deadline, the job keeps the partial result and is marked "timed out, resumable", which is ...
> Introspection 20261018-073427-vw9w (job 20261018-070117-qmxv); prevention it proposed: Make the producer stage resumable and size-aware. (1) Before the producer runs, a planning step must split any job with more than one separable deliverable into sub-jobs, each of which fits one deadline. (2) The producer must commit partial...
> Introspection 20261023-234113-5z2s (job 20261019-013309-nvbb); prevention it proposed: Give the producer stage a progress signal and an early-stop rule. It would write a checkpoint (files touched, criteria addressed) at intervals. If no checkpoint or diff appears within a fraction of the deadline, the stage aborts early and r...
> Introspection 20261023-234230-55v8 (job 20261023-234149-jcd2); prevention it proposed: Put the test budget in the harness, not in the prompt. Record the repo's scoped test command per crate and the typical full-suite duration. Run producer test commands through a wrapper that allows the full suite once, or twice at most, and ...
> Introspection 20261024-002507-470t (job 20261023-234149-jcd2); prevention it proposed: Give the producer stage a progress signal and an early stop. It should write a checkpoint of its partial diff and a short status note at set intervals. If it makes no progress for a set period, it should stop early and report what it found,...
> Introspection 20261024-145615-7s8b (job 20261024-140624-ewbs); prevention it proposed: Make the producer's deadline recoverable. (1) The producer commits checkpoints to the job branch and writes a short progress note as it goes. (2) On deadline, Athena keeps the partial work and the last note, marks the job "timed out with pa...
> Introspection 20261025-055532-4grg (job 20261025-034731-2am0); prevention it proposed: Make the producer stage report progress and keep partial work. It should write a checkpoint (files changed, last command, last test run) at intervals. On deadline, it should record that checkpoint as a timeout result separate from a real fa...
> Introspection 20261025-150430-h29z (job 20261017-044405-4zcq); prevention it proposed: Make a producer timeout a resumable state and not a bare failure. On deadline, Athena would snapshot the producer's working tree and its last activity, record the job as timed out, and let a retry start from that snapshot with a note of wha...
> Introspection 20261031-043209-pa0f (job 20261031-043018-kjy2); prevention it proposed: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena ove...
> Introspection 20261102-072959-zhys (job 20261031-230917-g47x); prevention it proposed: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the producer runs against a fixed wall-clock deadline (1800s) that its brief never states, so it can't pace itself or stop to report. Add a "Time budget" section to the producer brief with the real deadline, an instruction to finish the smallest complete change first, to run the full test suite at most once, and to leave the tree in a reportable state. Jobs 20261010-154006-vc5k, 20261016-052605-qw38 and 20261018-070117-qmxv failed only on this deadline and had no candidate; their requests are not redone here.

Assumptions:
- Goal and audience: the producer agent is the audience of the brief change, and Athena's operator judges whether deadline failures drop. The failure class is 'producer deadline exceeded on a small job'.
- Code survey: crates/athena-core/src/run/producer.rs (producer_snapshot and the producer loop, Ended::Deadline at line 395), crates/athena-core/src/run/mod.rs (line 999 formats 'the producer did not complete: deadline of Ns reached'; line 1010 handles Ended::Deadline; lines 1195-1213 build the resume RetrySeed), crates/athena-agent/src/procs.rs (the deadline reason text), and crates/athena-context/src/briefs.rs (producer_brief_with_evidence, retry_section, the resume note). A timed-out job already keeps its work and resumes with a progress note. No code puts the deadline in the brief (grep for deadline in athena-context finds only resume text). No record or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by reading the rendered brief text in a test.
- Existing code: the change extends crates/athena-context/src/briefs.rs (ProducerInputs, producer_brief_with_evidence) and the producer-input construction in crates/athena-core/src/run/; the existing resume and progress-note mechanism is reused unchanged.
- Field meaning: the new ProducerInputs field carries the producer deadline in seconds, read from the same config value that crates/athena-agent/src/procs.rs uses to enforce the deadline. The implementer should confirm the config key name in athena.toml.
- Governance: athena.toml and defaults.toml are not changed. The existing deadline value is only read. Consumers: none found for a new config key because none is added.
- Prompt files under crates/athena-context/prompts/ are not edited, because the reader, producer and verifier prompt sets are near their token budgets. The text lives in the generated brief, which has no token budget in the list above.
- The introspections' other proposals are alternatives not taken here: a progress watchdog and resumable checkpoints (resume already exists), and a refusal of identical-scope retries (a refusal rule, allowed only if giving the information cannot prevent the failure). They are left for follow-up.

## Acceptance criteria

1. [content] The producer brief in crates/athena-context/src/briefs.rs (producer_brief_with_evidence) has a new 'Time budget' section giving the producer's deadline in seconds, taken from the configured producer deadline and not a constant, and telling it to finish the smallest complete change first, run the full test suite at most once, and keep the tree reportable. Verify by reading the diff.
2. [behavior] A new unit test in crates/athena-context/src/briefs.rs fails on the base and passes with the change. It builds producer briefs for two different deadlines and asserts each brief states its own deadline value, and that the section appears for a fresh job, a retry and a resume. Run: cargo test -p athena-context.
3. [behavior] A test in crates/athena-core/src/run/tests.rs shows the deadline configured for the producer invocation is the number written into the stored brief-producer.md (the same value, not a copy). Run: cargo test -p athena-core.
4. [content] Root cause named in CHANGELOG.md and in the doc comment of the new section: the stage lacked its own deadline and pacing information. Jobs 20261010-154006-vc5k, 20261016-052605-qw38 and 20261018-070117-qmxv are addressed by this: they recorded no candidate, so nothing is kept from them. Their original requests (verifier snapshot note, auditor and warden roles, baseline checks) are not part of this change.
5. [content] No refusal rule is added: nothing in the diff refuses, holds or rejects a reading or job. Expected 0% of newly refused readings.
6. [behavior] cargo fmt --check, cargo clippy and cargo test pass, as recorded by Athena's checks on the final tree.
7. [behavior] Outcome check for the producer-deadline-exceeded-on-small-job class: running cargo test shows the brief of a small job (short deadline and one deliverable) carries the budget text, so the producer is told its limit before it starts.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding a progress watchdog, early-stop rule or checkpoint commits to the producer
- Making retry or auto-requeue refuse identical-scope attempts
- Redoing the requests of jobs 20261010-154006-vc5k (verifier snapshot note: crates/athena-context/prompts/verifier.md), 20261016-052605-qw38 (auditor and warden roles: roles.rs, warden.rs, auditor.rs, lib.rs, main.rs, drain.rs, store.rs, records.rs, design doc, decisions.md) and 20261018-070117-qmxv (baseline checks: checks.rs, routing.rs, display.rs, outputs.rs, verifier.md, lifecycle.rs, docs/design.md); their candidates changed no files, so no work is kept or dropped
- Changing the deadline value, athena.toml, defaults.toml or CI workflows
- Editing prompt files under crates/athena-context/prompts/

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: brief text plus the plumbing for the deadline value.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the stage's inputs, not a per-job workaround.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert the rendered brief text for different deadlines.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff. Check (a) `time_budget_section(deadline_s: u64)` formats the number from its argument, with no numeric deadline literal; (b) it is called in `producer_brief_with_evidence` after `pin_section` and before `failure_classes_section`, unconditionally; (c) `ProducerInputs.deadline_s` is filled in run/mod.rs from `producer::producer_deadline_s(&jc)`, the same accessor the invocation uses at producer.rs:344, which returns `jc.config.budget.producer_timeout_s`; (d) the text says the smallest change that meets every acceptance criterion comes first, the full suite runs at most once, and no half-edited file or broken build is left between steps. Run: `git diff -- crates/athena-context/src/briefs.rs crates/athena-core/src/run/producer.rs crates/athena-core/src/run/mod.rs`. Observes: The section is rendered from `inputs.deadline_s`. The value comes from the config accessor, not a constant, and the same accessor feeds the invocation. All three pacing instructions are in the text.
- Criterion 2: New unit test `time_budget_states_each_jobs_own_deadline` in crates/athena-context/src/briefs.rs. It builds `producer_inputs(None)` and sets `deadline_s` to 300, then to 5400. It asserts the first brief contains 'is stopped after 300 seconds of wall-clock time' and not '5400 seconds', and the second the reverse. It asserts both contain 'smallest change that meets every acceptance criterion', 'at most once', 'no half-edited file' and '`date`'. It then builds five brief shapes at 300 seconds: fresh; seeded retry (`retry_from = Some(seed())`); resume (`RetrySeed { progress_note: Some(..), candidate: Some(..), ..seed() }`); stale conflict (`conflicts` non-empty); unseeded retry (`retry_findings = Some(RetryFindings { .. })`). For each it asserts `brief.matches("## Time budget").count() == 1` and the '300 seconds' text. Run: `cargo test -p athena-context time_budget`. Observes: Each brief carries its own number and the section appears exactly once in all five shapes. The two deadlines are not substrings of each other.
  Fails without the behavior: On the base the test does not compile (`ProducerInputs.deadline_s` does not exist). With the field but a constant in the text, the brief for 5400 lacks '5400 seconds'. With the section only on the fresh path, the retry, resume or conflict shape lacks '## Time budget' and the count assertion fails.
  Test seam: Production entry point: `producer_brief` / `producer_brief_with_evidence` (crates/athena-context/src/briefs.rs:675, 681). Inputs are plain `ProducerInputs` values from the existing helper `producer_inputs(pin)` (briefs.rs:1797) and the existing `seed()` helper (briefs.rs:1811), over the in-file fixture `reading()` (briefs.rs:1474). No fake or double: the code under test runs as production code. The seam exists (read at briefs.rs:1797-1824). Step 6 adds `deadline_s` to the helper and every literal; step 7 adds the test.
- Criterion 3: New test `the_producer_deadline_in_the_stored_brief_is_the_one_the_invocation_runs_under` in crates/athena-core/src/run/tests.rs. Steps: `repo()` (tests.rs:288); read its athena.toml, `replacen("producer_timeout_s = 86400", "producer_timeout_s = 4321", 1)` and assert the text changed (the line is athena.toml:51); commit with `commit_file` (tests.rs:358). Wrap `base_script()` in a `Script` closure that, for `Role::Producer`, records `spec.timeout.as_secs()` and `spec.user_prompt` and then calls the base script (the pattern at tests.rs:858-903). Run `athena_with(&repo, &state, PASSES, crate::test_support::plenty_of_space, script)`, then `ask("describe the demo", &Quiet)` and `go(..., AcceptedBy::User, &Quiet)`, and assert the job landed. Load `Records::load(&athena.git, &athena.head().unwrap())` and take `jobs[id]`. Assert: recorded `spec.timeout` is 4321; the producer entry in `job.invocations` has `caps.timeout_s == 4321` (written from `spec.timeout` at crates/athena-core/src/invocation.rs:212); the job record's `caps.producer_timeout_s == 4321`; the `## Time budget` section (from that heading to the next '\n## ') of `job.briefs["brief-producer"]` equals the same section of the recorded prompt and contains '4321 seconds'; the stored brief does not contain '86400'; and `prompt.starts_with(&stored.replace(&format!("<state>/jobs/{id}/evidence"), &state.path().join("repo/jobs").join(id).join("evidence").display().to_string()))`. The reader forecast has p50 = 30 s (tests.rs:245), under the 4321 limit that `check_reading_size` (reading.rs:370) applies, so the reading is accepted. Run: `cargo test -p athena-core the_producer_deadline_in_the_stored_brief_is_the_one_the_invocation_runs_under`. Observes: The deadline the invocation ran under, the number recorded in the invocation and job records, and the number in the stored brief are all 4321. 4321 is neither the repository value (86400) nor a constant in the code, so a copy or a constant would show. The prompt is the stored brief with only the placeholder replaced.
  Fails without the behavior: Without the change the stored brief has no 'Time budget' section, so the '4321 seconds' assertion fails. If the brief took its number from another source (a constant, or the default 86400), the equality with `spec.timeout` and the '4321' assertion fail.
  Test seam: Production entry points: `Athena::ask` and `Athena::go` (crates/athena-core/src/run/mod.rs; `dispatch` builds the brief at mod.rs:1177 and the producer loop builds the spec at producer.rs:331-346), through the real `producer_deadline_s` accessor and `producer_brief_with_evidence`. Fake: `FakeAgentDriver` with a `Script` closure injected through `athena_with` (tests.rs:191-220). The closure only records `spec.timeout` and `spec.user_prompt` and then delegates to `base_script()` (tests.rs:224); the accessor and brief builder run as production code. Fixture: the test repository with its committed athena.toml rewritten through `commit_file`; config is read from the head commit by `JobConfig::at` (mod.rs:1030). Everything exists today (tests.rs:179-366, 858-1065); step 8 adds only the test.
- Criterion 4: Read the CHANGELOG entry (new bullet at the top of the Added list under [Unreleased]; the Added heading is at CHANGELOG.md:20) and the doc comment on `time_budget_section`. Both must name the root cause: the producer stage was never told its own per-run limit, so it could not pace against it and its existing pacing rules had nothing to hang on. The entry must list jobs 20261010-154006-vc5k, 20261016-052605-qw38 and 20261018-070117-qmxv as addressed by this. It must say they recorded no candidate, so nothing is kept from them, and their original requests (verifier snapshot note, auditor and warden roles, baseline checks) are not redone. It must note that the 1800 s of the failure class is the earlier value, that the live limit is 86400 s (athena.toml:51, CHANGELOG.md:12), that the number matters most if the limit is lowered again, and that the producer has no `date` command under the default grant. It must name the tests and say no refusal rule is added (0% newly refused) and that whether producers follow the pacing text is not measured. Run: `git diff -- CHANGELOG.md crates/athena-context/src/briefs.rs`. Observes: Both texts name the root cause. The entry covers the three jobs, the 86400 value, the date limitation, the tests and the 0% figure.
- Criterion 5: Read the whole diff for any new error, bail, hold, reject or refuse path. The production changes are one text section, one input field and one shared accessor. No admission, reading check, retry, landing or gate code is touched; `check_reading_size` (reading.rs:370) is untouched. Run: `git diff`. Observes: No refusal or hold is added; the expected share of past accepted readings newly refused is 0%. No reading rule, stem list or parser changes, so no refusal_corpus case is added or mapped and no refusal_corpus demonstration is needed.
- Criterion 6: Run the three checks Athena runs (athena.toml:93-109) on the final tree, each as its own whole command: `cargo fmt --all --check`, then `cargo clippy --all-targets --locked -- -D warnings`, then `cargo test --locked`. Preconditions: all edits are in the tree and Cargo.lock is unchanged (no dependency is added), so `--locked` holds. The test run also covers `producer_briefs_carry_no_predictions` (briefs.rs:1536; forbids '0.97', '0.8', 'p10' and 'cost'), the retry and resume brief tests (briefs.rs:1826-1941), `system_facts_budgets_match_computed` (crates/athena-cli/tests/budgets.rs) and the boundary scan in xtask/tests/policy.rs. This is not a protection test, so the root-user rule does not apply. Run: `cargo test --locked`. Observes: All three commands exit 0 on the final tree: fmt prints nothing, clippy reports no warnings, and every test passes, including the three new ones.
  Fails without the behavior: A `ProducerInputs` literal left without `deadline_s` (10 in the briefs.rs tests at 1538, 1568, 1604, 1636, 1745, 1798, 2266, 2289, 2310 and 2337, one in reading.rs:3581, one in run/mod.rs:1178) fails to compile under `cargo clippy --all-targets` and `cargo test`. Brief text containing 'cost' or '0.8' fails `producer_briefs_carry_no_predictions`. Unformatted code fails `cargo fmt --all --check`.
  Test seam: Production entry points: the three check commands Athena itself runs, as configured at athena.toml:93-109 (fmt, clippy, test). The fixture is the real final working tree of the job. Nothing is injected and there is no double. The seam exists today: the same commands run on the base tree.
- Criterion 7: New test `a_small_job_brief_carries_its_short_deadline` in crates/athena-context/src/briefs.rs. It builds `producer_inputs(None)` with `deadline_s = 120`. The fixture `reading()` has exactly one deliverable (CHANGELOG.md, create; briefs.rs:1514-1518). It asserts the brief contains '## Time budget' and 'is stopped after 120 seconds of wall-clock time'. It asserts the section's position is before '## Checks Athena will run' and before '## Working directory'. It asserts the section mentions `cargo test -p <crate> <filter>` and 'at most once'. The check `cargo test --locked` (criterion 6) also runs it. Run: `cargo test -p athena-context a_small_job_brief_carries_its_short_deadline`. Observes: A small job (one deliverable, short limit) is told its limit and the pacing rule before the checks list and before it starts.
  Fails without the behavior: Without the section the brief lacks '120 seconds' and the pacing text, so the test fails. If the section were placed after the checks list, the ordering assertion fails.
  Test seam: Production entry point: `producer_brief` (briefs.rs:675) with the helper `producer_inputs` (briefs.rs:1797) and the `reading()` fixture; inputs are plain values and there is no double. The seam exists; step 6 adds `deadline_s` to the helper and step 7 adds the test.

Planned files:

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 030c0240d8e2 → candidate 812031c2bf78. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   1 +
 crates/athena-context/src/briefs.rs    | 121 +++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs      |   1 +
 crates/athena-core/src/run/mod.rs      |   1 +
 crates/athena-core/src/run/producer.rs |   8 ++-
 crates/athena-core/src/run/tests.rs    |  92 +++++++++++++++++++++++++
 6 files changed, 223 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045647-w91y/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045647-w91y/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43640444541 for 812031c2bf78cade4cfef562a9a86ad095914b5f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43640444541 for 812031c2bf78cade4cfef562a9a86ad095914b5f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43640444541 for 812031c2bf78cade4cfef562a9a86ad095914b5f)
- ci-result: `not a check` passed (Check results for 812031c2bf78cade4cfef562a9a86ad095914b5f come from the CI run https://github.com/doodla/athena/actions/runs/43640444541 (910s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy and cargo test pass, as recorded by Athena's checks on the final tree.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context time_budget`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text

error[E0609]: no field `deadline_s` on type `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:1837:20
     |
1837 |             inputs.deadline_s = deadline_s;
     |                    ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `request`, `reading`, `checks`, `commands` ... and 4 others

error[E0609]: no field `deadline_s` on type `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:1865:15
     |
1865 |         fresh.deadline_s = 300;
     |               ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `request`, `reading`, `checks`, `commands` ... and 4 others

error[E0609]: no field `deadline_s` on type `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:1907:16
     |
1907 |         inputs.deadline_s = 120;
     |                ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `request`, `reading`, `checks`, `commands` ... and 4 others

error[E0560]: struct `briefs::ProducerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2370:13
     |
2370 |             deadline_s: 1800,
     |             ^^^^^^^^^^ `briefs::ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `briefs::ProducerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2394:13
     |
2394 |             deadline_s: 1800,
     |             ^^^^^^^^^^ `briefs::ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `briefs::ProducerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2416:13
     |
2416 |             deadline_s: 1800,
     |             ^^^^^^^^^^ `briefs::ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `briefs::ProducerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2444:13
     |
2444 |             deadline_s: 1800,
     |             ^^^^^^^^^^ `briefs::ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-context` (lib test) due to 13 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core the_producer_deadline_in_the_stored_brief_is_the_one_the_invocation_runs_under`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling libc v0.2.190
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling proc-macro2 v1.0.107
   Compiling winnow v0.7.15
   Compiling quote v1.0.47
   Compiling syn v3.0.7
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling getrandom v0.3.4
   Compiling linux-raw-sys v0.12.1
   Compiling bitflags v2.13.2
   Compiling aho-corasick v1.1.5
   Compiling toml v0.9.12+spec-1.1.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror v2.0.21
   Compiling regex-syntax v0.8.11
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling schemars_derive v1.2.2
   Compiling tempfile v3.27.0
   Compiling ref-cast v1.0.27
   Compiling regex-automata v0.4.18
   Compiling anyhow v1.0.104
   Compiling serde v1.0.229
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling indexmap v2.14.2
   Compiling schemars v1.2.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling regex v1.13.1
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0560]: struct `ProducerInputs` has no field named `deadline_s`
    --> crates/athena-core/src/reading.rs:3600:17
     |
3600 |                 deadline_s: 1800,
     |                 ^^^^^^^^^^ `ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

For more information about this error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text
     |
1865 |         fresh.deadline_s = 300;
     |               ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `request`, `reading`, `checks`, `commands` ... and 4 others

error[E0609]: no field `deadline_s` on type `briefs::ProducerInputs`
    --> crates/athena-context/src/briefs.rs:1907:16
     |
1907 |         inputs.deadline_s = 120;
     |                ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `request`, `reading`, `checks`, `commands` ... and 4 others

error[E0560]: struct `briefs::ProducerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2370:13
     |
2370 |             deadline_s: 1800,
     |             ^^^^^^^^^^ `briefs::ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `briefs::ProducerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2394:13
     |
2394 |             deadline_s: 1800,
     |             ^^^^^^^^^^ `briefs::ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `briefs::ProducerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2416:13
     |
2416 |             deadline_s: 1800,
     |             ^^^^^^^^^^ `briefs::ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `briefs::ProducerInputs` has no field named `deadline_s`
    --> crates/athena-context/src/briefs.rs:2444:13
     |
2444 |             deadline_s: 1800,
     |             ^^^^^^^^^^ `briefs::ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-context` (lib test) due to 13 previous errors
warning: build failed, waiting for other jobs to finish...
error[E0560]: struct `ProducerInputs` has no field named `deadline_s`
    --> crates/athena-core/src/reading.rs:3600:17
     |
3600 |                 deadline_s: 1800,
     |                 ^^^^^^^^^^ `ProducerInputs` does not have this field
     |
     = note: all struct fields are already assigned

For more information about this error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 1 previous error
```
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-context a_small_job_brief_carries_its_short_deadline`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
