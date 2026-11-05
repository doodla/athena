# Job 20261105-180325-s9pg: Enforce max_check_slots as a counting build-slot queue for local checks

## Request (verbatim)

> Enforce max_check_slots as a counting build-slot queue around local checks (step 3). Build step from docs/design/agent-and-build-slots.md section 8; follow that section for the files touched and how the effect is measured.
> 
> Acceptance:
> - local checks take a build slot from a counting queue of max_check_slots entries and free it when done, including when the job crashes
> - a test where two jobs need local checks with one slot: the second waits and then runs
> - a test where a slot holder's process dies: the slot frees without manual cleanup

## Reading (accepted by the user)

Add a lock-file counting semaphore (new slots.rs) of max_check_slots entries that SnapExec::exec takes around each local check and releases on drop or process death. Add build_wait_s/build_age_promote_s config, the resumable "environment: build slot wait" hold, and a queued_s field plus a "build wait" status line, as design section 8 step 3 says. Audience: the maintainer running the drain.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs (SnapExec::exec runs the disk gate once, then run_check per check; the entry point), crates/athena-core/src/state.rs (lock_file, try_lock_job use std File::try_lock; the lock dies with the process), crates/athena-core/src/disk.rs (LowDisk hold, write_hold/read_hold) and crates/athena-data/src/config.rs (PipelineConfig.max_check_slots, validated 1..=MAX_CHECK_SLOTS_CEILING). The slot is taken after refuse_if_stopped and the disk gate, before run_check, and held until the snapshot is restored.
- Code survey: the effect is measured directly by a queued_s field on each check in verification.json (CheckRun in crates/athena-data/src/records.rs, optional so old records still read; its round-trip/AnyRecord tests must still pass) and a 'build wait' line in the throughput section of crates/athena-core/src/status.rs. No new record file type is added.
- Only the local-check part of step 3 is in scope; the producer wrapper (step 4), landing rebuild (step 7) and slot-owned target directories (step 5) are later steps. Priority classes and ageing are implemented in slots.rs as the design says, with check/rerun as the only class used now.
- A limit of 1 reproduces today's one-build behaviour. Test hooks may use the existing check_seam and a tempdir state; a 'process dies' test uses a child process (or a separate File handle dropped) holding a slot, since OS file locks release on exit.
- athena.toml and defaults.toml need no change: new config fields have in-code defaults (build_wait_s 1800, build_age_promote_s 600).

## Acceptance criteria

1. [behavior] `cargo test -p athena-core` passes including a new slots test where two jobs need local checks with max_check_slots = 1: the second waits while the first holds the slot, then runs after release (asserted by order and a nonzero queued_s for the second).
2. [behavior] `cargo test -p athena-core` includes a test where a slot holder's process dies (a child process holding the slot lock is killed, or a holder is dropped on panic/error) and a waiter then acquires the slot with no manual cleanup or sweeper.
3. [behavior] A test with max_check_slots = 2 shows at most 2 holders at once and a third request waits.
4. [behavior] A test shows a check request waiting past build_wait_s returns a resumable hold with reason `environment: build slot wait`, uses no attempt, and is not recorded as a work failure.
5. [content] crates/athena-core/src/slots.rs exists and SnapExec::exec in crates/athena-core/src/run/checks.rs acquires a guard from it before run_check and drops it after the snapshot restore, on every return path (guard released by Drop).
6. [content] PipelineConfig in crates/athena-data/src/config.rs gains build_wait_s (default 1800) and build_age_promote_s (default 600) with validation and tests; unknown keys are still refused.
7. [behavior] Each check in verification.json carries an optional queued_s (old records still deserialize), and `cargo test` covers a 'build wait' line in the status throughput section that sums it.
8. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean, and the whole `cargo test` passes with the default max_check_slots = 1.
9. [content] No change to athena.toml, defaults.toml, workflows, or records/; CHANGELOG.md gets an entry for the change.

## Expected deliverables

- crates/athena-core/src/slots.rs (create)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/state.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/agent-and-build-slots.md (modify)

## Out of scope

- Producer cargo wrapper and build-slot subcommand (step 4)
- Landing rebuild slot use (step 7)
- Slot-owned warm target directories (step 5)
- Changes to athena.toml or defaults.toml
- Changing the sample metrics in records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust, using std file locks as state.rs does.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New slots.rs tests; cargo test passes and clippy is clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise waiting, release on crash, and the queued_s output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is measured by queued_s summed in the 'build wait' status line.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only step 3; steps 4 to 7 out of scope.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The limit is enforced by lock files, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-180325-s9pg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-180325-s9pg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

`cargo test -p athena-core` passes including a new slots test where two jobs need local checks with max_check_slots = 1: the second waits while the first holds the slot, then runs after release (asserted by order and a nonzero queued_s for the second).
`cargo test -p athena-core` includes a test where a slot holder's process dies (a child process holding the slot lock is killed, or a holder is dropped on panic/error) and a waiter then acquires the slot with no manual cleanup or sweeper.
A test with max_check_slots = 2 shows at most 2 holders at once and a third request waits.
A test shows a check request waiting past build_wait_s returns a resumable hold with reason `environment: build slot wait`, uses no attempt, and is not recorded as a work failure.
crates/athena-core/src/slots.rs exists and SnapExec::exec in crates/athena-core/src/run/checks.rs acquires a guard from it before run_check and drops it after the snapshot restore, on every return path (guard released by Drop).
PipelineConfig in crates/athena-data/src/config.rs gains build_wait_s (default 1800) and build_age_promote_s (default 600) with validation and tests; unknown keys are still refused.
Each check in verification.json carries an optional queued_s (old records still deserialize), and `cargo test` covers a 'build wait' line in the status throughput section that sums it.
`cargo clippy --all-targets` and `cargo fmt --check` are clean, and the whole `cargo test` passes with the default max_check_slots = 1.
No change to athena.toml, defaults.toml, workflows, or records/; CHANGELOG.md gets an entry for the change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
