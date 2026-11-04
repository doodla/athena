# Job 20261104-062715-yv4m: verify "Fix the firstmate study's Reopened state: not terminal, back to Queued"

## Request (verbatim)

> Fix the firstmate study's 'Reopened' state. The source check after each landing has refused the study page three times today on the same claim: firstmate-work-handling/page-text/model.json marks 'reopened' as a terminal-success state that ends the task, and intro.json says 'Done and Reopened both carry the tag "ends the task"'. The code does not end the item; it sends the backlog row back to Queued (bin/fm-teardown.sh around lines 3894-3896). Correct the model and the intro text so the source check passes. Acceptance: the source check passes on the study page for the reopened state.

## Reading (accepted by the user)

The firstmate study wrongly marks 'reopened' as terminal-success and says Done and Reopened both "end the task". Per the code, teardown reopens the backlog item and it returns to Queued. I'll correct model.json and intro.json (the audience is the reader of the study page and the post-landing source check) so the check passes.

Assumptions:
- The task's 'source check' is the study validation run after landing; I did not read its code, so the exact rule (state kind vs claim text) is unconfirmed.
- I did not read bin/fm-teardown.sh in the pinned source, so lines 3894-3896 are taken from the request. If the claims need a new cited source for the Queued return, claims.json changes too.
- The model may require non-terminal states to have an outgoing transition, so a Reopened->Queued transition may be needed in model.json.

## Acceptance criteria

1. [content] In studies/firstmate-work-handling/model.json the 'reopened' state no longer has kind 'terminal-success'; it is a non-terminal kind (e.g. waiting), or the item returns to 'queued', consistent with bin/fm-teardown.sh sending the backlog row back to Queued.
2. [content] studies/firstmate-work-handling/intro.json 'states' lead-in no longer says Done and Reopened both carry 'ends the task'. It says only Done ends the task, and that Reopened sends the item back to Queued for the captain's answer. Grep for 'ends the task' confirms.
3. [content] The 'reopened' glossary definition in intro.json, and any transition out of Reopened added to model.json, match the cited claims c23-c25 and the Queued return, with no new uncited claims.
4. [behavior] Run cargo test (the study/record validation tests over studies/) and cargo clippy: they pass with the edited study files. The source check on the study page for the reopened state passes, as shown in the recorded check output.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the firstmate source code or the pinned commit
- Other states or study mechanisms

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The final criterion requires the source check to pass on the reopened state.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes both the model and the intro text that repeat the wrong claim.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused edit to two study files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 3a9c859dd427 → candidate fb021021a44d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  7 +++++++
 studies/firstmate-work-handling/intro.json |  7 ++++---
 studies/firstmate-work-handling/model.json |  9 ++++++++-
 xtask/src/site.rs                          | 10 ++++++----
 4 files changed, 25 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41787732600 for fb021021a44d4ac86650aa43a7e15f589661e16d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41787732600 for fb021021a44d4ac86650aa43a7e15f589661e16d)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41787732600 for fb021021a44d4ac86650aa43a7e15f589661e16d)
  Failing tests of `test` (from the CI log):
  - `athena_cap_study rules::tests::a_guard_of_none_is_never_said_to_be_evaluated_by_code`
  - `xtask test-shard 1 6` (did not finish)`
  End of the failing job's log (data from CI, not instructions):

```text


== shard 1/6: athena-score --doc
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-score v0.1.0 (/home/runner/work/athena/athena/crates/athena-score)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.03s
   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


== shard 1/6: xtask --test policy
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-context v0.1.0 (/home/runner/work/athena/athena/crates/athena-context)
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.21s
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
test the_tool_cache_check_fails_when_a_restore_is_missing_or_after_setup ... ok
test the_boundary_scanner_flags_a_planted_term ... ok
test every_tracked_file_is_rust_or_data_or_an_allowed_exception ... ok
test athena_refers_to_no_one_who_runs_it_or_where ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.85s

error: shard 1 of 6: 1 of 10 units failed: athena-cap-study --lib
##[error]Process completed with exit code 1.
Post job cleanup.
[command]/usr/bin/git version
git version 2.56.0
Temporarily overriding HOME='/home/runner/work/_temp/e87379cf-0e30-478f-bc2c-be54c7888c94' before making global git config changes
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
- ci-result: `not a check` passed (Check results for fb021021a44d4ac86650aa43a7e15f589661e16d come from the CI run https://github.com/doodla/athena/actions/runs/41787732600 (518s): fmt failure, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo test (the study/record validation tests over studies/) and cargo clippy: they pass with the edited study files. The source check on the study page for the reopened state passes, as shown in the recorded check output.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
