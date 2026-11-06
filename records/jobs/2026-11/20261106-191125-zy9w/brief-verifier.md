# Job 20261106-191125-zy9w: verify "Retry: store failure lines, not CI cleanup, in failed-check tail"

## Request (verbatim)

> try again on 20261106-081018-0j4b

## Reading (accepted by the user)

Retry of job 20261106-081018-0j4b (stale: branch moved). Re-apply its verified fix so detail_from_job_logs cuts post-job cleanup and keeps failure lines, merged onto the new tip, and address its findings, mainly by showing the new tests fail on base.

Assumptions:
- Job 20261106-081018-0j4b passed verification but went stale (branch moved, e49abd98ebd1 touched governed content); its candidate is merged onto the tip. Its four changed files are kept as deliverables.
- Finding 1 (new tests not shown failing on base because they referenced missing helpers): addressed by putting the gh tests in crates/athena-core/src/gh/tests.rs against only the public detail_from_job_logs, so they compile on base and fail on assertions. Helper unit tests in checks.rs may stay but are not the base-failure evidence.
- Finding 2 (failure block keeps only marker plus 4 lines): fixed by extending a block until a blank line or next `----` header, within the n/2 cap. Finding 3 (moved lines look like last output): kept as a documented trade-off; the separator line stays and the comment states it.
- Code survey: read crates/athena-core/src/gh/client.rs (detail_from_job_logs, line 248, tails the whole job log) and crates/athena-core/src/checks.rs (tail_lines, line 193, plus its tests). No record or data file is added, so records validation, check_record_file and AnyRecord are not touched. The property is measured directly by calling detail_from_job_logs on a synthetic log and inspecting output_tail.
- Audience: Athena maintainers and later introspections diagnosing failed CI checks. The dropped candidates from 20261106-022423-vdtn (config ceiling) and 20261106-045349-f6x6 (mirrored_sources) stay dropped.
- If the merge conflicts with the tip, the producer resolves it from the conflicted merge; the shared child-process script writer change is untouched.

## Acceptance criteria

1. [content] CHANGELOG.md under Fixed and a code comment in crates/athena-core/src/gh/client.rs or checks.rs name the root cause: the stored tail was taken of the whole CI log, which ends with post-job cleanup, and failure lines were not selected.
2. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a test in gh/tests.rs with a failing test followed by a post-job cleanup block asserts output_tail contains the FAILED/panicked at lines and no cleanup text. It compiles on base using only existing public items and fails on assertions there.
3. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a test with a log yielding no parsed test names, followed by cleanup, asserts output_tail keeps lines with `test result`, `FAILED`, `failures:` or `panicked at`.
4. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a test with a multi-line panic message longer than 4 lines asserts the whole message survives in output_tail.
5. [behavior] Run `cargo test -p athena-core`: existing tail_lines, 200-line and byte cap, FAILING_UNITS_PREFIX and redaction tests pass, and a log with no failure lines and no cleanup keeps its plain tail.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass.
7. [content] Only crates/athena-core/src/gh/client.rs, gh/tests.rs, checks.rs and CHANGELOG.md change; no config, records, workflow or governance files change.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The max_concurrency ceiling change from job 20261106-022423-vdtn (config.rs, docs, athena.toml): dropped
- The mirrored_sources plan field, admission rule and prompt changes from job 20261106-045349-f6x6: dropped
- Fixing the flaky xtask repin test or drain tests
- Editing records/, athena.toml, defaults.toml or CI workflows
- Changing the local (non-CI) failed-check tail behavior beyond what the shared helper needs
- Reworking the e49abd98ebd1 child-process script writer change

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy, test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in detail_from_job_logs, where the tail is taken, not per job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests on synthetic logs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in four files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier reruns the cargo checks.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 19ff5bf7d411 → candidate 6f6ac70b21fe. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   7 ++
 crates/athena-core/src/checks.rs    | 156 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/gh/client.rs |  10 ++-
 crates/athena-core/src/gh/tests.rs  | 129 +++++++++++++++++++++++++++++
 4 files changed, 300 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-191125-zy9w/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-191125-zy9w/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42277518463 for 6f6ac70b21fed4a57e83422376b013f3d19005d2)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42277518463 for 6f6ac70b21fed4a57e83422376b013f3d19005d2)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42277518463 for 6f6ac70b21fed4a57e83422376b013f3d19005d2)
- ci-result: `not a check` passed (Check results for 6f6ac70b21fed4a57e83422376b013f3d19005d2 come from the CI run https://github.com/doodla/athena/actions/runs/42277518463 (725s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
