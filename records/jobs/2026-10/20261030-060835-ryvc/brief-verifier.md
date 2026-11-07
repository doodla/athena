# Job 20261030-060835-ryvc: verify "Require a measured stage timeline in readings of speed requests"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The request named a fix (move the call earlier) and a timing figure, but nothing required the readin
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: speed-change-requested-without-checking-measured-stage-timeline (20261025-110542-wa68)
> Cause: The request named a fix (move the call earlier) and a timing figure, but nothing required the reading or planner to check that figure against the real stage order and durations before planning around it. The class already exists for a speed...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261025-110857-eqhd (job 20261025-110525-8hzb); prevention it proposed: For any request that claims a delay or speed-up, the reading stage must first produce a timeline from the code and the recorded run timings: each stage in order, its measured duration, and where the claimed delay sits. The reading must stat...
> Introspection 20261025-183009-xw7y (job 20261025-034733-c453); prevention it proposed: Add a measure-first step for any request to speed up a pipeline or check. Before the reading is accepted, Athena runs or reads a timing breakdown (compile versus run, per test binary or stage) and puts it in the reading. The reading must th...
> Introspection 20261025-190659-jvd7 (job 20261025-190606-eebk); prevention it proposed: Add a required measurement step to any request or reading whose goal is speed. Before the reading fixes a method, it must record a per-stage or per-binary timeline from a real run. It must also try each candidate lever, such as a different ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Goal: stop speed or delay requests being planned from the request's own figures (jobs 20261025-110525-8hzb, 20261025-034733-c453, 20261025-190606-eebk). Add one reading-acceptance check, `check_speed_timeline`, that refuses a reading of such a request unless it has a `Timeline:` assumption naming the crates/ code read and a measured duration per stage. Audience: the reader, planner and verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: check_reading_acceptance calls refusing_check, which runs `step!` lines in order (check_prior_result_and_scope, check_criteria_evidence, check_code_survey, ...) and receives the request text; check_code_survey (reading.rs ~3910) is the model for a check on assumptions. Reading records are validated by build_reading; this change adds no record or data file, so check_record_file and AnyRecord are not involved. The property (a refused reading) is measured directly by calling refusing_check on real request and reading shapes, via crates/athena-core/tests/refusal_corpus.rs, which fails when a step! has no corpus case. Reader rules live in READING_RULES in crates/athena-context/src/briefs.rs.
- Root cause: the reading stage only checked a request's figures if the reader chose to, so readings planned around stated timings (move the call earlier, 12 minutes of checks, split compile versus run) without a stage timeline from the code. Fix at the reading gate so it covers the whole class, not one job.
- A speed request is detected from the request text by whole-word cues such as faster, speed up, slow, delay, latency, cut ... minutes, head start, timing. The rule requires an assumption starting `Timeline:` that names a crates/ path and gives at least two stages each with a duration figure (a number with s, ms, min or minutes) and where the claimed delay sits, or says no recorded timing exists.
- Prior jobs: 20261025-110525-8hzb (failed in planning; its request misplaced where the delay sits) and 20261025-034733-c453 (failed in planning; no measured compile/run split) had no candidate, so no files are kept; this change addresses their cause, the missing timeline, and does not redo their work. 20261025-190606-eebk's findings (6 minute target unmet, production behavior changed in secrets.rs and checks.rs, files outside the list) are addressed by dropping that candidate's secrets.rs and checks.rs changes; its CHANGELOG.md and docs/decisions.md edits are not kept as they were, and new entries are written for this change only.
- Expected 5% of newly refused readings: only readings of speed requests lacking a Timeline: assumption are refused; the higher refusal rate is intended.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new corpus case in crates/athena-core/tests/refusal_corpus.rs gives a speed request (e.g. 'move the call earlier to save 12 minutes') and a reading with no `Timeline:` assumption, and asserts refusing_check refuses it with `check_speed_timeline`; the same test fails if the step! line is removed.
2. [behavior] Run `cargo test -p athena-core`: a reading of the same speed request with a `Timeline:` assumption naming a crates/ path and per-stage durations is accepted, and a non-speed request without a timeline is accepted unchanged.
3. [content] crates/athena-core/src/reading.rs has `check_speed_timeline` added as one `step!(check_speed_timeline(reading, request))` line in refusing_check, with a doc comment naming the root cause and jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, and stating the refusal rate is intended (expected 5% of newly refused readings).
4. [content] READING_RULES in crates/athena-context/src/briefs.rs lists the new rule and names `check_speed_timeline`, and the reader brief tells the reader what a Timeline: assumption must hold; the briefs.rs tests for the rule list pass.
5. [content] docs/decisions.md records the decision with alternatives (prompt-only guidance, planner-stage check, reading gate check) and the cause; CHANGELOG.md has an entry.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass; no existing test is removed, ignored or weakened.
7. [content] crates/athena-gate/src/secrets.rs and crates/athena-core/src/checks.rs are unchanged by this job.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Speeding up any check, test or pipeline stage itself
- Re-doing the early pull request move of job 20261025-110525-8hzb or the nextest work of job 20261025-034733-c453
- Keeping the changes of job 20261025-190606-eebk to crates/athena-gate/src/secrets.rs and crates/athena-core/src/checks.rs (dropped)
- Changing athena.toml, defaults.toml or CI workflow files
- Changing planner or critic prompts

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the reading gate where speed requests are accepted, covering the class.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Corpus and unit tests, fmt, clippy and test criteria.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by a check in refusing_check, not prose only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test on refusal and acceptance cases.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check plus its rule line, tests and records.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the refusal corpus. A new `speed_timeline_cases()` in refusal_corpus.rs is added to `cases()`. Each Refuse case is `Expected::Refuse("check_speed_timeline")` and is built from `base_reading()` with one field changed, so no earlier check refuses it. Input forms and case ids: plain request 'Move the call earlier to save 12 minutes.' = `speed-request-without-a-timeline`; dash bullet = `speed-word-in-a-dash-bullet`; star bullet = `speed-word-in-a-star-bullet`; `1.` part = `speed-word-in-a-numbered-dot-part`; `2)` part = `speed-word-in-a-numbered-paren-part`; `(a)` part = `speed-word-in-a-lettered-paren-part`; retry complaint = `retry-complaint-still-too-slow-without-a-timeline`; a request that cites a job without relying on it = the existing root-cause cases at refusal_corpus.rs:604-617 stay Accept, plus new `root-cause-request-with-its-own-speed-goal-without-a-timeline` (Refuse); a crate-relative path = `timeline-with-a-crate-relative-path` (Refuse, names src/reading.rs); a brand word or environment variable is not read by this rule, so no case is added (the rule reads only whole words and durations). More Refuse cases: `root-cause-request-whose-cause-is-slowness-without-a-timeline`, `slowdown-request-without-a-timeline`, `throughput-design-request-without-a-timeline`, `high-priority-spec-request-without-a-timeline`, `approved-proposal-title-with-a-cue-without-a-timeline`, `takes-too-long-without-a-timeline`, `get-the-checks-under-6-minutes-without-a-timeline`, `reduce-ci-time-to-6-minutes-without-a-timeline`, `shorten-the-run-without-a-timeline`. Timeline-defect Refuse cases: `timeline-without-a-crates-path`, `timeline-with-one-stage-only`, `timeline-stage-without-a-duration`, `timeline-says-no-recorded-timing-without-stages`, `timeline-without-where-the-delay-sits`, `timeline-place-word-names-no-stage`, `timeline-label-not-at-the-start`. Fourteen new INPUT_FORMS tags are added, including `spec-requirement-quote` and `approved-proposal-quote`, and the asserted length is updated (read the file to confirm the count; about 23 to 37). The hestia-shaped cases come from a helper that mirrors the line shapes of the real builders. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: `corpus_cases_get_the_expected_outcome` passes: each Refuse case is refused by `check_speed_timeline` and by no earlier check. `every_acceptance_check_has_a_refusing_corpus_case` passes with the new step. `corpus_covers_the_listed_input_forms` passes with the new tags.
  Fails without the behavior: Remove the `step!(check_speed_timeline(reading, request));` line and every Refuse case is accepted ('expected Refuse("check_speed_timeline"), got accepted'). Drop the structural marker rule and the slowdown and throughput cases fail. Drop the human-form cues and those cases fail. Drop the Timeline requirement and the defect cases fail. Drop the Priority rule and the high-priority spec case fails.
  Test seam: Production entry point `athena_core::reading::refusing_check` (reading.rs:3226, exists), called by `run(c)` in refusal_corpus.rs (exists) with `Some(repo_files())` injected (exists). `case()` and `base_reading()` exist. Guards: capability is Some and question is None. `speed_timeline_cases()` and the builder-shape helper are added by plan step 8. No double holds check logic.
