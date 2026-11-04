# Job 20261010-154013-a9gz: verify "Land a job only on the branch it was dispatched from"

## Request (verbatim)

> Land a job only on the branch it was dispatched from: today it lands on whatever branch is checked out when it finishes, if its base is an ancestor there.

## Reading (accepted by the user)

Landing (`land::plan`, called from run/close.rs at ~228 and ~484) only checks the job's base is an ancestor of HEAD, so a job lands on whatever branch is checked out at finish. The change records the dispatch branch and makes the job stale (or blocked on retry) with a clear reason when the current branch differs. The audience is the Athena user who switches branches while jobs run.

Assumptions:
- Code survey: entry points are `plan` in crates/athena-core/src/land.rs (checks `is_ancestor(base, head)`, then classification, then fingerprint) and its two callers in crates/athena-core/src/run/close.rs (~228 first landing, ~484 retry after blocked); `current_branch()` is in crates/athena-git/src/lib.rs:344. The new branch check runs first in `plan`, before the ancestor check. `Outcome.branch` (crates/athena-data/src/records.rs:872) records the branch at finish, not at dispatch, so the dispatch branch must come from the job's dispatch-time state or context. Preferred: reuse an existing dispatch-time field and leave records.rs unchanged. If a new optional field is added to a record in crates/athena-data/src/records.rs, it must be accepted by `check_record_file` and `AnyRecord` there (the records validation, with `deny_unknown_fields`), and covered by that file's existing round-trip tests. The behavior is measured directly: by running a real land through `plan` and the close.rs paths on a git fixture with two branches and checking the branch tip and the outcome state.
- Detached HEAD at dispatch or at finish: treat a differing or absent branch as stale, with a reason saying so.
- Jobs dispatched before this change have no recorded dispatch branch; they keep the old behavior (no branch check) rather than going stale.

## Acceptance criteria

1. [behavior] Run `cargo test`. A new test dispatches a job on branch A, switches to branch B (which contains the base commit), finishes the job, and shows that B's tip is unchanged and the outcome is stale (or blocked) with a reason naming the dispatch branch.
2. [behavior] Run `cargo test`. A test shows a job still lands as a fast-forward when the checked-out branch at finish is the dispatch branch, including the records-only-changes replay case.
3. [behavior] Run `cargo test`. A test shows the retry-after-blocked landing path (close.rs ~484) also refuses when the current branch differs from the dispatch branch.
4. [behavior] Run `cargo clippy` and `cargo fmt --check`: both are clean, and the existing land and landing tests in crates/athena-core still pass.
5. [content] The header comment of crates/athena-core/src/land.rs states the rule that the current branch must equal the dispatch branch, alongside the existing ancestor rule.
6. [content] CHANGELOG.md has an entry stating that jobs now land only on the branch they were dispatched from.
7. [behavior] Run `cargo test`. If a record field is added, the round-trip tests for `check_record_file` and `AnyRecord` in crates/athena-data/src/records.rs pass with it and with records that lack it.
8. [behavior] This change refuses more landings, and the higher refusal rate is intended: expected 5% of newly refused readings.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)

## Out of scope

