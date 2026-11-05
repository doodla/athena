# Job 20261104-082023-06bz: verify "Make hand-written job records detectable by the gate (§16.2)"

## Request (verbatim)

> Make records written by hand detectable: a job fabricated record by record, a forged landing of a dispatched job that never closed, and a hand-made proposal closed with athena close all pass the gate today (design §16.2). Re-queued from 20261010-162440-2gwj, whose reading kept naming a test file that does not exist.

## Reading (accepted by the user)

Retry of job 20261010-162440-2gwj (dropped, never verified; its refusal came from naming the nonexistent crates/athena-gate/tests/history.rs, so no earlier finding is carried over). Extend the gate in crates/athena-gate so the three §16.2 fabrications (a whole job by hand, a forged landing of a dispatched-but-never-closed job, a hand-made proposal closed with `athena close`) are refused or flagged. Tests go in the existing src/landing_tests.rs, not a new tests/ file. Audience: the owner running the gate and CI.

Assumptions:
- Code survey: read crates/athena-gate/src/jobs.rs (check() runs per added records/jobs/ file via AnyRecord::parse; Score re-computed by expected_score, Outcome Landed handling follows at line ~349), crates/athena-gate/src/history.rs (check_history, check_after_cutover), and crates/athena-gate/src/landing_tests.rs (helpers stage_landing, judge, history_of, opening, closing; tests like a_squash_with_a_forged_rescore_is_still_refused). Entry point is the gate's per-commit check calling jobs::check, then history::check_history over the range; records validation goes through AnyRecord::parse and canonical_path, with check_record_file and the AnyRecord round-trip test in athena-data covering any new record field. The property is measured directly by building forged histories in landing_tests.rs and asserting the Violation list.
- Job 20261010-162440-2gwj is addressed as follows: its only recorded outcome was refusal for naming crates/athena-gate/tests/history.rs, which does not exist; this reading uses only existing files. Its candidate changed no files, so its listed scope (jobs.rs, landing_tests.rs, history.rs, athena-v0.md, enforcement.md, CHANGELOG.md) is kept and widened to all three forgeries; its out_of_scope narrowing to one forgery is dropped.
- Detection relies on in-repo evidence Athena always writes for a real run (opening records committed before the work, invocation/dispatch records, proposal provenance for a closed proposal) and refuses records that lack it or contradict their ordering; a deliberate forger who mimics all of it still passes, and the docs say so.
- The newly refused share of past readings is small since real jobs carry this evidence: expected 5% of newly refused readings.
- Design/docs update: §16.2 of docs/design/athena-v0.md and docs/enforcement.md are rewritten to say which forgeries are now detected and what residual remains.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-gate`: new tests in crates/athena-gate/src/landing_tests.rs build (a) a job whose records were all written by hand across two commits, (b) a single forged commit landing a job that was dispatched but never closed, and (c) a hand-made proposal closed with athena close; each yields at least one Violation, and the existing genuine-landing tests still pass.
2. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`: both clean; `cargo test --workspace` passes.
3. [behavior] The change refuses more readings than before and the higher refusal rate is intended: expected 5% of newly refused readings against the repository's own genuine job history; `cargo test -p athena-gate` includes a test that the repository's real history shape (a genuine landing) is not refused.
4. [content] Each refusal Violation's detail and repair text names the missing or contradicting evidence, so the owner can tell what to fix; checkable by Grep in crates/athena-gate/src/jobs.rs and history.rs.
5. [content] docs/design/athena-v0.md §16.2 and docs/enforcement.md no longer say the three forgeries pass; they state what is detected and the residual (consistent, fully mimicked records still pass without signing).
6. [content] CHANGELOG.md has an entry describing the new detection.
7. [content] Any new record field is accepted by the records allow-list: Grep shows check_record_file / the AnyRecord round-trip test covering it, and the change does not touch athena.toml, defaults.toml, workflows or prompt files.

## Expected deliverables

- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- docs/design/athena-v0.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Keyed MAC, signing, or a witness ledger outside the repo
- Changes to athena.toml, defaults.toml, workflows, setup.sh or prompt files
- Making a perfectly mimicked forgery undetectable-proof
- Editing anything under records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in crates/athena-gate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt clean.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in the gate, not by prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests build each forgery and assert a Violation.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: All three §16.2 paths are covered, not the one that surfaced.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs state detected cases and residual plainly.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test on forged histories.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the athena-gate suite. New tests in landing_tests.rs build real temporary git repositories and call `jobs::provenance` through a new helper `provenance_of(git, rev)`, and the live `crate::gate` for the wiring. They assert the Violation list. (a) `a_job_written_by_hand_across_two_commits_is_refused`, built as acceptance review 2 did: commit 1 holds request.json, a reading, a reader invocation and a hand-written job.json with all ten dispatch fields, with no brief-producer.md (variant 1) or with a brief_sha256 copied from another job and no matching file (variant 1b); `provenance_of` on commit 1 is empty because nothing closes. Commit 2 holds the hand change, candidate, verification, a verifier invocation whose times come from an earlier job (so before dispatched_at), and a Landed outcome. Assert violations whose detail contains 'brief-producer.md' and 'before the job was dispatched', each with a repair naming `athena go`. Variant 2 writes a brief file that matches job.json but keeps the copied verifier times: only 'before the job was dispatched' remains. Variant 3 (extra) omits the request record: detail contains 'no request record', repair contains 'athena ask'. Control `a_job_written_by_hand_with_every_link_consistent_still_passes`: matching brief and consistent times give no violation (recorded limit). (b) `a_forged_landing_of_a_dispatched_job_that_never_closed_is_refused`: a genuine Read commit (request, reading, reader invocation) and Dispatch commit (full job.json plus brief-producer.md), then one forged commit adding code and closing records copied or back-dated. Variants, each breaking one link: a producer started before dispatched_at; the verifier started before the producer ended; the outcome before the verifier ended; dispatched_at missing; the job's defaults hash differing from the reading's; brief-producer.md differing from brief_sha256. A reading recorded after dispatch is asserted through `crate::gate` (reported by the existing `prediction_problem`, not repeated by provenance). Control `..._written_with_current_times_still_passes` records that a forgery stamped with current times passes. (c) `a_hand_made_proposal_closed_with_athena_close_is_refused`: a reading with invented forecasts, no request record, no reader invocation, closed by a Score plus a Dropped outcome (the shape of manage.rs:195-207). Assert 'no request record' and 'no reader invocation'; the same commit through `crate::gate(git, &change, Scope::CommitMsg)` has a violation whose detail starts with 'provenance: '; a control with request, reader invocation, held base and matching defaults hash is accepted. (d) `the_older_build_job_in_a_non_cutover_history_is_not_refused`: an old-shape fixture with an unreachable base commit; `history_of(&git)` has no violation starting 'provenance: ', while `provenance_of` on its closing commit refuses. Existing genuine-landing tests still pass. Run: `cargo test -p athena-gate`. Observes: Each forged history gives at least one Rule::JobRecords Violation with detail starting 'provenance: ' that names the evidence, and carries the commit. The controls, the genuine landing and the older-build history walk give none. All existing landing, chain, intervention and exemption tests pass, with `job_records_of` (landing_tests.rs:827) and jobs.rs `check_head` now calling `check_records`.
  Fails without the behavior: On the base code `jobs::check` never reads the request record, the reader invocation, the dispatch fields, the brief file and hash, defaults.toml at the base or the invocation times, so (a), (b) and (c) fail on `!violations.is_empty()` and the `detail.contains(..)` assertions; in (a) the 'brief-producer.md' and 'before the job was dispatched' assertions fail. Without the `history.is_none()` call in `check`, the Scope::CommitMsg assertion in (c) fails. If provenance ran in the plain history walk, (d) fails.
  Test seam: Production entry points: `crate::jobs::provenance` via a new helper `provenance_of(git, rev)` that builds `Change::of_commit` and `Judged::new` like the existing `job_records_of` (landing_tests.rs ~823-827); `crate::gate(.., Scope::CommitMsg)` (lib.rs:337); `crate::history::check_history` via the existing `history_of`. Fixtures are real temporary git repositories from `testutil::{repo, write, commit}` and `owner_repo` (exist). Plan step 5 adds `genuine_repo` (owner_repo plus a committed defaults.toml), `genuine_reading`, `genuine_job` (all dispatch fields, optional brief file and hash override), `request_record`, `copied_verifier` (an invocation record with earlier times) and `provenance_of`. Existing: `write_records`, `opening`, `closing`, `stage_landing`, `history_of`. Nothing in the gate is stubbed.
