# Job 20261111-164018-fbp4: verify "Report each role's count and source in the role-need replay"

## Request (verbatim)

> try again on 20261111-145400-dx3z

## Reading (accepted by the user)

Retry of job 20261111-145400-dx3z, which ended with an empty diff because the base already fixes the cause (role_need reads Records::all_invocations; the steward, audience and auditor are never retire candidates). I read the request as: confirm that, and add the one missing piece, a replay that lists each role's run count and the data source it came from, with a test over known-active roles.

Assumptions:
- Code survey: I read crates/athena-core/src/role_need.rs (RoleNeed::compute counts via Records::all_invocations; used, retire_candidates and not_measured hold only (role, count) pairs with no source; tests include known_active_roles_are_never_retire_candidates_and_an_unused_role_is and this_module_reads_usage_only_through_all_invocations) and grep'd store.rs and docs/system-facts.md. No record or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched. Retry of job 20261111-145400-dx3z: its producer found the earlier fix already in the base and changed nothing, so that fix is kept, not redone.
- Job 20261111-145400-dx3z: its earlier work (system-facts section, store.rs doc comment, on-demand roles reported as not measured, tests, CHANGELOG entry) is already present and is kept unchanged. The gap left is that the output does not say which data source each count came from. This job adds that. Files that job's candidate changed: none. Its listed docs/spec.toml, status.rs, lib.rs, store.rs, system-facts.md and shard-timings.txt are dropped as already done or unneeded.
- Goal: Athena maintainers read the role-need replay and see that the steward, audience and auditor are counted and not flagged. The measure adds no refusal rule, so there is no refused-readings share to state.
- If the producer again changes nothing, its report must say why: already done (with evidence), blocked (by what), or gave up (why).
- Existing code: crates/athena-core/src/role_need.rs and crates/athena-core/src/store.rs (Records::all_invocations) are extended, not replaced.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core role_need`: a test builds Records via from_parts with a steward, audience and auditor run that appear in no job's invocations, and asserts each is counted with a nonzero count, labelled with the source it came from (outside job records), and is not a retire candidate.
2. [behavior] Run `cargo test -p athena-core role_need`: a replay test over a list of known-active roles asserts that each role reports its count and source, that no known-active role is flagged, and that a mixed record set yields both flagged and unflagged outcomes.
3. [content] crates/athena-core/src/role_need.rs gives every role in the output a count and a data-source label, and still reads usage only through Records::all_invocations (the existing source-check test passes).
4. [content] CHANGELOG.md has an entry naming the root cause (the usage measure read a partial run source) and the per-role count and source output.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass, including the existing all_invocations_covers_every_loaded_invocation and status_role_need tests.

## Expected deliverables

- crates/athena-core/src/role_need.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_role_need.rs (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- OLY-10-02 improvement_cycle and status_build_measurement.rs (job 20261029-233058-fveq work, neither changed nor reverted)
- Automatically creating or retiring roles
- OLY-9-12 (value of a new role against a simpler alternative)
- A generic refusal rule or gate on new measures
- Changes to prompts, athena.toml or defaults.toml
- Reworking the producer's structured-outcome reporting (introspection 20261111-163944-gnda design change)
- docs/system-facts.md, store.rs, status.rs, lib.rs and docs/spec.toml: already done in the base, kept as is

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria run the measure on mixed and known-active record sets and check the outcome.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on top of the existing fix.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bc6a02a2b6ca → candidate 891eda44fda2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   5 +
 crates/athena-cli/src/display.rs            |   7 +-
 crates/athena-cli/tests/status_role_need.rs |  13 +++
 crates/athena-core/src/role_need.rs         | 144 ++++++++++++++++++++++++++++
 4 files changed, 168 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-164018-fbp4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-164018-fbp4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43064855029 for 891eda44fda237a1a105a1be105e39dbedd401e2)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43064855029 for 891eda44fda237a1a105a1be105e39dbedd401e2)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43064855029 for 891eda44fda237a1a105a1be105e39dbedd401e2)
- ci-result: `not a check` passed (Check results for 891eda44fda237a1a105a1be105e39dbedd401e2 come from the CI run https://github.com/doodla/athena/actions/runs/43064855029 (733s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass, including the existing all_invocations_covers_every_loaded_invocation and status_role_need tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
