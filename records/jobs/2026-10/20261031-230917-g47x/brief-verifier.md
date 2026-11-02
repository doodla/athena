# Job 20261031-230917-g47x: verify "Keep a timed-out producer's edits as a candidate a retry resumes"

## Request (verbatim)

> Priority: high
> Keep the writer's work when it reaches its time limit
> 
> On 2026-10-31 three long jobs lost everything when the writer reached the 3000s deadline (the check-slowdown fix twice, and the GitHub-result job once). A deadline throws away the working copy, so a retry starts from zero (docs/decisions.md D124, alternative c).
> 
> When the producer reaches its deadline, commit its working copy as a kept candidate (as an environment stop already does), mark the job timed out with the kept candidate named, and let a retry resume from it: the next producer run starts from that candidate with a brief that says what was done and what the plan still needs.
> 
> Acceptance:
> - A test with a fake producer that times out after editing files shows the job keeps a candidate holding those edits.
> - A test shows a retry of that job starts its producer from the kept candidate, and its brief names the earlier attempt.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

When the producer reaches its deadline, its edits are kept, the job closes timed_out naming what was kept, and `athena retry` starts the next producer from them with a brief naming the earlier attempt. The goal is that long jobs stop losing work at the 3000s deadline; the audience is the owner running retries. The code already keeps a work tree and progress note, so the change closes the remaining gaps against the request.

