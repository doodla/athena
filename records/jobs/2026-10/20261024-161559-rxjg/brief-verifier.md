# Job 20261024-161559-rxjg: verify "Classify PR commits by content in pr_commits; fix all dependents"

## Request (verbatim)

> try again on 20261024-154552-e65v

## Reading (accepted by the user)

Rewrite pr_commits in athena-core land.rs to classify each commit in remote_base..head by content (records_only, declared_intervention, job_landing via the gate's closing_jobs), and update every caller and fixture (gh.rs, reconcile.rs, athena-cli reconcile tests, run.rs retry_landing, drain.rs if affected). Audience: Sri and the verifier running athena-core and athena-cli reconcile tests.

Assumptions:
- Owner-path fixture may use any directory name with a nested .gitattributes '* athena-exclude' (owner-notes/ fine, per user reply); the word the boundary scan bans is not used.
- Gate closing_jobs logic in crates/athena-gate/src/invariant.rs is reused, possibly made public, as single source of truth.
- Reconcile fixtures are updated to follow the new content rules (real closing records) rather than loosening the code; the change description states this.
- Callers found by grep: gh.rs, reconcile.rs, land.rs, athena-cli/tests/reconcile.rs; run.rs and drain.rs are audited for GitLandingRecords/pr_commits use and changed only if they depend on it.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core land::`. A realistic-range test (reading, dispatch, introspection commits, an owner-path commit under a directory with nested .gitattributes '* athena-exclude' named in the doc comment, an intervention adding records/interventions/<month>/<id>.json, then a job's candidate, verification and outcome; no Athena-Job trailers) yields records_only x4, declared_intervention, job_landing with that job and verification id.
2. [behavior] Run `cargo test -p athena-core land::`. Blocked-then-landed: a commit adding candidate, verification and blocked outcome is records_only; a later commit adding only outcome-landed.json plus governed work is job_landing for that job, reusing the gate's closing_jobs logic.
3. [behavior] Run `cargo test -p athena-core land::`. Refusal tests each make pr_commits return an error containing the commit's sha: governed change that is neither intervention nor closing records; landing with no verification; landing whose outcome is not landed; owner-path commit without the exclusion.
4. [behavior] Run `cargo test -p athena-core` and `cargo test -p athena-cli --test reconcile`. All pass, including the 5 core reconcile tests and the 2 CLI reconcile tests that failed before; the change description says whether fixtures or code changed and why.
5. [behavior] Run `cargo test -p athena-core gh::`. Landing commits carry kind/job/verification from pr_commits and pass LandingCommit::validate.
6. [content] pr_commits and PrCommit no longer read trailers or mention Athena-Job; doc comments state the content rules and that closing_jobs is the single source of truth. Old trailer-count tests are removed. An empty range is still an error.
7. [content] Change description lists every caller of pr_commits and every writer of landing records (gh.rs, run.rs retry_landing, reconcile.rs via GitLandingRecords, drain.rs if relevant) with the test covering each, and cites commands actually run.
8. [content] docs/design/records-history.md and CHANGELOG.md describe content-based classification and no longer describe the trailer rule.
9. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`; both clean.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/drain.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Adding a repo-wide caller-finding check mechanism
- Changing the gate's fingerprint semantics

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Run athena-core and athena-cli reconcile tests, clippy, fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert classified kinds and sha-bearing errors.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Reuse gate closing_jobs; fix all dependent callers and fixtures.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change to classification and its dependents.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and change description written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test: base commits nested owner-notes/.gitattributes '* athena-exclude'; then reading, dispatch, introspection, owner-notes file, intervention record, then job candidate/verification/outcome plus governed work; no trailers; call pr_commits. Run: `cargo test -p athena-core land::`. Observes: Kinds are [RecordsOnly x4, DeclaredIntervention, JobLanding]; last carries the fixture job and verification id.
  Fails without the behavior: With the trailer rule every commit errors; wrong classification changes the asserted kinds.
- Criterion 2: Unit test: commit 1 adds candidate, verification, blocked outcome; commit 2 adds only outcome-landed.json plus a governed file. Run: `cargo test -p athena-core land::`. Observes: Commit 1 RecordsOnly with no job; commit 2 JobLanding for that job with its verification id.
  Fails without the behavior: Without reusing closing_jobs the blocked commit is mis-classified or the outcome-landed-only commit does not map to its job.
- Criterion 3: Unit tests: governed change with no intervention/closing records; landing without verification; landed outcome absent (blocked outcome only with landing path); owner-path file without exclusion (including same-commit .gitattributes case). Run: `cargo test -p athena-core land::`. Observes: Each pr_commits Err text contains the offending commit sha.
  Fails without the behavior: Without the refusals these commits classify as records or landings and no error contains the sha.
- Criterion 4: Run the full core suite then the CLI reconcile tests. Run: `cargo test -p athena-core`. Observes: All pass including the 5 core reconcile tests; then `cargo test -p athena-cli --test reconcile` passes the 2 CLI tests; description says fixtures changed and why.
  Fails without the behavior: Trailer-only fixtures fail pr_commits, so those 7 tests fail unless fixtures carry real closing records or a records-only commit.
- Criterion 5: land_pr tests with a real landed-job commit and a records-only commit; assert landing.commits kind/job/verification and validate().unwrap(). Run: `cargo test -p athena-core gh::`. Observes: LandingCommits match pr_commits output; validate passes; intent.job_ids lists only job_landing jobs.
  Fails without the behavior: If kind/job/verification were not copied, records-only commits would become JobLanding and fail validate.
- Criterion 6: Trailerless tests plus pr_commits(base, base) asserting Err containing 'adds no commit'; read land.rs and confirm no trailers()/Athena-Job in pr_commits/PrCommit and doc comments state content rules and closing_jobs as single source; old trailer-count tests removed. Run: `cargo test -p athena-core land::`. Observes: Empty-range error; trailerless tests pass; docs as stated.
  Fails without the behavior: A leftover trailer read makes trailerless tests error; dropping the empty-range error fails its test.
- Criterion 7: Compare the change description with grep-found callers and writers: gh.rs GitLandingRecords::commits/intent/landing/write_landing, reconcile.rs via GitLandingRecords, crates/athena-cli/src/drain.rs PrLander::land (indirect, gh.rs land_pr tests), tests/external_calls.rs (records: None, never reaches pr_commits), run.rs retry_landing (writer, no pr_commits call), CLI reconcile tests; cited commands actually run. Run: `git diff`. Observes: Every caller/writer listed with its covering test and commands run with results.
- Criterion 8: Read diffs of docs/design/records-history.md and CHANGELOG.md. Run: `git diff`. Observes: Content-based classification and closing_jobs described, the multi-job squash refusal and parent-rules exclusion stated; the exactly-one-Athena-Job-trailer rule is gone; CHANGELOG entry under unreleased.
- Criterion 9: Run clippy, then fmt exactly as the criterion states. Run: `cargo clippy --all-targets`. Observes: Clean; then `cargo fmt --check` clean (also the check form `cargo fmt --all --check`).
  Fails without the behavior: A lint in new code (unused import after trailer removal, needless clone) or unformatted code makes the command exit non-zero.

Planned files:

- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## The change

Base 85058c550113 → candidate 2368b8041274. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  16 ++
 crates/athena-cli/tests/reconcile.rs |  86 +++++-
 crates/athena-core/src/gh.rs         |  72 +++--
 crates/athena-core/src/land.rs       | 494 ++++++++++++++++++++++++++++++++---
 crates/athena-core/src/reconcile.rs  |  21 +-
 crates/athena-gate/src/invariant.rs  |   5 +-
 docs/design/records-history.md       |  32 ++-
 7 files changed, 640 insertions(+), 86 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1094 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
