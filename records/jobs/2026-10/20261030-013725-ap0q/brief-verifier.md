# Job 20261030-013725-ap0q: verify "Give OLY-B-NOTE-01 the status specialization mechanism and a test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-NOTE-01 a mechanism
> 
> Requirement OLY-B-NOTE-01 in docs/spec.toml is active and has no mechanism.
> Requirement: The plausible starting model routing is revised from evidence.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-NOTE-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-NOTE-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Reads OLY-B-NOTE-01 (revise the starting model routing from evidence) as already served by the `specialization` figure, which compares producer model and effort per class of work from records and prints in `athena status`. The job records it as the mechanism in docs/spec.toml, with a measure ending `status: specialization`, and adds a test that fails when the figure or its line is missing. Audience: the owner reading `athena status`.

Assumptions:
- Goal: show that Athena revises the BRIEF.md startup model routing from recorded evidence. Audience: the owner who reads `athena status` and the verifier who runs the tests.
- Code survey: read crates/athena-core/src/status/planning.rs (`specialization`, `settings_of`, `SpecializationVerdict`, `SPECIALIZATION_MIN_JOBS`), crates/athena-cli/src/display.rs (the specialization block near line 192, `producer_settings_lines` near line 158, called near line 1334), crates/athena-core/src/routing.rs (CI-check routing, unrelated to model routing). The entry point is `athena status`. The change adds no record or data file under crates/, so `check_record_file`, `AnyRecord` and the round-trip test are not touched. The property is measured directly from the producer invocation records of request jobs, not through a stand-in.
- Existing code: crates/athena-core/src/status/planning.rs `specialization` and crates/athena-cli/src/display.rs specialization lines are reused and extended, not rebuilt; the spec.toml mechanism cites them.
- Environment: no CI workflow, push or PR is touched. The real-records check reads the local evidence snapshot or records through `athena status --snapshot`, read-only, reaching history.json and the job records. No `git log` read is needed.
- Governance: athena.toml, defaults.toml and .github/workflows are not changed; the outcome needs none of them.
- The requirement is read as met by reporting evidence per class of work and flagging a better setting. It is not read as automatic rerouting of roles, which would change config and is out of scope.
- If a gap exists (for example the specialization block is missing a line the measure cites), the producer makes the smallest display change so the `status: specialization` label is a line `athena status` prints.

## Acceptance criteria

1. [content] In docs/spec.toml, requirement OLY-B-NOTE-01 has a mechanism that is no longer "none" and names crates/athena-core/src/status/planning.rs (`specialization`) and crates/athena-cli/src/display.rs, both of which exist (Grep for the names); its owner stays Athena; its measure says how it is shown and ends with `status: specialization`, a heading line that display.rs prints in `athena status`.
2. [behavior] Run `cargo test`: a new test (for example in crates/athena-cli/tests/routing_evidence.rs) builds fixture records where a non-baseline producer model/effort has at least SPECIALIZATION_MIN_JOBS jobs and a higher landed share, and asserts the status text prints the `specialization` line with that setting flagged as helping. The test fails if the specialization call or its display is removed, and passes with it.
3. [behavior] Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records: the `specialization` block lists each class of work, and its per-setting job counts sum to the number of dispatched request jobs with a producer invocation. The expected range comes from counting those jobs in history.json and the recorded invocations, a source the change does not compute. Any class flagged as helping has at least 3 jobs in that setting.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including the existing planning and display tests and the records validation tests.
5. [content] The figure modules keep their present behavior: the diff to crates/athena-core/src/status/planning.rs, if any, adds no change to the landing time, throughput, keep rule or planning cost figures. The deliverables are only the files listed.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/status/planning.rs (modify)
- crates/athena-cli/tests/routing_evidence.rs (create)
- CHANGELOG.md (modify)

## Out of scope

- Automatically changing the configured model or effort per role from the evidence.
- Changing athena.toml, defaults.toml or CI workflows.
- Changing landing time, throughput, keep rule or planning cost figures.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Added a test that fails without the mechanism; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the printed status line and the flagged setting, not that files exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion compares the printed counts on real records with counts taken from history.json.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: a spec entry, a test and at most a small display fix.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4c7d1c4c6bd4 → candidate 68c981d62d43. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   1 +
 crates/athena-cli/src/display.rs            |   1 +
 crates/athena-cli/tests/routing_evidence.rs | 169 ++++++++++++++++++++++++++++
 crates/athena-core/src/snapshot.rs          |   3 +
 docs/spec.toml                              |   4 +-
 xtask/shard-timings.txt                     |   1 +
 6 files changed, 177 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-013725-ap0q/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013725-ap0q/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43274005227 for 68c981d62d43dcf69fc8424789c94927c00325b0)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43274005227 for 68c981d62d43dcf69fc8424789c94927c00325b0)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43274005227 for 68c981d62d43dcf69fc8424789c94927c00325b0)
- ci-result: `not a check` passed (Check results for 68c981d62d43dcf69fc8424789c94927c00325b0 come from the CI run https://github.com/doodla/athena/actions/runs/43274005227 (768s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including the existing planning and display tests and the records validation tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
