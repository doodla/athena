# Job 20261030-024702-v82w: verify "Give OLY-B-A3-02 a mechanism: learnings recorded, bundle attributed"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A3-02 a mechanism
> 
> Requirement OLY-B-A3-02 in docs/spec.toml is active and has no mechanism.
> Requirement: What Athena learns from firstmate and Hermes is recorded with rationale, and where the harness-engineering bundle shaped or did not shape a decision that is recorded and attributed under its license.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A3-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A3-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add an integration test in athena-core that checks the firstmate rationale records carry rationale fields and that the README attributes the harness-engineering bundle under CC BY 4.0, then record it as OLY-B-A3-02's mechanism with a measure ending `status: not shown`. The audience is the spec reviewer who checks the requirement has a real mechanism.

Assumptions:
- Term Hermes: in this request it means Hermes Agent, the studied self-improving agent that writes its own skills (the first sense in docs/ontology.toml, term Hermes), not the Orchestration god of the Olympus pantheon.
- Term firstmate: the first studied system, whose work handling is recorded in studies/firstmate-work-handling (docs/ontology.toml, BRIEF.md).
- Code survey: crates/athena-core/tests/model_roster.rs is the pattern (spec_entry_has_a_real_mechanism reads docs/spec.toml, checks the mechanism is not "none", owner is Athena, and the measure ends `status: not shown`); crates/athena-core/tests/buy_first.rs does the same with a stricter measure check; crates/athena-core/src/status.rs (~line 302) says a measure ends `status: <label>` or `status: not shown` and names the test. Entry point is the test file reading repo files via CARGO_MANIFEST_DIR/../..; no new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly on studies/firstmate-work-handling/rationales.json and README.md.
- Existing code: studies/firstmate-work-handling/rationales.json (study.rationale/1 records with decision, alternatives, tradeoff, provenance, sources), README.md lines ~322-323 (Harness Engineering attribution, CC BY 4.0), docs/design/athena-v0.md §Attribution (~line 1606), docs/design/requirements-map.md row A3-2 (marks it Partial: firstmate work handling only). The new test reads these and reuses the model_roster.rs spec-entry check pattern.
- The mechanism is a test, so the measure is `cargo test -p athena-core --test learnings_recorded; status: not shown`. The spec entry currently reads mechanism = "none" and measure = "The spec gap list; status: requirement".
- Hermes Agent has no study records in studies/ yet, so the test covers what exists (firstmate rationale records, attribution) and the spec mechanism says Hermes study rationales are not yet covered. It does not create a Hermes study.
- No refusal rule is added; Athena's gate and reading checks are unchanged. No config or governance file changes (athena.toml, defaults.toml, workflows). CHANGELOG.md gets the default entry. Environment: none, local files only.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-A3-02 no longer has mechanism = "none"; its mechanism names the new test file crates/athena-core/tests/learnings_recorded.rs and states the Hermes gap, the owner stays "Athena", and the measure ends with `status: not shown` and names the test command.
2. [content] crates/athena-core/tests/learnings_recorded.rs exists and asserts that every record in studies/firstmate-work-handling/rationales.json has a non-empty decision, alternatives, tradeoff, provenance and at least one source.
3. [content] The same test asserts README.md names the Harness Engineering bundle, its repository URL and the CC BY 4.0 license, and that docs/spec.toml's OLY-B-A3-02 entry is not mechanism = "none".
4. [behavior] Mutation checks inside the test, run with `cargo test -p athena-core --test learnings_recorded`: a copy of the rationales text with one tradeoff emptied, and a copy of the README text without the license line, are each reported as a problem by the checking function, while the real files pass. This shows the test fails without the mechanism and passes with it.
5. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (recorded Athena checks count under D115).
6. [content] CHANGELOG.md records the change. The deliverables are limited to docs/spec.toml, the new test file under crates/athena-core/tests/, and CHANGELOG.md.

## Expected deliverables

- crates/athena-core/tests/learnings_recorded.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/design/requirements-map.md (modify)

## Out of scope

