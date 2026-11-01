# Job 20261101-123956-y5q1: verify "Reclaim interrupted jobs' build dirs; hold on low disk before producer"

## Request (verbatim)

> try again on 20261101-102128-b1st
> The earlier attempt passed review and was rejected only because the verdict did not count GitHub's reused CI result as a test run; that is now fixed. Redo the same change.
> Priority: urgent

## Reading (accepted by the user)

Redo job 20261101-102128-b1st (rejected only because its gate commands were not seen running): at drain start and before each run, reclaim target/ of interrupted Dispatched jobs (no registry entry, free lock), keeping work trees and candidates, and hold the job as an environment stop on low disk before the producer. Audience: whoever runs `athena drain` unattended.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs (the disk check before the checks calls hold_for_disk at ~line 1303), crates/athena-core/src/run/close.rs (hold_for_disk), crates/athena-core/src/run/verification.rs (second hold_for_disk caller), crates/athena-core/src/state.rs (StateDir::with_free_space hook), crates/athena-core/src/disk.rs (cleanup and tests using with_free_space), crates/athena-cli/tests/drain.rs and tests/common/mod.rs (drain entry-point tests with simulated_free). No record or data file is added, so check_record_file/AnyRecord are unaffected. No refusal rule is tightened; the disk hold reuses the existing environment-stop mechanism. Free space is measured through the same hook the existing tests use.
- Prior job 20261101-102128-b1st: its candidate passed review. The one major finding (gate commands not cited) is addressed by running fmt, clippy and test as the final step and citing them in the criteria. The minor finding on close.rs is addressed by listing close.rs as a deliverable (its change is only a hold_for_disk doc comment naming the new caller), so the candidate's work is kept. The lock-window and records-reload findings are accepted by design and documented in CHANGELOG and drain.rs docs.
- The retry starts from the prior candidate's approach: the sweep removes only target/ for jobs that are Dispatched with no registry entry, are not on the keep list, and whose job lock is free; a pre-producer hold leaves no candidate and no resume mark.
- Per the earlier introspection, adding an Athena pre-verdict gate is a separate job and is not part of this one.

## Acceptance criteria

1. [behavior] A new integration test in crates/athena-cli/tests/drain.rs, run with `cargo test -p athena-cli --test drain`, enters through the drain entry point with an interrupted Dispatched job's target/ present and asserts target/ is gone after drain start while the job's work tree and candidate remain.
2. [behavior] A new test, run with `cargo test`, makes a job's lock held or its process alive (registry entry) and asserts its target/ is kept, and asserts a sweep before a later run reclaims a newly interrupted job's target/.
3. [behavior] A new test, run with `cargo test`, simulates low free disk only at the pre-producer point (via with_free_space) and asserts the job is held as an environment stop for low disk, the status says so, and the producer is never started.
4. [behavior] `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` all pass when run from the repository root.
5. [content] Grep of the diff shows a disk gate before the producer is invoked in crates/athena-core/src/run/producer.rs or run/mod.rs that holds through hold_for_disk, and no test fixture is pulled into a src unit test by #[path] from tests/common.
6. [content] CHANGELOG.md has a new entry on reclaiming interrupted jobs' build directories and the pre-producer disk check; module docs in drain.rs and disk.rs describe the new behavior.
7. [content] The diff does not touch athena.toml or defaults.toml, does not change the min_free_gb default, and does not delete work trees or candidates.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)

## Out of scope

- Deleting work trees or candidates of interrupted jobs
- Changing the min_free_gb default or athena.toml / defaults.toml
- Changing how closed and idle jobs are reclaimed
- Automatic retry policy changes for held jobs
- Including tests/common/mod.rs into src unit tests
- Adding an Athena-run pre-verdict gate (separate job)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the reclaim and the hold actually happening via tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy -D warnings and cargo test criterion.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands to run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Reclaim covers all interrupted jobs and adds a pre-producer disk gate, not the one instance.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Module docs and CHANGELOG entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d647159c3942 → candidate d9fa9a4354ce. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  10 +++
 crates/athena-cli/src/drain.rs         |  25 ++++++-
 crates/athena-cli/tests/drain.rs       |  56 ++++++++++++++
 crates/athena-core/src/disk.rs         | 132 ++++++++++++++++++++++++++++++++-
 crates/athena-core/src/run/close.rs    |   5 +-
 crates/athena-core/src/run/mod.rs      |  10 ++-
 crates/athena-core/src/run/tests.rs    |  66 +++++++++++++++++
 crates/athena-core/src/test_support.rs |  21 ++++++
 8 files changed, 318 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41311733126 for d9fa9a4354ce5b42105cbf7c789c3c8d61b07e7f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41311733126 for d9fa9a4354ce5b42105cbf7c789c3c8d61b07e7f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41311733126 for d9fa9a4354ce5b42105cbf7c789c3c8d61b07e7f)
- ci-result: `not a check` passed (Check results for d9fa9a4354ce5b42105cbf7c789c3c8d61b07e7f come from the CI run https://github.com/doodla/athena/actions/runs/41311733126 (585s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` all pass when run from the repository root.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
