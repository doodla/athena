# Job 20261102-102740-qx57: verify "Cache gitleaks and athena binary in CI with complete input keys"

## Request (verbatim)

> try again on 20261031-172040-kq9v

## Reading (accepted by the user)

Retry of job 20261031-172040-kq9v: redo the CI caches for gitleaks and the athena binary, with the athena-bin key and the setup --ci skip digest covering every compiled-in file, derived from one shared list that a repo test checks. Audience: CI maintainers and the throughput review.

Assumptions:
- Job 20261031-172040-kq9v is rejected; its candidate is discarded and this job starts from the base. Its files .github/workflows/ci.yml, CHANGELOG.md, xtask/src/main.rs and xtask/tests/policy.rs are kept as deliverables, redone from scratch. xtask/tests/ci_shards.rs and setup.sh are dropped unless a test there needs updating (setup.sh's --ci path calls xtask setup, so it is likely unchanged).
- Findings of 20261031-172040-kq9v addressed: (major) the key and skip digest omitted prompts/*.md, docs/system-facts.md and other include_str! inputs, so a restored binary could be stale. Fixed by hashing crates/** and xtask/** plus rust-toolchain.toml, Cargo.toml, Cargo.lock, athena.toml, defaults.toml, setup.sh and docs/system-facts.md, in the key and in the digest. (minor) no test tied the hashFiles list to input_files: now a policy test compares them. (minor) stamp written only on --ci builds: now written on every build, or removed on a non-CI build. (minor) restored binary reports the older source commit: stated in the ci.yml comment and CHANGELOG, accepted since the CI jobs only run read-only checks. (nit) skip path bypassed the dirty-tree refusal: the skip applies only with --ci, and this is documented.
- Code survey: I read xtask/src/main.rs (main dispatches to setup(); setup runs numbered steps, step 4 is sccache, skipped under --ci; the build and adopt step is later), crates/athena-core/src/status.rs (BUILD_PATHS = crates/, xtask/, Cargo.toml, Cargo.lock, setup.sh), and the claim that crates/athena-context/src/lib.rs and briefs.rs include prompts and docs/system-facts.md with include_str!. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The property (a compiled-in file change changes the key and the digest) is measured directly: the test hashes a temp tree, edits one file, and compares digests, and it parses the hashFiles globs from ci.yml.
- The shared list is one const in xtask (the athena build input globs). The digest function and a policy test both read it, and the test parses every athena-bin hashFiles in ci.yml and fails if any input path is not covered by a glob.
- The test enumerates compiled-in files independently by scanning include_str!/include_bytes! targets under crates/ and xtask/ plus the manifests, so a new compiled-in file outside the list fails the test.
- The gitleaks cache and the restore-before-setup ordering test from the original request are kept. Restore steps come before 'Set up (pinned toolchain, gitleaks, athena)' in fmt, clippy, test, history, attribution and pr-head jobs. Saves run on the default branch only, as today. All uses: lines stay pinned by a 40-character SHA.
- No rule is made stricter on readings, so no refusal-rate criterion applies.

## Acceptance criteria

1. [content] Grep of .github/workflows/ci.yml: every athena-bin cache key (all restore and save steps) uses hashFiles covering crates/**, xtask/**, rust-toolchain.toml, Cargo.toml, Cargo.lock, athena.toml, defaults.toml, setup.sh and docs/system-facts.md. In each of the fmt, clippy and test jobs, actions/cache/restore steps for gitleaks and the athena binary come before the step named 'Set up (pinned toolchain, gitleaks, athena)'.
2. [content] ci.yml has matching actions/cache/save steps with the existing condition (not a pull request, ref equal to the default branch, no exact cache hit). Every uses: line is pinned by a 40-character SHA. History, attribution and pr-head also restore before setup, or a comment states why they do not. No other workflow behavior changes.
3. [content] xtask has one shared list of athena build inputs that both the skip digest and the policy test read. The comment in ci.yml and the CHANGELOG do not claim more than the key covers.
4. [behavior] Run `cargo test -p xtask`. A test edits one compiled-in file (a prompt .md under a temp copy of the tree) and asserts the inputs digest changes. Another asserts the skip happens with a matching digest and cached binary and does not happen with a mismatched digest or no binary, and a third asserts the stamp is not left stale after a non-CI build.
5. [behavior] Run `cargo test -p xtask --test policy`. The new tests derive the compiled-in file set from include_str!/include_bytes! targets and the manifests, and fail if any athena-bin hashFiles key in ci.yml misses one. They also fail on a ci.yml variant with a restore step removed or placed after the setup step.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass, including xtask/tests/ci_shards.rs and the existing policy tests.
7. [content] CHANGELOG.md has a new entry describing the gitleaks and athena binary caches, their keys, and that the speed-up on GitHub Actions is not measured here.

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/src/main.rs (modify)
- xtask/tests/policy.rs (modify)
- CHANGELOG.md (modify)
- setup.sh (modify) (a governance file: allowed because the reading lists it)
- xtask/tests/ci_shards.rs (modify)

## Out of scope

- Measuring the real time saved on GitHub Actions; the later throughput review does that.
- Changing the toolchain, registry or target caches, the gate and pass logic, or the shard layout.
- Changing athena.toml, defaults.toml, tool pins or action SHAs (they are only hashed, not edited).
- Changing the build commit that a restored binary reports.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New xtask tests; cargo test and clippy -D warnings must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A policy test enforces key coverage and restore-before-setup order.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The key and digest derive from one shared list, not a patched list.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests edit a compiled-in file and check the digest changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the cargo commands.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments and the CHANGELOG are written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A policy test reads the real ci.yml. It requires every athena-bin step (all restore and save steps) to use a hashFiles list equal to the `xtask build-inputs` output, which is the nine globs. In fmt, clippy and test it requires the gitleaks and athena-bin restore steps to precede the step named 'Set up (pinned toolchain, gitleaks, athena)'. The verifier may also read `git diff -- .github/workflows/ci.yml`. Run: `cargo test -p xtask --test policy`. Observes: athena_bin_keys_cover_every_compiled_in_file and every_job_restores_gitleaks_and_athena_before_setup pass on the real ci.yml.
  Fails without the behavior: A key missing docs/system-facts.md or another glob, or a restore placed after setup or deleted, fails these tests. The mutant test in criterion 5 shows it.
  Test seam: Entry points: the pure functions `check_athena_bin_keys(yml, required)` and `check_restore_before_setup(yml)` in policy.rs, and the `xtask` binary via CARGO_BIN_EXE_xtask (the pattern already in xtask/tests/fingerprint.rs). Fixture: the real .github/workflows/ci.yml. No double. Step 7 adds the check functions and step 3 adds the subcommand; neither exists today.
- Criterion 2: Policy tests check three things. Every athena-bin and gitleaks save step is actions/cache/save, guarded by the default-branch condition and `cache-hit != 'true'`, and in the test job also by `matrix.shard == 0`. Every `uses:` line ends in @ plus 40 hex characters. history, attribution and pr-head restore before setup. The existing pass-marker test and the ci_shards tests show no other workflow behavior changed. The verifier reads the ci.yml comment for attribution and pr-head, which states that they restore only. Run: `cargo test -p xtask --test policy`. Observes: The new save-guard and SHA-pin tests pass, as do the existing the_pass_is_marked_and_saved_only_after_the_checks_and_the_shared_caches_stay_default_branch_only and the ci_shards tests.
  Fails without the behavior: A save step with no default-branch guard, an unpinned or short-SHA uses: line, or a missing restore in history, attribution or pr-head fails the new tests.
  Test seam: Entry points: `job_steps_in(text, id)`, `check_restore_before_setup`, and a save-guard check over the real ci.yml text in policy.rs. Fixtures: the real ci.yml and string mutants of it. No double. Step 7 adds the text-taking forms; today only the file-reading `job_text` and `job_steps` exist.
- Criterion 3: Run the subcommand and compare its output with the hashFiles(...) arguments in ci.yml. The policy test asserts set equality between the two. `inputs_digest` in xtask/src/main.rs reads the same const. The verifier reads the ci.yml comment and the CHANGELOG entry with git diff. They claim only the nine globs and say that docs/design, CHANGELOG.md, README.md and other docs are not in the key. Run: `cargo run -p xtask -- build-inputs`. Observes: It prints the nine globs, one per line, matching the ci.yml key. The comment and the CHANGELOG entry claim nothing beyond them.
  Fails without the behavior: If the digest used its own list or the key drifted from the const, the set-equality assertion in athena_bin_keys_cover_every_compiled_in_file fails.
  Test seam: Entry point: the `build-inputs` dispatch in main() printing `BUILD_INPUT_GLOBS`. Step 3 adds it; it does not exist today.
- Criterion 4: Unit tests in xtask/src/main.rs run the real functions over temp directories (tempfile is already an xtask dependency). (a) `inputs_digest_changes_when_a_compiled_in_file_is_edited` builds a temp tree with a file for every glob, including crates/athena-context/prompts/producer.md and docs/system-facts.md. It takes `inputs_digest(root)`, edits the prompt, and asserts the digest differs. It does the same for docs/system-facts.md and for a new file under crates/, and asserts that a CHANGELOG.md edit leaves the digest unchanged. (b) `skip_only_with_ci_matching_stamp_and_binary` calls `can_skip_build(ci, home, digest)`. It skips only with ci true, a binary file and a stamp equal to the digest. It does not skip with a different digest, no binary, no stamp, or ci false. (c) `a_non_ci_build_leaves_no_stale_stamp` calls `rebuild_stamped(home, digest, build)` with a stamp holding the old digest D1. The build closure asserts the stamp is already gone. Afterwards the stamp holds D2 and `can_skip_build(true, home, D1)` is false. A second case has the closure return Err and asserts no stamp remains. Run: `cargo test -p xtask`. Observes: All three tests pass among the xtask unit tests.
  Fails without the behavior: If the digest skipped any glob, (a) fails because the digest stays equal. Without the digest comparison, (b) fails because a mismatched digest or a missing binary would skip. If the stamp were not deleted before the rebuild, (c) fails because the closure sees the old stamp, and a failed build would leave it vouching for the old binary.
  Test seam: Entry points: `inputs_digest`, `can_skip_build` and `rebuild_stamped` in xtask/src/main.rs, which `build_and_adopt` calls. Fixtures: temp home and temp source tree. Injection: the cargo build is the closure parameter of `rebuild_stamped`, and the other collaborators are passed as arguments. The digest, comparison and stamp logic run for real; only the cargo build is replaced. Steps 2 to 5 add this seam; today `build_and_adopt` inlines the build.
- Criterion 5: `athena_bin_keys_cover_every_compiled_in_file` runs `xtask build-inputs`. It scans crates/ and xtask/ .rs files for include_str!/include_bytes! literals and keeps production includes (the rule is in the decisions). It adds the manifests and setup.sh. It asserts that every such path is covered by a glob in every athena-bin hashFiles of the real ci.yml, that the key globs equal the xtask list, that the set is non-empty, and that the prompt and docs/system-facts.md sanity paths are present. `a_stale_or_incomplete_ci_yml_is_flagged` runs `check_athena_bin_keys` on a copy of ci.yml with docs/system-facts.md removed from one key and on a copy with another glob removed. It also runs it on the real ci.yml with an extra required path crates/x/prompts/new.md. It runs `check_restore_before_setup` on a copy with the athena-bin restore deleted from fmt and on a copy with the restore moved after 'Set up'. Every mutant must return Err and the real file Ok. Run: `cargo test -p xtask --test policy`. Observes: The real-file tests pass and every mutated ci.yml variant is rejected.
  Fails without the behavior: Without key coverage, a key missing docs/system-facts.md or a prompt glob is accepted and the mutant test fails. Without the order check, a restore placed after setup or deleted is accepted.
  Test seam: Entry points: the xtask binary (CARGO_BIN_EXE_xtask) and `check_athena_bin_keys` / `check_restore_before_setup` in policy.rs. Fixtures: the real ci.yml and string mutations of it, passed as arguments. The scan reads the real repository files and no double holds the logic. Steps 3 and 7 add the subcommand and the functions; neither exists today.
- Criterion 6: Run the three Athena check commands on the finished tree. `cargo test --locked` runs xtask/tests/ci_shards.rs and the existing and new policy tests. The existing `parse_workflow` and `check_jobs` in ci_shards.rs parse the edited ci.yml, and the new unit tests in main.rs and policy.rs run too. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The ci_shards tests still accept the edited ci.yml: shard layout, job ids, timeouts and needs are unchanged.
  Fails without the behavior: A malformed or reordered new ci.yml step makes ci_shards.rs or policy.rs fail in `cargo test`. Unformatted code fails `cargo fmt --check`. A lint in the new main.rs code fails clippy -D warnings. A broken new test fails `cargo test`. Each leg fails on a defect in the change.
  Test seam: Entry points: the three cargo commands themselves, the same ones Athena runs (fmt, clippy, test). Fixtures: the real edited repository and the real .github/workflows/ci.yml, read by the existing `parse_workflow` / `check_jobs` in xtask/tests/ci_shards.rs (exists, read). No fake or double. Step 10 runs them.
- Criterion 7: Read the diff of CHANGELOG.md. The new entry sits under [Unreleased]. It names the gitleaks cache and the athena-bin cache with their paths and keys, and lists the globs. It says that `setup --ci` skips the build only with a matching stamp, and that a restored binary reports its older source commit. It says that pull requests changing crates/ or xtask/ always rebuild. It says the speed-up on GitHub Actions is not measured here. Existing policy tests that read CHANGELOG.md then run. Run: `git diff -- CHANGELOG.md`. Observes: The entry exists, is under [Unreleased], and states those facts. `cargo test -p xtask --test policy` still passes.
  Fails without the behavior: Without the entry, the diff is empty and the content check fails. A CHANGELOG format break fails the existing policy tests that read the file.
  Test seam: Entry point: the CHANGELOG.md file under [Unreleased]; the existing CHANGELOG-reading policy tests are the guard. Fixture: the real file. No double. Step 9 adds the entry.

Planned files:

- xtask/src/main.rs (modify)
- xtask/tests/policy.rs (modify)
- .github/workflows/ci.yml (modify)
- CHANGELOG.md (modify)
- setup.sh (modify)
- xtask/tests/ci_shards.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 791eb9e5e5b2 → candidate 010e46a0730e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml | 130 +++++++++++++++
 CHANGELOG.md             |  20 +++
 xtask/src/main.rs        | 286 +++++++++++++++++++++++++++++++--
 xtask/tests/policy.rs    | 403 ++++++++++++++++++++++++++++++++++++++++++++++-
 4 files changed, 827 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1996 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass, including xtask/tests/ci_shards.rs and the existing policy tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
