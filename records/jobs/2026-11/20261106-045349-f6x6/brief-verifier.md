# Job 20261106-045349-f6x6: verify "Add plan/critic convention for content mirroring a governing source"

## Request (verbatim)

> Priority: high
> 
> Add a plan and critic convention: when a job writes content that summarises or mirrors a governing source (definitions taken from briefs or docs, cache keys taken from build inputs, seeded records), the plan must name that source and list every place in it that the content depends on. That means every mention of a term, or every compiled-in file for a cache key. At least one acceptance criterion must compare the written content against that list, not just check its structure. The critic must reject a plan that leaves this out. Test it with a plan fixture that is missing the list and is rejected. Add a CHANGELOG entry.
> 
> Introspections that record the repeated cause: 20261017-182254-my85, 20261025-230247-cm9r, 20261102-102715-564y, 20261106-030930-hb84
> 
> Acceptance:
> - The change is made as described: Content a job writes from a partial view of its governing source (a term's senses, a cache key's inputs) is caught only at the final verdict; 3 jobs in class f62n
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261017-182305-f62n.json

## Reading (accepted by the user)

Planner prompt requires a plan that writes content summarising or mirroring a governing source (term definitions, cache keys, seeded records) to name the source and list every dependent place (each term mention, each compiled-in file). The critic prompt rejects plans without it. A fixture plan missing the list is shown rejected, and CHANGELOG gets an entry. Audience: Athena's planner and critic stages, and maintainers judging failure class f62n.

Assumptions:
- Code survey: read crates/athena-context/prompts/planner.md and critic.md (plain prompts, 966 and 902 of 1300 tokens, so about 334 and 398 left), crates/athena-core/src/admission.rs (dependents_findings, the shape check run on plans at line ~310, with unit tests) and crates/athena-core/tests/refusal_corpus.rs. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched.
- Reading: a prompt convention plus a test. Planner prompt gains a rule (name the source, list every dependent place, and require one criterion comparing written content to that list). Critic prompt gains a rule to raise a major finding (verdict revise or reject) when it is missing. The fixture test is an admission-level plan fixture without the list, asserted to yield a major finding. If a prompt-only critic rule cannot be tested deterministically, the check is an admission.rs shape rule, which is the assumption I take.
- The existing `dependents` section and its shape check are the model; the new list is a separate plan section or an extension of it, the producer's design choice, kept small.
- Failure-class record records/failure-classes/2026-10/20261017-182305-f62n.json is a reference and is not modified; records/ never changes.
- athena.toml and defaults.toml need no change; the planner and critic prompt token budgets must not be exceeded.

## Acceptance criteria

1. [content] crates/athena-context/prompts/planner.md states that a job writing content that summarises or mirrors a governing source must name the source and list every place in it the content depends on (every mention of a term, every compiled-in file for a cache key), and must have at least one criterion comparing the written content to that list, not just its structure.
2. [content] crates/athena-context/prompts/critic.md instructs the critic to reject (major finding) a plan that writes such mirrored content without the named source and complete list, or without a comparing criterion.
3. [behavior] A test with a plan fixture that lacks the source list is rejected (a major finding or refusal naming the missing list), while the same fixture with the list is accepted; check with `cargo test`.
4. [content] CHANGELOG.md has an entry for the convention, citing failure class f62n.
5. [content] Prompt sets stay within their token budgets (planner 1300, critic 1300), shown by the existing prompt budget test in `cargo test`.
6. [behavior] No unrelated behavior changes: `cargo test` passes including existing tests, `cargo clippy` is clean and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- docs/design/job-planning.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)

## Out of scope

