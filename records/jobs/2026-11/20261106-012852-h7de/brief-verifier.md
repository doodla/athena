# Job 20261106-012852-h7de: verify "Raise agent ceiling to 16, record group sizes, fix stale help"

## Request (verbatim)

> try again on 20261105-180339-w5h7

## Reading (accepted by the user)

Redo step 6 of docs/design/agent-and-build-slots.md section 8 from base (job 20261105-180339-w5h7 was rejected; its seven changed files are kept as deliverables and redone), and fix its major finding: `athena drain --help` still said "at most 8". Audience: the maintainer reading drain output and help to see what the ceiling allows.

Assumptions:
- Code survey: read crates/athena-data/src/config.rs (MAX_CONCURRENCY_CEILING at 269, validation at 722 and 745, tests near 1194 and 1458), crates/athena-cli/src/main.rs:275 (drain help text; help tests at 1808 and crates/athena-cli/tests/drain.rs:2054,2097), and grep found no group_sizes anywhere yet, so recording is new. No new record or data file under crates/ is added; the group sizes live in the in-memory drain Report, so AnyRecord and check_record_file are not touched. The group size is measured directly from the plan group's item count as the batch works it.
- Job 20261105-180339-w5h7: its seven changed files (CHANGELOG.md, athena.toml, display.rs, drain.rs, tests/drain.rs, config.rs, agent-and-build-slots.md) are kept as deliverables, but its changes were discarded so the work is redone from base. Its findings are addressed: major (main.rs:275 'at most 8') fixed by building the help text from MAX_CONCURRENCY_CEILING; minor stale '1 to 8' in parallel-jobs.md and async-drain.md fixed; minor untested display::drain call fixed by a test that goes through drain() output; nit about the Built note wording fixed (only max_concurrency says 'from 1 to 16').
- Job 20261105-180334-rj4j (slots, state, run checks) stays landed and untouched.
- Introspection 20261106-012825-ay9k prevention applied: help text derives the ceiling from the constant, a test checks it, and a repo grep for the old ceiling of 8 finds no stale mention outside CHANGELOG and unrelated uses (decisions.md:3255, hestia.rs:11 'rows 1 to 8', records-history.md:302 attempts are not the ceiling).
- The design's starting value for max_concurrency is 12 (section 7.2), as read in the earlier accepted reading.
- Step 6 touches crates/athena-core/src/plan.rs only if group item counts need a helper; it may stay unchanged.

## Acceptance criteria

1. [content] crates/athena-data/src/config.rs declares MAX_CONCURRENCY_CEILING as 16; its doc comments and the athena.toml [parallel] comment say 1 to 16, and athena.toml max_concurrency is 12 (athena.toml is a deliverable for this reason).
2. [content] crates/athena-cli/src/main.rs drain help text no longer hard-codes 8: it is built from MAX_CONCURRENCY_CEILING (or states 16), and Grep for 'at most 8' and 'allows 1 to 8' / 'max_concurrency 1 to 8' in crates/, athena.toml and docs/design finds none; async-drain.md and parallel-jobs.md say 1 to 16 (or note the later change).
3. [behavior] Run `cargo test -p athena-cli`: a test renders `drain --help` and asserts it contains the current MAX_CONCURRENCY_CEILING value formatted from the constant, so help and ceiling cannot disagree.
4. [behavior] Run `cargo test -p athena-data`: tests show max_concurrency = 16 and max_in_flight = 16 validate and 17 is refused (max_concurrency with the 'from 1 to 16' error).
5. [behavior] Run `cargo test -p athena-cli --test drain`: a drain of a plan with groups of different sizes asserts the Report's group-size record lists the size of each group worked, in order, and that groups not reached are not recorded.
6. [behavior] Run `cargo test -p athena-cli`: a test calls display::drain with a report and asserts the end-of-run output contains the exact group-sizes line with the mean, and prints no such line when no group was worked.
7. [content] docs/design/agent-and-build-slots.md step 6 has a short Built note stating the ceiling 16, starting value 12, what is recorded, and that peak in-flight and the low-disk count are not measured; it does not claim max_in_flight 17 gives the 'from 1 to 16' text, and lines respect the doc wrap width.
8. [content] CHANGELOG.md has an entry for the ceiling and group-size record, and an entry under Fixed for the drain help text now following the ceiling constant.
9. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass with no new warnings.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)
- docs/design/async-drain.md (modify)
- docs/design/parallel-jobs.md (modify)
- crates/athena-core/src/plan.rs (modify)

## Out of scope

