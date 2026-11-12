# Job 20261030-001207-ghn9: verify "Give OLY-A-12 a mechanism: surface changes need a design decision"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-A-12 a mechanism
> 
> Requirement OLY-A-12 in docs/spec.toml is active and has no mechanism.
> Requirement: The API surface is not treated as settled.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-A-12
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-A-12 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Extend the existing design-surface inventory (crates/athena-core/src/design_surface.rs) with a deterministic check that a change which adds, removes or alters the surface (commands, config keys, record schemas, roles, crates) also touches docs/decisions.md. Record it as OLY-A-12's mechanism and measure in docs/spec.toml, backed by a test that fails without it. Audience: the Athena owner and verifier checking that the API is not treated as settled.

Assumptions:
- Code survey: read crates/athena-core/src/design_surface.rs (inventory_at, diff, SurfaceDiff::is_empty/totals, existing tests), crates/athena-core/src/lib.rs (pub mod design_surface), and crates/athena-core/src/status.rs STATUS_MEASURE_LABELS. Entry point is the pure function over a SurfaceDiff and a list of changed paths; no resolution or probing order applies. No record or data file is added under crates/, so check_record_file/AnyRecord are untouched. The property is measured directly via diff output, not a stand-in.
- Existing code: crates/athena-core/src/design_surface.rs (inventory_at, diff, SurfaceDiff) is extended, and docs/decisions.md is the design-decision log; docs/spec.toml holds the requirement.
- Assumption: 'design decision' means an entry in docs/decisions.md in the same change; the mechanism is a library check, not yet wired into land or athena status, so the measure is `status: not shown` naming the test, since no status label fits.
- Consumers: none found (no config deliverable).
- Term ladder: not used in this reading.

## Acceptance criteria

1. [content] docs/spec.toml OLY-A-12 has mechanism naming the new function in crates/athena-core/src/design_surface.rs (Grep finds that function there), owner "Athena", and a measure ending `status: not shown` that names the test (cargo test -p athena-core design_surface).
2. [behavior] Run `cargo test -p athena-core design_surface`: a new test passes showing that when diff(old,new) is non-empty and the changed paths exclude docs/decisions.md the new check reports the undecided surface changes, and when docs/decisions.md is among the paths, or the diff is empty, it reports none. The test fails if the function body is replaced by returning no findings.
3. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`; all pass, including the xtask spec test that requires the measure marker.
4. [content] Only docs/spec.toml, crates/athena-core/src/design_surface.rs and CHANGELOG.md change; no governance file (athena.toml, defaults.toml, workflows) and no record schema is added, so the check_record_file/AnyRecord round trip is unaffected. The check is not wired into landing or refusal, so no reading is newly refused (expected 0% of newly refused readings).

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/design_surface.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Wiring the check into landing, admission or athena status output
- Adding new record schemas or athena.toml keys
- Changing other requirements' mechanisms

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Mechanism written in Rust inside athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Adds a unit test; fmt, clippy and test must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One function and test in an existing module; no wiring.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test checks both flagged and unflagged outcomes.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check by reading and grep. The two cargo test runs only confirm the new text is well-formed: the xtask spec parser and the hestia real-spec test both read the real docs/spec.toml. Run: `Grep 'pub fn undecided_surface_changes' in crates/athena-core/src/design_surface.rs; Read the OLY-A-12 table in docs/spec.toml; then cargo test -p xtask --test spec; cargo test -p athena-core --lib the_real_spec_records_every_field`. Observes: The OLY-A-12 mechanism names `undecided_surface_changes` in crates/athena-core/src/design_surface.rs, owner = "Athena", and the measure ends `status: not shown` and names `cargo test -p athena-core design_surface`. The Grep finds the function in design_surface.rs. Both spec tests pass: non-empty mechanism, owner not blank or none, and `measure_problem(&r.measure) == None` for every active requirement (hestia.rs:4290-4294; xtask/tests/spec.rs:627).
  Fails without the behavior: Without the spec edit the mechanism reads `none` and names no function, and the Grep finds no function. This is what the content criterion catches. The spec tests do not fail without the edit (they accept `none`, xtask/tests/spec.rs:645); they fail only if the new text is malformed, such as a measure missing its `status:` marker.
  Test seam: Entry points: Read and Grep for the content; xtask/tests/spec.rs (`check_how_it_is_met` 190-213 and the sweep 627-648) and hestia.rs `the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed` (4285-4294, `include_str!("../../../docs/spec.toml")`). Fixture: docs/spec.toml itself, edited in plan step 4. No double. Both test seams exist today.
- Criterion 2: Run the new unit test `a_surface_change_without_a_decision_is_reported_and_one_with_it_is_not` in design_surface.rs. It calls the production `undecided_surface_changes` on diffs from the production `diff` over two hand-built `SurfaceInventory` values. The old and new inventories differ by one added command, one removed crate and one changed role version. Calls: paths `["crates/a/src/lib.rs"]`; the same plus `docs/decisions.md`; `./docs/decisions.md`; and an empty diff `diff(&old,&old)` with paths excluding the log. Run: `cargo test -p athena-core design_surface`. Observes: The first call returns three findings, for commands, roles and crates, each naming its counts. The calls with the log in the paths (plain and with leading `./`) return an empty list. The empty diff returns an empty list even without the log. The existing design_surface tests still pass.
  Fails without the behavior: Replacing the function body with `Vec::new()` fails the first assertion, which expects three findings. Ignoring the log fails the second assertion. Always flagging fails the empty-diff assertion.
  Test seam: Entry point: `undecided_surface_changes`, added by plan step 1. Fixture: `SurfaceInventory` values (Default, public BTreeMaps) filled directly in the test and passed to the existing `diff` (design_surface.rs:722). Guard: an empty diff or the log among the paths short-circuits to no findings. The test is added by step 2 in the existing `tests` module (design_surface.rs:773). No double, and the logic under test is not in the fixture. The inventories and `diff` exist today; the function and test are added by steps 1 and 2.
- Criterion 3: Run the three checks Athena runs at the workspace root. Also run by name the tests that read the real spec.toml, decisions.md and CHANGELOG.md: cargo test -p athena-core --lib the_real_spec_records_every_field; cargo test -p xtask --test spec; cargo test -p athena-cli --test backtest; cargo test -p athena-cli --test mirrors; cargo test -p athena-cli decisions; cargo test -p athena-core --lib reading; cargo test -p athena-core --test refusal_corpus. Run: `cargo fmt --all --check; cargo clippy --all-targets --locked -- -D warnings; cargo test --locked`. Observes: All commands exit 0. The spec tests accept the new measure marker (`not shown` passes `measure_problem`). The mirror rules accept D172 (next id after D171, `## D172 · ...` heading, Provenance bullet, earlier bytes unchanged; the pinned-prefix test in crates/athena-cli/src/decisions.rs:246 still hashes the same first 359622 bytes). The backtest finds no new hit and no phrase lacking a target in D172. The reading.rs and refusal_corpus changelog and decision-text tests still find their items.
  Fails without the behavior: If D172 lacks the required shape or earlier lines of decisions.md are edited, mirrors and the pinned-prefix test fail. If D172 contains a supersession phrase, the backtest test fails. A malformed measure fails the hestia and xtask spec tests. Unformatted code fails fmt; a lint fails clippy with -D warnings.
  Test seam: Entry points: the three cargo commands and the named tests. Fixture: the real tracked files docs/spec.toml, docs/decisions.md, CHANGELOG.md and crate sources, read from disk or by `include_str!`, listed in `dependents`. Nothing is faked. All these seams exist today; the new inputs are the edits of steps 1 to 6.
