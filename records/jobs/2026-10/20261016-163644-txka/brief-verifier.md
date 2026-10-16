# Job 20261016-163644-txka: verify "Keep read requests in plan and drain queue; fix plan room check"

## Request (verbatim)

> try again on 20261016-161517-y9dy

## Reading (accepted by the user)

Fix plan/drain so requests read by `athena plan` stay in the queue (queued_work currently lists only Queued state). Also fix Drain::prepare's room check (drain.rs ~694), which requires read+run cap, so the default --budget 2 can never read; plan should reserve room only for the read.

Assumptions:
- Audience: the maintainer running athena plan/drain; verifier runs cargo test, clippy, fmt.
- In plan (not drain) the read needs room only for caps.read; drain still needs read+run before reading and running.
- Approved jobs in Proposed state on drained channels are listed once in queued_work.
- Request symptoms: (1) read requests vanish from plan and drain; (2) default budget 2 reads none; both are mapped to criteria.

## Acceptance criteria

1. [behavior] A test queues requests on a drained channel, runs plan with a budget that reads some (stub reader), and shows the read requests remain in the plan entries and in queue() with read=true. Run cargo test.
2. [behavior] A test shows that after a plan that reads requests, drain still works them: they are in its queue, not read again, run in plan order. cargo test.
3. [behavior] A second plan after the first prints the same open requests, not 'nothing is open'. cargo test.
4. [behavior] A test runs plan at the default budget (2.0) with the repo's reader_usd/job_usd config values and shows it reads at least one request and keeps it in the queue; the room check in Drain::prepare reserves only the read cap for plan. cargo test.
5. [behavior] Separate named tests, through queued_work over real records, show these stay out of the queue: a read request on a non-drained channel, a running job, a closed job, a declined job. cargo test.
6. [behavior] A test over real records with a work decision approving a Proposed job on a drained channel shows it is listed exactly once. cargo test.
7. [behavior] cargo test, cargo clippy and cargo fmt --check pass.
8. [content] The misleading test name in crates/athena-cli/tests/drain.rs (the_plan_of_a_read_job_names_a_missing_path_...) is replaced or backed by a test that really reads the job first.
9. [content] queued_work doc comment, drain.rs module text, README (lines wrapped to neighbouring width) and CHANGELOG describe the new queue rule and the corrected --budget meaning.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Changing reader_usd or job_usd values in athena.toml
- Changing the default --budget value
- Changes to other channels' behavior or job state machine

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests per case; cargo test, clippy, fmt --check pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise plan/drain behavior with the stub reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix queue rule and room check, not one instance.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to steward.rs, drain.rs, their tests, and docs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base cd54d5bc5400 → candidate 246f13598bf0. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                      |   9 ++
 README.md                         |   8 +-
 crates/athena-cli/src/drain.rs    |  32 ++--
 crates/athena-cli/tests/drain.rs  | 308 +++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/steward.rs |  19 ++-
 5 files changed, 357 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (757 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
