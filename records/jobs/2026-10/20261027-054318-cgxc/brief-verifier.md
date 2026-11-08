# Job 20261027-054318-cgxc: verify "Close stacked earlier PRs when a later PR lands their commits"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Athena lets a job be built on an earlier unmerged job's branch, but it doesn't record that dependenc
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: stacked-job-content-lands-via-later-pr (20261015-201512-vm2y)
> Cause: Athena lets a job be built on an earlier unmerged job's branch, but it doesn't record that dependency or track the earlier pull request. When the later pull request merges, nothing links it to the earlier ones whose commits it carries. The ...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261017-125701-mkgp (job 20261010-154004-60r4); prevention it proposed: Record the base job or pull request whenever a job is cut on top of an unmerged job. After any pull request merges, check each recorded earlier pull request that is still open. If all of its commits are now reachable from main, close it aut...
> Introspection 20261017-141752-2ffq (job 20261010-154007-jbt9); prevention it proposed: Record the base job or branch when a job is cut on top of unmerged work. After any pull request merges, run a check that finds open pull requests whose head commits are all reachable from main. Close each one automatically with a comment na...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a job can be cut on an earlier unmerged job's branch, but the pull request link (`PrLink` in crates/athena-core/src/pr.rs) records no base job or earlier PR, and nothing runs after a merge to reconcile open PRs whose commits main now carries. The fix records the base job on the link and, after a merge, closes each recorded earlier open PR whose head is reachable from the merged commit, with a comment naming the PR that carried it. This follows up jobs 20261010-154004-60r4 and 20261010-154007-jbt9, whose commits landed this way.

