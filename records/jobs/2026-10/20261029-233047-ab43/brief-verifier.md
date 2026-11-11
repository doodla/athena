# Job 20261029-233047-ab43: verify "Give OLY-9-09 a mechanism linking classes, improvements and roles"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-9-09 a mechanism
> 
> Requirement OLY-9-09 in docs/spec.toml is active and has no mechanism.
> Requirement: Phase 5 connects feedback classes, improvements, role effectiveness and long-term engineering outcomes.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-9-09
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-9-09 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add an athena-core module that joins each repeated failure class to the improvement request that addresses it, and to the role-effectiveness (specialization) and class-trend outcomes. Print it in `athena status` under the existing `repeated` label. Record it as OLY-9-09's mechanism and measure in docs/spec.toml, with a test that fails without it.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView, Records::status, specialization), crates/athena-core/src/class_trend.rs (ClassTrend::analyze/compute/lines), crates/athena-core/src/hestia.rs (RepeatedCause, repeated_causes, is_open) and the status display in crates/athena-cli/src/display.rs. Entry point is Records::status, which computes repeated_causes, specialization and class_trend before display prints them. The new module adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched.
- Existing code: extend crates/athena-core/src/hestia.rs RepeatedCause (class, request, landed), crates/athena-core/src/class_trend.rs and the specialization view in crates/athena-core/src/status.rs rather than rebuild them; docs/spec.toml OLY-9-08 and OLY-9-10 already use the repeated and trend labels.
- Goal: Athena's owner and the Hestia spec pass need to see feedback classes tied to improvements and role outcomes. The measure keeps the existing label `status: repeated`, which is in STATUS_MEASURE_LABELS.
- Consumers: none found for a config change; no athena.toml or defaults.toml edit is needed.
- The new join is display-only and adds no refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [content] docs/spec.toml OLY-9-09 has mechanism naming the new crates/athena-core module and its function (not "none"), owner "Athena", and a measure ending in `status: repeated`.
2. [behavior] Run `cargo test -p athena-core learning`: a fixture with a repeated failure class, a landed addressing request, and a specialization/class-trend outcome yields one line per class that names the class, the request and its landed/open state, and the outcome. The same test fails if the join function is stubbed to return nothing.
3. [behavior] Run `cargo test -p athena-cli`: the status display prints the new lines under the `repeated` label, and prints nothing extra when there are no repeated causes (an empty history is not shown as success).
4. [behavior] Run `cargo test` for the workspace, including the xtask spec test that checks the measure marker; it passes.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets`; both are clean.
6. [content] The diff changes only docs/spec.toml and files under crates/ (the new module, its mod line in lib.rs, status.rs and display.rs wiring), and adds no secrets or personal data.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/learning.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Changing other requirements' mechanisms (OLY-9-07, OLY-9-08, OLY-9-10)
- Changing prompts or role ladders

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new unit tests plus workspace cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test commands that check the printed join, not that files exist.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A criterion requires that no repeated causes prints nothing extra.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse existing hestia, class_trend and specialization code; no new crates.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1d08ff457ef3 → candidate ecef8a7310cd. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   3 +
 crates/athena-cli/src/display.rs   |  19 +++++
 crates/athena-core/src/learning.rs | 157 +++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs      |   1 +
 crates/athena-core/src/status.rs   |  28 +++++--
 docs/spec.toml                     |   4 +-
 6 files changed, 205 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-233047-ab43/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233047-ab43/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42997688470 for ecef8a7310cd1eaaddbb8ca36886f2d249c298f4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42997688470 for ecef8a7310cd1eaaddbb8ca36886f2d249c298f4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42997688470 for ecef8a7310cd1eaaddbb8ca36886f2d249c298f4)
- ci-result: `not a check` passed (Check results for ecef8a7310cd1eaaddbb8ca36886f2d249c298f4 come from the CI run https://github.com/doodla/athena/actions/runs/42997688470 (826s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check` and `cargo clippy --all-targets`; both are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
