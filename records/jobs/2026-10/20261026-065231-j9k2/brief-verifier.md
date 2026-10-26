# Job 20261026-065231-j9k2: verify "Build Hestia sweep skeleton, findings record, and accept D129"

## Request (verbatim)

> Priority: high
> Build job 1 of the Hestia design (docs/design/hestia.md section 9): the sweep skeleton and findings record, with the trigger settled. Also mark Hestia's decision accepted in docs/decisions.md, decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26).
> Acceptance:
> - A test where every check returns nothing yields a report listing all checks with counts; a test where a check errors yields a finding for it.
> - A command prints the last sweep.
> - The decision is marked accepted.
> Files: crates/athena-core/src/hestia.rs, crates/athena-core/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-data/src/records.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Implements job 1 of docs/design/hestia.md §9: a `hestia` module in athena-core with a Finding type and a sweep runner that lists every check with a count and turns a check error into a finding. It also adds a CLI command that prints the last sweep, settles the trigger, and marks D129 accepted by the brain under Sri's delegation. The audience is Sri and the later Hestia build jobs, which plug checks into this skeleton.

Assumptions:
- The trigger is settled as an explicit command (`athena hestia sweep` to run, `athena hestia last` to print), with a drain hook deferred to a later job; the decision text records this and why (no timer daemon exists, a command is testable and does not gate drains). If the repo shows a drain hook is cheap, the producer may add it, but the command is the minimum.
- The sweep report is stored through athena-data records.rs as a new record type, and `last` reads the newest one; the exact storage mirrors existing record patterns there.
- The skeleton ships with an empty or injected check list; real checks (rows 1-8) are later jobs. The check set is injectable so tests can supply fake checks.
- D129 is the Hestia decision (docs/decisions.md line ~3233, currently 'proposed'). Its status and heading become accepted, with 'decided by the brain under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26)'. D130 is untouched.
- docs/design/hestia.md section 10 open point on the trigger may be updated to point at the decision; this is optional and small.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-core/src/hestia.rs registers several checks that each return no findings. The resulting report lists every check by id with a count of 0 and has no findings. Run `cargo test -p athena-core hestia`.
2. [behavior] A unit test has one check return an error while the others succeed. The report holds a finding for that check (check id, subject, evidence containing the error, age, state), and the other checks are still listed with counts. Run `cargo test -p athena-core hestia`.
3. [behavior] A record-level test shows a swept report round-trips through athena-data records.rs and that reading the last sweep returns the most recent one. Run `cargo test -p athena-data`.
4. [behavior] `cargo run -- hestia last` (or the equivalent athena-cli invocation) prints the last sweep, listing each check with its count and each finding. With no sweep recorded it prints an explicit 'no sweep recorded' message, not an empty success. A CLI test covers both cases.
5. [content] Finding has the fields check id, subject, evidence, age and state, as specified in docs/design/hestia.md §9 job 1, and the module is exported from crates/athena-core/src/lib.rs.
6. [content] docs/decisions.md marks the Hestia decision (D129) accepted in both heading and Status line, names it as decided by the brain under Sri's delegation (BRIEF.md addenda of 2026-10-23 to 2026-10-26), and states the settled trigger with the alternatives considered and why. The 'not accepted; build jobs not queued' wording is removed or updated.
7. [content] CHANGELOG.md has an entry for the Hestia sweep skeleton, the command and the accepted decision, written for an outside reader.
8. [behavior] `cargo test`, `cargo clippy` (no warnings) and `cargo fmt --check` all pass across the workspace.
9. [content] No governance files (athena.toml, defaults.toml, workflows, Cargo lint config) are changed, and no check beyond the skeleton (rows 1-8) is implemented.

## Expected deliverables

