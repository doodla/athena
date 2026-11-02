# Job 20261031-172040-kq9v: verify "Cache gitleaks and the athena binary across CI runs"

## Request (verbatim)

> Priority: high
> Stop installing the same tools on every CI runner
> 
> In GitHub run 41221582236 (31 Oct 16:50) every fmt, clippy and test-shard job spent 65 to 105 seconds in the step 'Set up (pinned toolchain, gitleaks, athena)' before its check, nine times per run. Cache what that step installs (gitleaks and the athena binary built for the run) with actions/cache keyed on the inputs that decide them, so a warm runner restores instead of reinstalling. Keep actions pinned by SHA and caches saved only from the default branch, as ci.yml does today.
> 
> The producer cannot run GitHub Actions; acceptance is checked from the files and local tests. The real effect is measured by a later throughput review.
> 
> Acceptance:
> - ci.yml restores and (on the default branch only) saves a cache for what the setup step installs, keyed on the files that decide it, in every check job.
> - setup.sh --ci skips an install whose cached copy is present and matches.
> - An xtask test reads ci.yml and fails if a check job runs the setup step without the restore step before it.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: .github/workflows/ci.yml, setup.sh, xtask/tests/policy.rs, CHANGELOG.md

## Reading (accepted by the user)

Goal: cut the 65 to 105 s the 'Set up' step spends in each fmt, clippy and test-shard job, for the CI maintainers and the later throughput review. Add actions/cache restore and default-branch-only save steps for gitleaks and the athena binary, make `setup.sh --ci` skip a matching cached install, and add an xtask policy test that enforces the restore-before-setup order.

Assumptions:
- Code survey: I read .github/workflows/ci.yml, setup.sh and xtask/src/main.rs (install_gitleaks, build_and_adopt). setup.sh ends with `exec cargo run ... -p xtask -- setup "$@"`. xtask setup runs prerequisites, `cargo fetch`, install_gitleaks, a sccache step that is skipped under --ci, then build_and_adopt, which builds the athena-cli package (crates/athena-cli) in release mode and copies it to the athena home bin/athena. It also symlinks ~/.local/bin/athena. No crates/ record or data file is added, so check_record_file and AnyRecord are not touched.
- The install logic lives in xtask/src/main.rs, not in setup.sh. So the skip behaviour for 'setup.sh --ci skips an install whose cached copy is present and matches' is probably implemented there, and xtask/src/main.rs is listed as a likely deliverable. setup.sh may need only a small change or none, and the request's file list is not exhaustive.
- install_gitleaks already returns early when the pinned version is present and `gitleaks version` matches. The restored cache therefore only needs to put the binary at paths::tool_path. build_and_adopt always runs cargo build and embeds ATHENA_BUILD_COMMIT, so the new --ci skip must be keyed on the inputs of the binary (Cargo.lock, rust-toolchain.toml, athena.toml, and the sources under crates/ and xtask/). It must also still link ~/.local/bin/athena, which the history, attribution and pr-head jobs run.
- Cache keys: gitleaks keyed on hashFiles('athena.toml') plus runner OS and architecture, because the pin lives there. The athena binary is keyed on hashFiles of rust-toolchain.toml, Cargo.lock, Cargo.toml, crates/**/*.rs and xtask/**/*.rs, with no restore-keys fallback, so a stale binary is never used.
- Restore steps go before the 'Set up' step in every job that runs it: fmt, clippy, the test shards, history, attribution and pr-head. Save steps keep the existing guard (not a pull request and the default branch). Test shards save from shard 0 only, as the existing caches do. All actions stay pinned to the existing actions/cache SHA. The real speed-up is not measurable here, so the throughput review measures it later.
- The policy test follows the existing job_text/job_steps/step_index helpers in xtask/tests/policy.rs. CHANGELOG.md gets one entry. athena.toml, defaults.toml and setup.sh's pins are not edited.

## Acceptance criteria

1. [content] Grep of .github/workflows/ci.yml: in each of the fmt, clippy and test jobs, an actions/cache/restore step for gitleaks and one for the athena binary come before the step named 'Set up (pinned toolchain, gitleaks, athena)'. Each key uses hashFiles over the files that decide the install. Every `uses:` line is still pinned by a 40-character SHA.
2. [content] ci.yml has matching actions/cache/save steps for those two caches in the same jobs. Each carries the existing condition: not a pull request, ref equal to the default branch, and no exact cache hit. Neither cache is saved from any other event.
3. [content] The history, attribution and pr-head jobs that also run the setup step get the same restore step before it, or the diff states in a comment why they do not.
4. [behavior] With a cached gitleaks and athena binary present and matching, `./setup.sh --ci` skips both installs and prints that it skipped them. Without them it installs as before. Run `cargo test -p xtask`, which must include a test that exercises the skip and no-skip decisions.
5. [behavior] The new test in xtask/tests/policy.rs reads ci.yml and passes on the changed file. A test that feeds it a ci.yml variant with the restore step removed or placed after the setup step makes it fail. Run `cargo test -p xtask --test policy`.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass, including xtask/tests/ci_shards.rs and the existing policy tests.
7. [content] CHANGELOG.md has a new entry describing the gitleaks and athena binary caches and their keys, and stating that the speed-up is not measured here.

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/tests/policy.rs (modify)
- CHANGELOG.md (modify)
- xtask/src/main.rs (modify)
- setup.sh (modify) (a governance file: allowed because the reading lists it)
- xtask/tests/ci_shards.rs (modify)

## Out of scope

- Measuring the real time saved on GitHub Actions; the later throughput review does that.
- Changing the toolchain, registry or target caches, the gate and pass logic, or the shard layout.
- Changing athena.toml, defaults.toml, tool pins or action SHAs.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The skip logic and the policy test are Rust in xtask; setup.sh stays a thin shell shim.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The skip decision and the ci.yml policy are covered by tests, and fmt, clippy -D warnings and cargo test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The restore-before-setup rule is enforced by an xtask policy test, not by prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that the skip happens and that the policy test fails on a bad ci.yml, not only that steps exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the CI caching path.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check, and reads ci.yml.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that a warm runner restores instead of reinstalling, checked through the ci.yml keys and the skip tests. The real time saved is left to the throughput review.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: ci.yml comments and the CHANGELOG entry explain the keys for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base a32c91d73762 → candidate b30ae177864c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml | 121 +++++++++++++++++++++++++++++++++
 CHANGELOG.md             |  11 +++
 xtask/src/main.rs        | 169 ++++++++++++++++++++++++++++++++++++++++++++---
 xtask/tests/policy.rs    |  99 ++++++++++++++++++++++++++-
 4 files changed, 390 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1991 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test` all pass, including xtask/tests/ci_shards.rs and the existing policy tests.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
