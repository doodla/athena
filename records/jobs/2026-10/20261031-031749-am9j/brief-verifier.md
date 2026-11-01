# Job 20261031-031749-am9j: verify "Add stack-aware plan_rebase and rebase_job_branch for landing"

## Request (verbatim)

> try again on 20261031-030119-tw3v. Second half of async drain A6: rebase each verified candidate onto the latest main before it lands.
> Depends on 20261031-031744-c5yx
> Priority: urgent
> 
> The first half (20261031-031744-c5yx) adds the landing queue. This half adds the rebase the queue head runs before landing. Returning a conflicting candidate to its producer, and the thrash stop of async-drain.md section 6, are not in this job.
> 
> Design, from the round-3 plan of tw3v and its critic's findings, with the open ruling made:
> - A new `Steps::rebase` (default: clean) runs for the queue head before `Steps::land`. In production it is `Athena::rebase_for_landing`, a thin wrapper over a free function `rebase_job_branch` in crates/athena-core/src/gh/landing.rs that takes git, the state dir and the remote as arguments, so tests run it against a bare origin.
> - `plan_rebase` is stack-aware: landing commits are stacked on one local branch, so it replays the commits between the fork and the job's landing commit one at a time onto origin's main.
> - A conflict in the job's own landing commit holds that job back: not landed, main untouched, no retry, no drain stop. The next queued candidate still lands.
> - Ruling on the critic's major finding: an earlier commit on the stack that conflicts is carried as a records-only commit. Its records/ paths are kept and its code paths are dropped, so no job's records leave the branch. A test shows the held-back job's records still load from the new tip.
> - Earlier unlanded commits that replay cleanly still ride along in the next job's pull request, as they do today. Say this in CHANGELOG.md and async-drain.md; do not claim the re-check rule covers stacked content.
> - Size: the wrapper test and the second bare-origin test may be reported as read-only checks if the run is long.
> 
> Acceptance:
> - Core tests of plan_rebase: a clean move, already current, a conflict in the job's own commit, an earlier conflicting commit carried as records only (its job record still loads), and a clean earlier commit carried.
> - A bare-origin test: two jobs where main moved after the second was verified; the second lands rebased.
> - A drain test: a conflicting head is held back and the next candidate lands.
> - With max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs, docs/design/async-drain.md, CHANGELOG.md

## Reading (accepted by the user)

Job 20261031-030119-tw3v (A6 second half) failed in planning, so I cut it to one deliverable: core rebase logic. This adds `plan_rebase` and `rebase_job_branch` in athena-core's gh/landing.rs, replaying the stacked landing commits onto origin's main. A conflicting earlier commit is carried as records-only. The drain wiring is split off as a follow-up. It depends on 20261031-031744-c5yx, which landed the queue.

