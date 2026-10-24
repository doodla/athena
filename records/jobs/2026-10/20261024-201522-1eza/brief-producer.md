# Job 20261024-201522-1eza: Add `athena check --attribution` records-only attribution check

## Request (verbatim)

> Build job 5, part 1, of the records-history plan (docs/design/records-history.md sections 5.2, 6 and 10, decision D111): an attribution check that reads only athena/records, never commit-message trailers. It walks main's first-parent commits from the commit the first landing-intent record expects as its base onward, and requires each to be named as host_commit by a landing record whose head tree equals that commit's tree and whose expected base is its parent; a commit with a matching landing-intent but no landing record yet is pending for 15 minutes from the intent and fails after; any other commit fails as unattributed. Commits before that base are out of scope here (the old history check and, later, the legacy rule judge them). Expose it as a command (for example athena check --attribution) and add a non-blocking CI job on pushes to main and on schedule, beside the existing full-history check, listing .github/workflows/ci.yml as a governance deliverable. Tests: a properly landed squash passes; an unrecorded commit fails naming it; a pending one passes within 15 minutes and fails after; a records-branch commit that is not a fast-forward is reported.

## Reading (accepted by the user)

Add a check reading only the athena/records branch (never commit trailers) that walks main's first-parent commits from the first landing-intent's expected base and requires each to be attributed by a matching landing record, with a 15-minute pending window. Expose it as `athena check --attribution` and add a non-blocking CI job that explicitly fetches athena/records. Audience: the maintainer and CI.

Assumptions:
- Records data models (landing-intent, landing, host_commit, expected_base, head tree) already exist in athena-data/athena-git from earlier jobs.
- The check time is injectable so tests control the 15-minute window.
- A missing athena/records ref or no landing-intent is an explicit failure/error, not silent success.
- CHANGELOG.md gets an entry if the repo keeps one per change.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test with a properly landed squash (landing-intent plus landing whose host_commit is the commit, head tree equal to the commit's tree, expected base equal to its parent) passes the attribution check.
2. [behavior] Run `cargo test`: a main commit with no landing record and no matching intent makes the check fail, and the report names that commit's sha as unattributed.
3. [behavior] Run `cargo test`: a commit with a matching landing-intent and no landing passes when the check time is within 15 minutes of the intent's creation time, and fails after 15 minutes.
4. [behavior] Run `cargo test`: a records-branch history containing a commit that is not a fast-forward of its predecessor is reported by the check.
5. [behavior] Run `cargo run -- check --help`: `--attribution` is listed, and combining it with --history or --pr-head is rejected.
6. [content] The check reads only athena/records records and main's first-parent graph; nothing parses commit-message trailers; commits before the first intent's expected base are not judged. Code has doc comments citing records-history.md sections 5.2, 6, 10 and D111.
7. [content] .github/workflows/ci.yml has a new attribution job beside the history job: continue-on-error true, runs only on push to main and schedule, uses full-history checkout, explicitly fetches athena/records (e.g. `git fetch origin athena/records:athena/records`) before running `athena check --attribution`, pins actions by SHA like existing jobs, and is not a required check. The command reads the branch from the local or remote-tracking ref that was fetched.
8. [behavior] Run `cargo clippy`, `cargo fmt --check` and `cargo test`: all clean and passing for the workspace.

## Expected deliverables

- crates/athena-gate/src/attribution.rs (create)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Legacy-rule judging of commits before the base
- Changes to the existing full-history check
- Making the attribution job a required check
- Job 5 part 2 and later records-history jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for pass, unrecorded, pending, and non-fast-forward cases; clippy and fmt clean.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust within existing crates.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by CI job and command exit status.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests and the command rather than checking file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change.
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

Run `cargo test`: a test with a properly landed squash (landing-intent plus landing whose host_commit is the commit, head tree equal to the commit's tree, expected base equal to its parent) passes the attribution check.
Run `cargo test`: a main commit with no landing record and no matching intent makes the check fail, and the report names that commit's sha as unattributed.
Run `cargo test`: a commit with a matching landing-intent and no landing passes when the check time is within 15 minutes of the intent's creation time, and fails after 15 minutes.
Run `cargo test`: a records-branch history containing a commit that is not a fast-forward of its predecessor is reported by the check.
Run `cargo run -- check --help`: `--attribution` is listed, and combining it with --history or --pr-head is rejected.
The check reads only athena/records records and main's first-parent graph; nothing parses commit-message trailers; commits before the first intent's expected base are not judged. Code has doc comments citing records-history.md sections 5.2, 6, 10 and D111.
.github/workflows/ci.yml has a new attribution job beside the history job: continue-on-error true, runs only on push to main and schedule, uses full-history checkout, explicitly fetches athena/records (e.g. `git fetch origin athena/records:athena/records`) before running `athena check --attribution`, pins actions by SHA like existing jobs, and is not a required check. The command reads the branch from the local or remote-tracking ref that was fetched.
Run `cargo clippy`, `cargo fmt --check` and `cargo test`: all clean and passing for the workspace.

## Report

End with the producer report as your structured output.
