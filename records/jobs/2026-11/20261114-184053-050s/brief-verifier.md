# Job 20261114-184053-050s: verify "Raise stage time limits to 24 hours in athena.toml; update pinning tests"

## Request (verbatim)

> From Sri (the owner), word for word: "Drop the time limits for now. Let's just measure how long things take and why"
> 
> Background (not Sri's own words; added so the request can be read on its own). This replaces request 20261114-160818-cd94, whose background was wrong: a hand change raising the limits was reverted because repository tests pin them (config.rs the_repository_config_plans_hard_jobs_only_with_the_designs_caps and the budget test near line 1020; run/tests.rs a_check_killed_at_its_deadline_closes_as_an_environment_stop). Six jobs between 13 and 14 Nov failed when the planner, at xhigh effort on revision rounds, hit the 600 s planner limit.
> What is wanted, in this order:
> 1. Drop the time limits: raise reader, planner, critic, producer and verifier time limits in athena.toml so they no longer stop work (for example 24 hours), keeping a limit on checks only as a guard against a hung test; update the tests that pin the old values.
> 2. Measure: for each stage of each job (reading, each planning round, producer, checks, verifier, landing, waiting in the queue, waiting for disk), record and show how long it took and the main reason (model and effort, tokens, tool calls, build and test time, retries, waits).
> If one job cannot do both, do 1 first and queue 2 as its own request.

## Reading (accepted by the user)

Job 20261114-160818-cd94 was closed with no candidate. This job does part 1 of the replacement request: set reader, planner, critic, producer and verifier time limits in athena.toml to 24 hours (86400 s), keep a 4-hour guard on checks, and update the tests that pin the old values. Part 2, measuring stage durations, is queued as its own request.

Assumptions:
- Goal: Sri (owner, judging the result) wants stages not to be stopped by time limits. Per the request's own fallback, this job does item 1 only; item 2 (measuring and showing per-stage time and reasons) is queued as its own request and is out of scope here.
- Code survey: crates/athena-data/src/config.rs holds the Budget and JobPlan fields (reader/producer/verifier/check_timeout_s, planner/critic_timeout_s) and tests pinning them at lines ~986-987, 1020-1026; the occurrences near 1394 and 1465 use inline test configs and likely need no change unless they parse the repo config. crates/athena-core/src/run/tests.rs contains inline configs with planner_timeout_s = 600 (lines 296, 1338, 8438) that are self-contained test fixtures and stay unchanged. No records or data files are added, so check_record_file and AnyRecord are not involved. No refusal rule is added or tightened.
- Existing code: the timeout mechanism in crates/athena-core/src/run and crates/athena-data/src/config.rs is kept as is; only values change. The check deadline path (a_check_killed_at_its_deadline_closes_as_an_environment_stop) uses its own short test deadline, so it should not need changes.
- Consumers: athena.toml is read by crates/athena-data/src/config.rs (REPO_CONFIG test constant and runtime loading). If config validation caps timeouts below 86400, the producer must raise that cap too and say so.
- Previous job 20261114-160818-cd94 recorded no candidate, no deliverables and no changed files; nothing is kept or dropped from it. Its one finding (closed, replaced by this request because its background was wrong) is addressed by this reading: the hand change did not persist, so athena.toml and the pinning tests are changed together here.
- Check limit set to 4 hours (14400 s) as the hung-test guard, as in the prior job's background; bash_default_timeout_ms and bash_max_timeout_ms (agent tool timeouts at lines 73-74) are left unchanged.

## Acceptance criteria

1. [content] athena.toml sets reader_timeout_s, producer_timeout_s, verifier_timeout_s, planner_timeout_s and critic_timeout_s to 86400, and check_timeout_s to 14400 (a hung-test guard only). Check with Grep on 'timeout_s' in athena.toml.
2. [content] The pinned values in crates/athena-data/src/config.rs (the_repository_config_plans_hard_jobs_only_with_the_designs_caps near line 986 and the_repository_budget_caps_cover_their_sum near lines 1020-1026) assert the new values; no assertion on the repository config still pins 600, 300, 5400 or 3600 for these limits.
3. [behavior] cargo test passes, including the config tests and run::tests::a_check_killed_at_its_deadline_closes_as_an_environment_stop, so the check deadline mechanism still works. Evidence: cargo test, or a recorded Athena test check on the final tree.
4. [behavior] cargo clippy and cargo fmt --check are clean (recorded Athena checks on the final tree).
5. [content] Scope: the change touches only the deliverables (athena.toml, config.rs tests, CHANGELOG.md, optionally run/tests.rs and a new appended docs/decisions.md entry with a Provenance bullet). No stage-duration measuring or reporting is added; that is queued separately as part 2 of the request, and the 14 Nov planner-timeout failures are addressed by the limits no longer stopping work.
6. [content] CHANGELOG.md has an entry stating the limits were raised to 24 h, checks to 4 h, and that measurement is follow-up work. Outcome for Sri: a planner at xhigh effort on revision rounds is no longer cut off at 600 s, as shown by planner_timeout_s = 86400 in athena.toml.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Measuring and displaying per-stage durations and reasons (request part 2, queued separately)
- Inline test fixtures with planner_timeout_s = 600 in crates/athena-core/src/run/tests.rs unless they read athena.toml
- bash_default_timeout_ms and bash_max_timeout_ms
- records/ and owner files
- Prompt files

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the limits actually read as 86400 and that tests pass, not just that files changed.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only item 1 is done; measurement is queued separately.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1359a652603f → candidate 8663f54dae27. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  4 ++++
 athena.toml                         | 13 +++++++------
 crates/athena-core/src/run/tests.rs | 11 ++++-------
 crates/athena-data/src/config.rs    | 13 ++++++++-----
 4 files changed, 23 insertions(+), 18 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-184053-050s/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-184053-050s/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43632417828 for 8663f54dae276e70f748291600258ea468c8c6a4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43632417828 for 8663f54dae276e70f748291600258ea468c8c6a4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43632417828 for 8663f54dae276e70f748291600258ea468c8c6a4)
- ci-result: `not a check` passed (Check results for 8663f54dae276e70f748291600258ea468c8c6a4 come from the CI run https://github.com/doodla/athena/actions/runs/43632417828 (672s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test passes, including the config tests and run::tests::a_check_killed_at_its_deadline_closes_as_an_environment_stop, so the check deadline mechanism still works. Evidence: cargo test, or a recorded Athena test check on the final tree.; cargo clippy and cargo fmt --check are clean (recorded Athena checks on the final tree).)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