- Criterion 2: The corpus holds accept cases, and unit tests in reading.rs, hestia.rs and work_decisions.rs call the detector on real builder output. Corpus accepts: `speed-request-with-a-timeline` ('Timeline: read crates/athena-core/src/reading.rs; stages: compile 9 min, check 40 s; the delay sits in compile.'); `speed-request-timeline-says-no-recorded-timing` (stages named, no durations, place given); `retry-complaint-…-with-a-timeline`; `slowdown-request-with-a-timeline`; `throughput-design-request-with-a-timeline`; `root-cause-request-quoting-speed-words-in-its-prevention-needs-no-timeline`; `spec-request-with-faster-in-its-statement-needs-no-timeline` (Priority: normal, statement holds 'faster'); `high-priority-spec-request-with-a-timeline`; `approved-proposal-source-quoting-speed-words-needs-no-timeline`; `speed-request-question-reading-needs-no-timeline`; `speed-word-lookalikes-are-not-speed-requests` ('Add a timeline view and a speedometer widget. Write a quickstart guide. Wait 30 seconds before the retry. Cover the slowloris case.'); the non-speed pins `add-a-retry-delay-setting-…`, `fix-a-timing-attack-…`, `save-the-state-every-30-seconds-…`, `set-the-retry-timeout-to-10-s-…`, `reduce-the-retry-timeout-by-5-s-…`, `trim-the-log-after-5-minutes-…`, `reduce-the-grace-period-to-5-minutes-…`, `the-title-is-too-long-…`, `shorten-the-error-message-…` (each ending `-is-not-a-speed-request`); and the existing `steward-three-dash-bullets`. reading.rs unit tests `speed_requests_are_found_by_whole_word_cues_and_figures` and `a_timeline_assumption_needs_stages_durations_and_a_placed_delay` cover the positives, negatives and Timeline defects (one-line error starting 'the reading was refused because'). hestia.rs tests feed real output into `crate::reading::speed_request`: `slowdown_request(25.0, 60.0, &growth)` is true; `throughput_request` is true; `root_cause_request` with speed words only in prevention text is false; with Cause 'verifier runs time out because the suite is too slow' is true; this job's own visible Cause is false; cue-free text plus an own line 'Make it faster.' is true; `spec_request` for a gap with `speed: false` whose statement says 'faster' in passing is false (matching hestia.rs:3229-3233); `spec_request` for a `speed: true` gap is true. A work_decisions.rs test builds `approve_request` text and checks that a speed word in the source or evidence is false and one in the title is true. Both can be done by a unit test on the formatted text if building Records/Git is heavy; the producer chooses the existing fixture used by the test at work_decisions.rs:534. Run: `cargo test -p athena-core`. Observes: All accept cases return None from `refusing_check`; unit and builder-output tests pass; the check is skipped when capability is None or a question is set.
  Fails without the behavior: If the check refused every speed request regardless of the Timeline, `speed-request-with-a-timeline` fails. If the detector scanned prevention lines, the prevention accept case and the hestia.rs false test fail. If it scanned the spec statement, the spec 'faster' accept case and the hestia.rs spec test fail. If it scanned the approved-proposal Source, that accept case fails. If the Cause line were dropped, the slowness-cause Refuse case fails. If bare delay/timing/slow or any cut verb plus a figure counted, the retry-delay, timing-attack, 30-seconds, retry-timeout and grace-period pins fail. If `too long` alone counted, 'the title is too long' fails. If the check applied to non-speed requests, `steward-three-dash-bullets` fails.
  Test seam: Production entry points `refusing_check`, `check_speed_timeline` and `speed_request` (the last two added by step 3; `speed_request` is `pub(crate)`). The hestia.rs tests call the real `root_cause_request` (hestia.rs:453), `slowdown_request` (:1735), `throughput_request` (:1611) and `spec_request` (:691), all existing, with the builders and setups used at hestia.rs:3241, :3381-3400, :3609-3620 and :5765. `approve_request` (work_decisions.rs:279, exists) is called in the test at work_decisions.rs:534. `governed_reading` (reading.rs:7364) exists. No double holds check logic.
