# Job 20261027-054145-b5ka: verify "Stop stalled agents early instead of burning the full deadline"

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

The agent driver (crates/athena-agent/src/claude.rs) only enforces a fixed wall-clock deadline and has no progress tracking, so a stalled producer burns the full 1800s before the job learns it. Add a stall watchdog in the driver: if the agent's stream shows no activity for a fraction of the timeout, kill it and report a timed-out outcome whose reason says it stalled. The existing timed-out work-keeping path (keep_timed_out_work) then preserves the partial work. Audience: Athena maintainers and the verifier.

Assumptions:
- Root cause: the driver loop in claude.rs checks only Instant::now() >= clock + spec.timeout, with no notion of progress, so stalled and slow producers are indistinguishable until the full deadline.
- Work retention on timeout already exists (keep_timed_out_work, TimedOut outcome, athena retry seeding), so the missing piece is early detection, not resumability.
- Progress is read as any new stream-json line from the agent; the stall window is derived from spec.timeout (about one third, with a floor) so no AgentInvocation field is added and every role benefits.
- Stalled kills reuse InvocationOutcome::TimedOut with a distinct 'stalled' detail so downstream handling stays the same.
- No change to athena.toml or defaults.toml is needed.

## Acceptance criteria

1. [content] The change description names the root cause: the driver enforces only a fixed wall-clock deadline and never tracks progress, so a stalled agent is indistinguishable from a slow one.
2. [behavior] A new test in crates/athena-agent/src/claude.rs runs a stand-in agent that emits nothing and hangs, with a long timeout. It asserts the driver ends it well before the deadline, with outcome TimedOut and a detail that says it stalled. It fails on the base commit and passes with the change. Run: cargo test -p athena-agent.
3. [behavior] A test shows an agent that keeps emitting stream lines past the stall window is not killed early, and the existing deadline test (detail 'deadline of 2s reached') still passes. Run: cargo test -p athena-agent.
4. [behavior] cargo fmt --check, cargo clippy --workspace --all-targets, and cargo test --workspace all pass.
5. [content] The stall rule is implemented once in the shared driver loop, not per role or per job, so every agent role gets it.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Splitting jobs into sub-jobs in planning
- Changing the configured producer timeout values
- Per-test-command budget wrappers
- Changing records/ or past job outcomes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New stall tests plus workspace cargo test and clippy criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in the shared driver loop, not per job or role.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests run a hanging stand-in and check the early timed-out outcome.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust change only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the driver.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, and fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 5cd1cc075c11 → candidate 85350c45f505. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  8 +++++
 crates/athena-agent/src/claude.rs | 63 ++++++++++++++++++++++++++++++++++++++-
 crates/athena-agent/src/procs.rs  | 12 ++++++++
 3 files changed, 82 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1417 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --workspace --all-targets, and cargo test --workspace all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
