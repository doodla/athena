# Job 20261106-030952-0f7a: Retry: producer cargo build/test/clippy/check take a build slot

## Request (verbatim)

> try again on 20261106-002344-jn5t

## Reading (accepted by the user)

Retry of job 20261106-002344-jn5t (build step 4): an `athena`-linked `cargo` wrapper first on the producer PATH holds a build slot for build, test, clippy and check, finding the subcommand after global flags via the policy's shared list. Fixes the prior failure by registering the new test file in xtask/shard-timings.txt and adding the fmt/metadata no-slot test; audience is the maintainer and verifier.

Assumptions:
- Code survey: read crates/athena-agent/src/policy.rs (CARGO_GLOBAL_FLAGS, check_cargo), crates/athena-agent/src/env.rs (producer PATH), crates/athena-core/src/slots.rs (SlotQueue, Class::Producer, try_acquire), crates/athena-core/src/run/producer.rs and crates/athena-cli/src/main.rs (entry point dispatching on argv[0]). Slot wait is measured by a test with a real SlotQueue and a fake cargo. No record or data file is added under crates/, so check_record_file/AnyRecord are unaffected.
- Retry of job 20261106-002344-jn5t: its candidate design is kept (athena binary linked as `cargo`, dispatch on argv[0]); the findings are addressed as: (a) blocker, missing line in xtask/shard-timings.txt: add `athena-cli --test slot_exec <seconds>`; (b) major, criterion 4 evidence: the fmt/metadata no-slot test is kept and criteria cite runnable commands; (c) minor, files outside the list: crates/athena-cli/src/lib.rs, crates/athena-core/src/run/mod.rs and run/tests.rs are now listed as deliverables; (d) nit, no `build-slot` subcommand: the argv[0] deviation is accepted and the design doc says so.
- The earlier introspection's proposal for a general early check that every test file is registered in shard-timings.txt is a separate job, not part of this retry; here the file is simply registered by hand. The unlisted athena-agent/src/lib.rs and athena-cli/Cargo.toml were not changed by the candidate and change only if needed.
- Existing xtask test xtask/tests/ci_shards.rs checked_in_timings_cover_every_workspace_test_unit is the guard for the registration.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: the slot_exec tests pass, including one with one slot and two producer processes where the second blocks while the first holds the slot, then runs, and afterwards try_acquire succeeds.
2. [behavior] Run `cargo test --locked`: a test runs `cargo -q build` and `cargo --locked test` through the wrapper against a fake cargo and asserts both held a slot while running and freed it after; it fails if the subcommand lookup reads only args[0].
3. [behavior] Run `cargo test --locked`: a test where no slot frees within the wait limit asserts the documented message ('no build slot free for ... s; try again'), a non-failure environment exit, and that the job is recorded as held (`environment: build slot wait`), not failed; the real cargo is never started.
4. [behavior] Run `cargo test --locked`: a test shows `cargo fmt` and `cargo metadata` run through the wrapper without taking a slot even when every slot is held.
5. [behavior] Run `cargo test -p xtask --test ci_shards`: checked_in_timings_cover_every_workspace_test_unit passes. Grep shows xtask/shard-timings.txt has an `athena-cli --test slot_exec` line.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked` from the repo root: all pass. Evidence cites these commands.
7. [content] CARGO_GLOBAL_FLAGS in crates/athena-agent/src/policy.rs is the single shared list, Grep shows the wrapper uses it, and no second copy exists; check_cargo behavior is unchanged and its existing tests pass.
8. [content] crates/athena-agent/src/env.rs puts the wrapper directory first on the producer PATH only; the diff leaves verifier commands and environments unchanged.
9. [content] CHANGELOG.md has an entry for producer cargo commands taking a build slot. The diff touches no governance files (athena.toml, defaults.toml, workflows) and no records/.

## Expected deliverables

- crates/athena-cli/src/slot_exec.rs (create)
- crates/athena-cli/tests/slot_exec.rs (create)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-agent/src/env.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- crates/athena-core/src/slots.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/Cargo.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)

## Out of scope

- A general early check for unregistered test files and better recorded test-check output (separate job)
- Changing the verifier's commands, SnapExec or the local check path
- Slot-owned warm target directories (build step 5)
- Recording queued_s per producer command or the peak-slots statistic
- Changing the policy's allowed commands beyond sharing the global-flag list
- The landing rebuild slot

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check slots taken and freed, not that wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The shard-timings registration is fixed and the shared flag list avoids drift.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The early-check infrastructure is left out of this retry.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261106-002344-jn5t raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 4 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`
> blocker (xtask/shard-timings.txt): cargo test fails: the new test file is not registered in the checked-in shard timings CI run 42153510840 failed on `ci_shards checked_in_timings_cover_every_workspace_test_unit`. I reran `cargo test -p xtask --test ci_shards checked_in_timings_cover_every_workspace_test_unit` here and it fails: the workspace units include 'athena-cli --test slot_exec' but shard-timings.txt does not. Fix: add an 'athena-cli --test slot_exec <seconds>' line to xtask/shard-timings.txt. That file is not on the deliverables list, but the repository's own test requires it whenever a test file is added.

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

- path: <state>/jobs/20261106-030952-0f7a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-030952-0f7a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked`: the slot_exec tests pass, including one with one slot and two producer processes where the second blocks while the first holds the slot, then runs, and afterwards try_acquire succeeds.
Run `cargo test --locked`: a test runs `cargo -q build` and `cargo --locked test` through the wrapper against a fake cargo and asserts both held a slot while running and freed it after; it fails if the subcommand lookup reads only args[0].
Run `cargo test --locked`: a test where no slot frees within the wait limit asserts the documented message ('no build slot free for ... s; try again'), a non-failure environment exit, and that the job is recorded as held (`environment: build slot wait`), not failed; the real cargo is never started.
Run `cargo test --locked`: a test shows `cargo fmt` and `cargo metadata` run through the wrapper without taking a slot even when every slot is held.
Run `cargo test -p xtask --test ci_shards`: checked_in_timings_cover_every_workspace_test_unit passes. Grep shows xtask/shard-timings.txt has an `athena-cli --test slot_exec` line.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked` from the repo root: all pass. Evidence cites these commands.
CARGO_GLOBAL_FLAGS in crates/athena-agent/src/policy.rs is the single shared list, Grep shows the wrapper uses it, and no second copy exists; check_cargo behavior is unchanged and its existing tests pass.
crates/athena-agent/src/env.rs puts the wrapper directory first on the producer PATH only; the diff leaves verifier commands and environments unchanged.
CHANGELOG.md has an entry for producer cargo commands taking a build slot. The diff touches no governance files (athena.toml, defaults.toml, workflows) and no records/.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
