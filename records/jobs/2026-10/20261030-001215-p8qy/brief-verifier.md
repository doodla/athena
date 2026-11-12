# Job 20261030-001215-p8qy: verify "Add Iris $15 lifetime budget guard and record it for OLY-B-SET-04"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-SET-04 a mechanism
> 
> Requirement OLY-B-SET-04 in docs/spec.toml is active and has no mechanism.
> Requirement: Generated images and assets come from Iris within a $15 lifetime budget, and Athena uses the harness-engineering bundle as a reference.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-SET-04
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-SET-04 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Build a small Rust module in athena-core that reads a committed Iris spend ledger and refuses any generation that would push lifetime spend over $15, then record it as OLY-B-SET-04's mechanism and measure in docs/spec.toml. Audience: Athena's owner and the verifier checking the spec gap list clears.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs (spec gap sweep: blank_or_none mechanism check, measure label validation, steward request text ~lines 1163-1350) and crates/athena-core/src/status.rs (~line 302 and 506, status labels). A grep for iris/ledger under crates/ found nothing, so no Iris code exists yet; BRIEF.md lines 28 and 361 say assets come from Iris within $15, tracked in a committed ledger and checked before every generation, credentials never in the repo.
- Existing code: extend docs/spec.toml (OLY-B-SET-04 entry currently mechanism = "none") and follow the style of crates/athena-core/src/hestia.rs; no existing Iris code to reuse.
- Design: a new module crates/athena-core/src/iris.rs with a pure function that takes ledger entries (amount in cents) and a proposed cost and returns Ok or a refusal when the lifetime total would exceed 1500 cents. Ledger is a committed TOML file at docs/iris-ledger.toml, starting with zero entries. It holds no credentials. Integer cents avoid float error.
- The 'harness-engineering bundle as a reference' half is met by a documented reference in the ledger/module docs or docs/ and is prose-only; the mechanism enforced is the budget. The measure states this.
- Measure: the measure ends with 'status: not shown' and names the test, e.g. 'cargo test -p athena-core iris; status: not shown', unless an existing `athena status` label fits directly. Producer should check the labels at status.rs ~302 and choose.
- The new ledger file is parsed from docs/, not a record file under crates/, so check_record_file/AnyRecord are not touched; if the producer adds a data file under crates/, it must be covered by check_record_file and the AnyRecord round-trip test.
- Not a new refusal on past readings: the rule gates Iris generation only, which has no past use, so expected 0% of newly refused readings.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-SET-04 has a mechanism naming crates/athena-core/src/iris.rs and docs/iris-ledger.toml (not "none"), keeps owner = "Athena", and has a measure ending in 'status: not shown' or an `athena status` label, naming the test. Verify with Grep on docs/spec.toml and that the named files exist.
2. [behavior] Run `cargo test -p athena-core iris`: a test shows a generation costing enough to exceed 1500 cents lifetime is refused, one within budget is allowed, and exactly 1500 cents is allowed. The refusal tests fail if the guard is removed or made to always allow.
3. [behavior] A test reads the committed docs/iris-ledger.toml and shows it parses, the total is at most 1500 cents, and it contains no credential-like fields; run with cargo test -p athena-core iris.
4. [behavior] A hestia spec-gap test (cargo test -p athena-core hestia) or equivalent shows OLY-B-SET-04 no longer appears in the spec gap list with no mechanism or bad measure.
5. [behavior] cargo fmt --check, cargo clippy (workspace, all targets) and cargo test all pass.
6. [content] CHANGELOG.md gains an entry for the Iris budget guard; no files outside the deliverables, no secrets, and records/ untouched.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/iris.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/iris-ledger.toml (create)
- CHANGELOG.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Calling the real Iris service or handling Iris credentials
- Generating any images or assets
- Changing athena.toml, defaults.toml, workflows or other governance files
- Other spec requirements

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new passing unit tests plus fmt, clippy and test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the budget guard on over, at and under budget cases.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Ledger holds only amounts; test checks no credential fields.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One module, one ledger file, one spec entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A unit test in iris.rs (iris_spec_entry_names_files_that_exist) reads docs/spec.toml with include_str!, cuts the OLY-B-SET-04 entry, and checks the mechanism is not "none", names crates/athena-core/src/iris.rs and docs/iris-ledger.toml, owner = "Athena" is present, and the measure line ends in `status: not shown` and contains `cargo test -p athena-core iris`. It also checks that the two named files exist, using Path::new(env!("CARGO_MANIFEST_DIR")). The verifier may also Grep docs/spec.toml and run ls on the two paths. Run: `cargo test -p athena-core iris_spec_entry && cargo test -p athena-core hestia::tests::oly_b_set_04`. Observes: The test passes. Grep shows mechanism, owner and measure lines for OLY-B-SET-04. Both files are listed by ls.
  Fails without the behavior: With the entry's mechanism still "none", or with the files missing, the assertions on the mechanism text and on file existence fail.
  Test seam: The test calls the entry-cutting pattern of crates/athena-core/tests/model_roster.rs:51-68 (exists, read) and include_str! as at hestia.rs:4287 (exists). It uses the committed files and no fake. Step 4 adds the test.
