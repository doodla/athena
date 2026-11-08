# Job 20261026-144234-bz4e: Split process-lifecycle code out of drain.rs to get under limits

## Request (verbatim)

> Structural pass: crates/athena-cli/src/drain.rs
> 
> Code health found crates/athena-cli/src/drain.rs over a limit: 1798 production lines with 5 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week.
> 
> Measured: 1798 production lines (test modules excluded); longest function run_job at 131 lines; 5 landings in the last 7 days; 16 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: 1798 production lines with 5 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-cli/src/drain.rs

## Reading (accepted by the user)

Restructure crates/athena-cli/src/drain.rs, which is at 1798 production lines with 5 landings in a week against a limit of 1500. Move the pid-file, PidGuard, stop/restart and RealLifecycle section (around lines 3412 to the end) into a new child module, drain/lifecycle.rs. Re-export every public item from drain so callers in main.rs and display.rs compile unchanged. The audience is the code-health gate and the maintainers. There is no behavior change.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (item outline), crates/athena-cli/src/main.rs and crates/athena-cli/src/display.rs (callers through the drain:: path). Callers use drain::guarded, RealProcs, StopWait, PidRecord, DrainState, StopOutcome, RestartOutcome and reinstall_note. The drain.rs outline shows about 17 lines of imports plus doc comments at the top (up to line 260), then types, the Drain engine, drain_with, and from about line 3412 a self-contained process-lifecycle section: pid_file, log_file, PidRecord, Procs/RealProcs, DrainState, PidGuard, guarded, StopWait, stop_and_wait, restart, RealLifecycle. No records or data files are added, so the record validation allow-list is not involved.
- Code survey: the move target is crates/athena-cli/src/drain/lifecycle.rs, a child module of drain.rs. It is declared with `mod lifecycle; pub use lifecycle::*;` (or explicit re-exports), so existing paths such as crate::drain::DrainState keep working. The producer keeps the tests with the code they test, or leaves them in drain.rs if they need private items, and may widen visibility to pub(super) where required.
- The lifecycle section is chosen because it has the least coupling to the Drain engine. If the section turns out to be under 300 production lines, the producer also moves another cohesive group, for example the study_*/model_contradictions helpers or the PrLander code. The target is below 1500 production lines in drain.rs, with no function over 200 lines.
- No refusal rule is added or tightened, so no newly-refused-reading percentage applies.

## Acceptance criteria

1. [content] crates/athena-cli/src/drain.rs has fewer than 1500 production lines (test modules excluded), and a new crates/athena-cli/src/drain/lifecycle.rs holds the moved code. Check with `ls` and by reading both files.
2. [content] The diff only moves code, re-exports it and adjusts visibility or imports. No function bodies change in logic, and public paths such as crate::drain::guarded, DrainState, StopOutcome and RestartOutcome are still exported from drain. Check with `git diff`.
3. [behavior] `cargo test -p athena-cli` passes, with the same tests present as before the change and none deleted or ignored. This is the outcome check: the drain behavior is unchanged.
4. [behavior] `cargo clippy --all-targets -- -D warnings` is clean and `cargo fmt --check` passes.
5. [behavior] `cargo build` succeeds for the whole workspace, so main.rs and display.rs callers compile without edits.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/drain/lifecycle.rs (create)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any behavior change or refactor of the drain engine logic
- Changes to athena.toml code-health limits or other governance files
- Splitting other files such as display.rs or main.rs

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module split, with no behavior edits.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Existing tests must pass, and clippy and fmt must be clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the file is under the limit and behavior is unchanged, not that code was moved.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The file is split by responsibility rather than trimmed to just under the limit.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

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

- path: <state>/jobs/20261026-144234-bz4e/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144234-bz4e/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-cli/src/drain.rs has fewer than 1500 production lines (test modules excluded), and a new crates/athena-cli/src/drain/lifecycle.rs holds the moved code. Check with `ls` and by reading both files.
The diff only moves code, re-exports it and adjusts visibility or imports. No function bodies change in logic, and public paths such as crate::drain::guarded, DrainState, StopOutcome and RestartOutcome are still exported from drain. Check with `git diff`.
`cargo test -p athena-cli` passes, with the same tests present as before the change and none deleted or ignored. This is the outcome check: the drain behavior is unchanged.
`cargo clippy --all-targets -- -D warnings` is clean and `cargo fmt --check` passes.
`cargo build` succeeds for the whole workspace, so main.rs and display.rs callers compile without edits.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
