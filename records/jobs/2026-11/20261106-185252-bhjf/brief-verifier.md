# Job 20261106-185252-bhjf: verify "Retry 20261106-042815-ntc7: widen ontology scan to all governing docs"

## Request (verbatim)

> try again on 20261106-042815-ntc7

## Reading (accepted by the user)

Retry of stale job 20261106-042815-ntc7, which passed verification but could not land because the branch moved (drain max_concurrency change). Its verified candidate is kept and merged onto the tip; the audience is the verifier and the next producer.

Assumptions:
- Job 20261106-042815-ntc7 is addressed by keeping its whole candidate (CHANGELOG.md, crates/athena-cli/tests/ontology.rs, docs/decisions.md, docs/ontology.toml) and re-verifying on the merged tree. Its stale outcome (branch moved, dd76838c7c0f) needs only a clean merge; no content finding is outstanding.
- Its two nits (long lines in decisions.md/ontology.toml/ontology.rs; CHANGELOG entry under Added rather than Changed) are cosmetic and are optionally fixed if the producer touches those lines; they are not required.
- Code survey: crates/athena-cli/tests/ontology.rs is the only code read. Its error messages at lines ~257 and ~310 in the base still say 'is not BRIEF.md or one of the term's file sources'; the candidate changes them. The property (a sense found in a governing doc but missing from a term) is measured directly by the scratch-root unit test and the real-file test over docs/ontology.toml. No record or data file is added under crates/, so check_record_file and AnyRecord are not involved. No refusal rule for readings is added.
- Merge conflicts with the tip, if any, are in docs/decisions.md or CHANGELOG.md; the producer resolves them keeping both sides.

## Acceptance criteria

1. [content] docs/decisions.md D152 states the check scans BRIEF.md, README.md, every docs/*.md and the term's file sources, and no decision keeps the old narrower scope. Check with grep '^## D152'.
2. [content] D152 names the root cause (checks of shape and listed sources, not every governing source) and the tradeoff that docs/ subfolders are scanned only if a term lists them.
3. [content] Grep 'is not BRIEF.md or one of' in crates/athena-cli/tests/ontology.rs returns nothing; the messages name BRIEF.md, README.md, docs/*.md and the term's file sources.
4. [content] CHANGELOG.md has an entry naming the root cause of the scan-scope change, and the header comment in crates/athena-cli/tests/ontology.rs names it; prior CHANGELOG content and the new drain max_concurrency entries from the tip survive the merge.
5. [behavior] The scratch-root unit test flags a governing doc outside the term's listed sources that names a term in a sense its definition lacks. Run: cargo test -p athena-cli --test ontology.
6. [behavior] The real-file ontology test passes on docs/ontology.toml with no new excuses. Run cargo test -p athena-cli --test ontology, then cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/ontology.rs (modify)
- docs/ontology.toml (modify)

## Out of scope

- Redoing the y179 speed work, or changing secrets.rs, checks.rs or run.rs
- Changing ci.yml, xtask or setup.sh
- Adding ontology terms not caused by a misread
- Changes to athena.toml, defaults.toml or workflows
- Changing any study's concept model or files under studies/
- Recursive scanning of docs/ subfolders (recorded as a tradeoff)
- Building a code check that links decisions to the behavior they describe
- Reverting or touching the drain max_concurrency change on the tip

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and cargo test criterion included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run the ontology tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Criteria require the root cause named and fixed at the scan scope.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Re-verification on the merged tree.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 6f95b8151a31 → candidate 94cbe226a9d1. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  6 +++++
 crates/athena-cli/tests/ontology.rs | 46 +++++++++++++++++++++++++++++++++----
 crates/athena-core/src/mirrors.rs   | 41 +++++++++++++++++++++++++++------
 docs/decisions.md                   | 24 +++++++++++++++++++
 docs/ontology.toml                  |  4 ++--
 5 files changed, 108 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-185252-bhjf/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-185252-bhjf/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42275755295 for 94cbe226a9d1b6c38e92e644ea473522f932ec95)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42275755295 for 94cbe226a9d1b6c38e92e644ea473522f932ec95)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42275755295 for 94cbe226a9d1b6c38e92e644ea473522f932ec95)
  Failing tests of `test` (from the CI log):
  - `backtest the_real_log_has_no_condition_a_finding_against_the_confirmed_list`
  - `xtask test-shard 0 6` (did not finish)`
  End of the failing job's log (data from CI, not instructions):

```text
test cover_rejects_a_unit_in_no_shard ... ok
test cover_accepts_the_real_shards ... ok
test cover_rejects_a_unit_in_two_shards ... ok
test oracle_matches_files_on_disk ... ok
test partition_rejects_a_changed_shard_list ... ok
test test_shard_1_equals_the_workspace ... ok
test cover_rejects_an_oracle_unit_the_shards_lack ... ok
test cover_rejects_a_unit_twice_in_one_shard ... ok
test test_shard_rejects_bad_arguments ... ok
test workflow_actions_are_pinned ... ok
test workflow_caches_keep_three_layers_in_each_check_job ... ok
test shards_in_ci_yml_partition_every_workspace_test_target ... ok
test test_shard_output_is_stable_and_sorted ... ok
test workflow_jobs_are_separate_sharded_and_gated_by_ci ... ok
test workflow_ci_job_is_green_only_when_every_check_succeeded ... ok
test workflow_required_check_name_is_on_exactly_one_job_that_needs_every_check ... ok
test test_shard_6_is_disjoint_and_equals_the_workspace_target_set ... ok
test workflow_jobs_reject_a_missing_gate_condition_need_or_timeout ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s


== shard 0/6: xtask --test fingerprint
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running tests/fingerprint.rs (target/debug/deps/fingerprint-dc44ee0178a22a4a)

running 10 tests
test ci_cache_a_missing_entry_runs_the_full_job ... ok
test ci_cache_a_marker_for_another_fingerprint_does_not_skip ... ok
test ci_cache_a_pass_is_saved_and_the_same_fingerprint_then_skips ... ok
test ci_cache_bad_arguments_are_refused ... ok
test ci_cache_report_names_the_fingerprint_and_the_matched_key ... ok
test ci_cache_a_failed_run_writes_no_marker_and_the_next_run_is_full ... ok
test ci_cache_check_appends_its_answer_to_the_output_file ... ok
test fingerprint_outside_a_repository_fails ... ok
test fingerprint_changes_with_governed_content ... ok
test fingerprint_ignores_records_and_owner_paths ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

error: shard 0 of 6: 1 of 9 units failed: athena-cli --test backtest
##[error]Process completed with exit code 1.
Post job cleanup.
[command]/usr/bin/git version
git version 2.56.0
Temporarily overriding HOME='/home/runner/work/_temp/15cf0d84-d00b-4b06-bbae-9b5dce292e15' before making global git config changes
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
- ci-result: `not a check` passed (Check results for 94cbe226a9d1b6c38e92e644ea473522f932ec95 come from the CI run https://github.com/doodla/athena/actions/runs/42275755295 (481s): fmt success, clippy success, test failure.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "docs/decisions.md D152 states the check scans BRIEF.md, README.md, every docs/*.md and the term's file sources, and no decision keeps the old narrower scope. Check with grep '^## D152'.")

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
