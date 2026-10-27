# Job 20261027-074449-beam: Add a stall watchdog to the agent driver so stalled producers stop early

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The producer runs against a fixed wall-clock deadline. Nothing in the job tracks progress, checks in
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: producer-deadline-exceeded-on-small-job (20261017-032151-9ymk)
> Cause: The producer runs against a fixed wall-clock deadline. Nothing in the job tracks progress, checks in partway, or tells a stalled producer from a slow one. A short prompt-edit job therefore burns the full 1800 seconds before anyone learns it...
> Seen 9 time(s) in 8 jobs. The introspections that record it:
> Introspection 20261017-032141-7xvh (job 20261010-154006-vc5k); prevention it proposed: Add a producer progress watchdog. It would require a visible step, such as a file written or a commit made, within a short window (about 10 minutes). If none appears, it would stop the producer early and record its last activity and partial...
> Introspection 20261017-044251-1804 (job 20261016-052605-qw38); prevention it proposed: Change the producer stage so a deadline does not discard its work. (1) The producer commits checkpoints and writes a progress note as it goes. (2) On deadline, the job keeps the partial result and is marked "timed out, resumable", which is ...
> Introspection 20261018-073427-vw9w (job 20261018-070117-qmxv); prevention it proposed: Make the producer stage resumable and size-aware. (1) Before the producer runs, a planning step must split any job with more than one separable deliverable into sub-jobs, each of which fits one deadline. (2) The producer must commit partial...
> Introspection 20261023-234113-5z2s (job 20261019-013309-nvbb); prevention it proposed: Give the producer stage a progress signal and an early-stop rule. It would write a checkpoint (files touched, criteria addressed) at intervals. If no checkpoint or diff appears within a fraction of the deadline, the stage aborts early and r...
> Introspection 20261023-234230-55v8 (job 20261023-234149-jcd2); prevention it proposed: Put the test budget in the harness, not in the prompt. Record the repo's scoped test command per crate and the typical full-suite duration. Run producer test commands through a wrapper that allows the full suite once, or twice at most, and ...
> Introspection 20261024-002507-470t (job 20261023-234149-jcd2); prevention it proposed: Give the producer stage a progress signal and an early stop. It should write a checkpoint of its partial diff and a short status note at set intervals. If it makes no progress for a set period, it should stop early and report what it found,...
> Introspection 20261024-145615-7s8b (job 20261024-140624-ewbs); prevention it proposed: Make the producer's deadline recoverable. (1) The producer commits checkpoints to the job branch and writes a short progress note as it goes. (2) On deadline, Athena keeps the partial work and the last note, marks the job "timed out with pa...
> Introspection 20261025-055532-4grg (job 20261025-034731-2am0); prevention it proposed: Make the producer stage report progress and keep partial work. It should write a checkpoint (files changed, last command, last test run) at intervals. On deadline, it should record that checkpoint as a timeout result separate from a real fa...
> Introspection 20261025-150430-h29z (job 20261017-044405-4zcq); prevention it proposed: Make a producer timeout a resumable state and not a bare failure. On deadline, Athena would snapshot the producer's working tree and its last activity, record the job as timed out, and let a retry start from that snapshot with a note of wha...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the agent driver loop in crates/athena-agent/src/claude.rs enforces only the fixed wall-clock deadline (1800s for the producer). It cannot tell a stalled agent from a slow one, so a stalled run burns the whole deadline (the failure class behind jobs 20261010-154006-vc5k, 20261016-052605-qw38 and 20261018-070117-qmxv). Fix: the driver tracks the last time the agent's stream showed activity and ends the run early, as a timed-out outcome with a stall reason, when none appears for a fraction of the timeout (1/3, about 10 minutes at 1800s). Existing resumable-timeout handling (kept work tree, progress note) then applies.

