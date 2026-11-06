# Job 20261106-062207-cbcc: verify "Let athena retry re-verify a stale job's kept candidate on the new tip"

## Request (verbatim)

> try again on 20261106-041052-wzy7

## Reading (accepted by the user)

Retry of job 20261106-041052-wzy7: `athena retry` accepts a stale job, merges its kept candidate onto the tip, and on a clean merge re-checks, replays, re-verifies and lands it with no producer call; a conflict starts a producer retry from the candidate. Audience: users running parallel jobs, and the verifier. The reversal of alternative (d) is recorded as a NEW appended decision, not an edit to an existing line.

Assumptions:
- Code survey: entry is `Athena::retry` in crates/athena-core/src/reading.rs (~line 1103), then dispatch and resume_candidate in crates/athena-core/src/run/mod.rs, with close in run/close.rs. Staleness and the fingerprint-equality bail are in `plan()` in crates/athena-core/src/land.rs; the merge uses crates/athena-git/src/lib.rs. Retry CLI help is in crates/athena-cli/src/main.rs. Existing retry tests are in crates/athena-cli/tests/lifecycle.rs and crates/athena-cli/tests/retry.rs; unit tests in crates/athena-core/src/run/tests.rs. The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The producer-call count is measured directly from the fake agent's recorded calls.
- Job 20261106-041052-wzy7 findings addressed: (1) major, in-place annotation of docs/decisions.md line 297: dropped. The log is append-only and hash-pinned through D121 by crates/athena-cli/src/decisions.rs, so the reversal goes in a new appended decision that cites D10 by id; no existing line is edited. (2) README.md change: kept and now a listed deliverable. (3) reader brief still says 'rejected' for a stale retry: fixed in crates/athena-context/src/briefs.rs (the earlier crates/athena-core/src/briefs.rs path does not exist) and the pinned reading.rs test is updated. (4) retry help first line in main.rs and CHANGELOG README wording: fixed so the help covers stale jobs and CHANGELOG states only what the diff does.
- The prior candidate's work is kept as the starting point: CHANGELOG.md, README.md, main.rs, lifecycle.rs, land.rs, reading.rs, run/mod.rs, athena-git lib.rs, docs/decisions.md. A new retry starts from the base, so the producer redoes these with the fixes above.
- No governance file (athena.toml, defaults.toml, workflows) needs to change.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal instead of an error.
2. [behavior] Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and zero producer calls are recorded for the retry.
3. [behavior] Run `cargo test`: a test where the candidate conflicts with the moved tip shows the retry goes to a producer retry seeded from the kept candidate, and nothing lands without a new verification.
4. [behavior] Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree, and a landing whose verified fingerprint differs from the landed content is still refused (existing land.rs Plan::Stale tests still pass).
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass, including the decisions test the_decision_log_is_well_formed_and_only_appended_to.
6. [content] Grep docs/decisions.md: a NEW decision appended at the end states it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of the landing decision (D10), gives the reason that stale landings now recur with parallel jobs, and cites D10 by id. `git diff` shows no existing line of docs/decisions.md changed, only additions.
7. [content] CHANGELOG.md has an entry describing only what the diff does: retry accepts a stale job, merges its candidate onto the tip, re-verifies and lands on a clean merge without a producer run, and runs the producer from the candidate on conflict. README.md text saying stale jobs are refused is corrected.
8. [content] The retry help text in crates/athena-cli/src/main.rs and the reader brief for a stale retry (crates/athena-context/src/briefs.rs) no longer describe a stale job as simply 'rejected' or as starting from the base; Grep shows the updated wording and the updated test.
9. [content] Only the listed files change; no secrets or personal data are committed.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Automatic (drain-triggered) re-verification of stale jobs without an explicit athena retry
- Changing the staleness rules in land.rs plan() (what makes a job stale)
- Retrying other non-retryable states (landed, dropped, abandoned)
- Editing or annotating any existing line of docs/decisions.md or docs/decision-links.toml
- Prompt files and governance configuration

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the retry landing or retrying behavior by running cargo test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert zero producer calls and the verified fingerprint.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The decision is appended to the log rather than editing the pinned line.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs, CHANGELOG and README are written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test in crates/athena-cli/tests/lifecycle.rs, beside `a_governed_change_on_the_branch_makes_the_job_stale` (line 2027). Run a job so a user commit lands on the branch during verification and the job ends `Stale`. Then call `Athena::retry(job, &Lines::default())`. Run: `cargo test -p athena-cli --test lifecycle retry_on_a_stale_job_returns_a_proposal`. Observes: `retry` returns Ok with a Proposal whose `reading` is Some. The retry's request record is `try again on <job>`. The stale job is left unchanged. A queued job is still refused with the old message prefix (retry.rs:393 unchanged).
  Fails without the behavior: `earlier_attempt_with` (reading.rs:1362-1374) bails with `job … is stale; only a rejected or failed job can be tried again`, so `Athena::retry` returns Err and the `unwrap` in the test fails.
  Test seam: Production entry point `Athena::retry`. Fixture: `Fixture`, `Scenario.during_verification` (tests/common/mod.rs:428) and the fake agent driver, injected through `fx.athena(scenario.script())`. All exist today; the test is new (step 14). The tip commit changes `src/lib.rs` and the candidate only README.md, so the later merge is clean.
