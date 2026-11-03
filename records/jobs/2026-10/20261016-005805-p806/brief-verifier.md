# Job 20261016-005805-p806: verify "Accept newer Claude Code patch versions instead of stopping"

## Request (verbatim)

> Claude Code updates itself on this machine (2.1.296 to 2.1.299 to 2.1.301 in two days), and each update stops every agent against the pin until a person repins by hand, because repinning is itself a job that cannot start. Athena should not stop because its harness updated.

## Reading (accepted by the user)

Athena stops when the self-updating claude moves past the exact pin. I read the goal as accepting an installed claude with the same major.minor and a patch at or above the pin, recording the version actually reported. The audience is whoever runs Athena and currently repins by hand.

Assumptions:
- Code survey: read crates/athena-agent/src/claude.rs. check_pinned (line 61) runs inside ClaudeCodeDriver::new, after refuse_nested, and compares claude_version(bin) with == to the pin. The result is stored as the driver version. Tests at about lines 612 and 855 use a stand-in binary. crates/athena-data/src/config.rs holds claude_version; crates/athena-cli/src/wiring.rs builds a DeferredDriver from it. xtask/src/main.rs ensure_claude (line 621) also requires an exact match before installing the pin privately with npm. No record or data file is added, so check_record_file and AnyRecord are not affected. The version is measured by running the stand-in binary's --version, the same path the driver uses.
- The request is read as goal: Athena keeps running across Claude Code patch updates. Audience: whoever runs Athena. The pin in athena.toml stays and acts as the tested minimum.
- The earlier readings are not replies from the user, so no user reply overrides them. I chose Reading 1 because repinning cannot run as a job, so a repin command does not remove the stop.
- athena.toml is not edited. Only the check, the setup lookup that has to agree with it, and their docs change.
- This loosens a refusal rule, so it refuses fewer readings, not more. Expected 0% of newly refused readings.

## Acceptance criteria

1. [behavior] A stand-in claude reporting a newer patch than the pin (pin 2.1.299, binary 2.1.301) is accepted by check_pinned, and the returned version is 2.1.301. Run `cargo test -p athena-agent`.
2. [behavior] Tests show a binary with an older patch than the pin, a different major or minor, or an unparseable or empty version is still refused, with an error naming both the found and pinned versions. Run `cargo test -p athena-agent`.
3. [behavior] ClaudeCodeDriver::new built with a pin and a binary of a different accepted patch records the binary's reported version, not the pin, and a test asserts this. Run `cargo test -p athena-agent`.
4. [behavior] xtask ensure_claude uses the same compatibility rule, so a newer-patch claude on PATH is used rather than triggering an npm install. A test covers the comparison. Run `cargo test -p xtask`.
5. [behavior] `cargo test --workspace` and `cargo clippy --workspace --all-targets` pass, and `cargo fmt --check` is clean.
6. [content] docs/decisions.md gains an entry amending D03 that states the rule, the alternatives (any 2.x, auto-repin, keep exact), what is lost (an identical version on every run), and the reversal cost. CHANGELOG.md has an entry.
7. [content] The refusal message and README or setup docs tell the reader that a newer patch of the pinned major.minor needs no repin. athena.toml, defaults.toml and .github/workflows are unchanged, and no secrets or personal data are committed.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- xtask/src/main.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-cli/tests/live.rs (modify)

## Out of scope

