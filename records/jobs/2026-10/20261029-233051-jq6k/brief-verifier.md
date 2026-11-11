# Job 20261029-233051-jq6k: verify "Give OLY-9-11 a role-need mechanism shown in athena status"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-9-11 a mechanism
> 
> Requirement OLY-9-11 in docs/spec.toml is active and has no mechanism.
> Requirement: Phase 6 extends specialization, delegation and role evolution based on observed need.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-9-11
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-9-11 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a role-need measure in athena-core, computed from records and printed by `athena status` as a new `role need` line, so Phase 6 role evolution (add or retire roles) is driven by observed need. Record it as OLY-9-11's mechanism and measure in docs/spec.toml, with a test that fails without it. Audience: Athena owner and the Hestia spec pass.

Assumptions:
- Code survey: read crates/athena-core/src/class_trend.rs (the OLY-9-10 model: analyze, ClassTrend::compute, lines), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at ~305, StatusView fields, compute calls at ~1170 and ~1278), crates/athena-core/src/status/planning.rs (specialization at 488), crates/athena-core/src/learning.rs, crates/athena-cli/src/display.rs (trend_section ~1103, call ~1330), xtask/tests/spec.rs (spec validator requiring mechanism, owner, and a measure ending `status: <label>`).
- Existing code: extend crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView), reuse specialization in crates/athena-core/src/status/planning.rs and the class_trend.rs pattern; print via crates/athena-cli/src/display.rs; spec entry in docs/spec.toml.
- Design: a new module crates/athena-core/src/role_need.rs with a pure analyze function over per-class and per-role job counts and outcomes from Records. It reports a class of work with enough jobs and a repeated failure or slow median as a role-needed candidate, and a role with no invocations over a window as a retire candidate. It reports nothing below a minimum job count, like MIN_CLASS_JOBS. Candidates are advisory only; the mechanism does not create or retire roles.
- The new label is `role need`, added to STATUS_MEASURE_LABELS mapped to a new StatusView field, and printed at the start of a line by athena status. The record-file allow-list (check_record_file, AnyRecord) is not touched because no new record or data file is added; the measure is computed from existing records.
- The measure is read directly from records, not through a stand-in. OLY-9-12 is left untouched.
- Adds no refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-9-11 has a mechanism naming an existing path (crates/athena-core/src/role_need.rs), owner Athena, and a measure ending `status: role need`; the `role need` label is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
2. [behavior] Unit tests in crates/athena-core/src/role_need.rs fail when analyze is stubbed to return None and pass with it: a class with repeated failures above the minimum job count yields a role-needed candidate, an unused role yields a retire candidate, and a thin history yields nothing. Run `cargo test -p athena-core role_need`.
3. [behavior] A display test in crates/athena-cli/src/display.rs shows that StatusView with role-need candidates prints a line starting `role need`, and prints no such line when there is no candidate. Run `cargo test -p athena-cli`.
4. [behavior] `cargo test -p xtask` (the spec test in xtask/tests/spec.rs) passes with the updated OLY-9-11 entry; `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.
5. [content] The diff changes only docs/spec.toml, crates/athena-core/src/role_need.rs, lib.rs, status.rs, crates/athena-cli/src/display.rs, and CHANGELOG.md if the repo convention requires it; no governance files (athena.toml, defaults.toml) change.

## Expected deliverables

- crates/athena-core/src/role_need.rs (create)
- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- OLY-9-12 (value of a new role against a simpler alternative)
- Automatically creating or retiring roles
- Changes to prompts, athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and display tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check printed output and candidate results, not that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module following the class_trend pattern.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f6a923219f49 → candidate b005a4a2f196. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   4 +
 crates/athena-cli/src/display.rs    |  37 ++++++
 crates/athena-core/src/lib.rs       |   1 +
 crates/athena-core/src/role_need.rs | 219 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs    |   6 +
 docs/spec.toml                      |   4 +-
 6 files changed, 269 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-233051-jq6k/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233051-jq6k/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43004622903 for b005a4a2f196d8816e5f0191419023d1bde49b3e)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43004622903 for b005a4a2f196d8816e5f0191419023d1bde49b3e)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43004622903 for b005a4a2f196d8816e5f0191419023d1bde49b3e)
- ci-result: `not a check` passed (Check results for b005a4a2f196d8816e5f0191419023d1bde49b3e come from the CI run https://github.com/doodla/athena/actions/runs/43004622903 (689s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
