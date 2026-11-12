# Job 20261112-081144-cntr: verify "Give OLY-B-O5-01 a mechanism with an accepted restarts status label"

## Request (verbatim)

> try again on 20261030-013416-weaa

## Reading (accepted by the user)

Retry of rejected job 20261030-013416-weaa: record `athena restart` and the `restarts` line of `athena status` as the mechanism for OLY-B-O5-01, this time adding ("restarts", "restarts") to STATUS_MEASURE_LABELS so the measure ending `status: restarts` passes the spec checker. A new test fails on the base tree (label rejected, requirement still a gap) and passes after. Audience: the owner reading the spec gap list.

Assumptions:
- Prior job 20261030-013416-weaa (rejected) is addressed finding by finding: (a) the label `restarts` was rejected by the spec checker -> fixed by adding the entry to STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs (line 305, which today lists 16 labels and no `restarts`); (b) its test passed on the base tree -> replaced by a test in status.rs that fails on the base; (c) CHANGELOG claimed a rejected measure -> entry rewritten after the real-spec test passes; (d) behavior evidence not tied to commands that ran -> criteria name the exact cargo commands.
- Kept from that candidate: docs/spec.toml and CHANGELOG.md edits (redone). Dropped: its crates/athena-cli/tests/lifecycle.rs edit, because a base lifecycle test (around line 2940-2963) already records a restart via athena.restart and asserts status.restarts == 1; adding another proves nothing new. Its planned crates/athena-cli/tests/restart.rs does not exist at the base and is not created; crates/athena-core/src/manage.rs is not changed either.
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at 305; `pub restarts: usize` at 475; `restarts: records.restarts.len()` at 1322; existing forecasts-requirement test at ~1630-1656 and `every_measure_label_names_a_status_view_field_and_a_printed_line` at 1659, which requires a StatusView field `restarts` and a printed string starting `"restarts `), crates/athena-cli/src/display.rs:1411 (`restarts    {}` line), crates/athena-core/src/hestia.rs (measure_problem ~1219-1230 and spec_gaps, which reject unlisted labels), crates/athena-core/src/store.rs (Restart records load into records.restarts), crates/athena-cli/tests/lifecycle.rs (existing restart/status test). No new record or data file is added, so check_record_file/AnyRecord need no change. The property is measured directly by spec_gaps on the real docs/spec.toml requirement, not a stand-in.
- Existing code: docs/spec.toml (requirement OLY-B-O5-01 at line 2265, mechanism currently "none", measure "The spec gap list; status: requirement"), crates/athena-core/src/status.rs, crates/athena-core/src/hestia.rs, crates/athena-cli/src/display.rs, crates/athena-core/src/manage.rs (`restart`) are reused; only the label table, spec entry, a test and the changelog change.
- Consumers: none found beyond those named. Governance files athena.toml, defaults.toml and workflows are not changed.
- Spec mechanism text names crates/athena-core/src/manage.rs (`Manager::restart`, the `athena restart` command) and crates/athena-core/src/status.rs plus crates/athena-cli/src/display.rs (the `restarts` line); the mechanism records state of the work coming from Athena. Per-milestone or efficiency-drop enforcement stays prose-only and the measure says so.

## Acceptance criteria

