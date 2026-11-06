# Job 20261106-080857-9daz: verify "Fix flaky drain pid-file test by removing its stop-file race"

## Request (verbatim)

> Fix the flaky test status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain in crates/athena-cli/src/drain.rs.
> 
> It has failed the checks of two unrelated jobs on 6 Nov (a refusal-replay change and a planner convention change) and passed on re-runs of the same code. It starts real processes and reads their start time and command line from /proc, so it is likely sensitive to timing when the machine is busy (two jobs' checks run at once now).
> 
> Find the cause from the code and from a reproduction under load (for example the test run many times while other cargo tests run), state it, and fix it so the test is deterministic. Do not weaken what it checks: the pid file must still be the only way the drain is found, and a decoy whose command line also reads 'athena drain' must still be ignored.
> 
> Acceptance:
> - the cause is stated in the job report with the evidence that shows it
> - the test passes 200 runs in a row under load (recorded check output shows the loop)
> - the decoy and pid-file assertions are unchanged in meaning
> - CHANGELOG states only what the diff does
> 
> Files changed: crates/athena-cli/src/drain.rs (tests, and code only if the cause is there), CHANGELOG.md.

## Reading (accepted by the user)

Make status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain deterministic. Working hypothesis: stop_and_wait writes the stop file before it checks the state, and the test's `sh` drain polls that same file every 50 ms. Under load the drain can exit before the "asked only" call checks it, so that call returns NotRunning instead of Asked. The producer must confirm this by reproducing it under load, state the cause and evidence, fix the test, and add a CHANGELOG line. The audience is the maintainers whose checks fail on unrelated jobs.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs. The test is at about line 5051. Its helpers are sleeper, record_of, quick and settled (about lines 4968-5012). The code under test is drain_state (3496), stop_and_wait (3696), RealProcs::start_marker and linux_marker (3401-3431). drain_state reads the pid file, then compares the /proc/<pid>/stat start-time field with the recorded value. It never reads the command line, so the decoy is ignored simply because it is not in the pid file. No record or data file is added, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly by running the real test in a loop.
- Hypothesised cause: the test's shell loop exits on the stop file within 50 ms. stop_and_wait writes that file and only then calls drain_state. Under CPU contention the shell can exit and become a zombie in that gap. linux_marker then returns None, so the first call returns NotRunning(Dead) rather than Asked, and the assertion fails. The producer must confirm or correct this by reproduction. The fix, if confirmed, changes only the test's synchronisation, for example a drain stand-in that exits only on a second signal file the test creates after the asked-only assertions, while the real stop file is still what is written and checked.
- No production code change is expected unless the reproduction shows the cause is in drain.rs non-test code.
- Reproduction under load uses cargo test in a loop alongside other cargo tests. The run is recorded as check output.
- CHANGELOG.md gets one entry describing only the test change in the diff.

## Acceptance criteria

1. [content] The job report states the cause of the flake and the evidence for it: reproduction output under load, plus the code lines involved in crates/athena-cli/src/drain.rs.
2. [behavior] Run `cargo test -p athena-cli status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain` 200 times in a row while other cargo tests run in parallel. Every run passes, and the recorded check output shows the loop.
3. [content] In the diff, the test still writes a pid file naming only the real drain child. It still starts a decoy with arg0 'athena drain --budget 9' that is not in the pid file. It still asserts that drain_state returns Running(record), that the status text has the drain pid and not the decoy pid, that the stop outcomes are Asked and then Exited, and that the decoy is still alive at the end.
4. [content] CHANGELOG.md has an entry that describes only what the diff does, with no claims beyond it.
5. [behavior] Run `cargo test -p athena-cli`, `cargo clippy --all-targets` and `cargo fmt --check`. All pass with no new warnings.
6. [content] Only crates/athena-cli/src/drain.rs and CHANGELOG.md are changed. The change adds no new refusal rule, so no refusal-rate figure applies.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how drain_state or stop_and_wait detect the drain, unless the reproduction proves the cause is there.
- Fixing other tests in drain.rs.
- Changes to athena.toml, defaults.toml, workflows or other governance files.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The fix is Rust test code.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A 200-run loop under load is recorded.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is reproduced and stated, not hidden with retries or longer timeouts.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the test passing in a loop.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Two files only.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments and CHANGELOG are written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4c3a212f838f → candidate 320e1a46a510. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                   |  6 ++++++
 crates/athena-cli/src/drain.rs | 12 ++++++++++--
 2 files changed, 16 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-080857-9daz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-080857-9daz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42210150030 for 320e1a46a5109324586305c311d540ce6bd6b906)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42210150030 for 320e1a46a5109324586305c311d540ce6bd6b906)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42210150030 for 320e1a46a5109324586305c311d540ce6bd6b906)
- ci-result: `not a check` passed (Check results for 320e1a46a5109324586305c311d540ce6bd6b906 come from the CI run https://github.com/doodla/athena/actions/runs/42210150030 (711s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "The job report states the cause of the flake and the evidence for it: reproduction output under load, plus the code lines involved in crates/athena-cli/src/drain.rs.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
