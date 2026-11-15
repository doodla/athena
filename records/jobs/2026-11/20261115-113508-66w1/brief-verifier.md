# Job 20261115-113508-66w1: verify "Scale planner time limit when revision escalates effort"

## Request (verbatim)

> try again on 20261114-052309-w3zt

## Reading (accepted by the user)

Retry of job 20261114-052309-w3zt (itself a retry of 20261114-041123-hvyh, whose round 2 ran at xhigh for 606 s against the 600 s limit). In planning_round, when planner_revision_setting raises effort above the planner's own, the planner time limit is multiplied by a named constant (2) instead of staying at planner_timeout_s. The earlier candidate's three files are kept as the starting point. The job recorded no findings, so there are none to fix.

Assumptions:
- Goal and audience: the owner (Sri) and the Athena operators who judge whether xhigh revision rounds complete instead of timing out.
- Code survey: read crates/athena-core/src/run/planning.rs lines 125-175. planning_round starts the planner at the ordinary effort. When round > 1 or mechanical_passes > 0, planner_revision_setting (crates/athena-data models config) may raise model and effort. The AgentSpec is then built with cfg.planner_timeout_s at line 162. Tests live in crates/athena-core/src/run/tests.rs, whose config fixtures set planner_timeout_s = 600. The change adds no record or data file, so the records allow-list, AnyRecord and check_record_file are not touched. The timeout is measured directly on the AgentSpec the planner is invoked with.
- Job 20261114-052309-w3zt was dispatched but never verified and recorded no findings. Its candidate changed CHANGELOG.md, planning.rs and tests.rs. That work is kept as the starting point and re-verified here. The changes to config.rs and docs/decisions.md that it listed but never made are dropped.
- Escalation is detected by comparing the effort returned by planner_revision_setting with the planner's own effort. The multiplier is a named constant, PLANNER_ESCALATED_TIMEOUT_FACTOR = 2, with a doc comment. The scaling uses saturating_mul.
- No new config key is added, so the Consumers rule is not triggered. Governance files (athena.toml, defaults.toml) are unchanged.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/run/tests.rs sets planner_revision_effort stronger than the planner's effort and planner_timeout_s = 600. It runs a revision round (round 2) and asserts the planner AgentSpec timeout is greater than 600 s (1200 s). Run: cargo test -p athena-core.
2. [behavior] A test asserts that round 1, and a revision with planner_revision_effort unset, weaker or equal, all keep exactly planner_timeout_s. Run: cargo test -p athena-core. The test fails if scaling is applied unconditionally.
3. [content] The diff in crates/athena-core/src/run/planning.rs scales cfg.planner_timeout_s only when the revision setting raised the effort. It uses a named constant whose doc comment cites job 20261114-041123-hvyh (606 s against a 600 s limit at xhigh).
4. [content] The deliverables are only planning.rs, run/tests.rs and CHANGELOG.md. The diff (git diff) leaves athena.toml, defaults.toml, hestia.rs, crates/athena-data/src/config.rs, docs/decisions.md, the prompts and records/ unchanged.
5. [content] CHANGELOG.md has a new entry recording the planner time limit scaling on escalated revisions and citing job 20261114-041123-hvyh.
6. [behavior] cargo fmt --check, cargo clippy and cargo test pass on the final tree (recorded Athena checks on the same tree count).

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The follow-up queueing feature from job 20261114-041123-hvyh (followups.rs, steward.rs, manage.rs, reading.rs, lib.rs, crates/athena-cli/tests/drain.rs): dropped, to be retried as its own request.
- Backfilling lost follow-ups from job 20261114-034034-fgwz, and the hestia.rs findings; hestia.rs is unchanged.
- crates/athena-data/src/config.rs and docs/decisions.md, which job 20261114-052309-w3zt listed but its candidate never changed: dropped.
- A new athena.toml or defaults.toml key, changes to planner_timeout_s itself, the critic timeout, planner prompts, workflows and records/.
- Changing which rounds escalate effort.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that the planner's time limit is actually larger on an escalated revision and unchanged otherwise, not that a constant exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus fmt, clippy and test criteria.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A unit test builds the repo with `repo_with_revision_keys(3, None, Some("xhigh"))` (planner_timeout_s = 600 at tests.rs:1516, planner effort high). It runs `run_for_revision` with `[good_plan()]` and `[revising_critique(), accepting_critique()]`, so round 1 is followed by a round 2 planner call. It asserts the planner call efforts are `high`, `xhigh` and the recorded planner timeouts are `[600 s, 1200 s]`, so round 2 is greater than 600 s. It also asserts the round-2 prompt contains `deadline of 1200 seconds`. Run: `cargo test -p athena-core planner_revision_escalated_effort_scales_the_planner_time_limit`. Observes: The planner spec timeouts are 600 s then 1200 s, and the round-2 brief states 1200 seconds.
  Fails without the behavior: Without the scaling, the round-2 planner call keeps a timeout of 600 s and the assertion `timeouts[1] == 1200 s` fails.
  Test seam: Production entry point: `Athena::go` into `plan_round` and `planning_round` (planning.rs:29, 83). The fake agent `Script` closure in `run_for_revision` (tests.rs:1559) already records `spec.role`, model and effort. Step 4 makes it also push `spec.timeout` for `Role::Planner` into a new `planner_timeouts` field on `RevisionRun`. The Script seam exists (read at tests.rs:1450 and 1559). The timeout field is read the same way at tests.rs:1451. The production logic stays out of the double.