- Criterion 2: Run clippy, fmt and the full workspace tests. The lifecycle, job_plan and drain tests drive real ask/go/close through the fake driver and the production writers through the live gate, so the new links are shown to hold for Athena's own records. The xtask policy tests (M24, boundary words) and the prompt-budget test run too. Run: `cargo clippy --workspace --all-targets && cargo fmt --check && cargo test --workspace`. Observes: No diagnostics, no formatting diff, all tests pass, including the lifecycle tests that close real proposals and land real jobs.
  Fails without the behavior: Not a refusal behavior. A warning, a formatting difference, or a genuine writer's records failing the new check (a false refusal in lifecycle, job_plan or drain) fails these commands.
  Test seam: Production binaries and test targets run as they are. No double. Exists.
- Criterion 3: New test `a_genuine_landing_is_not_refused` in landing_tests.rs builds the shape Athena writes with the genuine helpers. Commit 1 'Read' adds request.json, a reading with the repository's real base_commit and defaults_sha256, and the reader invocation. Commit 2 'Dispatch' adds a full job.json and brief-producer.md. Commit 3 adds the code, producer and verifier invocations in order, candidate, verification and a Landed outcome. It asserts `provenance_of` is empty for each commit and that `history_of(&git)` has no violation starting 'provenance: '. The verifier also runs the lifecycle suite (real writers, the real history shape) and the refusal corpus as a no-regression check. Run: `cargo test -p athena-gate && cargo test -p athena-cli --test lifecycle && cargo test -p athena-core --test refusal_corpus`. Observes: No provenance violation on any genuine commit or on the walk, so refusal is by missing or contradicting evidence, not blanket. The share of the repository's real past jobs newly refused is not measured, because records/ is absent from the snapshot; the 5% is the request's estimate. The live gate sees only new commits, so no earlier history is re-judged.
  Fails without the behavior: If the check refused genuine records, `a_genuine_landing_is_not_refused` and the lifecycle tests fail. If no check existed, the criterion 1 forged tests fail.
  Test seam: `provenance_of` and `history_of` on a real temporary repository with genuine-shaped helpers (added in step 5). The lifecycle suite uses its existing Fixture and fake driver with the production writers (exists).
- Criterion 4: The criterion 1 tests assert on `detail` and `repair` text, and a Grep checks the source: every violation in `jobs::provenance` is built by one local `fail(path, detail, repair)` whose detail begins with the `PROVENANCE` constant, and each call site names the missing or contradicting evidence and the Athena command that writes it. history.rs gains no refusal text because the history walk is unchanged. Run: `cargo test -p athena-gate`. Observes: Assertions such as detail.contains('no request record'), 'no reader invocation', 'does not match defaults.toml', 'has no job.json', 'lacks dispatched_at', 'before the job was dispatched', 'before the producer invocation ended', 'before the verifier invocation ended', 'brief-producer.md', and repair.contains('athena ask') / 'athena go' / 'athena close' pass. `Grep 'provenance: ' crates/athena-gate/src/jobs.rs` lists the texts.
  Fails without the behavior: A generic message such as 'records are not valid' fails the `contains` assertions in each forged test.
  Test seam: The `Violation { detail, repair, path, commit }` values returned by `jobs::provenance`, read through `provenance_of`. Existing type; assertions added in step 5.