Assumptions:
- Code survey: read crates/athena-core/src/pr.rs (PrLink holds pr_number, host, repository, head, job, state, merged_commit; no base job; Host trait has only merged_commit; observe_merge/check_merged/apply_outcome record the merge outcome) and crates/athena-core/src/gh/landing.rs (land_pr publishes, observes the merge, merges over REST, then writes the landing record and cutover; it never looks at other jobs' links). Entry point is land_pr, run after publish and merge confirmation. The property is measured directly by unit tests with fake Host and Ancestry implementations, as the existing pr.rs tests do.
- Existing code: crates/athena-core/src/pr.rs (PrLink, store_link, load_link, Host, Ancestry, observe_merge) and crates/athena-core/src/gh/landing.rs (land_pr, confirm_existing_merge) are extended, not replaced. PrLink is a state-directory file (pr-link.json), not a records-branch record, so no AnyRecord or check_record_file change is expected. New PrLink fields use serde default so old links still parse.
- Environment: the host is GitHub through the existing gh client in crates/athena-core/src/gh; closing a pull request and commenting on it are new write calls on the same token that already merges. The tests use fakes and need no network. Data reached: the job state directory's pr-link.json files and the remote's main history.
- Reading of the cause: the originating gap is that a stacked job's dependency is never recorded, not that one job's PR was left open. The class fix is a recorded base (the earlier job's id, from the branch the job is cut on) plus a post-merge reconciliation. The reconciliation closes an earlier PR only when it is still open and every commit of its head is reachable from the merged commit. Otherwise it leaves the PR alone.
- Both referenced jobs are already landed with passing verification. Their own findings are not reopened and do not apply here: for 20261010-154004-60r4 the display.rs, lifecycle-reason and deliverable-list findings; for 20261010-154007-jbt9 the weak manage.rs subject test. Their landed code is kept, and none of their files are deliverables.
- Audience: Athena maintainers who stack jobs and review the PR list. The outcome they check is that no stale open PR remains after the later one merges.

## Acceptance criteria

1. [content] The change description names the root cause: a job built on an unmerged job's branch has no recorded dependency and no post-merge reconciliation of the earlier PRs it carries. The fix is in crates/athena-core/src/pr.rs (the link) and crates/athena-core/src/gh/landing.rs (land_pr), not in one job's data. Check with git diff.
2. [content] PrLink gains a defaulted field recording the base job (or earlier PR). A test in pr.rs shows an old pr-link.json without the field still loads (git diff shows the test).
3. [behavior] A new unit test of the reconciliation (run cargo test -p athena-core) shows an open earlier PR whose head is reachable from the merged commit is closed with a comment naming the merging PR. It fails on the pre-change code, which has no such step.
4. [behavior] A new unit test shows an earlier open PR with a commit not reachable from the merged commit is left open, and an already merged or closed PR is not touched. This guards against closing live work.
5. [behavior] A test is generic over the cause: two different earlier jobs stacked in a chain (A, then B on A, then C on B) are both closed when C lands, with no job id hard-coded. Run cargo test -p athena-core.
6. [behavior] A failure to close an earlier PR is reported as a warning in PrReport.warnings and does not undo or fail the merge or the landing record. Check with a unit test using a failing fake host (cargo test -p athena-core).
7. [content] CHANGELOG.md has an entry for the change, and docs describing the PR link or landing (docs/ file that mentions pr-link) are updated if they list the link's fields. Check with git diff and grep.
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.

## Expected deliverables

- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)

## Out of scope

- Automatically detecting the base job from git history for jobs already landed
- Changing how the stacked branch is cut or how rebases work (rebase_job_branch, plan_rebase)
- Closing PRs by any rule other than all commits reachable from the merged commit
- Fixing the earlier jobs' own minor findings (display.rs provisional cost display, lifecycle test reason, manage.rs subject test input)
- Files changed by jobs 20261010-154004-60r4 and 20261010-154007-jbt9: claude.rs, scoring.rs, records.rs, athena-score lib.rs, lifecycle.rs, manage.rs, reading.rs, store.rs, athena-core lib.rs. Their landed work is kept unchanged.
- Governance files: athena.toml, defaults.toml and workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests, with clippy and fmt clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the missing dependency record and the missing post-merge reconciliation, not one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail without the change.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that no stale open PR remains after a later PR lands its commits.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the PR link and landing code.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs cargo fmt, clippy and test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff. The CHANGELOG entry and the doc comments on PrLink::base_jobs and close_carried_prs state the root cause (a job built on an unmerged job's branch has no recorded dependency and no post-merge reconciliation), say the base is found at landing, and list the shapes closed and left open (table rows 5 and 8). Run: `git diff -- CHANGELOG.md crates/athena-core/src/pr.rs crates/athena-core/src/gh/landing.rs`. Observes: The text names the cause; `base_jobs` is added in pr.rs; the record call and the reconciliation function are in landing.rs; no job id literal appears outside tests.
- Criterion 2: Unit test `an_old_link_file_without_base_jobs_still_loads` in pr.rs. It writes a pr-link.json with only the old keys at `link_path(&state, &job)`, calls `load_link`, also stores a link with `base_jobs` set and loads it back, and checks an empty list serializes without the key. A second test in gh/tests.rs, `land_pr_stores_the_bases_it_found_in_the_link`, shows `base_jobs` is recorded at that point: after `Rig::run_with` the stored link holds the earlier job id. Run: `cargo test -p athena-core base_jobs`. Observes: The old file parses with `base_jobs` empty; a non-empty list round-trips; an empty list writes no `base_jobs` key; the link stored by `land_pr` for a stacked job names the earlier job.
- Criterion 3: Tests in gh/tests.rs using `Rig`. Store an open link for an earlier job (head e1) with `store_link`, then run `Rig::run_with` (production `land_pr`) with merge_via_rest on and green checks. Case 1 is a merge-commit merge (e1 an ancestor of the merged commit). Case 2 is a squash merge (e1 an ancestor of the merging head cand1 only). Case 3 is real git: a `Git::isolated` repo, the earlier landing commit and C on the branch, a real squash commit made with `commit_tree` and leaked as at tests.rs:4115. Run: `cargo test -p athena-core landing_a_stacked_job_closes_the_earlier_pull_request`. Observes: After the report shows LinkState::Landed, the host's closed list holds exactly the earlier PR's number; the comment names the merging PR #42 and the merged commit; the earlier job's stored link keeps its own state; a warning line reports the close.
  Fails without the behavior: With the `close_carried_prs` call removed from `land_pr`, or `record_bases` not called, the host's closed list is empty and the assertion fails. The decision to close lives in production `land_pr`/`close_carried_prs`; the fake only records calls and returns scripted statuses.
  Test seam: Entry point: `land_pr` in landing.rs via `Rig::run_with` (exists, tests.rs:1469). Guards: link Landed after observe_merge and `base_jobs` non-empty. Preconditions: earlier link stored with `store_link`, its PR Open on the fake host. Fakes: `FakeHost` (tests.rs:1252) gains `statuses`, `closed`, `close_fails` and `status_fails` fields and `pr_status`/`close_pr` overrides, and a new `Edges` ancestry fake that reports given (ancestor, descendant) pairs. Both are passed as the `host` and `ancestry` arguments of `run_with`. FakeHost extension and Edges are added by plan step 6; Rig::run_with exists.
- Criterion 4: Tests `an_earlier_pull_request_with_a_commit_the_merge_does_not_carry_stays_open`, `a_merged_or_closed_earlier_pull_request_is_not_touched`, and a real-git table test `shapes_left_open` covering rows 5, 6 and 8. First: the later link is stored with `base_jobs` already naming the earlier job, the earlier link head is e1, the fake host reports head e2 (pushed since), and Edges has e1 an ancestor of cand1 only, so e2 is reachable from neither arm. A real-git variant reports a head sha git does not know, so the ancestry call errors, the PR stays open and a warning names it. Second: the host reports Merged for one earlier PR and Closed for another, both heads reachable. Third (real git, `Git::isolated`): an earlier job whose PR head is an off-branch candidate made with `commit_tree` (parent = base, not on the branch), and one whose old published head was replaced on the branch by a same-tree commit with a new hash. Run: `cargo test -p athena-core earlier_pull_request`. Observes: The host's `closed` list is empty in every case and no write call is made; the report state is still Landed; pr_status was asked only for the open, recorded PRs; the unknown-head case has a warning naming the PR number.
  Fails without the behavior: A reconciliation that closes on the recorded link alone, or skips the status check, closes the PR whose head e2 is not carried, or the merged and closed ones, and the closed-list assertion fails.
  Test seam: Same as criterion 3: production `land_pr` via `Rig::run_with` (exists), extended `FakeHost` and `Edges` (step 6). The real-git cases use `&Git` as `ancestry` (`Git` implements `Ancestry`, pr.rs:178; exists) and `Git::isolated` as the pr.rs tests do; the fixtures `crate::land::fixtures::{landed, plain, commit}` exist (land.rs:307-405).
- Criterion 5: Test `a_chain_of_stacked_jobs_closes_every_earlier_pull_request` in gh/tests.rs against a realistic-branch fixture built with `Git::isolated`. Job ids are generated by `JobId::parse` in a loop over three ids, none named in assertions (Rig.job is set to the last). Base commit; for each job, an off-branch candidate (`commit_tree` off the branch tip, as run/mod.rs:741) and a landing commit on the branch (`fixtures::landed` plus a code file). Between A and B come a records-only commit and an intervention commit (paths as at tests.rs:3982). C's PR head is its landing commit. A and B have stored open links whose heads are their landing commits (published heads). A fourth job D has a stored link whose head is its off-branch candidate. The merged commit is a real squash commit (parent = base only), leaked, with FakeContent mapping it to 'verified'. Case 1: `land_pr` for C records `base_jobs` itself from real ancestry and closes A and B. Case 2: links stored C->[B] and B->[A] only, to show the walk through the earlier links' own `base_jobs`. Case 3: the merged commit is a merge commit (parents base and C's head). Case 4 is the table's row 8 replay. The exact warnings list is asserted. Run: `cargo test -p athena-core a_chain_of_stacked_jobs_closes_every_earlier_pull_request`. Observes: Both earlier PR numbers are in the host's closed list after C lands, in every case; D's number and C's own PR number are not; each comment names C's PR; the warnings list is exactly the close notices, with no stray ancestry warning, because the merged commit is real.
  Fails without the behavior: A reconciliation that follows only the first recorded base leaves A open in case 2 (or leaves both open in case 1); a rule that tries to close D would put its number in the closed list; a rule that skips the second arm closes nothing in the squash case.
  Test seam: Entry point: production `land_pr` via `Rig::run_with` with `&Git` as `ancestry` (exists). Git history is real, built with `Git::isolated` and `fixtures` (exist). Fake host and `store_link` as for criterion 3 (extended FakeHost added by step 6). The merged commit is leaked per tests.rs:4115 and assigned with `host.merge_makes = squash` as at tests.rs:4147 (exists).
- Criterion 6: Test `a_failure_to_close_is_a_warning_and_the_landing_stands` in gh/tests.rs. The fake host's `close_fails` is set so `close_pr` errors; a second case makes `pr_status` error; a third lets the close succeed and the comment fail. The stack is as in criterion 3, and `Rig::run_with` calls `land_pr`. Run: `cargo test -p athena-core a_failure_to_close_is_a_warning_and_the_landing_stands`. Observes: `land_pr` returns Ok; `report.link.state` is Landed with `merged_commit` set; `Rig::stored()` shows Landed; `report.warnings` holds an entry naming the earlier PR number and the host's error text (the comment-failure case says the PR is closed); host.merges is 1 and no merge is retried.
  Fails without the behavior: If the close error were propagated with `?`, `land_pr` would return Err and the unwrap fails. If the link were stored before the reconciliation and rolled back, the stored-state assertion fails.
  Test seam: Entry points: production `land_pr` and `close_carried_prs`. The failing `close_pr` is the fake's scripted error only; the decision to warn and continue is production code. Seam is the extended `FakeHost` (`close_fails`, `status_fails`) added by step 6; `Rig::run_with` and `Rig::stored` exist (tests.rs:1469, 1507).
- Criterion 7: Read the diff of CHANGELOG.md, docs/records.md and docs/design/native-pr-landing.md, then grep for the field name. The CHANGELOG entry lists closed and left-open shapes. Run: `git diff -- CHANGELOG.md docs/records.md docs/design/native-pr-landing.md`. Observes: CHANGELOG.md has an Unreleased entry naming the root cause and the left-open shapes; native-pr-landing.md step 3 (the only doc mentioning pr-link, line 153) lists `base_jobs`; records.md has a short note that the link is state, not a record, and holds the base jobs; no ontology marker line is edited (docs/ontology.toml:56,128,129).
- Criterion 8: Run the three checks Athena runs over the whole workspace, which also compiles the cli crate tests that hold PrLink literals. The doc-check, ontology, boundary and `system_facts_budgets_match_computed` tests run inside `cargo test`. The test `an_unmerged_link_whose_pull_request_is_not_merged_stays_unconfirmed` (confirm_existing_merge returns None) pins the drain-side outcome of decision 'drain does not change'. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0; every PrLink literal compiles with `base_jobs`; the new tests for criteria 2-6 pass.
  Fails without the behavior: A missed `base_jobs` at any `PrLink {` literal is a compile error in that crate's tests, so `cargo test` fails; a formatting or lint slip fails the first two commands.
  Test seam: Entry points: the three cargo commands as Athena runs them (fmt, clippy, test) over the workspace. No fake or fixture; each crate's own test fixtures run unchanged. The seam exists today (the commands are the repository's checks); no plan step adds it.

Planned files:

- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)
- docs/design/native-pr-landing.md (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/after_merge_tests.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/external_calls.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base fb366fdd7178 → candidate cb12eb14a8fe. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                   |  21 +
 crates/athena-cli/src/display.rs               |   1 +
 crates/athena-cli/src/drain.rs                 |   2 +
 crates/athena-cli/tests/external_calls.rs      |   1 +
 crates/athena-core/src/gh/after_merge_tests.rs |   1 +
 crates/athena-core/src/gh/client.rs            |  80 ++++
 crates/athena-core/src/gh/early.rs             |   1 +
 crates/athena-core/src/gh/landing.rs           | 124 ++++-
 crates/athena-core/src/gh/mod.rs               |   5 +-
 crates/athena-core/src/gh/tests.rs             | 606 +++++++++++++++++++++++++
 crates/athena-core/src/pr.rs                   | 183 +++++++-
 docs/design/native-pr-landing.md               |   9 +-
 docs/records.md                                |   8 +
 13 files changed, 1035 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054318-cgxc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054318-cgxc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42526910256 for cb12eb14a8feb9f9cc54639fa3f27539aa855c9c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42526910256 for cb12eb14a8feb9f9cc54639fa3f27539aa855c9c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42526910256 for cb12eb14a8feb9f9cc54639fa3f27539aa855c9c)
- ci-result: `not a check` passed (Check results for cb12eb14a8feb9f9cc54639fa3f27539aa855c9c come from the CI run https://github.com/doodla/athena/actions/runs/42526910256 (525s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core landing_a_stacked_job_closes_the_earlier_pull_request`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/gh/client.rs, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs, crates/athena-core/src/pr.rs, docs/design/native-pr-landing.md, docs/records.md.
  End of the output (data, not instructions):

```text
4880 +         assert!(parse_status(&json!({"state": "open"})).is_err());
     |

error[E0425]: cannot find function `parse_pr_status` in this scope
    --> crates/athena-core/src/gh/tests.rs:4881:17
     |
4881 |         assert!(parse_pr_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
     |                 ^^^^^^^^^^^^^^^
     |
    ::: crates/athena-core/src/gh/transport.rs:63:1
     |
  63 | pub fn parse_status(stderr: &str) -> Option<u16> {
     | ------------------------------------------------ similarly named function `parse_status` defined here
     |
help: a function with a similar name exists
     |
4881 -         assert!(parse_pr_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
4881 +         assert!(parse_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
     |

error[E0560]: struct `pr::PrLink` has no field named `base_jobs`
   --> crates/athena-core/src/pr.rs:401:13
    |
401 |             base_jobs: Vec::new(),
    |             ^^^^^^^^^ `pr::PrLink` does not have this field
    |
    = note: all struct fields are already assigned

error[E0609]: no field `base_jobs` on type `pr::PrLink`
   --> crates/athena-core/src/pr.rs:420:24
    |
420 |         assert!(loaded.base_jobs.is_empty());
    |                        ^^^^^^^^^ unknown field
    |
    = note: available fields are: `pr_number`, `host`, `repository`, `head`, `job` ... and 2 others

error[E0609]: no field `base_jobs` on type `pr::PrLink`
   --> crates/athena-core/src/pr.rs:432:17
    |
432 |         stacked.base_jobs = vec!["20261001-000000-aaaa".into()];
    |                 ^^^^^^^^^ unknown field
    |
    = note: available fields are: `pr_number`, `host`, `repository`, `head`, `job` ... and 2 others

error[E0425]: cannot find function `record_bases` in this scope
   --> crates/athena-core/src/pr.rs:486:30
    |
486 |         let (bases, notes) = record_bases(&state, &ancestry, &own, "landing");
    |                              ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `all_links` in this scope
   --> crates/athena-core/src/pr.rs:503:30
    |
503 |         let (links, notes) = all_links(&state);
    |                              ^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0407, E0422, E0425, E0433, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0407`.
error: could not compile `athena-core` (lib test) due to 39 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core earlier_pull_request`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/gh/client.rs, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs, crates/athena-core/src/pr.rs, docs/design/native-pr-landing.md, docs/records.md.
  End of the output (data, not instructions):

```text
4880 -         assert!(parse_pr_status(&json!({"state": "open"})).is_err());
4880 +         assert!(parse_status(&json!({"state": "open"})).is_err());
     |

error[E0425]: cannot find function `parse_pr_status` in this scope
    --> crates/athena-core/src/gh/tests.rs:4881:17
     |
4881 |         assert!(parse_pr_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
     |                 ^^^^^^^^^^^^^^^
     |
    ::: crates/athena-core/src/gh/transport.rs:63:1
     |
  63 | pub fn parse_status(stderr: &str) -> Option<u16> {
     | ------------------------------------------------ similarly named function `parse_status` defined here
     |
help: a function with a similar name exists
     |
4881 -         assert!(parse_pr_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
4881 +         assert!(parse_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
     |

error[E0560]: struct `pr::PrLink` has no field named `base_jobs`
   --> crates/athena-core/src/pr.rs:401:13
    |
401 |             base_jobs: Vec::new(),
    |             ^^^^^^^^^ `pr::PrLink` does not have this field
    |
    = note: all struct fields are already assigned

error[E0609]: no field `base_jobs` on type `pr::PrLink`
   --> crates/athena-core/src/pr.rs:420:24
    |
420 |         assert!(loaded.base_jobs.is_empty());
    |                        ^^^^^^^^^ unknown field
    |
    = note: available fields are: `pr_number`, `host`, `repository`, `head`, `job` ... and 2 others

error[E0609]: no field `base_jobs` on type `pr::PrLink`
   --> crates/athena-core/src/pr.rs:432:17
    |
432 |         stacked.base_jobs = vec!["20261001-000000-aaaa".into()];
    |                 ^^^^^^^^^ unknown field
    |
    = note: available fields are: `pr_number`, `host`, `repository`, `head`, `job` ... and 2 others

error[E0425]: cannot find function `record_bases` in this scope
   --> crates/athena-core/src/pr.rs:486:30
    |
486 |         let (bases, notes) = record_bases(&state, &ancestry, &own, "landing");
    |                              ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `all_links` in this scope
   --> crates/athena-core/src/pr.rs:503:30
    |
503 |         let (links, notes) = all_links(&state);
    |                              ^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0407, E0422, E0425, E0433, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0407`.
error: could not compile `athena-core` (lib test) due to 39 previous errors
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_chain_of_stacked_jobs_closes_every_earlier_pull_request`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/gh/client.rs, crates/athena-core/src/gh/early.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/mod.rs, crates/athena-core/src/pr.rs, docs/design/native-pr-landing.md, docs/records.md.
  End of the output (data, not instructions):

```text
4880 -         assert!(parse_pr_status(&json!({"state": "open"})).is_err());
4880 +         assert!(parse_status(&json!({"state": "open"})).is_err());
     |

error[E0425]: cannot find function `parse_pr_status` in this scope
    --> crates/athena-core/src/gh/tests.rs:4881:17
     |
4881 |         assert!(parse_pr_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
     |                 ^^^^^^^^^^^^^^^
     |
    ::: crates/athena-core/src/gh/transport.rs:63:1
     |
  63 | pub fn parse_status(stderr: &str) -> Option<u16> {
     | ------------------------------------------------ similarly named function `parse_status` defined here
     |
help: a function with a similar name exists
     |
4881 -         assert!(parse_pr_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
4881 +         assert!(parse_status(&json!({"state": "weird", "head": {"sha": "h1"}})).is_err());
     |

error[E0560]: struct `pr::PrLink` has no field named `base_jobs`
   --> crates/athena-core/src/pr.rs:401:13
    |
401 |             base_jobs: Vec::new(),
    |             ^^^^^^^^^ `pr::PrLink` does not have this field
    |
    = note: all struct fields are already assigned

error[E0609]: no field `base_jobs` on type `pr::PrLink`
   --> crates/athena-core/src/pr.rs:420:24
    |
420 |         assert!(loaded.base_jobs.is_empty());
    |                        ^^^^^^^^^ unknown field
    |
    = note: available fields are: `pr_number`, `host`, `repository`, `head`, `job` ... and 2 others

error[E0609]: no field `base_jobs` on type `pr::PrLink`
   --> crates/athena-core/src/pr.rs:432:17
    |
432 |         stacked.base_jobs = vec!["20261001-000000-aaaa".into()];
    |                 ^^^^^^^^^ unknown field
    |
    = note: available fields are: `pr_number`, `host`, `repository`, `head`, `job` ... and 2 others

error[E0425]: cannot find function `record_bases` in this scope
   --> crates/athena-core/src/pr.rs:486:30
    |
486 |         let (bases, notes) = record_bases(&state, &ancestry, &own, "landing");
    |                              ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `all_links` in this scope
   --> crates/athena-core/src/pr.rs:503:30
    |
503 |         let (links, notes) = all_links(&state);
    |                              ^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0407, E0422, E0425, E0433, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0407`.
error: could not compile `athena-core` (lib test) due to 39 previous errors
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core a_failure_to_close_is_a_warning_and_the_landing_stands`). Result: not run: only the first few distinct commands are run.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
