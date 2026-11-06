# Job 20261106-162047-hkx0: verify "Retry a stale job by merging its candidate onto the branch tip"

## Request (verbatim)

> try again on 20261106-161947-pdwd

## Reading (accepted by the user)

Retry of job 20261106-161947-pdwd (rejected, never verified, no findings, no candidate recorded): redo the work from the base with the same scope. `athena retry` on a stale job merges the kept candidate onto the tip. A clean merge is re-checked, re-verified and landed with no producer call, and a conflict goes to a producer retry seeded from the candidate. Audience: users running parallel jobs, and the verifier.

Assumptions:
- Job 20261106-161947-pdwd: it recorded no findings and no candidate, so there is no earlier work to keep and nothing to fix. Every file its candidate could have changed is listed as a deliverable and is rebuilt from the base. The scope and out_of_scope items of that job carry over unchanged.
- Code survey: I grepped crates/ for Stale. It appears in crates/athena-core/src/land.rs (Plan::Stale), crates/athena-core/src/run/close.rs, crates/athena-core/src/reading.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/tests/lifecycle.rs and crates/athena-data/src/records.rs. The entry point is the retry command in crates/athena-cli/src/main.rs, then the retry eligibility check in reading.rs, then run/mod.rs and run/close.rs. I did not read file bodies, so the producer must read them first. The merge needs a helper in crates/athena-git/src/lib.rs. The landing rule in land.rs plan() stays as is. Re-verification produces a new fingerprint that the existing check compares with what lands. No new record or data file is added, so check_record_file and AnyRecord are not affected.
- Existing code: this extends crates/athena-core/src/land.rs, crates/athena-core/src/run/close.rs, crates/athena-git/src/lib.rs and docs/decisions.md. It adds no new module.
- docs/decisions.md is append-only and decision D10 holds the left-out alternative (d). The new decision is appended at the end and reverses (d).
- The decisions.rs guard test and its 'append at end of file' message come from the accepted earlier reading, and the producer should keep them.
- athena.toml, defaults.toml and the workflows need no change.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test shows `athena retry` on a stale job returns a proposal rather than an error.
2. [behavior] Run `cargo test`: a test where the kept candidate merges cleanly onto a moved branch tip shows checks, replay and verification run on the merged tree, the job lands, and no planner or producer call is recorded for the retry.
3. [behavior] Run `cargo test`: a test where the candidate conflicts with the moved tip shows the retry goes to a producer run seeded from the candidate, and nothing lands without a new verification.
4. [behavior] Run `cargo test`: a test shows the landed content's fingerprint equals the fingerprint verified on the merged tree. The existing land.rs Plan::Stale tests still pass, so a landing whose verified fingerprint differs is refused.
5. [behavior] Run `cargo test`: a decisions.rs test shows the log guard fails with a message containing 'append at end of file' when an entry is inserted before the end of the pinned base, or when a new entry's id is not one greater than the previous highest. It passes on the real docs/decisions.md.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.
7. [content] `git diff -U0 -- docs/decisions.md` shows a single hunk of added lines after the last line of the base file and no changed existing line. The new decision cites D10 by id, says it reverses the left-out alternative (d) 'Automatic re-verification on the new base', and gives the reason that stale landings now recur with parallel jobs.
8. [content] CHANGELOG.md has an entry describing only what the diff does, and the README.md text saying stale jobs are refused is corrected.
9. [content] The retry help text in crates/athena-cli/src/main.rs and the stale reader brief in crates/athena-context/src/briefs.rs no longer call a stale job simply rejected. They state that the retry starts from the base when the candidate cannot be merged, and the brief test covers this.
10. [content] Only the listed files change, and no secrets or personal data are committed.

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
- crates/athena-cli/src/decisions.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Automatic (drain-triggered) re-verification of stale jobs without an explicit athena retry
- Changing the staleness rules in land.rs plan() (what makes a job stale)
- Retrying other non-retryable states (landed, dropped, abandoned)
- Editing or annotating any existing line of docs/decisions.md or docs/decision-links.toml
- Prompt files and governance configuration (athena.toml, defaults.toml, workflows)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The criteria require cargo test, clippy and fmt to pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test runs that exercise retry.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the retry lands or goes to a producer, not that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The landing fingerprint criterion checks that what lands is what was verified.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision, CHANGELOG and README text are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test in crates/athena-cli/tests/retry.rs, plus a reading.rs unit test. It makes a job stale the way lifecycle.rs:2027 does (hook commits a hand edit to src/lib.rs plus an intervention record on the branch during verification), then calls `Athena::retry` on it. Run: `cargo test --locked -p athena-cli --test retry a_stale_job_can_be_retried && cargo test --locked -p athena-core a_stale_job_is_accepted`. Observes: `retry` returns Ok with a proposal that has a reading and no `reader_error`. The recorded request is 'try again on <job>'. The old job's outcome is still stale. The reading.rs unit test shows a stale job is accepted and its reason carried.
  Fails without the behavior: `earlier_attempt_with` (reading.rs:1362-1374) bails 'only a rejected or failed job can be tried again' for `Closed(Stale)`, so `Athena::retry` returns Err and the test's unwrap fails.
  Test seam: Entry point `Athena::retry` (reading.rs:1103). Fixture `Fixture::new()` and `Scenario.during_verification` (tests/common/mod.rs:428, 637) injected through `fx.athena(model.script(scenario.script()))`, as `setup` in retry.rs:102 does. The reader fake names the retried job in its summary (`naming_earlier_jobs`, common/mod.rs:573). All exist today. Step 7 adds the test; step 2 the production change.
