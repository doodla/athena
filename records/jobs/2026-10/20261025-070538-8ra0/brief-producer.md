# Job 20261025-070538-8ra0: Open the job's PR early at verification start so CI caches during verify

## Request (verbatim)

> try again on 20261025-055626-er96

## Reading (accepted by the user)

Retry of the early-push job. When verification starts in athena go, push the candidate and open or update the job's PR through the host seam, with no merge, so CI fills the fingerprint cache in the PR's scope. Failures are non-fatal; a missing or non-GitHub origin skips silently so the CLI tests stay green; land reuses the PR and sets link.head to the landing commit after the replace.

Assumptions:
- Goal: faster landings for the maintainer; the judge is the verifier running cargo tests.
- Per the user's decision, a missing or non-GitHub origin skips silently; only real host errors on a GitHub remote warn.
- GhEarly builds GhCli itself; land_pr runs replace_early_head before the remote_head read, or sets link.head to the landing commit.
- The CLI tests need no edits because the silent skip keeps them green.
- Dependency 20261025-055624-ab95 (fingerprint cache) is already in the repository.

## Acceptance criteria

1. [behavior] A fake-host test shows that when verification starts in Athena::go, the candidate is pushed to the job's PR branch and the PR is opened or updated via Remote::push and PrHost::open_pr before verification completes, with no merge call. Run: cargo test -p athena-core.
2. [behavior] A fake-host test shows that a failing early push or early open_pr leaves the job's outcome identical to the no-failure case (job does not fail; land proceeds as today). Run: cargo test -p athena-core.
3. [behavior] A test shows land still pushes the landing commit to the same branch, reuses the early PR (no second open_pr), uses the lease-pinned replace through check_replace, sets link.head to the landing commit after the replace and before the head check, and merges guarded with the landing commit (merge_heads == [L]). Run: cargo test -p athena-core.
4. [behavior] A test runs go with no origin or a non-GitHub origin and shows the early step is skipped silently: no output line contains 'warning'. Only a host error on a GitHub remote prints a warning. Run: cargo test --workspace.
5. [behavior] cargo test --workspace (including the existing CLI lifecycle tests, with no new 'warning' line), cargo clippy --workspace --all-targets, and cargo fmt --check all pass clean. Each new test fails without the new behavior.
6. [content] The production early-push implementation builds its own GhCli runner, so the Athena::go signature and CLI call sites are unchanged.
7. [content] CHANGELOG.md says verification start opens or updates the job's PR so CI caches by fingerprint, that it never merges, that a missing or non-GitHub origin is skipped quietly, and that failures are non-fatal.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Changing how merge works or when it happens
- Changing the fingerprint cache from 20261025-055624-ab95
- Changing the Athena::go signature or CLI call sites

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New fake-host tests plus workspace test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check pushes, PR opens, silent skip and non-fatal failure by test.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The silent skip is decided in the early step itself, not patched in the tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
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

A fake-host test shows that when verification starts in Athena::go, the candidate is pushed to the job's PR branch and the PR is opened or updated via Remote::push and PrHost::open_pr before verification completes, with no merge call. Run: cargo test -p athena-core.
A fake-host test shows that a failing early push or early open_pr leaves the job's outcome identical to the no-failure case (job does not fail; land proceeds as today). Run: cargo test -p athena-core.
A test shows land still pushes the landing commit to the same branch, reuses the early PR (no second open_pr), uses the lease-pinned replace through check_replace, sets link.head to the landing commit after the replace and before the head check, and merges guarded with the landing commit (merge_heads == [L]). Run: cargo test -p athena-core.
A test runs go with no origin or a non-GitHub origin and shows the early step is skipped silently: no output line contains 'warning'. Only a host error on a GitHub remote prints a warning. Run: cargo test --workspace.
cargo test --workspace (including the existing CLI lifecycle tests, with no new 'warning' line), cargo clippy --workspace --all-targets, and cargo fmt --check all pass clean. Each new test fails without the new behavior.
The production early-push implementation builds its own GhCli runner, so the Athena::go signature and CLI call sites are unchanged.
CHANGELOG.md says verification start opens or updates the job's PR so CI caches by fingerprint, that it never merges, that a missing or non-GitHub origin is skipped quietly, and that failures are non-fatal.

## Report

End with the producer report as your structured output.