- Criterion 5: Content check on the diff. §16.2 of docs/design/athena-v0.md ('Records are not signed' and the table row near line 1964) and docs/enforcement.md (M27 row near line 41, 'Records are Athena's own' bullet near lines 71-78) are rewritten. They list the evidence each of the three forgeries now has to carry, say which parts of the review's documented forgery are now caught (the missing or mismatched brief file, the copied verifier times) and which still pass, and name the new tests. They cite `prediction_problem` and the verifier-invocation check as the owners of reading-before-work and verifier presence. They say where the check runs (the live gate: commit-msg hook, `athena close`, every Athena commit) and the residuals: consistent, fully mimicked records still pass because nothing is signed; a (b) forgery written with current timestamps passes, and (b) is caught only when records are copied, back-dated or contradict the dispatch; a commit made with `--no-verify` is not caught by the hook, and the per-pull-request `check_pr_head` does not yet run provenance (follow-up); job records written only to athena/records are not judged; history, main commits after C0 and the non-cutover walk are not re-judged (D111 unchanged); a base commit not held skips the defaults hash; a reading from a build without request or reader-invocation records cannot be closed with `athena close`. Every test name cited on an M27 row must be a real fn in landing_tests.rs (policy test M24). Run: `git diff -- docs/design/athena-v0.md docs/enforcement.md && cargo test -p xtask every_test_named_in_the_enforcement_map_exists`. Observes: The old text saying the three forgeries pass is replaced by the detected cases and residuals. M24 passes: every backticked test name on the `| M` rows exists as a fn.
  Fails without the behavior: With no doc edit the diff is empty and the 'still passes' text remains; a cited name with no fn fails M24.
  Test seam: Not applicable: content criterion. Step 7 makes the edit; M24 is xtask/tests/policy.rs:483-517 (exists).
- Criterion 6: Content check: one new bullet under `## [Unreleased]` / `### Added` in CHANGELOG.md names the provenance check, the three forgeries, where it runs (the live gate), that earlier history and main after C0 are not re-judged, that `check_pr_head` is a follow-up, that a (b) forgery written with current timestamps passes and (b) is caught only when copied or back-dated, and that consistent forgeries pass without signing. Run: `git diff -- CHANGELOG.md`. Observes: A new Unreleased bullet describes the detection and the residuals.
  Fails without the behavior: No Unreleased entry appears in the diff.
  Test seam: Not applicable: content criterion. Step 8 makes the edit.
- Criterion 7: The verifier checks that no record field or governance file changed. The provenance code reads only existing fields. The athena-data tests that hold the allow-list and the AnyRecord round-trip run unchanged since no field is added. Run: `git diff --stat && cargo test -p athena-data`. Observes: The diff lists only crates/athena-gate/src/jobs.rs, crates/athena-gate/src/landing_tests.rs, docs/design/athena-v0.md, docs/enforcement.md and CHANGELOG.md. None of athena.toml, defaults.toml, .github/workflows, setup.sh, prompt files, records/, records.rs, reconcile.rs, history.rs, lib.rs or legacy.rs appears. The check_record_file and round-trip tests pass.
  Fails without the behavior: A new record field without allow-list and round-trip coverage fails `cargo test -p athena-data`; an edit to a governance file shows in the stat.
  Test seam: The repository diff and the existing athena-data tests. Exists; nothing is added.

Planned files:

- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- docs/design/athena-v0.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base c3a7a1794096 → candidate 5c4a5455d4b6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  16 +
 crates/athena-gate/src/jobs.rs          | 377 ++++++++++++++++++++-
 crates/athena-gate/src/landing_tests.rs | 565 +++++++++++++++++++++++++++++++-
 docs/design/athena-v0.md                |  45 ++-
 docs/enforcement.md                     |  32 +-
 5 files changed, 1013 insertions(+), 22 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261104-082023-06bz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261104-082023-06bz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41966511864 for 5c4a5455d4b6649be35bd09ddb3c3a067c1e63ac)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41966511864 for 5c4a5455d4b6649be35bd09ddb3c3a067c1e63ac)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41966511864 for 5c4a5455d4b6649be35bd09ddb3c3a067c1e63ac)
