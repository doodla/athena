# Job 20261106-041052-wzy7: verify "Let athena retry re-verify a stale job's kept candidate on the new tip"

## Request (verbatim)

> Let a stale job be tried again on the current branch tip, keeping its candidate.
> 
> With two jobs running at once, a job that passed checks, replay and verification can go stale because the other job landed first on content it governs. Today a stale job can only be asked again from scratch, which repeats the whole producer run. docs/decisions.md says automatic re-verification on the new base was left out until stale landings recur; they now recur with parallel jobs.
> 
> athena retry accepts a stale job. The retry merges the kept candidate onto the current branch tip. If the merge is clean, it re-runs checks, replay and verification on the merged result and lands it as usual; it does not re-run the producer. If the merge conflicts, the retry runs the producer from the kept candidate as a normal retry does.
> 
> Acceptance:
> - athena retry on a stale job no longer errors
> - a test: a candidate that merges cleanly onto a moved branch is re-checked and re-verified on the merged tree and lands, with no producer call recorded
> - a test: a candidate that conflicts with the moved branch goes to a producer retry
> - the landing still requires the verified fingerprint to match what lands (re-verification produces the new fingerprint)
> - docs/decisions.md records the change as reversing the left-out alternative (d), with the reason
> - CHANGELOG states only what the diff does
> 
> Files changed: the retry command, the landing path for stale jobs, tests, docs/decisions.md, CHANGELOG.md.

## Reading (accepted by the user)