- Criterion 2: Lifecycle test. Setup: commit a multi-line README.md on the fixture root before `ask`. Run 1: the producer edit changes README.md line 1; the hook (armed once with an AtomicBool, copied from lifecycle.rs:2033-2069) appends a final line to README.md and commits it with an intervention record, so the job goes stale. Then `retry` and `go`. Assertions: (a) the roles in `scenario.invocations` after the retry contain Reader and Verifier and no Planner or Producer; (b) the verifier snapshot in `seen_by_verifier` (the entry added by the retry) holds both the candidate's line 1 and the tip's appended line in the same README.md; (c) the retry job's candidate record has `candidate_commit` equal to the commit whose fingerprint was verified, `git show <candidate_commit>:README.md` holds both hunks, and `changes` lists README.md only as the candidate's own change against the tip (not the tip's appended line alone); (d) the retry's verification record has measured check runs, and the UI lines contain no 'they do not run again' and no 'ci ... reused' line; (e) the retry's replay record, or a 'replay' UI line, names that same candidate commit (the producer reads `replay_stage`, run/mod.rs:1897-1950, first to pick the exact record and field); (f) `go` ends Landed, HEAD's README.md holds both hunks, and `check_history` has no violations. A second variant keeps the disjoint-file case (README.md vs src/lib.rs) as a smoke test only. Run: `cargo test --locked -p athena-cli --test lifecycle a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_call`. Observes: No planner or producer after the retry. The checks, replay and verifier each ran on the commit recorded as the retry's candidate, whose tree is the three-way merge of candidate and tip in a file both edited. The job landed with both hunks.
  Fails without the behavior: Without the stale seed and merge mode, the retry plans and produces from the base (a Producer role appears, so (a) fails) or `retry` errors. If merge mode replayed whole files instead of merging, the appended tip line is lost and (b), (c) and (f) fail. If check runs were reused or skipped, (d) fails.
  Test seam: Entry points `Athena::retry` then `Athena::go` (run/mod.rs:360), reaching the new `stale_seed` in `dispatch` and then `resume_candidate` in merge mode. The fixture is `Fixture` with `Scenario::invocations`, `Scenario::seen_by_verifier` (common/mod.rs:430-431) and `Scenario.edits` (whole-file content), injected as in lifecycle.rs:2029-2070 with `fx.athena(scenario.script())`. The fixtures exist. The multi-line README is written in the test by the existing `write` helper (common/mod.rs:298) and committed with `Git::run`. Steps 3-5 add `stale_seed` and the merge mode.
- Criterion 3: Two lifecycle tests. (i) The hook changes README.md line 1 on the branch (multi-line base), so the producer's line-1 edit conflicts. `retry` then `go`. A second call of the hook, in the retry's verifier turn, reads README.md at HEAD and finds the hand edit, so nothing landed before verification. (ii) Mandatory guard test: same setup, but the retry is built with a second `fx.athena(scenario2.script())` where scenario2 has its own `reading` listing README.md and src/lib.rs and edits only src/lib.rs. The retry's producer therefore never writes README.md, so any marker in the candidate comes from the seed. Also run/tests.rs unit tests of `has_conflict_markers` (opening and closing line, a Markdown `=======` underline only, empty text) and of the `check_and_verify` closing. Run: `cargo test --locked -p athena-cli --test lifecycle a_stale_job_whose_candidate_conflicts && cargo test --locked -p athena-core conflict_markers`. Observes: (i) After the retry a Producer invocation exists. The last `seen_by_producer` brief names the stale job, says its candidate was merged onto the tip with conflicts, lists README.md, and says that a path without marker lines keeps one side and must be decided. The verifier count rises by one and the retry job lands with a passing verification. (ii) The retry job is not landed: HEAD's README.md is unchanged, the verifier invocation count did not rise, and the closing reason names README.md and conflict markers. Markers the producer never wrote reached the guard, which shows the work tree was seeded from the conflicted merge.
  Fails without the behavior: Without the conflict path, `merge_tree` returns None and the retry is a plain base run whose brief has no stale-retry section, so the brief assertion in (i) fails. Without the seed, scenario2's producer would leave plain README.md and (ii) would not close at the guard (the verifier count would rise). Without the marker guard, (ii) runs the verifier on a tree holding markers and the verifier-count assertion fails.
  Test seam: `Athena::retry`, `Athena::go` and `dispatch` reach `Git::merge_tree_report` (conflict) and `BaseThenTree` seeding (snapshot.rs:97, unchanged). Fixture `Scenario` with `during_verification`, `reading`, `edits` and `seen_by_producer` (common/mod.rs:419-433), a second Scenario injected via a second `fx.athena(...)` on the same Fixture, as retry.rs `setup` builds one Athena per script. The guard is reached through `check_and_verify` via `RunContext.conflicts`, which step 5 adds. The fixtures exist; no change to common/mod.rs is needed. The producer reads `reading_for` and the reading builder near common/mod.rs:359 first to build the two-file reading so the scope check passes.
- Criterion 4: The criterion-2 lifecycle test also asserts that `fingerprint(git, landed_commit, retry_base)` (athena_git::attrs) equals `Finished.verification.verified_fingerprint` of the retry job, and that this fingerprint is that of the merged tree holding both README.md hunks. The existing land.rs tests (`landing_refuses_content_that_differs_from_the_verified_content`, land.rs:704, and the two Plan::Stale tests at 712 and 725) are run as a regression guard only, not as evidence of the new behavior. Run: `cargo test --locked -p athena-cli --test lifecycle a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_call && cargo test --locked -p athena-core land::`. Observes: The landed fingerprint equals the verified one. The existing Plan::Stale tests and the mismatch refusal still pass.
  Fails without the behavior: If the merged candidate were not the commit that was verified, the landed fingerprint would differ and `plan` would bail 'differs from the verified content' (land.rs:137-141), failing the lifecycle test.
  Test seam: `land::plan` (land.rs:70) unchanged, reached through `Athena::go` landing in the lifecycle fixture, which exists. The land.rs tests use the existing `pass(...)`, `repo()` and `commit()` helpers (land.rs:408-430). No step adds a seam.
- Criterion 5: Unit tests in decisions.rs calling the private `check_log(text, pinned_len, pinned_sha256)` (decisions.rs:209-250) on synthetic logs built with `athena_data::sha256_hex`: (a) an entry inserted before the pinned end with ascending ids; (b) a new entry D04 after D02 (a gap past the pin); (c) a correct append D03, which passes. The existing real-file test reads docs/decisions.md with the re-pinned constants. Run: `cargo test --locked -p athena-cli decisions`. Observes: (a) and (b) return Err containing 'append at end of file'. (c) returns Ok. The real file returns Ok, with D159 the only entry past the pin.
  Fails without the behavior: Today the prefix-change message says 'add new ones at the end', and an entry that skips an id is accepted, so the new assertions on the text and on the gap fail.
  Test seam: `check_log` exists and is called directly; the synthetic strings are built in the test as the existing test at about line 268 does. Step 8 changes the constants and the rule.
- Criterion 6: Run the three configured checks, plus the refusal corpus and mirrors targets as regression guards (no refusal or matching rule changes). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0, including the existing resume tests in run/tests.rs (about lines 3035, 3091, 3470, 3661, 4504), the mirrors target (AppendOnly on docs/decisions.md) and `cargo test -p athena-core --test refusal_corpus`, which I did not read case by case and do not claim a result for.
  Fails without the behavior: A missed `RetrySeed` literal, a new `RunContext` field not set, an unformatted file, a changed message or a wrong pin makes one of the three fail.
  Test seam: The whole workspace; no double. Nothing to add.
- Criterion 7: Read the diff of docs/decisions.md, then run the decisions and mirrors tests. Run: `git diff -U0 -- docs/decisions.md`. Observes: One hunk of added lines after the last line of the base file and no '-' lines. The new '## D159 · ' entry has a Provenance bullet and cites D10 by id. It says it reverses the left-out alternative (d) 'Automatic re-verification on the new base' because stale landings now recur with parallel jobs. It contains none of 'supersedes', 'superseded by' or 'amends' (checked with `grep -inE` on the new lines). `cargo test --locked -p athena-cli decisions` and `cargo test --locked -p athena-cli --test mirrors` pass.
- Criterion 8: Read the diff of CHANGELOG.md and README.md. Run: `git diff -U0 -- CHANGELOG.md README.md`. Observes: CHANGELOG.md has one [Unreleased] entry that describes only the diff: a stale job can be retried; a clean merge is re-verified and lands with no producer call; a conflict goes to a seeded producer run and a candidate that still holds conflict markers is closed before verification (marker-less conflicts such as modify/delete are left to the producer's explicit decision and the verifier); the guard message and pin changed; the help and briefs changed; and the `display::finished` hint 'ask again to redo the work' is unchanged. README.md no longer says stale jobs are refused: lines 48-58 and the `athena retry` table row say a stale job is retried by merging its candidate onto the tip, or from the base when it cannot be merged.
- Criterion 9: Tests in briefs.rs build an `EarlierAttempt` with `end_state: "stale"` and call `reader_brief`; a second test calls `producer_brief_with_evidence` with `RetrySeed.conflicts` non-empty. A test in main.rs's test module reads the retry help through `Cli::command()`. Run: `cargo test --locked -p athena-context stale && cargo test --locked -p athena-context conflict && cargo test --locked -p athena-cli retry_help`. Observes: The stale reader brief does not contain 'was rejected' or 'changes were discarded'. It says the job went stale, that its candidate is merged onto the tip when it can be, and that the retry starts from the base when it cannot. The retry help says the same. The producer brief with conflicts names the paths, says the markers must be resolved, and says a path without marker lines keeps only one side and needs an explicit decision.
  Fails without the behavior: Today `reader_brief` prints 'which was rejected' for any attempt that is not `failed` (briefs.rs:2206-2210) and 'Any changes that attempt made were discarded' (about 2250), so the new assertions fail.
  Test seam: `reader_brief(&ReaderInputs)` and `producer_brief_with_evidence` are called directly with the existing input builders in the briefs.rs tests (about 2483 and 2725). The clap definition at main.rs:44-53 is read through `Cli::command()`. These exist; steps 4 and 7 change the text.
- Criterion 10: List the changed files and read the diff. Run: `git status`. Observes: Exactly the 14 listed files change (confirmed with `git diff --stat`). snapshot.rs, candidate.rs, display.rs, docs/decision-links.toml, athena.toml, defaults.toml, workflows and xtask/shard-timings.txt are untouched. No added line holds a secret or an email address.

Planned files:

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
- crates/athena-cli/src/decisions.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c2606d07cde1 → candidate 6856f3c45878. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  26 ++
 README.md                            |  13 +-
 crates/athena-cli/src/decisions.rs   | 121 ++++++++--
 crates/athena-cli/src/main.rs        |  30 ++-
 crates/athena-cli/tests/lifecycle.rs | 457 +++++++++++++++++++++++++++++++++++
 crates/athena-cli/tests/retry.rs     |  96 +++++++-
 crates/athena-context/src/briefs.rs  | 142 ++++++++++-
 crates/athena-core/src/land.rs       |   7 +
 crates/athena-core/src/reading.rs    |  60 ++++-
 crates/athena-core/src/run/close.rs  |   5 +
 crates/athena-core/src/run/mod.rs    | 295 ++++++++++++++++++++--
 crates/athena-core/src/run/tests.rs  | 155 ++++++++++++
 crates/athena-git/src/lib.rs         | 160 ++++++++++++
 docs/decisions.md                    |  47 ++++
 14 files changed, 1541 insertions(+), 73 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-162047-hkx0/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-162047-hkx0/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42264154987 for 6856f3c45878f406586c21430e7b558152e0cc59)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42264154987 for 6856f3c45878f406586c21430e7b558152e0cc59)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42264154987 for 6856f3c45878f406586c21430e7b558152e0cc59)
