# Job 20261027-160815-k3wz: verify "Limit previous-result check to the job a retry or follow-up continues"

## Request (verbatim)

> Priority: urgent
> The previous-result check added in commit d510d97 refuses a reading whenever the request text names an earlier job id but the reading does not restate that job's result. It was meant for retries and follow-ups of a job, but it also fires on requests that cite earlier jobs only as evidence, such as every root-cause request the error review writes (they cite the introspections of past jobs). On 2026-10-27 it refused the reading of a root-cause request twice and stopped the drain; the reader is not given those jobs' results, so it cannot satisfy the check. Fix: apply the check only to the job a retry or follow-up continues (its previous_job link, or a request that says it follows up a named job), and give the reader those jobs' recorded results; requests that cite jobs as evidence are read as before.
> Acceptance:
> - A test reads a root-cause request that cites two past jobs' introspections as evidence and shows the reading is accepted without restating their results.
> - A test shows a retry whose reading omits the previous attempt's result is still refused, and one that includes it is accepted.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/reading.rs, CHANGELOG.md

## Reading (accepted by the user)

The previous-result check in reading.rs currently runs over every job id cited in the request text. It should run only for the retried job (previous_job) and for jobs a request explicitly says it follows up. Jobs cited as evidence, such as the introspections in error-review root-cause requests, are shown to the reader as before and are not checked. The audience is the drain operator, who needs the drain to stop refusing those readings.

Assumptions:
- Follow-up detection is a small, documented phrase match on the request text, such as 'follows up job <id>' or 'follow-up of <id>'. It names the continued job. Other cited ids are evidence.
- The retried job (the `attempt` argument of read_for, from previous_job) is always checked and its recorded result is given to the reader as now.
- A followed-up job is carried to the reader with its recorded result, as the existing referenced_jobs carrying does. Evidence jobs are still carried but are not passed to check_prior_result_and_scope.
- The scope check (deliverables and out_of_scope) keeps applying only when there is at least one continued job.
- Only reading.rs and CHANGELOG.md change. No prompt files change, because the reader prompt has 3 tokens of headroom.

## Acceptance criteria

1. [behavior] A new test in reading.rs reads a root-cause request that cites two past jobs' introspections as evidence, with no previous_job and no follow-up wording. The reading does not restate those jobs' results and is accepted. Run `cargo test -p athena-core reading`.
2. [behavior] A new test shows that a retry whose reading omits the previous attempt's job id and result is still refused with 'the previous result'. A second test shows that the same retry with a reading that names the previous job is accepted. Run `cargo test -p athena-core reading`.
3. [behavior] A test shows that a request saying it follows up a named job is checked for that job only. A reading that omits the followed-up job is refused. A different job cited in the same request as evidence is not required to be named.
4. [content] read_for passes check_prior_result_and_scope only the retried job and the explicitly followed-up jobs. Evidence-only referenced jobs are still given to the reader in the brief. The doc comments describe this distinction.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including the existing prior-result tests.
6. [content] CHANGELOG.md has an entry describing that the previous-result check applies only to the continued job and no longer refuses evidence citations.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the reader prompt files or the token budgets
- Changing how the error review writes its requests
- Changing the scope check (deliverables and out_of_scope) rules for genuine retries
- Changing the MAX_REFERENCED_JOBS limit or the redaction of carried jobs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests are added, and tests, clippy and fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The tests exercise accepted and refused readings rather than checking that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is scoped to the job being continued, not special-cased for root-cause requests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in one source file plus the changelog.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that the drain-stopping reading is accepted.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 82c1d7e274b3 → candidate 1639047fbb1b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  10 ++
 crates/athena-core/src/reading.rs | 228 +++++++++++++++++++++++++++++++++++++-
 2 files changed, 232 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1486 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including the existing prior-result tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