- An xtask repin command that rewrites athena.toml
- Accepting a different major or minor version
- Editing athena.toml, defaults.toml or workflows
- Changing how Claude Code is invoked or its flags

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in the agent and xtask crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests with stand-in binaries; cargo test and clippy must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that a newer patch is accepted and others are still refused.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run the stand-in tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision entry and error text are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the version check and its docs.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data are committed.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The compatibility rule is enforced in check_pinned and tested.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in claude.rs `stand_in_tests`: write a stand-in `claude` that prints `2.1.301 (Claude Code)`, call `check_pinned(&bin, "2.1.299")`, assert Ok("2.1.301"). Second assertion in the same test: a stand-in reporting exactly 2.1.299 returns Ok("2.1.299"). Run: `cargo test -p athena-agent newer_patch`. Observes: check_pinned returns Ok with the binary's reported version 2.1.301, not the pin.
  Fails without the behavior: With the old `version != pinned` comparison, check_pinned on a 2.1.301 stand-in against pin 2.1.299 returns Err(Spawn), so the unwrap in the test fails.
  Test seam: Production entry point `claude::check_pinned`, which runs the stand-in through `claude_version` (the real --version path). Fixture: new helper `stand_in_version(dir, version)` writes the existing STAND_IN script with the 2.1.299 literal replaced; the path is passed as `bin`. `stand_in` exists (claude.rs:603); `stand_in_version` is added by step 2.
- Criterion 2: (a) A table test of pure `accepts(found, pinned)`: 2.1.298 vs 2.1.299 refused; 2.2.0, 2.0.999 and 3.1.299 refused; "", "abc", "2.1", "2.1.301-beta" refused; 2.1.1000 vs 2.1.999 accepted (numeric order); "2.1.301-beta" vs pin "2.1.301-beta" accepted (the equal-string escape). (b) The same refused forms through stand-in binaries: check_pinned returns Err(DriverError::Spawn(msg)); msg contains the found text (quoted), the pinned version, and 'needs no repin'. The existing test with pin 9.9.9 (claude.rs:852) still passes. Run: `cargo test -p athena-agent refused`. Observes: Every refused form returns an error that names the found and the pinned version; every accepted form returns Ok.
  Fails without the behavior: A string-order comparison or one that ignored major/minor would accept the older-patch, other-minor, other-major and unparseable rows; a message missing either version fails the contains assertions; without the escape row's rule that row would fail.
  Test seam: Production entry points `claude::check_pinned` and `claude::accepts`. Fixture: `stand_in_version` (added by step 2), including an empty version and a non-version string. Nothing is faked inside the rule.
- Criterion 3: Two tests. In claude.rs: build a stand-in reporting 2.1.301, call `ClaudeCodeDriver::with_marker(bin, "2.1.299", registry, None)` and assert `info().version == "2.1.301"` and name claude-code; repeat for a stand-in reporting 2.1.299; `with_marker(.., Some(job))` returns DriverError::Nested. In wiring.rs (finding 1): `DeferredDriver::for_test("2.1.299", registry, stand_in_2_1_301, None)`: before any use `info().version` is the pin 2.1.299; after `probe()` (the call `invoke` makes first) it is 2.1.301; a stand-in that fails `--version` leaves info() at the pin. Run: `cargo test -p athena-agent records_the_reported && cargo test -p athena-cli deferred_driver`. Observes: The driver's info() version equals the binary's reported version, not the pin, in the agent crate and in the wrapper whose info() the invocation records read.
  Fails without the behavior: If `new` stored the pin, info().version would be 2.1.299 and the 2.1.301 assertion fails. If DeferredDriver::info still returned `self.pinned`, the wiring test's post-probe assertion fails.
  Test seam: Production entry points `ClaudeCodeDriver::with_marker` (added by step 3; `new` is a one-line call to it) and `DeferredDriver::probe`/`info` (step 4 adds probe, the test constructor and the info change). Fixtures: `stand_in_version` in claude.rs; in wiring.rs a small shell stand-in written into `tempfile::tempdir()` (tempfile is a dev-dependency of athena-cli) and `Registry::new(tempdir)` (pub, procs.rs:133). The bin path and None marker are injected through the test constructor; the version is measured by running the stand-in's --version, the same path the driver uses.
