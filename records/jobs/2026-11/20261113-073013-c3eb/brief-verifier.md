# Job 20261113-073013-c3eb: verify "Add critique ledger mechanism for OLY-B-S6-01 (retry c6s3)"

## Request (verbatim)

> try again on 20261030-044539-c6s3

## Reading (accepted by the user)

Retry of rejected job 20261030-044539-c6s3: add docs/critique-ledger.toml with a strict validating parser in athena-core, a test, and a spec entry for OLY-B-S6-01. This time regenerate docs/design/strategy-view.md whenever the requirements-map row S6-1 changes, so the stale-view test passes.

Assumptions:
- Goal and audience: the verifier and spec-gap readers need OLY-B-S6-01 to stop being a gap, with a mechanism that a test proves.
- Previous job 20261030-044539-c6s3 is addressed by keeping its approach and fixing each finding: (1) blocker/minor test failure: strategy_view_table_matches_the_map failed because requirements-map.md row S6-1 changed without regenerating docs/design/strategy-view.md; fix is to run `athena strategy` (cargo run -p athena-cli -- strategy) and commit the regenerated view, which is now a deliverable; (2) major D43: cite only commands that ran, so run cargo test, clippy and fmt --check on the full workspace; (3) minor shard-timings.txt: xtask/shard-timings.txt is now an explicit deliverable, because the new test file must be registered there; (4) minor weak check_standard: kept as designed and disclosed, with the limit noted in CHANGELOG and the map, since the accepted reading allows it.
- Files from the earlier candidate: CHANGELOG.md, critique_ledger.rs, lib.rs, tests/critique_ledger.rs, docs/critique-ledger.toml, requirements-map.md and spec.toml are all kept as deliverables; xtask/shard-timings.txt is kept as a deliverable; docs/design/strategy-view.md is added.
- Code survey: read crates/athena-core/src/lib.rs (module registration) and docs/spec.toml lines ~3021 (OLY-B-S6-01 has mechanism "none", owner Athena). The new parser lives in crates/athena-core/src/critique_ledger.rs and is called from crates/athena-core/tests/critique_ledger.rs. The ledger is a docs/ TOML file, not a record file under crates/, so check_record_file and AnyRecord do not apply. The property is measured directly by parsing the real ledger and the real spec entry.
- The producer should check how the strategy test (crates/athena-cli strategy::tests::strategy_view_table_matches_the_map) derives counts, and regenerate rather than hand-edit.
- No new refusal rule applies to readings; the validator only governs the ledger file.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-S6-01 has a mechanism naming crates/athena-core/src/critique_ledger.rs and docs/critique-ledger.toml (not "none"), owner "Athena", and a measure ending `status: not shown` that names `cargo test -p athena-core --test critique_ledger`.
2. [content] docs/critique-ledger.toml has [[entry]] tables covering critique, improvement and open_question, each with id, kind, subject, status, text and source, and at least one entry with subject athena.
3. [content] crates/athena-core/src/lib.rs registers the critique_ledger module with a doc line; the parser rejects unknown keys, unknown kinds or statuses, blank text and duplicate ids.
4. [behavior] Run `cargo test -p athena-core --test critique_ledger`: it parses the real ledger and spec entry and passes; negative cases (duplicate id, bad kind, no athena entry, missing kind coverage) fail validation, so the test fails if the ledger or validator is removed.
5. [behavior] Run `cargo test -p athena-cli --lib strategy_view_table_matches_the_map`: it passes, because docs/design/strategy-view.md was regenerated with `athena strategy` and matches docs/design/requirements-map.md row S6-1.
6. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` for the workspace, each as its own command; each passes, or Athena's recorded same-tree pass (D115) is cited.
7. [content] xtask/shard-timings.txt lists the new critique_ledger test; CHANGELOG.md has an entry for this job; no existing entry of docs/decisions.md and nothing under records/ is edited.

## Expected deliverables

- docs/spec.toml (modify)
- docs/critique-ledger.toml (create)
- crates/athena-core/src/critique_ledger.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/critique_ledger.rs (create)
- CHANGELOG.md (modify)
- docs/design/requirements-map.md (modify)
- docs/design/strategy-view.md (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- Adding an `athena status` line for the ledger
- Changing athena.toml, defaults.toml or CI workflows
- Editing existing entries of docs/decisions.md or anything under records/
- Automatically generating ledger entries from reviews or runs
- Adding severity, impact or effort fields to the ledger

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Behavior criteria run the named tests and cite commands.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Workspace cargo test, clippy and fmt --check are criteria.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 69d427d289a1 → candidate 8daafaccaa45. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   1 +
 crates/athena-core/src/critique_ledger.rs   | 141 ++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs               |   4 +
 crates/athena-core/tests/critique_ledger.rs | 141 ++++++++++++++++++++++++++++
 docs/critique-ledger.toml                   |  30 ++++++
 docs/design/requirements-map.md             |   2 +-
 docs/design/strategy-view.md                |   9 +-
 docs/spec.toml                              |   4 +-
 xtask/shard-timings.txt                     |   1 +
 9 files changed, 325 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261113-073013-c3eb/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-073013-c3eb/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43390509283 for 8daafaccaa459364e42a55796491f96a8f774ff7)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43390509283 for 8daafaccaa459364e42a55796491f96a8f774ff7)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43390509283 for 8daafaccaa459364e42a55796491f96a8f774ff7)
- ci-result: `not a check` passed (Check results for 8daafaccaa459364e42a55796491f96a8f774ff7 come from the CI run https://github.com/doodla/athena/actions/runs/43390509283 (891s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` for the workspace, each as its own command; each passes, or Athena's recorded same-tree pass (D115) is cited.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