- Criterion 3: A corpus test `speed_timeline_refusal_is_declared_intended`, added in step 8 and modelled on `design_existing_code_refusal_is_declared_intended` (refusal_corpus.rs:1102), reads reading.rs through `READING_SOURCE` and asserts that the exact line `step!(check_speed_timeline(reading, request));` appears once in `refusing_check`, directly after the `check_outcome_state` step. It also asserts that the doc paragraph above `pub fn check_speed_timeline(` is one `///` paragraph holding the root-cause sentence, the three ids, the limits (shape not truth; crates/ path only; quoted text of other people not scanned), 'the higher refusal rate is intended' and 'expected 5% of newly refused readings', and that `replay_declaration` (reading.rs:4337) returns share 5 for it. Run: `cargo test -p athena-core --test refusal_corpus speed_timeline_refusal_is_declared_intended`. Observes: The step line is present once after `check_outcome_state`; the doc holds the ids 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk; the declared share is 5.
  Fails without the behavior: Without the step line or the doc paragraph the string assertions fail.
  Test seam: `include_str!("../src/reading.rs")` as READING_SOURCE (exists) and `athena_core::reading::replay_declaration` (exists). The test is added by step 8.
- Criterion 4: Update `the_reader_brief_names_each_reading_rule_in_one_short_line` (rule count 6 to 7, `check_speed_timeline` added to the named-check list) keeping the 220-character and 2400-byte caps. Add `the_reader_brief_says_what_a_timeline_assumption_holds`, which finds the rule line in the rendered reader brief. Run: `cargo test -p athena-context briefs`. Observes: The `# Reading rules` section has seven lines; one names `check_speed_timeline`, `Timeline:`, a crates/ path, `stages:`, measured durations and where the delay sits; every line is at most 220 characters and the section is under 2400 bytes.
  Fails without the behavior: Without the rule line the count is 6 and the named-check loop fails for `check_speed_timeline`.
  Test seam: `reader_brief(&reader_inputs(&reading()))` (briefs.rs:2251; helper at briefs.rs:4385, both exist). `READING_RULES` is the constant step 6 changes. No fake.