- Criterion 2: Unit test in crates/athena-core/src/run/tests.rs next to `interrupted_job_with_a_candidate_resumes_without_planner_or_producer` (line 3762). Wrap `counting(...)` so the first verifier turn commits `OTHER.md` on the branch (the job goes Stale), then `retry` the job and `go` the retry job with a `Lines` ui. Run: `cargo test -p athena-core --lib a_stale_job_whose_candidate_merges_cleanly`. Observes: After the first go the counts are (planner 0, producer 1, verifier 1) and the outcome is Stale. After retry and go: the producer count is still 1 and the verifier count is 2. The ui lines include `resume     `, `checks     ` and `replay     `, and the state is Landed. README.md at HEAD contains the demo text and `OTHER.md` is kept. The retry's candidate record has `base_commit` equal to the new tip's prediction commit.
  Fails without the behavior: Without the stale path in `dispatch`/`resume_candidate`, either `retry` errors (criterion 1), or the retry job runs the producer, so the producer count rises from 1 to 2 and the asserts on `(planner, producer, …)` fail. If the merged tree skipped the checks, the `checks     ` and `replay     ` lines would be missing. If it skipped verification, the verifier count would stay 1.
  Test seam: Entry points `Athena::retry` and `Athena::go`, which reach `dispatch`, `stale_seed` and `resume_candidate`. Counting double: `Counts` and `counting()` (run/tests.rs:2119, 2152), `repo()`, `athena_with()`, `Lines` all exist. The tip-moving wrapper script is new in step 13. It only runs `git commit`; the merge and verification logic stay in production code.
- Criterion 3: Unit test in crates/athena-core/src/run/tests.rs. The tip commit rewrites the same README.md line the candidate changed, so the job goes Stale. After `retry`, check the repository head is unchanged by the retry command, then `go` the retry job. Run: `cargo test -p athena-core --lib a_stale_job_whose_candidate_conflicts`. Observes: A second producer run happens, so the producer count is 2. That run's work directory README.md contains `<<<<<<<` and the kept candidate's text. Its producer brief contains the conflict section and names the stale job. The verifier runs again (count 2) before any landing. The state is Landed with a Verification record whose `verified_fingerprint` equals the fingerprint of the landed commit.
  Fails without the behavior: Without the conflict branch the retry either lands the overlaid candidate, losing the tip's README change, or fails. The asserts that the producer ran a second time, saw markers in a README seeded from the candidate, and that the verifier count rose before the landing, would fail.
  Test seam: Entry points `Athena::retry` and `Athena::go`. Double: the same `counting()` wrapper, extended to record the producer's `spec.workdir` README and `spec.user_prompt` (the pattern is `two_phases` in tests/retry.rs:525). It is injected through `athena_with(... script)`. Existing: `Counts`, `athena_with`. New in step 13: the recording wrapper and the tip-moving commit.
- Criterion 4: land.rs unit test with the existing `repo()`, `commit()` and `pass()` fixtures. Move the tip, build a candidate commit on the tip from `Git::merge_tree_conflicts`, then call `plan`. The core test above also asserts the end-to-end equality. `land::tests` runs the existing `Plan::Stale` tests (`a_base_that_is_no_longer_an_ancestor…`, `a_changed_exclusion_rule…`) unchanged. Run: `cargo test -p athena-core --lib a_merged_candidate_lands_content_with_the_verified_fingerprint && cargo test -p athena-core --lib a_stale_job_whose_candidate_merges_cleanly && cargo test -p athena-core --lib land::tests`. Observes: `plan(...)` returns `Plan::Tree(t)` with `fingerprint(t, base) == verified.fingerprint()`. The same call with a different fingerprint returns Err. In the core test the retry's `verification.verified_fingerprint` equals `fingerprint(git, landed_commit, retry_base)`.
  Fails without the behavior: If the landed tree did not match the verified merged tree, `plan()` would bail with `the content to land … differs from the verified content` and the equality assert would fail. If the wrong-fingerprint case no longer errored, `landing_refuses_content_that_differs_from_the_verified_content` and the new refusal assert would fail.
  Test seam: Production entry point `land::plan`; `Git::merge_tree_conflicts` is new (step 3). Fixtures `base_and_candidate`, `pass`, `commit` exist in land.rs tests (used at lines 590-611, 706-709). Nothing is faked: real git repositories in temp dirs.