- Studying Hermes Agent or producing new study records
- Changing the Athena gate, reading checks or any refusal rule
- Editing docs/decisions.md or records/
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test fails on mutated inputs and passes on the real files; fmt, clippy, test are criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Mutation cases check the behavior, not only that the file exists.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a test and named in the spec.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The Hermes coverage gap is stated in the mechanism text.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test reads the real docs/spec.toml, parses it with parse_spec, finds OLY-B-A3-02 and runs the production gap and measure checks. Run: `cargo test -p athena-core --test learnings_recorded spec_entry_has_a_real_mechanism`. Observes: The mechanism is not `none` and contains `crates/athena-core/tests/learnings_recorded.rs` and `Hermes`. The owner is `Athena`. measure_problem is None. The measure ends `status: not shown` and contains `cargo test -p athena-core --test learnings_recorded`. spec_gaps over the parsed spec holds no OLY-B-A3-02. On the base spec (mechanism `none`, label `status: requirement`) the first assertion fails.
  Fails without the behavior: Reverting the spec entry to mechanism = "none" fails the `none` assertion and the spec_gaps assertion. A second test, spec_check_refuses_a_none_mechanism, sets the mechanism to `none` on the parsed entry and asserts spec_gaps reports it.
  Test seam: Entry point athena_core::hestia::{parse_spec, spec_gaps, measure_problem} (exist, hestia.rs:1219, imported as in buy_first.rs:8) on docs/spec.toml read by a `repo_file` helper copied from buy_first.rs:14-19 (CARGO_MANIFEST_DIR/../..). Step 1 adds the helper and the test. No fake.
- Criterion 2: Call rationale_problems on the real rationales.json and claims.json text. Run: `cargo test -p athena-core --test learnings_recorded the_firstmate_rationale_records_carry_their_rationale`. Observes: The six cards (ra1-ra6, counted in the test) give an empty problem list. Each has a non-blank decision, tradeoff and reversal_cost, a non-blank alternative and a provenance in stated/evidenced/inferred. Each reaches at least one quoted source: ra1, ra2, ra3, ra5 and ra6 through their own sources, ra4 (`sources: []`) through the quotes of c7 and c8 in claims.json (claims.json:51, 58). Every cited claim id exists.
  Fails without the behavior: A checker that ignores the fields or the source reach returns nothing on the mutated copies in criterion 4, so those tests fail. A checker that required own `sources` on every card would fail here at ra4.
  Test seam: rationale_problems(rationales_text, claims_text) -> Vec<String> in the test file (step 2), parsing with serde_json (an athena-core dependency, Cargo.toml:22), driven by the real files read with repo_file. A snapshot of rules.rs:368-394 and 419-432, stricter on source reach; the production path on this study is crates/athena-cli/tests/drain.rs:7287. No double.
- Criterion 3: Call readme_attribution_problems on the real README.md and design_attribution_problems on the real docs/design/athena-v0.md. The third part, that the spec entry is not `none`, is shown by the criterion 1 test. Run: `cargo test -p athena-core --test learnings_recorded the_readme_and_design_attribute_the_harness_engineering_bundle`. Observes: README.md names `Harness Engineering`, `https://github.com/lopopolo/harness-engineering` and `CC BY 4.0` (README.md:322-324). athena-v0.md has the section 15 heading, the license, the URL and the table header `Departure, and why` with at least one row (athena-v0.md:1604-1625). Both return empty problem lists.
  Fails without the behavior: A README without the license or a design file without the section 15 table returns problems; criterion 4 shows this.
  Test seam: Two text-in, problems-out functions in the test file (step 2); real files read by repo_file. No double.
- Criterion 4: Mutation tests run the checkers on edited copies of the real text made with str::replace. Each first asserts the target text is present in the real file, so a mutation that changes nothing cannot pass. Each also asserts the real text gives no problem. Run: `cargo test -p athena-core --test learnings_recorded`. Observes: (a) The first `tradeoff` value emptied: names ra1 and tradeoff. (b) ra1's `reversal_cost` emptied: names ra1. (c) ra1's own `"quote": "Nothing else ... - is"` blanked and ra1's `"claims": ["c1", "c13"]` set to `[]`: names ra1 as a stated card with no quoted source, and only ra1. (d) ra4's `"claims": ["c7", "c8"]` set to `[]` (its `sources` is already empty): names ra4 as reaching no quoted source, and only ra4. (e) ra4's claims changed to `["c7", "c99"]`: names c99 as a missing claim. (f) README with the `CC BY 4.0` link removed: names the license. (g) Design file with the `Departure, and why` header removed: names the table.
  Fails without the behavior: If the checkers were removed or always returned an empty list, every mutated copy would report no problem and (a)-(g) would fail. If the real files lacked the rationale or the attribution, the real-file tests would fail.
  Test seam: rationale_problems, readme_attribution_problems and design_attribution_problems called directly with text; mutated text comes from str::replace on repo_file output. Added by steps 1-3. No double.
