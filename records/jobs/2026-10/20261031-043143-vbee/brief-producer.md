# Job 20261031-043143-vbee: Time workspace tests once and fix the single slowest test cause

## Request (verbatim)

> try again on 20261031-043018-kjy2. Narrowed: find the slowest tests from one timed run and fix only the biggest cause.
> Priority: urgent
> 
> Both earlier attempts (9h6z, kjy2) hit the 3000s producer deadline because each full test run takes about 15 minutes and the job ran it several times. This attempt has a strict budget: run the full test suite at most twice, once to measure and once to confirm.
> 
> 1. Run the workspace tests once with per-test timing, for example `cargo test --workspace -- -Z unstable-options --report-time` is not available on stable, so use the elapsed time each test binary prints ("finished in N s") and `--test-threads` as the checks stage uses it. Save the ten slowest test binaries and, inside the slowest binary, the slowest tests, into the job report.
> 2. Fix only the single biggest cause, if it is a test that waits, sleeps, retries, builds a large fixture repeatedly or spawns a process it does not need. Keep what the test covers.
> 3. Run the full suite a second time and report the before and after time of the binary you changed and of the whole run.
> If the biggest cause cannot be fixed without losing coverage, change nothing and report the timings; that is an acceptable outcome for this job.
> 
> Acceptance:
> - The job report lists the ten slowest test binaries with their times from one full run of the base.
> - Either one change makes the changed test binary at least 30% faster with the same tests passing, shown by before and after times, or the report explains why the biggest cause cannot be fixed without losing coverage and changes no code.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry if code changed.

## Reading (accepted by the user)

Retry of 20261031-043018-kjy2 (and 9h6z), narrowed per the user: one timed full test run to list the ten slowest test binaries, then one fix of the biggest cause inside the slowest binary, or no code change if coverage would be lost. The CHANGELOG entry carries the timings. Wider cause hunting across crates and build settings is left to follow-up requests.

Assumptions:
- Code survey: read crates/athena-cli/tests/drain.rs, crates/athena-cli/src/drain.rs, crates/athena-core/src/checks.rs and crates/athena-agent/src/procs.rs by search; sleep calls and process spawns cluster in crates/athena-cli/tests/drain.rs and crates/athena-agent/src/procs.rs, the likely slow tests. Per-binary 'finished in N s' lines from cargo test are the direct measure; no validation allow-list or record file is added.
- Prior jobs: kjy2's and 9h6z's candidates changed nothing (no candidate), so there is no work to keep or drop; their deliverables other than the changed test file and CHANGELOG.md (Cargo.toml files, status_durations.rs, run/tests.rs) are dropped as follow-ups unless the timings show they are the slowest binary.
- Goal: the maintainer shrinks check time. Audience: the maintainer and verifier reading CHANGELOG.md. The producer runs the full suite at most twice (measure, confirm).
- Scope kept to one deliverable (one test file's fix plus its CHANGELOG entry) so the job fits inside the verifier timeout; a no-fix outcome with only the CHANGELOG timings is acceptable.
- The fix is judged by 'finished in' times of the changed test binary, at least 30% faster with the same tests passing.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry listing the ten slowest test binaries with 'finished in' times from one full run of the base, and the slowest tests inside the slowest binary.
2. [content] Either the entry gives before and after times of the one changed test binary showing at least 30% faster plus before and after times of the whole run, or it explains why the biggest cause cannot be fixed without losing coverage and no code changes (git diff shows only CHANGELOG.md).
3. [behavior] Run `cargo test --workspace`: all tests pass, and the test count is not lower than before. git diff shows no removed #[test] functions and no added #[ignore].
4. [behavior] Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.
5. [content] git diff touches at most one test file plus CHANGELOG.md; athena.toml, Cargo.toml files and check_timeout_s are unchanged.

## Expected deliverables

- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/status_durations.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Changing check_timeout_s or other values in athena.toml
- Restructuring or parallelizing pipeline stages
- Deleting tests or marking them #[ignore]
- Fixing more than the single biggest cause; further causes are follow-up requests
- Changing Cargo.toml or crates/athena-core/Cargo.toml build settings (follow-up)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the changed binary's before/after time, not the activity.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy -D warnings and fmt --check must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single cause, single test file.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix only the biggest cause found by timing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Same tests pass; no tests deleted or ignored.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Changes are Rust tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 39 cases in 39 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict gate in the job pipeline. After the producer finishes, Athena itself runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` on the workspace. It records each command with its exit status. If any command fails, it sends the failure output back to the ...
- 14 cases in 12 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make disk a managed budget. Before a job is held, or when free disk drops below the need, Athena reclaims what it can on its own. That means pruning the build folders of held jobs and the job's own writer build once a recoverable form exists, such as the committed diff or a patch saved in the job's ...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

23 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md has a new entry listing the ten slowest test binaries with 'finished in' times from one full run of the base, and the slowest tests inside the slowest binary.
Either the entry gives before and after times of the one changed test binary showing at least 30% faster plus before and after times of the whole run, or it explains why the biggest cause cannot be fixed without losing coverage and no code changes (git diff shows only CHANGELOG.md).
Run `cargo test --workspace`: all tests pass, and the test count is not lower than before. git diff shows no removed #[test] functions and no added #[ignore].
Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: both are clean.
git diff touches at most one test file plus CHANGELOG.md; athena.toml, Cargo.toml files and check_timeout_s are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
