# Job 20261028-124903-wngz: verify "Retry 3d47: replay refusal-rule changes over 50 past readings"

## Request (verbatim)

> try again on 20261028-103330-3d47

## Reading (accepted by the user)

Retry of job 20261028-103330-3d47, which failed only because the agent OAuth token expired (401) before the producer did any work; that is an environment stop, not a finding on the work, so the work is the same as its accepted reading. Add an `athena check-reading` subcommand and replay the newest 50 accepted readings through base and candidate builds. Store the result as a typed record, show it to the verifier, and hold the job if over 10% are newly refused. Audience: maintainers and the verifier.

Assumptions:
- Job 20261028-103330-3d47 ended with no verification; its single recorded outcome was the 401 expired token, so no code finding needs fixing. Its discarded changes start again from the base.
- The introspection's proposed credential preflight and 401 environment-stop work is a separate job; this retry does only the replay work, as the typed request says. If the token is still expired, the retry will fail the same way.
- Same design as the accepted reading: replay is run by the `athena check-reading` subcommand from base and candidate builds, every candidate gets it, and there is no flag detection and no replay.md.
- The 10% threshold is a code constant. The override in tests sits in front of tree resolution and the subcommand probe. No-model-call is shown by comparing model call counts with and without a candidate rule change.

## Acceptance criteria

1. [behavior] `athena check-reading` takes a stored request and reading and prints accept or refuse with the reason, using the reading.rs rules; a fixture test shows both outputs (cargo test -p athena-cli).
2. [behavior] A test builds a candidate-like rule change over fixture records and lists the newly refused requests (refused by candidate, accepted by base) with the count (cargo test -p athena-core).
3. [behavior] A test shows the replay uses only the newest 50 requests with accepted readings, and that an empty history gives 'no replay: <reason>', not a pass (cargo test -p athena-core).
4. [behavior] A test shows more than 10% newly refused is held with the list, exactly 10% or less is not held, and a candidate whose criteria intend a higher rate is not held, with the replay still recorded (cargo test -p athena-core).
5. [behavior] A pipeline test, with the override in front of tree resolution and the subcommand probe, shows a candidate over the threshold is Rejected, not landed; the typed replay record and the Athena Blocker finding name the list, the records check accepts the job directory, and no replay.md is created (cargo test -p athena-core).
6. [behavior] The typed replay field or record kind has a schema round-trip test and passes the records check (cargo test -p athena-data).
7. [behavior] A pipeline test shows model call counts are equal for a run with a candidate rule change and one without, so the replay makes no model call (cargo test -p athena-core).
8. [behavior] A test in crates/athena-context/src/briefs.rs shows the verifier brief carries the replay summary (newly refused count, total, list) or 'no replay: <reason>', and stays within its size limit with a very long list (cargo test -p athena-context).
9. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
10. [content] CHANGELOG.md has an entry describing the subcommand, the replay, the 10% hold, the typed record and the verifier brief summary.
11. [content] git diff shows no change to athena.toml, defaults.toml or .github/workflows; the 10% threshold is a code constant; no existing reading rule changed; no replay.md is written and no skip switch is added.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/verify.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Changing any existing reading rule
- Detecting refusal-rule changes by a flag or by matching diffs
- Creating a free-form replay.md in the job directory
- Adding a switch to skip the replay
- Automatic disk cleanup or disk reservation before the verifier
- Credential preflight and 401 environment-stop handling (separate job from the introspection)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests per module plus fmt, clippy -D warnings and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name the cargo test commands that exercise the behavior.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check refusals listed and held, not just that code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Credential preflight left out as a separate job.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Prior failure was an environment stop (expired token), not a work defect, so the work is unchanged.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in main.rs `mod tests` call `check_reading_command(request_path, reading_path)`, the function `run` calls for `Command::CheckReading`. It reads a request file and a reading file (a serialized `AnyRecord::Reading` written to a tempdir) and returns the printed line and the exit code. Preconditions: both files exist and the reading parses with `AnyRecord::Parse`; no repository, model or network is needed. Fixture A is a reading with command-checkable criteria. Fixture B has a criterion resting on 'the producer's report', which `check_criteria_evidence` refuses. `Cli::try_parse_from(["athena","check-reading","--request",..,"--reading",..])` is also tested to give `Command::CheckReading`. The clap variant is added in step 3; `check_reading_text` in reading.rs (step 2) calls `check_reading_acceptance`; nothing is injected. Run: `cargo test -p athena-cli check_reading`. Observes: A returns ('accept', 0). B returns a line starting 'refuse: ' that contains the 'resting on the producer's report' reason, and code 2. An unreadable reading file gives an error, not 'accept'.
  Fails without the behavior: Without the subcommand the parse test fails. If the handler printed accept for everything, fixture B fails. If it ignored the reading.rs rules, the reason text check fails.
