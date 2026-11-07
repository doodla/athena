# Job 20261106-145032-rcvz: Fix the stand-in drain race in the drain pid-file stop test

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

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-145032-rcvz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-145032-rcvz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-cli status_and_stop_find_the_drain_by_pid_file`: the fixed test passes, and a new test that makes the stand-in exit before the waiting stop_and_wait call passes deterministically by asserting the old failure outcome NotRunning(Dead(pid)).
In the diff, the fixed test's stand-in does not exit on the stop file alone; it also waits for a release file that the test creates after the waiting stop_and_wait call has started, with no reliance on sleeps to win the race.
The original test's assertions on the pid file record, drain_status output, Asked outcome, stop file existence, Exited(record) result, and decoy untouched remain with unchanged meaning.
Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass and are clean.
CHANGELOG.md has an entry describing only what the diff does (a test race fixed; no change to drain-stop behavior), and only drain.rs and CHANGELOG.md change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
