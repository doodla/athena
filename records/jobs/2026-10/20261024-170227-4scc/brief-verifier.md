# Job 20261024-170227-4scc: verify "Reland content-based pr_commits with a full change description"

## Request (verbatim)

> try again on 20261024-161559-rxjg

## Reading (accepted by the user)

Retry of rejected job 20261024-161559-rxjg: rewrite pr_commits in athena-core land.rs to classify commits in remote_base..head by content (records_only, declared_intervention, job_landing via the gate's closing_jobs), with no trailers. The earlier code passed every check; this retry must also give a change description listing every caller and writer with its covering test and the commands run. Audience: Sri and the verifier.

Assumptions:
- The earlier code was correct; the retry reproduces it and fixes the missing change description.
- run.rs and drain.rs do not call pr_commits, so they likely stay unchanged; the description says so.
- The two minor/nit findings (duplicated CLI fixtures, outcome-not-landed test using a missing outcome) are cheap to fix and are addressed.
- No athena.toml, defaults.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core land::`. A realistic-range test (reading, dispatch and introspection commits, an owner-path commit under a directory with a nested .gitattributes '* athena-exclude', an intervention adding records/interventions/<month>/<id>.json, then a job's candidate, verification and outcome; no Athena-Job trailers) yields records_only x4, declared_intervention, and job_landing carrying that job and its verification id.
2. [behavior] Run `cargo test -p athena-core land::`. Blocked-then-landed: a commit adding candidate, verification and a blocked outcome is records_only; a later commit adding only outcome-landed.json plus governed work is job_landing for that job, reusing the gate's closing_jobs.
3. [behavior] Run `cargo test -p athena-core land::`. Refusal tests each make pr_commits return an error containing the commit's sha: governed change that is neither intervention nor closing records; landing with no verification; landing whose outcome exists but is not landed (a real blocked outcome); owner-path commit without the exclusion.
4. [behavior] Run `cargo test -p athena-core` and `cargo test -p athena-cli --test reconcile`. All pass, including the core reconcile tests and the CLI reconcile tests. Run `cargo test -p athena-core gh::`: landing commits carry kind, job and verification from pr_commits and pass LandingCommit::validate.
5. [content] pr_commits and PrCommit no longer read trailers or mention Athena-Job; doc comments state the content rules and that closing_jobs is the single source of truth. Old trailer-count tests are removed. An empty range is still an error.
6. [content] The change description (commit message body and CHANGELOG entry) lists every caller of pr_commits and every writer of landing records (gh.rs GitLandingRecords::commits, reconcile.rs via GitLandingRecords, crates/athena-cli tests/reconcile.rs, run.rs retry_landing, crates/athena-cli/src/drain.rs PrLander), each with its covering test or the reason it needed no change, and cites the commands actually run.
7. [content] docs/design/records-history.md and CHANGELOG.md describe content-based classification and no longer describe the trailer rule.
8. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`; both are clean.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- The pre-handover evidence check in Athena proposed by the introspection
- Changing the gate's closing_jobs semantics
- Changes to records/ or owner files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run named test filters and check classification outcomes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Classification reuses closing_jobs rather than a new rule.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Change description written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in land.rs builds the realistic range in a temp repo with no Athena-Job trailers and asserts the exact kind sequence, and the job and verification id of the last commit. Run: `cargo test -p athena-core land::`. Observes: Test passes; kinds are [records_only x4, declared_intervention, job_landing] with the job's id and verification id.
  Fails without the behavior: If classification used trailers or ignored nested .gitattributes, the commits would error or be misclassified: the expected list records_only x4, declared_intervention, job_landing(job, verification id) would not match.
- Criterion 2: Unit test: commit 1 adds candidate, verification and a blocked outcome; commit 2 adds outcome-landed.json plus governed work. Run: `cargo test -p athena-core land::`. Observes: Commit 1 is records_only; commit 2 is job_landing for that job.
  Fails without the behavior: If the blocked commit were counted as a landing (closing_jobs not reused), the first commit would be job_landing and the assertion fails; if the later commit were not recognised it would be an error.
- Criterion 3: Four unit tests, each asserting pr_commits returns Err whose text contains the offending commit's sha: governed change with no intervention/closing records; landing records with no verification.json; outcome exists but is blocked (real blocked outcome record) with governed work; owner-path commit under a directory lacking the '* athena-exclude' rule. Run: `cargo test -p athena-core land::`. Observes: Each returns Err containing the sha.
  Fails without the behavior: Without the refusals, each range would return Ok with some kind (or a trailer error without the sha) and the error-contains-sha assertions would fail.
- Criterion 4: Run the three suites (each as its own simple command if chained commands are not allowed); the gh:: test asserts landing commits carry kind, job and verification from pr_commits and pass LandingCommit::validate. Run: `cargo test -p athena-core && cargo test -p athena-cli --test reconcile && cargo test -p athena-core gh::`. Observes: All pass, including core reconcile tests and CLI reconcile tests.
  Fails without the behavior: If pr_commits output did not map to valid LandingCommit values (kind/job/verification), intent/landing writes in gh:: and reconcile tests fail validate.
- Criterion 5: Read the diff: pr_commits/PrCommit have no trailers call or 'Athena-Job' text; doc comments give the content rules and name closing_jobs as the single source of truth; the old trailer-count tests are gone; the empty-range test still passes under `cargo test -p athena-core land::`. Run: `git diff -- crates/athena-core/src/land.rs`. Observes: No trailer reads or Athena-Job mentions in pr_commits/PrCommit; docs present; empty range returns an error.
- Criterion 6: Read the commit message body and the CHANGELOG.md entry: both list gh.rs GitLandingRecords::commits, reconcile.rs via GitLandingRecords, crates/athena-cli/tests/reconcile.rs, run.rs retry_landing, drain.rs PrLander, each with its covering test or the reason it needed no change, plus the commands run (the cargo test, clippy and fmt commands above). Run: `git show --stat HEAD`. Observes: Every caller/writer named with a test or a no-change reason; commands cited.
- Criterion 7: Read the diff: §5.3 and any trailer-rule text in records-history.md now describe content-based classification (records_only, declared_intervention, job_landing via closing_jobs); the CHANGELOG entry says the same. Run: `git diff -- docs/design/records-history.md CHANGELOG.md`. Observes: No remaining statement that each PR commit needs one Athena-Job trailer.
- Criterion 8: Run `cargo clippy --all-targets` and `cargo fmt --check`. Run: `cargo clippy --all-targets`. Observes: Both exit 0 with no warnings or diffs.
  Fails without the behavior: Unused imports (JobId parse, Context) or format drift would make clippy or fmt report.

Planned files:

- crates/athena-core/src/land.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## The change

Base 95f0122d40b3 → candidate e7fda7d71ef3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  48 +++-
 crates/athena-cli/tests/reconcile.rs |  89 +++++++-
 crates/athena-core/src/gh.rs         | 138 ++++++++++--
 crates/athena-core/src/land.rs       | 411 ++++++++++++++++++++++++++++++++---
 crates/athena-core/src/reconcile.rs  |  24 +-
 crates/athena-gate/src/invariant.rs  |  44 +++-
 docs/design/records-history.md       |  30 ++-
 7 files changed, 697 insertions(+), 87 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1095 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
