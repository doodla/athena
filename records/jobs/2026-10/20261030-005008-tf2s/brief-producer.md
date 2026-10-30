# Job 20261030-005008-tf2s: Retry A3: in-flight table recording live work tree and checks stage

## Request (verbatim)

> try again on 20261029-142853-x4nr

## Reading (accepted by the user)

Retry job 20261029-142853-x4nr (build job A3, D145): add the local in-flight table (job id, stage, worktree) in state.rs, written by the drain at each stage change and read at drain start, fixing the rejected candidate's major finding that RealSteps::worktree returned the kept-work directory instead of the live work tree. Audience: the drain's later crash-recovery jobs and the verifier. max_in_flight = 1 behaves as today.

Assumptions:
- Code survey: read crates/athena-core/src/state.rs (StateDir::job_dir at line 52, kept_work_dir at line 58 = <root>/kept/<id>/work) and crates/athena-core/src/run/producer.rs:87 (live work tree = job_dir.join("work")); drain entry point is drain in crates/athena-cli/src/drain.rs with the Steps trait and RealSteps; existing CLI tests in crates/athena-cli/tests/drain.rs. The in-flight file is a plain local state file under the state root, not a record, so check_record_file and AnyRecord do not apply; the round-trip is covered by state.rs unit tests. The property (recorded path equals the producer's live directory) is measured directly by comparing the table row with the path the real producer code uses, not through a stub.
- Job 20261029-142853-x4nr's candidate is dropped and rebuilt from the base; its design is reused: InFlightStage/InFlightEntry/InFlightTable (read, set, remove) in state.rs, in-flight.json in the state root written via temp file and rename, defaulted Steps methods in_flight_table and worktree, a row removed when the run fails or landing finishes, leftover rows reported at start, missing file = empty, corrupt file = note.
- Findings of 20261029-142853-x4nr addressed: (major) RealSteps::worktree now returns the live work tree via a StateDir helper (job_dir(id).join("work")) that producer.rs also uses; (minor) the checks stage is written so the table distinguishes producer, checks and landing; (minor) a corrupt table is not silently replaced: set/remove keep the bad file by renaming it aside before writing, and I/O errors are not treated as an empty table; (minor) the design doc gets a note in section 11 that build job 3 is implemented.
- Job 20261029-142849-04d9 (A2, [drain.pipeline] config) stays as landed and is not touched.
- Prior art tried for this class (a test of recorded state through the real implementation) is reused here, and made specific: the test must compare the recorded path with the directory the real producer code uses.

## Acceptance criteria

1. [behavior] A test using the real steps implementation (RealSteps), not a stub, shows the table row's worktree equals the producer's live work tree (the StateDir job_dir/work path that run/producer.rs uses) while the producer stage runs, and is not the kept/<id>/work path. Run: cargo test -p athena-cli drain.
2. [behavior] A test shows a timed-out job whose work is kept records the live work tree while it runs, and the kept-work directory is not used in the table. Run: cargo test -p athena-cli drain.
3. [behavior] A test calls drain with test Steps and max_in_flight = 1 and shows the in-flight file holds job id, stage and worktree at the producer, checks and landing stages, and no entry after the job closes. Run: cargo test -p athena-cli drain.
4. [behavior] Tests pre-write an in-flight file with a leftover entry and show it is read and reported at drain start; a missing file and a corrupt file do not fail the drain, and a corrupt file is kept (renamed aside) rather than silently overwritten. Run: cargo test -p athena-cli and cargo test -p athena-core state.
5. [behavior] state.rs unit tests show the table round-trips, updates a stage in place, removes an entry, and writes atomically. Run: cargo test -p athena-core.
6. [behavior] No existing test in drain.rs is modified or deleted (git diff), and cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace pass.
7. [content] CHANGELOG.md has a new entry describing the in-flight table (job id, stage, worktree; written at stage changes; read at drain start; unchanged at max_in_flight = 1). Check with git diff CHANGELOG.md.
8. [content] docs/design/async-drain.md section 11 notes that build job 3 is implemented. The diff touches only drain.rs, state.rs, CHANGELOG.md, the design doc, and tests in crates/athena-cli/tests/drain.rs if needed; athena.toml, defaults.toml and workflows are unchanged. New public items have doc comments for an outside reader.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/state.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Resuming or acting on leftover table entries (producer retry, landing re-check): later build jobs
- Running more than one producer at once, or raising max_in_flight to 3 (build job 10)
- Reading [drain.pipeline] values in the drain runtime
- Fixing A2's findings in config.rs (job 20261029-142849-04d9 work is kept as landed)
- Changes to athena.toml, defaults.toml, .github/workflows
- Disk-guard, check-slot, read-ahead, conflict-return behavior

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the recorded path through the real steps, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the work tree path at its source with a shared StateDir helper.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the named cargo commands.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the drain and state files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261029-142853-x4nr raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-cli/src/drain.rs:1856-1858 (RealSteps::worktree)): In the real drain the table records the wrong work tree: the kept-work directory for timed-out jobs, not the producer's live work tree. `RealSteps::worktree` returns `self.athena.state.kept_work_dir(&id)`, which is `<root>/kept/<id>/work` (state.rs:58-60). Its doc comment says it is where a timed-out producer's work tree is kept, and `keep_timed_out_work` (run/mod.rs:602-617) moves the work there only on a timeout. Such a job's row is untracked by then, because its run outcome is not Landed. The live work tree is `job_dir.join("work")` (run/producer.rs:87), which is `<root>/jobs/<id>/work`. So every row in production names a directory that normally does not exist, and the crash-recovery job that reads this table would use a wrong path. This is a guess where the code could have been read; the fix is `self.athena.state.job_dir(&id).join("work")`, ideally through a `StateDir` helper that producer.rs also uses. No test covers `RealSteps::worktree`, because the drain tests use a stub `/work/<job>`.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 34 cases in 34 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test for any recorded-state feature that runs through the real implementation (RealSteps) instead of a fake. It would start a producer stage, read the in-flight table entry, and assert that the recorded path equals the directory the producer is actually working in. It would cover both the norm...
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