Make `athena retry` accept a stale job. It merges the kept candidate onto the current branch tip. A clean merge re-runs checks, replay and verification on the merged tree and lands it with the new fingerprint, with no producer call. A conflict starts a producer retry seeded from the kept candidate. Audience: users running parallel jobs, and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (Athena::retry at ~1103 calls ask with 'try again on <id>'; qualifies_for_tree_retry and tree_retry_seed at ~1645-1758 only accept Rejected jobs, so a Stale job is not seeded; resumable_job/retries_by_job treat Stale as an unrun state), crates/athena-core/src/land.rs (plan() returns Plan::Stale/Tree/OtherBranch; landing requires P ancestor of H, same classification, and Verified fingerprint match), and crates/athena-core/src/run/close.rs (Stale outcome written at ~289-312; retry_landing at ~451 re-runs plan on a blocked job and bails on Stale). Behavior is measured by existing run tests (crates/athena-core/src/run/tests.rs) and CLI tests (crates/athena-cli/tests/lifecycle.rs, drain.rs) using a fake producer whose calls are counted.
- Existing code: the records for a stale job already hold its candidate commit and base; the new path reuses plan(), Verified::from_record, fingerprint, the tree-retry seeding in reading.rs and the checks/replay/verifier stages in crates/athena-core/src/run/. The merge uses the git layer's merge-tree in crates/athena-git; no new crate is added (buy-first).
- Where ask refuses non-retryable states, the Stale state is added to the accepted ones. The merge-clean/conflict decision is made once on the current tip, before any producer call; the merged tree gets a fresh candidate and a fresh verification record, and the fingerprint is recomputed, so the landing check still compares verified to landed content.
- No new record type or schema change is expected; existing candidate, verification and outcome records are reused (so the records allow-list and AnyRecord need no change). If one is needed, add it to AnyRecord with a round-trip test.
- This adds no refusal rule; it removes one (stale retry no longer errors). No replay refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows) are not changed.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal instead of an error (the stale-state refusal in ask/retry is gone).
2. [behavior] Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and the producer call count recorded for the retry is zero.
3. [behavior] Run `cargo test`: a test where the candidate conflicts with the moved branch shows the retry goes to a producer retry seeded from the kept candidate (one producer call recorded, nothing lands without a new verification).
4. [behavior] Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree (not the original candidate's), and a landing whose verified fingerprint differs from the landed content is still refused.
5. [behavior] Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass, including existing land.rs tests for Plan::Stale.
6. [content] docs/decisions.md records a new decision (or a revision) stating that it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of the landing decision (the one at ~line 292), with the reason that stale landings now recur with parallel jobs, and the original line 297 is annotated as reversed.
7. [content] CHANGELOG.md has an entry describing only what the diff does: athena retry accepts a stale job, merges its candidate onto the tip, re-verifies and lands on a clean merge without a producer run, and runs the producer from the candidate on conflict.
8. [content] Only the retry command, the stale landing path, tests, docs/decisions.md and CHANGELOG.md change; no secrets or personal data are committed.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Automatic (drain-triggered) re-verification of stale jobs without an explicit athena retry
- Changing the staleness rules in land.rs plan() (what makes a job stale)
- Retrying other non-retryable states (landed, dropped, abandoned)
- Prompt files and governance configuration

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for clean-merge landing and conflict retry; cargo test and clippy pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test as the way to exercise it.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the producer call count, the landing and the fingerprint, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The stale retry path is changed in the retry and landing code, not by a workaround.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the listed files.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a stale job lands re-verified without a producer run.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The fingerprint-match landing rule stays enforced by code and is tested.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: End-to-end test against the real `Athena::retry`: a job really goes stale (hook style of `a_governed_change_on_the_branch_makes_the_job_stale`, lifecycle.rs:2027-2069), then `retry` on it. Plus reading.rs unit tests on `earlier_attempt_with`/`earlier_attempt`, and a second lifecycle test that has the hook check out a branch holding the records (OtherBranch route, close.rs 273-289; lifecycle.rs:1995 shows the checkout) so `retry` and `go` run on the other branch. Run: `cargo test -p athena-cli --test lifecycle retry_accepts_a_stale_job && cargo test -p athena-core --lib reading::tests::a_stale_job_can_be_tried_again && cargo test -p athena-cli --test lifecycle a_stale_job_from_another_branch_is_retried_from_scratch`. Observes: `athena.retry(stale_job, &Lines::default())` returns Ok with a Proposal whose request is `try again on <stale job>`. In reading.rs `earlier_attempt` on a Stale job returns Some, and Landed and Abandoned are still refused with the 'only a rejected or failed job' prefix. In the other-branch test exactly one Producer entry is recorded after the stale close, the Reader/Producer/Verifier order is that of an ordinary retry, and the merged candidate is not landed.
  Fails without the behavior: `earlier_attempt_with` (reading.rs:1362-1374) bails 'job .. is stale; only a rejected or failed job can be tried again', so `retry` returns Err and the unwrap fails. Without the branch gate, the other-branch test sees zero Producer entries.
  Test seam: Production entry points `Athena::retry` (reading.rs ~1103) and `earlier_attempt_with`. Fixture `Fixture::new()`, `Scenario::default().script()` and the `during_verification` hook exist (tests/common/mod.rs:428, lifecycle.rs:2027); reading.rs helpers `records_with` / `job_with_checks` exist (~4636). New tests are added in steps 4 and 11; git is real, nothing under test is mocked.
- Criterion 2: End-to-end test. Job A (default scenario, candidate changes README.md) goes stale because the hook commits a governed change to src/lib.rs plus an intervention record during A's verification. Then `retry`, then `go` on the retry job. Count roles in `Scenario.invocations` between the end of A and the end of the retry, load the retry job's records with `Records::load`, and read `seen_by_verifier`. A second test, `..._already_on_the_tip_...`, makes the hook also write A's exact README.md text so the merge adds nothing. Run: `cargo test -p athena-cli --test lifecycle a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_run && cargo test -p athena-cli --test lifecycle a_stale_job_whose_candidate_is_already_on_the_tip_fails_without_a_producer_run`. Observes: Clean test: the retry job finishes Landed. `invocations` gains no `Role::Producer` entry after A's close and gains a `Role::Verifier` entry. The retry's Verification record has non-empty `checks`. A Replay record exists for the retry job; its `candidate_commit` equals the retry Candidate's `candidate_commit` (merged commit) and its `base_commit` equals that Candidate's `base_commit` (prediction commit). `seen_by_verifier` contains both A's README.md greeting and the hand edit to src/lib.rs. The landed tree holds both changes. Already-on-tip test: the retry job ends Failed with a reason containing 'already on the branch tip', no Producer or Verifier entry follows, and the tip's non-records tree is unchanged.
  Fails without the behavior: Without the clean-merge path the retry is an ordinary producer retry: a Producer entry is recorded and the zero-producer assertion fails; no Replay record exists on a merged candidate and the verifier does not see both edits. Without the three-way merge the hand change is overwritten or the job does not land. Without the equal-tree rule the merge gives an empty candidate or a producer retry runs.
  Test seam: Entry points `Athena::retry` then `Athena::go` (`dispatch`, mod.rs:823, then `resume_candidate`, mod.rs:522; replay in `check_and_verify`/`replay_stage`, ~1774). The fake driver `Scenario::script()` with `invocations: Arc<Mutex<Vec<Role>>>` and `seen_by_verifier` exist (common/mod.rs:430-433, push at 570, 632); the `during_verification` hook exists (common/mod.rs:428, 637); `Records::load` exists (lifecycle.rs:3607). The `stale_retry` step and the merged branch of `resume_candidate` are added in steps 6-8; the double implements no merge.
- Criterion 3: End-to-end tests with a wrapper `Script` around `Scenario::script()` that overrides `FakeTurn.edits` for both Producer calls. Call 1 edits README.md and src/lib.rs line 1; the hook lands a different src/lib.rs line 1 plus an intervention record during A's verification, so the merge conflicts on src/lib.rs only. Call 2 (the retry) reads `spec.workdir` into an `Arc<Mutex<..>>` before it edits, and resolves src/lib.rs. The unresolved variant makes no edit, or leaves the markers in place. Run: `cargo test -p athena-cli --test lifecycle a_stale_job_whose_candidate_conflicts_goes_to_a_producer_retry_seeded_from_it && cargo test -p athena-cli --test lifecycle a_conflict_retry_whose_producer_leaves_the_conflict_unresolved_fails_before_the_checks`. Observes: Seeded test: exactly one Producer entry for the retry job. At the start of that call src/lib.rs in the producer's workdir holds `<<<<<<<` markers and README.md already holds A's greeting (so the test cannot pass on the clean path, where no producer runs and no markers exist). The producer brief names src/lib.rs and the stale job. A Verifier entry has a higher index than that Producer entry, and the job lands with the README.md change and the resolved src/lib.rs. Unresolved test: the job ends Failed with a reason naming src/lib.rs, no Verifier entry follows the Producer entry, no checks are recorded, and the tip's non-records tree is unchanged.
  Fails without the behavior: A plain retry from the new base lacks the markers and the README.md change in the producer's workdir, so those assertions and the landed-README assertion fail. A clean-only implementation never reaches a producer. Without the gate, the unresolved seed goes to admission, the checks and the verifier (candidate.rs 134-150), a Verifier entry is recorded and the job is not failed.
  Test seam: Entry points `Athena::retry` and `Athena::go`. `merge_kept_candidate` returns Conflict, so `dispatch` sets `retry_seed`, `ctx.seed_commit` and `ctx.seeded_conflicts`, and `producer_snapshot` (producer.rs:116-125) takes the existing `SnapshotRepo::BaseThenTree` route. The gate is added in step 9. `Script` is the boxed closure over `&AgentInvocation` that `Scenario::script()` returns; `seen_by_producer` and `invocations` exist (common/mod.rs:431-433). The wrapper varies only producer edits and records inputs; the merge, seed and gate are production code. Unit tests in land.rs (`merge_kept_candidate`, `has_conflict_markers`) and athena-git (`merge_tree_report`, incl. a path with a space and a non-ASCII character) cover the pieces.
- Criterion 4: (a) Read the retry job's Verification and Candidate records after the landing in the clean test. (b) A land.rs unit test on the real `plan()` with a real git repository: after the base moves, `merge_kept_candidate` gives a clean tree; a commit on the new tip with that tree and a `Verified` built from the merged tree's fingerprint gives `Plan::Tree`; a `Verified` carrying the original candidate's fingerprint makes `plan()` return Err. (c) The existing differing-content refusal test. Run: `cargo test -p athena-cli --test lifecycle a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_run && cargo test -p athena-core --lib land::tests::a_kept_candidate_merged_onto_a_moved_tip && cargo test -p athena-core --lib land::tests::landing_refuses_content_that_differs_from_the_verified_content`. Observes: (a) The retry job's `verified_fingerprint` equals `athena_git::attrs::fingerprint(git, landed_commit, retry_candidate.base_commit)` and differs from the stale job's. (b) `Plan::Tree` with the merged-tree fingerprint; Err ('differs from the verified content') with the original candidate's. (c) The existing refusal still passes.
  Fails without the behavior: If the retry reused the original fingerprint, the equality bail at land.rs 136-142 refuses the landing and (b)'s second assertion shows the Err. If the landing skipped the check, that Err assertion fails.
  Test seam: Production `plan()` (land.rs:70) is called directly with a real `Verified` from the existing `pass()` helper and the `base_and_candidate()` / `commit()` fixtures in the land.rs tests, all existing. `merge_kept_candidate` is added in step 2.
- Criterion 5: Whole-workspace checks, including the existing land.rs Plan::Stale tests, the refusal corpus and the backtest. No double. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test -p athena-core --test refusal_corpus && cargo test -p athena-cli --test backtest`. Observes: All pass. `a_base_that_is_no_longer_an_ancestor_makes_the_job_stale` and `a_changed_exclusion_rule_makes_the_job_stale` still see Plan::Stale. Refusal corpus: the retry-related ids I read (`retry-complaint-reading-names-the-job` at refusal_corpus.rs:351, `retry-complaint-reading-omits-the-job` at :369) test request text, not job state; this change removes a job-state refusal and adds no request-text form, so no case is added and I cite no result for them.
  Fails without the behavior: Not a behavior of its own: a formatting, lint or regression failure in any touched file fails this.
  Test seam: Whole-workspace commands; nothing is mocked.
- Criterion 6: Content check of the diff. Run: `git diff -- docs/decisions.md`. Observes: A new D157 after D156 states that it reverses alternative (d) 'Automatic re-verification on the new base' of D10, with the reason that stale landings now recur with parallel jobs. It has Case, Decision, Alternatives, Tradeoff, Reversal cost and Provenance. It records the dispatch-branch and commit-exists conditions (no ancestry condition, and why), the already-on-tip failure, the conflict gate and that a failed conflict retry loses the kept candidate, the report-gaps note for the clean-merge verifier, the reader-brief wording misstatement for a stale retry, the shared-append-only-file conflict expectation, and what stays (the fingerprint rule, no automatic re-verification). Line 297 is still one line and ends with 'Reversed by D157: stale landings now recur with parallel jobs.'; line 293 is still one line and ends with the D157 note. No added line contains 'superseded by', 'supersedes' or 'amends' followed by a D-number. `cargo test -p athena-cli --test backtest` passes.
- Criterion 7: Content check of the diff. Run: `git diff -- CHANGELOG.md`. Observes: One bullet at the top of `[Unreleased]` / `### Added` says only what the diff does: `athena retry` accepts a stale job; with the dispatch branch checked out and the kept commits present it merges the candidate onto the tip; a clean merge re-runs checks, replay and verification and lands with a new fingerprint and no producer run; a merge that adds nothing fails the retry; a conflict starts a producer retry from the conflicted tree and fails before the checks if a conflicted path is unchanged or holds markers; a stale job from another checked-out branch, or with its kept commits gone, is retried from the base. It lists `Git::merge_tree_report`, names the root cause, cites D157 and the README lines.
- Criterion 8: Scope check of the final diff and the repository's policy scans. Run: `git status && cargo test -p xtask --test policy`. Observes: Only these paths changed: crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, reading.rs, run/mod.rs (and run/close.rs, run/tests.rs only if needed), crates/athena-cli/src/main.rs, crates/athena-cli/tests/lifecycle.rs, README.md (three lines), docs/decisions.md and CHANGELOG.md. athena.toml, defaults.toml, workflows, prompt files, docs/decision-links.toml, crates/athena-context, drain.rs, tests/drain.rs and display.rs are untouched. The xtask policy test (banned words, personal-data allow-list) passes.

Planned files:

- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- README.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 6e27b3fe0622 → candidate 09fc10a0e335. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  14 ++
 README.md                            |  11 +-
 crates/athena-cli/src/main.rs        |   7 +
 crates/athena-cli/tests/lifecycle.rs | 412 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/land.rs       | 127 +++++++++++
 crates/athena-core/src/reading.rs    | 163 +++++++++++++-
 crates/athena-core/src/run/mod.rs    | 261 ++++++++++++++++++++--
 crates/athena-git/src/lib.rs         | 117 ++++++++++
 docs/decisions.md                    |  71 ++++++
 9 files changed, 1152 insertions(+), 31 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-041052-wzy7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-041052-wzy7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42172124217 for 09fc10a0e335c18f1a7aabf5a8910679536ff1b3)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42172124217 for 09fc10a0e335c18f1a7aabf5a8910679536ff1b3)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42172124217 for 09fc10a0e335c18f1a7aabf5a8910679536ff1b3)