- Criterion 5: The step 8 corpus test also reads docs/decisions.md and CHANGELOG.md with whitespace collapsed. decisions.md must hold `## D163`, `check_speed_timeline`, the alternatives (prompt-only guidance, planner-stage check, reading gate check, verifier-stage check, running the stages), the cause wording, the quoted-text rules (prevention lines, spec statement, approved-proposal source), the limits (crates/ path; shape not truth; builder-quoted job titles in a design review are scanned) and the measured share or the reason it could not be measured. The changelog must hold `check_speed_timeline`, 'higher refusal rate is intended', 'expected 5% of newly refused readings', D163, and the four files beyond the listed deliverables. The decision-log guard confirms D163 is well formed and appended. Run: `cargo test -p athena-core --test refusal_corpus speed_timeline_refusal_is_declared_intended && cargo test -p athena-cli the_decision_log_is_well_formed_and_only_appended_to`. Observes: Both pass; `check_log` (athena-cli decisions.rs:218) finds D163 in order with a Provenance bullet and the earlier text unchanged.
  Fails without the behavior: Without the D163 entry or the changelog entry the string assertions fail.
  Test seam: include_str of `../../../CHANGELOG.md` and `../../../docs/decisions.md` (the form the existing test uses, exists); `check_log` exists.
- Criterion 6: Run the three checks Athena runs. No test is removed or ignored; the only edits to existing assertions are the rule count 6 to 7 in briefs.rs and the INPUT_FORMS length, both forced by added rules and tags and said so in the summary. The full run covers every drain test through the fake reader and the real gate in a fixture repo with no crates/ directory, the hestia tests that assert the rendered text byte for byte, the work_decisions tests, and the new drain test `the_fake_readers_timeline_passes_the_speed_gate_and_the_reference_check`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. `a_speed_improving_request_is_read_as_urgent_with_a_reason_and_an_ordinary_one_is_not` (drain.rs:7008) passes because the fake's Timeline passes the speed gate and, through the absence phrase, the reference check.
  Fails without the behavior: Without the fake.rs line the drain test fails: the request is refused twice by `check_speed_timeline`. With a bare crates/ path in the fake, every fixture-based drain test fails in `check_reading_references`. If ABSENCE_PHRASES stops skipping the fake's clause, the pin test fails with a message naming the fake's Timeline.
  Test seam: Whole workspace. `athena_agent::fake::deterministic_turn` (fake.rs:195, exists) is the injected reader via `fx.athena(deterministic())` (drain.rs:7010, exists). The new drain test uses the same `Fixture`, the recorded reading from `Records::load`, and the file list as at drain.rs:3971. Step 7 adds the fixed assumption; the fake holds no check logic.
