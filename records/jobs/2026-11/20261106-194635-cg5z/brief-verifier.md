# Job 20261106-194635-cg5z: verify "Add athena-testkit write_executable helper and exec-write scan test"

## Request (verbatim)

> try again on 20261106-104023-w7my

## Reading (accepted by the user)

Retry of job 20261106-104023-w7my, which was refused at admission only because crates/athena-testkit/Cargo.toml (a governance file with [lints]) was not named in a criterion. Same plan: one test-only helper writing executable stand-ins via an sh child, all test sites converted, plus a scan test; now with the Cargo.toml named in a criterion.

Assumptions:
- Prior job 20261106-104023-w7my: its only recorded finding was the admission refusal for crates/athena-testkit/Cargo.toml as an unlisted-in-criteria governance file. It is fixed by listing that file as a deliverable and naming it in a criterion. The candidate's 25 file changes are all kept as deliverables, including the deletion of crates/athena-git/src/script_writer.rs.
- Code survey: read crates/athena-git/src/snapshot.rs (script_writer use at ~513; production set_permissions at 445-478), crates/athena-git/src/lib.rs (pub mod script_writer), crates/athena-core/src/checks.rs (~1003), crates/athena-core/src/reading.rs (~8217), crates/athena-agent/src/claude.rs (~639), crates/athena-cli/tests/reconcile.rs (~134), crates/athena-cli/tests/slot_exec.rs (~54, a raw in-process set_permissions 0o755). Production uses in crates/athena-core/src/{toolconfig,pin,snapshot}.rs and crates/athena-cli/src/slot_exec.rs are not stand-in writes. The change adds no record or data file under crates/ and no refusal rule on readings.
- Existing code: crates/athena-git/src/script_writer.rs, crates/athena-git/src/lib.rs and docs/architecture.md are extended or replaced by the new crate. The existing helper is moved to athena-testkit.
- Governance: the new crate's Cargo.toml follows the workspace [lints] convention. Cargo.toml, Cargo.lock and the other crates' Cargo.toml files change only to add the dev-dependency and workspace member. athena.toml, defaults.toml and workflows do not change.
- Audience: maintainers running cargo test, who need flaky ETXTBSY failures gone and a mechanical guard against new in-process executable writes.

## Acceptance criteria