- Criterion 5: Run the three Athena checks. The decisions test lives in crates/athena-cli/src/decisions.rs:253. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The decisions test passes with `PINNED_LEN` and `PINNED_SHA256` untouched, because D157 is appended after the pinned prefix and has a Provenance bullet.
  Fails without the behavior: A formatting or lint slip, an edit to a pinned decision line (`the_decision_log_is_well_formed_and_only_appended_to` would report `the first 230046 bytes changed`), or a D157 hit not listed in the links file (the real-log backtest test) would fail the run.
  Test seam: Not applicable; it runs the whole workspace. No test double is involved.
- Criterion 6: Read the diff of docs/decisions.md and Grep for `## D157` and `Automatic re-verification on the new base`. Run `cargo test -p athena-cli --lib the_decision_log_is_well_formed_and_only_appended_to` and `cargo test -p athena-cli --test backtest the_real_log_has_no_condition_a_finding_against_the_confirmed_list`. Run: `git diff -U0 -- docs/decisions.md`. Observes: The diff has only `+` lines, all after D156. The new entry D157 states it reverses the left-out alternative (d) 'Automatic re-verification on the new base' of the landing decision D10 (cited by id), and gives the reason that stale landings now recur with parallel jobs. It has no `supersedes`, `superseded by` or `amends` phrase.
  Fails without the behavior: An edit to D10 (line 297) would show as a removed `-` line and would also fail `the_decision_log_is_well_formed_and_only_appended_to`.
  Test seam: Not a behavior criterion; there is no double. Existing test files read the real docs/decisions.md and docs/decision-links.toml.
- Criterion 7: Read the diff. Grep CHANGELOG.md for `stale` under `## [Unreleased]` and README.md lines 48-63. Run: `git diff -- CHANGELOG.md README.md`. Observes: CHANGELOG has one new entry under Unreleased/Added that says only what the diff does. Retry accepts a stale job and merges its kept candidate onto the tip. A clean merge is checked, replayed, re-verified and landed with no producer run. A conflict runs the producer from the candidate, and nothing lands without a new verification. It names the root cause and D157. README no longer lists stale among the refused states and describes the stale retry.
  Fails without the behavior: If README.md still said stale jobs are refused, the old sentence at README.md lines 57-58 (`Jobs that are landed, stale, in flight, or unknown are refused`) would remain in the diff context unchanged.
  Test seam: Not a behavior criterion; there is no double.
- Criterion 8: Unit tests plus one CLI test. The briefs.rs test builds `EarlierAttempt { end_state: "stale", .. }` and calls `reader_brief`. The reading.rs test replaces Stale in `only_rejected_or_failed_jobs_can_be_tried_again` (line 4653) with a test that `earlier_attempt` accepts a stale job and carries its reason. The CLI test runs `athena retry --help`. Run: `cargo test -p athena-context reader_briefs_say_a_stale_earlier_attempt_went_stale && cargo test -p athena-core --lib a_stale_job_can_be_tried_again && cargo test -p athena-cli --test retry the_retry_command_help_covers_stale_jobs`. Observes: The brief contains `went stale` and says the kept candidate is merged onto the tip. It does not contain `was rejected` or `A retry starts from the base` unqualified for a stale job. The help text names stale jobs and the merge behavior.
  Fails without the behavior: The brief would still say `which was rejected` and `A retry starts from the base` for a stale job, so the new brief asserts fail. The help text would still start `Try a rejected or failed job again`, so the help assert fails.
  Test seam: Entry points `reader_brief`, `earlier_attempt` and the binary `CARGO_BIN_EXE_athena`. Double: none for the briefs and the reading; the CLI test runs the real binary like `the_retry_command_takes_a_job_and_the_same_options_as_ask` (retry.rs:874). Fixtures `job_with_checks`, `records_with` and `outcome` exist in reading.rs tests; the new tests are added in steps 6 and 10.
- Criterion 9: Run `git status` and `git diff --stat`, then Grep the diff for secrets and email-like text. The boundary scan (`xtask` policy test) runs inside `cargo test --locked`. Run: `git status`. Observes: Only the 13 listed files are modified. No new files. No secrets, no email-like text, no banned words.
  Fails without the behavior: A file outside the list (for example display.rs or decision-links.toml) would appear in the status output.
  Test seam: Not a behavior criterion; there is no double.

Planned files:

- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 656c935d7fd5 → candidate afe0dc347833. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  12 ++
 README.md                            |  13 +-
 crates/athena-cli/src/main.rs        |   7 +-
 crates/athena-cli/tests/lifecycle.rs |  81 +++++++++++
 crates/athena-cli/tests/retry.rs     | 108 ++++++++++++++
 crates/athena-context/src/briefs.rs  | 112 ++++++++++++++-
 crates/athena-core/src/land.rs       |  46 ++++++
 crates/athena-core/src/reading.rs    |  66 +++++++--
 crates/athena-core/src/run/close.rs  |   3 +
 crates/athena-core/src/run/mod.rs    | 271 +++++++++++++++++++++++++++++------
 crates/athena-core/src/run/tests.rs  | 224 +++++++++++++++++++++++++++++
 crates/athena-git/src/lib.rs         |  82 +++++++++++
 docs/decisions.md                    |  42 ++++++
 13 files changed, 1002 insertions(+), 65 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-062207-cbcc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-062207-cbcc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42179613452 for afe0dc347833726b716c197a2d35f2fbfc61634d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42179613452 for afe0dc347833726b716c197a2d35f2fbfc61634d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42179613452 for afe0dc347833726b716c197a2d35f2fbfc61634d)
- ci-result: `not a check` passed (Check results for afe0dc347833726b716c197a2d35f2fbfc61634d come from the CI run https://github.com/doodla/athena/actions/runs/42179613452 (586s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass, including the decisions test the_decision_log_is_well_formed_and_only_appended_to.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test lifecycle retry_on_a_stale_job_returns_a_proposal`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-git/src/lib.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling regex-syntax v0.8.11
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling regex-automata v0.4.18
   Compiling tempfile v3.27.0
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling colorchoice v1.0.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling anstream v1.0.0
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling heck v0.5.0
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.1
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling clap v4.6.7
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 07s
     Running tests/lifecycle.rs (<job>/target/verify/debug/deps/lifecycle-81c3c517131d8419)

running 1 test
test retry_on_a_stale_job_returns_a_proposal_instead_of_an_error ... FAILED

failures:

---- retry_on_a_stale_job_returns_a_proposal_instead_of_an_error stdout ----

thread 'retry_on_a_stale_job_returns_a_proposal_instead_of_an_error' (23125) panicked at crates/athena-cli/tests/lifecycle.rs:2168:10:
called `Result::unwrap()` on an `Err` value: job 20261106-072702-zd73 is stale; only a rejected or failed job can be tried again (so can a timed out one), or one that was interrupted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    retry_on_a_stale_job_returns_a_proposal_instead_of_an_error

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 72 filtered out; finished in 8.41s

error: test failed, to rerun pass `-p athena-cli --test lifecycle`
error: 1 target failed:
    `-p athena-cli --test lifecycle`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib a_stale_job_whose_candidate_merges_cleanly`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-git/src/lib.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling serde_derive_internals v0.30.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0599]: no method named `merge_tree_conflicts` found for struct `athena_git::Git` in the current scope
   --> crates/athena-core/src/land.rs:723:26
    |
723 |         let merged = git.merge_tree_conflicts(&p, &tip, &c).unwrap();
    |                          ^^^^^^^^^^^^^^^^^^^^
    |
help: there is a method `merge_tree` with a similar name
    |
723 -         let merged = git.merge_tree_conflicts(&p, &tip, &c).unwrap();
723 +         let merged = git.merge_tree(&p, &tip, &c).unwrap();
    |

error[E0560]: struct `RetrySeed` has no field named `conflicts`
    --> crates/athena-core/src/reading.rs:5973:21
     |
5973 |                     conflicts: None,
     |                     ^^^^^^^^^ `RetrySeed` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0560, E0599.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 2 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib a_stale_job_whose_candidate_conflicts`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-git/src/lib.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0599]: no method named `merge_tree_conflicts` found for struct `athena_git::Git` in the current scope
   --> crates/athena-core/src/land.rs:723:26
    |
723 |         let merged = git.merge_tree_conflicts(&p, &tip, &c).unwrap();
    |                          ^^^^^^^^^^^^^^^^^^^^
    |
help: there is a method `merge_tree` with a similar name
    |
723 -         let merged = git.merge_tree_conflicts(&p, &tip, &c).unwrap();
723 +         let merged = git.merge_tree(&p, &tip, &c).unwrap();
    |

error[E0560]: struct `RetrySeed` has no field named `conflicts`
    --> crates/athena-core/src/reading.rs:5973:21
     |
5973 |                     conflicts: None,
     |                     ^^^^^^^^^ `RetrySeed` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0560, E0599.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 2 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core --lib a_merged_candidate_lands_content_with_the_verified_fingerprint && cargo test -p athena-core --lib a_stale_job_whose_candidate_merges_cleanly && cargo test -p athena-core --lib land::tests`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -U0 -- docs/decisions.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git diff -- CHANGELOG.md README.md`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test -p athena-context reader_briefs_say_a_stale_earlier_attempt_went_stale && cargo test -p athena-core --lib a_stale_job_can_be_tried_again && cargo test -p athena-cli --test retry the_retry_command_help_covers_stale_jobs`). Result: not run: not a plain cargo test command.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`git status`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