Assumptions:
- Code survey: crates/athena-core/src/run/producer.rs (producer_loop calls keep_timed_out_work when the invocation outcome is TimedOut), crates/athena-core/src/run/mod.rs (keep_timed_out_work, timeout_seed, TimeoutSeed, the environment-stop resume mark in write_resume_mark/retry), crates/athena-core/src/state.rs (kept_work_dir), crates/athena-cli/tests/lifecycle.rs (deadline_script and the existing timed-out tests), crates/athena-data/src/records.rs (progress-note.md is in the JOB_MARKDOWN allow-list; no new record file is added, so check_record_file/AnyRecord/round-trip are unaffected).
- Existing state: at the base, a deadline already moves the work tree to a kept directory, writes progress-note.md, closes TimedOut, and a retry seeds the producer's workdir from it with a '## Resume' brief section. It does not commit the edits as a candidate as an environment stop does, and the closing names the kept directory only in the note.
- Reading: make the timed-out outcome keep a committed candidate (the same candidate and resume-mark mechanism an environment stop uses), name that candidate in the outcome reason and note, and make the retry's producer brief name the earlier job and what the plan still needs. If the producer finds the behavior fully present, it extends tests and docs and says so.
- Pinned studies keep nothing today and stay that way unless the candidate mechanism supports them.
- The request names no refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test lifecycle`: a test with a fake producer that edits files then times out shows the job closes timed_out and keeps a candidate (a commit, not only a work directory) whose diff holds those edits; the outcome reason or note names the kept candidate.
2. [behavior] Run `cargo test -p athena-cli --test lifecycle`: a test retries that timed-out job and shows the next producer starts with the kept edits present and its brief names the earlier job id and the files already changed, plus what the plan still needs.
3. [behavior] Run `cargo test`: the whole suite passes, including the history check on the records of a timed-out job.
4. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both are clean.
5. [content] CHANGELOG.md has an entry stating that a producer's work is kept when it reaches its deadline and that a retry resumes from it.
6. [content] docs/decisions.md records the call (keep the candidate on deadline) as a new decision referencing D124 alternative c, with sources read.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/state.rs (modify)

## Out of scope

- Changing producer_timeout_s in athena.toml
- Making the producer finish faster or splitting long jobs
- Retries of GitHub-result or check-slowdown jobs beyond the generic mechanism
- Keeping work for pinned study checkouts

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for keep and resume, plus fmt, clippy -D warnings and the suite.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the timeout and retry tests and check the candidate's edits and the brief text.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the deadline path, not per job.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and changelog written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the deadline and retry path.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Rework the existing lifecycle test a_producer_that_hits_its_deadline_keeps_its_work_and_closes_timed_out. The fake producer edits src/partial.rs and README.md and then returns outcome TimedOut. After go, assert state TimedOut. Read candidate.json at the closing head via record_at, require candidate_commit is Some, and read the file contents from that commit (git show_file) to see the edits. Assert the commit's parent is the job's base commit, and that the short commit is in the outcome reason and in the note. Run: `cargo test -p athena-cli --test lifecycle a_producer_that_hits_its_deadline`. Observes: The job closes timed_out with a committed candidate holding both edits; the reason and note name it; check_history has no violations.
  Fails without the behavior: At the base no candidate record is written for a timed-out job: the test's assertions that candidate.json has a candidate_commit whose diff against the base holds src/partial.rs and README.md, and that the outcome reason and progress-note.md contain the short commit, fail.
  Test seam: Entry point Athena::go via the existing lifecycle Fixture (fx.athena(script)); the fake producer is the existing deadline_script (athena_agent::fake::Script) injected into the Athena fake driver; the git repository is real. Exists (read in crates/athena-cli/tests/lifecycle.rs lines 2193-2282); the candidate assertions are new test code in step 6.
- Criterion 2: Rework retrying_a_timed_out_job_starts_from_the_kept_work_with_the_note. The first producer times out after editing the two files. athena.retry then go on the new job. The second fake producer records (workdir has src/partial.rs, user prompt). Assert the file is present, the prompt contains '## Resume', the first job's id, the kept candidate's short commit, both changed files, and a still-to-do line naming a planned file the first run left unchanged (CHANGELOG.md, which D89 adds to every change reading's deliverables; the test first asserts that deliverable is on the accepted reading). Also assert the kept directory does not exist after the first job closes (the commit is the keep). Keep retrying_a_timed_out_job_whose_base_moved_starts_clean unchanged: it must still start clean. Add a case where the commit is deleted and the kept directory remains is not needed; the fallback is covered by a core unit test in run/tests.rs. Run: `cargo test -p athena-cli --test lifecycle retrying_a_timed_out_job`. Observes: The retry's producer sees the earlier edits in its working directory and a brief that names the earlier job, the candidate, the changed files and what is still to do.
  Fails without the behavior: If the retry did not seed from the kept candidate, seen[0].0 (src/partial.rs present before the producer acts) would be false. If the brief lacked the earlier job id, the candidate commit or the still-to-do list, the brief assertions would fail; at base the brief has no 'still to do' section.
  Test seam: Entry points Athena::retry and Athena::go through the lifecycle Fixture; the fake producer is deadline_script with its Seen recorder (existing, lifecycle.rs lines 2189-2221). The new assertions are test code in step 6; the brief wording comes from briefs.rs (step 4).
- Criterion 3: Run the whole suite. It includes athena.check_history() on the timed-out job (existing assertion in the lifecycle test) and the unit tests added in run/tests.rs for the note text and the seed fallback. Run: `cargo test --locked`. Observes: All tests pass, including the history check on a job that closed timed_out with a candidate.
  Fails without the behavior: Without the fix to state.rs/producer.rs callers, a mismatch between the new candidate record and the history check, or the existing kept-work tests, would fail the suite; the history check assertion in the lifecycle test covers the timed-out job's records.
  Test seam: athena.check_history() on the lifecycle Fixture (exists, lifecycle.rs line 2279); unit tests call the pure note builder and timeout_seed in crates/athena-core/src/run/tests.rs (the builder is added in step 3).
- Criterion 4: Run fmt and clippy with -D warnings. The verifier runs `cargo clippy --all-targets --locked -- -D warnings` as a second command. Run: `cargo fmt --all --check`. Observes: Both exit 0 with no diagnostics.
  Fails without the behavior: Unformatted code or any clippy warning (for example an unused kept_work_dir import, or a needless clone in the new helper) fails the command.
  Test seam: None needed: a lint check, not a behavior.
- Criterion 5: Read the diff and check the Unreleased section for an entry that says a producer's work is kept as a candidate when it reaches its deadline, the job closes timed_out naming it, and `athena retry` resumes from it with a brief naming the earlier job, the changed files and what is still to do. Run: `git diff -- CHANGELOG.md`. Observes: The entry is present under [Unreleased] and written for an outside reader.
  Fails without the behavior: Without the entry the diff of CHANGELOG.md is empty.
  Test seam: None needed: a content check.
- Criterion 6: Read the diff for a new decision after D150 (title: the producer's work is kept as a candidate at its deadline) citing D124 alternative (c), the decision, alternatives (no resume mark, directory-only keep, delete directory mechanism), tradeoff, reversal cost, and provenance listing files read (run/mod.rs, run/producer.rs, run/candidate.rs, state.rs, briefs.rs, lifecycle.rs, reading.rs, disk.rs) and what was not measured. Run: `git diff -- docs/decisions.md`. Observes: The decision is present, numbered after D150, and references D124 (c).
  Fails without the behavior: Without the record the diff of decisions.md is empty.
  Test seam: None needed: a content check.

Planned files:

- crates/athena-core/src/run/candidate.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cb137d6d181a → candidate cc530d2b63a3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  16 +++
 crates/athena-cli/tests/lifecycle.rs    | 101 +++++++++++++++--
 crates/athena-context/src/briefs.rs     |  30 ++++-
 crates/athena-core/src/reading.rs       |   1 +
 crates/athena-core/src/run/candidate.rs | 101 +++++++++++------
 crates/athena-core/src/run/mod.rs       | 191 +++++++++++++++++++++++++-------
 crates/athena-core/src/run/producer.rs  |  12 +-
 crates/athena-core/src/run/tests.rs     | 134 ++++++++++++++++++++++
 crates/athena-core/src/state.rs         |   7 +-
 docs/decisions.md                       |  51 +++++++++
 10 files changed, 543 insertions(+), 101 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41419187728 for cc530d2b63a348516a5610d8bfa49242ad5565e0)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41419187728 for cc530d2b63a348516a5610d8bfa49242ad5565e0)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41419187728 for cc530d2b63a348516a5610d8bfa49242ad5565e0)
- ci-result: `not a check` passed (Check results for cc530d2b63a348516a5610d8bfa49242ad5565e0 come from the CI run https://github.com/doodla/athena/actions/runs/41419187728 (425s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: both are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
