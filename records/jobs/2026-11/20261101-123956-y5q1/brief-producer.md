# Job 20261101-123956-y5q1: Reclaim interrupted jobs' build dirs; hold on low disk before producer

## Request (verbatim)

> try again on 20261101-102128-b1st
> The earlier attempt passed review and was rejected only because the verdict did not count GitHub's reused CI result as a test run; that is now fixed. Redo the same change.
> Priority: urgent

## Reading (accepted by the user)

Redo job 20261101-102128-b1st (rejected only because its gate commands were not seen running): at drain start and before each run, reclaim target/ of interrupted Dispatched jobs (no registry entry, free lock), keeping work trees and candidates, and hold the job as an environment stop on low disk before the producer. Audience: whoever runs `athena drain` unattended.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs (the disk check before the checks calls hold_for_disk at ~line 1303), crates/athena-core/src/run/close.rs (hold_for_disk), crates/athena-core/src/run/verification.rs (second hold_for_disk caller), crates/athena-core/src/state.rs (StateDir::with_free_space hook), crates/athena-core/src/disk.rs (cleanup and tests using with_free_space), crates/athena-cli/tests/drain.rs and tests/common/mod.rs (drain entry-point tests with simulated_free). No record or data file is added, so check_record_file/AnyRecord are unaffected. No refusal rule is tightened; the disk hold reuses the existing environment-stop mechanism. Free space is measured through the same hook the existing tests use.
- Prior job 20261101-102128-b1st: its candidate passed review. The one major finding (gate commands not cited) is addressed by running fmt, clippy and test as the final step and citing them in the criteria. The minor finding on close.rs is addressed by listing close.rs as a deliverable (its change is only a hold_for_disk doc comment naming the new caller), so the candidate's work is kept. The lock-window and records-reload findings are accepted by design and documented in CHANGELOG and drain.rs docs.
- The retry starts from the prior candidate's approach: the sweep removes only target/ for jobs that are Dispatched with no registry entry, are not on the keep list, and whose job lock is free; a pre-producer hold leaves no candidate and no resume mark.
- Per the earlier introspection, adding an Athena pre-verdict gate is a separate job and is not part of this one.

## Acceptance criteria

1. [behavior] A new integration test in crates/athena-cli/tests/drain.rs, run with `cargo test -p athena-cli --test drain`, enters through the drain entry point with an interrupted Dispatched job's target/ present and asserts target/ is gone after drain start while the job's work tree and candidate remain.
2. [behavior] A new test, run with `cargo test`, makes a job's lock held or its process alive (registry entry) and asserts its target/ is kept, and asserts a sweep before a later run reclaims a newly interrupted job's target/.
3. [behavior] A new test, run with `cargo test`, simulates low free disk only at the pre-producer point (via with_free_space) and asserts the job is held as an environment stop for low disk, the status says so, and the producer is never started.
4. [behavior] `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` all pass when run from the repository root.
5. [content] Grep of the diff shows a disk gate before the producer is invoked in crates/athena-core/src/run/producer.rs or run/mod.rs that holds through hold_for_disk, and no test fixture is pulled into a src unit test by #[path] from tests/common.
6. [content] CHANGELOG.md has a new entry on reclaiming interrupted jobs' build directories and the pre-producer disk check; module docs in drain.rs and disk.rs describe the new behavior.
7. [content] The diff does not touch athena.toml or defaults.toml, does not change the min_free_gb default, and does not delete work trees or candidates.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)

## Out of scope

- Deleting work trees or candidates of interrupted jobs
- Changing the min_free_gb default or athena.toml / defaults.toml
- Changing how closed and idle jobs are reclaimed
- Automatic retry policy changes for held jobs
- Including tests/common/mod.rs into src unit tests
- Adding an Athena-run pre-verdict gate (separate job)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the reclaim and the hold actually happening via tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy -D warnings and cargo test criterion.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands to run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Reclaim covers all interrupted jobs and adds a pre-producer disk gate, not the one instance.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Module docs and CHANGELOG entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261101-102128-b1st raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 4 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 38 cases in 38 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a pre-verdict step in code. Before the verifier runs, Athena runs the repo gate (fmt, clippy, test) itself and records the exact commands and exit codes. It also checks that every behavior criterion has a demonstration that names a command in that record. If the gate fails or a criterion has no ...
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
A new test, run with `cargo test`, simulates low free disk only at the pre-producer point (via with_free_space) and asserts the job is held as an environment stop for low disk, the status says so, and the producer is never started.
`cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` all pass when run from the repository root.
Grep of the diff shows a disk gate before the producer is invoked in crates/athena-core/src/run/producer.rs or run/mod.rs that holds through hold_for_disk, and no test fixture is pulled into a src unit test by #[path] from tests/common.
CHANGELOG.md has a new entry on reclaiming interrupted jobs' build directories and the pre-producer disk check; module docs in drain.rs and disk.rs describe the new behavior.
The diff does not touch athena.toml or defaults.toml, does not change the min_free_gb default, and does not delete work trees or candidates.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
