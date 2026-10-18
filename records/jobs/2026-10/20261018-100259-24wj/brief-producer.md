# Job 20261018-100259-24wj: Make land --pr wait for failed runs, rerun once, name failing test

## Request (verbatim)

> try again on 20261018-095724-dqht

## Reading (accepted by the user)

Retry of the failed job: in land_pr's bounded check wait, when checks go Red, keep waiting until the failed workflow runs complete, rerun failed jobs once via GitHub's rerun-failed-jobs endpoint, wait again, then merge if green or report the failing test's name if red. Audience: the operator who wants unattended landings.

Assumptions:
- The retry lives inside land_pr's check wait in crates/athena-core/src/gh.rs and reuses HostOperation::ReadChecks and the existing host-call retry and error-log path.
- The merge gate is unchanged.
- The total wait across the first wait, the rerun and the second wait stays within a bound (about twice the existing 30 looks of 20 s).
- Rerun staleness is detected with run_attempt from the actions/runs reply, with a look allowance as fallback.
- GitHub refuses rerun-failed-jobs on a run still in progress, so the code waits for completion first.
- Docs go in docs/design/native-pr-landing.md and CHANGELOG.md.
- The file list is carried from the earlier attempt; I did not re-read the repository.

## Acceptance criteria

1. [behavior] A FakeHost test where the first read is Red while the run is still in progress shows no rerun is requested until the run completes, then exactly one rerun. Total reads stay within 2 x attempts. The test fails without the wait-for-completion behavior. Run `cargo test -p athena-core gh`.
2. [behavior] A test with the read sequence [Red, Red, Pending, Red] asserts exact counts: first wait 1 read, second wait 3 reads, 1 rerun, no second rerun. A stale run_attempt (unchanged after the rerun) is treated as not yet restarted. Run `cargo test -p athena-core gh`.
3. [behavior] A test where the rerun turns green shows land_pr reports Green and merges when merge_via_rest is on. Run `cargo test -p athena-core gh`.
4. [behavior] A test where the second result is Red shows no merge, no further rerun, and the PrReport carries the failing test's name. Checks still pending after the bound give Pending with no rerun and no merge. Run `cargo test -p athena-core gh`.
5. [content] display::land_pr and the JSON report name the failing test and say the failed job was re-run once. The failure message names the test whether or not a rerun happened.
6. [content] GhClient calls the rerun-failed-jobs endpoint through the existing HostOperation retry and redaction path. A parser test on a sample payload covers the failing test name. The plan or docs list, for each new GitHub call, the state it requires and what GitHub returns when that state is missing.
7. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass with no warnings.
8. [content] docs/design/native-pr-landing.md and CHANGELOG.md state the wait, the single rerun, the completion precondition and the failing-test report.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/external_calls.rs (modify)

## Out of scope

- Changing the merge gate
- Retrying more than once
- Changing the Pages rerun behavior
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests plus cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check reruns, merges and reported names by test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG entry for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the premature-Red-before-run-complete cause, not one instance.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing host-call path, not new machinery.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change inside land_pr's check wait.
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

## Report

End with the producer report as your structured output.
