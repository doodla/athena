# Job 20261030-033225-g36k: verify "Track verification cost, catches, false positives and escapes in status"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A7-04 a mechanism
> 
> Requirement OLY-B-A7-04 in docs/spec.toml is active and has no mechanism.
> Requirement: Verification is tracked: its cost, what it caught, false positives and errors that escaped.
> Owner: Apollo
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A7-04
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A7-04 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a "verification" section to `athena status` computed from existing verification, job and revert/intervention records: verifier cost, rejections caught, false positives (rejections later overruled), and escapes (landed jobs later reverted or fixed). Register the label and set OLY-B-A7-04's mechanism and measure to `status: verification`. Audience: Apollo and the owner reading status.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at ~305, MistakeReport and mistake_report at ~584, StatusView built at ~1321), crates/athena-core/src/status/planning.rs (keep rule, later-harm pattern to reuse), crates/athena-cli/src/display.rs (status() at 1258, mistakes() at 512), crates/athena-data/src/records.rs (Verification with agent_verdict/final_verdict/findings, CheckRun, Totals.cost_execution, Finding). Entry point is Status::status() building StatusView, rendered by display::status. No new record or data file under crates/ is added, so check_record_file/AnyRecord/round-trip need no change; records are only read.
- Existing code: extend crates/athena-core/src/status.rs, crates/athena-core/src/status/planning.rs (later-harm logic) and crates/athena-data/src/records.rs types; docs/spec.toml existing label convention. Reuse the later-harm/revert detection rather than building a second one.
- Definitions: caught = verification with final_verdict rejecting; false positive = a rejection later overruled or whose job landed by the owner/retry unchanged (best available from records, labelled as such); escaped = landed job later cited as a revert or fix. If a figure cannot be derived from records, the line says so rather than showing a guess.
- Governance: athena.toml, defaults.toml and workflows are not changed; no Consumers entry needed.
- Not a refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [content] In docs/spec.toml, OLY-B-A7-04 has a `mechanism` naming the verification report code in crates/athena-core/src/status (not "none"), keeps owner "Apollo", and a `measure` ending with `status: verification`. Check with Grep on docs/spec.toml.
2. [content] STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs contains the label ("verification", "verification") mapped to a new StatusView field, and crates/athena-cli/src/display.rs prints a line starting with `verification` from that field. Check with Grep.
3. [behavior] A unit test in crates/athena-core builds Records with verifications (rejected and passed), job totals, an overruled rejection and a landed job later reverted, and asserts the report counts total verifier cost, rejections caught, false positives and escapes. The test fails when the computation is stubbed out and passes with it. Run `cargo test`.
4. [behavior] A test in crates/athena-cli/tests renders `athena_cli::display::status` and asserts the output contains a line starting with `verification` showing cost, caught, false positives and escaped counts. Run `cargo test`.
5. [behavior] With empty Records, a test asserts the verification line contains the text `none yet` and no `0` counts shown as success (no-empty-success). Run `cargo test`.
6. [behavior] The xtask spec test accepts the new measure marker and label. Run `cargo test` (covers xtask).
7. [behavior] fmt, clippy and test pass: run `cargo fmt --check`, `cargo clippy`, `cargo test`.
8. [content] CHANGELOG.md has an added entry for this change; no existing entry of docs/decisions.md or anything under records/ is edited.

## Expected deliverables

- crates/athena-core/src/status/verification.rs (create)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- docs/enforcement.md (modify)

## Out of scope

- Changing verifier behavior, verdict rules or gates
- Editing existing records/ or docs/decisions.md entries
- Other spec requirements

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus fmt, clippy, test.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Empty-records criterion requires 'none yet'.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check rendered status output and counts.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9284350cbe17 → candidate eec234764cac. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                  |  1 +
 crates/athena-cli/src/display.rs              | 10 ++++
 crates/athena-cli/tests/introspection.rs      | 37 ++++++++++++
 crates/athena-core/src/status.rs              | 81 +++++++++++++++++++++++++++
 crates/athena-core/src/status/planning.rs     |  4 +-
 crates/athena-core/src/status/verification.rs | 64 +++++++++++++++++++++
 docs/enforcement.md                           |  1 +
 docs/spec.toml                                |  4 +-
 8 files changed, 198 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-033225-g36k/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-033225-g36k/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43330738124 for eec234764cac3b8937e8b1fa904c7adf30788a9e)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43330738124 for eec234764cac3b8937e8b1fa904c7adf30788a9e)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43330738124 for eec234764cac3b8937e8b1fa904c7adf30788a9e)
- ci-result: `not a check` passed (Check results for eec234764cac3b8937e8b1fa904c7adf30788a9e come from the CI run https://github.com/doodla/athena/actions/runs/43330738124 (915s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: fmt, clippy and test pass: run `cargo fmt --check`, `cargo clippy`, `cargo test`.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
