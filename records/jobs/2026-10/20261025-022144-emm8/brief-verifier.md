# Job 20261025-022144-emm8: verify "Stop Athena- trailers in commits after cutover C0 (retry)"

## Request (verbatim)

> try again on 20261025-012657-hac3

## Reading (accepted by the user)

Retry of D111 job 7 part 1b: after cutover C0, Committer::build_commit_in, commit_with_staged and squash_text write no Athena- trailers, while trace and the auditor still read trailers up to C0. Keeps the earlier design; fixes the two verifier failures with a scratch repo whose pre-C0 history passes the old rules, so check_history and check_attribution report no violations, and tests the fallbacks through the real entry points.

Assumptions:
- Audience: the independent verifier and maintainers.
- Design is kept from the last attempt, per the user's reply.
- Dependency job 20261025-001232-vesh (the cutover) is already landed in the base.
- Same deliverable set as the accepted earlier reading; unneeded files will simply not change.

## Acceptance criteria

1. [behavior] Run cargo test: a scratch-repository test builds pre-C0 history that passes the old history rules (bootstrap commit, a job landed the old way with trailers, then the cutover job as C0). It then builds ONE job branch holding an intervention commit (commit_with_staged), a records-only commit (Committer) and the job landing commit, and lands it via land_pr with a squash merge. The squash commit and each of the three PR commits (via the landing record's second parent) have no 'Athena-' line.
2. [behavior] Run cargo test: in that repository the landing record classifies the three commits as declared_intervention, records_only and job_landing, and the test asserts check_history and check_attribution return an empty violations list on the post-C0 base. It makes no assertion that violations exist.
3. [behavior] Run cargo test: squash_text has a strict form and a relaxed post-C0 form, each tested. The relaxed form emits no Athena- line and is used for post-C0 landings.
4. [behavior] Run cargo test: a test in commits.rs covers the after-C0 strip (cutover landed: no trailers) and the kept cases (no cutover_job, or no landing record yet: trailers still written). Existing tests pass unchanged.
5. [behavior] Run cargo test: a test in status.rs drives the real athena trace/status entry point on a post-C0 job, and a separate test in auditor.rs drives the auditor reading a post-C0 job's files. Each runs in a scratch repository and covers both sides of C0: trailers read for commits up to and including C0, C0..head fallback after it, with pre-C0 results unchanged.
6. [behavior] Run cargo test: after C0 check_history runs only the secrets and attribution checks. The gate's own check keeps content and jobs checks and drops only the trailer-dependent invariant::check.
7. [content] CHANGELOG.md and docs/decisions.md describe the end of trailer writing after C0 (D111, job 7 part 1b). They name the readers that still use trailers up to C0 and state that after C0 the history check runs only secrets and attribution. The change description lists each trailer reader and writer with its covering test.
8. [behavior] Run cargo clippy --all-targets, cargo fmt --check and cargo test: all clean. athena.toml and .github/workflows files are unchanged, and pre-C0 behavior is unchanged.

## Expected deliverables

- crates/athena-core/src/commits.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing behavior for pre-C0 history
- Changes to athena.toml, defaults.toml or CI workflows
- Removing trailer readers for commits up to C0
- Other parts of the records-history plan

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New behavior has tests per reader and writer; clippy, fmt and test must be clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert that check_history and check_attribution pass and that trailers are absent, not that wiring exists.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Keeps the single focused design of the earlier attempt.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Each criterion maps to a test whose assertion states the criterion's expected outcome.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: lifecycle.rs scratch repo: bootstrap commit, old-way job landed with trailers, cutover job as C0 (landing record + manifest), hooks installed. One branch: commit_with_staged intervention, Committer records-only commit, job landing commit; land_pr with squash. Read messages via git.message. Run: `cargo test --locked post_c0_job_branch_writes_no_athena_trailers`. Observes: The squash commit and each of the three PR commits (via the landing record's second parent) have no line starting with 'Athena-'; land_pr found the landed commit.
  Fails without the behavior: A writer emitting trailers leaves Athena- lines; a hook requiring trailers refuses the intervention; a trailer-based landed_commit stops land_pr with 'job is not landed'.
