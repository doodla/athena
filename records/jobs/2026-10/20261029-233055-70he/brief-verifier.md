# Job 20261029-233055-70he: verify "Give OLY-9-12 a role-value and safe-retirement mechanism"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-9-12 a mechanism
> 
> Requirement OLY-9-12 in docs/spec.toml is active and has no mechanism.
> Requirement: Phase 6 is accepted when new roles and workflows show value against simpler alternatives and can be retired without losing institutional knowledge.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-9-12
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-9-12 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add crates/athena-core/src/role_value.rs, which judges a new role or workflow against a simpler alternative by reusing method_experiment::compare. It also allows retiring a role only if every piece of knowledge it holds is recorded elsewhere. Then record it as OLY-9-12's mechanism and measure in docs/spec.toml, following the standalone-module precedent of OLY-9-05.

Assumptions:
- Goal: the Athena maintainers and the Hestia spec review can check that new roles and workflows show value against simpler alternatives and can be retired safely. The spec shows OLY-9-12 with mechanism "none" and a placeholder measure.
- Code survey: read docs/spec.toml (OLY-9-01 to OLY-9-12), crates/athena-core/src/method_experiment.rs (Direction, MethodExperiment, compare, Outcome and Verdict, plus the refusals for an empty method, a missing arm and a zero bound), crates/athena-core/src/lib.rs (the `pub mod method_experiment` declaration), and the STATUS_MEASURE_LABELS list in crates/athena-core/src/status.rs. xtask/tests/spec.rs requires every active requirement to have a mechanism, an owner and a measure, and I read its mechanism tests by grep only. The new module is a pure library function and adds no record or data file, so check_record_file, AnyRecord and the round-trip test do not apply. Nothing in the module runs through an entry point or a probe order. The property is measured directly by unit tests on the verdict and retirement functions.
- Existing code: crates/athena-core/src/method_experiment.rs is reused for the comparison. The OLY-9-05 spec entry is the model for a mechanism that is a standalone module with a `cargo test -p athena-core <module>` measure.
- Following OLY-9-05, the module is not wired into `athena status`, so the measure uses `status: not shown` and names the test. No status label matches this requirement, and adding one is out of scope.
- This adds no refusal rule to the running pipeline; the refusals are inside the new library function only, so no replay refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows) are unchanged; the only config-like file edited is docs/spec.toml, which the request names.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-9-12 has a mechanism naming crates/athena-core/src/role_value.rs (not "none"), keeps owner Athena, and a measure ending in `status: not shown` that names `cargo test -p athena-core role_value`. Check with Grep on docs/spec.toml.
2. [content] crates/athena-core/src/role_value.rs exists, is declared in crates/athena-core/src/lib.rs, and its value judgment calls method_experiment::compare instead of re-implementing the comparison (Grep for the call).
3. [behavior] Run `cargo test -p athena-core role_value`. Tests show: a role that beats its simpler alternative is judged worth keeping; one no better or worse is judged not worth keeping; an empty role name, a missing arm or a zero bound is refused. These tests would fail if the module were stubbed out.
4. [behavior] Run `cargo test -p athena-core role_value`. Tests show retirement is allowed only when every item of the role's institutional knowledge is recorded elsewhere. Retirement is refused, naming the unrecorded items, when any item is missing, and the case where a role holds no knowledge items is covered explicitly.
5. [behavior] Run `cargo test -p xtask --test spec`. The tests that require every active requirement to have a mechanism, an owner and a status-marked measure pass with OLY-9-12 filled in.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All three pass.

## Expected deliverables

- crates/athena-core/src/role_value.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Wiring role_value into athena status, the CLI display or the drain
- Adding or retiring any actual role in crates/athena-data/src/roles.rs
- Other requirements with mechanism none (OLY-9-06, OLY-9-11)
- Prompt files and athena.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new module has unit tests, and cargo test, clippy and fmt are criteria.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing method_experiment comparison instead of building a new one.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module, one lib.rs line and one spec entry.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the role_value tests and check the verdicts and refusals.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7a7e412ee11c → candidate 6cdceed399a9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   5 ++
 crates/athena-core/src/lib.rs        |   1 +
 crates/athena-core/src/role_value.rs | 137 +++++++++++++++++++++++++++++++++++
 docs/spec.toml                       |   4 +-
 4 files changed, 145 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-233055-70he/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233055-70he/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43011686755 for 6cdceed399a9598bf56c3d3006be43ccc9947c39)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43011686755 for 6cdceed399a9598bf56c3d3006be43ccc9947c39)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43011686755 for 6cdceed399a9598bf56c3d3006be43ccc9947c39)
- ci-result: `not a check` passed (Check results for 6cdceed399a9598bf56c3d3006be43ccc9947c39 come from the CI run https://github.com/doodla/athena/actions/runs/43011686755 (821s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All three pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