1. [content] docs/spec.toml OLY-B-O5-01 has a mechanism naming crates/athena-core/src/manage.rs and the `restarts` line of athena status (status.rs, display.rs) instead of "none", owner stays Athena, and the measure explains how the requirement is shown and ends with `status: restarts`. Check with Grep; every named path exists (ls).
2. [content] crates/athena-core/src/status.rs STATUS_MEASURE_LABELS contains ("restarts", "restarts"); the `restarts    ` line in crates/athena-cli/src/display.rs matches the label. Check with Grep.
3. [behavior] Run `cargo test -p athena-core --lib status` and `cargo test -p athena-core --lib hestia`: a new status.rs test loads the real docs/spec.toml requirement OLY-B-O5-01, asserts measure_problem is None, the measure ends `status: restarts`, and spec_gaps no longer lists it; with mechanism reset to "none" it is a gap again. The test fails on the base tree (label rejected) and passes now; the existing `the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed` and `every_measure_label_names_a_status_view_field_and_a_printed_line` tests pass.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`; all pass, including the xtask spec test.
5. [content] CHANGELOG.md has an entry stating the mechanism and measure `status: restarts` and the new label; the diff touches only docs/spec.toml, crates/athena-core/src/status.rs and CHANGELOG.md (the deliverables); status/planning.rs, manage.rs and lifecycle.rs are unchanged.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Enforcing a restart per milestone or on an efficiency drop with a new gate or refusal
- Changing crates/athena-core/src/manage.rs, crates/athena-core/src/status/planning.rs, or crates/athena-cli/tests/lifecycle.rs (dropped from the prior candidate)
- Creating crates/athena-cli/tests/restart.rs (does not exist at the base; not added)
- Changing athena.toml, defaults.toml or workflows
- Editing records/ or other requirements in docs/spec.toml

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the spec gap list on the real spec, not that wiring exists.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A new test fails on the base tree and names the cargo commands to run.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and cargo test criterion.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Grep the OLY-B-O5-01 block (base line 2265) and list the three paths named in the mechanism. Run: `Grep 'status: restarts' and 'manage.rs' in docs/spec.toml with -A 4 after 'id = "OLY-B-O5-01"'; ls crates/athena-core/src/manage.rs crates/athena-core/src/status.rs crates/athena-cli/src/display.rs`. Observes: mechanism is no longer "none" and names crates/athena-core/src/manage.rs, status.rs and display.rs; owner = "Athena"; the measure explains how the requirement is shown and ends `status: restarts`; ls lists all three files.
- Criterion 2: Grep the label table and the printed line. Run: `Grep '("restarts", "restarts")' in crates/athena-core/src/status.rs; Grep 'restarts    ' in crates/athena-cli/src/display.rs`. Observes: One match in STATUS_MEASURE_LABELS after the forecasts entry; display.rs:1411 prints `restarts    {}`, beginning with the label text.
- Criterion 3: Run the new unit test oly_b_o5_01_is_met_by_athena_restart_and_shown_by_the_restarts_line and the neighbouring spec tests on the real docs/spec.toml. The test (a) asserts STATUS_MEASURE_LABELS contains ("restarts","restarts") and crate::hestia::measure_problem("Restarts counted; status: restarts") is None; (b) parses include_str!("../../../docs/spec.toml") with parse_spec, finds OLY-B-O5-01, asserts mechanism != "none" and contains manage.rs, status.rs and display.rs, owner == "Athena", measure.ends_with("status: restarts"), measure_problem(&req.measure) is None, and spec_gaps has no entry with that id; (c) clones the requirement with mechanism "none" and asserts spec_gaps returns one gap with no_mechanism. Run: `cargo test -p athena-core --lib status && cargo test -p athena-core --lib hestia`. Observes: All tests pass, including the new one, every_measure_label_names_a_status_view_field_and_a_printed_line (status.rs:1659), and in the hestia run the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed (hestia.rs, about line 4286), which runs measure_problem over every active requirement.
  Fails without the behavior: On the base tree STATUS_MEASURE_LABELS (status.rs:305-322) has no restarts row, so assertion (a) fails and measure_problem returns Some("the measure names `restarts`, which `athena status` does not print"); the spec entry also has mechanism "none" and measure `status: requirement`, so (b) fails and the requirement is in the gap list. Removing only the table row after the change makes the test fail again.
  Test seam: Production entry points crate::hestia::parse_spec, measure_problem and spec_gaps, called on the real docs/spec.toml via include_str!, as the sibling test does (status.rs:1632-1656). No fake or fixture. The only constructed input is the cloned requirement with mechanism "none" in part (c), which is passed to the real spec_gaps. The seam exists today; plan step 4 adds the test.
- Criterion 4: Run the three checks Athena runs on the result. The `cargo test --locked` run contains two tests that read the real docs/spec.toml: the new status.rs test (parse_spec, measure_problem, spec_gaps on include_str! of the file) and xtask/tests/spec.rs the_real_spec_passes (line 314), which calls real_spec() (reads docs/spec.toml from disk) and check_how_it_is_met (line 194). Preconditions: the spec has the unique id OLY-B-O5-01, and the quotes and headings of its sources are unchanged. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0: formatting is clean, clippy reports no warnings on the new test, and the new status.rs test and the_real_spec_passes both pass over the edited spec, so a non-blank mechanism, a role owner and a measure ending `status: restarts` with a known label are accepted by the library checker and the xtask checker. crates/athena-gate/tests/bootstrap_requirement.rs reads only OLY-B-O1-01 and is unaffected.
  Fails without the behavior: If the table row were missing, the new status.rs test inside `cargo test --locked` fails (label rejected). If the spec measure lacked the `status:` marker or the mechanism were blank, the new test and xtask the_real_spec_passes fail. If the test were badly formatted or had an unused binding, `cargo fmt --check` or clippy -D warnings fails.
  Test seam: The harness is the three Athena check commands. They call cargo test, which runs production parse_spec, measure_problem, spec_gaps (status.rs test) and the xtask spec checker (xtask/tests/spec.rs the_real_spec_passes) on the real docs/spec.toml; there is no fake or injection. The seam exists today (read: status.rs:1630-1656, xtask/tests/spec.rs:94-98 and 314); the new test is added by plan step 4.
- Criterion 5: Grep the changelog and inspect the diff. Run: `Grep 'OLY-B-O5-01' in CHANGELOG.md; git status; git diff --stat`. Observes: An Unreleased/Added entry names the mechanism, the measure `status: restarts` and the new restarts row in STATUS_MEASURE_LABELS. The diff lists only docs/spec.toml, crates/athena-core/src/status.rs and CHANGELOG.md; manage.rs, status/planning.rs and lifecycle.rs are not listed.

Planned files:

- crates/athena-core/src/status.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7fdb19ddde83 → candidate 7224b3933959. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  5 +++++
 crates/athena-core/src/status.rs | 30 ++++++++++++++++++++++++++++++
 docs/spec.toml                   |  4 ++--
 3 files changed, 37 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261112-081144-cntr/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-081144-cntr/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43183140381 for 7224b393395980d127a134011843c6b42f532db4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43183140381 for 7224b393395980d127a134011843c6b42f532db4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43183140381 for 7224b393395980d127a134011843c6b42f532db4)
- ci-result: `not a check` passed (Check results for 7224b393395980d127a134011843c6b42f532db4 come from the CI run https://github.com/doodla/athena/actions/runs/43183140381 (843s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`; all pass, including the xtask spec test.)
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-core --lib status && cargo test -p athena-core --lib hestia`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