- Criterion 7: Compare the two files with the job's base commit. No step edits either file; the check lives in reading.rs. Run: `git diff --stat -- crates/athena-gate/src/secrets.rs crates/athena-core/src/checks.rs`. Observes: No output: neither file is changed.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 1 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

- 20261105-163159-vsfy: the reading was refused because the request asks for something to be faster or a delay to be cut, and its reading has no valid `Timeline:` assumption: it has no assumption that starts `Timeline:`; the

## The change

Base 762e4572ac02 → candidate fa2f7362bc0b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  23 ++
 crates/athena-agent/src/fake.rs            |   8 +-
 crates/athena-cli/tests/drain.rs           |  48 +++
 crates/athena-context/src/briefs.rs        |  31 +-
 crates/athena-core/src/hestia.rs           | 124 +++++-
 crates/athena-core/src/reading.rs          | 499 +++++++++++++++++++++-
 crates/athena-core/src/work_decisions.rs   |  34 +-
 crates/athena-core/tests/refusal_corpus.rs | 639 ++++++++++++++++++++++++++++-
 docs/decisions.md                          |  70 ++++
 9 files changed, 1464 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-060835-ryvc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060835-ryvc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42375578555 for fa2f7362bc0b0f56485477d6f73c6d4e039dee50)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42375578555 for fa2f7362bc0b0f56485477d6f73c6d4e039dee50)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42375578555 for fa2f7362bc0b0f56485477d6f73c6d4e039dee50)
