# Job 20261031-085321-w7hj: verify "Name the final CI job "fmt, clippy, tests" to satisfy branch protection"

## Request (verbatim)

> Priority: urgent
> Give the final CI job the required check's name, "fmt, clippy, tests"
> 
> Since 20261031-001537-0rz6 split CI, the old job "fmt, clippy, tests" no longer exists; its result now comes from the final job named `ci`. Branch protection on main still requires a check named "fmt, clippy, tests", so pull request #314 cannot merge ("Required status check "fmt, clippy, tests" is expected", HTTP 405), and no landing to main can merge until this is fixed.
> 
> Change only the display name of the final job in .github/workflows/ci.yml from `ci` to `fmt, clippy, tests`, keeping its job id, its needs, its `if: always()` and its steps. Update anything in the repository that looks for the job by its display name (for example xtask/tests/policy.rs and xtask/tests/ci_shards.rs) and the CHANGELOG entry of 0rz6.
> 
> Acceptance:
> - In .github/workflows/ci.yml, the job that needs gate, fmt, clippy and test and runs with `if: always()` has `name: fmt, clippy, tests`, and no other job has that name.
> - A test in xtask asserts that exactly one job carries the name "fmt, clippy, tests" and that it needs every check job; it fails on the current file.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: .github/workflows/ci.yml, xtask/tests/ci_shards.rs, xtask/tests/policy.rs, CHANGELOG.md

## Reading (accepted by the user)

Follow-up to job 20261031-001537-0rz6: change only the display name of the final job (id `ci`) in ci.yml to `fmt, clippy, tests`, so the required check exists again and PR #314 can merge. Update the ci_shards.rs check that expects `name: ci`, add a test that exactly one job has that name and needs every check job, and amend the 0rz6 CHANGELOG entry. Audience: the maintainer and branch protection.

Assumptions:
- Job 20261031-001537-0rz6 (landed, pass, 4 findings) is addressed thus: the EARLY_RUN_JOB finding (crates/athena-core/src/gh/early.rs:25 names `fmt, clippy, tests`) no longer applies once the final job carries that name again, so early.rs and its tests stay unchanged. The extra xtask files finding and the oracle and cache findings are unrelated to the rename and stay as landed.
- Files of 0rz6's candidate: ci.yml and CHANGELOG.md are kept as deliverables here; xtask/tests/ci_shards.rs and xtask/tests/policy.rs are kept as deliverables; docs/design/ci-routing.md is a possible small deliverable (it says branch protection requires `ci`); xtask/Cargo.toml, xtask/src/main.rs, xtask/src/shard.rs, xtask/src/ci_cache.rs, xtask/tests/fingerprint.rs and Cargo.lock stay as landed and are excluded.
- Code survey: crates/athena-core/src/gh/early.rs:25 holds EARLY_RUN_JOB = "fmt, clippy, tests" and crates/athena-core/src/gh/tests.rs uses that name; I read both by Grep and they already match the new name, so no change is needed there. check_jobs in xtask/tests/ci_shards.rs line 467 asserts `    name: ci` and must change; policy.rs line 350 only has a negative sample string `name: ci` and is likely unaffected.
- The job id stays `ci`; only the name line at ci.yml line 401 changes. The workflow-level `name: ci` at line 37 is a different thing and is left alone. Keep needs, `if: always()` and steps.
- No records, no athena.toml or defaults.toml change. ci.yml lives under .github/workflows/ and is listed as a deliverable.

## Acceptance criteria

