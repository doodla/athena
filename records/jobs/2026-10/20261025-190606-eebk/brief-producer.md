# Job 20261025-190606-eebk: Cut slowest athena-cli test binaries' run time without weakening tests

## Request (verbatim)

> try again on 20261025-183644-t67q

## Reading (accepted by the user)

Retry of the test-suite speedup, following the user's reply: nextest is rejected (584 s vs 673 s, about 13%), so the job instead finds where lifecycle, job_plan, drain, retry and steward tests spend time and cuts it. Audience: the user, who reads docs/decisions.md and later job times.

Assumptions:
- The user's latest reply (do not adopt nextest; speed up the slowest binaries; target full cargo test under 6 minutes) overrides the earlier nextest readings.
- Test paths are inferred from the user's binary names; the producer finds the real files and shared helpers.
- The 6-minute target is measured by the producer on its own machine and reported; the verifier can only confirm tests pass.
- Request priority is high and speed first, but this is not marked urgent since it only asks for a speedup.

## Acceptance criteria

1. [content] docs/decisions.md has a new decision with per-binary before and after times (measured in the job, labeled as measured) for athena-cli tests/lifecycle.rs, job_plan.rs, drain.rs, retry.rs and steward.rs, and the cause found for each (sleeps, poll intervals, repeated setup, locks, or real git/process work).
2. [content] The decision records the compile 67 s versus run 673 s split (4 cores, measured by the brain) and lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as a rejected alternative with that reason; it also lists sccache per D79 and a shared target dir as considered alternatives, and claims no unmeasured number.
3. [content] No test is deleted, ignored, or has its assertions weakened; the test count under cargo test --locked is not lower than before, and no athena.toml, ci.yml or tool pin is changed. CHANGELOG.md has an entry.
4. [content] Speedups come from test-only mechanisms such as shorter test intervals, fixtures built once, or avoiding needless git/process work, and production defaults are unchanged unless a test-only override is added.
5. [behavior] Run `cargo test --locked`: it passes with every test still running, and the job's reported wall time is under 6 minutes on its machine. A test that fails still fails the run.
6. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-core/src/run.rs (modify)

## Out of scope

- Adopting cargo-nextest or any new pinned tool
- Changing athena.toml, ci.yml, or the test check
- Removing, ignoring, or weakening any test
- Adopting sccache or a shared target directory

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria require measured before/after times and a passing full suite, not just edited files.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision numbers must be measured in the job or labeled as brain-measured or estimates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must be clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to the test timing change, with no tool or CI changes.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria use cargo commands the verifier can run.
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

docs/decisions.md has a new decision with per-binary before and after times (measured in the job, labeled as measured) for athena-cli tests/lifecycle.rs, job_plan.rs, drain.rs, retry.rs and steward.rs, and the cause found for each (sleeps, poll intervals, repeated setup, locks, or real git/process work).
The decision records the compile 67 s versus run 673 s split (4 cores, measured by the brain) and lists cargo-nextest 0.9.148 (584 s vs 673 s, about 13%) as a rejected alternative with that reason; it also lists sccache per D79 and a shared target dir as considered alternatives, and claims no unmeasured number.
No test is deleted, ignored, or has its assertions weakened; the test count under cargo test --locked is not lower than before, and no athena.toml, ci.yml or tool pin is changed. CHANGELOG.md has an entry.
Speedups come from test-only mechanisms such as shorter test intervals, fixtures built once, or avoiding needless git/process work, and production defaults are unchanged unless a test-only override is added.
Run `cargo test --locked`: it passes with every test still running, and the job's reported wall time is under 6 minutes on its machine. A test that fails still fails the run.
Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Report

End with the producer report as your structured output.