1. [content] crates/athena-testkit/Cargo.toml exists as a std-only dev-dependency crate with a workspace [lints] table, and is a workspace member in Cargo.toml. CHANGELOG.md names the root cause: executable stand-ins written in-process with no single helper and no mechanical enumeration of write sites. It also names the fix at the origin: athena_testkit::write_executable, which writes via an sh child (cat > tmp && chmod 755 tmp && mv tmp final).
2. [content] Grep of test code (xtask/src/main.rs, crates/athena-cli/tests/*.rs, crates/athena-agent/src/claude.rs, crates/athena-core/src/{checks,reading}.rs, crates/athena-git/src/snapshot.rs tests) finds no in-process executable write. The xtask gitleaks fake and crates/athena-cli/tests/slot_exec.rs use the helper, and crates/athena-git/src/script_writer.rs is deleted with no remaining references.
3. [behavior] Run `cargo test -p athena-testkit`. The scan test no_in_process_exec_writes passes on the tree. Its planted-violation unit tests show it fails on set_executable(&fake) wrappers, set_permissions with 0o755, from_mode and Command::new("chmod") in test code. These tests would fail without the detection logic.
4. [behavior] Run `cargo test -p athena-testkit --test stand_in_stress`. The helper variant records 0 ETXTBSY over many write-then-exec cycles with concurrent forking threads. The in-process variant's hit count appears in the recorded test output.
5. [content] git diff shows no non-test production change. The busy-file retry in reading.rs check_stub is removed, and production retry_busy in claude.rs is untouched.
6. [content] docs/decisions.md and docs/decision-links.toml have an appended entry saying the decision covering crates/athena-git/src/snapshot.rs was checked and stands. xtask/shard-timings.txt lists the new test files. docs/architecture.md's crate table lists athena-testkit.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`. All pass.

## Expected deliverables

- crates/athena-testkit/Cargo.toml (create)
- crates/athena-testkit/src/lib.rs (create)
- crates/athena-testkit/tests/stand_in_stress.rs (create)
- crates/athena-testkit/tests/no_in_process_exec_writes.rs (create)
- Cargo.toml (modify)
- Cargo.lock (modify)
- xtask/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-git/Cargo.toml (modify)
- xtask/src/main.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-cli/tests/slot_exec.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-git/src/script_writer.rs (delete)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- docs/architecture.md (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Out of scope

- Retries at run sites or in production code
- Changing production behavior of claude.rs or other non-test code
- Non-Linux platform support
- athena.toml, defaults.toml and workflow changes
- Production-code set_permissions uses in crates/athena-core/src/{toolconfig,pin,snapshot}.rs, crates/athena-cli/src/slot_exec.rs and crates/athena-git/src/snapshot.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The testkit has unit and integration tests, and the workspace test and clippy criterion is included.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is one helper plus a scan, not per-site patches.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the scan against planted violations and the stress test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The scan test enforces the rule mechanically.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The stress test measures ETXTBSY counts directly.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the manifests and changelog in the diff. Also run `cargo test -p xtask --test policy dependency_direction_matches_the_architecture`, which fails if a workspace member is missing from the docs row and checks the member through cargo metadata. Run: `cargo metadata --no-deps --format-version 1 && git diff --stat && git diff -- Cargo.toml crates/athena-testkit/Cargo.toml CHANGELOG.md`. Observes: `cargo metadata` lists package athena-testkit with an empty normal dependency list and `lints` inherited from the workspace; Cargo.toml `members` and `[workspace.dependencies]` name it; CHANGELOG.md [Unreleased] has an entry that states the root cause (stand-ins written in-process, no single helper, no mechanical enumeration of write sites) and the fix `athena_testkit::write_executable` via `cat > tmp && chmod 755 tmp && mv tmp final`.
- Criterion 2: Content check by the scan itself over the listed paths: the test `the_tree_has_no_in_process_executable_write` walks xtask/src, xtask/tests, crates/*/src and crates/*/tests and asserts no violations outside the exempt list. A supporting grep (Grep tool, pattern `script_writer|set_permissions|from_mode|set_executable`) over the listed files shows only production hits; `ls crates/athena-git/src` shows no script_writer.rs. Run: `cargo test -p athena-testkit --test no_in_process_exec_writes`. Observes: The test passes; the Grep finds `script_writer` nowhere in .rs files, and xtask/src/main.rs, crates/athena-cli/tests/slot_exec.rs, reconcile.rs, claude.rs, checks.rs, reading.rs and the snapshot.rs tests call `athena_testkit::write_executable`.
- Criterion 3: The test binary tests/no_in_process_exec_writes.rs contains `no_in_process_exec_writes` (the tree scan, named in the criterion, which passes on the tree) and planted-violation tests. A planted test passes a literal source string to `violations(path, text)`. It asserts a non-empty result for each planted form in a tests/ path and in the region after `#[cfg(test)]` of a src path. It asserts an empty result for a production-region use before `#[cfg(test)]`, a comment line, `.permissions().mode()` and a `write_executable` call. A guard test asserts each exempt-list file still exists and still has a hit. Run: `cargo test -p athena-testkit`. Observes: All tests pass: the scan on the tree passes, and every planted test shows a violation was found.
  Fails without the behavior: If the detection function returned no violations, every planted-violation test (`set_executable(&fake)` wrapper, `set_permissions` with `0o755`, `Permissions::from_mode(0o755)`, `Command::new("chmod")`, `.set_mode(0o755)`) would fail its assertion that the line is reported with its line number. If the tree walk found no files, the vacuity assertion (more than 100 files scanned, including xtask/src/main.rs and crates/athena-cli/tests/slot_exec.rs) would fail. Reverting any single converted site would fail `the_tree_has_no_in_process_executable_write`, naming the file and line.
  Test seam: Production entry point: `violations(rel_path: &str, text: &str) -> Vec<(usize, &'static str)>`, defined in crates/athena-testkit/tests/no_in_process_exec_writes.rs (added in step 6) and called by both the tree scan and the planted tests. Fixture: literal source strings and the real source tree, read through `CARGO_MANIFEST_DIR/../..`. The double holds only data (text); the detection logic is the function itself, not a stub. The seam is added by step 6.
- Criterion 4: Run the stress binary with `--nocapture`. The test `helper_variant_never_meets_a_busy_script` (4 writer threads x 200 cycles, 4 forker threads running `true`) calls `athena_testkit::write_executable` and then `Command::output()` on the file. It counts `ErrorKind::ExecutableFileBusy` and asserts 0, and asserts each output equals the cycle's expected line. `in_process_variant_reports_its_busy_count` does the same with `std::fs::write` plus an in-process chmod and prints the count. Run: `cargo test -p athena-testkit --test stand_in_stress -- --nocapture`. Observes: Output has `helper variant: 0 ETXTBSY in 800 cycles` and `in-process variant: <n> ETXTBSY in 800 cycles` (n is whatever was measured, possibly 0); both tests pass.
  Fails without the behavior: Without the helper's out-of-process write, the helper variant would write in-process like the other variant and meet ETXTBSY, so its assertion of zero busy errors would fail (this is the flake the change removes). If `write_executable` were a no-op the exec would fail with NotFound or permission denied and the test would fail. The in-process variant's printed count shows the race is real on this host.
  Test seam: Production entry point: `athena_testkit::write_executable(path, bytes)` in crates/athena-testkit/src/lib.rs (step 1); the file is run through `std::process::Command`. Fixture: `tempfile::tempdir()` and forker threads (real `true` processes). Nothing is injected or doubled. The helper is the code under test, and the in-process variant is a deliberate negative control. Both are added by step 7.
- Criterion 5: Read the diff. Every touched .rs path is a test file, a `#[cfg(test)]` region, the new testkit crate, or crates/athena-git/src/lib.rs (the `pub mod script_writer;` line only). `git diff -- crates/athena-agent/src/claude.rs` shows hunks only inside `mod stand_in_tests` (starts line 612), and `retry_busy` (line 47) is unchanged. `git diff -- crates/athena-core/src/reading.rs` shows the loop in `check_stub` (8221-8237) gone. Run: `git diff --stat`. Observes: No hunk outside test code or Cargo manifests; no `Text file busy` string and no `thread::sleep` retry in reading.rs `check_stub`.
- Criterion 6: The decisions test (crates/athena-cli/src/decisions.rs, the real-file tests at 411-412 and 480-503) parses the real docs/decisions.md and docs/decision-links.toml and requires every parsed phrase hit to be listed and every listed link to be found, so the D160 line number must be exact. The ci_shards test requires xtask/shard-timings.txt to equal the set of workspace test units, so it fails unless the four athena-testkit lines are added. The policy test fails unless docs/architecture.md has an `athena-testkit` row. Run: `cargo test -p athena-cli decisions && cargo test -p xtask --test ci_shards checked_in_timings_cover_every_workspace_test_unit && cargo test -p xtask --test policy dependency_direction_matches_the_architecture`. Observes: All three pass; the diff shows an appended `## D160` entry whose Decision line starts `Amends D08 in prose only`, a `[[link]]` with `older = "D08"`, `newer = "D160"`, `status = "amended"` and the matching `line`, four `athena-testkit` lines in shard-timings.txt (`--doc`, `--lib`, `--test no_in_process_exec_writes`, `--test stand_in_stress`) and the architecture row.
- Criterion 7: Run the three workspace commands as the checks do (`--locked`). The producer first runs `cargo build` once without `--locked` to let cargo write the new Cargo.lock entries. Run: `cargo fmt --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked`. Observes: All three exit 0; the test run includes athena-testkit's lib tests and both integration test files.
  Fails without the behavior: Each conversion step is covered. An unconverted `athena_git::script_writer` path would not compile once script_writer.rs is deleted. A leftover unused import (`PermissionsExt` in tests/slot_exec.rs, `Duration` in reading.rs) fails clippy under -D warnings. A stale Cargo.lock fails `--locked`. A missing shard-timings line or architecture row fails the policy and ci_shards tests. A wrong D160 link line fails the decisions test.
  Test seam: Entry points: the existing workspace cargo commands. Fixtures: none added beyond steps 1-12; each converted site now goes through `athena_testkit::write_executable` (exists from step 1).

Planned files:

- crates/athena-testkit/Cargo.toml (create)
- crates/athena-testkit/src/lib.rs (create)
- crates/athena-testkit/tests/stand_in_stress.rs (create)
- crates/athena-testkit/tests/no_in_process_exec_writes.rs (create)
- Cargo.toml (modify)
- Cargo.lock (modify)
- xtask/Cargo.toml (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-agent/Cargo.toml (modify)
- crates/athena-core/Cargo.toml (modify)
- crates/athena-git/Cargo.toml (modify)
- xtask/src/main.rs (modify)
- crates/athena-cli/tests/reconcile.rs (modify)
- crates/athena-cli/tests/slot_exec.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-git/src/script_writer.rs (delete)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- docs/architecture.md (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 85f74ae7604e → candidate 82d34244c941. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |  15 ++
 Cargo.lock                                         |  13 +-
 Cargo.toml                                         |   2 +
 crates/athena-agent/Cargo.toml                     |   2 +-
 crates/athena-agent/src/claude.rs                  |   2 +-
 crates/athena-cli/Cargo.toml                       |   1 +
 crates/athena-cli/tests/reconcile.rs               |   2 +-
 crates/athena-cli/tests/slot_exec.rs               |   4 +-
 crates/athena-core/Cargo.toml                      |   3 +
 crates/athena-core/src/checks.rs                   |   2 +-
 crates/athena-core/src/reading.rs                  |  18 +-
 crates/athena-core/src/test_support.rs             |   1 +
 crates/athena-git/Cargo.toml                       |   3 +
 crates/athena-git/src/lib.rs                       |   1 -
 crates/athena-git/src/snapshot.rs                  |   5 +-
 crates/athena-testkit/Cargo.toml                   |  16 ++
 .../script_writer.rs => athena-testkit/src/lib.rs} |  49 +---
 .../tests/no_in_process_exec_writes.rs             | 264 +++++++++++++++++++++
 crates/athena-testkit/tests/stand_in_stress.rs     |  91 +++++++
 docs/architecture.md                               |   1 +
 docs/decision-links.toml                           |   7 +
 docs/decisions.md                                  |  21 ++
 xtask/Cargo.toml                                   |   1 +
 xtask/shard-timings.txt                            |   4 +
 xtask/src/main.rs                                  |   4 +-
 25 files changed, 464 insertions(+), 68 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-194635-cg5z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-194635-cg5z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42286757004 for 82d34244c941beccc2b4bc23d03c3ae646e58e5c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42286757004 for 82d34244c941beccc2b4bc23d03c3ae646e58e5c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42286757004 for 82d34244c941beccc2b4bc23d03c3ae646e58e5c)
- ci-result: `not a check` passed (Check results for 82d34244c941beccc2b4bc23d03c3ae646e58e5c come from the CI run https://github.com/doodla/athena/actions/runs/42286757004 (588s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace`. All pass.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-testkit`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-git/src/lib.rs, crates/athena-git/src/script_writer.rs, crates/athena-testkit/src/lib.rs, docs/architecture.md, docs/decision-links.toml, docs/decisions.md, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling libc v0.2.190
   Compiling rustix v1.1.5
   Compiling getrandom v0.3.4
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling cfg-if v1.0.5
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
error[E0433]: cannot find module or crate `athena_testkit` in this scope
  --> crates/athena-testkit/tests/stand_in_stress.rs:72:33
   |
72 |     let hits = run(|path, body| athena_testkit::write_executable(path, body));
   |                                 ^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `athena_testkit`
   |
   = help: if you wanted to use a crate named `athena_testkit`, use `cargo add athena_testkit` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `athena-testkit` (test "stand_in_stress") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-testkit --test stand_in_stress -- --nocapture`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-git/src/lib.rs, crates/athena-git/src/script_writer.rs, crates/athena-testkit/src/lib.rs, docs/architecture.md, docs/decision-links.toml, docs/decisions.md, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
error[E0433]: cannot find module or crate `athena_testkit` in this scope
  --> crates/athena-testkit/tests/stand_in_stress.rs:72:33
   |
72 |     let hits = run(|path, body| athena_testkit::write_executable(path, body));
   |                                 ^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `athena_testkit`
   |
   = help: if you wanted to use a crate named `athena_testkit`, use `cargo add athena_testkit` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0433`.
error: could not compile `athena-testkit` (test "stand_in_stress") due to 1 previous error
```
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