A test using the real steps implementation (RealSteps), not a stub, shows the table row's worktree equals the producer's live work tree (the StateDir job_dir/work path that run/producer.rs uses) while the producer stage runs, and is not the kept/<id>/work path. Run: cargo test -p athena-cli drain.
A test shows a timed-out job whose work is kept records the live work tree while it runs, and the kept-work directory is not used in the table. Run: cargo test -p athena-cli drain.
A test calls drain with test Steps and max_in_flight = 1 and shows the in-flight file holds job id, stage and worktree at the producer, checks and landing stages, and no entry after the job closes. Run: cargo test -p athena-cli drain.
Tests pre-write an in-flight file with a leftover entry and show it is read and reported at drain start; a missing file and a corrupt file do not fail the drain, and a corrupt file is kept (renamed aside) rather than silently overwritten. Run: cargo test -p athena-cli and cargo test -p athena-core state.
state.rs unit tests show the table round-trips, updates a stage in place, removes an entry, and writes atomically. Run: cargo test -p athena-core.
No existing test in drain.rs is modified or deleted (git diff), and cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and cargo test --workspace pass.
CHANGELOG.md has a new entry describing the in-flight table (job id, stage, worktree; written at stage changes; read at drain start; unchanged at max_in_flight = 1). Check with git diff CHANGELOG.md.
docs/design/async-drain.md section 11 notes that build job 3 is implemented. The diff touches only drain.rs, state.rs, CHANGELOG.md, the design doc, and tests in crates/athena-cli/tests/drain.rs if needed; athena.toml, defaults.toml and workflows are unchanged. New public items have doc comments for an outside reader.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
