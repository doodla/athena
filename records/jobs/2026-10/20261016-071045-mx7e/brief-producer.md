# Job 20261016-071045-mx7e: Keep read requests in the plan and drain queue; plan reads fit budget

## Request (verbatim)

> athena plan (from 20261016-060424-614b) reads unread requests to fill their plans, but reading moves a request from queued to proposed, and plan and drain only look at queued ones: after one 'athena plan --budget 15' all 25 open requests became proposed, plan says nothing is open, and drain has nothing to work. Also at the default --budget 2 it read none ('no room in the job limit or the budget'). Read requests must stay in the plan and the drain's queue.

## Reading (accepted by the user)

`athena plan` reads unread requests, which makes them Proposed, and the queue view (steward::queued_work plus approved_work) then drops them, so plan and drain see nothing open. Fix the queue so a drained-channel job that was read and has a runnable reading stays in the plan and the drain queue as a read item, and make planning reads need only the reader cap (not reader+job caps), so the default $2 budget reads. Audience: the operator running `athena plan` then `athena drain`.

Assumptions:
- Root cause 1: RealSteps::queue (crates/athena-cli/src/drain.rs ~1228) lists only approved_work (steward-approved proposals) and queued_work (JobState::Queued, drained channels); a request read by the planning pass becomes JobState::Proposed and matches neither.
- Fix 1: add a steward function (e.g. read_work) listing JobState::Proposed jobs on DRAINED_CHANNELS whose latest reading has a capability, and queue them as read=true items with plan entries, deduplicated against approved_work. Declined jobs, readings without capability, and jobs the user has already closed stay out.
- Root cause 2: Drain::prepare requires budget.fits(caps.read + caps.run) (reader_usd 0.50 + job_usd 8.00 = 8.50 in athena.toml), which exceeds the default --budget 2, so nothing is read. Planning never runs the job, so in plan mode it should require only caps.read (drain keeps its own run check).
- A read item whose plan is still incomplete after its reading remains in the plan and is flagged for correction (existing note), not dropped.
- The nit about the misleading test name in drain.rs tests and the default-budget read-only wording from job 614b are not required; only the two reported defects are fixed. Docs/README/CHANGELOG are updated where they say read requests leave the queue.

## Acceptance criteria

1. [behavior] A test using stub steps and a real-records fixture shows: queue a vague request, run plan with a budget that allows reading, then a second `plan` (and the drain's queue view) still lists that request, now as read, instead of 'nothing is open'. Run: cargo test -p athena-cli.
2. [behavior] A test shows plan at the default budget 2 with reader_usd 0.50 and job_usd 8.00 reads an underspecified unread request (no 'no room in the job limit or the budget' note), while drain still requires reader+job room before starting a job. Run: cargo test -p athena-cli.
3. [behavior] A test shows drain works a request that plan already read: it runs it without a second reading and counts it as read=true. Run: cargo test.
4. [behavior] Declined jobs, jobs with no capability reading, and closed jobs are not listed in the queue; a unit test in athena-core steward covers the new listing function. Run: cargo test -p athena-core.
5. [behavior] cargo clippy --all-targets is clean and cargo fmt --check passes.
6. [content] README/help text for `athena plan` and the drain doc comment no longer say that a read request leaves the queue; CHANGELOG notes the fix.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Migrating or re-queuing the 25 already-proposed requests by hand (the new listing picks them up if they have a runnable reading).
- Making `athena plan` read-only by default.
- Renaming the misleading test from the earlier verification nit.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fix is in Rust crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for both defects; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the queue view's state filter and the plan budget gate, not just the 25 stuck requests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check plan then plan/drain still seeing the request, not that functions exist.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior verified by running cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to queue listing and plan budget gate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and README updated for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

## Report

End with the producer report as your structured output.
