# Job 20261026-083650-jhde: Make tests independent of real free disk by setting the minimum to 0

## Request (verbatim)

> Priority: urgent
> The disk guard (9102f8f, default minimum 10 GB free) makes tests depend on the machine's real free space: athena-cli tests/binary.rs a_nested_run_works_in_a_scratch_copy_with_fake_agents failed in job 20261026-064950-38jx with 'environment: low disk: 9.0 GB free, 10.0 GB required', because the job's own build had used about 19 GB of the session's allowance. Tests must not depend on real free disk: let the minimum be set through the test fixtures' config (for example 0 in test configs) or inject the free-space reading, so the guard is still tested with simulated values and nested runs in tests are unaffected.
> Acceptance:
> - The nested-run test passes with free disk below 10 GB (shown by a test that simulates low free space for the nested run's environment and still passes because the test config sets the minimum to 0).
> - The disk guard's own tests still show a hold and a fail-fast with simulated low space.
> Files: crates/athena-core/src/disk.rs, crates/athena-cli/tests/common/mod.rs, crates/athena-cli/tests/binary.rs, CHANGELOG.md

## Reading (accepted by the user)

The shared test config (test_config in tests/common/mod.rs) gets [disk] min_free_gb = 0, so CLI tests, including the nested-run test, no longer depend on the machine's free space. A new test makes the nested run see low free space (a minimum above any real free space is not simulated; the reading is injected or the guard is exercised with simulated values) and shows it still passes. The disk guard's own tests keep showing a hold and a fail-fast with simulated values. The audience is the maintainers and verifier running cargo test on small-disk sandboxes.

Assumptions:
- Fixture::new writes test_config() as the fixture repo's athena.toml, and the nested scratch copy inherits it, so setting [disk] min_free_gb = 0 there reaches the nested run.
- Drain tests that exercise the hold already set their own minimum explicitly; they must keep doing so after the default becomes 0 (drain.rs may need a small edit if they rely on the default 10).
- Simulating low free space for the nested run is done by an injectable free-space reading in disk.rs (e.g. an env override read only when set) or equivalent; the production default of 10 GB is unchanged.
- athena.toml is not changed: the minimum is set in the test config only.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-cli --test binary`: a_nested_run_works_in_a_scratch_copy_with_fake_agents passes, and a companion test simulates low free space (below 10 GB) in the nested run's environment and still passes because the test config sets min_free_gb to 0.
2. [behavior] Run `cargo test --locked -p athena-core disk`: the disk guard's tests still show a hold (job stays queued, 'environment: low disk') and a fail-fast with simulated low free space.
3. [content] crates/athena-cli/tests/common/mod.rs test_config sets [disk] min_free_gb = 0 with a comment explaining why, and the production default (10 GB in athena-data config) is unchanged.
4. [behavior] Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`: all pass, and no test in the suite reads the machine's real free disk to decide its result.
5. [content] CHANGELOG.md has an entry stating that tests no longer depend on real free disk space and how the minimum or free-space reading is set in tests.

## Expected deliverables

- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Changing the production default minimum of 10 GB
- The code-health findings from job 20261026-064950-38jx (hooks test file, baseline file, table header width)
- Changing athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass; the guard stays tested with simulated values.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the nested-run test and the disk tests rather than checking that config lines exist.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The shared test config is fixed, not just the one failing test.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the four listed files plus drain.rs only if needed.
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

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test --locked -p athena-cli --test binary`: a_nested_run_works_in_a_scratch_copy_with_fake_agents passes, and a companion test simulates low free space (below 10 GB) in the nested run's environment and still passes because the test config sets min_free_gb to 0.
Run `cargo test --locked -p athena-core disk`: the disk guard's tests still show a hold (job stays queued, 'environment: low disk') and a fail-fast with simulated low free space.
crates/athena-cli/tests/common/mod.rs test_config sets [disk] min_free_gb = 0 with a comment explaining why, and the production default (10 GB in athena-data config) is unchanged.
Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`: all pass, and no test in the suite reads the machine's real free disk to decide its result.
CHANGELOG.md has an entry stating that tests no longer depend on real free disk space and how the minimum or free-space reading is set in tests.

## Report

End with the producer report as your structured output.
