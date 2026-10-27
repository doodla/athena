# Job 20261027-054151-ace0: verify "Carry prior result and change scope into retry readings; refuse"

## Request (verbatim)

> Priority: urgent
> 
> Make the reading stage's input include the previous attempt's result and the allowed change scope for a retried or follow-up job. Then add a check that refuses a reading that leaves either one out. Check for overlap with the queued item reading-omits-prior-verifier-finding-or-end-state.
> 
> Introspections that record the repeated cause: 20261016-060515-0801, 20261024-020022-z573, 20261024-115134-0pna, 20261024-154638-19r5, 20261024-161644-yhq2, 20261024-170310-67x6, 20261025-022225-ffsv
> 
> Acceptance:
> - The change the request describes is made, with the root cause named
> - A test fails without the change and passes with it
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

For a retried or follow-up job, the reader's brief must always state the previous attempt's result (verdict, findings, end state) and the allowed change scope (deliverables and out-of-scope). A new check then refuses any reading that omits either. Audience: the reader agent and the verifier. Root cause: the brief carries these only partly and nothing enforces the reading to use them.

Assumptions:
- Existing machinery (EarlierAttempt in athena-context briefs.rs, earlier_attempt/referenced_jobs in athena-core reading.rs) already carries findings, reading criteria and deliverables; the gap is the end state and out-of-scope (change scope) for retries and follow-ups, and that no check refuses a reading that ignores them. I did not see the actual introspection records, which are absent.
- The queued item reading-omits-prior-verifier-finding-or-end-state is in records/, which I cannot read. I assume it overlaps on the prior finding and end-state half. This job covers it, adds the scope half and the refusal check, and names the overlap in its summary and the changelog; it does not edit the queue.
- The check lives where a reading is accepted after read_for in athena-core and fails the reading with a clear message, so the reader is asked again within the existing revision loop.
- A reading 'leaves out' the prior result if it neither addresses the earlier findings or end state in its summary, assumptions or criteria, nor states why they no longer apply. A deterministic check can test this by comparing the reading's text and criteria against the carried findings, deliverables and out-of-scope.

## Acceptance criteria

1. [content] The reader brief for a retry (try again on <job>) and for a follow-up job that names an earlier job includes a section with the previous attempt's result (verdict, findings or outcome reason, end state) and the allowed change scope (deliverable paths and out-of-scope items), with the scope present even if the earlier reading had none.
2. [content] A new check function in athena-core refuses a reading for a retried or follow-up job that does not reflect the prior result or does not state the change scope, returning an error that names which of the two is missing. It does not apply to a first-attempt job.
3. [content] The change summary or CHANGELOG states the root cause (the brief and the acceptance of a reading did not require the prior result and scope) and states the overlap with reading-omits-prior-verifier-finding-or-end-state: what it covers and what is left.
4. [behavior] New unit tests in crates/athena-context and crates/athena-core fail on the pre-change code and pass after. They cover the brief containing the result and scope, a reading missing the result refused, a reading missing the scope refused, a complete reading accepted, and a first attempt unaffected. Run `cargo test` and compare with the base.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root with no new warnings.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Editing or closing the queued item reading-omits-prior-verifier-finding-or-end-state (records/ is not changed)
- Changing the verifier, producer or landing behavior
- Changing athena.toml, defaults.toml or workflows
- Rewriting the introspection records

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New failing-then-passing tests plus cargo test and clippy criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The criteria require naming the root cause and adding a mechanism check, not only prompt text.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A code check enforces the rule instead of prose only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The tests exercise the brief output and the refusal.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that a reading omitting the result or scope is refused.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the reading and brief code.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base eb550629642f → candidate a66b822a0fd3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  20 +++
 crates/athena-agent/src/fake.rs       |  24 ++-
 crates/athena-cli/tests/common/mod.rs |  20 ++-
 crates/athena-cli/tests/retry.rs      |  81 ++++++++++
 crates/athena-context/src/briefs.rs   | 143 ++++++++++++++++++
 crates/athena-core/src/reading.rs     | 273 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/scope_guard.rs |  10 +-
 7 files changed, 564 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1454 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root with no new warnings.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