Assumptions:
- Code survey: I read crates/athena-core/src/land.rs (resync_tip exists at line 99) and grepped crates/. The producer must read crates/athena-core/src/gh/landing.rs (Athena::land_pr, landed_commit_for, after_merge, resync_onto_merge, Records::load reading from the local branch head), crates/athena-core/src/gh/mod.rs (re-exports) and crates/athena-git (replay, merge_tree, reset_keep, fetch_remote_branch). plan_rebase is new code. The behavior is measured directly by calling production plan_rebase and rebase_job_branch against real git repositories, including a bare origin, not through mocks. No record file or AnyRecord kind is added, so check_record_file and the round-trip test are unaffected. This change adds no refusal rule.
- Goal and audience: the maintainer and verifier want each verified candidate rebased onto the latest main before it lands, with no job's records lost. This job delivers only the core mechanism that the drain will call.
- Retry of job 20261031-030119-tw3v. Its candidate recorded no changed files, so there is nothing to keep. Its earlier findings are addressed as follows. The major finding (record loss) is fixed by the user's ruling: a conflicting earlier commit is carried as a records-only commit. The gh/mod.rs finding is fixed by listing it as a deliverable. The CHANGELOG finding is fixed by stating that clean earlier unlanded commits ride along, and that the re-check rule does not cover stacked content. The queue-order finding is fixed because job 20261031-031744-c5yx already documented it. The size finding is fixed by this cut.
- Job 20261031-031744-c5yx (landed, verified) supplied the LandingQueue in crates/athena-cli/src/drain.rs, docs/design/async-drain.md and CHANGELOG.md. Those files are kept as they are, except that async-drain.md and CHANGELOG.md get new notes here. Its minor finding about await_head not checking stop_requested is outside this job.
- Scope cut for the verifier timeout. This job does not touch crates/athena-cli/src/drain.rs. Steps::rebase, Athena::rebase_for_landing, the drain test of a held-back head and the max_in_flight = 1 pass-through are a follow-up request that depends on this one. For that reason the CHANGELOG and docs entries describe only the core function and what the drain will do with it.
- Governance files (athena.toml, defaults.toml, workflows, lints) need no change.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core plan_rebase`. Core tests call production plan_rebase on real git repositories. They cover a clean move onto a moved main, already current, a conflict in the job's own landing commit (the plan reports a hold with main untouched), an earlier conflicting commit carried as records only, and a clean earlier commit carried.
2. [behavior] Run `cargo test -p athena-core`. In the records-only test, the earlier job's records/ paths are present at the new tip and its code paths are dropped. Records::load from the new tip still loads that held-back job's record.
3. [behavior] Run `cargo test -p athena-core rebase_job_branch`. A test against a bare origin has two jobs stacked locally, with main moved after the second was verified. rebase_job_branch (taking git, the state dir and the remote as arguments) leaves the second job's landing commit rebased onto origin's main, and `git log` shows a linear history. A second case, a conflict in the job's own commit, leaves origin's main and the local branch unchanged.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, and the existing tests are unchanged, including the athena-cli drain tests.
5. [content] `git diff` shows changes only to crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs (the re-export), docs/design/async-drain.md and CHANGELOG.md. drain.rs is untouched.
6. [content] CHANGELOG.md has an entry for the core rebase. docs/design/async-drain.md has a note. Both say that earlier unlanded commits which replay cleanly still ride along in the next job's pull request as they do today, and that the re-check rule does not cover stacked content. Both say that a conflicting earlier commit is kept as records only. Both name the drain wiring as a follow-up.

## Expected deliverables

- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Steps::rebase, Athena::rebase_for_landing and wiring the rebase into crates/athena-cli/src/drain.rs: follow-up request A6-rebase-drain, which depends on this one
- The drain test of a held-back head with the next candidate landing, and the Athena wrapper test: follow-up with the drain wiring
- Returning a conflicting candidate to its producer, send_back/drop_landed, and the section-6 thrash stop
- Raising max_in_flight above 1 (build job 10)
- Changes to athena.toml, defaults.toml, .github/workflows, crates/athena-cli/tests/drain.rs and crates/athena-core/src/run/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for plan_rebase and rebase_job_branch; fmt, clippy and the test suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run real git repositories, including a bare origin, through production functions.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Cut to the core mechanism; the drain wiring is a follow-up.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Records are carried on the branch rather than dropped, so a held-back job's records are not lost.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and design doc notes are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Tests in a `#[cfg(test)] mod rebase_tests` in landing.rs call the production `plan_rebase` on real temporary git repositories. Each is made with `Git::isolated` and `git init`, with commits from `crate::land::fixtures::commit/landed/plain` (`pub(crate) mod fixtures`, land.rs ~243; the repo helper is in the land.rs tests ~351 and is copied locally). Cases: plan_rebase_clean_move (main gets an outside commit and the job's commit sits on the old main); plan_rebase_already_current (onto is an ancestor of the landing commit); plan_rebase_conflict_in_own_commit (main and the job's commit edit the same line); plan_rebase_earlier_conflict_is_records_only (stack b then c, b conflicts with the new main, c is disjoint); plan_rebase_clean_earlier_commit_is_carried (stack b then c, both clean); plan_rebase_records_only_commit_is_a_hold (see below); plan_rebase_parentless_commit_is_an_error. Run: `cargo test --locked -p athena-core plan_rebase`. Observes: (a) Moved. The new landing commit's parent is the outside commit and its message equals the old one. (b) Current. (c) Conflict whose `commit` is the job's own commit; the main ref is not asserted here, because the pure plan moves no ref. (d) Moved with `records_only` equal to [b's old commit]. The tip's parent chain is onto, b's records-only commit, then c. (e) Moved with `records_only` empty, and the tip holds b's file and c's file. (f) After the (d) rebase, `landed_commit_from_records(git, tip, job_b, fingerprint)` returns the carried commit, which is the post-C0 lookup the critic named. `plan_rebase(onto, tip, carried_b)` returns Conflict naming it and not Current. Its message has the `Athena-Records-Only:` marker and no Athena-Job trailer. (g) A parentless commit gives Err.
  Fails without the behavior: Without the replay, the clean-move test sees no new parent chain. Without the conflict check on the job's own commit, case (c) gets Moved or Err. Without the records-only carry, case (d) sees a Conflict or a branch missing b's records. Without the Current check, case (b) sees Moved. Without the marker check, case (f) gets Current for a job with no code.
  Test seam: Production `plan_rebase` and `landed_commit_from_records` called directly on real repositories from `Git::isolated` and `land::fixtures`. These exist today (read in land.rs). Step 3 adds the tests. There is no mock. The test sets `onto` as a commit hash, so no remote is needed.
