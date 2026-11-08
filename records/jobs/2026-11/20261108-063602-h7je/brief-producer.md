# Job 20261108-063602-h7je: Split drain.rs into child modules to get under 1500 production lines

## Request (verbatim)

> try again on 20261026-144234-bz4e

## Reading (accepted by the user)

Retry of job 20261026-144234-bz4e: the earlier request's 1798-line figure was stale; drain.rs now has about 4585 production lines (first test module at line 4586). Split it into several child modules (lifecycle, real_steps, lander, engine) so drain.rs ends under 1500 production lines, with no behavior change. Audience: the code-health gate and maintainers.

Assumptions:
- Job 20261026-144234-bz4e is addressed: its blocker (drain.rs still ~4013 production lines) is fixed by moving four groups, not one; its major finding (stale figure, no re-measure) is fixed by requiring the producer to measure production lines of drain.rs before and after and report both numbers; its minor CHANGELOG finding is fixed by wording the entry only after the measured result is true. Its candidate files are all kept as deliverables: CHANGELOG.md, drain.rs, drain/lifecycle.rs (the lifecycle move itself was sound and is redone). main.rs was in its scope but is not expected to change, since drain re-exports keep paths stable.
- Code survey: read crates/athena-cli/src/drain.rs top-level items. Production code runs to line 4585; test modules start at 4586 (batches_tests, tests, in_flight_tests, read_ahead_tests, landing_queue_tests). Groups by line: lifecycle (PidRecord..RealLifecycle, 3457-4027, ~570 lines); RealSteps and its Steps impl (4028-4585, ~560); Lander/PrLander/PlanReport and study helpers (3199-3456, ~260); the Drain engine (State, Carry, LandingQueue, Drain, ItemEnd, RunStep and impls, 1181-3198, ~2000); data types and the Steps trait (288-1180, ~900 incl. InFlight and Report). Callers in main.rs and display.rs use crate::drain paths, so glob or explicit re-exports must keep those paths. Tests use super::*, so private items moved out need pub(super) visibility and re-import in drain.rs.
- Target math: 4585 minus lifecycle, real_steps, lander and engine (about 3390 lines) leaves roughly 1200 production lines in drain.rs, under 1500. If the engine move proves not cleanly separable, the producer moves the next largest cohesive group (for example InFlight/Report types) instead, and reports the measured number.
- Measuring production lines: test modules (#[cfg(test)] mod blocks) are excluded, as code_health::measure_source does. The producer measures with that tool or by counting lines before the first #[cfg(test)] module, in each new file as well, and every new file must stay under 1500 lines too, since the limit is per file with 5 landings a week.
- Child modules live under crates/athena-cli/src/drain/ and are declared in drain.rs with mod and pub use re-exports. Moves only; no function body logic changes.
- Governance: athena.toml code-health limits are not changed; no governance file is a deliverable.

## Acceptance criteria

1. [content] Production lines of crates/athena-cli/src/drain.rs (everything before the first #[cfg(test)] module) are fewer than 1500. Check by Grep for '^#\[cfg\(test\)\]' with line numbers in drain.rs and reading the first match; the line before it is the production line count.
2. [content] New files under crates/athena-cli/src/drain/ (at least lifecycle.rs, real_steps.rs, lander.rs, engine.rs) each have fewer than 1500 production lines, checked by Grep for line numbers of the first #[cfg(test)] or the file length via Read.
3. [content] The diff (git diff) only moves code, adds mod and re-export lines, and adjusts imports or visibility such as pub(super). No function body logic changes, and crate::drain public paths used by main.rs and display.rs (guarded, DrainState, StopOutcome, RestartOutcome and others) remain exported from drain.
4. [content] No test is deleted, ignored or edited beyond import paths: the #[cfg(test)] modules stay in drain.rs (or move with their code) with the same test function names. Check with git diff and Grep for '#\[test\]' counts before and after.
5. [content] CHANGELOG.md has an entry under Changed that describes the split and states only what is true after the measurement (drain.rs under 1500 production lines).
6. [behavior] Run `cargo test -p athena-cli`: it passes with the same number of tests as before the change (outcome check: drain behavior is unchanged).
7. [behavior] Run `cargo clippy --all-targets -- -D warnings` (clean) and `cargo fmt --check` (passes).
8. [behavior] Run `cargo build`: the whole workspace compiles, so main.rs and display.rs callers need no edits.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/drain/lifecycle.rs (create)
- crates/athena-cli/src/drain/real_steps.rs (create)
- crates/athena-cli/src/drain/lander.rs (create)
- crates/athena-cli/src/drain/engine.rs (create)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Any behavior change or logic refactor of the drain engine
- Changes to athena.toml code-health limits or other governance files
- Splitting other files such as display.rs or main.rs
- Adding the proposed reading-time size-measurement check to Athena itself

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust module moves.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the measured line count and that tests still pass, not only that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused split with no logic change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Re-measure the file instead of trusting the stale figure.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Modules are named by responsibility.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261026-144234-bz4e raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> blocker (crates/athena-cli/src/drain.rs): drain.rs is still about 4013 production lines, far above the 1500 limit it breached and above the 2000 absolute limit, so the structural pass does not reach its goal. In the candidate, drain.rs is 6522 lines, and its #[cfg(test)] modules span 4010–4064, 4066–5125, 5127–5232, 5234–6197 and 6199–6522. code_health::measure_source counts production lines as total lines minus those ranges, which gives about 4013 (my count). The change removed a net 575 lines from a file of about 4588 production lines. Code health will flag the file again on the next landing.
> major (crates/athena-cli/src/drain.rs): The request's line count was stale, and neither the reading nor the producer re-measured the file before planning or finishing the split. The request was queued on 2026-10-26 with 1798 lines. The current file has production code through line 4009. The reading's fallback ('the target is below 1500 production lines in drain.rs') was not acted on. A sounder approach is to measure the file first (measure_source, or the status code-health table) and pick enough cohesive groups to clear the limit.
> major: The reading does not match the request, so passing criteria do not show the request was met The reading took the request's figure of 1798 production lines as current and planned to move one section (from about line 3412 to the end of the production code). In this tree, drain.rs's production code runs to line 4009 before the first test module, so it has about 4588 production lines by the code-health measure, not 1798. The figure was measured when the request was queued on 2026-10-26. The reading should have re-measured the file and planned a split big enough to get under 1500. That means moving several cohesive groups (for example PrLander, the study_*/model_contradictions helpers, the in-flight table and the landing queue), not just the lifecycle block.

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
- 9 cases in 9 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs when a reading is accepted. For any request that names a size or line-count target, it measures the named files in the current snapshot and records the measured values in the reading. It refuses or flags the reading if the stated figure differs from the measurement. Requir...

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

- path: <state>/jobs/20261108-063602-h7je/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-063602-h7je/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Production lines of crates/athena-cli/src/drain.rs (everything before the first #[cfg(test)] module) are fewer than 1500. Check by Grep for '^#\[cfg\(test\)\]' with line numbers in drain.rs and reading the first match; the line before it is the production line count.
New files under crates/athena-cli/src/drain/ (at least lifecycle.rs, real_steps.rs, lander.rs, engine.rs) each have fewer than 1500 production lines, checked by Grep for line numbers of the first #[cfg(test)] or the file length via Read.
The diff (git diff) only moves code, adds mod and re-export lines, and adjusts imports or visibility such as pub(super). No function body logic changes, and crate::drain public paths used by main.rs and display.rs (guarded, DrainState, StopOutcome, RestartOutcome and others) remain exported from drain.
No test is deleted, ignored or edited beyond import paths: the #[cfg(test)] modules stay in drain.rs (or move with their code) with the same test function names. Check with git diff and Grep for '#\[test\]' counts before and after.
CHANGELOG.md has an entry under Changed that describes the split and states only what is true after the measurement (drain.rs under 1500 production lines).
Run `cargo test -p athena-cli`: it passes with the same number of tests as before the change (outcome check: drain behavior is unchanged).
Run `cargo clippy --all-targets -- -D warnings` (clean) and `cargo fmt --check` (passes).
Run `cargo build`: the whole workspace compiles, so main.rs and display.rs callers need no edits.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
