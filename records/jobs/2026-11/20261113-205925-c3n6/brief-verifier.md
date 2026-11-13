# Job 20261113-205925-c3n6: verify "Escalate planner model and effort on revision rounds via new keys"

## Request (verbatim)

> try again on 20261113-194336-scfc

## Reading (accepted by the user)

Retry of job 20261113-194336-scfc, following the introspection's proposed request. Add optional [models] planner_revision_model and planner_revision_effort. After a critic revise or mechanical plan-rule revision, the planner uses them only if set and not weaker than the current planner setting. Do not reuse hard_producer_*. Round 1 is unchanged. The owner who runs Athena can check this in tests.

Assumptions:
- Prior job 20261113-194336-scfc: it failed in planning with no candidate, so nothing is kept. Its findings are addressed as follows. (a) Reusing hard_producer_* was a no-op here and could lower the setting: new dedicated keys are used and a weaker value is ignored. (b) The effect in this repo was misstated: the plan and CHANGELOG must state the effective planner model and effort with and without the change. (c) The criterion 1 wording about producer_setting no longer applies, because producer_setting is not used. Its out_of_scope items 'no new config keys' and 'no athena.toml change' are superseded by this request, which names new keys and an athena.toml entry. Its other out_of_scope items stay.
- Code survey: read crates/athena-data/src/config.rs. Models has hard_producer_effort at line 331. producer_setting is at line 346. effort_for is at line 830, and it maps the planner to verifier_effort. Key validation is around line 669. Not read in this session, taken from the earlier job's findings: crates/athena-core/src/run/planning.rs (planning_round, lines 138-155), crates/athena-core/src/reading.rs (agent_spec, line 443), and the fake-agent helpers in crates/athena-core/src/run/tests.rs. The entry is Athena::go. No new record or data file is added, so check_record_file and AnyRecord are not involved. The property is measured by the recorded planner invocation model and effort per round.
- Existing code: crates/athena-data/src/config.rs (Models, validation, effort_for), crates/athena-core/src/run/planning.rs (planning_round), docs/decisions.md and docs/spec.toml are extended.
- Consumers: planner_revision_* are read only by planning_round in crates/athena-core/src/run/planning.rs and validated in crates/athena-data/src/config.rs. The athena.toml entry is the only user-set value.
- Assumption: 'not weaker' is judged for effort by position in EFFORT_LEVELS (config.rs:337). For the model, it is judged by position in producer_allowed, which is assumed to be ordered weakest to strongest. The producer reads this ordering and states it in a doc comment. A key that would lower either value is ignored, and only that part falls back.
- Assumption: both keys must be in the allowed lists when set (model in producer_allowed, effort in EFFORT_LEVELS), matching how the hard_producer_* keys are validated. Unset keys leave behavior unchanged.
- Assumption: athena.toml gets a commented or actual entry for the new keys. The producer picks values stronger than the current planner settings so the change takes effect here, and the CHANGELOG says what the keys do in this repo.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/tests.rs drives Athena::go with the fake agent and a critic revise. It shows that the first planner invocation records the ordinary model and effort, and that the next one records planner_revision_model and planner_revision_effort. Run `cargo test -p athena-core` and cite the test name.
2. [behavior] New tests show that a mechanical plan-rule revision also escalates the next planner call. They also show that a key weaker than the current planner model or effort is ignored, and that with both keys unset every round keeps the same model and effort. Run `cargo test -p athena-core`.
3. [behavior] New config.rs unit tests show that the keys parse as optional, that a value outside the allowed lists is rejected, and that an existing config without the keys still loads. Run `cargo test -p athena-data`.
4. [content] In crates/athena-data/src/config.rs, Models has optional planner_revision_model and planner_revision_effort. They are not derived from hard_producer_model or hard_producer_effort. Grep planning.rs for a diff showing no use of producer_setting or hard_producer_* in the planner path.
5. [behavior] The owner can see the escalation. The planner line printed by planning_round, or the invocation record, shows the model used on the revision round. A test in crates/athena-cli/tests/drain.rs or in the core tests asserts it. Run `cargo test -p athena-cli` if drain.rs is changed.
6. [content] athena.toml has the new planner_revision_* entry under [models]. CHANGELOG.md has an entry that states the effective planner model and effort in this repo with and without the change, and what the keys do. docs/spec.toml, where it describes planning, mentions the revision setting.
7. [content] docs/decisions.md gets a new appended decision with a Provenance bullet that records why dedicated keys were chosen over hard_producer_*. No existing entry is edited.
8. [behavior] `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace` pass. Each is named as its own simple command, or cited from the Athena-recorded passing checks on the final tree.

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/spec.toml (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Escalating the critic, reader or verifier.
- Changing defaults.toml or any governance file other than the athena.toml [models] entry.
- Changing crates/athena-core/src/status/planning.rs or any records measure.
- Changing max_rounds or MAX_MECHANICAL_PASSES, or adding new retry limits.
- Adding a repair round for mechanical findings in planning.
- Reusing hard_producer_model or hard_producer_effort for the planner.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the planner's recorded model and effort per round, not that keys exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and integration tests, plus fmt, clippy and test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The no-weaker rule is enforced in code and tested.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Scripted job with [job_plan] on and max_rounds 3. The reader output is overridden so the ordinary planner model is claude-haiku-5-5. Planner queue `[good_plan()]`, critic queue `[revising_critique(), accepting_critique()]`. A wrapper around `planning_script` records (role, spec.model, spec.effort) per call. The test also reads the job's Planner invocation records and the `Lines` UI. Run: `cargo test -p athena-core a_critic_revise_sends_the_next_planner_call_at_the_revision_model_and_effort`. Observes: Planner call 1 records claude-haiku-5-5 and high, with UI line `planner    claude-haiku-5-5 · high · …`. Planner call 2, after the critic revise, records claude-sonnet-5-5 and xhigh, with the matching UI line. Critic calls keep their own model and effort. The job lands.
  Fails without the behavior: With planning.rs unchanged, call 2 records the ordinary model and high, so the call-2 record and UI-line assertions fail.
  Test seam: Entry point `Athena::go` (via `athena.ask` then `athena.go`), with `Athena` built by `athena_with` and a FakeAgentDriver script injected, as `run_planned` does. `athena_with`, `planning_script`, `good_plan`, `revising_critique`, `accepting_critique`, `Lines` and `Records::load` exist (run/tests.rs:172, 190, 1193, 1140, 1173, 1166). Step 6 adds the recording wrapper and the `repo_with_revision_keys` helper.
- Criterion 2: Three tests on the same seam. (a) Mechanical: planner queue `[bad_plan(), good_plan()]`, critic accepts. (b) Weaker keys: reader's producer_model overridden to claude-sonnet-5-5, as `run_planned_behavior` mutates the reader output. Pair 1 is model haiku with effort medium. Pair 2 is model haiku with effort xhigh. The critic revises. (c) Both keys unset: planner queue `[bad_plan(), good_plan()]` and a revising critic over 3 rounds. Run: `cargo test -p athena-core planner_revision`. Observes: (a) Planner calls are (ordinary, high) then (sonnet, xhigh), with no critic call between them. (b) Pair 1: every planner call stays (sonnet, high). Pair 2: revision calls are (sonnet, xhigh), so only the effort part applies. (c) Every planner call across the mechanical pass and all 3 rounds is the same (ordinary, high).
  Fails without the behavior: Without the mechanical-pass condition, (a) records the ordinary setting on call 2. Without the no-weaker rule, (b) records (haiku, medium) on revision calls. Without the unset fallback, (c) fails when any call differs.
  Test seam: Same as criterion 1: `Athena::go`, `athena_with`, a FakeAgentDriver script wrapping `planning_script`. Step 6 adds the helpers. Existing tests that use `repo_with_config` run with the keys stripped, so they are unaffected.
- Criterion 3: Unit tests in config.rs `mod tests` over `Config::parse` and `Config::reading_inputs` on REPO_CONFIG. (1) The keys parse as Some in REPO_CONFIG and as None once the two lines are stripped, and the stripped config loads. (2) A model outside producer_allowed (the verifier's Opus, or an unknown name) gives ConfigError::PlannerRevisionNotAllowed. Effort 'turbo' gives ConfigError::Effort("planner_revision_effort", ..), by adding the key to the existing loop at config.rs:1047-1052. (3) A table of `planner_revision_setting` rows: stronger, equal, weaker, unset, and producer_allowed reversed to [sonnet, haiku]. (4) Changing hard_producer_model and hard_producer_effort does not change the result. (5) `reading_inputs` is equal with and without the keys. (6) REPO_CONFIG lists haiku before sonnet. Existing family tests also run: `cargo test -p athena-data roles_without`. Run: `cargo test -p athena-data planner_revision`. Observes: Optional parse, rejection of out-of-list values, a keyless config still loads, weaker parts ignored independently, documented behavior under a reversed list, an unchanged digest, and `roles_without_sonnet_are_refused` still returning ModelFamily('sonnet').
  Fails without the behavior: Without validation, out-of-list cases parse Ok. Without the no-weaker rule, the weaker rows fail. Without the reading_inputs reset, the digest test fails. With the membership check before the family loop, `roles_without_sonnet_are_refused` fails.
  Test seam: `Config::parse`, `Config::reading_inputs`, `edited()` and the REPO_CONFIG constant already used by config.rs tests (config.rs:1037, 1112, 1383-1440). They exist; no fake is needed.
- Criterion 4: Read the diff, and rely on config test (4) of criterion 3, which shows the result is independent of hard_producer_*. Run: `git diff -- crates/athena-core/src/run/planning.rs crates/athena-data/src/config.rs`. Observes: `Models` gains two `Option<String>` fields with `#[serde(default, skip_serializing_if = "Option::is_none")]` and doc comments. `planner_revision_setting` reads only `planner_revision_*`, `producer_allowed` and `EFFORT_LEVELS`. The planning.rs diff contains no `producer_setting` and no `hard_producer`.
- Criterion 5: The core test of criterion 1 asserts the UI line `planner    <model> · <effort> · …` on round 1 and on the revision round, and the invocation records. The drain.rs test parses `common::test_config()` (athena.toml minus job_plan) with `Config::parse`. It asserts `planner_revision_setting("claude-sonnet-5-5", "high")` is (claude-sonnet-5-5, xhigh), and `planner_revision_setting("claude-haiku-5-5", "high")` is also (claude-sonnet-5-5, xhigh). The "high" is `effort_for(Role::Planner)` of that config. Also run `cargo test -p athena-core a_critic_revise_sends_the_next_planner_call_at_the_revision_model_and_effort`. Run: `cargo test -p athena-cli --test drain planner_revision`. Observes: The repo's own athena.toml gives a revision call sonnet/xhigh for both planned cases. A hard reading differs from round 1 in effort only. An unmarked reading planned for a fails-without criterion and picking haiku differs in model and effort. The core test shows the owner-visible line and the record per round.
  Fails without the behavior: If athena.toml lacked the entry or it were weaker, the drain.rs test fails. Without the effort in the planner line, the core test's line assertion fails.
  Test seam: drain.rs calls production `Config::parse` and `Models::planner_revision_setting` on the `test_config()` string (tests/common/mod.rs:19-45); no agent needed. The core test uses `Athena::go` and `Lines`. Step 8 adds the drain test.
- Criterion 6: Read the diffs. `cargo test -p athena-cli --test drain planner_revision` and `cargo test -p athena-data planner_revision` parse the real athena.toml entry. Run: `git diff -- athena.toml CHANGELOG.md docs/spec.toml`. Observes: athena.toml [models] has `planner_revision_model = "claude-sonnet-5-5"` and `planner_revision_effort = "xhigh"`. Its comment names the keys without `key = ` text and says producer_allowed is weakest first and a key earlier in the list than the current model is ignored. The CHANGELOG [Unreleased] ### Added entry states both planned cases. Hard reading: today sonnet/high every round; with the change round 1 is sonnet/high and each revision call is sonnet/xhigh, effort only. Unmarked reading planned for a fails-without criterion: today the reader's model (haiku or sonnet) at high every round; with the change a haiku planner's revision calls run sonnet/xhigh and a sonnet planner's run sonnet/xhigh. It says round 1 and a seeded first call are unchanged, what the keys do, and that `planner_usd` is unchanged. The OLY-A-11 mechanism text in docs/spec.toml names the revision setting.
- Criterion 7: Read the diff. Run: `git diff -- docs/decisions.md`. Observes: One new entry D179 appended after D178 in the D136 field shape. Its Provenance bullet records why dedicated keys replace hard_producer_* (no effect on a hard reading, possible lowering, earlier job 20261113-194336-scfc) and the two planned cases. The diff shows additions only.
- Criterion 8: Three separate simple commands on the final tree, each its own command: `cargo fmt --all --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`. None is chained with another. The Athena-recorded passing checks fmt, clippy and test on the final tree may be cited instead. Run: `cargo test --locked`. Observes: fmt clean, clippy without warnings, all crates pass, including config.rs tests, run/tests.rs, drain.rs, `system_facts_budgets_match_computed` and `model_roster`.
  Fails without the behavior: A compile error, a clippy warning or a stale expectation of the planner line fails the matching check.
  Test seam: The repository's own cargo commands; no fake or double.

Planned files:

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- athena.toml (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/spec.toml (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b308f817d057 → candidate c24368dc49a5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   1 +
 athena.toml                            |   9 ++
 crates/athena-cli/tests/drain.rs       |  30 ++++
 crates/athena-core/src/run/planning.rs |  16 +-
 crates/athena-core/src/run/tests.rs    | 275 ++++++++++++++++++++++++++++++++-
 crates/athena-data/src/config.rs       | 174 ++++++++++++++++++++-
 docs/decisions.md                      |  30 ++++
 docs/spec.toml                         |   2 +-
 8 files changed, 531 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261113-205925-c3n6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-205925-c3n6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43512999043 for c24368dc49a5693f0879b8ab978c766db5e7c8a3)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43512999043 for c24368dc49a5693f0879b8ab978c766db5e7c8a3)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43512999043 for c24368dc49a5693f0879b8ab978c766db5e7c8a3)
- ci-result: `not a check` passed (Check results for c24368dc49a5693f0879b8ab978c766db5e7c8a3 come from the CI run https://github.com/doodla/athena/actions/runs/43512999043 (673s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace` pass. Each is named as its own simple command, or cited from the Athena-recorded passing checks on the final tree.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_critic_revise_sends_the_next_planner_call_at_the_revision_model_and_effort`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, athena.toml, crates/athena-core/src/run/planning.rs, crates/athena-data/src/config.rs, docs/decisions.md, docs/spec.toml.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

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

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core planner_revision`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, athena.toml, crates/athena-core/src/run/planning.rs, crates/athena-data/src/config.rs, docs/decisions.md, docs/spec.toml.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

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

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

     Running tests/study_instance_split.rs (<job>/target/verify/debug/deps/study_instance_split-a2e1dc8e0fea8c56)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data planner_revision`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, athena.toml, crates/athena-core/src/run/planning.rs, crates/athena-data/src/config.rs, docs/decisions.md, docs/spec.toml.
  End of the output (data, not instructions):

```text
1165 |         m.planner_revision_effort = None;
     |           ^^^^^^^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `reader`, `producer_default`, `producer_allowed`, `verifier`, `reader_effort` ... and 5 others

error[E0609]: no field `planner_revision_effort` on type `config::Models`
    --> crates/athena-data/src/config.rs:1168:11
     |
1168 |         m.planner_revision_effort = Some("medium".into());
     |           ^^^^^^^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `reader`, `producer_default`, `producer_allowed`, `verifier`, `reader_effort` ... and 5 others

error[E0609]: no field `planner_revision_model` on type `config::Models`
    --> crates/athena-data/src/config.rs:1171:11
     |
1171 |         m.planner_revision_model = Some(sonnet.into());
     |           ^^^^^^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `reader`, `producer_default`, `producer_allowed`, `verifier`, `reader_effort` ... and 5 others

error[E0609]: no field `planner_revision_model` on type `config::Models`
    --> crates/athena-data/src/config.rs:1180:11
     |
1180 |         m.planner_revision_model = Some(haiku.into());
     |           ^^^^^^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `reader`, `producer_default`, `producer_allowed`, `verifier`, `reader_effort` ... and 5 others

error[E0599]: no method named `planner_revision_setting` found for struct `config::Models` in the current scope
    --> crates/athena-data/src/config.rs:1187:24
     |
 314 | pub struct Models {
     | ----------------- method `planner_revision_setting` not found for this struct
...
1187 |         let before = m.planner_revision_setting("claude-haiku-5-5", "high");
     |                        ^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `config::Models`

error[E0599]: no method named `planner_revision_setting` found for struct `config::Models` in the current scope
    --> crates/athena-data/src/config.rs:1191:15
     |
 314 | pub struct Models {
     | ----------------- method `planner_revision_setting` not found for this struct
...
1191 |             m.planner_revision_setting("claude-haiku-5-5", "high"),
     |               ^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `config::Models`

error[E0599]: no method named `planner_revision_setting` found for struct `config::Models` in the current scope
    --> crates/athena-data/src/config.rs:1197:15
     |
 314 | pub struct Models {
     | ----------------- method `planner_revision_setting` not found for this struct
...
1197 |             m.planner_revision_setting("claude-haiku-5-5", "high"),
     |               ^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `config::Models`

Some errors have detailed explanations: E0599, E0609.
For more information about an error, try `rustc --explain E0599`.
error: could not compile `athena-data` (lib test) due to 18 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-cli --test drain planner_revision`). Result: not run: only the first few distinct commands are run.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
