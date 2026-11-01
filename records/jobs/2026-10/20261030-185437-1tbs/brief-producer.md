# Job 20261030-185437-1tbs: Reclaim interrupted jobs' build dirs; gate disk before the producer

## Request (verbatim)

> Priority: high
> Build directories of interrupted jobs are never reclaimed, so a crash fills the disk on the next run. On 2026-10-30 the container restarted while a producer was running; its job stayed Dispatched with its build directory in place, drain's cleanup only reclaims closed and idle jobs, and the next job's producer died when the disk filled (0 MB free), twice, stopping the drain.
> Change: at drain start, and before each producer run, reclaim the build directory of every job whose process is gone (interrupted), keeping its work tree and candidate; and check free disk before the producer as well as before the checks.
> Acceptance:
> - A test through the drain entry point with an interrupted job's build directory present shows it reclaimed at drain start while its work tree remains.
> - A test shows a low-disk condition before the producer holds the job as an environment stop instead of letting the producer start.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs, CHANGELOG.md

## Reading (accepted by the user)

A crash leaves a Dispatched job's build directory behind and nothing reclaims it, so the next job's producer fills the disk. Drain will reclaim build directories of jobs whose process is gone (keeping work tree and candidate) at drain start and before each producer run, and will check free disk before the producer as well as before the checks. The audience is whoever runs `athena drain` unattended.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs (gate, sweep_idle_targets, disk_gate, LowDisk, LOW_DISK, df_free_bytes with the FREE_BYTES_ENV test hook, is_environment_reason, resumable_stop) and crates/athena-cli/src/drain.rs (module docs on low disk holds; held handling near the LowDisk downcast). Today the gate runs at dispatch and before checks; sweep_idle_targets reclaims only closed and idle jobs. The new sweep reuses the interrupted-process notion the drain already uses for Dispatched jobs.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- The change adds a sweep for interrupted jobs in disk.rs, called from drain.rs at drain start and before each producer run, plus a disk_gate call before the producer. Low disk before the producer is held as an environment stop with the existing LowDisk text.
- The simulated free-space hook FREE_BYTES_ENV is used by tests for the low-disk case; the work tree and candidate of interrupted jobs are never deleted.
- No governance file (athena.toml, defaults.toml, workflows) needs changing.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/src/drain.rs, run with `cargo test -p athena-cli`, enters through the drain entry point with an interrupted Dispatched job's build directory present, and asserts the build directory is gone after drain start while the job's work tree and candidate remain.
2. [behavior] A new test, run with `cargo test`, simulates low free disk before the producer (via the free-bytes hook) and asserts the job is held as an environment stop (low disk) and the producer is never started.
3. [behavior] A test shows a build directory of a job whose process is still alive is not reclaimed, and a sweep before a second producer run reclaims a newly interrupted job's directory.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
5. [content] CHANGELOG.md has a new entry describing the reclaim of interrupted jobs' build directories and the pre-producer disk check; `git diff` shows it.
6. [content] The diff touches only crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs and CHANGELOG.md (plus tests inside those files), and the module docs in drain.rs and disk.rs describe the new behavior.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Deleting work trees or candidates of interrupted jobs
- Changing the min_free_gb default or athena.toml / defaults.toml
- Changing how closed and idle jobs are reclaimed
- Automatic retry policy changes for held jobs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy -D warnings and fmt are criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run through cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests check the directory removed and the producer not started.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Reclaims every interrupted job, not only the one that surfaced.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Three files, one focused change.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 37 cases in 37 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the producer's work goes to the verifier, run the full check set (`cargo test`, clippy, fmt) automatically and send any failure back to the producer. At the same time, record a baseline of the same checks on the unchanged base so a failure is attributed to the job or to the base. Add a plan-t...
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

A new test in crates/athena-cli/src/drain.rs, run with `cargo test -p athena-cli`, enters through the drain entry point with an interrupted Dispatched job's build directory present, and asserts the build directory is gone after drain start while the job's work tree and candidate remain.
A new test, run with `cargo test`, simulates low free disk before the producer (via the free-bytes hook) and asserts the job is held as an environment stop (low disk) and the producer is never started.
A test shows a build directory of a job whose process is still alive is not reclaimed, and a sweep before a second producer run reclaims a newly interrupted job's directory.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has a new entry describing the reclaim of interrupted jobs' build directories and the pre-producer disk check; `git diff` shows it.
The diff touches only crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs and CHANGELOG.md (plus tests inside those files), and the module docs in drain.rs and disk.rs describe the new behavior.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