- Changing how jobs are dispatched or read.
- Changing athena.toml, defaults.toml or workflow files.
- Auto-switching branches or auto-redispatching stale jobs.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the branch rule; cargo test and clippy clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checked by running cargo test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule goes in land::plan, so both landing paths get it.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The land.rs header and CHANGELOG state the rule for an outside reader.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that a job does not land on the wrong branch, not that wiring exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the landing decision.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two integration tests drive the real `Athena::go` flow in crates/athena-cli/tests/lifecycle.rs, plus a land.rs unit test on `plan`. Test (a), `a_job_finished_on_a_branch_with_its_records_goes_stale`: the fixture is on `main`. A guarded-once `Scenario.during_verification` hook runs `git checkout -q -b other` (so `other` holds the dispatch commit) and stores `other`'s tip in an `Arc<Mutex<String>>`. After `go` it asserts: state `Stale` and the reason contains `main`; `other`'s new tip has the captured tip as its parent; `git diff --name-only` between them lists only paths under `records/`; README.md has no greeting; `fingerprint(new tip, base)` equals `fingerprint(base, base)`; `main`'s tip is unchanged; `job.json` at the dispatch commit has `dispatch_branch` `main`; `check_history()` reports no violations. After `git checkout -q main` it asserts `status()` lists the job in `open` with state `interrupted`, and `go` on it returns an error containing `interrupted`. Test (b), `a_job_finished_on_a_branch_without_its_records_changes_nothing`: before `ask`, the test runs `git branch other` at the fixture's initial tip, so `other` has the base but not the dispatch commit. The hook captures `main`'s tip and runs `git checkout -q other`. It asserts `go` returns an error naming `main` and saying nothing was recorded or landed; `other`'s tip equals the initial tip exactly; README.md has no greeting; `git ls-tree -r --name-only other` lists nothing under `records/jobs/`; after `git checkout -q main`, `status()` shows the job `interrupted`. The unit test `a_job_dispatched_elsewhere_is_refused_on_another_branch` calls `plan` with `Dispatched::Branch("main")` on a repository whose current branch is `other`, with the base an ancestor of head, and expects `Plan::OtherBranch` naming `main`. Run: `cargo test --locked -p athena-cli --test lifecycle a_job_finished_on_a_branch && cargo test --locked -p athena-core land::tests::a_job_dispatched_elsewhere`. Observes: A job finished on another branch lands nothing: state `stale` with a reason naming the dispatch branch when the other branch holds the job's records; no commit and an error naming the dispatch branch when it does not. The other branch gets no job content in either case, and in case (b) its tip is unchanged. The gate accepts the case (a) records commit. After returning, the dispatch branch shows the job as `interrupted`.
  Fails without the behavior: Without the check, `plan` returns `Plan::Tree` because the base is an ancestor of `other`'s head, and the job lands on `other` with state `Landed`. The state, reason, README, fingerprint, tip and records assertions in (a) and (b) all fail. The unit test fails because `plan` returns `Plan::Tree`.
  Test seam: Production entry points: `Athena::go` (reaching `close_dispatched`, then `land_or_block`, close.rs:204) and `land::plan`. Injection: the existing `Scenario.during_verification` hook (`Option<Arc<dyn Fn() + Send + Sync>>`, tests/common/mod.rs:422, called at :631; lifecycle.rs:1726 and :1773 already use it), and `fx.git.run` for the branch setup. The hook only runs git checkout commands and records a tip; the branch decision stays in production code. Both exist today. The `Dispatched` argument and `Plan::OtherBranch` are added in steps 4 and 5. The unit test reuses the existing `repo()`, `commit()` and `pass()` helpers.