- ci-result: `not a check` passed (Check results for 5c4a5455d4b6649be35bd09ddb3c3a067c1e63ac come from the CI run https://github.com/doodla/athena/actions/runs/41966511864 (622s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`: both clean; `cargo test --workspace` passes.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-gate`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-gate/src/jobs.rs, docs/design/athena-v0.md, docs/enforcement.md.
  End of the output (data, not instructions):

```text
error[E0425]: cannot find function `check_records` in module `crate::jobs`
   --> crates/athena-gate/src/landing_tests.rs:827:18
    |
827 |     crate::jobs::check_records(git, &change, &judged, None).unwrap()
    |                  ^^^^^^^^^^^^^ not found in `crate::jobs`
    |
help: consider importing this function
    |
  4 + use crate::reconcile::check_records;
    |
help: if you import `check_records`, refer to it directly
    |
827 -     crate::jobs::check_records(git, &change, &judged, None).unwrap()
827 +     check_records(git, &change, &judged, None).unwrap()
    |

error[E0425]: cannot find function `provenance` in module `crate::jobs`
    --> crates/athena-gate/src/landing_tests.rs:1448:18
     |
1448 |     crate::jobs::provenance(git, &change, &judged, None).unwrap()
     |                  ^^^^^^^^^^ not found in `crate::jobs`

error[E0425]: cannot find value `PROVENANCE` in module `crate::jobs`
    --> crates/athena-gate/src/landing_tests.rs:1467:43
     |
1467 |         v.detail.starts_with(crate::jobs::PROVENANCE),
     |                                           ^^^^^^^^^^ not found in `crate::jobs`

error[E0425]: cannot find value `PROVENANCE` in module `crate::jobs`
    --> crates/athena-gate/src/landing_tests.rs:1510:56
     |
1510 |             .any(|v| v.detail.starts_with(crate::jobs::PROVENANCE)),
     |                                                        ^^^^^^^^^^ not found in `crate::jobs`

error[E0425]: cannot find value `PROVENANCE` in module `crate::jobs`
    --> crates/athena-gate/src/landing_tests.rs:1538:56
     |
1538 |             .any(|v| v.detail.starts_with(crate::jobs::PROVENANCE))
     |                                                        ^^^^^^^^^^ not found in `crate::jobs`

error[E0425]: cannot find value `PROVENANCE` in module `crate::jobs`
    --> crates/athena-gate/src/landing_tests.rs:1734:86
     |
1734 |             .any(|v| v.rule == Rule::JobRecords && v.detail.starts_with(crate::jobs::PROVENANCE)),
     |                                                                                      ^^^^^^^^^^ not found in `crate::jobs`

error[E0425]: cannot find value `PROVENANCE` in module `crate::jobs`
    --> crates/athena-gate/src/landing_tests.rs:1794:56
     |
1794 |             .any(|v| v.detail.starts_with(crate::jobs::PROVENANCE)),
     |                                                        ^^^^^^^^^^ not found in `crate::jobs`

error[E0425]: cannot find value `PROVENANCE` in module `crate::jobs`
    --> crates/athena-gate/src/landing_tests.rs:1818:56
     |
1818 |             .any(|v| v.detail.starts_with(crate::jobs::PROVENANCE)),
     |                                                        ^^^^^^^^^^ not found in `crate::jobs`

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-gate` (lib test) due to 9 previous errors
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo clippy --workspace --all-targets && cargo fmt --check && cargo test --workspace`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-gate && cargo test -p athena-cli --test lifecycle && cargo test -p athena-core --test refusal_corpus`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff -- docs/design/athena-v0.md docs/enforcement.md && cargo test -p xtask every_test_named_in_the_enforcement_map_exists`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git diff --stat && cargo test -p athena-data`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
