# Job 20261016-161517-y9dy: Keep read requests in the plan and drain queue

## Request (verbatim)

> athena plan (from 20261016-060424-614b) reads unread requests to fill their plans, but reading moves a request from queued to proposed, and plan and drain only look at queued ones: after one 'athena plan --budget 15' all 25 open requests became proposed, plan says nothing is open, and drain has nothing to work. Also at the default --budget 2 it read none ('no room in the job limit or the budget'). Read requests must stay in the plan and the drain's queue.

## Reading (accepted by the user)

Reading a queued request (state Queued to Proposed) drops it from `athena plan` and `athena drain`, because `steward::queued_work` only lists state Queued. I will make the queue also include requests on the drained channels that have been read but not run, as Items with read=true, so plan and drain keep seeing them after a read.

Assumptions:
- The goal is for operators using `athena plan` and `athena drain`: a request that has been read but not yet run stays in the plan and in the drain's queue.
- 'Read' means state Proposed on the steward or `ask --later` channels; jobs approved through work decisions are already listed by approved_work and must not be listed twice.
- The 'default --budget 2 read none' remark is read as a symptom to check: if the room check is wrong, fix it. Otherwise it is left as is, and this change makes it harmless because unread requests stay queued.
- Declined requests stay out of the queue, since they wait for a correction or close.
- The default plan budget and the plan's read-by-default behavior are not changed.

## Acceptance criteria

1. [behavior] A test queues requests on a drained channel, runs `plan` with a budget that reads some of them (stub reader), and then shows that the read requests are still in the plan entries and in `queue()` with read=true. Run `cargo test`.
2. [behavior] A test shows that after a plan that reads requests, `drain` still works them: they are in its queue, are not read a second time, and are run in plan order. Run `cargo test`.
3. [behavior] A second `plan` after the first one reads requests prints the same open requests, not 'nothing is open'. The test checks this with the stub-backed plan output.
4. [behavior] Read requests on channels other than the drained ones, jobs that are running or closed, and declined jobs do not enter the queue. A test covers each. `cargo test`, `cargo clippy` and `cargo fmt --check` pass.
5. [content] The misleading test name `the_plan_of_a_read_job_names_a_missing_path_and_an_unresolved_referenced_job` in crates/athena-cli/tests/drain.rs is replaced or backed by a test that really reads the job before checking its plan.
6. [content] The doc comment of `queued_work` and the module text of drain.rs describe the new rule: read requests stay queued for the drain until they run or close.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Making `athena plan` read-only by default.
- Changing JobState semantics or the state machine in store.rs.
- Drain behavior for other channels or declined jobs.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the plan and drain queue are added, with cargo test and clippy clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in how the queue is derived from job state, not in each caller.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that read requests remain in the plan and are worked by drain.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands to run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the queue selection.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A test asserts the plan is non-empty after reading, not just that nothing errors.
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