- Criterion 2: The existing lifecycle test `a_candidate_lands_over_a_records_only_interleave` stays and gains assertions that the state is `Landed` and that `job.json` at the dispatch commit has `dispatch_branch` `main`. A new land.rs unit test `a_job_lands_when_the_dispatch_branch_is_checked_out` calls `plan` with `Dispatched::Branch("main")` on `main`, twice. Once with head equal to the base (expects `Plan::Tree` of the candidate's tree). Once after a commit that changes only an excluded path (`PRIVATE.md`), which takes the `merge_tree` replay branch (expects `Plan::Tree`). Run: `cargo test --locked -p athena-cli --test lifecycle a_candidate_lands_over_a_records_only_interleave && cargo test --locked -p athena-core land::tests::a_job_lands_when`. Observes: A fast-forward landing with state `Landed` occurs on the dispatch branch, both with head equal to the base and after records-only changes were replayed.
  Fails without the behavior: A rule that compared the wrong branch, or refused whenever the head moved, turns these into a refusal and fails the `Landed` and `Plan::Tree` assertions. The old behavior also passes, so this pins the allow side of the rule, not the refusal.
  Test seam: Production entry points: `Athena::go` and `land::plan`. Fixtures: the existing `Fixture` and `during_verification` hook add the records-only commit; the land.rs unit test uses the existing `repo()`, `commit()` and `pass()` helpers. All exist today.
- Criterion 3: New lifecycle test `a_blocked_job_will_not_retry_on_another_branch`. It copies the setup of `local_edits_block_the_landing_and_go_retries_it` (lifecycle.rs ~1773): `during_verification` writes a local edit, so `go` ends `Blocked` on `main`. The test restores README.md, runs `git checkout -q -b other` (which holds the blocked records), records `other`'s tip, and calls `athena.go` on the job, reaching `retry_landing`. It asserts: the call is an error naming `main` and saying the job stays blocked; `other`'s tip is unchanged; README.md has no greeting; no `outcome-landed.json` exists. It then runs `git checkout -q main` and `go` again, and asserts `Landed`. Run: `cargo test --locked -p athena-cli --test lifecycle a_blocked_job_will_not_retry_on_another_branch`. Observes: The retry on the wrong branch is refused with the dispatch branch in the error, the job stays blocked, and nothing is committed. The retry on the dispatch branch lands.
  Fails without the behavior: Without the check in `plan`, the retry on `other` lands the change: `go` returns `Ok` with state `Landed`, so the `is_err` assertion fails.
  Test seam: Production entry points: `Athena::go` on a blocked job (reaching `retry_landing`, close.rs:387) and `land::plan`. The fixture and hook exist today, and `local_edits_block_the_landing_and_go_retries_it` already uses them. The new test adds no seam.
- Criterion 4: Run the three checks Athena runs on the result, then the land tests in athena-core and the lifecycle landing tests. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked -p athena-core land`. Observes: Formatting and clippy report nothing. All land.rs tests (the squash resync, `pr_commits` and the existing `plan` tests, which now pass `&Dispatched::Unknown`) pass with the new signature.
  Fails without the behavior: A missed call site of `plan`, a missed `Plan` match arm, or a missed `Job` struct literal fails the build. A changed behavior in the existing `plan` tests fails them.
  Test seam: The cargo commands run the production code as is. The existing land.rs tests call `plan` directly, with the argument added in step 4.
- Criterion 5: Content check by reading the `//!` header of crates/athena-core/src/land.rs in the diff. Run: `git diff -- crates/athena-core/src/land.rs`. Observes: The header's rule block has a line 'require the current branch is the branch the job was dispatched from, else stale' next to the ancestor line. A sentence for an outside reader covers the detached case (recorded as an empty branch name), and jobs dispatched before the field existed (no check).
- Criterion 6: Content check by reading the CHANGELOG.md diff. Run: `git diff -- CHANGELOG.md`. Observes: An entry under `## [Unreleased]`, in a `### Changed` section (added if absent), says jobs now land only on the branch they were dispatched from. A job finished on another branch lands nothing: it goes stale, or stays blocked on retry.
- Criterion 7: New tests in the records.rs test module. (a) A `Job` with `dispatch_branch: Some("main")` and one with `Some("")` pass through `AnyRecord::Job(..).to_json()`, `AnyRecord::parse` and `check_record_file`, and equal the original. (b) A job JSON with no `dispatch_branch` key parses, yields `None`, and serializes without the key. (c) The existing `Job` test literal (~4850) gets `dispatch_branch: None` and still round-trips. Run: `cargo test --locked -p athena-data records`. Observes: All records round-trip tests pass, with and without the new field.
  Fails without the behavior: If the field were missing from `Job`, `deny_unknown_fields` would make `check_record_file` and `AnyRecord::parse` reject the record that has it. If it lacked `#[serde(default)]`, the record without it would fail to parse.
  Test seam: Production entry points: `AnyRecord::parse` and `check_record_file` (records.rs ~3045). The tests build records in memory and need no fixture. They exist today. The store.rs `dispatched()` literal (~597) is updated in step 3.
- Criterion 8: The refusals are shown by the criterion 1 and 3 tests. The '5% of newly refused readings' figure is an expectation about use, not measured by any test, and the plan does not claim to check it. The full suite shows the extra refusals are limited to jobs that switch branches between dispatch and finish. Run: `cargo test --locked`. Observes: The full suite passes. New refusals appear only in the new tests; same-branch landings in lifecycle, drain, reconcile, trailer_stop and history_pr_head are unchanged (each creates its branch before `ask`, or does not dispatch a job across branches).
  Fails without the behavior: A rule that refused ordinary same-branch landings would fail the existing landing tests, such as `landed` in lifecycle.rs near line 223.
  Test seam: Production entry point: `Athena::go` through the existing integration suite. The fixtures exist today.

Planned files:

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 6226e319db0c → candidate 206f48b1cae4. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  11 ++
 crates/athena-cli/tests/lifecycle.rs | 199 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/land.rs       | 175 ++++++++++++++++++++++++++++--
 crates/athena-core/src/run/close.rs  |  34 +++++-
 crates/athena-core/src/run/mod.rs    |   4 +-
 crates/athena-core/src/store.rs      |   1 +
 crates/athena-data/src/records.rs    |  61 +++++++++++
 7 files changed, 475 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41766474295 for 206f48b1cae4e43a302b881f5d25ace9da880fce)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41766474295 for 206f48b1cae4e43a302b881f5d25ace9da880fce)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41766474295 for 206f48b1cae4e43a302b881f5d25ace9da880fce)
- ci-result: `not a check` passed (Check results for 206f48b1cae4e43a302b881f5d25ace9da880fce come from the CI run https://github.com/doodla/athena/actions/runs/41766474295 (628s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy` and `cargo fmt --check`: both are clean, and the existing land and landing tests in crates/athena-core still pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
