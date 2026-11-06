# Job 20261105-180339-w5h7: verify "Raise agent ceiling to 16 and record plan-group sizes per batch"

## Request (verbatim)

> Raise the agent bound and record plan-group sizes (step 6). Build step from docs/design/agent-and-build-slots.md section 8; follow that section for the files touched and how the effect is measured.
> 
> Acceptance:
> - MAX_CONCURRENCY_CEILING allows the agent count the design gives, and athena.toml sets the design's starting value
> - each batch records the size of every plan group it worked
> - tests cover the new ceiling and the group-size record
> Depends on 20261105-180334-rj4j

## Reading (accepted by the user)

Step 6 of docs/design/agent-and-build-slots.md section 8 (depends on landed job 20261105-180334-rj4j, whose work is kept): raise MAX_CONCURRENCY_CEILING from 8 to 16, set athena.toml max_concurrency to the design's starting value of 12, and make each drain batch record the size of every plan group it worked and print it in the end-of-run lines. Audience: the maintainer reading drain output to judge whether group size, not the ceiling, caps agent use.

Assumptions:
- Code survey: crates/athena-data/src/config.rs holds MAX_CONCURRENCY_CEILING (line 269, now 8), used by the validation of [parallel] max_concurrency and max_in_flight (lines ~722, 745) and in its tests (lines ~1194, 1438-1466, which already derive from the constant). crates/athena-cli/src/drain.rs holds Report.plan_groups (planned groups, built at ~3060 from plan.groups) and drain_concurrent, which works one group at a time; crates/athena-cli/src/display.rs ~701 prints plan_groups in the end-of-run lines; crates/athena-core/src/plan.rs builds Plan.groups (line ~660). The group size is measured directly as the item count of each group the batch actually dispatched, not the planned groups.
- Job 20261105-180334-rj4j (step 5) landed and passed; its work is kept as landed and none of its files are re-changed except where this step needs them (config.rs and athena.toml are not among its out-of-scope, config.rs it touched for slot_max_gb). Its findings: disk.rs per-slot T, the cargo-internal-error delete rule, and the cleaning env nit are not fixed here; they are unrelated to the ceiling and group-size record and remain known gaps for a later job.
- The design (section 7.2 answer) gives 12 agent slots to start and 16 after memory is measured, and section 8 step 6 sets the ceiling from 8 to 16. So the athena.toml starting value is 12 and the ceiling is 16. The design's own wording says athena.toml is set by the maintainer after jobs 1 and 3, but the request explicitly asks for it, so it is set here.
- 'Batch' means one drain run (the Report). No records/ schema is added; the group-size record is a new Report field (e.g. group_sizes: Vec<usize>, in order of work) shown in the end-of-run lines, with the mean group size, so check_record_file/AnyRecord are not involved.
- The disk guard in disk.rs still holds jobs on low disk; the rollback (max_concurrency back to 2) is documented, not coded.

## Acceptance criteria

1. [content] crates/athena-data/src/config.rs declares MAX_CONCURRENCY_CEILING as 16, and its doc comments and the athena.toml comment ('the schema allows 1 to 8') say 1 to 16.
2. [content] athena.toml [parallel] max_concurrency is 12 (the design's section 7.2 starting value), with its comment updated; athena.toml is listed as a deliverable for this reason.
3. [behavior] Run `cargo test -p athena-data`: tests show max_concurrency = 16 and max_in_flight = 16 validate, and 17 (ceiling + 1) is refused with the 'from 1 to 16' error.
4. [behavior] Run `cargo test -p athena-cli --test drain`: a test drains a plan with groups of different sizes (e.g. two items disjoint in one group, one item alone in another) and asserts the Report's new group-size record lists the size of each group the batch worked, in order, and that groups not reached are not recorded.
5. [behavior] Run `cargo test -p athena-cli` display tests: the end-of-run lines print the group sizes and their mean (an exact expected string asserted), and print nothing misleading when no group was worked.
6. [content] docs/design/agent-and-build-slots.md step 6 gets a short Built note stating the ceiling, the starting value 12, what is recorded, and what was not measured (peak in-flight and the low-disk count are read after real drains); CHANGELOG.md has an entry.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass with no new warnings.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Disk.rs per-slot build time T, the cargo-internal-error slot delete rule and the cleaning env nit from job 20261105-180334-rj4j's findings
- Files changed by job 20261105-180334-rj4j (slots.rs, state.rs, run/checks.rs, run/mod.rs, run/tests.rs, athena-agent/src/env.rs): kept as landed
- Changing defaults.toml or [drain.pipeline] values
- Landing rebuild slot use (step 7) and producer cargo wrapper (step 4)
- Measuring peak in-flight jobs or low-disk counts from real drains
- Changing records/ or adding a record type

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus the athena.toml config value.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new ceiling and group-size tests plus fmt, clippy and test passing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests asserting recorded sizes and the refusal at 17, not that fields exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that group sizes show what caps agent use; the Built note says what stays unmeasured.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: a constant, a config value and one recorded field.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The ceiling is enforced by config validation, tested at 16 and 17.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 365a05bc8084 → candidate a0192713183f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  7 +++++++
 athena.toml                          |  7 ++++---
 crates/athena-cli/src/display.rs     | 40 ++++++++++++++++++++++++++++++++++++
 crates/athena-cli/src/drain.rs       | 16 +++++++++++++++
 crates/athena-cli/tests/drain.rs     | 38 ++++++++++++++++++++++++++++++++++
 crates/athena-data/src/config.rs     | 26 +++++++++++++++++++++--
 docs/design/agent-and-build-slots.md | 10 +++++++++
 7 files changed, 139 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-180339-w5h7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-180339-w5h7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42142292758 for a0192713183fcd2923ca1f09b8dc032ca07eaa1c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42142292758 for a0192713183fcd2923ca1f09b8dc032ca07eaa1c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42142292758 for a0192713183fcd2923ca1f09b8dc032ca07eaa1c)
- ci-result: `not a check` passed (Check results for a0192713183fcd2923ca1f09b8dc032ca07eaa1c come from the CI run https://github.com/doodla/athena/actions/runs/42142292758 (642s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass with no new warnings.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
