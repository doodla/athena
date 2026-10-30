# Job 20261029-142858-5c0p: Add per-job worktree lifecycle and in-flight disk bound (A4)

## Request (verbatim)

> Priority: urgent
> Async drain build job A4 (docs/design/async-drain.md, decision D145). Per-job worktree start (job 4): create and remove a worktree and its target directory per job, with the disk bound of section 5 computed from job A1's measured T and S. Depends on 20261029-142853-x4nr and 20261029-142844-ynv3. With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/disk.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A4 of D145: the run creates a git worktree and its own target directory per job and removes both when the job ends, and disk.rs gains the section 5 bound in_flight = floor((free - reserve - slots*T)/(T+S)). T and S are A1's figures; job 20261029-142844-ynv3 (A1) measured none, so the bound takes T and S as inputs with the design's labeled estimates (6 GB, 0.2 GB) as defaults. With max_in_flight = 1, behavior is unchanged.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs (df_free_bytes, sweep_idle_targets, StateDir/format_bytes use), docs/design/async-drain.md section 5 (bound formula, T/S estimates); run/mod.rs and run/producer.rs are the entry points where the work tree is made (job_dir/work). The real entry point is the run of a job through run/mod.rs; the disk bound is a pure function in disk.rs tested directly. No record or data file is added, so check_record_file/AnyRecord are not involved.
- Goal: let a later build job start producers early in their own worktrees; audience is Sri and the drain verifier, who check behavior by cargo test.
- A1 measured nothing, so T and S are parameters with the design's estimates as defaults, labeled estimates in code docs and the CHANGELOG; replacing them with measurements is later work.
- If the work tree already uses a git worktree, the change adds the per-job target directory and guaranteed removal of both on every end path; otherwise it adds both.
- Governance files athena.toml and defaults.toml need no change; none is a deliverable.

## Acceptance criteria

1. [behavior] A test through the run entry point in crates/athena-core/src/run/mod.rs shows a job gets its own worktree and its own target directory under the job dir while it runs, and both are removed after success and after failure; run cargo test -p athena-core.
2. [behavior] Unit tests in crates/athena-core/src/disk.rs show the bound floor((free_gb - reserve_gb - slots*T)/(T+S)) for numeric cases, including exactly one job, zero when free disk is below the reserve, and no underflow with unsigned or negative inputs; run cargo test -p athena-core disk.
3. [behavior] With max_in_flight = 1 the existing drain tests pass unchanged: git diff shows no edits to existing test bodies, and cargo test passes.
4. [behavior] cargo fmt --check and cargo clippy --all-targets -- -D warnings are clean and cargo test passes for the workspace.
5. [content] T and S in disk.rs are marked as estimates (6 GB and 0.2 GB per docs/design/async-drain.md section 5) because A1 (20261029-142844-ynv3) recorded no measurement; no measured number is claimed.
6. [content] CHANGELOG.md has an entry for per-job worktree start and the disk bound, naming build job A4 and stating that T and S are estimates.
7. [content] The worktree path used is the live work tree (job_dir/work, per run/producer.rs), not the kept-work directory, addressing the A3 finding (20261029-142853-x4nr).
8. [content] athena.toml, defaults.toml and .github/workflows are unchanged (git diff).

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/state.rs (modify)
- docs/design/async-drain.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Job 20261029-142844-ynv3 (A1): its candidate changes to CHANGELOG.md, docs/decisions.md and docs/design/async-drain.md are kept as landed or as-is; its missing measurements are not redone here, and no T or S is invented
- Job 20261029-142853-x4nr (A3): its candidate changes to drain.rs, state.rs and CHANGELOG.md are not reworked here; its findings (wrong worktree path in RealSteps::worktree, missing checks stage) are left to a follow-up, except that this job's worktree path must be the live one, job_dir/work
- Running more than one producer at once or raising max_in_flight above 1 (later build jobs)
- Resuming leftover in-flight rows, read-ahead, conflict-return, landing rebase
- Changes to athena.toml, defaults.toml, .github/workflows, or re-enabling sccache
- Changes to crates/athena-cli/src/drain.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for worktree lifecycle and the bound; fmt, clippy, test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise create/remove and numeric bound outputs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks are cleanup after every end path and the bound's values.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to run/mod.rs, disk.rs and CHANGELOG.md.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: T and S labeled as estimates.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 35 cases in 35 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Have the producer and checks code emit typed stage events (an enum with Producer, Checks and so on) at each transition, including the send-back transition, and have the in-flight table record only those. Do not parse display strings. Add a test that runs the real drain path through checks failing an...
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

A test through the run entry point in crates/athena-core/src/run/mod.rs shows a job gets its own worktree and its own target directory under the job dir while it runs, and both are removed after success and after failure; run cargo test -p athena-core.
Unit tests in crates/athena-core/src/disk.rs show the bound floor((free_gb - reserve_gb - slots*T)/(T+S)) for numeric cases, including exactly one job, zero when free disk is below the reserve, and no underflow with unsigned or negative inputs; run cargo test -p athena-core disk.
With max_in_flight = 1 the existing drain tests pass unchanged: git diff shows no edits to existing test bodies, and cargo test passes.
cargo fmt --check and cargo clippy --all-targets -- -D warnings are clean and cargo test passes for the workspace.
T and S in disk.rs are marked as estimates (6 GB and 0.2 GB per docs/design/async-drain.md section 5) because A1 (20261029-142844-ynv3) recorded no measurement; no measured number is claimed.
CHANGELOG.md has an entry for per-job worktree start and the disk bound, naming build job A4 and stating that T and S are estimates.
The worktree path used is the live work tree (job_dir/work, per run/producer.rs), not the kept-work directory, addressing the A3 finding (20261029-142853-x4nr).
athena.toml, defaults.toml and .github/workflows are unchanged (git diff).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