- ci-result: `not a check` passed (Check results for 6856f3c45878f406586c21430e7b558152e0cc59 come from the CI run https://github.com/doodla/athena/actions/runs/42264154987 (687s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test --locked -p athena-cli --test retry a_stale_job_can_be_retried && cargo test --locked -p athena-core a_stale_job_is_accepted`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test lifecycle a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_call`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/land.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-git/src/lib.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 32.19s
     Running tests/lifecycle.rs (<job>/target/verify/debug/deps/lifecycle-81c3c517131d8419)

running 1 test
test a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_call ... FAILED

failures:

---- a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_call stdout ----

thread 'a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_call' (5220) panicked at crates/athena-cli/tests/lifecycle.rs:2251:58:
called `Result::unwrap()` on an `Err` value: job 20261106-175627-z3q5 is stale; only a rejected or failed job can be tried again (so can a timed out one), or one that was interrupted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_call

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 76 filtered out; finished in 7.27s

error: test failed, to rerun pass `-p athena-cli --test lifecycle`
error: 1 target failed:
    `-p athena-cli --test lifecycle`
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test --locked -p athena-cli --test lifecycle a_stale_job_whose_candidate_conflicts && cargo test --locked -p athena-core conflict_markers`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked -p athena-cli --test lifecycle a_stale_job_whose_candidate_merges_cleanly_lands_on_retry_with_no_producer_call && cargo test --locked -p athena-core land::`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli decisions`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/land.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-git/src/lib.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     Running tests/retry.rs (<job>/target/verify/debug/deps/retry-dfa3ca62f3670f5e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.00s

     Running tests/review.rs (<job>/target/verify/debug/deps/review-e2b31eef9d3c5417)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

     Running tests/status_build_measurement.rs (<job>/target/verify/debug/deps/status_build_measurement-86be37d815dff1dd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

     Running tests/status_durations.rs (<job>/target/verify/debug/deps/status_durations-a24da118fb9e96d2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

     Running tests/status_snapshot.rs (<job>/target/verify/debug/deps/status_snapshot-8e4f5018c0b5ea3b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/steward.rs (<job>/target/verify/debug/deps/steward-98cf1b798d3221f9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

     Running tests/trailer_stop.rs (<job>/target/verify/debug/deps/trailer_stop-985b0e3d0a0c45f2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/version.rs (<job>/target/verify/debug/deps/version-55574b07738a66c3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/watcher.rs (<job>/target/verify/debug/deps/watcher-5380c34aed879c72)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/work_decisions.rs (<job>/target/verify/debug/deps/work_decisions-a47b7f34852351f5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`cargo test --locked -p athena-context stale && cargo test --locked -p athena-context conflict && cargo test --locked -p athena-cli retry_help`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
