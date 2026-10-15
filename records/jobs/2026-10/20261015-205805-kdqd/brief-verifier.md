# Job 20261015-205805-kdqd: verify "Let athena close abandon a job whose old usage records fail reconcile"

## Request (verbatim)

> Job 20261015-131540-2fsa is stuck: it was interrupted when the gate refused its landing, athena close is refused because its invocation record (written before the usage fix) fails 'usage records reconcile', and athena retry refuses a dispatched job. Give me a way to close or abandon a job like this.

## Reading (accepted by the user)

Job 20261015-131540-2fsa is dispatched and interrupted, and its pre-fix invocation record fails the gate's "usage records reconcile" rule. That blocks the Abandoned close commit in `Athena::close`, and `retry` refuses dispatched jobs. I read the goal as making `athena close <job> --why` finish such a job (the operator is the audience), without weakening the gate for new records.

Assumptions:
- The close commit fails because the gate re-checks the job's already-committed invocation record. I read manage.rs and reconcile.rs but did not run the failing case.
- The fix goes in `close`, not `retry`, because D52 and the design say `close` is the way out for interrupted jobs.
- Running `athena close` on the real job is an operator action after the change lands, and is not part of this change.
- The exemption should not loosen reconciliation for newly written records.

## Acceptance criteria

1. [behavior] A new test builds a dispatched, interrupted job whose committed invocation record has a per-model cost sum that disagrees with its reported total (the pre-fix shape). `athena close <job> --why ...` then succeeds and writes outcome-abandoned or outcome-failed, a score record and a commit. Run `cargo test`.
2. [behavior] A companion test shows the gate still rejects a new invocation record that fails reconciliation, and a new outcome whose totals do not match its invocations. Run `cargo test -p athena-gate`.
3. [content] The closing outcome's totals are computed from the job's actual invocation records, so the outcome itself reconciles. The legacy record is left unmodified (records stay append-only).
4. [content] The exemption is narrow and visible: it covers only records already committed before the close, or needs an explicit flag, and `athena close` prints which legacy records it let through.
5. [content] docs/decisions.md has a new decision entry with alternatives (flag vs automatic exemption vs rewriting records), and CHANGELOG.md mentions the change, so an operator can find how to close a stuck job.
6. [behavior] `cargo clippy` is clean and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-core/src/manage.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Rewriting or deleting the existing invocation record in history
- Making `athena retry` accept dispatched jobs
- Closing job 20261015-131540-2fsa itself
- Changing the study or explainer work that job was doing

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the close path and the still-enforced gate; cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check that closing a job with a legacy bad record succeeds.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix targets any job with pre-fix records, not just this one job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The gate remains the enforcement for new records, covered by a test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the close path and the gate scope.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and changelog entries are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 1c1504d93dce → candidate 503a4dc0e717. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |  12 ++++
 crates/athena-cli/src/main.rs        |  13 +++-
 crates/athena-cli/tests/lifecycle.rs | 117 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/commits.rs    |  40 ++++++++++--
 crates/athena-core/src/manage.rs     |  39 +++++++++++-
 crates/athena-gate/src/lib.rs        |  16 ++++-
 crates/athena-gate/src/reconcile.rs  |  95 ++++++++++++++++++++++++++++
 docs/decisions.md                    |  25 ++++++++
 docs/design/athena-v0.md             |   2 +-
 9 files changed, 346 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (709 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