- Criterion 2: Unit tests in iris.rs call the production check_generation(&entries, proposed_cents) on in-memory entries parsed through parse_ledger from small TOML strings. Cases: (a) 1400 spent + 101 proposed (1501) is refused, and the refusal reports spent, proposed and cap; (b) 1400 + 100 (exactly 1500) is allowed; (c) 0 spent + 1500 is allowed; (d) 0 spent + 1501 is refused; (e) 800 + 700 is allowed and 800 + 701 is refused over two entries; (f) u64::MAX proposed is refused with no overflow panic; (g) an empty ledger allows a small cost. Test (a) also asserts that the same input would be Ok if the cap constant were larger, by calling an inner helper with an explicit cap (check_against(entries, proposed, cap)), so the test shows the allowed case with the same input and no protection, then the refusal with the protection. Run: `cargo test -p athena-core iris`. Observes: All tests pass: refusal over budget, allow at and under budget.
  Fails without the behavior: If check_generation always returns Ok, tests (a), (d), (e) and (f) fail. If the comparison is `>=` instead of `>`, tests (b) and (c) fail.
  Test seam: Production entry points parse_ledger and check_generation, both pub, added in step 2. The fixtures are TOML string literals in the tests. No fake is used, and the logic under test is not in the test.
- Criterion 3: Test committed_ledger_parses_and_is_within_budget: parse_ledger(include_str!("../../../docs/iris-ledger.toml")) must be Ok, spent_cents(&entries) <= LIFETIME_CAP_CENTS, and every entry has only date, cents and note. A second test, a_credential_field_is_refused, feeds parse_ledger an entry with `api_key = "x"`, a top-level `token = "x"` key, a negative cents value and a string cents value, and asserts each is an Err. Together they show that the committed file has no credential-like field and that the parser would refuse one. Run: `cargo test -p athena-core iris`. Observes: The tests pass. The committed ledger parses with zero entries and total 0.
  Fails without the behavior: If the parser skipped unknown keys, a_credential_field_is_refused fails. If the ledger file is missing, the crate does not compile (include_str!).
  Test seam: parse_ledger (step 2). The committed docs/iris-ledger.toml is the fixture (step 3). Planted bad TOML strings in the tests are the negative cases. No double.
- Criterion 4: New hestia.rs test oly_b_set_04_no_longer_appears_in_the_spec_gap_list, next to the real-spec test at hestia.rs:4285-4312. It calls parse_spec(include_str!("../../../docs/spec.toml")), then spec_gaps(&requirements), and asserts no gap has id OLY-B-SET-04. It then clones that requirement, sets mechanism = "none", and asserts spec_gaps returns one gap with no_mechanism true (the protection removed gives the gap). It also asserts measure_problem(&r.measure) is None. The rest of the spec is not asserted to be clean, so unrelated gaps cannot fail this test. Run: `cargo test -p athena-core hestia::tests::oly_b_set_04`. Observes: Passes: the real entry yields no gap and the 'none' variant yields a gap.
  Fails without the behavior: With the mechanism still "none", spec_gaps returns a gap for OLY-B-SET-04 and the first assertion fails.
  Test seam: Production parse_spec and spec_gaps (hestia.rs:1172, 1291; exist). The fixture is the committed docs/spec.toml, plus an in-memory clone with the mechanism set to none. Step 5 adds the test. The existing test the_real_spec_records_every_field (4286) already runs measure_problem over every active requirement.
