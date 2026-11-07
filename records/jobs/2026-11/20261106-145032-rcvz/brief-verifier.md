# Job 20261106-145032-rcvz: verify "Fix the stand-in drain race in the drain pid-file stop test"

## Request (verbatim)

> Fix the race in the test status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain (crates/athena-cli/src/drain.rs). The cause is known.
> 
> GitHub's run on main (6 Nov, run 42231304557) failed at drain.rs:5096 with left NotRunning(Dead(pid)), right Exited(record). The test's stand-in drain is a shell loop that exits as soon as the stop file exists, polling every 0.05 s. The first stop_and_wait call (no wait) creates the stop file; the second call (with a wait) expects to find the process still running and watch it exit. When the machine is busy, the stand-in has already exited before the second call starts, so the second call sees a dead process instead.
> 
> Fix it so the outcome does not depend on that timing: either the stand-in keeps running until the test releases it (for example it waits for a second file the test creates after the second call has started), or stop_and_wait reports Exited when the pid file's process has exited after a stop was asked. Choose whichever keeps the drain-stop --wait behaviour right for a real user, and say why in the report.
> 
> Acceptance:
> - a test reproduces the old failure deterministically (it makes the stand-in exit before the waiting call) and shows the new code or test passes it
> - the original test's checks on the pid file and the decoy are unchanged in meaning
> - all tests pass; CHANGELOG states only what the diff does
> 
> Files changed: crates/athena-cli/src/drain.rs, CHANGELOG.md.

## Reading (accepted by the user)

Make the test's stand-in drain wait for a second "release" file that the test creates after the waiting stop_and_wait has started, so the outcome no longer depends on timing. Add a deterministic test showing the old failure when the stand-in exits before the waiting call. Production stop_and_wait stays unchanged, so a dead pid still reports NotRunning(Dead). Goal: a stable CI for the maintainers.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs: stop_and_wait (~line 3776) writes the stop file, calls drain_state, returns NotRunning if the pid file's process is dead, Asked with no wait, otherwise polls until the process differs and returns Exited. The test is at ~line 5137; its stand-in is `sh -c` looping until the stop file exists. No record or data file is added, so the records allow-list is not involved.
- Chosen fix is test-side: a dead process with no prior observation of it running cannot be told from a crashed or stale drain, so changing stop_and_wait to report Exited could hide a stale pid file from a real `drain-stop --wait` user. The report must say this.
- The test releases the stand-in from a thread or after a delayed start of the waiting call, e.g. the stand-in loops until a second file exists and the test creates it from a helper thread once the wait has begun (the stop file already exists, so the release is the only thing that ends it).
- The reproduction test makes the stand-in exit (creates both files and reaps or waits for it) before calling stop_and_wait with a wait, and asserts the NotRunning(Dead) outcome that caused the CI failure, showing the old test shape was timing-dependent.
- CHANGELOG.md gets one entry stating only what the diff does (test made deterministic, no behavior change). athena.toml, defaults.toml and workflows are not changed.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli status_and_stop_find_the_drain_by_pid_file`: the fixed test passes, and a new test that makes the stand-in exit before the waiting stop_and_wait call passes deterministically by asserting the old failure outcome NotRunning(Dead(pid)).
2. [content] In the diff, the fixed test's stand-in does not exit on the stop file alone; it also waits for a release file that the test creates after the waiting stop_and_wait call has started, with no reliance on sleeps to win the race.
3. [content] The original test's assertions on the pid file record, drain_status output, Asked outcome, stop file existence, Exited(record) result, and decoy untouched remain with unchanged meaning.
4. [behavior] Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass and are clean.
5. [content] CHANGELOG.md has an entry describing only what the diff does (a test race fixed; no change to drain-stop behavior), and only drain.rs and CHANGELOG.md change.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing stop_and_wait behavior or the drain-stop CLI
- Other flaky tests in the repository
- Governance files and CI workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fix and tests written in Rust
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Removes the timing dependency rather than adding sleeps or retries
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests and check outcomes
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Two files only
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2980ff659c0c → candidate 3a4ca7d9c632. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                   |  4 +++
 crates/athena-cli/src/drain.rs | 79 +++++++++++++++++++++++++++++++++++++-----
 2 files changed, 74 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-145032-rcvz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-145032-rcvz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42422890539 for 3a4ca7d9c63227b78f88e639679e35c490d4ca31)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42422890539 for 3a4ca7d9c63227b78f88e639679e35c490d4ca31)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42422890539 for 3a4ca7d9c63227b78f88e639679e35c490d4ca31)
- ci-result: `not a check` passed (Check results for 3a4ca7d9c63227b78f88e639679e35c490d4ca31 come from the CI run https://github.com/doodla/athena/actions/runs/42422890539 (651s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass and are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
