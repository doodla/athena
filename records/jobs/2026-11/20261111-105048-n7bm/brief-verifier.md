# Job 20261111-105048-n7bm: verify "Add role-need measure to athena status for OLY-9-11 (retry)"

## Request (verbatim)

> try again on 20261029-233051-jq6k

## Reading (accepted by the user)

Retry of job 20261029-233051-jq6k: add a role-need measure in athena-core, printed by `athena status` as a `role need` line and recorded as OLY-9-11's mechanism and measure. It reads each role's runs from where they are recorded (job invocations plus steward_runs, audience_runs and auditor_runs), so steward, audience and auditor are never flagged for retirement. Audience: Athena owner and the Hestia spec pass.

Assumptions:
- Code survey: crates/athena-core/src/store.rs (Records at ~239-251 keeps steward_runs, audience_runs, auditor_runs apart from request_jobs invocations; built at ~421-456), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at ~305, checked at ~1597), crates/athena-core/src/hestia.rs (~1166, ~1230 validates that a measure ends with a known `status: <label>`), crates/athena-cli/src/display.rs (status rendering). The measure is computed directly from Records.
- Existing code: extend crates/athena-core/src/status.rs (StatusView, STATUS_MEASURE_LABELS) and crates/athena-cli/src/display.rs; follow the class_trend pattern of recent windows; reuse the Records accessors in crates/athena-core/src/store.rs without changing them.
- Job 20261029-233051-jq6k findings addressed: (blocker) retire rule blind to steward/audience/auditor: compute now reads runs from job invocations AND the three separate run lists; a role with no readable source is shown as `not measured`, never `retire`. (minor) compute untested: add tests over Records covering outcome mapping (Failed/Rejected/TimedOut count as failed; Declined/Closed excluded), ordering and role visibility. (minor) unbounded class history: role-needed uses only the newest N closed jobs per class. (nit) display test renders a StatusView through status(), not the section helper.
- Files from job 20261029-233051-jq6k's candidate: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/role_need.rs, crates/athena-core/src/status.rs and docs/spec.toml are all kept as deliverables and redone from the base with the fixes above.
- Term role need: per role, a status of needed (recent failure concentration in a class), in use, retire candidate (no runs in a recent window across all run sources) or not measured. Advisory only; no automatic role changes.
- No new record or data file is added under crates/, so the records validation allow-list and AnyRecord are unaffected. No prompts change, so the token budgets are unaffected. No governance files change.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-9-11 has mechanism naming an existing path crates/athena-core/src/role_need.rs, owner Athena, and a measure ending `status: role need`; the label `role need` is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
2. [behavior] A test over Records built with steward_runs, audience_runs and auditor_runs (and few or no job invocations for them) shows RoleNeed::compute does not flag steward, audience or auditor for retirement; replaying over a real-shape fixture lists them as not flagged. Run `cargo test -p athena-core role_need`.
3. [behavior] Tests in role_need.rs show a role truly unused in the recent window, with its source measured, is flagged retire; a role with no recorded source is shown `not measured`; a class with repeated recent failures yields role-needed; thin history yields nothing; outcome mapping (Failed/Rejected/TimedOut failed, Declined/Closed excluded) and only the recent window per class are honoured. Tests fail if compute/analyze is stubbed. Run `cargo test -p athena-core role_need`.
4. [behavior] A display test in crates/athena-cli/src/display.rs renders a StatusView through status() and shows a line starting `role need` for every role in ALL_ROLES (including not measured ones), and no candidate advice when none. Run `cargo test -p athena-cli`.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` (including the OLY spec test in xtask/tests/spec.rs) pass.
6. [content] The diff changes only docs/spec.toml, crates/athena-core/src/role_need.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs and CHANGELOG.md (entry added); athena.toml, defaults.toml and prompts are unchanged.

## Expected deliverables

- crates/athena-core/src/role_need.rs (create)
- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- OLY-9-12 (value of a new role against a simpler alternative)
- Automatically creating or retiring roles
- Changes to prompts, athena.toml or defaults.toml
- Changing how store.rs records steward, audience or auditor runs

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the replayed behavior (steward, audience, auditor not flagged), not file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests fail without the mechanism; fmt, clippy and test required.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix addresses the multi-source blindness, not just the three roles.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 41f24754a16a → candidate 20220bd429ad. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   5 +
 crates/athena-cli/src/display.rs    | 121 +++++++++
 crates/athena-core/src/lib.rs       |   1 +
 crates/athena-core/src/role_need.rs | 499 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs    |   5 +
 docs/spec.toml                      |   4 +-
 6 files changed, 633 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-105048-n7bm/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-105048-n7bm/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43008212228 for 20220bd429ada729ac59ccf82089e74fb914dcef)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43008212228 for 20220bd429ada729ac59ccf82089e74fb914dcef)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43008212228 for 20220bd429ada729ac59ccf82089e74fb914dcef)
  Failing tests of `test` (from the CI log):
  - `job_plan an_unplanned_jobs_status_text_and_json_gain_nothing`
  - `xtask test-shard 5 6` (did not finish)`
  Failing units (`-p` package and selector): athena-cli --test job_plan
  End of the failing job's log (data from CI, not instructions):