- Criterion 5: Run the three Athena checks on the final tree. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The mirror check finds a timing line for `athena-core --test learnings_recorded` (mirrors.rs:206-236). The athena-cli test derived_units_agree_with_cargo_metadata (crates/athena-cli/tests/mirrors.rs:214-223) still agrees. The strategy-view and requirements-map tests pass with the edited A3-2 row.
  Fails without the behavior: Without the shard-timings line, missing_timing_lines reports `athena-core --test learnings_recorded` as a test unit with no line.
  Test seam: The repository's own cargo commands; no seam needed.
- Criterion 6: Compare the changed files with the deliverables. Run: `git diff --stat`. Observes: Changed files: crates/athena-core/tests/learnings_recorded.rs, docs/spec.toml, CHANGELOG.md, docs/design/requirements-map.md (a listed deliverable) and xtask/shard-timings.txt (one added line, required by the mirror rule; the only file outside the listed deliverables). CHANGELOG.md has a new bullet under `## [Unreleased]` / `### Added` naming OLY-B-A3-02, the test, the Hermes gap, the unchecked per-row truth of section 15 and `status: not shown`. athena.toml, defaults.toml, workflows, docs/decisions.md, records/ and the study files are untouched.
  Fails without the behavior: A missing CHANGELOG bullet or an edited governance or study file is visible in the diff.
  Test seam: No seam; the check reads the diff.

Planned files:

- crates/athena-core/tests/learnings_recorded.rs (create)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/design/requirements-map.md (modify)
- xtask/shard-timings.txt (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 0a478d210f64 → candidate 34c9aedbf524. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                   |   1 +
 crates/athena-core/tests/learnings_recorded.rs | 332 +++++++++++++++++++++++++
 docs/design/requirements-map.md                |   2 +-
 docs/spec.toml                                 |   4 +-
 xtask/shard-timings.txt                        |   1 +
 5 files changed, 337 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-024702-v82w/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-024702-v82w/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43311836661 for 34c9aedbf524b29f40b498a5ec6421c2cd5a2208)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43311836661 for 34c9aedbf524b29f40b498a5ec6421c2cd5a2208)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43311836661 for 34c9aedbf524b29f40b498a5ec6421c2cd5a2208)
- ci-result: `not a check` passed (Check results for 34c9aedbf524b29f40b498a5ec6421c2cd5a2208 come from the CI run https://github.com/doodla/athena/actions/runs/43311836661 (762s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (recorded Athena checks count under D115).)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test learnings_recorded spec_entry_has_a_real_mechanism`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, docs/design/requirements-map.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling thiserror-impl v2.0.21
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling ref-cast v1.0.27
   Compiling jiff v0.2.38
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling dyn-clone v1.0.20
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 34.62s
     Running tests/learnings_recorded.rs (<job>/target/verify/debug/deps/learnings_recorded-ac4f2643c7b6c897)

running 1 test
test spec_entry_has_a_real_mechanism ... FAILED

failures:

---- spec_entry_has_a_real_mechanism stdout ----

thread 'spec_entry_has_a_real_mechanism' (22335) panicked at crates/athena-core/tests/learnings_recorded.rs:294:5:
assertion `left != right` failed
  left: "none"
 right: "none"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    spec_entry_has_a_real_mechanism

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p athena-core --test learnings_recorded`
error: 1 target failed:
    `-p athena-core --test learnings_recorded`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test learnings_recorded the_firstmate_rationale_records_carry_their_rationale`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, docs/design/requirements-map.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/learnings_recorded.rs (<job>/target/verify/debug/deps/learnings_recorded-ac4f2643c7b6c897)

running 1 test
test the_firstmate_rationale_records_carry_their_rationale ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test learnings_recorded the_readme_and_design_attribute_the_harness_engineering_bundle`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, docs/design/requirements-map.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/learnings_recorded.rs (<job>/target/verify/debug/deps/learnings_recorded-ac4f2643c7b6c897)

running 1 test
test the_readme_and_design_attribute_the_harness_engineering_bundle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core --test learnings_recorded`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
