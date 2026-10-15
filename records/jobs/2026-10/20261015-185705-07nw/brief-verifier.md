# Job 20261015-185705-07nw: verify "Make the negative end-to-end reconciliation test truly disagree"

## Request (verbatim)

> try again on 20261015-160432-g1tq

## Reading (accepted by the user)

Retry of 160432-g1tq. Fix the negative test a_result_whose_totals_disagree_with_model_usage_is_flagged_end_to_end so it builds the record, through the same parse and record path as the positive test, from an event whose totals really disagree. It then asserts check_invocation returns a Reconciliation violation. The audience is the Athena owner, who needs verified jobs to land.

Assumptions:
- The repository snapshot already holds the modelUsage definition (stream.rs, claude.rs output_from_result), a legacy-free check_invocation, and both end-to-end tests in crates/athena-core/src/invocation.rs. The remaining work is checking and tightening them, not re-implementing.
- Under the single modelUsage definition, the measured tokens come from the requested model's modelUsage entry. A through-path disagreement is therefore reachable only in the costs (total_cost_usd against the modelUsage costUSD sum), unless the parser falls back to top-level usage. The test and changelog say so.
- The negative test names the exact disagreement it builds and asserts that the input is bad before calling check_invocation. It also asserts that the violation is the sum comparison, not the 'no usage for the requested model' branch.
- Any hand-edited-record case is labelled a gate-only check and is not presented as the through-path case.
- The producer shows the negative test fails when the cost comparison in check_invocation is disabled. It reports this in its output, then restores the check, and the final diff leaves the check intact.

## Acceptance criteria

1. [behavior] Run `cargo test`: a_result_whose_totals_disagree_with_model_usage_is_flagged_end_to_end passes. It builds the record from a result event with modelUsage for the requested model and a total_cost_usd that differs from the per-model costUSD sum. It uses the same parse and record-building helper as the positive test. It asserts, before calling check_invocation, that the record's reported total differs from the modelUsage sum. It then asserts exactly one Reconciliation violation whose detail names the per-model cost sum.
2. [behavior] Run `cargo test`: the positive test a_result_whose_top_level_usage_includes_a_subagent_reconciles_end_to_end still passes. The job 2fsa top-level usage 90/52656/3634696/134508 differs from the modelUsage of 6/3136/275219/3821, the record carries the modelUsage tokens, and check_invocation returns no violations.
3. [content] The negative test has no case that edits a built record by hand and calls it end-to-end. Any such case is labelled a gate-only check. The doc comments on the test and the fixtures state that token disagreement cannot arise through the parser under the modelUsage definition.
4. [content] CHANGELOG.md records the fix and the modelUsage definition. It states honestly which disagreement the end-to-end negative test covers.
5. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`: all pass and are clean, including the existing stream, claude, gate and landing tests.

## Expected deliverables

- crates/athena-core/src/invocation.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-agent/src/stream.rs (modify)
- crates/athena-agent/src/claude.rs (modify)

## Out of scope

- Changing the reconciliation rule or the modelUsage definition beyond what the tests need.
- Changes to athena.toml, defaults.toml or workflows.
- A general mutation-testing framework for negative tests.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check that the disagreement is detected through the real parse and record path, not that a test exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The test input must violate the invariant it claims to violate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the test, plus the changelog if needed.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test to exercise the behavior.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Fixture comments explain why each event is built the way it is.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base fc2f53afdca3 → candidate 024503546ec5. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |  5 ++++-
 crates/athena-core/src/invocation.rs | 10 ++++++++--
 2 files changed, 12 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (706 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
