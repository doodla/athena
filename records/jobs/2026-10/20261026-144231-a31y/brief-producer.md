# Job 20261026-144231-a31y: Split records.rs into submodules under the code-health limits

## Request (verbatim)

> Structural pass: crates/athena-data/src/records.rs
> 
> Code health found crates/athena-data/src/records.rs over a limit: 2543 production lines, over the limit of 2000; 2543 production lines with 7 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week.
> 
> Measured: 2543 production lines (test modules excluded); longest function canonical_path at 85 lines; 7 landings in the last 7 days; 17 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: 2543 production lines, over the limit of 2000
> - The file is under the limit it breached, as measured by code health: 2543 production lines with 7 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-data/src/records.rs

## Reading (accepted by the user)

Restructure crates/athena-data/src/records.rs (2543 production lines) by moving cohesive groups of record types into submodules under crates/athena-data/src/records/, re-exported from records so every public path is unchanged. The goal is for code health to see records.rs under 1500 production lines, with no behavior change; the audience is the code-health gate and maintainers.

Assumptions:
- Code Survey: Code survey: read crates/athena-data/src/records.rs and crates/athena-data/src/lib.rs (pub mod records). records.rs holds ~3200 lines: core job/invocation/verification types (top), introspection, failure-class, proposal, work-item, audience-rating, audit, throughput, build-measurement, Hestia sweep, job-plan/critique, baseline and replay types, then the AnyRecord enum, RecordError, AnyRecord impl, check_record_file and is_job_dir, then #[cfg(test)] mod tests at line 3216.
- Code survey: no record or data file is added, so the check_record_file/AnyRecord allow-list and round-trip test are unchanged; AnyRecord and check_record_file stay in records.rs and keep working through re-exports.
- Plan: move self-contained groups (e.g. introspection+class labels+proposals; work items+audience+audit; throughput+build measurement+Hestia; plan/critique/baseline/replay) into records/*.rs as private modules with `pub use` in records.rs, preserving every public path such as athena_data::records::Foo. Items used across modules use pub(crate)/use super::*. Leave the tests module in place so tests still compile.
- Limit reading: code health counts production lines excluding test modules; the 1500-line limit is the tighter target, so records.rs should end clearly below 1500 production lines and each new file also below it.
- No behavior change: no signatures, serde attributes, schema constants or validation logic are edited, only moved.

## Acceptance criteria

1. [behavior] cargo test -p athena-data passes with the existing tests unchanged in substance (run cargo test).
2. [behavior] cargo clippy --all-targets and cargo fmt --check are clean.
3. [content] crates/athena-data/src/records.rs has well under 1500 production lines (excluding the cfg(test) module), and every new file under crates/athena-data/src/records/ is also under 1500; verifiable with Read/Grep on line counts.
4. [content] All previously public items remain reachable at the same athena_data::records::* paths via pub use re-exports; no other crate needs import edits (cargo check --workspace passes).
5. [content] The diff only moves code: no changes to serde attributes, schema constants, field types, or function bodies, checkable with git diff.
6. [content] AnyRecord, check_record_file and RecordError remain in records.rs and still cover every moved record type.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/records/introspection.rs (create)
- crates/athena-data/src/records/work.rs (create)
- crates/athena-data/src/records/throughput.rs (create)
- crates/athena-data/src/records/plan.rs (create)
- crates/athena-data/src/records/core.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Behavior changes, renamed public items, or edits to other crates
- Splitting the tests module
- Changes to athena.toml or code-health limits

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Pure Rust refactor.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move-only refactor.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test/clippy/fmt.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Split by responsibility, not trimming lines.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier runs cargo checks.
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

- path: <state>/jobs/20261026-144231-a31y/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-144231-a31y/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

cargo test -p athena-data passes with the existing tests unchanged in substance (run cargo test).
cargo clippy --all-targets and cargo fmt --check are clean.
crates/athena-data/src/records.rs has well under 1500 production lines (excluding the cfg(test) module), and every new file under crates/athena-data/src/records/ is also under 1500; verifiable with Read/Grep on line counts.
All previously public items remain reachable at the same athena_data::records::* paths via pub use re-exports; no other crate needs import edits (cargo check --workspace passes).
The diff only moves code: no changes to serde attributes, schema constants, field types, or function bodies, checkable with git diff.
AnyRecord, check_record_file and RecordError remain in records.rs and still cover every moved record type.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
