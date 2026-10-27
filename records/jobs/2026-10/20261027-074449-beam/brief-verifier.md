# Job 20261027-074449-beam: verify "Add a stall watchdog to the agent driver so stalled producers stop early"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 506f700438b4 → candidate 87e47b41e060. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  6 ++++
 crates/athena-agent/src/claude.rs | 71 ++++++++++++++++++++++++++++++++++++++-
 crates/athena-agent/src/procs.rs  | 14 ++++++++
 3 files changed, 90 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1460 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets and cargo test (whole workspace) all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
