# Job 20261112-180947-td6x: verify "Give OLY-B-NOTE-01 a specialization mechanism, measure and tests"

## Request (verbatim)

> try again on 20261030-013725-ap0q

## Reading (accepted by the user)

Retry of job 20261030-013725-ap0q: record the existing `specialization` status figure as the mechanism for OLY-B-NOTE-01 in docs/spec.toml, keep its snapshot print, and add tests for both `athena status` and `status --snapshot`. Audience: the owner reading `athena status`. The earlier candidate is kept (CHANGELOG.md, display.rs, routing_evidence.rs, snapshot.rs, spec.toml, shard-timings.txt), with the snapshot test gap fixed and real-records evidence cited from commands that ran.

Assumptions:
- Addresses job 20261030-013725-ap0q (rejected): its major finding (behavior criterion 3 evidence cites a command not seen running) is fixed by a criterion the verifier runs itself on real records against independent counts; its minor finding (snapshot path has no test) is fixed by a test calling display::snapshot_status; the out-of-scope-files finding is fixed by listing snapshot.rs and xtask/shard-timings.txt as deliverables (candidate work kept); the nit about 'unrecorded' effort counting as a setting is left alone, it predates the change.
- Code survey: read crates/athena-core/src/status/planning.rs (`specialization` figure, SPECIALIZATION_MIN_JOBS), crates/athena-cli/src/display.rs (`status` and `snapshot_status` print the block), crates/athena-core/src/snapshot.rs (SnapshotView, specialization field added by the earlier candidate). The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are untouched. The figure is measured directly from producer invocations per class of work, not through a stand-in. The records validation test reads docs/spec.toml and must still pass.
- Existing code: crates/athena-core/src/status/planning.rs (`specialization`), crates/athena-cli/src/display.rs, crates/athena-core/src/snapshot.rs, and docs/spec.toml are reused or extended, not rebuilt.
- Environment: the real-records check runs `cargo run -p athena-cli -- status --snapshot <evidence directory>` read-only (no writes, no network) over the evidence snapshot holding history.json and the recorded invocations; the independent counts come from those files, which the change does not compute.
- Goal and audience: requirement OLY-B-NOTE-01 is shown to the owner through the `specialization` heading in `athena status`; the measure ends `status: specialization`.
- Governance: athena.toml, defaults.toml and .github/workflows are not changed. xtask/shard-timings.txt is a data file the ci_shards test needs for the new test file, not a governance file.
- Shared files: shard-timings.txt gains a line for routing_evidence.rs only if ci_shards requires it.

## Acceptance criteria

1. [content] In docs/spec.toml, requirement OLY-B-NOTE-01 has a mechanism that is no longer "none" and names crates/athena-core/src/status/planning.rs (`specialization`) and crates/athena-cli/src/display.rs, both found by Grep; owner stays Athena; measure says how it is shown and ends with `status: specialization`, a heading display.rs prints.
2. [behavior] Run `cargo test`: crates/athena-cli/tests/routing_evidence.rs builds fixture records where a non-baseline model/effort setting with at least SPECIALIZATION_MIN_JOBS jobs has a higher landed share, and asserts `display::status` prints the `specialization` line flagging it as helping; a second case with fewer jobs asserts no flag. Removing the figure or its display makes it fail.
3. [behavior] Run `cargo test`: a test in routing_evidence.rs calls `display::snapshot_status` on a fixture snapshot and asserts the `specialization` block appears; it fails if the snapshot print line in display.rs or the SnapshotView field in snapshot.rs is removed.
4. [behavior] Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records: the `specialization` block lists each class of work and its per-setting job counts sum to the expected range, which is the number of dispatched request jobs with a producer invocation counted from history.json and the recorded invocations (a source the change does not compute), within 0 difference. Every class flagged as helping has at least 3 jobs in that setting.
5. [behavior] Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` before and after on the real records: the landing time, throughput, keep rule and planning cost lines are unchanged; the landing time is within the expected range of a few minutes taken from commit times in history.json.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including existing planning, display, records validation and xtask ci_shards tests.
7. [content] CHANGELOG.md has an entry for the OLY-B-NOTE-01 mechanism and the snapshot print; the diff changes only the listed deliverables.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/snapshot.rs (modify)
- crates/athena-cli/tests/routing_evidence.rs (create)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)
- crates/athena-core/src/status/planning.rs (modify)

## Out of scope

- Automatically changing the configured model or effort per role from the evidence.
- Changing athena.toml, defaults.toml or CI workflows.
- Changing landing time, throughput, keep rule or planning cost figures (checked by the before/after real-records criterion).
- Excluding 'unrecorded' effort from the flag or adding a margin to the helping rule.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Real-records run compared with independent counts from history.json.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion plus new tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the status command and tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 989cc0578fd6 → candidate a5c7e6f6725e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   1 +
 crates/athena-cli/src/display.rs            |   1 +
 crates/athena-cli/tests/routing_evidence.rs | 188 ++++++++++++++++++++++++++++
 crates/athena-core/src/snapshot.rs          |   5 +-
 docs/spec.toml                              |   4 +-
 xtask/shard-timings.txt                     |   1 +
 6 files changed, 197 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261112-180947-td6x/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-180947-td6x/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43282822612 for a5c7e6f6725e2ae381f97cd1485f3880c1fe5241)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43282822612 for a5c7e6f6725e2ae381f97cd1485f3880c1fe5241)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43282822612 for a5c7e6f6725e2ae381f97cd1485f3880c1fe5241)
- ci-result: `not a check` passed (Check results for a5c7e6f6725e2ae381f97cd1485f3880c1fe5241 come from the CI run https://github.com/doodla/athena/actions/runs/43282822612 (656s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "Run `cargo test`: crates/athena-cli/tests/routing_evidence.rs builds fixture records where a non-baseline model/effort setting with at least SPECIALIZATION_MIN_JOBS jobs has a higher landed share, and asserts `display::status` prints the `specialization` line flagging it as helping; a second case with fewer jobs asserts no flag. Removing the figure or its display makes it fail."; "Run `cargo test`: a test in routing_evidence.rs calls `display::snapshot_status` on a fixture snapshot and asserts the `specialization` block appears; it fails if the snapshot print line in display.rs or the SnapshotView field in snapshot.rs is removed."; "Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records: the `specialization` block lists each class of work and its per-setting job counts sum to the expected range, which is the number of dispatched request jobs with a producer invocation counted from history.json and the recorded invocations (a source the change does not compute), within 0 difference. Every class flagged as helping has at least 3 jobs in that setting."; "Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` before and after on the real records: the landing time, throughput, keep rule and planning cost lines are unchanged; the landing time is within the expected range of a few minutes taken from commit times in history.json."; "Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including existing planning, display, records validation and xtask ci_shards tests."; "CHANGELOG.md has an entry for the OLY-B-NOTE-01 mechanism and the snapshot print; the diff changes only the listed deliverables.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