- Criterion 2: One test runs three cases with a critic revise (so round 2 occurs) and asserts every planner timeout is exactly 600 s. Case (a): keys unset, effort high on both calls. Case (b): `planner_revision_effort = "medium"`, which is weaker and so is ignored (the same rule as tests.rs:1694). Case (c): `planner_revision_effort = "high"`, equal to the planner effort. In every case round 1's call is 600 s. A separate assertion in the escalated test covers a mechanical-pass revision (`bad_plan`, then `good_plan`) with `planner_revision_effort = "xhigh"` and expects its second call at 1200 s. Run: `cargo test -p athena-core planner_time_limit_is_unchanged`. Observes: All planner timeouts are 600 s in cases a to c; round 1 is 600 s in all cases.
  Fails without the behavior: If the scaling were unconditional or keyed on `round > 1`, the unset, weaker and equal cases would show 1200 s and the test would fail.
  Test seam: The same `run_for_revision` helper and `repo_with_revision_keys` (tests.rs:1500, 1549), both existing, with the `planner_timeouts` field added in step 4. Efforts come from the config the fake runs under. Nothing in the double decides the timeout.
- Criterion 3: Read the diff. It shows `PLANNER_ESCALATED_TIMEOUT_FACTOR` with a doc comment naming job 20261114-041123-hvyh (606 s against 600 s at xhigh). It shows the timeout multiplied only inside a branch that compares the returned effort with the base effort, and `saturating_mul` used. Run: `git diff -- crates/athena-core/src/run/planning.rs`. Observes: The constant, its doc comment citing the job id, the single guarded multiplication, and the same timeout value feeding `deadline_s` and `agent_spec`.
- Criterion 4: List the changed files. Only crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/tests.rs and CHANGELOG.md appear. athena.toml, defaults.toml, hestia.rs, crates/athena-data/src/config.rs, docs/decisions.md, prompt files and records/ are absent. Run: `git diff --stat`. Observes: Exactly three paths in the stat.
- Criterion 5: Read the diff. It adds one bullet under `## [Unreleased]` / `### Changed`. The bullet says the planner time limit is multiplied by 2 when a revision raises the effort and is unchanged otherwise. It cites job 20261114-041123-hvyh (606 s against 600 s at xhigh) and names `PLANNER_ESCALATED_TIMEOUT_FACTOR` and the tests. Run: `git diff -- CHANGELOG.md`. Observes: A new Unreleased bullet with the job id and the cause.
- Criterion 6: Run the three Athena checks on the final tree. The existing tests that must stay green: `the_planner_brief_states_the_enforced_deadline_and_a_planner_deadline_still_stops_planning` (tests.rs:1437, round 1 only, factor not applied), the four `planner_revision_*` tests (tests.rs:1617-1758) and the drain.rs `planner_revision_setting` uses (drain.rs:8124, 8132, unchanged API). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0.
  Fails without the behavior: A formatting slip, an unused field (`planner_timeouts` not read) or an existing test broken by the reordering would fail one of the three commands.
  Test seam: The commands are the Athena checks themselves. No double is involved.

Planned files:

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 238142f84aeb → candidate 7ea8a104df10. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  1 +
 crates/athena-core/src/run/planning.rs | 38 ++++++++++----
 crates/athena-core/src/run/tests.rs    | 90 +++++++++++++++++++++++++++++++++-
 3 files changed, 118 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261115-113508-66w1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261115-113508-66w1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43762750092 for 7ea8a104df10f2a9d36f5ef79240291105f06fef)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43762750092 for 7ea8a104df10f2a9d36f5ef79240291105f06fef)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43762750092 for 7ea8a104df10f2a9d36f5ef79240291105f06fef)
- ci-result: `not a check` passed (Check results for 7ea8a104df10f2a9d36f5ef79240291105f06fef come from the CI run https://github.com/doodla/athena/actions/runs/43762750092 (771s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy and cargo test pass on the final tree (recorded Athena checks on the same tree count).)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core planner_revision_escalated_effort_scales_the_planner_time_limit`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.00s

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
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core planner_time_limit_is_unchanged`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
     Running tests/comparison.rs (<job>/target/verify/debug/deps/comparison-58f5baa8c5e0bbd4)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.00s

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

```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
