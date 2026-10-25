# Job 20261025-055626-er96: Open the job's PR early so CI runs during verification

## Request (verbatim)

> Priority: high
> Speed first, cut A part 2: when verification of a job starts, athena land pushes the verified candidate to the job's pull-request branch so CI's 'fmt, clippy, tests' run for that code finishes (and is cached by its fingerprint, from 20261025-055624-ab95) while verification runs, and the landing run on the same branch reuses it. Depends on 20261025-055624-ab95. A push failure must not fail the job; land behaves as today.
> Acceptance:
> - A test with the fake host shows the early push through the host seam when verification starts, and that a push failure leaves the job's outcome unchanged.
> - The landing still pushes the job's landing commit to the same branch.
> Files: crates/athena-core/src/gh.rs, crates/athena-core/src/run.rs, CHANGELOG.md

## Reading (accepted by the user)

When verification starts, athena go pushes the candidate to the job's PR branch and opens (or updates) the job's pull request through the host seam (Remote::push, PrHost::open_pr), without merging. This way the pull_request CI run fills the fingerprint cache in that PR's scope while verification runs. Any failure is logged and ignored. The landing push and merge stay as today and reuse the same branch and PR. The audience is the maintainer who wants faster landings.

Assumptions:
- Job 20261025-055624-ab95 (the CI fingerprint cache) is already landed.
- The user's reply takes precedence: the early step opens or updates the PR, not just pushes the branch.
- The early step runs in the athena go flow when verification begins, and reuses the same branch name and PR lookup that land uses.
- No workflow, athena.toml or defaults.toml change is needed.

## Acceptance criteria

1. [behavior] A fake-host test shows that when verification starts, the candidate is pushed to the job's PR branch and the job's PR is opened or updated through the host seam (Remote::push and PrHost::open_pr), before verification completes. Run: cargo test -p athena-core.
2. [behavior] A fake-host test shows that when the early push or the early PR open fails, the job's outcome is identical to the no-failure case: the job does not fail and land proceeds as today. Run: cargo test -p athena-core.
3. [behavior] A fake-host test shows that the early step never calls merge. Merging happens only in athena land after verification passes.
4. [behavior] A test shows that the landing still pushes the job's landing commit to the same branch and reuses the early-opened PR, without opening a second one. Run: cargo test -p athena-core.
5. [behavior] cargo test, cargo clippy and cargo fmt --check all pass with no warnings.
6. [content] CHANGELOG.md has an entry saying that verification start opens or updates the job's PR so CI caches by fingerprint, and that failures are non-fatal.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-core/src/pr.rs (modify)

## Out of scope

- Changing ci.yml or the xtask fingerprint
- Merging the PR before verification passes
- Changing how land merges or waits for checks

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-host tests cover the early push, the failure path and the landing.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the push and PR open happening and the outcome unchanged on failure.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the host seam and the run flow.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
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

A fake-host test shows that when verification starts, the candidate is pushed to the job's PR branch and the job's PR is opened or updated through the host seam (Remote::push and PrHost::open_pr), before verification completes. Run: cargo test -p athena-core.
A fake-host test shows that when the early push or the early PR open fails, the job's outcome is identical to the no-failure case: the job does not fail and land proceeds as today. Run: cargo test -p athena-core.
A fake-host test shows that the early step never calls merge. Merging happens only in athena land after verification passes.
A test shows that the landing still pushes the job's landing commit to the same branch and reuses the early-opened PR, without opening a second one. Run: cargo test -p athena-core.
cargo test, cargo clippy and cargo fmt --check all pass with no warnings.
CHANGELOG.md has an entry saying that verification start opens or updates the job's PR so CI caches by fingerprint, and that failures are non-fatal.

## Report

End with the producer report as your structured output.