- crates/athena-core/src/hestia.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-data/src/records.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/hestia.md (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-data/src/lib.rs (modify)

## Out of scope

- Implementing any actual check (rows 1 to 8, jobs 2 to 10)
- Model-backed checks, budget enforcement, fix-job emission
- A timer or daemon trigger
- Changes to D130 or the scope guard (D127)
- Governance files and records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust in the existing crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the module, the record and the command; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the report contents and the printed output, not that the types exist.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: The report lists checks with counts even when empty, and the 'last' command says explicitly when nothing was recorded.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Skeleton only; no real checks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The command is exercised through cargo run and tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the report and the printed sweep, which are the outcomes.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Check errors become findings by code, not by convention.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test with three fake clean checks run through run_sweep. Run: `cargo test -p athena-core hestia`. Observes: All three ids are listed with count 0 and the findings list is empty.
  Fails without the behavior: If run_sweep dropped clean checks from the report, the assertion that every id appears with count 0 would fail.
- Criterion 2: Unit test with one erroring check, one check with a real finding and clean ones. Run: `cargo test -p athena-core hestia`. Observes: One finding for the failing check (check id, subject, evidence containing the error text, age, state open); entry marked errored; the other checks are listed with their counts.
  Fails without the behavior: If an Err were propagated or ignored, the test would fail on the missing finding or on run_sweep returning an error.
- Criterion 3: Record tests: to_json then parse at the canonical path gives an equal sweep; the path is records/hestia-sweeps/<month>/<id>.json; validate rejects mismatched counts and duplicate ids. A core test (cargo test -p athena-core hestia) commits two sweeps and checks last_sweep returns the later. Run: `cargo test -p athena-data`. Observes: Round-trip equality, rejection cases, and the newest sweep returned.
  Fails without the behavior: If the AnyRecord variant, validate or the load path were missing, parse or last-selection would fail.
- Criterion 4: Tests in athena-cli/src/lib.rs call the renderer for no sweep and for a sweep with a finding. Also exercise `cargo run -- hestia last` in a repository with and without a sweep. Run: `cargo test -p athena-cli hestia`. Observes: 'no sweep recorded' when none exists; otherwise each check with its count and each finding with all five fields.
  Fails without the behavior: If `last` printed nothing on empty records, the 'no sweep recorded' assertion would fail; a missing finding line would fail the second test.
- Criterion 5: Tests build athena_core::hestia::Finding with all five fields; git diff shows `pub mod hestia;` in lib.rs. Run: `cargo test -p athena-core hestia`. Observes: Tests compile and pass.
  Fails without the behavior: The test would not compile if a field or the export were missing.
- Criterion 6: Read the docs/decisions.md diff. Run: `git diff`. Observes: D129 heading and Status say accepted, decided by the brain under Sri's delegation (BRIEF.md addenda 2026-10-23 to 2026-10-26); the trigger and alternatives are stated; the 'not accepted; build jobs not queued' wording is gone; D130 is unchanged.
- Criterion 7: Read the CHANGELOG.md diff. Run: `git diff`. Observes: An Unreleased > Added entry covering the skeleton, both commands, the record location and accepted D129, saying no checks ship yet.
- Criterion 8: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked. Run: `cargo test --locked`. Observes: All exit 0 workspace-wide, including the boundary-word scan and the budget tests.
  Fails without the behavior: A clippy warning, an unformatted file, or a failing test would make a command exit non-zero. For example, a missing Records::load arm for the new variant would panic in the core load test, and a banned word in the docs would fail the policy test.
- Criterion 9: git status and git diff over the changed paths. Run: `git status`. Observes: Only the listed deliverables plus store.rs changed; no governance file changed; default_checks is empty and no row 1-8 check exists.

Planned files:

- crates/athena-core/src/hestia.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- docs/decisions.md (modify)
- docs/design/hestia.md (modify)
- CHANGELOG.md (modify)

## The change

Base d7b7f2930f56 → candidate 9d02963dff2a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  10 ++
 crates/athena-cli/src/lib.rs      | 129 ++++++++++++++
 crates/athena-cli/src/main.rs     |  37 ++++
 crates/athena-core/src/hestia.rs  | 354 ++++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs     |   4 +
 crates/athena-core/src/store.rs   |   7 +
 crates/athena-data/src/lib.rs     |   3 +-
 crates/athena-data/src/records.rs | 186 ++++++++++++++++++++
 docs/decisions.md                 |  17 +-
 docs/design/hestia.md             |  10 +-
 docs/design/roadmap.md            |   2 +-
 docs/records.md                   |   1 +
 12 files changed, 750 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1293 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test`, `cargo clippy` (no warnings) and `cargo fmt --check` all pass across the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
