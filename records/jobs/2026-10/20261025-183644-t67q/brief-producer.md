# Job 20261025-183644-t67q: Run the test check through pinned cargo-nextest, visibly

## Request (verbatim)

> try again on 20261025-034733-c453

## Reading (accepted by the user)

Retry of the nextest speedup. Athena's `test` check, CI and producer guidance run the suite via a pinned cargo-nextest plus `cargo test --doc`. The check env gets ATHENA_HOME explicitly, and each test check records and prints which runner it used, so a fallback is never silent. The audience is the user, reading docs/decisions.md and later job times.

Assumptions:
- Per the user's reply, ATHENA_HOME = "{athena_home}" is added to both capability env tables so checks find the pinned tools directory.
- The wrapper prints one plain line naming its runner (nextest or cargo test fallback), and the check records it; a fallback is visible, not silent.
- Because the user's reply allows a visible fallback, the fallback stays but is announced; CI install fails loudly on download or checksum mismatch.
- The producer may be unable to fetch the real nextest version and sha256; if it can't, it must stop and report rather than invent a checksum.
- The files checks.rs, gh.rs and config.rs are needed for nextest output parsing and the tools schema, so they are deliverables.
- Measured numbers (compile 67 s, run 673 s, 4 cores, by the brain) go into the decision; no after number is claimed before it is measured on later jobs that used nextest.

## Acceptance criteria

1. [content] docs/decisions.md has a new decision recording the measured split (compile 67 s, run 673 s, 4 cores, measured by the brain, slowest binaries listed), the chosen change (pinned cargo-nextest plus cargo test --doc), and at least three alternatives (sccache per D79, shared target dir, splitting slow tests) with reasons.
2. [content] The decision states the before medians (9.7 min checks, 17.6 min producers, 29 jobs), says after-medians count only later jobs whose check used nextest, and claims no unmeasured after number.
3. [content] athena.toml [tools] pins cargo-nextest by version and a real checksum like the other tools; the test check for change and study uses nextest plus a separate doctest run with no test filtered or ignored; both capability env tables set ATHENA_HOME = "{athena_home}".
4. [content] .github/workflows/ci.yml installs the pinned nextest, fails loudly if download or checksum fails, and runs nextest and cargo test --doc under the one `test` check; CHANGELOG.md has an entry with the measured split.
5. [behavior] Run `cargo test --locked`: it passes. New unit tests show nextest summary counts and failing test names are parsed (checks.rs, gh.rs), libtest output still parses, a check environment built from the real athena.toml contains ATHENA_HOME, the wrapper's runner line names the runner used, and a nonzero nextest or doctest exit fails the check.
6. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Expected deliverables

- docs/decisions.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- xtask/src/main.rs (modify)
- xtask/tests/policy.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run.rs (modify)

## Out of scope

- Adopting sccache or a shared target directory
- Splitting slow integration test binaries (only as fallback if nextest cannot be installed)
- Filtering, ignoring or skipping any test
- Changing CI routing away from one `test` check

## User defaults that apply

- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Adopt cargo-nextest rather than build a parallel test runner.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for parsing, env and runner line; cargo test and clippy must be clean.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision cites measured split and labels after numbers as pending.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check parsing, exit failure and the ATHENA_HOME env by tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: After median measured on later jobs that used nextest.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Runner visibility and checksum pin enforced in code and tests.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in committed files.
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

docs/decisions.md has a new decision recording the measured split (compile 67 s, run 673 s, 4 cores, measured by the brain, slowest binaries listed), the chosen change (pinned cargo-nextest plus cargo test --doc), and at least three alternatives (sccache per D79, shared target dir, splitting slow tests) with reasons.
The decision states the before medians (9.7 min checks, 17.6 min producers, 29 jobs), says after-medians count only later jobs whose check used nextest, and claims no unmeasured after number.
athena.toml [tools] pins cargo-nextest by version and a real checksum like the other tools; the test check for change and study uses nextest plus a separate doctest run with no test filtered or ignored; both capability env tables set ATHENA_HOME = "{athena_home}".
.github/workflows/ci.yml installs the pinned nextest, fails loudly if download or checksum fails, and runs nextest and cargo test --doc under the one `test` check; CHANGELOG.md has an entry with the measured split.
Run `cargo test --locked`: it passes. New unit tests show nextest summary counts and failing test names are parsed (checks.rs, gh.rs), libtest output still parses, a check environment built from the real athena.toml contains ATHENA_HOME, the wrapper's runner line names the runner used, and a nonzero nextest or doctest exit fails the check.
Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Report

End with the producer report as your structured output.
