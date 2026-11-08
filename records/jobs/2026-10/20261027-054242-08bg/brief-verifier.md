# Job 20261027-054242-08bg: verify "Tell the reader which pipeline roles are enabled; target briefs to them"

## Request (verbatim)

> Priority: normal
> 
> Give the reader the list of currently enabled pipeline roles, taken from the effective config (for example [job_plan] enabled), and add a reading-brief rule that brief content targets only enabled roles. Acceptance: a test shows the reading brief names the enabled roles from the config.
> 
> Introspections that record the repeated cause: 20261026-202058-73tz, 20261026-210555-xpn0
> 
> Acceptance:
> - The change is made as described: Reading puts new content into planner/critic briefs although job planning is off in athena.toml; user redirects it to the producer by hand
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261026-202109-ckhx.json

## Reading (accepted by the user)

The reader brief gains an "Enabled roles" section built from the effective config (reader, producer, verifier always; planner and critic only when [job_plan] enabled is true), plus a tenth reading rule that brief content targets only enabled roles. Goal: stop readings from putting new content into planner/critic briefs when job planning is off; the audience is the reader agent and the maintainers who redirect it by hand.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (ReaderInputs at ~2214, READING_RULES at ~2247 as a [&str; 9], reader_brief at ~2276, tests near ~4511-4590 that assert READING_RULES.len() == 9) and crates/athena-core/src/reading.rs (read_once builds ReaderInputs at ~567 from jc.config) and crates/athena-data/src/config.rs (JobPlanConfig.enabled, default false; job_plan is an Option). Entry point: read_once -> reader_brief. The role list is computed from jc.config.job_plan in read_once and passed as a new ReaderInputs field. No record or data file is added under crates/, so the records allow-list is not touched.
- Enabled roles: reader, producer and verifier always run; planner and critic are listed only when config.job_plan is Some and enabled is true. 'hard' mode still counts as enabled; the section may note the mode. The effective config is the already-merged jc.config.
- The new rule is prose only (no check_* refusal): it adds no refusal, so no refused-readings percentage applies. It is added to READING_RULES with the count constant and existing count tests updated.
- The reader prompt token budget (3 left) is unaffected: changes are in Rust brief text rendered at run time, not in the prompt files.
- athena.toml and defaults.toml are not changed; only read for the current [job_plan] enabled = true.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-context/src/briefs.rs builds a reader brief with job planning disabled and asserts the brief names reader, producer and verifier as enabled roles and does not list planner or critic as enabled; run `cargo test -p athena-context`.
2. [behavior] A second test builds the brief with job planning enabled and asserts planner and critic appear in the enabled-roles section; run `cargo test -p athena-context`.
3. [behavior] A test in crates/athena-core (e.g. reading.rs tests) shows read_once/reader input construction derives the role list from the config's [job_plan] enabled value, not a hard-coded list; run `cargo test -p athena-core`.
4. [content] crates/athena-context/src/briefs.rs READING_RULES contains a new rule stating that brief content targets only enabled roles, marked `(prose only)`, and the array length and existing length assertions are updated consistently.
5. [content] reader_brief output has a heading section listing the currently enabled roles, checkable by Grep of the heading constant in briefs.rs.
6. [behavior] No unrelated behavior changes: `cargo test` passes with all existing tests, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)

## Out of scope

- Changing planner, critic, producer or verifier briefs or prompts
- Making the rule a refusal check (check_*) in athena-core
- Changing athena.toml, defaults.toml or the [job_plan] settings
- Editing records/ failure-class files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the brief output names the enabled roles, not just that fields exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the brief builder and its caller.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Gives the reader the config fact so it stops guessing roles, rather than patching one reading.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule marked prose only; the role list itself is derived from config by code.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered brief for enabled and disabled roles.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 255feaad5a7c → candidate 2db4f70c92d2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  8 ++++++
 crates/athena-context/src/briefs.rs | 54 ++++++++++++++++++++++++++++++++++---
 crates/athena-core/src/reading.rs   | 15 +++++++++++
 crates/athena-data/src/config.rs    | 28 +++++++++++++++++++
 4 files changed, 101 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054242-08bg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054242-08bg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42500466031 for 2db4f70c92d2bb34ddbe0cb4d36b6a6d287a8348)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42500466031 for 2db4f70c92d2bb34ddbe0cb4d36b6a6d287a8348)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42500466031 for 2db4f70c92d2bb34ddbe0cb4d36b6a6d287a8348)
- ci-result: `not a check` passed (Check results for 2db4f70c92d2bb34ddbe0cb4d36b6a6d287a8348 come from the CI run https://github.com/doodla/athena/actions/runs/42500466031 (536s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No unrelated behavior changes: `cargo test` passes with all existing tests, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
