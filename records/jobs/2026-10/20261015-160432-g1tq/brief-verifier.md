# Job 20261015-160432-g1tq: verify "Reconcile invocation tokens with modelUsage, with end-to-end test"

## Request (verbatim)

> try again on 20261015-145809-7565

## Reading (accepted by the user)

Retry of job 145809-7565: make the invocation's measured tokens and the gate's reconciliation rule use one definition (modelUsage), starting from the base because the earlier changes were discarded. This time add a test that runs a result event through parsing, the invocation record and check_invocation, plus a negative case. The audience is the Athena owner, who needs verified jobs to land.

Assumptions:
- The root cause is that the invocation's tokens come from the result event's top-level usage, while the gate compares them with modelUsage. The two differ when subagents run.
- The one definition is the sum of modelUsage over all models, stated in a doc comment on the usage field. If the producer finds the requested model's modelUsage fits the code better, that is acceptable if the doc comment, the invocation tokens and the gate agree.
- Older records in records/ that used several models may fail the new check. The producer should keep check_records tolerant of them, for example by comparing against the requested model's entry when the legacy shape applies, or by noting the limit. records/ is absent from the snapshot and is never changed.
- The earlier attempt's design was sound; the failure was the missing end-to-end test, so the retry repeats the fix and adds that test.
- The final report includes a table mapping each acceptance criterion to the named test or command that shows it is met.

## Acceptance criteria

1. [behavior] Run cargo test: a named end-to-end test feeds a result event through the stream parser and the claude driver (fake script) or the invocation constructor. Its top-level usage differs from modelUsage (main thread plus subagent usage on the requested model, using 90/52656/3634696/134508 against 6/3136/275219/3821). It then runs check_invocation on the resulting record and asserts there is no Reconciliation violation.
2. [behavior] Run cargo test: a named negative test builds the record from a result event whose totals really disagree with modelUsage, through the same path. It asserts that check_invocation returns a Reconciliation violation.
3. [content] One definition of an invocation's measured tokens is stated in a doc comment (on ResultEvent.usage and AgentOutput.usage in lib.rs). The invocation record and the gate's reconcile rule both use it.
4. [content] The job 2fsa numbers (90/52656/3634696/134508 and 6/3136/275219/3821) appear in a regression test fixture or comment, and CHANGELOG.md records the fix.
5. [behavior] Run cargo test, cargo clippy --all-targets and cargo fmt --check: all pass and are clean, including the existing stream, claude and landing tests.

## Expected deliverables

- crates/athena-agent/src/stream.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding a pre-handover criterion-check mechanism to the job runner (already tried for this failure class).
- Editing records/ or migrating older records.
- Changing governance files such as athena.toml or defaults.toml.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Align the recorded tokens and the gate on one definition rather than loosening the check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require tests that run a result event through to check_invocation, not tests that only construct pieces.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy --all-targets and fmt --check must be clean.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the usage path, with the changelog.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 45a74b5328c8 → candidate d398a30ff4d1. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |  10 +++
 crates/athena-agent/src/claude.rs    | 119 ++++++++++++++++++++++-----------
 crates/athena-agent/src/lib.rs       |   5 ++
 crates/athena-agent/src/stream.rs    |  55 ++++++++++++++++
 crates/athena-core/src/invocation.rs | 123 ++++++++++++++++++++++++++++++++++-
 crates/athena-gate/src/reconcile.rs  |  56 ++++++++++++++--
 6 files changed, 321 insertions(+), 47 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (691 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