```text
== shard 5/6: athena-gate --lib
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-git v0.1.0 (/home/runner/work/athena/athena/crates/athena-git)
   Compiling athena-score v0.1.0 (/home/runner/work/athena/athena/crates/athena-score)
   Compiling athena-gate v0.1.0 (/home/runner/work/athena/athena/crates/athena-gate)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.45s
     Running unittests src/lib.rs (target/debug/deps/athena_gate-d4b52202855a4113)

running 141 tests
test attribution::tests::a_pending_commit_passes_within_fifteen_minutes_and_fails_after ... ok
test attribution::tests::a_missing_branch_no_intent_and_an_off_main_base_are_explicit_findings ... ok
test attribution::tests::a_landing_whose_intent_does_not_match_is_a_finding ... ok
test attribution::tests::a_properly_landed_squash_passes ... ok
test attribution::tests::a_records_commit_that_is_not_a_fast_forward_is_reported ... ok
test attribution::tests::a_trailer_does_not_attribute_a_commit ... ok
test attribution::tests::an_invalid_record_file_is_a_finding ... ok
test attribution::tests::a_later_landing_fast_forwards_past_the_remote_checkpoint ... ok
test attribution::tests::an_unrecorded_commit_fails_and_is_named ... ok
test attribution::tests::a_records_tip_the_remote_lacks_gets_no_checkpoint_and_nothing_is_lost ... ok
test attribution::tests::checkpoint_write_failure_keeps_verdict ... ok
test attribution::tests::checkpoint_reaches_the_remote_records_branch ... ok
test attribution::tests::checkpoint_lands_on_a_remote_that_moved_ahead_of_the_checked_tip ... ok
test attribution::tests::commits_before_the_first_intents_base_are_not_judged ... ok
test attribution::tests::diverged_local_and_tracking_refs_are_reported ... ok
test attribution::tests::check_version_change_forces_full_replay ... ok
test attribution::tests::failing_check_writes_no_checkpoint ... ok
test attribution::tests::full_replay_ignores_checkpoints_and_read_only_never_writes ... ok
test attribution::tests::intent_before_checkpoint_landing_after_passes ... ok
test attribution::tests::ci_incremental_run_from_the_remote_checkpoint_judges_only_later_commits ... ok
test attribution::tests::incremental_run_judges_only_new_commits ... ok
test attribution::tests::the_tracking_ref_alone_is_read ... ok
test attribution::tests::passing_check_writes_checkpoint ... ok
test content::tests::a_configuration_that_does_not_parse_is_refused ... ok
test content::tests::added_lines_with_private_email_addresses_are_refused ... ok
test attribution::tests::tampered_checkpoint_is_caught_by_full_replay ... ok
test content::tests::config_edits_without_schema_is_the_other_side_of_the_deferral ... ok
test content::tests::deleting_the_schema_does_not_defer_the_parse ... ok
test content::tests::a_new_section_with_its_schema_change_in_one_commit_is_not_refused ... ok
test content::tests::email_redaction_keeps_allowed_addresses ... ok
…


== shard 5/6: xtask --test site_page
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-context v0.1.0 (/home/runner/work/athena/athena/crates/athena-context)
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.48s
     Running tests/site_page.rs (target/debug/deps/site_page-df8c5508f9226333)

running 5 tests
test the_rendered_page_puts_glossary_entries_and_the_next_step_link_before_the_first_card ... ok
test the_rendered_page_has_the_newcomer_fixes ... ok
test the_wrong_fixture_page_fails_every_check ... ok
test the_rendered_page_shows_agent_decided_transitions_as_the_model_says ... ok
test the_rendered_page_shows_exactly_the_model_limit_findings ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

error: shard 5 of 6: 1 of 12 units failed: athena-cli --test job_plan
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 20220bd429ada729ac59ccf82089e74fb914dcef come from the CI run https://github.com/doodla/athena/actions/runs/43008212228 (916s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` (including the OLY spec test in xtask/tests/spec.rs) pass.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
