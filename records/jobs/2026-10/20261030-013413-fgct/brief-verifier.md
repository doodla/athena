# Job 20261030-013413-fgct: verify "Add effort-split status line and mechanism for OLY-B-O4-02"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-O4-02 a mechanism
> 
> Requirement OLY-B-O4-02 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena tracks how effort splits between the study and itself, and every milestone leaves the repo presentable, with what is done published and verified and what is not listed.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-O4-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-O4-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add an `effort` figure to `athena status` showing the share of recorded job cost spent on study-capability jobs versus Athena's own change jobs, then record it as the mechanism for OLY-B-O4-02 with a measure ending `status: effort`. The "what is not done is listed" half rests on the existing `requirement` spec-gap list. Audience: the owner reading `athena status` and the spec test that checks measures.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView fields, render) and crates/athena-core/src/status/planning.rs (planning_cost, producer_settings, specialization over Records/JobRecords); the entry point is `athena status` in athena-cli calling the status module; the figure is computed directly from recorded job capability (owner.capability) and cost, not a stand-in. No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected.
- Existing code: extends crates/athena-core/src/status.rs and crates/athena-core/src/status/planning.rs and reuses their cost and capability accessors; docs/spec.toml is edited, not created.
- Environment: reads only the local evidence snapshot directory passed to `athena status --snapshot`, read-only; no CI workflow, push, PR or records-branch access; history.json in that snapshot is the independent source.
- Effort is measured as recorded job cost in USD split by capability: study versus change (Athena itself). Wall time is not used because cost is already recorded per job.
- The 'presentable milestone' half is covered by the existing `requirement` spec-gap list and landing verification; the mechanism text says so rather than adding a new milestone gate.
- Governance files (athena.toml, defaults.toml, workflows) are not changed; CHANGELOG.md is optional.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-O4-02 has a mechanism naming the new effort-split function (in crates/athena-core/src/status/planning.rs) and the `effort` status label instead of "none", keeps owner "Athena", and its measure ends with `status: effort`; Grep shows the function exists.
2. [content] STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs contains an ("effort", ...) entry, and the `athena status` renderer prints a line starting with `effort`, so the measure label is one status prints.
3. [behavior] Run `cargo test -p athena-core`: a new unit test builds fixture records with study-capability and change jobs of known costs and asserts the study share and Athena share; it fails when the effort-split function is stubbed out or removed and passes with it. Also an empty record set yields an explicit 'no jobs recorded' line, not a zero share presented as success.
4. [behavior] Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records: the `effort` line's job counts and study/self shares are within the expected range taken from history.json (count of landed jobs by capability) and recorded job costs, sources this change does not compute; the shares sum to 100% and the total job count does not exceed the number of jobs in history.json.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root; all pass, including the xtask spec test that requires each measure to end with a status label.
6. [content] The change adds no new refusal rule; no readings are newly refused (expected 0% of newly refused readings).

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/status/planning.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- A new milestone gate or publish step
- Changing the keep rule, planning cost, landing time or throughput figures
- Changes to athena.toml, defaults.toml or .github/workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit test that fails without the mechanism; fmt, clippy, test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run status on real records and compare to history.json.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: Empty records print an explicit none-recorded line.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Requirement recorded with a code mechanism and status label.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base a3e7923352ce → candidate 1ce7b6009320. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |  4 +++
 crates/athena-cli/src/display.rs          | 54 +++++++++++++++++++++++++++--
 crates/athena-core/src/snapshot.rs        |  3 ++
 crates/athena-core/src/status.rs          | 54 +++++++++++++++++++++++++++++
 crates/athena-core/src/status/planning.rs | 57 +++++++++++++++++++++++++++++++
 docs/spec.toml                            |  4 +--
 6 files changed, 172 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-013413-fgct/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013413-fgct/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43175601002 for 1ce7b60093202fbe4050c64f110353b93d043a28)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43175601002 for 1ce7b60093202fbe4050c64f110353b93d043a28)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43175601002 for 1ce7b60093202fbe4050c64f110353b93d043a28)
- ci-result: `not a check` passed (Check results for 1ce7b60093202fbe4050c64f110353b93d043a28 come from the CI run https://github.com/doodla/athena/actions/runs/43175601002 (605s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root; all pass, including the xtask spec test that requires each measure to end with a status label.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
