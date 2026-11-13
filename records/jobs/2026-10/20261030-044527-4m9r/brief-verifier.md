# Job 20261030-044527-4m9r: verify "Give OLY-B-S3-02 a divergence-finding mechanism and measure"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-S3-02 a mechanism
> 
> Requirement OLY-B-S3-02 in docs/spec.toml is active and has no mechanism.
> Requirement: Observed behavior is reconciled against what the prompts and docs say, and every divergence is a first-class finding.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-S3-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-S3-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small athena-core module that reconciles observed behavior against documented claims and emits every mismatch as a first-class Divergence finding. Record it as the mechanism of OLY-B-S3-02 in docs/spec.toml with a measure ending `status: not shown`, plus a test that fails without it.

Assumptions:
- Goal and audience: Athena's owner and the verifier need to see that prompt/doc claims are compared with observed behavior and each mismatch becomes a named finding, not a silent drop.
- Code survey: read crates/athena-core/src/lab.rs (the OLY-B-S3-01 sibling: pure module, caller supplies text), crates/athena-core/src/lib.rs (pub mod list and module doc bullets), and crates/athena-core/tests/lab_runs.rs (unit tests plus spec_entry_has_a_real_mechanism, which reads docs/spec.toml and checks mechanism, owner, and a measure ending `status: not shown`). Entry point is a pure function; no resolution/probing order applies. The new module adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by calling the reconcile function on claim and observation inputs.
- Existing code: crates/athena-core/src/lab.rs and crates/athena-core/tests/lab_runs.rs are the pattern to follow; docs/spec.toml holds the entry (mechanism = "none" at the OLY-B-S3-02 entry, around line 2979).
- Design: new module crates/athena-core/src/divergence.rs, registered as `pub mod divergence;` in lib.rs with a doc bullet. Reconcile takes a list of claims (source path, text, expected value) and a list of observations (claim key, observed value) and returns Divergence findings: Mismatch (both present, differ), Unobserved (claim without observation), Undocumented (observation without claim). It runs nothing and reads no files. Not wired into prompts or CLI, so the measure is `status: not shown`.
- Alternative considered: a CLI command `athena reconcile`; rejected as larger than one focused change and `athena status` has no label for it, so the measure uses `status: not shown`.
- Environment: none; no CI, push or records access.

## Acceptance criteria

1. [content] docs/spec.toml entry id = "OLY-B-S3-02" no longer has mechanism = "none"; its mechanism names crates/athena-core/src/divergence.rs and the reconcile function, owner stays "Athena", and measure names crates/athena-core/tests/divergence.rs and ends with `status: not shown`.
2. [content] crates/athena-core/src/divergence.rs exists, is declared in crates/athena-core/src/lib.rs with a doc bullet, and defines Divergence findings of three kinds (mismatch, unobserved claim, undocumented observation) returned by a reconcile function.
3. [behavior] Run `cargo test -p athena-core --test divergence`: reconcile on matching input gives no findings, and on a differing value, a claim with no observation, and an observation with no claim gives exactly one finding of the matching kind each, none dropped. The test cannot compile or pass without the module.
4. [behavior] The same test file has a spec test that reads docs/spec.toml and fails while OLY-B-S3-02 has mechanism = "none" (run via cargo test -p athena-core --test divergence).
5. [behavior] cargo fmt --check passes.
6. [behavior] cargo clippy --all-targets passes with no warnings.
7. [behavior] cargo test passes for the workspace.
8. [content] CHANGELOG.md has a new entry recording OLY-B-S3-02's mechanism, its test name and that its measure ends `status: not shown`.

## Expected deliverables

- crates/athena-core/src/divergence.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/divergence.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Wiring the reconciler into prompts, roles or the athena CLI
- Actually reconciling the real prompts and docs against live runs
- Changes to any other requirement in docs/spec.toml, records/, athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has its own integration test; fmt, clippy, test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test checks each divergence kind is reported, not just that the module exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One pure module and one spec entry.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Module doc comments explain the requirement for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d5e80fb23b13 → candidate 71d75703a825. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   1 +
 crates/athena-core/src/divergence.rs   |  92 +++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs          |   4 ++
 crates/athena-core/tests/divergence.rs | 104 +++++++++++++++++++++++++++++++++
 docs/spec.toml                         |   4 +-
 xtask/shard-timings.txt                |   1 +
 6 files changed, 204 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-044527-4m9r/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-044527-4m9r/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43374776965 for 71d75703a82539a9a776da4d37d9c30ceef35f11)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43374776965 for 71d75703a82539a9a776da4d37d9c30ceef35f11)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43374776965 for 71d75703a82539a9a776da4d37d9c30ceef35f11)
- ci-result: `not a check` passed (Check results for 71d75703a82539a9a776da4d37d9c30ceef35f11 come from the CI run https://github.com/doodla/athena/actions/runs/43374776965 (819s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check passes.; cargo clippy --all-targets passes with no warnings.; cargo test passes for the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
