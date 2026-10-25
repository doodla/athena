# Job 20261016-170410-arfv: verify "Keep plan-read requests in the plan and drain queue; fix --budget room"

## Request (verbatim)

> try again on 20261016-163644-txka

## Reading (accepted by the user)

Retry of the rejected fix. Requests read by `athena plan` must stay in the plan and drain's queue (queued_work currently lists only Queued), and plan's room check in Drain::prepare must reserve only the read cap so the default --budget 2 reads. The audience is the operator running plan then drain. The retry also fixes the earlier misses: criterion 7 had no recorded commands, a running-job test faked state, and README wrapping was uneven.

Assumptions:
- The typed request 'try again' takes precedence: redo the original plan/drain fix, not the proposed pre-handover evidence-check feature.
- Goal is that an operator who runs `athena plan --budget N` then `athena drain` sees read requests still open and worked in plan order.
- Read cap is the reader_usd config value; plan at default budget 2.0 must be able to afford one read.
- The producer must run cargo test, cargo clippy and cargo fmt --check on the final state and record the exact commands and results in its handover.
- No governance file changes are needed.

## Acceptance criteria

1. [behavior] A test queues requests on a drained channel, runs plan with a budget that reads some (stub reader), and shows the read requests remain in the plan entries and in queue() with read=true. Run cargo test.
2. [behavior] A test shows that after a plan that reads requests, drain still works them: they are in its queue, not read again, and run in plan order. Run cargo test.
3. [behavior] A second plan after the first lists the same open requests, not 'nothing is open'. Run cargo test.
4. [behavior] A test runs plan at the default budget (2.0) with the repo's reader_usd/job_usd values, shows it reads at least one request and keeps it queued; Drain::prepare's room check reserves only the read cap for plan. Run cargo test.
5. [behavior] Separate named tests through queued_work over real records show these stay out of the queue: a read request on a non-drained channel, a closed job, a declined job, and a running job. The running-job test uses a genuinely dispatched-but-unfinished record, not outcome fields cleared in memory. Run cargo test.
6. [behavior] A test over real records with a work decision approving a Proposed job on a drained channel shows it is listed exactly once. Run cargo test.
7. [behavior] cargo test, cargo clippy and cargo fmt --check all pass, and the handover cites each exact command that ran with its exit status and output tail.
8. [content] The misleading test name in crates/athena-cli/tests/drain.rs (the_plan_of_a_read_job_names_a_missing_path_...) is replaced or backed by a test that really reads the job first.
9. [content] queued_work doc comment, drain.rs module text, README and CHANGELOG describe the new queue rule and the corrected --budget meaning; README paragraphs are re-wrapped to the neighbouring line width with no early breaks.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- A new pre-handover evidence-check mechanism in the build pipeline
- Changes to the reader, verifier or other commands beyond plan/drain queue handling
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New named tests plus cargo test, clippy and fmt --check criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check plan and drain behavior through tests, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix queued_work's queue rule and the room check, not just the observed case.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Handover must cite commands that ran, as the last verdict demanded.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the plan/drain queue.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base e57794a29eab → candidate ff521978cb5c. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                      |   8 +
 README.md                         |  20 +-
 crates/athena-cli/src/drain.rs    |  44 +++--
 crates/athena-cli/src/main.rs     |   3 +-
 crates/athena-cli/tests/drain.rs  | 383 +++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/steward.rs |  19 +-
 6 files changed, 440 insertions(+), 37 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (758 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