Assumptions:
- Goal: stop a stalled producer early instead of letting it burn the full deadline. Audience: Athena's operator and maintainers who read the failure outcome.
- Job 20261010-154006-vc5k, job 20261016-052605-qw38 and job 20261018-070117-qmxv: each ended with 'deadline of 1800s reached' and was never verified. Their original requests (verifier prompt note, auditor and warden roles, baseline checks) are separate work and are NOT redone here. The shared cause is addressed, so a retry of them can fail fast or resume. Their prior deliverables are not changed by this job.
- The 'keep partial work, resumable timeout' half of the proposals already exists (keep_timed_out_work in athena-core run/mod.rs, InvocationOutcome::TimedOut, a progress note, and a retry seeded from the kept tree). The missing piece is the stall detection, so this job adds only that.
- Activity means any line on the agent's stdout stream (tool calls, messages), so a hung or silent agent is caught. The stall window is derived from the invocation timeout (timeout/3) rather than a new AgentInvocation field, to avoid touching every constructor.
- A stall stop reuses InvocationOutcome::TimedOut with a distinct detail such as 'no progress for Ns'; no new outcome variant, so records and the schema are unchanged.

## Acceptance criteria

1. [content] The change description names the root cause: the driver has only a fixed deadline and no progress tracking or stall detection. The fix is in the driver loop in crates/athena-agent/src/claude.rs, with the reason string built in procs.rs next to deadline_reason.
2. [behavior] A new test in crates/athena-agent/src/claude.rs uses a fake agent that emits one line and then goes silent, with a long timeout. It asserts that the run ends well before the timeout with outcome TimedOut and a detail naming a stall (no progress). It fails on the base commit and passes with the change. Run: cargo test -p athena-agent.
3. [behavior] A second test shows an agent that keeps emitting lines, slowly but within the stall window, is not stopped early and still completes. It proves slow is told apart from stalled, so the fix covers the class and not one job.
4. [behavior] The existing deadline test (detail 'deadline of 2s reached') still passes unchanged, and an agent that is silent for the whole timeout still reports the deadline reason when the stall window is not shorter than the timeout.
5. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test (whole workspace) all pass.
6. [content] CHANGELOG.md has an entry describing the stall watchdog and its window.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-agent/src/lib.rs (modify)

## Out of scope

- Redoing the original work of jobs 20261010-154006-vc5k (verifier prompt note), 20261016-052605-qw38 (auditor and warden roles) and 20261018-070117-qmxv (baseline checks); the verifier prompt and briefs, auditor and warden modules, and check baseline logic stay unchanged
- Splitting jobs into sub-jobs or a planning step
- A test-budget wrapper for producer commands (introspection 5z2s / jcd2 test budget)
- Changing the 1800s deadline value, athena.toml, defaults.toml or CI workflows
- Reworking the existing kept-work and retry mechanism
- Adding a new InvocationOutcome variant or record schema change

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New stall and no-false-stall tests; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in the shared driver loop used by all roles, not in one job or prompt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run a fake silent agent and check it stops early with a stall reason.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, with no new invocation field and no new outcome variant.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a stalled run ends early, not that a watchdog exists.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
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

16 more qualifying classes are left out.

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

The change description names the root cause: the driver has only a fixed deadline and no progress tracking or stall detection. The fix is in the driver loop in crates/athena-agent/src/claude.rs, with the reason string built in procs.rs next to deadline_reason.
A new test in crates/athena-agent/src/claude.rs uses a fake agent that emits one line and then goes silent, with a long timeout. It asserts that the run ends well before the timeout with outcome TimedOut and a detail naming a stall (no progress). It fails on the base commit and passes with the change. Run: cargo test -p athena-agent.
A second test shows an agent that keeps emitting lines, slowly but within the stall window, is not stopped early and still completes. It proves slow is told apart from stalled, so the fix covers the class and not one job.
The existing deadline test (detail 'deadline of 2s reached') still passes unchanged, and an agent that is silent for the whole timeout still reports the deadline reason when the stall window is not shorter than the timeout.
cargo fmt --check, cargo clippy --all-targets and cargo test (whole workspace) all pass.
CHANGELOG.md has an entry describing the stall watchdog and its window.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
