# Job 20261025-055624-ab95: Skip 'fmt, clippy, tests' CI when the governed fingerprint already

## Request (verbatim)

> Priority: high
> Speed first, cut A part 1 (job 20261025-034731-2am0 timed out as one job): make the 'fmt, clippy, tests' CI job reuse its own earlier pass for unchanged code. Add an xtask command that prints the governed-content fingerprint of the checked-out tree (reusing athena-gate's fingerprint, which ignores records/ and owner paths). In .github/workflows/ci.yml, before setup, look the fingerprint up in actions/cache; on a hit of a passing entry, report it and skip the build and checks; otherwise run the full job and save the entry only after it passes. GitHub scopes caches by branch, so entries saved on a pull-request branch serve later runs on that branch, and main's shared caches stay written only from main. Keep the job name 'fmt, clippy, tests'.
> Acceptance:
> - An xtask test shows the fingerprint unchanged by a records-only or owner-path change and changed by a governed change.
> - The workflow saves an entry only after a full pass, and a missing or failed entry runs the full job (shown by a test of the step script or the workflow logic).
> - docs/decisions.md records the mechanism, the measured landing median before (15.4 minutes over 33 jobs, 2026-10-23 to 25) and the alternatives.
> Files: xtask/src/main.rs, .github/workflows/ci.yml, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Add `xtask fingerprint` (printing athena-git's governed-content fingerprint of HEAD) and make the 'fmt, clippy, tests' job look that fingerprint up in actions/cache before setup. A hit skips setup and checks; a miss runs everything and saves a marker only after a pass. Audience: the owner and the verifier judging CI speed and soundness.

Assumptions:
- Part A1 only: no change to athena land or gh.rs and no early push of the candidate; the after-measurement is deferred.
- The fingerprint is computed from the checked-out HEAD with rules from that same commit, as athena-git::attrs::fingerprint(git, HEAD, HEAD) does.
- xtask currently depends only on athena-data and policy forbids other edges; reusing athena-git needs an xtask->athena-git edge, so docs/architecture.md and xtask/Cargo.toml are updated to allow it. This is the one governance-adjacent change.
- The step script logic is testable by putting it in xtask (e.g. a subcommand deciding hit/miss and writing the marker) so the workflow only calls it.
- Cache key is like ci-pass-<fingerprint>; GitHub's branch scoping gives the described PR/main behavior, and the pass entry is saved by a step that runs only on success (not default-branch-only).
- The workflow runs xtask with the runner's preinstalled cargo before setup.sh; no change to athena.toml or defaults.toml.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test builds a temporary git repo and shows the printed fingerprint is unchanged by a records-only change and by an owner-path change, and changes after a governed change.
2. [behavior] Run `cargo test -p xtask`: a test of the step script/workflow logic shows a missing entry or a failed run leads to the full job and no save, a hit of a passing entry skips the build and checks, and the save happens only after the checks pass.
3. [content] .github/workflows/ci.yml keeps the job name 'fmt, clippy, tests'; the cache lookup (actions/cache/restore, key from the xtask fingerprint) comes before 'Set up'; setup, the checks and the existing cache steps are skipped on a hit; the new save step runs only when the checks succeeded.
4. [content] Saving is done on pull-request branches for their own fingerprint entries, while the shared toolchain/registry/target caches are still saved only from the default branch; the entry is written under a key that includes the fingerprint and a marker whose presence means 'passed' (a failed run never writes one).
5. [content] The hit step reports the reused result in the log and step summary (fingerprint and matched key), so a reader can see why the build was skipped.
6. [content] The fingerprint comes from athena-git's `fingerprint` (not a reimplementation); any dependency or architecture-doc change this needs keeps `dependency_direction_matches_the_architecture` passing.
7. [content] docs/decisions.md has an entry giving the mechanism, the measured landing median before (15.4 minutes over 33 jobs, 2026-10-23 to 25), the alternatives considered, and that the after-figure is not yet measured; CHANGELOG.md has a matching entry.
8. [behavior] Run `cargo test` and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: all pass.

## Expected deliverables

- xtask/src/main.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/architecture.md (modify)
- xtask/tests/fingerprint.rs (create)
- xtask/tests/policy.rs (modify)

## Out of scope

- Changing athena land or crates/athena-core/src/gh.rs, or pushing the candidate early for CI.
- Caching for the 'the full history' job or other jobs.
- Measuring the landing time after the change.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fingerprint command, script logic and tests are all Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the fingerprint and the hit/miss/save logic; cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing reuse and rerun behavior, not just wiring.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: decisions.md cites the measured 15.4 min over 33 jobs and labels the after-figure unmeasured.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Uses actions/cache and the existing athena-git fingerprint.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only part A1; early push is left out.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Pass-only save is enforced by step conditions and tested.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that unchanged governed content skips the job while governed changes run it.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

Run `cargo test -p xtask`: a test builds a temporary git repo and shows the printed fingerprint is unchanged by a records-only change and by an owner-path change, and changes after a governed change.
Run `cargo test -p xtask`: a test of the step script/workflow logic shows a missing entry or a failed run leads to the full job and no save, a hit of a passing entry skips the build and checks, and the save happens only after the checks pass.
.github/workflows/ci.yml keeps the job name 'fmt, clippy, tests'; the cache lookup (actions/cache/restore, key from the xtask fingerprint) comes before 'Set up'; setup, the checks and the existing cache steps are skipped on a hit; the new save step runs only when the checks succeeded.
Saving is done on pull-request branches for their own fingerprint entries, while the shared toolchain/registry/target caches are still saved only from the default branch; the entry is written under a key that includes the fingerprint and a marker whose presence means 'passed' (a failed run never writes one).
The hit step reports the reused result in the log and step summary (fingerprint and matched key), so a reader can see why the build was skipped.
The fingerprint comes from athena-git's `fingerprint` (not a reimplementation); any dependency or architecture-doc change this needs keeps `dependency_direction_matches_the_architecture` passing.
docs/decisions.md has an entry giving the mechanism, the measured landing median before (15.4 minutes over 33 jobs, 2026-10-23 to 25), the alternatives considered, and that the after-figure is not yet measured; CHANGELOG.md has a matching entry.
Run `cargo test` and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: all pass.

## Report

End with the producer report as your structured output.