- Criterion 4: List the changed paths in the producer's tree and compare them with the expected deliverables. Run: `git status`. Observes: Only docs/spec.toml, crates/athena-core/src/design_surface.rs, CHANGELOG.md and docs/decisions.md are modified; run/checks.rs is untouched. athena.toml, defaults.toml and .github/workflows are untouched and no record schema is added, so `AnyRecord` and `check_record_file` are untouched. No caller of the new function exists outside its test, so nothing is newly refused (0 newly refused).

Planned files:

- crates/athena-core/src/design_surface.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base b74a18c97216 → candidate 0e8c55e15311. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |  6 +++
 crates/athena-core/src/design_surface.rs | 73 ++++++++++++++++++++++++++++++++
 docs/decisions.md                        | 27 ++++++++++++
 docs/spec.toml                           |  4 +-
 4 files changed, 108 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-001207-ghn9/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-001207-ghn9/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43136345781 for 0e8c55e15311adce16f003ee761b26bf9fbe5221)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43136345781 for 0e8c55e15311adce16f003ee761b26bf9fbe5221)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43136345781 for 0e8c55e15311adce16f003ee761b26bf9fbe5221)
- ci-result: `not a check` passed (Check results for 0e8c55e15311adce16f003ee761b26bf9fbe5221 come from the CI run https://github.com/doodla/athena/actions/runs/43136345781 (944s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`; all pass, including the xtask spec test that requires the measure marker.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`Grep 'pub fn undecided_surface_changes' in crates/athena-core/src/design_surface.rs; Read the OLY-A-12 table in docs/spec.toml; then cargo test -p xtask --test spec; cargo test -p athena-core --lib the_real_spec_records_every_field`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core design_surface`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/design_surface.rs, docs/decisions.md, docs/spec.toml.
  End of the output (data, not instructions):

```text
   Compiling bitflags v2.13.2
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling tempfile v3.27.0
   Compiling schemars_derive v1.2.2
   Compiling regex-automata v0.4.18
   Compiling dyn-clone v1.0.20
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling equivalent v1.0.2
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling regex v1.13.1
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `undecided_surface_changes` in this scope
    --> crates/athena-core/src/design_surface.rs:1090:24
     |
1090 |         let findings = undecided_surface_changes(&changes, &["crates/a/src/lib.rs"]);
     |                        ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `undecided_surface_changes` in this scope
    --> crates/athena-core/src/design_surface.rs:1099:20
     |
1099 |         assert_eq!(undecided_surface_changes::<&str>(&changes, &[]).len(), 3);
     |                    ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `undecided_surface_changes` in this scope
    --> crates/athena-core/src/design_surface.rs:1102:17
     |
1102 |         assert!(undecided_surface_changes(&changes, &with_log).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `undecided_surface_changes` in this scope
    --> crates/athena-core/src/design_surface.rs:1103:17
     |
1103 |         assert!(undecided_surface_changes(&changes, &["./docs/decisions.md"]).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `undecided_surface_changes` in this scope
    --> crates/athena-core/src/design_surface.rs:1106:17
     |
1106 |         assert!(undecided_surface_changes(&same, &["crates/a/src/lib.rs"]).is_empty());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo fmt --all --check; cargo clippy --all-targets --locked -- -D warnings; cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
