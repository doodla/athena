# Job 20261030-185530-b5hm: Hold and auto-resume jobs on low disk instead of failing them

## Request (verbatim)

> try again on 20261030-060812-etgx

## Reading (accepted by the user)

Retry of job 20261030-060812-etgx (dispatched, never verified, no candidate or findings). The free-disk guard before the checks and the verifier must hold the job resumable from its built candidate when space stays short after cleanup, rather than closing it failed with a manual `athena retry`. A later drain resumes it automatically. Audience: the maintainer running drains and the verifier running cargo test.

Assumptions:
- Code survey: read-level survey of crates/athena-core/src/disk.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/status.rs, crates/athena-core/src/scope_guard.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs and crates/athena-cli/tests/binary.rs, which mention low disk or ATHENA_FREE_BYTES. The guard runs after the build, before the checks and before the verifier. Free space is measured directly, with ATHENA_FREE_BYTES as the test override. No new record or data file is added under crates/, so check_record_file and AnyRecord are not touched; any new state field must round-trip in the existing state tests.
- Job 20261030-060812-etgx had no candidate, no changes and no findings, so there is no earlier work to keep or drop; this retry starts from the base with the same scope.
- The prior accepted reading is reused as the baseline. The new request takes precedence, and it only says to try again.
- Work of job 20261025-033738-yz2d (auto-propose) and job 20261025-202447-xgk9 stays out of this change, as in the out_of_scope list.
- This adds no refusal rule. It removes a failing outcome, so no refusal-rate figure applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A new test with simulated low space (ATHENA_FREE_BYTES) after the build shows the job ends held and not Failed: no `athena retry` is needed, no verifier or producer verdict is recorded, and the outcome text names `environment: low disk` with the free and required amounts.
2. [behavior] Run `cargo test -p athena-cli --test drain`. A new test shows a later drain pass, once the simulated free space is raised, resumes the held job from its kept candidate with no new producer run and no `athena retry`, and the job finishes.
3. [behavior] Run `cargo test -p athena-cli --test drain`. A new test shows space that stays short across passes leaves the job held and uses up no drain attempt. Each of these tests fails on the pre-change code.
4. [behavior] Run `cargo test -p athena-core`. A test shows a low-disk hold before the checks and before the verifier is not counted as a work failure in introspection or failure classes, and that the idle-build cleanup still runs first and lets the job proceed when it frees enough space.
5. [behavior] Run `cargo test -p athena-cli --test binary`. `athena status` shows the held job with its low-disk reason and the amounts.
6. [content] Grep of crates/athena-core/src/run/mod.rs and crates/athena-core/src/run/verification.rs shows the low-disk path no longer calls `self.failure(` with a `before the checks` or `before the verifier` retry message.
7. [content] CHANGELOG.md has an entry for the hold-and-auto-resume behavior, and the module docs in crates/athena-core/src/disk.rs and crates/athena-cli/src/drain.rs describe it instead of fail-and-retry.
8. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. All pass.
9. [content] Governance files are unchanged: athena.toml, defaults.toml, workflows, and the Cargo and toolchain files are not in the diff.

## Expected deliverables

- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Automatic proposing after introspection (job 20261025-033738-yz2d): main.rs, introspect.rs, proposals.rs, tests/introspection.rs, tests/proposals.rs and its edits to drain.rs and tests/drain.rs; that job is rerun separately
- The test-suite speedup and other work of job 20261025-202447-xgk9
- Guarding combined disk use of parallel jobs, or planning around disk
- Changing the [disk] min_free_gb default or config schema, or editing athena.toml, defaults.toml or workflows
- Deleting committed records or touching a running job's files in the cleanup
- Removing `athena retry` or its handling of old failed environment jobs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criterion requires cargo test, clippy and fmt to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run named tests with simulated low space.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; proposing work is excluded.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The guard's fail outcome is changed to a hold, not just the retry message.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is exercised by tests the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 11 cases in 9 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: When a job failed on a producer deadline, make retry and auto-requeue refuse an identical-scope attempt. Require either a narrowed reading (one timed run, a single cause, a cap on full test runs) or an explicit the person running Athena override. Hold the retry and show the previous deadline, the sp...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...
- 8 cases in 8 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Make Athena own disk reclamation. When a producer finishes, it deletes that job's producer build folder, since the checks rebuild from cold anyway. Before the checks it also removes regenerable caches and finished baseline build folders. If free space is still under the guard, the job is held (not f...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...

22 more qualifying classes are left out.

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

Run `cargo test -p athena-cli --test drain`. A new test with simulated low space (ATHENA_FREE_BYTES) after the build shows the job ends held and not Failed: no `athena retry` is needed, no verifier or producer verdict is recorded, and the outcome text names `environment: low disk` with the free and required amounts.
Run `cargo test -p athena-cli --test drain`. A new test shows a later drain pass, once the simulated free space is raised, resumes the held job from its kept candidate with no new producer run and no `athena retry`, and the job finishes.
Run `cargo test -p athena-cli --test drain`. A new test shows space that stays short across passes leaves the job held and uses up no drain attempt. Each of these tests fails on the pre-change code.
Run `cargo test -p athena-core`. A test shows a low-disk hold before the checks and before the verifier is not counted as a work failure in introspection or failure classes, and that the idle-build cleanup still runs first and lets the job proceed when it frees enough space.
Run `cargo test -p athena-cli --test binary`. `athena status` shows the held job with its low-disk reason and the amounts.
Grep of crates/athena-core/src/run/mod.rs and crates/athena-core/src/run/verification.rs shows the low-disk path no longer calls `self.failure(` with a `before the checks` or `before the verifier` retry message.
CHANGELOG.md has an entry for the hold-and-auto-resume behavior, and the module docs in crates/athena-core/src/disk.rs and crates/athena-cli/src/drain.rs describe it instead of fail-and-retry.
Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`. All pass.
Governance files are unchanged: athena.toml, defaults.toml, workflows, and the Cargo and toolchain files are not in the diff.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