- Editing records/ including the f62n failure-class record or the four introspections
- Changing athena.toml or defaults.toml
- Changes to reader, producer or verifier prompts
- Retroactively auditing past jobs' plans

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture test for the rejection; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: fmt, clippy and test criteria run by the verifier.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced by a tested plan check where possible, with the critic prompt as backup.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion checks a plan missing the list is rejected.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Prompt edits plus one fixture test and a CHANGELOG entry.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Prompt and changelog wording is plain and self-contained.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of planner.md. The producer's final wording must match the bullet checked here, which is: `- mirrored_sources: content mirroring a governing source: name it and list every place it depends on (each term mention, each compiled-in cache-key file); a demonstration compares content to each, not only structure.` (final wording may differ slightly but must keep all of: the source named, every dependent place listed with the term-mention and compiled-in-file examples, a demonstration comparing content to each place, 'not only structure'/'not only its structure'). Also check `cargo test --locked -p athena-context frames` passes, so the file is at most 2097 bytes. Run: `git diff -- crates/athena-context/prompts/planner.md`. Observes: A new bullet in planner.md that states: content that mirrors a governing source needs the source named and every place it depends on listed (each mention of a term, each compiled-in file for a cache key); at least one demonstration compares the written content to those places, not only its structure. The lines trimmed to make room still contain the protected phrases ('seam, fake or fixture', 'Never put the logic under test inside the double', 'refusal_corpus', 'calling an external service (GitHub API, CI run, deploy)', 'must name the state it needs', 'how the plan checks or waits for it'). The bullet is also repeated in the `PlanMirrorOutput` field docs in outputs.rs, which the planner sees as schema descriptions.
- Criterion 2: Read the diff of critic.md. The bullet reads about: `- If the plan writes content mirroring a governing source (term definitions, cache keys, seeded records), Grep that source. Reject it (major finding) when mirrored_sources is empty, omits a place you find, or the comparing criterion's demonstration checks only structure, not the content against each place. Admission checks only shape.` (at most about 340 bytes; critic.md stays at or below 2097 bytes). Run: `git diff -- crates/athena-context/prompts/critic.md`. Observes: A new critic bullet that tells the critic to Grep the source and raise a major finding for a plan that mirrors content with no named source or complete place list, or whose comparing demonstration checks only structure. It marks completeness as the critic's own check. The existing phrases 'major finding when a demonstration names no seam' and 'In round one, raise a major finding' are intact.
- Criterion 3: Run the tests whose names contain `mirror`: three layers, each pairing the incomplete plan with the same plan with its places listed. The filter `mirror` matches `mirrored_source...` tests (admission.rs, records.rs, outputs.rs), `mirror_cases_get_the_expected_outcome` (refusal_corpus.rs) and `athena_sends_a_plan_that_mirrors_a_source_without_places_back_to_the_planner` (job_plan.rs). Refusal-rule input forms and the corpus case that covers each (all added by step 8, each with its source citing f62n or one of the four introspections): entry with a source and an empty places list -> `mirrored-source-no-places` (Refuse, 'lists no places'); places that are all blank -> `mirrored-source-blank-place` (Refuse, 'lists no places'); blank source -> `mirrored-source-blank-source` (Refuse, 'blank source'); comparison 0 -> `mirrored-source-comparison-zero` (Refuse, 'comparison'); comparison above the criteria count -> `mirrored-source-comparison-out-of-range` (Refuse, 'comparison'); every mention of a term listed -> `mirrored-source-complete-term-mentions` (Accept); every compiled-in cache-key file listed -> `mirrored-source-complete-cache-key-files` (Accept); no entries declared -> `mirrored-source-none-declared` (Accept). The existing corpus cases in `plan_cases` (refusal_corpus.rs:971-993) are unchanged and I read them. Corpus demonstration the producer and verifier run: `cargo test -p athena-core --test refusal_corpus` (I cannot run it and cite no result). Run: `cargo test --locked mirror`. Observes: Each incomplete plan (no places, only blank places, blank source, comparison 0, comparison above the criteria count) yields exactly one major Athena finding at location `mirrored sources, entry 1` whose summary says the entry lists no places or names the comparison fault. The scripted job (round 1 incomplete, round 2 with the places listed) makes 2 planner calls and 1 critic call, lands (`OutcomeState::Landed`), and the round-2 planner brief contains `lists no places`. The complete plans and the plan with no mirrored sources yield no finding. A comparison that points at a criterion with no demonstration yields only the existing 'needs exactly one' finding.
  Fails without the behavior: Without `mirrored_sources_findings`, the incomplete fixtures yield no finding, so these fail: the admission.rs unit test `a_mirrored_source_with_no_places_is_a_major_finding_naming_the_missing_list`; the corpus case `mirrored-source-no-places` (through `mirror_cases_get_the_expected_outcome`); and the job_plan.rs test `athena_sends_a_plan_that_mirrors_a_source_without_places_back_to_the_planner`. In the last one the round-1 plan would reach the critic, so the total critic calls would be 2, not the asserted 1 (none in round 1), and the round-2 planner brief would not contain `lists no places`. The complete-fixture and no-mirrored-sources cases pass either way, which shows the rule does not refuse good plans.
  Test seam: Entry points: `admission::plan_findings` (admission.rs:172) for the unit and corpus tests, and `Athena::go` with the scripted fake planner and critic for the end-to-end test. Fixtures and injection: admission.rs `plan_findings_for` (line ~870) becomes a wrapper over a new inner `plan_findings_with(dependents, open_questions, mirrored)` (step 7 adds it); the corpus `plan_for` (refusal_corpus.rs:~998) keeps its signature and delegates to a new `plan_with_mirrors(dependents, mirrors)`, used by a new `MirrorCase` table with its own `mirror_mismatches` (step 8; the existing `plan_mismatches` hard-codes a `dependents` location prefix, so it is not reused); the job_plan.rs test sets `scenario.plans = [incomplete, complete]` as the test at line ~856 does and uses `good_plan()` (line ~457) with a `mirrored_sources` entry whose `comparison` is 1, which `good_demos()` (line ~450) already demonstrates. The existing fixtures exist today (I read them). Steps 1 and 3 add the field and the rule; steps 7 to 9 add the tests. The plan JSON goes through the real `PlanOutput` parse and `plan_findings`; the double is only the scripted planner and critic text, not the rule.