- Criterion 2: The test plan_rebase_earlier_conflict_is_records_only goes further. b's landing commit is built with `fixtures::landed(job_b)` for its records and `fixtures::plain` for a code file that conflicts with the moved main. c's commit uses `fixtures::landed(job_c)` and a disjoint code file. After `plan_rebase` for c, it reads the new tip. It calls `git.show_file(tip, <b's code path>)`, `git.show_file` for each of b's records/ paths, and `crate::store::Records::load(&git, &tip)`, the production loader that `land_pr` uses. The whole athena-core suite runs, so every existing test runs too. If `Records::load` fails on the fixtures' repository, the producer fixes the fixture setup (the fixtures write records through AnyRecord::canonical_path and to_json, land.rs ~254, and store.rs tests call Records::load on such commits) or reports criterion 2 as not met. A path-only check is not accepted as a substitute. Run: `cargo test --locked -p athena-core`. Observes: At the new tip b's code path is absent and b's records/ paths are present. `Records::load(&git, &tip)` returns jobs for both b and c, and b's candidate, verification and outcome records are readable. The suite passes.
  Fails without the behavior: If the carry dropped records/, or kept the code paths, the path assertions fail. If it replaced all of records/ with a snapshot, c's record would be missing and `Records::load` would lack job c. Without the carry, `Records::load` at the new tip would lack b's job.
  Test seam: Production `plan_rebase` and `Records::load` on a real repository, with fixtures from land.rs (exist today, read). The test is added in step 3. Whether `Records::load` accepts the fixtures' repository is settled by running the test, with the fallback above.
