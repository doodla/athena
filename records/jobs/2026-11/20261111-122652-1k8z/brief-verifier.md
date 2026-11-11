# Job 20261111-122652-1k8z: verify "Give OLY-10-02 a mechanism whose evaluated stage needs real evaluation"

## Request (verbatim)

> try again on 20261029-233058-fveq

## Reading (accepted by the user)

Retry of job 20261029-233058-fveq: add a tested improvement-cycle check in athena-data, shown in `athena status`, and record it as the OLY-10-02 mechanism and measure. Unlike the rejected attempt, "evaluated candidate" counts only when a recorded evaluation result (method_experiment-style baseline vs candidate) exists, and the finding links to the proposal. The goal is for Athena's owner to see whether the final acceptance chain is shown, without the line claiming success on real data where nothing was evaluated.

Assumptions:
- Code survey: read crates/athena-core/src/proposals.rs (REPEAT_THRESHOLD = 3, proposals need 3 distinct jobs, so a jobs.len()>=2 filter is true for every real proposal), crates/athena-core/src/method_experiment.rs and role_value.rs (baseline-versus-candidate compare gives a verdict), crates/athena-data/src/records.rs (AnyRecord at line 1076, check_record_file at line 1402 is the allow-list and round-trip entry), crates/athena-core/src/status.rs (StatusView and its builder) and crates/athena-cli/src/display.rs. The check is a pure function over already-loaded records; status builds it from landed jobs and display prints one line.
- Job 20261029-233058-fveq: kept from its candidate: the improvement_cycle pure check, the status line, the spec.toml edit, the CHANGELOG entry and the status.rs plumbing. Fixed: (a) the evaluated stage no longer uses jobs.len()>=2; it uses a recorded evaluation result for that candidate, with the field named in code; (b) the finding is linked to the proposal by introspection id or failure class; (c) the status.rs doc comment is put back on build_measurement_line. The two extra files (status.rs, status_build_measurement.rs) are now listed as deliverables.
- Existing code: crates/athena-core/src/method_experiment.rs, crates/athena-core/src/proposals.rs, crates/athena-data/src/records/introspection.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs. The job reuses or extends these and creates none of them.
- If no evaluation record type exists in the records, the producer reuses the existing method_experiment result shape rather than adding a new record type. If a new type is added, it goes in the AnyRecord/check_record_file allow-list with a round-trip test.
- On today's real records the likely honest result is `status: not shown`, since no candidate has a recorded evaluation. That is acceptable only if the measure says so and the replay test shows both outcomes on fixtures.
- Goal and audience: Athena's owner reading `athena status` and the spec measure.

## Acceptance criteria

1. [content] docs/spec.toml OLY-10-02 has a mechanism naming an existing crates/ path and function (not "none"), owner Athena kept, and a measure ending in `status: <label>` where the label is a line `athena status` prints, or `status: not shown` plus the test name. Check by Grep on docs/spec.toml and on the named function in crates/.
2. [content] In crates/athena-data/src/records/introspection.rs, fn improvement_cycle no longer contains a `jobs.len() >= 2`-style filter for the evaluated stage. A code comment names the record field used as the evaluation signal. Check with Grep for the removed filter and the comment.
3. [behavior] Run `cargo test -p athena-data introspection`. Tests with fixtures shaped like real records show: a proposal with no evaluation record is not counted as evaluated; one with an evaluation record is; a finding about a different failure class or introspection than the proposal does not complete the chain; and a set missing any one stage reports that stage missing. The tests fail if the check always returns shown or always missing.
4. [behavior] An empty record set reports not shown, never success (no-empty-success). A test in `cargo test -p athena-data introspection` covers it.
5. [behavior] A test replays the check over a fixture mixing evaluated and unevaluated proposals, with the proposals built like real ones (3 distinct source jobs). It asserts both outcomes occur and that the stage is neither true for all nor false for all. Run `cargo test -p athena-data introspection`.
6. [behavior] `athena status` prints the `improvement cycle:` line, and display tests assert it for both shown and `not shown (missing: ...)`, with the unevaluated case naming the evaluation stage. Run `cargo test -p athena-cli` and `cargo test -p athena-core status`. All existing tests that read status output still pass.
7. [content] In crates/athena-core/src/status.rs the doc comment sits on the function it describes: build_measurement_line keeps its own rustdoc and improvement_cycle_line has its own. Check with git diff.
8. [content] Any new record or data type is covered by the allow-list and round-trip test (check_record_file/AnyRecord in crates/athena-data/src/records.rs), or no new record type is added. Check by Grep and git diff.
9. [content] CHANGELOG.md has an entry for the OLY-10-02 improvement-cycle check. Check with git diff.
10. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_build_measurement.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-data/src/records.rs (modify)

## Out of scope

- Running a real end-to-end demonstration with a live defect
- OLY-10-01, OLY-10-03 and other requirements
- Changing athena.toml, defaults.toml or workflows
- Prompt changes
- Generic enforcement of a replay check for all status measures (the prevention already tried for OLY-9-11)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the evaluated stage distinguishes true from false, not just that the line exists.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty record set is not shown, covered by a test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and display tests, plus fmt, clippy and test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 0fbfdc2d969e → candidate a8688710def8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |   7 +
 crates/athena-cli/src/display.rs                   |   1 +
 .../athena-cli/tests/status_build_measurement.rs   |  43 +++
 crates/athena-core/src/evidence_labels.rs          |   1 +
 crates/athena-core/src/proposals.rs                |   1 +
 crates/athena-core/src/status.rs                   |  69 ++++
 crates/athena-data/src/records.rs                  |  26 ++
 crates/athena-data/src/records/introspection.rs    | 395 +++++++++++++++++++++
 docs/spec.toml                                     |   4 +-
 9 files changed, 545 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-122652-1k8z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-122652-1k8z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43026475964 for a8688710def8640bff739140d269b54fa13334ca)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43026475964 for a8688710def8640bff739140d269b54fa13334ca)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43026475964 for a8688710def8640bff739140d269b54fa13334ca)
- ci-result: `not a check` passed (Check results for a8688710def8640bff739140d269b54fa13334ca come from the CI run https://github.com/doodla/athena/actions/runs/43026475964 (632s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
