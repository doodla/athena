# Job 20261030-044523-fyn6: verify "Give OLY-B-S3-01 a lab run-config mechanism and test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-S3-01 a mechanism
> 
> Requirement OLY-B-S3-01 in docs/spec.toml is active and has no mechanism.
> Requirement: All three systems are run for real within spend caps on purpose-built tasks, and every run is observable and reproducible from a recorded configuration.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-S3-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-S3-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a small athena-core lab module that parses and validates a recorded lab configuration (all three systems, a task, a pinned commit and a positive spend cap each), refuses a run once its cap is spent, and gives each run a config fingerprint. Record it as the mechanism of OLY-B-S3-01 in docs/spec.toml with a "status: not shown" measure naming the test that shows it.

Assumptions:
- Goal and audience: the owner and the verifier judge whether OLY-B-S3-01 has a real, tested mechanism; the lab config is what a later operator uses to run the three studied systems.
- Code survey: read docs/spec.toml (the OLY-B-S3-01 entry at line ~2961 has mechanism = "none" and measure "The spec gap list; status: requirement"), crates/athena-core/tests/model_roster.rs (pattern for a spec-entry test: finds the entry, rejects mechanism none, checks owner and measure ending), crates/athena-core/src/status.rs (status labels such as keep rule, repeated, mistakes), and crates/athena-data/src/config.rs (Config::parse and validate, whose refusal pattern the lab config follows). Entry point is a new crates/athena-core/src/lab.rs: parse, then validate, then check the cap before each run. No record or data file under crates/ is added, so check_record_file and AnyRecord are untouched; the lab config is read from a caller-supplied TOML string, not stored as a record. The property is measured directly by calling the validator and the cap check, not a stand-in.
- Existing code: docs/spec.toml, crates/athena-core/tests/model_roster.rs (test pattern reused), crates/athena-data/src/config.rs (validation style extended by analogy). No existing spend-cap or lab code was found by grep in crates/.
- The measure is 'status: not shown' because no `athena status` label measures lab runs; the measure names the test file that shows it.
- Buy-first: the config fingerprint uses a hash crate already in Cargo.lock if one exists, otherwise std's stable hashing is avoided and a documented existing dependency is reused; no new heavy crate is added. toml and serde are already used by the workspace.
- Environment: none; no CI workflow, push or history read is needed. This adds a refusal rule only for new lab configs, not for past job readings, so no replay refusal-rate criterion applies.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-S3-01 no longer has mechanism = "none"; its mechanism names crates/athena-core/src/lab.rs, owner stays "Athena", and its measure ends with `status: not shown` and names the test file crates/athena-core/tests/lab_runs.rs.
2. [content] crates/athena-core/src/lab.rs exists, is exported from crates/athena-core/src/lib.rs, and defines a lab configuration with the three systems, a task, a pinned commit and a spend cap in USD per system, plus a fingerprint of the configuration.
3. [behavior] Run `cargo test -p athena-core --test lab_runs`: a valid config with three systems and positive caps is accepted; configs missing a system, with a zero or negative cap, or without a pinned commit are refused; a run whose recorded spend reaches its cap is refused; the same config gives the same fingerprint and a changed cap gives a different one.
4. [behavior] The test lab_runs.rs also reads docs/spec.toml and fails if the OLY-B-S3-01 entry has mechanism none, no owner, or a measure not ending in a status label; it fails when lab.rs's validation is removed and passes with it.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` as separate commands (or the recorded Athena checks on the final tree); all pass.
6. [content] CHANGELOG.md gains an entry for this job; no existing entry of docs/decisions.md or records/ is edited.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/lab.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/lab_runs.rs (create)
- CHANGELOG.md (modify)
- crates/athena-core/Cargo.toml (modify)

## Out of scope

- Actually running the three studied systems or spending money on them.
- Changing other requirements in docs/spec.toml, athena.toml, defaults.toml or workflows.
- Editing docs/decisions.md or records/.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has tests; fmt, clippy and test criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the validator and cap check rather than checking that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module plus its test and spec entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f798602b0712 → candidate 23209abdaad5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   1 +
 crates/athena-core/src/lab.rs        | 138 +++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs        |   3 +
 crates/athena-core/tests/lab_runs.rs | 114 +++++++++++++++++++++++++++++
 docs/spec.toml                       |   4 +-
 xtask/shard-timings.txt              |   1 +
 6 files changed, 259 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-044523-fyn6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-044523-fyn6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43370252092 for 23209abdaad5a8ee2ed087091acec477b1acb292)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43370252092 for 23209abdaad5a8ee2ed087091acec477b1acb292)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43370252092 for 23209abdaad5a8ee2ed087091acec477b1acb292)
- ci-result: `not a check` passed (Check results for 23209abdaad5a8ee2ed087091acec477b1acb292 come from the CI run https://github.com/doodla/athena/actions/runs/43370252092 (764s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` as separate commands (or the recorded Athena checks on the final tree); all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