- Criterion 3: Tests in landing.rs call the production `rebase_job_branch(git, state, remote_name, default_branch, job, verified_fp, sign, ui)` against a real bare origin. The origin is `git init --bare` in a temp directory. The work repository, made with `Git::isolated`, has `origin` pointing at it (via `git remote add origin <path>`). It has two jobs' landing commits stacked on one local branch. A third clone pushes an outside commit to the bare main after the second job was verified. State is a `StateDir` on a temp directory, and `Ui` is the crate's quiet or recording test Ui. The setup shape is taken from `records_flow::World` (gh/tests.rs ~1971) and `test_support`, which the producer reads first. The published-head precondition holds because no work branch is pushed. Cases: rebase_job_branch_lands_second_job_on_the_moved_main; rebase_job_branch_conflict_changes_nothing; rebase_job_branch_unchecked_when_origin_is_unreachable; rebase_job_branch_unchecked_for_a_published_head. Run: `cargo test --locked -p athena-core rebase_job_branch`. Observes: First case: Moved. `git rev-list --merges <bare main>..<tip>` is empty, and `git rev-list --parents -n 1` shows exactly one parent for each new commit. The second job's new landing commit has the bare main's tip as an ancestor. The first job's commit rides along. Second case: Conflict. The local branch tip equals the old tip and the bare main equals the old main. Extras: Unchecked, with no ref moved.
  Fails without the behavior: Without the replay in rebase_job_branch, job 2's landing commit stays on the old main and the ancestor assertion fails. A rebuilt branch with merge commits fails the rev-list --merges check. Without the conflict path, the local branch or the bare main moves, or the call returns Err.
  Test seam: Production `rebase_job_branch` with git, the state dir and the remote name passed as arguments, against a real bare origin. The Ui and StateDir helpers exist. Step 4 adds the tests. Cut line: the conflict case and the two Unchecked extras may be reported as read-only checks if the run is long. The Moved case and the plan_rebase tests are never cut.
- Criterion 4: Run the three Athena checks on the final tree. The new items are `pub` and re-exported, so the dead-code lint does not fire. No `#[allow]` is added. No existing test and no drain.rs line is edited, so the drain tests run as they were. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0.
  Fails without the behavior: Unformatted code fails fmt. An unused item or parameter fails clippy under -D warnings. A changed or broken existing test fails cargo test.
  Test seam: Existing toolchain commands; there is no seam and nothing is added to any test double.
- Criterion 5: Read the diff listing after the change. Then run `git diff -- crates/athena-core/src/gh/mod.rs` to confirm the change is the one re-export list line. Run: `git diff --stat`. Observes: Only crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs, docs/design/async-drain.md and CHANGELOG.md appear. crates/athena-cli/src/drain.rs is absent.
  Fails without the behavior: A drain.rs or any other file in the diff would show here.
- Criterion 6: Read both diffs. The CHANGELOG entry goes under `### Added` in `## [Unreleased]` (line 185). The async-drain.md note goes in section 6 after the first-half note (lines 137-141). Both are written for an outside reader. Run: `git diff -- CHANGELOG.md docs/design/async-drain.md`. Observes: Both texts say: earlier unlanded commits that replay cleanly still ride along in the next job's pull request as they do today; the re-check rule does not cover stacked content; a conflicting earlier commit is kept as records only, so its records stay and its code is held back, and a marker keeps it from being found as that job's landing commit; queue order is pass order, not stacking order; a Moved result may change verified content and the re-check is job 7; and the drain wiring (Steps::rebase and the drain test) is a follow-up. The CHANGELOG entry also names the gh/mod.rs re-export.
  Fails without the behavior: A missing sentence is visible in the diff.

Planned files:

- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 98db149eb00b → candidate 6246f7a23581. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  15 +
 crates/athena-core/src/gh/landing.rs | 608 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/gh/mod.rs     |   5 +-
 docs/design/async-drain.md           |  25 ++
 4 files changed, 651 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41336360303 for 6246f7a23581c79544a3e02c93a84c84c41e8903)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41336360303 for 6246f7a23581c79544a3e02c93a84c84c41e8903)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41336360303 for 6246f7a23581c79544a3e02c93a84c84c41e8903)
- ci-result: `not a check` passed (Check results for 6246f7a23581c79544a3e02c93a84c84c41e8903 come from the CI run https://github.com/doodla/athena/actions/runs/41336360303 (470s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, and the existing tests are unchanged, including the athena-cli drain tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
