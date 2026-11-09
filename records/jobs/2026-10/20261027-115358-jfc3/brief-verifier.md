# Job 20261027-115358-jfc3: verify "Keep working branch append-only after landing; push it after each"

## Request (verbatim)

> Priority: high
> After a landing, Athena resyncs the working branch onto the merged commit by rewriting it (resync_onto_merge in the gh module), so the local branch and its remote copy diverge after every landing (2026-10-27: ahead 2, behind 46 during a drain) and a plain push is refused; the hand-run landing script used to force-push, and athena drain does not push at all. Keep the working branch's history append-only: bring the merged main in with a merge commit instead of rewriting, and have athena drain and athena land --pr push the working branch after each landing with a plain push.
> Acceptance:
> - A test lands a job on a branch that has a remote copy and shows the branch is a descendant of both its previous remote head and the merged main afterwards, so a plain push succeeds.
> - A drain test shows the working branch is pushed after each landing.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/gh/mod.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Replace the rewrite in resync_onto_merge/resync_tip with a merge commit of the merged main into the working branch, so the branch stays a descendant of its remote head. Then have athena land --pr and athena drain plain-push the working branch after each landing. Audience: the maintainer who runs drains and sees a refused push.

Assumptions:
- Code survey: read crates/athena-core/src/land.rs (resync_onto_merge at ~311 and resync_tip at ~218, which calls git.replay to rewrite the branch and then reset_keep), crates/athena-core/src/gh/landing.rs (after_merge ~959 calls resync_onto_merge after a landed PR), crates/athena-git/src/lib.rs (push_branch is a plain non-forced push; push_if_remote_at is a guarded force; reset_keep, merge_tree, is_ancestor exist) and crates/athena-cli/src/drain.rs (PrLander::land ~3256 calls athena.land_pr; no push after landing). The request names gh/mod.rs, but the rewrite lives in land.rs, so land.rs is the file to change; gh/landing.rs is the likely place for the push after after_merge.
- Existing callers and tests of resync_onto_merge/resync_tip (crates/athena-cli/tests/trailer_stop.rs, crates/athena-core/src/test_support.rs, crates/athena-core/src/gh/tests.rs rebase_tests) assert the rewritten shape and will need updating to the merge shape.
- Environment: git remote (origin, GitHub) write access to push the working branch; tests use local bare-repo remotes and fake hosts, no network. The push is plain (no force); a refused push is reported as a warning and does not undo the landing.
- The merge commit has the landed branch as first parent and the merged main as second parent, is signed when the repo signs commits, and leaves local edits alone as reset_keep did; a conflict is an error and the branch is unchanged.
- The no-op cases stay: if merged is already an ancestor of head, or landed is not on head, nothing moves.
- No config changes: athena.toml and defaults.toml are not touched.

## Acceptance criteria

