# Job 20261025-034237-2rep: verify "Make model and effort explicit per role; escalate hard producer jobs"

## Request (verbatim)

> Priority: high
> Manage and upgrade the intelligence used in planning, and make effort explicit (BRIEF.md addendum of 2026-10-25, 'Tell Athena to manage and upgrade intelligence in planning. Effort should also be explicit.'). Today the planner is always claude-sonnet-5-5 and the critic claude-opus-5-5 at the CLI's default effort, whatever the job; 35% of plans pass the first round and 16 of 44 accepted plans still failed or were rejected. Make the model and the effort level of every agent invocation explicit in config per role, recorded on each invocation record, and let the reading choose a stronger planner model or higher effort for a job it judges hard (for example one that changes landing, the history check or records), with the choice and its reason recorded. Measure the effect: per model and effort, the first-round accept rate, the landed share and the cost per landed job, shown in athena status. Absorbs request 20261013-042934-a1j8 (record each invocation's effort level).
> Acceptance:
> - Every invocation record names its model and effort, and a test shows both reach the agent driver.
> - A test shows a reading that marks a job hard gets the stronger planner setting, with its reason recorded, and an ordinary job keeps the default.
> - athena status shows the planning measures broken down by model and effort.
> Files: athena.toml, crates/athena-data/src/config.rs, crates/athena-data/src/records.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/run.rs, crates/athena-core/src/status.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Config names model and effort for the reader, producer and verifier. Both reach the claude driver and are recorded on every invocation record. A reading that judges a job hard picks a stronger producer model or higher effort, with the reason recorded. athena status shows first-pass verification rate, landed share and cost per landed job by producer model and effort. Audience: the Athena owner tuning quality. Planner and critic are dropped from scope because planning is off.

Assumptions:
- Planner and critic roles are out because job planning is off (f80b05a); only reader, producer and verifier are configured.
- The reading output gains a hard flag and reason field; the escalation picks the producer model or effort from a configured hard setting.
- Effort values are a closed set matching the CLI's levels; the exact list comes from the claude driver's supported flag.
- Existing records stay loadable via optional fields.

## Acceptance criteria

1. [behavior] A test (cargo test) shows the configured model and effort for each role reach the claude driver as CLI arguments and appear on that invocation's record.
2. [behavior] A test (cargo test) shows a reading marking a job hard gets the stronger producer model or effort with its reason recorded, while an ordinary reading keeps the configured default and records no escalation reason.
3. [behavior] A test (cargo test) shows athena status output lists first-pass verification rate, landed share and cost per landed job per producer model and effort. A bucket with no landed jobs shows no division by zero and no false success.
4. [content] athena.toml (listed as a deliverable) sets model and effort for reader, producer and verifier, plus the hard-job producer setting. config.rs parses them and rejects unknown effort values. Older records without effort still load.
5. [content] CHANGELOG.md has an entry that notes request 20261013-042934-a1j8 is absorbed. docs/decisions.md records the decision with alternatives, including dropping the planner/critic roles because planning is off.
6. [behavior] cargo test, cargo clippy and cargo fmt --check pass.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Planner and critic roles (planning is off)
- Re-enabling job planning
- Changing the claude_version pin

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for each behavior; cargo test and clippy are clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the driver arguments, the escalation and the status output, not just that fields exist.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Status handles buckets with zero landed jobs explicitly.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Config validation rejects unknown effort values.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision and changelog entries are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 6c0c5a3d67e7 → candidate 33701ecf5f13. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  12 ++
 athena.toml                             |  10 ++
 crates/athena-agent/src/claude.rs       |  33 ++++++
 crates/athena-agent/src/fake.rs         |   2 +
 crates/athena-agent/src/lib.rs          |   2 +
 crates/athena-cli/src/display.rs        |  66 ++++++++++-
 crates/athena-cli/tests/drain.rs        | 109 +++++++++++++++++-
 crates/athena-cli/tests/live.rs         |   1 +
 crates/athena-context/src/briefs.rs     |   2 +
 crates/athena-context/src/outputs.rs    |   5 +
 crates/athena-core/src/admission.rs     |   2 +
 crates/athena-core/src/introspect.rs    |   1 +
 crates/athena-core/src/invocation.rs    |  14 +++
 crates/athena-core/src/pr_check.rs      |   3 +
 crates/athena-core/src/reading.rs       |  14 ++-
 crates/athena-core/src/run.rs           |   6 +-
 crates/athena-core/src/status.rs        | 193 ++++++++++++++++++++++++++++++++
 crates/athena-data/src/config.rs        | 132 ++++++++++++++++++++++
 crates/athena-data/src/records.rs       |  31 +++++
 crates/athena-gate/src/landing_tests.rs |   3 +
 crates/athena-gate/src/reconcile.rs     |   1 +
 docs/decisions.md                       |  36 ++++++
 docs/records.md                         |   5 +
 23 files changed, 679 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1377 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy and cargo fmt --check pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
