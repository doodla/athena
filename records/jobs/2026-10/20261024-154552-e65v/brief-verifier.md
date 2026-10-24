# Job 20261024-154552-e65v: verify "Classify PR commits by content in pr_commits, not trailers"

## Request (verbatim)

> try again on 20261024-153908-1rve

## Reading (accepted by the user)

Rewrite pr_commits in athena-core land.rs to classify each commit in remote_base..head by content (records_only, declared_intervention, job_landing) so `athena land --pr` stops refusing real PRs. Job landing reuses the gate's closing_jobs logic, so a blocked job later landed via outcome-landed.json is accepted. Audience: Sri and the verifier running the athena-core land and gh tests.

Assumptions:
- Per the user's reply, a job_landing is recognised exactly as the gate's closing_jobs does: the commit adds any one of the job's closing files and the job's outcome in that commit's tree is landed; the function is reused (made pub) rather than a new rule written.
- operator/ is an owner path via a nested operator/.gitattributes containing '* athena-exclude'; attrs.rs uses git check-attr --source, which honours nested files, so a lease commit under operator/ is records_only. The fixture mirrors this with a nested operator/.gitattributes.
- A job_landing requires the job's verification record to be present in the commit's tree so the verification id can be set; missing verification or non-landed outcome is refused with the sha.
- The previous reading's deliverables (land.rs, gh.rs, invariant.rs, records-history.md, CHANGELOG.md) still apply; invariant.rs changes only to expose reusable closing-job logic.
- Only athena-core land and gh tests are run during work; a narrow athena-gate invariant run is allowed only if invariant.rs changes, and the reason is stated.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core land::`. A realistic-range test builds: reading, dispatch and introspection commits, a lease commit under operator/ (fixture has a nested operator/.gitattributes with '* athena-exclude', named in the test's doc comment), an intervention commit adding records/interventions/<month>/<id>.json, then a job's landing adding candidate, verification and outcome. No commit has an Athena-Job trailer. Kinds asserted in order: records_only x4, declared_intervention, job_landing with that job and verification id.
2. [behavior] Run `cargo test -p athena-core land::`. A blocked-then-landed test: a commit adding candidate, verification and a blocked outcome yields records_only; a following commit adding only outcome-landed.json plus governed work yields job_landing for that job with its verification id.
3. [behavior] Run `cargo test -p athena-core land::`. Refusal tests: a commit changing governed content that is neither an intervention nor a job's closing records, a landing with no verification record, and a landing whose outcome is not landed each make pr_commits return an error containing that commit's sha. A companion assertion shows an operator/ commit without the exclusion is refused with its sha.
4. [behavior] Run `cargo test -p athena-core gh::`. The gh tests pass; commits built for a Landing record carry the kind (and job/verification) from pr_commits instead of a hard-coded JobLanding and pass LandingCommit::validate.
5. [content] pr_commits and PrCommit no longer read trailers or mention Athena-Job; PrCommit carries kind plus optional job and verification. Doc comments state the content rules: records_only when fingerprint equals the parent's using the gate's fingerprint function; job_landing via the gate's closing_jobs/not_landed logic (single source of truth). An empty range is still an error.
6. [content] The old trailer-count tests are removed, not left alongside the new ones. The change description lists every writer of landing records (including run.rs retry_landing) and the test covering each, and states why any athena-gate invariant test run was done.
7. [content] docs/design/records-history.md and CHANGELOG.md describe the content-based classification and no longer describe the trailer rule.
8. [behavior] Run `cargo clippy -p athena-core --all-targets` and `cargo fmt --check`; both are clean.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing run.rs retry_landing or other landing-record writers
- Changing .gitattributes or attrs.rs semantics
- Running the full workspace test suite
- Changing the gate's invariant semantics beyond exposing reusable logic

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New land tests replace trailer tests; clippy and fmt criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests asserting commit kinds and sha-bearing errors.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Trailer dependence is removed rather than excepted.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to pr_commits and its caller plus docs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog explain the content rules for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New unit test in land.rs tests; the doc comment names the nested operator/.gitattributes ('* athena-exclude') the fixture writes. Builds reading, dispatch, introspection, lease commit under operator/, an intervention commit adding records/interventions/<month>/<id>.json, then a landing adding candidate.json, verification.json, outcome.json (landed) plus a governed file; no commit message has a trailer. Run: `cargo test -p athena-core land::`. Observes: pr_commits returns kinds [RecordsOnly x4, DeclaredIntervention, JobLanding]; the last has the job id and verification == the job's record id.
  Fails without the behavior: With the trailer rule, every commit lacking an Athena-Job trailer makes pr_commits error, so the test fails at the first reading commit; with a wrong order the intervention or lease commit gets the wrong kind and the assert_eq on the kinds fails.
- Criterion 2: New unit test: commit 1 adds candidate, verification and outcome.json with state blocked (no governed change); commit 2 adds only outcome-landed.json (after_blocked true, state landed) plus a governed file. Run: `cargo test -p athena-core land::`. Observes: kinds [RecordsOnly, JobLanding]; the second carries the job and its verification id.
  Fails without the behavior: If closing_jobs/not_landed were not reused, the blocked commit would be a job_landing (or refused) and the later outcome-landed commit would not be recognised; the test fails.
- Criterion 3: Four unit tests: (a) a governed hand edit with no records; (b) a landing adding candidate.json and outcome.json (landed) but no verification.json; (c) a governed change with a candidate, verification and a non-landed outcome; (d) the same operator/ lease commit in a repo whose operator/.gitattributes is absent. Run: `cargo test -p athena-core land::`. Observes: each pr_commits returns Err whose text contains that commit's full sha.
  Fails without the behavior: If governed content passed unchecked, or a landing without verification or with a blocked/failed outcome were accepted, pr_commits would return Ok and unwrap_err would panic; if the sha were missing from the message the contains assertion fails.
- Criterion 4: Existing records_flow tests with the World fixture changed so its first commit adds the job's candidate, verification and landed outcome plus b.txt. Add one test where the PR also has a records-only commit, and assert the stored Landing.commits kinds and that every entry passes LandingCommit::validate. Replace the trailer-less test with a governed hand edit that is refused, naming the sha, with nothing recorded or merged. Run: `cargo test -p athena-core gh::`. Observes: All gh:: tests pass; the stored landing commits carry job_landing with job and verification, and records_only with none.
  Fails without the behavior: With a hard-coded JobLanding the records_flow test for a records-only or intervention commit would carry the wrong kind; with the old fixture (governed file, no records) pr_commits would refuse and the landing tests would fail.
- Criterion 5: Read land.rs: PrCommit {sha, tree, kind, job: Option, verification: Option}; pr_commits and its doc comments contain no 'trailer' or 'Athena-Job' text; docs state the fingerprint-equals-parent rule and closing_jobs/not_landed reuse; the empty-range bail! stays, covered by an existing-style test. Check with the test run above. Run: `cargo test -p athena-core land::`. Observes: The empty-range test passes; no trailer or Athena-Job mention in pr_commits or PrCommit.
  Fails without the behavior: Not applicable (content criterion); the tests compile only against the new PrCommit shape.
- Criterion 6: The old trailer tests are deleted: land.rs has none for pr_commits, and gh.rs's a_commit_without_a_job_trailer_is_named... is replaced. The run shows only the new test names. The change description lists every writer of landing records: GitLandingRecords::intent and ::landing (gh.rs, covered by records_flow tests and the new kind test) and run.rs retry_landing (which writes outcome records and does not call pr_commits; covered by existing run.rs tests, unchanged), and states why any athena-gate run was done (a narrow run of the closing_jobs-related invariant tests only because invariant.rs gained a pub). Run: `cargo test -p athena-core land::`. Observes: Test listing has no trailer-count tests; the description has the list.
  Fails without the behavior: Not applicable (content criterion).
- Criterion 7: Review the diff of docs/design/records-history.md (section 5.3 'Which commits' bullet and the failure-path table row) and CHANGELOG.md (new Changed entry under Unreleased, plus fix the Added line saying pr_commits is unchanged). Run: `git diff`. Observes: Both describe the three content rules and no longer say 'exactly one Athena-Job trailer'.
  Fails without the behavior: Not applicable (content criterion).
- Criterion 8: Run cargo clippy -p athena-core --all-targets, then cargo fmt --check. Run: `cargo clippy -p athena-core --all-targets`. Observes: Both finish with no warnings and no diff.
  Fails without the behavior: Unused imports (JobId parse, Context) or an unformatted file would fail clippy or fmt.

Planned files:

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## The change

Base acf974c9a8f8 → candidate 573e67059176. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  15 +-
 crates/athena-core/src/gh.rs        |  83 +++++--
 crates/athena-core/src/land.rs      | 432 +++++++++++++++++++++++++++++++++---
 crates/athena-gate/src/invariant.rs |   5 +-
 docs/design/records-history.md      |  14 +-
 5 files changed, 493 insertions(+), 56 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` failed (358 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
