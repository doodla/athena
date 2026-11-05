# Job 20261105-044134-zycx: Wait for main's in-progress CI run before running the baseline locally

## Request (verbatim)

> Wait for main's GitHub run instead of re-testing locally. When a job starts right after a landing, main's GitHub run for the same code is usually still in progress, so the baseline falls back to a 20-minute local run ('main: no usable CI result ... on main'). This happened in 3 of the last 10 jobs. When main's run for the base's content is in progress, wait for it (reading it every 20 seconds, up to the existing ci_wait_s limit) and reuse it if it passes; run locally only if it fails, is missing, or the wait runs out.
> 
> Files changed: crates/athena-core/src/run/checks.rs and its tests.
> 
> Acceptance:
> - a test where main's run for the base's content is in progress and then passes: the baseline is reused, with no local run
> - a test where main's run fails: the baseline runs locally
> - the existing baseline-reuse tests still pass

## Reading (accepted by the user)

When the baseline lookup finds main's push run for the base's content still in progress, wait for it (a read every 20 s, up to the existing ci_wait_s) and reuse it if it passes. If it fails, is missing, or the wait runs out, the baseline runs locally as today. The audience is Athena operators, who now avoid the 20-minute local baseline right after a landing.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs. reused_baseline resolves the base's content commit, then tries (1) the landed job's candidate run on its CI branch, then (2) the origin/<default> tip run when its fingerprint equals the base's. Both go through green_run, which calls with_ci_source_on(branch, 0, ..), so it makes exactly one read with no wait. A run that is unfinished falls to 'no usable CI result for {sha} on {branch}' and the baseline runs locally in baseline_checks.
- Code survey: crates/athena-core/src/run/mod.rs with_ci_source_on turns wait_s into polls via routing::ci_polls (a read every CI_POLL = 20 s, polls = wait_s/20 + 1) and paces reads with the test sleeper seam ci_seam. The candidate path at mod.rs ~1596 already passes jc.config.budget.ci_wait_s. The change threads ci_wait_s (from ctx.jc.config.budget) into green_run for the main-tip lookup (step 2), where main's run is the one likely still in progress; this reuses the existing wait machinery and adds no new limit.
- Assumption: waiting applies to main's run for the base's content (lookup 2). The landed-candidate lookup (1) is for a finished run on the job's CI branch and may keep one read. Failed, missing, or timed-out runs still fall back to a local run with the existing 'baseline runs locally: ...' line.
- Code survey: the existing baseline-reuse tests are in crates/athena-core/src/run/tests.rs (baseline_reused_from_*, baseline_runs_locally_when_*), using CiReplies/CiReply (Green, Red, Unfinished, Missing) and ci_reuse_run_with. The new tests go there. CiReplies needs a way to answer Unfinished for the first N reads and then Green; run/tests.rs is therefore a likely extra file beyond checks.rs. No record or data file is added under crates/, so the records allow-list is untouched. No refusal rule is added.
- The test for the green case must count reads (baseline_reads) to show the wait polled more than once, and the sleeper seam must show 20 s pauses, so the wait is measured directly.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/tests.rs has main's run for the base's content return in-progress on the first reads and then green. It asserts reused_from is Some(CI_RUN_ADDRESS), baseline_ran is false, more than one read of main happened, and the recorded sleeps are 20 s each. Run: cargo test -p athena-core baseline.
2. [behavior] A new test has main's run for the base's content in progress and then failing (red). It asserts baseline_ran is true, reused_from is None, and the 'ci         baseline runs locally' line is printed. Run: cargo test -p athena-core baseline.
3. [behavior] A new test has main's run still in progress when the ci_wait_s bound is exhausted. It asserts the number of reads equals ci_polls(ci_wait_s) and the baseline runs locally. Run: cargo test -p athena-core baseline.
4. [behavior] All existing baseline tests still pass unchanged: baseline_reused_from_the_landed_candidates_run_runs_nothing_locally, baseline_reused_from_the_default_branch_run_runs_nothing_locally, baseline_reused_from_a_squash_on_main_with_the_bases_fingerprint, baseline_runs_locally_when_main_tip_has_other_governed_content and baseline_runs_locally_when_no_green_ci_result_exists. Run: cargo test -p athena-core.
5. [behavior] cargo clippy --all-targets and cargo fmt --check are clean.
6. [content] The doc comments on reused_baseline and green_run in crates/athena-core/src/run/checks.rs no longer say 'no wait'. They state that main's in-progress run is waited for, reading every 20 s up to ci_wait_s.
7. [content] The diff touches no governance file (athena.toml, defaults.toml, workflows), and reuses ci_wait_s and routing::CI_POLL without adding a new limit.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the ci_wait_s default or adding a new config key
- Changing how the candidate's own checks wait for CI (run/mod.rs ~1596)
- Changing routing.rs, ci_result.rs, or the gh client
- Changing the fingerprint rules for deciding whether main's tip matches the base

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and its tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for wait-then-pass, fail, and timeout; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria assert reuse, no local run, and read counts, not wiring.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing Wait, Paced, ci_polls and ci_wait_s machinery.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the baseline lookup.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run through cargo test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Waits for main's in-progress run instead of falling back on every race after a landing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments are updated for an outside reader.
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

28 more qualifying classes are left out.

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

- path: <state>/jobs/20261105-044134-zycx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-044134-zycx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-core/src/run/tests.rs has main's run for the base's content return in-progress on the first reads and then green. It asserts reused_from is Some(CI_RUN_ADDRESS), baseline_ran is false, more than one read of main happened, and the recorded sleeps are 20 s each. Run: cargo test -p athena-core baseline.
A new test has main's run for the base's content in progress and then failing (red). It asserts baseline_ran is true, reused_from is None, and the 'ci         baseline runs locally' line is printed. Run: cargo test -p athena-core baseline.
A new test has main's run still in progress when the ci_wait_s bound is exhausted. It asserts the number of reads equals ci_polls(ci_wait_s) and the baseline runs locally. Run: cargo test -p athena-core baseline.
All existing baseline tests still pass unchanged: baseline_reused_from_the_landed_candidates_run_runs_nothing_locally, baseline_reused_from_the_default_branch_run_runs_nothing_locally, baseline_reused_from_a_squash_on_main_with_the_bases_fingerprint, baseline_runs_locally_when_main_tip_has_other_governed_content and baseline_runs_locally_when_no_green_ci_result_exists. Run: cargo test -p athena-core.
cargo clippy --all-targets and cargo fmt --check are clean.
The doc comments on reused_baseline and green_run in crates/athena-core/src/run/checks.rs no longer say 'no wait'. They state that main's in-progress run is waited for, reading every 20 s up to ci_wait_s.
The diff touches no governance file (athena.toml, defaults.toml, workflows), and reuses ci_wait_s and routing::CI_POLL without adding a new limit.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