1. [behavior] A test in crates/athena-core (run: cargo test -p athena-core) lands a job on a branch with a bare-repo remote copy, resyncs onto a squash-merged main, and asserts the branch is a descendant of both its previous remote head and the merged commit (git is_ancestor both ways true), and that a plain push (push_branch) then succeeds.
2. [behavior] A drain test in crates/athena-cli/src/drain.rs (run: cargo test -p athena-cli drain) shows the working branch is pushed after each landing, once per landed job, with a plain push; a failed push is shown as a warning and the landing stays landed.
3. [content] resync_tip/resync_onto_merge in crates/athena-core/src/land.rs no longer call git.replay or otherwise rewrite existing branch commits; the diff shows a merge commit instead, and no force or push_if_remote_at is used for the working branch push (Grep for --force in the diff finds none).
4. [behavior] A test shows the unchanged no-op cases (merged already an ancestor of head; landed not on head) and a conflicting merge leaves the branch unchanged with an error (cargo test -p athena-core).
5. [behavior] athena land --pr pushes the working branch after the landing; a test covers it in crates/athena-core/src/gh/tests.rs or the CLI tests.
6. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
7. [content] CHANGELOG.md has a new entry describing the append-only resync and the push after each landing.
8. [content] Docs and doc comments that describe the branch being rebuilt/rewritten (resync_onto_merge doc, after_merge comment) are updated to describe the merge commit.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-cli/tests/trailer_stop.rs (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Changing the hand-run landing script
- Changing how the PR/squash merge itself is made on the host
- Force-pushing or push_if_remote_at for the working branch
- athena.toml, defaults.toml and workflow changes

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the landing and drain and check ancestry and push outcome.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Stops the history rewrite itself instead of force-pushing around it.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name commands the verifier runs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG describe the new behavior.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A land.rs unit test on real git with a real bare repository. Build the landing scenario on the branch (job A landed, later job B, a hand edit). Run `git init --bare` in a tempdir, add it as `origin` and push the branch plainly, so the remote holds the previous head. Build the host squash as the existing resync test does (squash tree equal to the landed tree, parent = base). Call resync_tip and move the branch with reset_keep. Contrast in the same test: `git.replay(squash, landed, head, false)` gives a commit that is not a descendant of the previous remote head, and push_branch of it errors. Run: `cargo test -p athena-core after_a_squash_the_branch_is_merged_with_it_and_a_plain_push_succeeds`. Observes: is_ancestor(previous remote head, tip) and is_ancestor(squash, tip) are both true; git.push_branch("origin", branch, &tip) succeeds; the bare repository's branch equals tip; the replay contrast is refused.
  Fails without the behavior: With the replay-based resync_tip the tip has one parent and is not a descendant of the previous remote head, so the is_ancestor(previous remote head, tip) assertion fails and the plain push is refused.
  Test seam: Entry point athena_core::land::resync_tip (exists, land.rs:218) with its two no-op guards (merged already an ancestor of head; landed not on head). Fixture: the land.rs test helpers `repo()` (line 470) and `commit_message` (518), which exist, plus a bare repository made in the test with git init --bare and attached with `git remote add origin`; step 9 adds a small bare-repo helper. Nothing is faked; real git.
- Criterion 2: Three layers, one push per landing. (a) drain.rs unit test over the public `drain` entry point with the in-crate Stub: two Landed jobs give calls `[.., land job-a, push, .., land job-b, push]`; a rejected job gives no `push`. (b) tests/drain.rs drives `drain::drain` through RealSteps with a fake Lander on the existing Fixture. The test first runs `git checkout -b work` in the fixture (the Fixture checks out main, which the helper never pushes), adds a bare origin, pushes `work` once, queues two steward requests with distinct keys, and uses a lander that records at each ask the origin's `work` sha and the local head. It asserts: at the second ask the origin's `work` holds the head recorded at the first ask; after the drain the origin's `work` is an ancestor of the local head and holds the second landing's head; report.landed == 2. (c) A test with a lander that overrides `pushes_working_branch` to true: the origin's branch stays at its earlier sha, so the hook does not push twice; plus a unit test that `PrLander.pushes_working_branch()` is true. (d) A refused-push test: the origin first receives a sibling commit made with `git commit-tree` on the base commit and a plain push to the empty bare repo, so origin's `work` is not an ancestor of the local head. The drain shows report.landed == 1, landing text `landed`, the origin's `work` unchanged, and a captured Ui line starting `warning: could not push`. (e) Core tests of Athena::push_working_branch: a bare origin is pushed once and the second call is silent; an origin at a missing path warns and returns false; no origin and a detached HEAD are silent; a checkout on `main` is not pushed and prints the default-branch line; a checkout on a branch that origin/HEAD names as default is not pushed. Run: `cargo test -p athena-cli the_drain_pushes_the_working_branch_after_each_landing && cargo test -p athena-cli --test drain the_real_steps_push_the_working_branch_after_each_landing && cargo test -p athena-cli --test drain a_lander_that_pushes_itself_is_not_pushed_for_again && cargo test -p athena-cli --test drain a_refused_push_is_a_warning_and_the_job_stays_landed && cargo test -p athena-core push_working_branch`. Observes: (a) call order with one push per landing, none for a rejected job. (b) the origin's branch follows the local head after each landing. (c) no second push for a self-pushing lander. (d) a refused plain push is a warning and the job stays landed. (e) bare origin equals head after one push; the warning text for a missing origin; the default branch is never pushed.
  Fails without the behavior: Without the Drain::land call, Stub.calls has no `push` and the order assertion fails. Without the RealSteps override, the bare origin's branch stays at the earlier head and the ancestor assertions fail. Without the warning branch, test (d) finds no `warning: could not push` line. If the push were forced, the divergent origin in (d) would be overwritten and its tip would change. Without the default-branch skip, the main-checkout test finds main pushed.
  Test seam: Entry points drain::drain -> Drain::land (exists, drain.rs:1785; step 7 adds the hook call) -> RealSteps::push_after_landing (step 7) -> Athena::push_working_branch (step 5). Doubles: the in-crate Stub (exists, drain.rs:4684; step 7 adds a push_after_landing that only records "push"). In tests/drain.rs the Lander fake follows the RecordingLander pattern (exists, line 2291) with a probe that reads the bare origin's branch through git and holds no push logic. The bare origin and the sibling commit are made in the tests with git commands through the fixture's `fx.git`. RealSteps, Fixture, queue_steward_request, rated_scenario and limits exist in tests/drain.rs; step 8 adds the keyed queue helper and a line-collecting Ui if none exists. Core tests use `set_up()` from code_health::tests (exists).
- Criterion 3: A unit test on real git: after resync_tip on the criterion 1 scenario, assert git.parents(tip) == [head, merged], every commit of the old branch is an ancestor of tip, the merge message carries no `Athena-` trailer, and tree_of(tip) equals tree_of(head). Then review the diff for forced pushes and replay calls. Run: `cargo test -p athena-core resync_tip_adds_a_merge_commit_and_rewrites_nothing`. Observes: Parents [head, merged]; all old commits are ancestors. The diff of land.rs, gh/landing.rs and drain.rs (`git diff -- crates/athena-core/src/land.rs crates/athena-core/src/gh/landing.rs crates/athena-cli/src/drain.rs`) shows resync_tip calling merge_tree and commit_tree, no `replay(` call, no `--force`, no `push_if_remote_at`, and only GitRemote::push for the working branch.
  Fails without the behavior: With replay the tip has a single parent and the old commits are not ancestors, so the parents and ancestry assertions fail.
  Test seam: Entry point resync_tip (exists, land.rs:218). Same real-git fixtures as criterion 1 (repo(), commit_message exist). The diff review needs no seam.
- Criterion 4: land.rs unit tests on real repositories: (a) merged already an ancestor of head (a second run, and an older main commit as `merged`) returns None; (b) landed not on head returns None; (c) job B edits the line job A landed and the squash differs there: resync_tip errors with text containing `conflict` and head is unchanged; (d) the host's tree carries a governed file the branch lacks (main moved while the up-to-date rule was off): resync_tip errors naming the path and head is unchanged; (e) Athena::resync_onto_merge on case (c) returns an error saying the branch is unchanged and athena.head() is the same; (f) a hand-made merge of head with an older main commit that takes the older version of a governed file goes through land::classify_commit and is refused as GovernedChange. Run: `cargo test -p athena-core resync`. Observes: None for (a) and (b); errors for (c) and (d) with head the same sha before and after; the unchanged-branch text for (e); the GovernedChange refusal for (f).
  Fails without the behavior: A resync that moved the branch on conflict or on a governed difference, or that created a merge in the no-op cases, would change head or return Some/Ok, so the assertions fail. If the gate were loosened, (f) would see the revert accepted.
  Test seam: Entry points resync_tip (exists, land.rs:218), Athena::resync_onto_merge (exists, land.rs:311) with `set_up()` from code_health::tests, and land::classify_commit (exists, land.rs:284). Real temporary repositories, no doubles. The governed-fingerprint check is added to resync_tip in step 1.
- Criterion 5: gh/tests.rs tests through Athena::after_merge, which land_pr calls at landing.rs:952 for both `athena land --pr` and PrLander. `after_a_merge_the_working_branch_is_pushed`: set_up() Athena on a branch named `work`, a bare `origin` with the branch pushed, a landed commit made with the `land` helper, the host squash built as the existing after_merge test does, then after_merge with a Landed report. Further tests: a report that is not Landed pushes nothing and logs nothing; an origin at a missing path gives a `warning:` line and after_merge still returns Ok; a checkout on `main` is not pushed. `landing_records_leave_out_landed_commits` on the landing-world fixture near gh/tests.rs:3782-4051: a landing is recorded for job A, the host's commit is on the base, a resync merge follows, then job B lands; the second intent and record list only the merge (RecordsOnly) and job B's commit. A land.rs realistic-branch test (intervention commit, records-only commit, job A landing, squash, resync, job B landing) checks that pr_check::check_pr_head(git, merged, tip) has no offence, every commit of merged..tip classifies, and pr_commits_excluding leaves the merge and job B. A land.rs test shows record_bases still names an open earlier link whose head is job A's landed commit after the resync (the replay contrast: not an ancestor), and the close_carried_prs comment at landing.rs:376-377 is updated. Run: `cargo test -p athena-core after_a_merge_the_working_branch_is_pushed && cargo test -p athena-core after_merge && cargo test -p athena-core landing_records_leave_out_landed_commits && cargo test -p athena-core realistic_branch && cargo test -p athena-core open_earlier_pull_request`. Observes: The bare repository's branch equals athena.head() after after_merge, and the earlier remote head is its ancestor; the Lines log shows a `pushed` line after the `merged` line. A not-landed report leaves the bare branch where it was. A missing origin gives a warning and Ok. The main checkout is untouched. The second landing lists only job B's commits. The realistic branch has no offence. The earlier link is a base.
  Fails without the behavior: Without the push call in after_merge, the bare branch stays at the earlier head and the equality fails. Without the record-based exclusion, the second landing intent also names job A. Without the merge, the earlier link's head is not an ancestor.
  Test seam: Entry points Athena::after_merge (pub(crate), exists, landing.rs:959), GitLandingRecords (exists, gh/records.rs), land::pr_commits_excluding (step 3), pr::record_bases (exists, pr.rs:190). Fixtures: set_up()/land() (exist), a bare origin built in the test, a `NoPages` PagesHost double added in step 10 (answers no runs, holds no landing logic), and the existing `Lines` Ui collector (gh/tests.rs:3505). `athena land --pr` itself needs the real gh (commands.rs:284-316) and is not run; it reaches the same land_pr -> after_merge path.
- Criterion 6: Run the three checks Athena runs, then each affected crate separately: athena-core (calls classify_pre_merge and resync), athena-cli, athena-gate (new pin test), athena-git (keeps replay). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test -p athena-core && cargo test -p athena-cli && cargo test -p athena-gate && cargo test -p athena-git`. Observes: fmt and clippy report nothing; all tests pass, including the gate pin test that documents the pre-cutover limit.
  Fails without the behavior: If the two checkout fixtures still called the merge-making resync, the attribution and history suites that use them would fail on a cutover commit off main's first-parent line. If the after_merge needle were not updated it would fail on the new message. If Drain::land did not call the hook, the drain tests fail. Unused imports from the removed replay call would fail clippy -D warnings.
  Test seam: Entry points are the cargo commands over the workspace (exist). Fixtures are the repository's own suites with temporary git repositories, bare remotes and the in-crate Stub; the fixture edits are in steps 8 to 11.
- Criterion 7: Read the CHANGELOG diff produced by step 12. Run: `git diff -- CHANGELOG.md`. Observes: A new bullet under Unreleased says: the post-landing resync merges the host's commit into the working branch instead of rewriting it; `athena land --pr` and `athena drain` push the working branch after each landing with a plain push (a refusal is a warning and the landing stands; the default branch is never pushed; the push rule now covers the working branch as well as the job's own branch); landing records leave out commits already landed; an open earlier pull request below a later landing is now carried and closed; the root cause (the replay rewrote commit ids, so branch and remote copy diverged); the known limit that the history check before the cutover refuses a head holding a landed job and its squash, with the follow-up; and that the pull-request range grows with each landing, with a pointer to per-job branches (D167).
- Criterion 8: Read the doc-comment diff and search the changed files for the old wording (`rebuilt`, `replayed`, `rebuild`). Run: `git diff -- crates/athena-core/src/land.rs crates/athena-core/src/gh/landing.rs crates/athena-core/src/test_support.rs`. Observes: The resync_tip, resync_onto_merge, pr_commits and after_merge doc comments, the after_merge inline comment, the close_carried_prs comment (landing.rs:376-377) and the three test_support.rs comments (lines 26, 46 and 616) describe the merge commit and the push; no comment still says the branch is rebuilt or replayed. docs/design/native-pr-landing.md section 5.2a, D88 and docs/enforcement.md (which says Athena does not push) are named as follow-ups, not edited.

Planned files:

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/records.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/after_merge_tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/tests/trailer_stop.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 99e8171be89e → candidate f9d1b7a440a2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                   |  29 ++
 crates/athena-cli/src/drain.rs                 |  68 ++++-
 crates/athena-cli/tests/drain.rs               | 162 ++++++++++
 crates/athena-cli/tests/trailer_stop.rs        |   9 +-
 crates/athena-core/src/gh/after_merge_tests.rs | 219 +++++++++++++-
 crates/athena-core/src/gh/landing.rs           |  15 +-
 crates/athena-core/src/gh/records.rs           |  36 ++-
 crates/athena-core/src/land.rs                 | 396 ++++++++++++++++++++-----
 crates/athena-core/src/test_support.rs         |  19 +-
 crates/athena-gate/src/landing_tests.rs        |  45 +++
 10 files changed, 905 insertions(+), 93 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-115358-jfc3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-115358-jfc3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42557910068 for f9d1b7a440a218c2381466329f9f022e507c3f32)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42557910068 for f9d1b7a440a218c2381466329f9f022e507c3f32)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42557910068 for f9d1b7a440a218c2381466329f9f022e507c3f32)
- ci-result: `not a check` passed (Check results for f9d1b7a440a218c2381466329f9f022e507c3f32 come from the CI run https://github.com/doodla/athena/actions/runs/42557910068 (669s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core after_a_squash_the_branch_is_merged_with_it_and_a_plain_push_succeeds`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/records.rs, crates/athena-core/src/land.rs.
  End of the output (data, not instructions):

```text
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `push_working_branch` found for struct `Athena` in the current scope
   --> crates/athena-core/src/gh/after_merge_tests.rs:292:20
    |
292 |     assert!(athena.push_working_branch(&ui));
    |                    ^^^^^^^^^^^^^^^^^^^
    |
   ::: crates/athena-core/src/lib.rs:196:1
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `push_working_branch` found for struct `Athena` in the current scope
   --> crates/athena-core/src/gh/after_merge_tests.rs:316:21
    |
316 |     assert!(!athena.push_working_branch(&ui));
    |                     ^^^^^^^^^^^^^^^^^^^
    |
   ::: crates/athena-core/src/lib.rs:196:1
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0425]: cannot find function `pr_commits_excluding` in this scope
   --> crates/athena-core/src/land.rs:665:21
    |
665 |         let empty = pr_commits_excluding(&git, &start, &landed, std::slice::from_ref(&landed))
    |                     ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `pr_commits_excluding` in this scope
   --> crates/athena-core/src/land.rs:671:23
    |
671 |         let unknown = pr_commits_excluding(&git, &start, &landed, &[missing])
    |                       ^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 6 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-cli the_drain_pushes_the_working_branch_after_each_landing && cargo test -p athena-cli --test drain the_real_steps_push_the_working_branch_after_each_landing && cargo test -p athena-cli --test drain a_lander_that_pushes_itself_is_not_pushed_for_again && cargo test -p athena-cli --test drain a_refused_push_is_a_warning_and_the_job_stays_landed && cargo test -p athena-core push_working_branch`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core resync_tip_adds_a_merge_commit_and_rewrites_nothing`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/records.rs, crates/athena-core/src/land.rs.
  End of the output (data, not instructions):

```text
   ::: crates/athena-core/src/lib.rs:196:1
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `push_working_branch` found for struct `Athena` in the current scope
   --> crates/athena-core/src/gh/after_merge_tests.rs:292:20
    |
292 |     assert!(athena.push_working_branch(&ui));
    |                    ^^^^^^^^^^^^^^^^^^^
    |
   ::: crates/athena-core/src/lib.rs:196:1
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `push_working_branch` found for struct `Athena` in the current scope
   --> crates/athena-core/src/gh/after_merge_tests.rs:316:21
    |
316 |     assert!(!athena.push_working_branch(&ui));
    |                     ^^^^^^^^^^^^^^^^^^^
    |
   ::: crates/athena-core/src/lib.rs:196:1
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0425]: cannot find function `pr_commits_excluding` in this scope
   --> crates/athena-core/src/land.rs:665:21
    |