- Disk.rs per-slot build time T, the cargo-internal-error slot delete rule and the cleaning env nit from job 20261105-180334-rj4j's findings
- Files changed by job 20261105-180334-rj4j (slots.rs, state.rs, run/checks.rs, run/mod.rs, run/tests.rs, athena-agent/src/env.rs): kept as landed
- Changing defaults.toml or [drain.pipeline] values
- Landing rebuild slot use (step 7) and producer cargo wrapper (step 4)
- Measuring peak in-flight jobs or low-disk counts from real drains
- Changing records/ or adding a record type
- Unrelated 'rows 1 to 8' and 'at most 8 attempts' mentions that are not the agent ceiling

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the ceiling, group sizes, display and help; fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Help text derives from the constant rather than editing the number.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that check help, refusal, recording and display output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check what the maintainer sees in help and drain output.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused step-6 change plus the stale-mention fix.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces that help and the ceiling agree.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base cd236536a2fc → candidate 888ddc33968d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  9 +++++++
 athena.toml                          |  4 +--
 crates/athena-cli/src/display.rs     | 49 ++++++++++++++++++++++++++++++++++++
 crates/athena-cli/src/drain.rs       | 11 ++++++++
 crates/athena-cli/src/main.rs        | 32 +++++++++++++++++++++--
 crates/athena-cli/tests/drain.rs     | 41 ++++++++++++++++++++++++++++++
 crates/athena-data/src/config.rs     | 29 ++++++++++++++++++++-
 docs/design/agent-and-build-slots.md |  7 ++++++
 docs/design/async-drain.md           |  6 ++---
 docs/design/parallel-jobs.md         |  2 +-
 10 files changed, 181 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-012852-h7de/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-012852-h7de/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42146629240 for 888ddc33968dc02e3141aab438a997cd5e5e16e8)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42146629240 for 888ddc33968dc02e3141aab438a997cd5e5e16e8)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42146629240 for 888ddc33968dc02e3141aab438a997cd5e5e16e8)
  Failing tests of `test` (from the CI log):
  - `athena_cli drain::tests::status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain`
  - `xtask test-shard 1 6` (did not finish)`
  End of the failing job's log (data from CI, not instructions):

```text
test site::tests::the_real_study_shows_failures_and_limits_and_no_placeholder_guard ... ok
test site::tests::the_t10_card_explains_why_a_merge_stays_ready_and_cites_claims ... ok
test tests::sccache_install_skips_when_present_and_fails_clearly ... ok
test tests::setup_refuses_inside_a_job ... ok
test tests::skip_only_with_ci_matching_stamp_and_binary ... ok
test tests::the_built_binary_is_found_wherever_cargo_put_it ... ok
test tests::repin_refuses_an_empty_or_unparseable_version_and_keeps_the_file ... ok
test walltime::tests::median_and_p90_use_middle_and_nearest_rank ... ok
test site::tests::claims_are_listed_in_numeric_order_and_no_claim_repeats_a_source ... ok
test tests::repin_sets_only_the_version_and_is_idempotent ... ok
test tests::the_inputs_digest_changes_when_a_compiled_in_file_is_edited ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s


== shard 1/6: xtask --test policy
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.49s
     Running tests/policy.rs (target/debug/deps/policy-f6c712cfa0f68d15)

running 17 tests
test a_stray_script_is_flagged ... ok
test an_undocumented_edge_is_flagged ... ok
test athena_bin_keys_cover_every_compiled_in_file ... ok
test dependency_direction_matches_the_architecture ... ok
test every_action_is_pinned_by_a_full_commit_sha ... ok
test every_job_restores_the_tool_caches_before_it_runs_the_setup_step ... ok
test a_stale_or_incomplete_ci_yml_is_flagged ... ok
test a_missing_test_name_is_flagged ... ok
test the_admission_scan_finds_what_the_policy_scan_finds ... ok
test every_test_named_in_the_enforcement_map_exists ... ok
test the_checks_job_looks_up_its_earlier_pass_before_setup_and_skips_on_a_hit ... ok
test the_pass_is_marked_and_saved_only_after_the_checks_and_the_shared_caches_stay_default_branch_only ... ok
test the_producer_brief_lists_exactly_the_scanners_terms ... ok
test every_tracked_file_is_rust_or_data_or_an_allowed_exception ... ok
test the_boundary_scanner_flags_a_planted_term ... ok
test the_tool_cache_check_fails_when_a_restore_is_missing_or_after_setup ... ok
test athena_refers_to_no_one_who_runs_it_or_where ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.29s

error: shard 1 of 6: 1 of 10 units failed: athena-cli --lib
##[error]Process completed with exit code 1.
Post job cleanup.
[command]/usr/bin/git version
git version 2.56.0
Temporarily overriding HOME='/home/runner/work/_temp/87162ab9-dc8e-4a7f-ab9b-43e363b13ca6' before making global git config changes
Adding repository directory to the temporary git global config as a safe directory
[command]/usr/bin/git config --global --add safe.directory /home/runner/work/athena/athena
Removing SSH command configuration
[command]/usr/bin/git config --local --name-only --get-regexp core\.sshCommand
[command]/usr/bin/git submodule foreach --recursive sh -c "git config --local --name-only --get-regexp 'core\.sshCommand' && git config --local --unset-all 'core.sshCommand' || :"
Removing HTTP extra header
[command]/usr/bin/git config --local --name-only --get-regexp http\.https\:\/\/github\.com\/\.extraheader
[command]/usr/bin/git submodule foreach --recursive sh -c "git config --local --name-only --get-regexp 'http\.https\:\/\/github\.com\/\.extraheader' && git config --local --unset-all 'http.https://github.com/.extraheader' || :"
Removing includeIf entries pointing to credentials config files
[command]/usr/bin/git config --local --name-only --get-regexp ^includeIf\.gitdir:
[command]/usr/bin/git submodule foreach --recursive git config --local --show-origin --name-only --get-regexp remote.origin.url
Cleaning up orphan processes
##[warning]Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684. For more information see: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/
```
- ci-result: `not a check` passed (Check results for 888ddc33968dc02e3141aab438a997cd5e5e16e8 come from the CI run https://github.com/doodla/athena/actions/runs/42146629240 (552s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass with no new warnings.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