- Criterion 4: Read the CHANGELOG diff. Run: `git diff -- CHANGELOG.md`. Observes: A new entry under Unreleased / Added describes the `mirrored_sources` plan section, the admission rule and the planner and critic prompt rules. It states the root cause in the `Root cause:` style of the neighbouring entries (content written from a partial view of its governing source was caught only at the final verdict) and cites failure class f62n and jobs 20261017-182254-my85, 20261025-230247-cm9r, 20261102-102715-564y and 20261106-030930-hb84. It states that admission checks only the shape and that completeness and structure-only comparisons are the critic's prose-only check, so the shape rule is not read as closing f62n.
- Criterion 5: Run the tests whose names contain `budget`. This includes the frame test `planner_and_critic_frames_are_short_and_fit_the_budgets` (crates/athena-context/src/lib.rs:394; its name contains 'budgets'), `every_role_set_is_within_its_budget` and `system_facts_budgets_match_computed` (crates/athena-cli/tests/budgets.rs:125). The producer copies the computed planner and critic sizes into the two `budget` lines of docs/system-facts.md and never changes a budget. Run: `cargo test --locked budget`. Observes: The planner and critic sets stay within 1300 tokens, each frame stays under 700 tokens (`estimate_tokens(frame.text) < 700`, lib.rs:404), and the `budget planner` and `budget critic` lines in docs/system-facts.md equal the computed values.
- Criterion 6: Run the full suite, then clippy and fmt as Athena does: `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo fmt --all --check`. The corpus demonstration is `cargo test -p athena-core --test refusal_corpus`. Run: `cargo test --locked`. Observes: All existing and new tests pass; clippy and fmt are clean. A plan with no mirrored content gets no new finding, renders in `plan_body` as before, and an old record re-serialises byte for byte.
  Fails without the behavior: If the new field changed how old plans serialise, the records.rs byte-for-byte round-trip test for an empty `mirrored_sources` (new in step 1) and the existing job_plan.rs and corpus accept cases would fail. If the rule refused complete or unrelated plans, the unchanged accept cases of the corpus `plan_cases` and the existing admission tests would fail.
  Test seam: Entry points: `AnyRecord::parse` and `to_json` in records.rs, with the existing old-record round-trip test `plan_dependents_are_optional_in_old_records_and_round_trip_when_set` (records.rs:~4471) as the model, and `plan_findings` through the existing corpus accept cases (refusal_corpus.rs:971-993). Both exist; step 1 adds the new round-trip test. No double is involved.

Planned files:

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)
- docs/system-facts.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base 25c0e5a2c332 → candidate 9c64dc206a64. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  22 ++++
 crates/athena-cli/tests/job_plan.rs        |  31 +++++
 crates/athena-context/prompts/critic.md    |   1 +
 crates/athena-context/prompts/planner.md   |   9 +-
 crates/athena-context/src/briefs.rs        |  51 ++++++++
 crates/athena-context/src/outputs.rs       |  82 +++++++++++-
 crates/athena-core/src/admission.rs        | 199 ++++++++++++++++++++++++++++-
 crates/athena-core/src/reading.rs          |   1 +
 crates/athena-core/src/run/mod.rs          |   2 +-
 crates/athena-core/src/run/planning.rs     |  10 ++
 crates/athena-core/src/run_pin_tests.rs    |   1 +
 crates/athena-core/src/status.rs           |   1 +
 crates/athena-core/src/store.rs            |   1 +
 crates/athena-core/tests/refusal_corpus.rs | 185 ++++++++++++++++++++++++++-
 crates/athena-data/src/records.rs          |  60 +++++++++
 docs/design/job-planning.md                |  21 ++-
 docs/system-facts.md                       |   4 +-
 17 files changed, 664 insertions(+), 17 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-045349-f6x6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-045349-f6x6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42184204731 for 9c64dc206a646f980b2d8c1cd0a9a52945cfaaa3)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42184204731 for 9c64dc206a646f980b2d8c1cd0a9a52945cfaaa3)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42184204731 for 9c64dc206a646f980b2d8c1cd0a9a52945cfaaa3)
  Failing tests of `test` (from the CI log):
  - `athena_cli drain::tests::status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain`
  - `xtask test-shard 1 6` (did not finish)`
  End of the failing job's log (data from CI, not instructions):

```text
test tests::input_files_follow_the_shared_list ... ok
test site::tests::the_t10_card_explains_why_a_merge_stays_ready_and_cites_claims ... ok
test site::tests::the_real_study_shows_failures_and_limits_and_no_placeholder_guard ... ok
test tests::sccache_install_skips_when_present_and_fails_clearly ... ok
test tests::setup_refuses_inside_a_job ... ok
test tests::skip_only_with_ci_matching_stamp_and_binary ... ok
test tests::the_built_binary_is_found_wherever_cargo_put_it ... ok
test walltime::tests::median_and_p90_use_middle_and_nearest_rank ... ok
test tests::repin_sets_only_the_version_and_is_idempotent ... ok
test tests::repin_refuses_an_empty_or_unparseable_version_and_keeps_the_file ... ok
test tests::the_inputs_digest_changes_when_a_compiled_in_file_is_edited ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


