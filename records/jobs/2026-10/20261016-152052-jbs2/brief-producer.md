# Job 20261016-152052-jbs2: Keep read requests in the plan and drain queue

## Request (verbatim)

> athena plan (from 20261016-060424-614b) reads unread requests to fill their plans, but reading moves a request from queued to proposed, and plan and drain only look at queued ones: after one 'athena plan --budget 15' all 25 open requests became proposed, plan says nothing is open, and drain has nothing to work. Also at the default --budget 2 it read none ('no room in the job limit or the budget'). Read requests must stay in the plan and the drain's queue.

## Reading (accepted by the user)

Reading a request currently moves it from queued to proposed, so plan and drain drop it. Make read requests stay in the plan and the drain queue, and explain or fix why plan at the default budget 2 reads none. The audience is the maintainer running plan and drain.

Assumptions:
- Goal: the maintainer running athena plan and drain on a real queue. Read requests must stay in plan and drain.
- The fix is to make plan and drain treat read (proposed) requests as workable, or to keep them queued after reading. I chose this over a new state.
- The default budget 2 reading none is a separate symptom. I will make it consistent, with a test, only if the cause is simple.
- Exact files are not confirmed. They are guessed from grep hits on Proposed in steward.rs, drain.rs, main.rs, and reading.rs.

## Acceptance criteria

1. [behavior] A test queues several unread requests, runs plan with a budget that reads some, then checks the read requests still appear in the plan and in the drain's queue. Run `cargo test`.
2. [behavior] After `athena plan --budget N` reads requests, `athena plan` does not say nothing is open, and drain still has the read requests to work. Checked by a stub-backed CLI test run with `cargo test`.
3. [content] A request that has been read is still schedulable by plan and drain (it is no longer dropped from the queue). Its state or its selection is changed at the root cause, not only in plan.
4. [behavior] The default budget gives the reading room to read unread requests, or the 'no room in the job limit or the budget' message is explained and covered by a test. Run `cargo test`.
5. [behavior] `cargo test` passes, `cargo clippy` is clean and `cargo fmt --check` passes.
6. [content] README/help and CHANGELOG describe that reading keeps requests queued. The misleading test name from the earlier verification is fixed.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Making plan read-only by default (an earlier nit, not requested here).
- Redesigning the job state machine beyond keeping read requests workable.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Regression tests plus cargo test and clippy.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the queue-state handling, not just the plan output.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that read requests stay in plan and drain.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is exercised by CLI tests.
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