- Criterion 4: Unit tests in xtask `tests`: (a) `claude_accepts(reported_line, pin)` takes the first whitespace token of a line such as `2.1.301 (Claude Code)`, with the same rows as the claude.rs table including the equal-string row. (b) `pick_claude(&[(PathBuf, Option<String>)], pin)`, which `ensure_claude` calls: a PATH candidate at 2.1.301 with the private one at 2.1.299 picks the PATH candidate (no npm); PATH at 2.1.298 with private at 2.1.299 picks private; PATH of a different minor and no private install picks none, so the npm path is reached. Run: `cargo test -p xtask claude_compat`. Observes: A newer-patch claude is chosen, and an older or different-minor one is skipped. Only when none is chosen does the npm install path run.
  Fails without the behavior: With the old `== Some(pin)` comparison, the 2.1.301 PATH candidate is not picked and the pick_claude test for that pair fails; a wrong loop condition now fails the pick_claude tests.
  Test seam: Production entry points `claude_accepts` and `pick_claude`, both pure, added by step 6; `ensure_claude` runs `found(...)` for each candidate and passes the results to `pick_claude`. No double. Limit: `ensure_claude` itself runs claude and npm and is not run by the test; the only logic left in it is gathering the candidates' --version text and the npm fallback.
- Criterion 5: Run the whole-workspace commands. Athena's own checks run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. Run: `cargo test --workspace`. Observes: All three exit 0, including the decisions-log prefix test, the policy scans and the system-facts budget test.
  Fails without the behavior: A compile error, failing test, clippy warning or unformatted file makes the matching command exit nonzero.
  Test seam: No seam: whole-workspace commands. Existing stand-in tests (claude.rs:612, 855) keep their outcomes because 2.1.299 vs pin 2.1.299 and vs 9.9.9 are decided as before. The existing wiring.rs test is untouched.
- Criterion 6: Content check. docs/decisions.md ends with `## D154 · A newer patch of the pinned Claude Code is accepted (amends D03)` in the D153 layout: Case, Root cause, Decision (the rule; DeferredDriver::info follows the reported version), Alternatives (any 2.x, auto-repin, keep exact), Tradeoff (a run no longer has one identical version; the dispatch-time job record, written at run/mod.rs:916 before the binary is probed, and the pre-use status line still show the pin), Reversal cost (low: restore the equality in claude.rs, xtask and the DeferredDriver::info line), Provenance. CHANGELOG.md [Unreleased] gets a Changed entry. The verifier reads both; the decisions test checks the append-only prefix. Run: `cargo test -p athena-cli decisions`. Observes: D154 exists after D153 with the listed parts, D03 bytes are unchanged, and the CHANGELOG entry exists.
  Fails without the behavior: Editing D03 or inserting D154 anywhere but the end fails the append-only prefix check in crates/athena-cli/src/decisions.rs; a missing entry leaves the content criterion unmet when read.
  Test seam: No behavior seam (content criterion). The existing decisions log test checks the append-only prefix.
- Criterion 7: The criterion 2 message assertion covers the refusal text ('needs no repin' plus both versions). README.md line 26, which names 2.1.299 although athena.toml pins 2.1.302, is replaced with: Claude Code at the pinned version or a later patch of the same major.minor needs no repin, or npm to install the pin. `git diff --stat` lists no athena.toml, defaults.toml or .github path; the policy test scans tracked files. Run: `git diff --stat`. Observes: The diff lists no athena.toml, defaults.toml or .github path; README and the error text carry the hint; the policy test passes.
  Fails without the behavior: An edit to a governance file shows in the diff; a refusal text without the hint fails the criterion 2 message test.
  Test seam: No behavior seam beyond the criterion 2 message assertion on `check_pinned` (exists after step 2). The scan runs with `cargo test -p xtask --test policy`.

