# Job 20261025-132633-06hy: verify "Wait only for the 'fmt, clippy, tests' check run before landing push"

## Request (verbatim)

> Priority: urgent
> The wait before the landing push (gh.rs around line 2048) stops at the first failed check run on the early candidate, but on a candidate 'the full history' and 'the pull request head' always fail (a candidate is not a landing), so it never waits for the job the cache serves: on PR for job 20261025-033731-5hac it gave up after 140 s with 'a run failed' while 'fmt, clippy, tests' passed at 13:10, and the landing took 18 minutes again. Wait only for the check run named 'fmt, clippy, tests' on the candidate; ignore the other jobs' conclusions there.
> Acceptance:
> - A fake-host test where the candidate's history job fails and 'fmt, clippy, tests' is still running shows land waiting for 'fmt, clippy, tests' to complete before pushing.
> - A test where 'fmt, clippy, tests' itself failed pushes at once.
> Files: crates/athena-core/src/gh.rs, CHANGELOG.md

## Reading (accepted by the user)

The pre-push wait in gh.rs (wait_for_early_run) ends at any failed workflow run on the early candidate. Because the candidate's history and PR-head jobs always fail, it never waits for the cached job. Read the candidate's check run named 'fmt, clippy, tests' only: wait until it completes, push at once if it failed, and ignore the other jobs' conclusions. The audience is the maintainer running land, who wants it to stop taking 18 minutes.

Assumptions:
- Workflow-run data (ci_runs) cannot name a job, so a PrHost method reads the commit's check runs (the check-runs endpoint already used by checks) and picks the one named 'fmt, clippy, tests'; all PrHost implementations (real, test_support fakes, gh.rs test hosts) are updated.
- A missing named check run is treated like no runs: push at once; timeout and unreadable replies behave as before.
- The 20-second step and the bound are unchanged.

## Acceptance criteria

1. [behavior] A new fake-host test in gh.rs: the candidate has a failed history job and 'fmt, clippy, tests' in progress then completed; land polls (sleeps) until 'fmt, clippy, tests' completes and only then replaces the early head. Run cargo test -p athena-core and see it pass.
2. [behavior] A new test: 'fmt, clippy, tests' on the candidate is already failed, with the other jobs in any state; land pushes at once with no sleeping and the note says that job failed. Run cargo test -p athena-core.
3. [behavior] Existing early-run tests (waits before pushing, pushes at once when completed, pushes anyway on timeout, unreadable or no runs) still pass, and cargo clippy and cargo fmt --check are clean.
4. [content] wait_for_early_run no longer ends on any failed workflow run; it decides only from the check run named 'fmt, clippy, tests' (a constant), found through the host seam, and its doc comment says so and why (a candidate is not a landing).
5. [content] CHANGELOG.md has an entry describing the changed wait.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Changing the CI workflow or job names
- Changing wait_for_checks after the landing push
- Changing the timeout length

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus clippy and fmt clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise wait versus push-at-once through the fake host.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the wait's selection of runs, not the timeout.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in gh.rs and the changelog.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in gh.rs using RunHost whose script returns, per look, the named job in_progress, in_progress, completed/success, and a failed 'the full history' job each time. Asserts the log is check_run early1, sleep 20, check_run early1, sleep 20, check_run early1, replace, and the note says waited 40s and completed. Run: `cargo test -p athena-core land_pr_waits_for_the_named_check_run_when_the_history_job_failed`. Observes: land sleeps until 'fmt, clippy, tests' completes and only then replaces the early head, despite the failed history job.
  Fails without the behavior: The old code stops at the failed history run on the first look, so the log would be one look then replace with no sleeps and the test fails.
- Criterion 2: New test: named job completed/failure (and a variant with the other jobs in_progress); asserts log is exactly [check_run early1, replace], no sleep entry, and the note contains 'fmt, clippy, tests' and 'failed'. Run: `cargo test -p athena-core land_pr_pushes_at_once_when_the_named_check_run_failed`. Observes: Immediate push with no sleeping and a note that names the failed job.
  Fails without the behavior: If the wait did not read the named job's failure it would sleep while the other job runs, so a sleep entry would appear.
- Criterion 3: Existing early-run tests (waits before pushing, pushes at once when completed, pushes anyway on timeout, failure/no-run/unreadable) are adapted to script the named check run through RunHost; run the full suite and lint. Run: `cargo test -p athena-core && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`. Observes: All pass; the timeout test still shows 3 sleeps at a 60 s bound and the default bound is 600 s.
  Fails without the behavior: A wait that ignored the named job's in-progress state or bound would break the waits and timeout tests.
- Criterion 4: Inspect the diff: wait_for_early_run calls job.host.check_run(early_head, EARLY_RUN_JOB), contains no CiRunState::failed over ci_runs, and its doc comment says it reads only that job because a candidate is not a landing so the other jobs always fail there. Also grep that ci_runs is no longer called in wait_for_early_run. Run: `git diff crates/athena-core/src/gh.rs`. Observes: The constant, the seam call and the doc comment are present; no workflow-run conclusion is used.
- Criterion 5: Inspect the diff for a new entry under Unreleased describing that land waits only for the candidate's 'fmt, clippy, tests' check run and ignores other jobs. Run: `git diff CHANGELOG.md`. Observes: The entry is present in the Unreleased section.

Planned files:

- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- CHANGELOG.md (modify)

## The change

Base f37b9bcaac19 → candidate 84308a19486b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   5 +
 crates/athena-core/src/gh.rs           | 190 ++++++++++++++++++++++++++++-----
 crates/athena-core/src/test_support.rs |   8 ++
 3 files changed, 174 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1230 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
