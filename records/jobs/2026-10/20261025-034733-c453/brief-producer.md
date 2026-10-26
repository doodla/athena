# Job 20261025-034733-c453: Run the test check with cargo-nextest to cut suite run time

## Request (verbatim)

> Priority: high
> Speed first, cut B (BRIEF.md addendum of 2026-10-25): make the test suite faster. Measured from the logs of 29 jobs: Athena's checks take a median 9.7 minutes (fmt, clippy and the full cargo test with about 1190 tests), the same suite runs again in CI, and producers spend a median 17.6 minutes, much of it building targeted tests. Measure the split between compiling and running, then cut the larger: for example adopting the pinned sccache (decision D79) for checks, producers and CI, sharing one target directory for Athena's checks where the gate's isolation allows, cargo nextest, or splitting slow integration tests. Report before and after medians.
> Acceptance:
> - docs/decisions.md records the compile and run split (measured) and the chosen change with alternatives.
> - The full suite still runs every test, and a test that fails still fails the check.
> - The median check time after the change is measured on jobs run after it.
> Files: athena.toml, crates/athena-core/src/run.rs, .github/workflows/ci.yml, xtask, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Goal: faster Athena checks, CI and producer test loops, judged by the user reading docs/decisions.md and later job times. Per the user's measurements (compile 67 s, run 673 s), adopt a pinned cargo-nextest for the test check, CI and producer guidance, with `cargo test --doc` run separately so every test still runs. If nextest can't be installed, fall back to splitting the slowest binaries.

Assumptions:
- Compile/run split numbers (67 s vs 673 s on 4 cores) are taken from the user's reply and recorded as measured by the brain.
- The test check in athena.toml (both change and study) becomes nextest plus a separate doctest step, which may need a check-list entry or a wrapper command.
- Producer and verifier command allow-lists need 'nextest' added as a cargo subcommand so the new check and guidance work.
- Pinning follows the existing [tools] pattern (e.g. gitleaks) by version and checksum; nextest is installed in CI and Athena's check environment the same way.
- After-change median cannot be measured in this job; the decision states how it will be measured on later jobs and claims no after number.
- Audience: the user and the later verifier reading docs/decisions.md, athena.toml and CI.

## Acceptance criteria

1. [content] docs/decisions.md has a new decision recording the measured split (compile 67 s via cargo test --no-run, run 673 s warm, 4 cores, measured by the brain, with the slowest binaries listed), the chosen change (cargo-nextest plus cargo test --doc), and at least three alternatives (sccache per D79, shared target dir, splitting slow tests) with reasons.
2. [content] The decision states the before medians (9.7 min checks, 17.6 min producers, 29 jobs) and how the after median will be measured on later jobs; it claims no unmeasured after number.
3. [content] athena.toml [tools] pins cargo-nextest by version and checksum like the other tools, and the test check (change and study) uses nextest plus a separate doctest run, with no test filtered or ignored.
4. [content] .github/workflows/ci.yml installs the pinned nextest and runs nextest and cargo test --doc, and CHANGELOG.md has an entry with the measured split.
5. [behavior] Run `cargo test --locked`: it passes clean with about 1190 tests, so a failing test still exits non-zero; the check definitions in athena.toml and CI are read to confirm a nonzero nextest or doctest exit fails the check.
6. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Expected deliverables

- docs/decisions.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- crates/athena-core/src/run.rs (modify)
- xtask/src/main.rs (modify)
- xtask/tests/policy.rs (modify)

## Out of scope

- Adopting sccache or a shared target directory
- Rewriting or deleting existing tests
- Changing the gate's isolation model
- Measuring the after-change median in this job

## User defaults that apply

- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Adopt cargo-nextest instead of building a parallel test runner; record alternatives.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision entry cites the measured split and labels the unmeasured after-median.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that every test still runs and failures still fail the check.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt stay clean.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The check is enforced in athena.toml and CI, not prose.
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

docs/decisions.md has a new decision recording the measured split (compile 67 s via cargo test --no-run, run 673 s warm, 4 cores, measured by the brain, with the slowest binaries listed), the chosen change (cargo-nextest plus cargo test --doc), and at least three alternatives (sccache per D79, shared target dir, splitting slow tests) with reasons.
The decision states the before medians (9.7 min checks, 17.6 min producers, 29 jobs) and how the after median will be measured on later jobs; it claims no unmeasured after number.
athena.toml [tools] pins cargo-nextest by version and checksum like the other tools, and the test check (change and study) uses nextest plus a separate doctest run, with no test filtered or ignored.
.github/workflows/ci.yml installs the pinned nextest and runs nextest and cargo test --doc, and CHANGELOG.md has an entry with the measured split.
Run `cargo test --locked`: it passes clean with about 1190 tests, so a failing test still exits non-zero; the check definitions in athena.toml and CI are read to confirm a nonzero nextest or doctest exit fails the check.
Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Report

End with the producer report as your structured output.
