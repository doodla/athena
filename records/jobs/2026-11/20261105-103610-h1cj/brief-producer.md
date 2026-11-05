# Job 20261105-103610-h1cj: Read ahead queued items while a single job runs, up to the limit

## Request (verbatim)

> Read and plan the next queued requests while the current job runs. Reading and planning are model calls with no build, but today a batch reads and plans an item only when it reaches it. While jobs run, read and plan the next queued items ahead, up to a limit, so a job is ready to dispatch when a slot frees.
> 
> Files changed: crates/athena-cli/src/drain.rs and its tests.
> 
> Acceptance:
> - a test where a batch runs one job and has three queued items: the readings of the next items are made while the first job runs
> - a test where the look-ahead limit is 1: only one item is read ahead
> - an item whose look-ahead reading is refused is reported as today, and the batch still runs its current job

## Reading (accepted by the user)

Read-ahead (reading and, for hard jobs, planning the next queued items while a job runs) already exists in crates/athena-cli/src/drain.rs, but only for two or more concurrent jobs. It is off for `drain` and for a bound of 1, so a batch running one job reads each next item only when it reaches it. I will make read-ahead work with one job at a time, limited by `Steps::read_ahead`, so the next job is ready when the slot frees. The audience is the person running a batch and the verifier who runs the tests.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs. The entry points are `drain` (calls run_serial) and `drain_concurrent` (calls run_serial when workers == 1, else run_parallel). `drain_with` plans, takes batches_for(...).remove(0), truncates it to one item when one_at_a_time, marks it tried, then calls run(d, vec![now], rest). `run_parallel` returns early to run_batches when `window == 0 || workers < 2`; otherwise it spawns `read_ahead_pump`. `pick_ahead` chooses items in later batches. `read_ahead_pump` calls `Steps::read`, then `plan_ahead`, and stores an `Ahead`. A refused or erroring reading is dropped with a note and is read again by `claim`/`attempts` when the item starts. Reading holds `caps.read` against the budget.
- The change is in drain.rs and its tests only. The module doc, which says nothing is read ahead with one job at a time, must be updated. The existing test `read_ahead_is_off_with_a_bound_of_one` (read_ahead_tests module) asserts the old behavior for bound 1 and must change. Its window-0 case stays valid.
- Design: when only one job runs at a time and `Steps::read_ahead()` > 0, the pump runs beside the single worker. When drain_with truncates a batch to one item, the dropped remainder is passed on in `later` so it can be read ahead. The pump keeps its existing guards: window, job limit, budget fit for a read plus a run, disjoint touched paths, and stop. `Steps::read_ahead()` is the look-ahead limit, so no new config is needed. The default stays 0 in the trait, and the real steps do not turn it on, so athena.toml is not touched.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The change adds no refusal rule and does not tighten any, so no newly-refused-readings percentage applies. A reading refused during look-ahead is not a new refusal: it is reported as today, when the item starts.
- Tests use the existing `World` fake in read_ahead_tests, which logs `read x`, `plan_ahead x`, `land x`, and gates calls. The property is measured through the log order of `Steps` calls and the report, not through stand-ins like flags.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/src/drain.rs: one job at a time (drain, or drain_concurrent with bound 1), a queue of one running job plus three more queued items, window 3. While the first job is gated in `run`, the log shows `read` for the next items before `land` of the first job. Verified with `cargo test -p athena-cli read_ahead`.
2. [behavior] A new test with look-ahead limit 1 and the same queue: while the first job runs, exactly one later item is read (`world.count("read c") == 0` and `read b` is logged, or the equivalent for the fake's item names), and the next is read only after the held reading is claimed. Verified with `cargo test -p athena-cli read_ahead`.
3. [behavior] A new test where the look-ahead reading of a queued item is refused (not runnable or an error) while the first job runs: the batch still lands the first job, the refused item is reported as it is today (same line, or same stop or reading-refused outcome as when read at its turn), and the pump's drop note appears in `report.notes`. Verified with `cargo test -p athena-cli read_ahead`.
4. [behavior] The existing read-ahead tests still pass, with `read_ahead_is_off_with_a_bound_of_one` rewritten so that window 0 still means no reading ahead for drain and for drain_concurrent(1). Every item is read after the job before it landed, and `plan_ahead` is never called. Verified with `cargo test -p athena-cli` and `cargo clippy --all-targets -- -D warnings`.
5. [content] The module doc in crates/athena-cli/src/drain.rs (the Read-ahead bullet) no longer says nothing is read ahead with one job at a time. It says read-ahead applies there too and is bounded by `Steps::read_ahead`. `cargo fmt --check` passes.
6. [content] Only crates/athena-cli/src/drain.rs changes, plus an optional CHANGELOG.md line if the repository's convention requires one. No governance file (athena.toml, defaults.toml, workflows) is changed.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Turning read-ahead on in the real steps or setting a non-zero default for `read_ahead` in athena.toml
- Changing how readings are refused, or the refusal rules
- Changing landing, rating, or rebase behavior
- Changes outside crates/athena-cli/src/drain.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in drain.rs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Three new tests; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test commands the verifier runs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the order of read, run, and land calls in the logs, not the existence of wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to one file.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The module doc and test comments are updated for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The look-ahead limit is enforced by the existing `pick_ahead` window check and covered by a test.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the next job's reading exists before the slot frees, not that code was added.
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
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
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

- path: <state>/jobs/20261105-103610-h1cj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-103610-h1cj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-cli/src/drain.rs: one job at a time (drain, or drain_concurrent with bound 1), a queue of one running job plus three more queued items, window 3. While the first job is gated in `run`, the log shows `read` for the next items before `land` of the first job. Verified with `cargo test -p athena-cli read_ahead`.
A new test with look-ahead limit 1 and the same queue: while the first job runs, exactly one later item is read (`world.count("read c") == 0` and `read b` is logged, or the equivalent for the fake's item names), and the next is read only after the held reading is claimed. Verified with `cargo test -p athena-cli read_ahead`.
A new test where the look-ahead reading of a queued item is refused (not runnable or an error) while the first job runs: the batch still lands the first job, the refused item is reported as it is today (same line, or same stop or reading-refused outcome as when read at its turn), and the pump's drop note appears in `report.notes`. Verified with `cargo test -p athena-cli read_ahead`.
The existing read-ahead tests still pass, with `read_ahead_is_off_with_a_bound_of_one` rewritten so that window 0 still means no reading ahead for drain and for drain_concurrent(1). Every item is read after the job before it landed, and `plan_ahead` is never called. Verified with `cargo test -p athena-cli` and `cargo clippy --all-targets -- -D warnings`.
The module doc in crates/athena-cli/src/drain.rs (the Read-ahead bullet) no longer says nothing is read ahead with one job at a time. It says read-ahead applies there too and is bounded by `Steps::read_ahead`. `cargo fmt --check` passes.
Only crates/athena-cli/src/drain.rs changes, plus an optional CHANGELOG.md line if the repository's convention requires one. No governance file (athena.toml, defaults.toml, workflows) is changed.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