- Criterion 2: Unit test in reading.rs. It calls the production `replay(&cases, &base, &candidate)` over five fixture `ReplayCase`s. `base` is a checker that calls the real `check_reading_acceptance`. `candidate` wraps `base` and adds one extra refusal (title contains a marker). Two cases are refused by both (a criterion on the producer's report), one is accepted by both, and two carry the marker and are accepted by base. `ReadingChecker`, `ReplayCase` and `replay` are added in step 2; they do not exist today. Run: `cargo test -p athena-core replay_lists`. Observes: `newly_refused` holds exactly the two marker jobs, each with its reason, `newly_refused.len()` is 2, and `total` is 5. The two refused-by-both cases are not listed.
  Fails without the behavior: If `replay` listed every candidate refusal, or compared in the wrong direction, the refused-by-both cases would appear and the count would be 4 or more.
- Criterion 3: Two unit tests in reading.rs on the production `replay_cases(&Records, exclude: Option<&JobId>)` and `replay`. (a) Build `Records` with 60 jobs that have `accepted_reading` and a matching reading and request (ids 2020-dated and ascending), plus 5 jobs with none, plus one job named in `exclude`. `replay_cases` must return 50 cases, which are the 50 newest ids, with no unaccepted job and not the excluded one. (b) `replay_cases` on empty `Records` returns no cases, and `replay(&[], ..)` gives `no_replay == Some("no replay: no accepted readings in the history")`, `held == false`, `total == 0`. Records are built in memory (`Records::default()` and `JobRecords` fields), so no repository is needed. Run: `cargo test -p athena-core replay_window`. Observes: Exactly 50 cases, which are the newest ids. For the empty history a reason beginning 'no replay:' and a record that is not a pass.
  Fails without the behavior: With no window, 60 cases would be replayed. If an empty history were a pass, `no_replay` would be None.
- Criterion 4: Unit tests in reading.rs on the production `replay_outcome(total, newly, intended)`, `criteria_intend_higher_refusal` and `replay`. Case 1: 10 cases, 2 newly refused, criteria without intent: held with the 2-item list. Case 2: 10 cases, 1 newly refused: exactly 10%, not held. Case 3: 10 cases, 2 newly refused, with criteria text 'the change refuses more readings than the base'. The test calls `criteria_intend_higher_refusal` on those criteria (intent is not passed by hand), so `intended` is true, `held` false, and the 2-item list and the record are still produced. Case 4 (negative): criteria that mention refusal without intent ('readings that already refuse stay refused') give intended false, so the 2 newly refused are held. Candidate/base checkers are as in criterion 2. Run: `cargo test -p athena-core replay_hold`. Observes: `held` true with the list in case 1; false at exactly 10%; false in case 3 with `newly_refused.len() == 2` still recorded; true in case 4.
  Fails without the behavior: A `>=` comparison would hold case 2. A missing intent check would hold case 3. A matcher without inflected phrases would hold case 3. An over-broad matcher would not hold case 4.
- Criterion 5: Pipeline test in run/mod.rs using the existing `repo()`, `athena_with(.., base_script())` and `Lines` helpers. Precondition: a new test helper `seed_accepted_jobs(&athena, 10)` commits 10 jobs, each with a Request, a Reading and a Job record whose `accepted_reading` is set, through `athena.commit_records(&[NewFile::record(..)], ..)`. That is the path `go_job` reads (`Records::load(&self.git, &head)` at run/mod.rs:192-193), and the ids are 2020-dated so they are older than the job under test. Three of the ten readings carry a marker in the title. The test then sets the `REPLAY_OVERRIDE` guard (added in step 4, read first in `replay_checkers`, so before tree resolution and the probe) to checkers: base = a checker calling the real `check_reading_acceptance`, candidate = that base plus a refusal on the marker. It calls `athena.ask(..)` and `athena.go(..)` and loads the records. The stage is added in step 5. Run: `cargo test -p athena-core replay_holds_over_threshold`. Observes: First `replay.total == 10`, `no_replay` is None and `newly_refused.len() == 3` (30%, above 10%), so an empty load, which would give 'no replay', cannot pass silently. Then: the final state is `OutcomeState::Rejected`, not Landed; `replay.json` parses as `AnyRecord::Replay` with `held` true and the 3 ids; `verification.json` holds an Athena-source Blocker finding that names the same ids; `check_record_file` accepts every file in the job directory; the job directory has no `replay.md`.
  Fails without the behavior: Without the stage, no replay.json exists and the job lands. Without the `judge` finding, the job is not Rejected. If the override were read after tree resolution, the fake pipeline would stop at 'no replay' and `total` would be 0.
- Criterion 6: Unit tests in records.rs beside the Baseline tests. A sample `Replay` is wrapped in `AnyRecord::Replay`, then `to_json`, `check_record_file(canonical_path, bytes)` and an equality check after `AnyRecord::parse`. `validate` rejects `no_replay` with a non-empty list, more refused than total, and `held` with an empty list. A store.rs test (`replay_record_loads`) puts a `<job>/replay.json` next to a request and a job in a fixture and calls `Records::load`, then reads `jobs[..].replay`. The test uses the same loading helpers as the neighbouring store tests. Run: `cargo test -p athena-data replay && cargo test -p athena-core replay_record_loads`. Observes: The round trip equals the original, the schema is `athena.replay/1`, the path is `<job>/replay.json`, invalid ones return `RecordError::Invalid`, and `Records::load` neither panics nor drops the record.
  Fails without the behavior: If the variant, `canonical_path` or the schema were missing, parse or the misplaced-path check would fail. Without the `job_of` arm `Records::load` panics; without the `JobRecords::add` arm `replay` stays None.
- Criterion 7: Two tests in run/mod.rs. (a) A script wrapper `role_log(base_script())` appends the `Role` of EVERY invocation (any role) to an `Arc<Mutex<Vec<Role>>>`. The test runs two jobs on identical repos seeded with `seed_accepted_jobs(.., 10)`. Run one has `REPLAY_OVERRIDE` with a candidate equal to the base (no rule change); run two has a candidate that adds a refusal, so it is held. It compares the two logs truncated at the first `Role::Verifier`, which leaves out the introspector call that follows the rejection. It also compares each job's written `Invocation` records minus introspection roles. It asserts that run two's replay.json is held. (b) `binary_checker_has_no_agent_handle`: the production `BinaryChecker::new(path)` (built from a path only; no driver, no `Athena`) is run over a fixture request and reading using an executable stub script that prints a `refuse: ..` line, and its output is parsed by the production parser. `BinaryChecker` and the log wrapper are added in steps 4 and 5. Run: `cargo test -p athena-core replay_makes_no_model_call`. Observes: The role sequences up to the verifier are identical ([Reader, Producer, Verifier]) for both runs, and the Invocation records match. The production checker returns the parsed accept or refuse from a path alone.
  Fails without the behavior: If the replay stage called an agent, an extra role would appear before `Verifier` in run two's log (or in both), and the prefix comparison or the exact-sequence assert would fail. A `BinaryChecker` that needed a driver would not compile in test (b).
- Criterion 8: Unit tests in briefs.rs call the production `verifier_brief_with_plan` with the new `VerifierInputs.replay: Option<ReplayView>`. Case 1: 2 of 10 refused gives a section containing '2 of 10' and both ids with their reasons. Case 2: `no_replay: Some(reason)` gives 'no replay: <reason>'. Case 3: 1000 refused entries with 2000-character reasons give a section no longer than the named cap (20 entries x 200 characters plus headings), 'and 980 more', and a brief under a fixed byte bound. Every existing `VerifierInputs` literal gains `replay: None`; with None, the brief text is unchanged. Run: `cargo test -p athena-context replay`. Observes: The summary text, the cap, and the unchanged brief when there is no replay.
  Fails without the behavior: If the summary were not rendered the substring checks fail. If the list were uncapped the length bound fails.
- Criterion 9: Run the three Athena checks in full after step 9. Every `VerdictInputs` and `VerifierInputs` literal gets the new `Option` field. Existing pipeline tests now also write a replay.json ('no replay: ...' from the fake driver); any test that lists job-directory files is updated. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A formatting error, a lint warning or any failing test fails the matching command.
- Criterion 10: Read the diff of the Unreleased section. Run: `git diff -- CHANGELOG.md`. Observes: An entry names `athena check-reading`, the newest-50 replay, the 10% hold, the `athena.replay/1` record and the verifier brief summary.
- Criterion 11: Read the diff stat, then `git status` for untracked files, then `git diff -- crates/athena-core/src/reading.rs` for the existing rules. Run: `git diff --stat`. Observes: No change to athena.toml, defaults.toml or .github/workflows. The threshold is `const REPLAY_HOLD_PERCENT: usize = 10`. The reading.rs diff only adds code; no existing check function is edited. No replay.md and no skip switch. Files outside the brief's list: store.rs only, for the two arms.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## The change

Base 7abf45ecda49 → candidate 3e5f6fdbe9d7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  21 ++
 crates/athena-cli/src/main.rs       | 191 ++++++++++++
 crates/athena-context/src/briefs.rs | 165 ++++++++++
 crates/athena-core/src/lib.rs       |  35 +++
 crates/athena-core/src/reading.rs   | 583 +++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/run/mod.rs   | 489 +++++++++++++++++++++++++++++-
 crates/athena-core/src/store.rs     |  48 ++-
 crates/athena-core/src/verify.rs    | 115 ++++++-
 crates/athena-data/src/records.rs   | 163 ++++++++++
 docs/records.md                     |   1 +
 10 files changed, 1799 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1561 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