- ci-result: `not a check` passed (Check results for 09fc10a0e335c18f1a7aabf5a8910679536ff1b3 come from the CI run https://github.com/doodla/athena/actions/runs/42172124217 (692s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "docs/decisions.md records a new decision (or a revision) stating that it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of the landing decision (the one at ~line 292), with the reason that stale landings now recur with parallel jobs, and the original line 297 is annotated as reversed.")
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass, including existing land.rs tests for Plan::Stale.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-cli --test lifecycle retry_accepts_a_stale_job && cargo test -p athena-core --lib reading::tests::a_stale_job_can_be_tried_again && cargo test -p athena-cli --test lifecycle a_stale_job_from_another_branch_is_retried_from_scratch`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-cli --test lifecycle a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_run && cargo test -p athena-cli --test lifecycle a_stale_job_whose_candidate_is_already_on_the_tip_fails_without_a_producer_run`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-cli --test lifecycle a_stale_job_whose_candidate_conflicts_goes_to_a_producer_retry_seeded_from_it && cargo test -p athena-cli --test lifecycle a_conflict_retry_whose_producer_leaves_the_conflict_unresolved_fails_before_the_checks`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-cli --test lifecycle a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_run && cargo test -p athena-core --lib land::tests::a_kept_candidate_merged_onto_a_moved_tip && cargo test -p athena-core --lib land::tests::landing_refuses_content_that_differs_from_the_verified_content`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test -p athena-core --test refusal_corpus && cargo test -p athena-cli --test backtest`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