- ci-result: `not a check` passed (Check results for fa2f7362bc0b0f56485477d6f73c6d4e039dee50 come from the CI run https://github.com/doodla/athena/actions/runs/42375578555 (721s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass; no existing test is removed, ignored or weakened.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/work_decisions.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
obs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
speed-word-in-a-lettered-paren-part (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
head-start-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
retry-complaint-still-too-slow-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
takes-too-long-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
get-the-checks-under-6-minutes-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
reduce-ci-time-to-6-minutes-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
shorten-the-run-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
root-cause-request-with-its-own-speed-goal-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
root-cause-request-whose-cause-is-slowness-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
slowdown-request-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
throughput-design-request-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
high-priority-spec-request-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
approved-proposal-title-with-a-cue-without-a-timeline (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
timeline-without-a-crates-path (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
timeline-with-a-crate-relative-path (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
timeline-with-one-stage-only (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
timeline-stage-without-a-duration (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
timeline-says-no-recorded-timing-without-stages (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
timeline-without-where-the-delay-sits (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
timeline-place-word-names-no-stage (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
timeline-label-not-at-the-start (jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk, which asked for speed; rebuilt from the job briefs, not recorded requests): expected Refuse("check_speed_timeline"), got accepted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- speed_timeline_refusal_is_declared_intended stdout ----

thread 'speed_timeline_refusal_is_declared_intended' (19194) panicked at crates/athena-core/tests/refusal_corpus.rs:1707:5:
assertion `left == right` failed
  left: "step!(check_boundary_and_repo_files(reading, repo_files));"
 right: "step!(check_speed_timeline(reading, request));"


failures:
    corpus_cases_get_the_expected_outcome
    speed_timeline_refusal_is_declared_intended

test result: FAILED. 11 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/work_decisions.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |
7459 |         assert!(!speed_request(&spec("normal")));
     |                  ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `speed_request` in this scope
    --> crates/athena-core/src/reading.rs:7461:17
     |
7461 |         assert!(speed_request("Throughput slowdown: checks grew\n\nBody."));
     |                 ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `speed_request` in this scope
    --> crates/athena-core/src/reading.rs:7462:17
     |
7462 |         assert!(speed_request("Throughput design: a bottleneck\n\nBody."));
     |                 ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_speed_timeline` in this scope
    --> crates/athena-core/src/reading.rs:7474:21
     |
7474 |         let error = check_speed_timeline(&timeline_reading(None), request)
     |                     ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_speed_timeline` in this scope
    --> crates/athena-core/src/reading.rs:7483:9
     |
7483 |         check_speed_timeline(&timeline_reading(Some(good)), request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_speed_timeline` in this scope
    --> crates/athena-core/src/reading.rs:7491:13
     |
7491 |             check_speed_timeline(&timeline_reading(Some(ok)), request)
     |             ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_speed_timeline` in this scope
    --> crates/athena-core/src/reading.rs:7532:25
     |
7532 |             let error = check_speed_timeline(&timeline_reading(Some(bad)), request)
     |                         ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_speed_timeline` in this scope
    --> crates/athena-core/src/reading.rs:7539:9
     |
7539 |         check_speed_timeline(&timeline_reading(None), "Update the guide.").unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_speed_timeline` in this scope
    --> crates/athena-core/src/reading.rs:7542:9
     |
7542 |         check_speed_timeline(&declined, request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_speed_timeline` in this scope
    --> crates/athena-core/src/reading.rs:7545:9
     |
7545 |         check_speed_timeline(&asking, request).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 31 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus speed_timeline_refusal_is_declared_intended`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/work_decisions.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 1 test
test speed_timeline_refusal_is_declared_intended ... FAILED

failures:

---- speed_timeline_refusal_is_declared_intended stdout ----

thread 'speed_timeline_refusal_is_declared_intended' (19383) panicked at crates/athena-core/tests/refusal_corpus.rs:1707:5:
assertion `left == right` failed
  left: "step!(check_boundary_and_repo_files(reading, repo_files));"
 right: "step!(check_speed_timeline(reading, request));"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    speed_timeline_refusal_is_declared_intended

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-context briefs`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core --test refusal_corpus speed_timeline_refusal_is_declared_intended && cargo test -p athena-cli the_decision_log_is_well_formed_and_only_appended_to`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