665 |         let empty = pr_commits_excluding(&git, &start, &landed, std::slice::from_ref(&landed))
    |                     ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `pr_commits_excluding` in this scope
   --> crates/athena-core/src/land.rs:671:23
    |
671 |         let unknown = pr_commits_excluding(&git, &start, &landed, &[missing])
    |                       ^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 6 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core resync`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/records.rs, crates/athena-core/src/land.rs.
  End of the output (data, not instructions):

```text
   ::: crates/athena-core/src/lib.rs:196:1
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `push_working_branch` found for struct `Athena` in the current scope
   --> crates/athena-core/src/gh/after_merge_tests.rs:292:20
    |
292 |     assert!(athena.push_working_branch(&ui));
    |                    ^^^^^^^^^^^^^^^^^^^
    |
   ::: crates/athena-core/src/lib.rs:196:1
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `push_working_branch` found for struct `Athena` in the current scope
   --> crates/athena-core/src/gh/after_merge_tests.rs:316:21
    |
316 |     assert!(!athena.push_working_branch(&ui));
    |                     ^^^^^^^^^^^^^^^^^^^
    |
   ::: crates/athena-core/src/lib.rs:196:1
    |
196 | pub struct Athena {
    | ----------------- method `push_working_branch` not found for this struct
    |
help: there is a method `push_ci_branch` with a similar name, but with different arguments
   --> crates/athena-core/src/gh/early.rs:384:5
    |
384 |     pub(crate) fn push_ci_branch(&self, job: &JobId, commit: &str, ui: &dyn Ui) {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0425]: cannot find function `pr_commits_excluding` in this scope
   --> crates/athena-core/src/land.rs:665:21
    |
665 |         let empty = pr_commits_excluding(&git, &start, &landed, std::slice::from_ref(&landed))
    |                     ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `pr_commits_excluding` in this scope
   --> crates/athena-core/src/land.rs:671:23
    |
671 |         let unknown = pr_commits_excluding(&git, &start, &landed, &[missing])
    |                       ^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 6 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core after_a_merge_the_working_branch_is_pushed && cargo test -p athena-core after_merge && cargo test -p athena-core landing_records_leave_out_landed_commits && cargo test -p athena-core realistic_branch && cargo test -p athena-core open_earlier_pull_request`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test -p athena-core && cargo test -p athena-cli && cargo test -p athena-gate && cargo test -p athena-git`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
