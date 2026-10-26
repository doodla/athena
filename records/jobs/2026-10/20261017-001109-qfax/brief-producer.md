# Job 20261017-001109-qfax: Hold jobs on low disk after cleanup; default 10G minimum free

## Request (verbatim)

> Delete the cargo target/ build caches of a job workspace once the job reaches a terminal state, and sweep idle workspaces too. Before a job or a verification starts, check free disk space against a configured minimum, with a default of 10G. If space is low, run the cleanup first. If space is still low, hold the job with an explicit 'environment: low disk' status and tell the operator; do not reject it. Add tests: with low disk simulated, the cleanup runs, the job is held rather than rejected, and verification goes ahead once space is freed. Then re-run the request of job 20261016-044847-hem1.

## Reading (accepted by the user)

Add a minimum-free-disk check (default 10G, a default in config.rs, athena.toml untouched) before dispatch and before the checks. On low space it runs cleanup of terminal and idle workspaces' target/; if still low, it holds the job as 'environment: low disk' and tells the user, without rejecting it. Audience: the user running drain. Re-running job 20261016-044847-hem1 is a separate follow-up after this lands.

Assumptions:
- Deleting a closed job's target/ already landed in 173d12e; this change reuses that cleanup and adds the idle-workspace sweep if absent.
- The minimum is a config.rs default applying when the setting is absent; athena.toml is not edited.
- Free space is injectable (a trait or function parameter) so tests can simulate low disk.
- Re-running job 20261016-044847-hem1 is done by Athena after landing, not as an edit here.
- Docs and output say 'the user', never 'the operator'.
- The exact file layout is a guess; likely homes are config.rs, drain.rs and run.rs.

## Acceptance criteria

1. [behavior] With free disk simulated below the minimum, a test shows cleanup runs before the job is dispatched and before the checks. Run `cargo test --locked` and check the new disk tests pass.
2. [behavior] A test shows that when space is still low after cleanup, the job is held with status 'environment: low disk', is not rejected, and the user is told, with free and required amounts. Run `cargo test --locked`.
3. [behavior] A test shows that once space is freed, verification goes ahead for the held job. Run `cargo test --locked`.
4. [behavior] A test shows that the idle-workspace sweep removes target/ and leaves source files. Run `cargo test --locked`.
5. [content] config.rs defines a minimum-free-disk default of 10G used when the setting is absent, and a test shows the default parses. athena.toml is unchanged. The docs name the setting and the 'environment: low disk' status.
6. [content] The user can see from drain output and status that a held job shows the reason and the free and required amounts. Wording says 'the user', not 'the operator'.
7. [behavior] `cargo clippy --all-targets --locked -- -D warnings` is clean and `cargo fmt --all --check` passes.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/disk.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Editing athena.toml
- Re-implementing deletion of a closed job's target/ (landed in 173d12e)
- Editing the planning pass of job 20261016-044847-hem1 in this change

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New disk tests, with cargo test, clippy and fmt required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name tests that exercise cleanup, hold and resume.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs written for an outside reader, using 'the user'.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; the hem1 re-run is separate.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is done before dispatch and before the checks, which is the cause of the full-disk failures.
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

With free disk simulated below the minimum, a test shows cleanup runs before the job is dispatched and before the checks. Run `cargo test --locked` and check the new disk tests pass.
A test shows that when space is still low after cleanup, the job is held with status 'environment: low disk', is not rejected, and the user is told, with free and required amounts. Run `cargo test --locked`.
A test shows that once space is freed, verification goes ahead for the held job. Run `cargo test --locked`.
A test shows that the idle-workspace sweep removes target/ and leaves source files. Run `cargo test --locked`.
config.rs defines a minimum-free-disk default of 10G used when the setting is absent, and a test shows the default parses. athena.toml is unchanged. The docs name the setting and the 'environment: low disk' status.
The user can see from drain output and status that a held job shows the reason and the free and required amounts. Wording says 'the user', not 'the operator'.
`cargo clippy --all-targets --locked -- -D warnings` is clean and `cargo fmt --all --check` passes.

## Report

End with the producer report as your structured output.
