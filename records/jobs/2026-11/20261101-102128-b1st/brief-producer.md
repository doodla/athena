# Job 20261101-102128-b1st: Reclaim interrupted jobs' build dirs and gate disk before the producer

## Request (verbatim)

> try again on 20261030-185437-1tbs

## Reading (accepted by the user)

Retry of job 20261030-185437-1tbs (failed in planning: its test fixture include could not compile in a lib unit test). Reclaim target/ of interrupted (Dispatched, no registry entry, free lock) jobs at drain start and before each run, keeping work trees and candidates, and hold the job as an environment stop on low disk before the producer. Audience: whoever runs `athena drain` unattended.

Assumptions:
- Code Survey: Code survey: read crates/athena-core/src/disk.rs (sweep_idle_targets at :160 uses try_lock_job and remove_target and skips only `keep`; free-bytes hook via FREE_BYTES_ENV at :130; disk_gate at :446), crates/athena-core/src/run/mod.rs (disk_gate calls at :749 and :1301, before dispatch), crates/athena-core/src/run/close.rs (hold_for_disk :176), crates/athena-core/src/state.rs (remove_target :251, with_free_space :224), crates/athena-cli/src/drain.rs (drain :2273, drain_concurrent :2284, drain_with :2332) and the integration tests crates/athena-cli/tests/drain.rs with its Fixture. No record or data file is added, so no records allow-list change.
- Job 20261030-185437-1tbs: its candidate changed no files and nothing is kept. Its planning findings are addressed as follows. (major) The #[path] include of tests/common/mod.rs into a drain.rs unit test cannot compile (it names athena_cli::wiring::capabilities); fixed by putting the tests in the existing integration test file crates/athena-cli/tests/drain.rs, which already uses Fixture and can import athena_cli. (minor) Criterion 2 passed on old code because the dispatch gate already holds before the agents; settled by reading the code: the producer is entered in run/producer.rs, so the change adds a disk_gate before the producer there, holding through hold_for_disk, and the user is not asked. (nit) Do not rely on a bare Job stub in Records::load; the disk.rs test uses a real ask+go flow with a killed producer if a stub is not accepted.
- Existing code: reuse crates/athena-core/src/disk.rs sweep_idle_targets and the lock/remove logic, crates/athena-core/src/state.rs remove_target and try_lock_job, crates/athena-core/src/run/close.rs hold_for_disk and crates/athena-core/src/run/mod.rs disk_gate. Extend them; do not build a parallel mechanism.
- Interrupted means: records show Dispatched, there is no registry entry (process gone), and the job lock is free. The lock is held during removal and the drain's own jobs are skipped. Only target/ is removed.
- The scope widens beyond the earlier file list to crates/athena-core/src/run/producer.rs (and possibly run/mod.rs) because the pre-producer gate belongs there (root-cause default). The earlier list's athena-cli drain.rs, athena-core disk.rs and CHANGELOG.md stay in scope.

## Acceptance criteria

1. [behavior] A new integration test in crates/athena-cli/tests/drain.rs, run with `cargo test -p athena-cli --test drain`, enters through the drain entry point with an interrupted Dispatched job's target/ present and asserts target/ is gone after drain start while the job's work tree and candidate remain.
2. [behavior] A new test, run with `cargo test`, makes a job's lock held or its process alive (registry entry) and asserts its target/ is kept, and asserts a sweep before a later run reclaims a newly interrupted job's target/.
3. [behavior] A new test, run with `cargo test`, simulates low free disk only at the pre-producer point (via the free-bytes hook or with_free_space) and asserts the job is held as an environment stop (low disk), the status says so, and the producer is never started.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
5. [content] Grep of the diff shows a disk gate before the producer is invoked in crates/athena-core/src/run/producer.rs (or its caller) that holds through hold_for_disk, and no test fixture is pulled in by #[path] from tests/common into a src unit test.
6. [content] CHANGELOG.md has a new entry describing the reclaim of interrupted jobs' build directories and the pre-producer disk check; `git diff` shows it. Module docs in drain.rs and disk.rs describe the new behavior.
7. [content] The diff does not touch athena.toml or defaults.toml, does not change the min_free_gb default, and does not delete work trees or candidates.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)

## Out of scope

- Deleting work trees or candidates of interrupted jobs
- Changing the min_free_gb default or athena.toml / defaults.toml
- Changing how closed and idle jobs are reclaimed
- Automatic retry policy changes for held jobs
- Including tests/common/mod.rs into src unit tests

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy -D warnings.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Reclaims at the source and gates the producer itself, not only the checks.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise drain and the gate through tests.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by the gate and sweep code, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in a few files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the earlier job's request and failure from the brief.
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

A new integration test in crates/athena-cli/tests/drain.rs, run with `cargo test -p athena-cli --test drain`, enters through the drain entry point with an interrupted Dispatched job's target/ present and asserts target/ is gone after drain start while the job's work tree and candidate remain.
A new test, run with `cargo test`, makes a job's lock held or its process alive (registry entry) and asserts its target/ is kept, and asserts a sweep before a later run reclaims a newly interrupted job's target/.
A new test, run with `cargo test`, simulates low free disk only at the pre-producer point (via the free-bytes hook or with_free_space) and asserts the job is held as an environment stop (low disk), the status says so, and the producer is never started.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
Grep of the diff shows a disk gate before the producer is invoked in crates/athena-core/src/run/producer.rs (or its caller) that holds through hold_for_disk, and no test fixture is pulled in by #[path] from tests/common into a src unit test.
CHANGELOG.md has a new entry describing the reclaim of interrupted jobs' build directories and the pre-producer disk check; `git diff` shows it. Module docs in drain.rs and disk.rs describe the new behavior.
The diff does not touch athena.toml or defaults.toml, does not change the min_free_gb default, and does not delete work trees or candidates.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