== shard 1/6: xtask --test policy
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.58s
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

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.73s

error: shard 1 of 6: 1 of 10 units failed: athena-cli --lib
##[error]Process completed with exit code 1.
Post job cleanup.
[command]/usr/bin/git version
git version 2.56.0
Temporarily overriding HOME='/home/runner/work/_temp/6fe97d3e-d94b-4b75-ad04-4bcb1fc2a0e2' before making global git config changes
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
- ci-result: `not a check` passed (Check results for 9c64dc206a646f980b2d8c1cd0a9a52945cfaaa3 come from the CI run https://github.com/doodla/athena/actions/runs/42184204731 (575s): fmt success, clippy success, test failure.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked mirror`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/critic.md, crates/athena-context/prompts/planner.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/outputs.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/design/job-planning.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   --> crates/athena-core/src/admission.rs:684:13
    |
684 |             mirrored_sources: vec![],
    |             ^^^^^^^^^^^^^^^^ `athena_data::records::JobPlan` does not have this field
    |
    = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::JobPlan` has no field named `mirrored_sources`
   --> crates/athena-core/src/admission.rs:808:13
    |
808 |             mirrored_sources: vec![],
    |             ^^^^^^^^^^^^^^^^ `athena_data::records::JobPlan` does not have this field
    |
    = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::JobPlan` has no field named `mirrored_sources`
   --> crates/athena-core/src/admission.rs:907:13
    |
907 |             mirrored_sources,
    |             ^^^^^^^^^^^^^^^^ `athena_data::records::JobPlan` does not have this field
    |
    = note: all struct fields are already assigned

   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0560]: struct `athena_data::records::JobPlan` has no field named `mirrored_sources`
    --> crates/athena-core/src/reading.rs:4525:13
     |
4525 |             mirrored_sources: vec![],
     |             ^^^^^^^^^^^^^^^^ `athena_data::records::JobPlan` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::JobPlan` has no field named `mirrored_sources`
    --> crates/athena-core/src/run_pin_tests.rs:1172:9
     |
1172 |         mirrored_sources: vec![],
     |         ^^^^^^^^^^^^^^^^ `athena_data::records::JobPlan` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::JobPlan` has no field named `mirrored_sources`
    --> crates/athena-core/src/status.rs:2514:13
     |
2514 |             mirrored_sources: vec![],
     |             ^^^^^^^^^^^^^^^^ `athena_data::records::JobPlan` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::JobPlan` has no field named `mirrored_sources`
   --> crates/athena-core/src/store.rs:615:13
    |
615 |             mirrored_sources: vec![],
    |             ^^^^^^^^^^^^^^^^ `athena_data::records::JobPlan` does not have this field
    |
    = note: all struct fields are already assigned

Some errors have detailed explanations: E0422, E0425, E0560.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 11 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/critic.md, crates/athena-context/prompts/planner.md, crates/athena-context/src/briefs.rs, crates/athena-context/src/outputs.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/design/job-planning.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0432]: unresolved import `athena_data::records::PlanMirror`
   --> crates/athena-core/tests/refusal_corpus.rs:895:24
    |
895 |         PlanDependent, PlanMirror, Role, Severity,
    |                        ^^^^^^^^^^ no `PlanMirror` in `records`

error[E0560]: struct `athena_data::records::JobPlan` has no field named `mirrored_sources`
    --> crates/athena-core/tests/refusal_corpus.rs:1014:13
     |
1014 |             mirrored_sources: mirrors,
     |             ^^^^^^^^^^^^^^^^ `athena_data::records::JobPlan` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0432, E0560.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `athena-core` (test "refusal_corpus") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
```

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
