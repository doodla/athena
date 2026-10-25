# Job 20261024-191951-4pqw: Add `athena check --pr-head` pre-merge check and non-blocking CI job

## Request (verbatim)

> Build job 4, part 1, of the records-history plan (docs/design/records-history.md sections 5.1 and 10, decision D111): a pre-merge check on a pull request's head, as a command (for example athena check --pr-head <base> <head>) that passes only when every commit between the base and the head classifies (via land.rs pr_commits) as a job landing whose verification passed and whose prediction precedes its outcome, a declared intervention, or records-only, and the head's content equals the last job landing's verified content plus only records-only commits after it; otherwise it fails naming each offending commit. Add it to .github/workflows/ci.yml as a new job on pull requests that does not block merging yet (it becomes required at the cutover, job 7), and list .github/workflows/ci.yml as a governance deliverable. Tests: a head with an unverified commit fails; a head shaped like a real Athena pull request passes; a head with a declared intervention after the job landing fails, since a squash of it is not a valid landing until the cutover.

## Reading (accepted by the user)

Adds a command (`athena check --pr-head <base> <head>`) that classifies every commit in base..head with `land::pr_commits` and passes only if each is a verified job landing with prediction before outcome, a declared intervention, or records-only, and the head's content equals the last job landing's verified content plus records-only commits. A new non-required pull_request job in ci.yml runs it. The audience is maintainers and CI reviewing Athena PRs before the job-7 cutover.

Assumptions:
- The command is a new `--pr-head <base> <head>` option on the existing `Check` subcommand (conflicting with --history), since `athena check` already exists; a separate subcommand is acceptable if clap makes that cleaner.
- The core logic lives in athena-core (land.rs or a small new module) so tests can build git fixtures with the existing land.rs test fixtures.
- 'Prediction precedes outcome' is checked within the head's tree/history using the records the job landing carries, per design section 5.1/10; the producer reads the design doc for the exact rule.
- The CI job is simply not added to branch protection; nothing in athena.toml or defaults.toml changes.
- Goal: a pre-merge gate for Athena PRs; audience is the maintainer and CI reading pass/fail output.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test builds a PR range containing a commit that changes governed content with no verification or intervention, and the pr-head check fails and names that commit's sha.
2. [behavior] Run `cargo test`: a test builds a head shaped like a real Athena PR (job landing with verification, prediction before outcome, optional records-only commits after) and the check passes.
3. [behavior] Run `cargo test`: a test with a declared-intervention commit after the job landing fails, naming that commit and saying a squash of it is not a valid landing until the cutover.
4. [behavior] Run `cargo test`: a test shows a job landing whose prediction does not precede its outcome, and a head whose content differs from the last landing's verified content plus records-only commits, each fail naming the offending commit; every offending commit is listed, not only the first.
5. [content] `athena check --pr-head <base> <head>` exists in the CLI, uses `land::pr_commits` for classification, exits non-zero on failure and zero on pass, and the existing `check` and `check --history` behavior is unchanged.
6. [content] .github/workflows/ci.yml gains a new job that runs only on pull_request, passes the PR base and head shas (with full history fetched) to the command, and is commented as not yet required (becomes required at the cutover, job 7). Existing jobs are unchanged.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean; `cargo test` passes fully.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/pr_check.rs (create)
- crates/athena-cli/src/display.rs (modify)
- README.md (modify)

## Out of scope

- Making the new CI job a required status (job 7 cutover).
- Records-branch ordering checks or changes to how landings are written (job 4 part 2 and later).
- Changing pr_commits classification rules.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Three requested tests plus prediction-order and content-mismatch cases; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the check on built PR shapes and assert pass/fail and named commits.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a CI job and command, marked not-yet-required.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command, one CI job, no changes to pr_commits.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Error messages name commits and the reason; changelog entry written for outside readers.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
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

Run `cargo test`: a test builds a PR range containing a commit that changes governed content with no verification or intervention, and the pr-head check fails and names that commit's sha.
Run `cargo test`: a test builds a head shaped like a real Athena PR (job landing with verification, prediction before outcome, optional records-only commits after) and the check passes.
Run `cargo test`: a test with a declared-intervention commit after the job landing fails, naming that commit and saying a squash of it is not a valid landing until the cutover.
Run `cargo test`: a test shows a job landing whose prediction does not precede its outcome, and a head whose content differs from the last landing's verified content plus records-only commits, each fail naming the offending commit; every offending commit is listed, not only the first.
`athena check --pr-head <base> <head>` exists in the CLI, uses `land::pr_commits` for classification, exits non-zero on failure and zero on pass, and the existing `check` and `check --history` behavior is unchanged.
.github/workflows/ci.yml gains a new job that runs only on pull_request, passes the PR base and head shas (with full history fetched) to the command, and is commented as not yet required (becomes required at the cutover, job 7). Existing jobs are unchanged.
Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean; `cargo test` passes fully.

## Report

End with the producer report as your structured output.
