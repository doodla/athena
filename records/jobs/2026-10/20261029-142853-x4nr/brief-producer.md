# Job 20261029-142853-x4nr: Add in-flight table written at stage changes, read at drain start

## Request (verbatim)

> Priority: urgent
> Async drain build job A3 (docs/design/async-drain.md, decision D145). Add the in-flight table (job 3): a local file of job id, stage and worktree, written at each stage change and read at drain start. Depends on 20261029-142849-04d9. With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/state.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A3 (D145, design section 7 crash bullet): add a local in-flight table file (job id, stage, worktree path) in athena-core state.rs, written by the drain at each stage change and read at drain start. Follows job 20261029-142849-04d9 (A2, [drain.pipeline] config), whose result is kept and unchanged; with max_in_flight = 1 drain behaves as today.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (entry points drain and drain_concurrent -> drain_with, which reads the queue then runs run_serial/run_parallel; Steps trait, Drain struct) and crates/athena-core/src/state.rs (StateDir with root, job_dir, kept_work_dir; ResumeStage/ResumeMark). The table is a new type in state.rs beside StateDir, stored as a file under the state root; no record or data file is added under crates/, so the records allow-list, check_record_file and AnyRecord are not touched. The behavior is measured directly by calling drain/drain_concurrent with test Steps and reading the file back.
- Job 20261029-142849-04d9 (A2): its landed changes to config.rs, CHANGELOG.md and docs/design/async-drain.md are kept and not edited here. Its findings: the stale line 21 in the design doc (minor), the unhelpful DrainPipeline error text (minor) and the MAX_CHECK_SLOTS_CEILING nit were all in A2's own files; they are left to a separate job and do not affect A3, which does not read the [drain.pipeline] values.
- Stages recorded follow the design: at least producer and landing (plus checks); an entry is removed when a job closes. Atomic write via temp file and rename; a missing file means an empty table, a corrupt file is reported as a note, not a crash.
- Read at drain start means loaded and surfaced (a note or line listing leftover entries, and entries kept in memory); acting on them to resume jobs is a later build job. Drain does not change decisions from the table, so existing behavior is preserved.
- Not a refusal rule: nothing newly refuses readings, so no refusal-rate criterion is needed.

## Acceptance criteria

1. [behavior] A test calls drain (the real entry point) with test Steps and max_in_flight = 1 and shows the in-flight file holds the job id, stage and worktree for the job at each stage change, and no entry for it after the job closes. Run: cargo test -p athena-cli drain.
2. [behavior] A test pre-writes an in-flight file with a leftover entry, starts drain, and shows the entry is read at start and reported; a missing file and a corrupt file are both handled without failing the drain. Run: cargo test -p athena-cli and cargo test -p athena-core state.
3. [behavior] state.rs unit tests show the table round-trips (write then read gives the same job id, stage, worktree), updates a job's stage in place, and removes an entry, with an atomic write. Run: cargo test -p athena-core.
4. [behavior] All existing drain tests pass unchanged: git diff shows no existing test in drain.rs modified or deleted, and cargo test passes.
5. [behavior] cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace all pass.
6. [content] CHANGELOG.md has a new entry describing the in-flight table (file with job id, stage, worktree; written at stage changes; read at drain start; behavior unchanged at max_in_flight = 1). Check with git diff CHANGELOG.md.
7. [content] The diff touches only crates/athena-cli/src/drain.rs, crates/athena-core/src/state.rs and CHANGELOG.md (plus athena.toml, defaults.toml and workflows unchanged), and the new public items have doc comments written for an outside reader.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/state.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)

## Out of scope

- Resuming or acting on leftover table entries (producer retry, landing re-check): later build jobs
- Running more than one producer at once, or raising max_in_flight to 3 (build job 10)
- Reading [drain.pipeline] values in the drain runtime
- Fixing A2's findings in config.rs and the design doc (job 20261029-142849-04d9 work is kept as landed)
- Changes to athena.toml, defaults.toml, .github/workflows
- Disk-guard, check-slot, read-ahead, conflict-return behavior

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust in the two named crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the file contents through drain, not that types exist.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the three named files.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog written for outside readers.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: File holds only job ids, stages and paths; no secrets.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

20 more qualifying classes are left out.

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

A test calls drain (the real entry point) with test Steps and max_in_flight = 1 and shows the in-flight file holds the job id, stage and worktree for the job at each stage change, and no entry for it after the job closes. Run: cargo test -p athena-cli drain.
A test pre-writes an in-flight file with a leftover entry, starts drain, and shows the entry is read at start and reported; a missing file and a corrupt file are both handled without failing the drain. Run: cargo test -p athena-cli and cargo test -p athena-core state.
state.rs unit tests show the table round-trips (write then read gives the same job id, stage, worktree), updates a job's stage in place, and removes an entry, with an atomic write. Run: cargo test -p athena-core.
All existing drain tests pass unchanged: git diff shows no existing test in drain.rs modified or deleted, and cargo test passes.
cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace all pass.
CHANGELOG.md has a new entry describing the in-flight table (file with job id, stage, worktree; written at stage changes; read at drain start; behavior unchanged at max_in_flight = 1). Check with git diff CHANGELOG.md.
The diff touches only crates/athena-cli/src/drain.rs, crates/athena-core/src/state.rs and CHANGELOG.md (plus athena.toml, defaults.toml and workflows unchanged), and the new public items have doc comments written for an outside reader.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