- Criterion 2: Same test: assert check_history and check_attribution violations are empty on the pre-C0 base first, then read the landing record and assert the kinds, then assert both empty on the post-C0 base. No assertion that violations exist. Run: `cargo test --locked post_c0_job_branch_writes_no_athena_trailers`. Observes: Kinds declared_intervention, records_only, job_landing; both reports empty.
  Fails without the behavior: Wrong classification or violations reported if attribution or the gate skip were missing.
- Criterion 3: athena-git unit tests for strict form (exactly one Athena-Job; zero or two refused) and relaxed form (no Athena- line); gh.rs test builds the post-C0 PR request and checks commit_message. Run: `cargo test --locked squash_text`. Observes: Strict unchanged; relaxed emits no Athena- line; post-C0 landing uses relaxed.
  Fails without the behavior: Without the relaxed form a trailer-free post-C0 message is refused; a changed strict form accepts zero/two Athena-Job.
- Criterion 4: Committer::commit_records in scratch repos: Shown with c0 ancestor; no cutover_job; cutover_job but no landing record; Pending. Run: `cargo test --locked -p athena-core commits::`. Observes: No Athena- line after C0 and the gate accepts it; trailers still written in the three kept cases; existing tests unchanged.
  Fails without the behavior: Without the strip the after-C0 message keeps trailers; without the guard kept cases lose them.
- Criterion 5: status.rs test calls Athena::trace on a scratch repo with a job landed up to C0 (trailers) and one after (trailer-free); auditor.rs test drives the public auditor run on both jobs; run.rs test resumes a blocked post-C0 job via retry_landing and lands it. Run: `cargo test --locked post_c0`. Observes: Pre-C0 job gives same commits/files as before; post-C0 job gives its commits/files from C0..head; blocked post-C0 job lands without the trailer search.
  Fails without the behavior: No fallback: post-C0 trace lists no commits, auditor errors 'none could be read', retry_landing bails 'closing commit not found'.
- Criterion 6: history.rs test: trailer-free post-C0 commit gives no Rule::Invariant, planted secret still reported. lib.rs gate tests in CommitMsg and Commit scopes after C0: trailer-free commit passes; content violation still reported; a bad job record still reported by jobs::check; pre-C0 trailer-free commit still reports Rule::Invariant. Run: `cargo test --locked -p athena-gate after_cutover`. Observes: Only invariant::check is dropped; content and jobs checks stay in gate; check_history after C0 runs secrets, attribution (plus records-file checks).
  Fails without the behavior: If invariant::check still ran, the trailer-free commit reports Rule::Invariant; if content or jobs were dropped the bad record passes.
- Criterion 7: Read the diff. Run: `git diff -- CHANGELOG.md docs/decisions.md`. Observes: D111 job 7 part 1b entries say writing stops after C0; name readers still using trailers up to C0 (invariant check, trace, auditor, run.rs, landed_commit); state that after C0 the history check runs secrets and attribution (noting records-file checks stay); list each writer and reader with its covering test.
- Criterion 8: Run all three, then git status and git diff --stat. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0; no athena.toml, defaults.toml or .github/workflows changes; existing pre-C0 tests pass unchanged.
  Fails without the behavior: A pre-C0 behavior change breaks an existing test; format/lint slips fail.

Planned files:

- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-core/src/commits.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base 6c04fc659bbf → candidate 9ea787e3d555. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  19 ++
 crates/athena-cli/tests/auditor.rs    | 125 ++++++++++++
 crates/athena-core/src/auditor.rs     |  18 ++
 crates/athena-core/src/commits.rs     | 351 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/gh.rs          | 293 +++++++++++++++++++++++++++-
 crates/athena-core/src/manage.rs      |   7 +-
 crates/athena-core/src/run.rs         |  31 +--
 crates/athena-core/src/status.rs      |  69 ++++++-
 crates/athena-gate/src/attribution.rs |  23 +++
 crates/athena-gate/src/history.rs     | 118 +++++++++++-
 crates/athena-gate/src/lib.rs         |  16 +-
 crates/athena-git/src/lib.rs          |  40 ++++
 docs/decisions.md                     |  37 ++++
 13 files changed, 1126 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1190 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