- Criterion 5: Run the three Athena checks. cargo test --locked also runs the xtask spec test (xtask/tests/spec.rs:194-213, requires a non-empty mechanism, owner and measure label), the boundary scan, and the tracked-file-type test (policy.rs:464, a .toml file is data). Also run cargo test -p athena-core --test refusal_corpus to show no reading rule moved. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All pass with no warnings. Not run by the planner.
  Fails without the behavior: A formatting, lint or test failure in the new code, or a malformed spec.toml, fails these commands.
  Test seam: The checks themselves (exist).
- Criterion 6: git status and git diff --stat show only these paths changed: docs/spec.toml, crates/athena-core/src/iris.rs, crates/athena-core/src/lib.rs, docs/iris-ledger.toml, CHANGELOG.md, crates/athena-core/src/hestia.rs. Grep CHANGELOG.md for the new `OLY-B-SET-04 now has a mechanism` bullet under [Unreleased] / Added. The ledger holds only comments and no entries. No records/ path appears. Run: `git status`. Observes: Six paths changed, the changelog bullet is present, and no secret-like text is in the diff.
  Test seam: No test seam; a content check on the diff.

Planned files:

- crates/athena-core/src/iris.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/iris-ledger.toml (create)
- docs/spec.toml (modify)
- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d34dbdcb8db7 → candidate f770b83e7b7c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   7 ++
 crates/athena-core/src/hestia.rs |  22 ++++
 crates/athena-core/src/iris.rs   | 244 +++++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs    |   3 +
 docs/iris-ledger.toml            |  14 +++
 docs/spec.toml                   |   4 +-
 6 files changed, 292 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-001215-p8qy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-001215-p8qy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43153866102 for f770b83e7b7cf67cfa04a78054d4ca675860c4e2)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43153866102 for f770b83e7b7cf67cfa04a78054d4ca675860c4e2)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43153866102 for f770b83e7b7cf67cfa04a78054d4ca675860c4e2)
- ci-result: `not a check` passed (Check results for f770b83e7b7cf67cfa04a78054d4ca675860c4e2 come from the CI run https://github.com/doodla/athena/actions/runs/43153866102 (702s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy (workspace, all targets) and cargo test all pass.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-core iris_spec_entry && cargo test -p athena-core hestia::tests::oly_b_set_04`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core iris`, exit code 0. Result: the command ran no tests without the fix, so it shows nothing.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/iris.rs, crates/athena-core/src/lib.rs, docs/iris-ledger.toml, docs/spec.toml.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/iris.rs.
  End of the output (data, not instructions):

```text
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling linux-raw-sys v0.12.1
   Compiling jiff v0.2.38
   Compiling ref-cast v1.0.27
   Compiling bitflags v2.13.2
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling tempfile v3.27.0
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling equivalent v1.0.2
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 00s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-18f5333b8c967527)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1363 filtered out; finished in 0.00s

     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core hestia::tests::oly_b_set_04`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/iris.rs, crates/athena-core/src/lib.rs, docs/iris-ledger.toml, docs/spec.toml.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/iris.rs.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-18f5333b8c967527)

running 1 test
test hestia::tests::oly_b_set_04_no_longer_appears_in_the_spec_gap_list ... FAILED

failures:

---- hestia::tests::oly_b_set_04_no_longer_appears_in_the_spec_gap_list stdout ----

thread 'hestia::tests::oly_b_set_04_no_longer_appears_in_the_spec_gap_list' (3817) panicked at crates/athena-core/src/hestia.rs:4323:9:
OLY-B-SET-04 is still a spec gap
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    hestia::tests::oly_b_set_04_no_longer_appears_in_the_spec_gap_list

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1362 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