Planned files:

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- xtask/src/main.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-cli/tests/live.rs (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base 07a4f54f3b10 → candidate 8df5d6d6cb8d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  10 +++
 README.md                         |   6 +-
 crates/athena-agent/src/claude.rs | 150 ++++++++++++++++++++++++++++++++++++--
 crates/athena-cli/src/wiring.rs   |  82 +++++++++++++++++++--
 crates/athena-cli/tests/live.rs   |   4 +-
 docs/decisions.md                 |  44 +++++++++++
 xtask/src/main.rs                 | 124 +++++++++++++++++++++++++++----
 7 files changed, 390 insertions(+), 30 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41710043053 for 8df5d6d6cb8df6e57a3910b43443653b051d8a51)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41710043053 for 8df5d6d6cb8df6e57a3910b43443653b051d8a51)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41710043053 for 8df5d6d6cb8df6e57a3910b43443653b051d8a51)
  Failing tests of `test` (from the CI log):
  - `backtest the_real_log_has_no_condition_a_finding_against_the_confirmed_list`
  - `xtask test-shard 0 6` (did not finish)`
  End of the failing job's log (data from CI, not instructions):

```text
test test_shard_6_is_disjoint_and_equals_the_workspace_target_set ... ok
test workflow_jobs_reject_a_missing_gate_condition_need_or_timeout ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s


== shard 0/6: xtask --test fingerprint
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
     Running tests/fingerprint.rs (target/debug/deps/fingerprint-dc44ee0178a22a4a)

running 10 tests
test ci_cache_a_missing_entry_runs_the_full_job ... ok
test ci_cache_a_marker_for_another_fingerprint_does_not_skip ... ok
test ci_cache_a_pass_is_saved_and_the_same_fingerprint_then_skips ... ok
test ci_cache_bad_arguments_are_refused ... ok
test ci_cache_report_names_the_fingerprint_and_the_matched_key ... ok
test ci_cache_a_failed_run_writes_no_marker_and_the_next_run_is_full ... ok
test ci_cache_check_appends_its_answer_to_the_output_file ... ok
test fingerprint_outside_a_repository_fails ... ok
test fingerprint_changes_with_governed_content ... ok
test fingerprint_ignores_records_and_owner_paths ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s


== shard 0/6: xtask --test walltime
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
     Running tests/walltime.rs (target/debug/deps/walltime-1886a37680fa6661)

running 6 tests
test missing_timestamps_are_stated_not_printed_as_zero ... ok
test failed_and_rejected_jobs_are_reported_apart ... ok
test median_and_p90_of_dispatch_to_verified_time ... ok
test no_records_is_stated ... ok
test the_window_is_the_last_n_jobs_by_dispatch ... ok
test fewer_than_twenty_jobs_is_stated_and_twenty_is_not_flagged ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: shard 0 of 6: 1 of 9 units failed: athena-cli --test backtest
##[error]Process completed with exit code 1.
Post job cleanup.
[command]/usr/bin/git version
git version 2.56.0
Temporarily overriding HOME='/home/runner/work/_temp/ad78a0c0-b5ed-4aeb-b69a-07855ca47425' before making global git config changes
Adding repository directory to the temporary git global config as a safe directory
[command]/usr/bin/git config --global --add safe.directory /home/runner/work/athena/athena
Removing SSH command configuration
[command]/usr/bin/git config --local --name-only --get-regexp core\.sshCommand
[command]/usr/bin/git submodule foreach --recursive sh -c "git config --local --name-only --get-regexp 'core\.sshCommand' && git config --local --unset-all 'core.sshCommand' || :"
Removing HTTP extra header
[command]/usr/bin/git config --local --name-only --get-regexp http\.https\:\/\/github\.com\/\.extraheader
[command]/usr/bin/git submodule foreach --recursive sh -c "git config --local --name-only --get-regexp 'http\.https\:\/\/github\.com\/\.extraheader' && git config --local --unset-all 'http.https://github.com/.extraheader' || :"
Removing includeIf entries pointing to credentials config files
[command]/usr/bin/git config --local --name-only --get-regexp ^includeIf\.gitdir:
[command]/usr/bin/git submodule foreach --recursive git config --local --show-origin --name-only --get-regexp remote.origin.url
Cleaning up orphan processes
##[warning]Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684. For more information see: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/
```
- ci-result: `not a check` passed (Check results for 8df5d6d6cb8df6e57a3910b43443653b051d8a51 come from the CI run https://github.com/doodla/athena/actions/runs/41710043053 (702s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test --workspace` and `cargo clippy --workspace --all-targets` pass, and `cargo fmt --check` is clean.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
