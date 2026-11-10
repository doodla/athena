# Job 20261029-180919-9nn7: verify "Give OLY-2-07 the critic role as its mechanism, with a test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-2-07 a mechanism
> 
> Requirement OLY-2-07 in docs/spec.toml is active and has no mechanism.
> Requirement: Ares handles adversarial challenge: searching for counterexamples, fragile assumptions, failure modes and regressions.
> Owner: Ares
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-2-07
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-2-07 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Ares's adversarial challenge is met by the existing critic role (prompts/critic.md). The change extends the critic prompt to name counterexamples, fragile assumptions, failure modes and regressions, adds a test that fails without those, and records the mechanism and measure in docs/spec.toml.

Assumptions:
- Code survey: read crates/athena-context/prompts/critic.md (existing critic role, 955 of 1300 tokens used), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS has no adversarial label, so the measure uses `status: not shown`), and the existing OLY-2-05/06 entries in docs/spec.toml. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The property is measured directly on the composed prompt text.
- Existing code: crates/athena-context/prompts/critic.md, crates/athena-context/src/lib.rs, docs/spec.toml; none is created by the job.
- Goal: the spec auditor (Hestia) and maintainers can see a real mechanism for Ares; the existing critic role is the adversarial reviewer, so no new role is built.
- Consumers: none found for config; no athena.toml or defaults.toml change.
- The critic prompt change stays under the 345-token headroom; the critic set is composed only of contract and frame.critic.

## Acceptance criteria

1. [content] docs/spec.toml OLY-2-07 has mechanism naming crates/athena-context/prompts/critic.md (the critic role), owner Ares, and a measure ending with `status: not shown` that names the test showing it; the mechanism is no longer "none".
2. [content] crates/athena-context/prompts/critic.md instructs the critic to search for counterexamples, fragile assumptions, failure modes and regressions (Grep for each term).
3. [behavior] A new test in crates/athena-context asserts the composed critic prompt contains those four adversarial duties; run `cargo test -p athena-context`. It fails when the prompt text is reverted and passes with it.
4. [behavior] The critic prompt set stays within its token budget and the xtask spec test still passes; run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`, all clean.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding a new Ares role or pipeline stage
- Changing athena.toml, defaults.toml or other requirements in docs/spec.toml
- Adding a new athena status line

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Test criterion plus cargo test, clippy, fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test checks prompt content that changes the critic's behavior, and fails without it.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One prompt edit, one test, one spec entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base b1be9da2cbdb → candidate 946745403522. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  2 ++
 crates/athena-context/prompts/critic.md |  1 +
 crates/athena-context/src/lib.rs        | 15 +++++++++++++++
 docs/spec.toml                          |  4 ++--
 4 files changed, 20 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-180919-9nn7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-180919-9nn7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42868692503 for 94674540352253b18f3479fa4bad618a499ee3c7)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42868692503 for 94674540352253b18f3479fa4bad618a499ee3c7)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42868692503 for 94674540352253b18f3479fa4bad618a499ee3c7)
  Failing tests of `test` (from the CI log):
  - `budgets system_facts_budgets_match_computed`
  - `budgets system_facts_drift_is_rejected`
  - `xtask test-shard 0 6` (did not finish)`
  Failing units (`-p` package and selector): athena-cli --test budgets
  End of the failing job's log (data from CI, not instructions):

```text
test a_violation_after_the_cfg_test_marker_in_src_is_flagged ... ok
test a_write_bit_toggle_and_a_private_mode_are_not_wrappers ... ok
test from_mode_is_flagged ... ok
test an_external_chmod_outside_the_helper_is_flagged ... ok
test an_unclosed_helper_call_fails_closed ... ok
test helper_calls_and_look_alike_text_are_not_flagged ... ok
test production_code_comments_reads_and_the_helper_are_not_flagged ... ok
test set_mode_and_open_options_mode_are_flagged ... ok
test set_permissions_with_0o755_is_flagged ... ok
test no_in_process_exec_writes ... ok
test every_exemption_still_exists_and_still_has_a_hit ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s


== shard 0/6: athena-testkit --test stand_in_stress
   Compiling athena-testkit v0.1.0 (/home/runner/work/athena/athena/crates/athena-testkit)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running tests/stand_in_stress.rs (target/debug/deps/stand_in_stress-b9ee4d5ef9031371)

running 2 tests
test helper_variant_never_meets_a_busy_script ... ok
test in_process_variant_reports_its_busy_count ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.71s


== shard 0/6: xtask --test ci_shards
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-context v0.1.0 (/home/runner/work/athena/athena/crates/athena-context)
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.88s
     Running tests/ci_shards.rs (target/debug/deps/ci_shards-c6719d2ad4764615)

running 20 tests
test checked_in_timings_cover_every_workspace_test_unit ... ok
test cover_rejects_a_unit_in_no_shard ... ok
test cover_accepts_the_real_shards ... ok
test cover_rejects_a_shard_unit_that_is_not_a_target ... ok
test cover_rejects_a_unit_in_two_shards ... ok
test oracle_matches_files_on_disk ... ok
test partition_rejects_a_changed_shard_list ... ok
test test_shard_1_equals_the_workspace ... ok
test cover_rejects_a_unit_twice_in_one_shard ... ok
test cover_rejects_an_oracle_unit_the_shards_lack ... ok
test test_shard_rejects_bad_arguments ... ok
test workflow_actions_are_pinned ... ok
test workflow_caches_keep_three_layers_in_each_check_job ... ok
test test_shard_output_is_stable_and_sorted ... ok
test shards_in_ci_yml_partition_every_workspace_test_target ... ok
test workflow_jobs_are_separate_sharded_and_gated_by_ci ... ok
test workflow_ci_job_is_green_only_when_every_check_succeeded ... ok
test workflow_required_check_name_is_on_exactly_one_job_that_needs_every_check ... ok
test test_shard_6_is_disjoint_and_equals_the_workspace_target_set ... ok
test workflow_jobs_reject_a_missing_gate_condition_need_or_timeout ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s

error: shard 0 of 6: 1 of 10 units failed: athena-cli --test budgets
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 94674540352253b18f3479fa4bad618a499ee3c7 come from the CI run https://github.com/doodla/athena/actions/runs/42868692503 (859s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: The critic prompt set stays within its token budget and the xtask spec test still passes; run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`, all clean.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