1. [content] In .github/workflows/ci.yml the job with id `ci` (needs gate, fmt, clippy, test; `if: always()`; same steps) has the line `    name: fmt, clippy, tests`, and Grep finds no other job with that name.
2. [content] xtask/tests/ci_shards.rs contains a test that parses ci.yml and asserts that exactly one job is named `fmt, clippy, tests` and that its `needs` lists every check job (gate, fmt, clippy, test); it would fail on the previous file, where the final job is named `ci`. The old `name: ci` assertion in check_jobs is replaced.
3. [behavior] Run `cargo test -p xtask`: the new test and all existing ci_shards and policy tests pass, including the mutation tests that drop `if: always()` or a need.
4. [behavior] Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`; both are clean.
5. [content] CHANGELOG.md amends the 0rz6 entry: the final job is named `fmt, clippy, tests` (job id `ci`), branch protection needs no change, and the EARLY_RUN_JOB known effect is removed or marked resolved. A new entry notes the rename and the reason (PR #314 405 error).
6. [content] The diff changes the display name only: job id, needs, `if: always()`, steps and every other job in ci.yml are unchanged (check with git diff).

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/tests/ci_shards.rs (modify)
- CHANGELOG.md (modify)
- xtask/tests/policy.rs (modify)
- docs/design/ci-routing.md (modify)

## Out of scope

- Changing branch protection settings or the job id `ci`
- crates/athena-core/src/gh/early.rs and its tests (the name matches again)
- The shard split, xtask/src/shard.rs, main.rs, ci_cache.rs, fingerprint.rs, xtask/Cargo.toml and Cargo.lock from 0rz6 (kept as landed)
- athena.toml, defaults.toml, setup.sh, rust-toolchain.toml
- Reading CI results back into a job

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New xtask test plus clippy and the full test suite pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test asserts the job name and its needs from the real file, not that wiring exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused rename; no other CI change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The required check name is enforced by a test on ci.yml.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the name mismatch with branch protection itself, not by editing protection settings.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: The new test reads the real ci.yml and requires the job with id `ci` to carry `    name: fmt, clippy, tests` and exactly one job block to have that name. The verifier also reads the diff of ci.yml. Run: `cargo test -p xtask --test ci_shards -- workflow_required_check_name`. Observes: The test passes and `git diff -- .github/workflows/ci.yml` shows `name: ci` replaced by `name: fmt, clippy, tests` on the `ci` job; the workflow-level `name: ci` on line 37 is unchanged.
- Criterion 2: Run the new test, which parses the real ci.yml with `jobs()`. Its mutation cases feed altered text to `check_required_name`: name reverted to `ci`, name copied onto the `pass` job, `fmt` dropped from needs. Each must return Err. Run: `cargo test -p xtask --test ci_shards -- workflow_required_check_name`. Observes: On the real file the test passes: exactly one job has the name, its id is `ci`, and its needs include gate, fmt, clippy and test. Each mutated text is rejected. Run against the base ci.yml, the same test fails.
  Fails without the behavior: If ci.yml still had `name: ci`, the helper finds zero jobs named `fmt, clippy, tests` and the test fails. If another job took the name, the count is 2 and the duplicate mutation case fails.
  Test seam: Production entry points: ci_shards.rs `jobs(&ci_yml())` and the new `check_required_name(text: &str) -> Result<(), String>`. Fixture: the real .github/workflows/ci.yml read via CARGO_MANIFEST_DIR; mutations are string replacements on that text in the test. `jobs()` and `ci_yml()` exist today (read); `check_required_name` is added in step 2.
- Criterion 3: Run the whole xtask test package against the edited ci.yml. Run: `cargo test -p xtask`. Observes: All ci_shards, policy and fingerprint tests pass, including workflow_jobs_reject_a_missing_gate_condition_need_or_timeout (drop `if: always()`, drop a need, drop a timeout).
  Fails without the behavior: If the rename broke check_jobs, or the helper were wrongly strict (for example requiring `pass`), workflow_jobs_are_separate_sharded_and_gated_by_ci would fail. If a dropped `if: always()` or need were no longer rejected, the mutation test would fail.
  Test seam: Entry point is check_jobs(text) in xtask/tests/ci_shards.rs, called on ci_yml() and on mutated copies. Fixture: the real ci.yml, with string-replacement mutations in the test. Both exist today; step 2 only changes the name assertion inside check_jobs.
- Criterion 4: Run the lint and format checks over the whole workspace including test targets, so the new helper and test in xtask/tests/ci_shards.rs are compiled and linted. Then run `cargo fmt --all --check` as a second invocation. Run: `cargo clippy --workspace --all-targets --locked -- -D warnings`. Observes: Clippy exits 0 with no warnings in the xtask ci_shards test target; `cargo fmt --all --check` exits 0 with no diff.
  Fails without the behavior: If the new test code had a clippy warning (an unused helper, a needless borrow, a redundant clone), `-D warnings` turns it into a non-zero exit; if the code were not rustfmt-formatted, `cargo fmt --all --check` prints a diff and exits non-zero.
  Test seam: Production entry points are the cargo clippy and cargo fmt commands Athena itself runs on the result. The fixture is the edited source tree, with xtask/tests/ci_shards.rs as the changed target; there is no injected double. The seam exists today (commands and the xtask test target are present); the code it checks is added in steps 2 and 3.
- Criterion 5: Read the diff. The 0rz6 entry under Added says the final job is named `fmt, clippy, tests` (job id `ci`) and that branch protection needs no change. Known effect (1) is marked resolved for the name, with the `ci`-before-`pass` marker caveat kept. A new Fixed entry names the rename and the reason, PR #314 refused with HTTP 405 for the missing check. Run: `git diff -- CHANGELOG.md docs/design/ci-routing.md`. Observes: The diff has the amended Added bullet and the new Fixed bullet, and ci-routing.md section 4.2 says branch protection requires `fmt, clippy, tests` (job id `ci`).
- Criterion 6: Read the git diff of ci.yml. Run: `git diff -- .github/workflows/ci.yml`. Observes: Exactly one changed line, the name line of the `ci` job. The job id, `needs: [gate, fmt, clippy, test]`, `if: always()`, the steps and every other job are untouched.

Planned files:

- .github/workflows/ci.yml (modify)
- xtask/tests/ci_shards.rs (modify)
- CHANGELOG.md (modify)
- docs/design/ci-routing.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d84169633b44 → candidate f57ab834f5f1. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml  |  2 +-
 CHANGELOG.md              | 23 +++++++++-----
 docs/design/ci-routing.md | 14 ++++-----
 xtask/tests/ci_shards.rs  | 76 +++++++++++++++++++++++++++++++++++++++++++++--
 4 files changed, 96 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1811 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
