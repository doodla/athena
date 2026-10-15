# Job 20261015-145809-7565: verify "Make invocation token totals agree with per-model usage and the gate"

## Request (verbatim)

> The gate refused Athena's own landing of verified job 20261015-131540-2fsa: 'usage records reconcile' says the producer used 90/52656/3634696/134508 tokens but the invocation records 6/3136/275219/3821. Athena's invocation records and the gate must agree on what an invocation used.

## Reading (accepted by the user)

The gate's reconciliation rule compares the requested model's modelUsage tokens with the invocation's tokens, which come from the result event's top-level usage. In job 2fsa those disagreed (90/52656/3634696/134508 vs 6/3136/275219/3821). I read the request as: fix the root cause so records Athena writes always reconcile, and have the gate check the same quantity. The audience is the Athena owner, who needs verified jobs to land.

Assumptions:
- Root cause: the result event's top-level usage covers less than modelUsage, probably the main thread only. modelUsage also includes subagent or other-turn usage. I inferred this from reading stream.rs and reconcile.rs. I did not see the failing record.
- The preferred fix is to make the recorded invocation tokens come from modelUsage when it is present, and to keep top-level usage as the fallback. The gate then compares the sum over all models. Loosening the gate alone would leave the records disagreeing internally.
- No governance file changes are needed.

## Acceptance criteria

1. [behavior] Run cargo test: a new test feeds a result event whose top-level usage differs from modelUsage (a main thread plus subagent usage on the requested model). It shows the resulting invocation record passes check_invocation with no Reconciliation violation.
2. [behavior] Run cargo test: a reconcile test shows that an invocation whose measured tokens genuinely disagree with its per-model usage is still flagged.
3. [content] The invocation's measured tokens and the gate's check use one definition. The definition is stated in a doc comment: either the sum of modelUsage over all models or the requested model's modelUsage. The gate compares like with like.
4. [behavior] Run cargo clippy --all-targets and cargo fmt --check: both are clean. Existing stream and claude tests still pass.
5. [content] The defect's numbers (90/52656/3634696/134508 against 6/3136/275219/3821) appear in a regression test fixture or comment, so the owner can see the job 2fsa failure is covered.

## Expected deliverables

- crates/athena-agent/src/stream.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)

## Out of scope

- Re-landing job 20261015-131540-2fsa or editing its records
- Changing the study or explainer content of that job
- Altering cost reconciliation or outcome-total reconciliation beyond what the token fix requires

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix how the invocation's tokens are derived and make the gate check the same quantity, instead of relaxing the gate for one job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add regression tests, with cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run a reconciliation over an event shaped like the failing one.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the agent stream parsing and the gate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base b97bc76adc11 → candidate 5ac0c3173627. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |  6 +++
 crates/athena-agent/src/stream.rs   | 33 +++++++++++++++
 crates/athena-gate/src/reconcile.rs | 84 ++++++++++++++++++++++++++++++-------
 3 files changed, 108 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (688 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
