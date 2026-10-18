# Job 20261018-073507-j8rs: Record which checks already fail on the base commit before producing

## Request (verbatim)

> Before the producer starts, run the job's checks on the base commit and record which ones already fail, as part of the job's records (part 1 of 2; part 2 uses them at verification). Keep it to recording: no change to how verification judges.

## Reading (accepted by the user)

Before the producer starts, Athena runs the capability's configured checks (fmt, clippy, test) on the base commit and writes a job record listing each check's name, pass/fail and exit code. Only recording is added. The verifier's judgment is unchanged, because part 2 will consume the record. The audience is the maintainer, who reads the job's records, and part 2's verifier.

Assumptions:
- The baseline reuses run_check from crates/athena-core/src/checks.rs on a snapshot of the base commit, with the capability's env and check_timeout_s, and stores the existing CheckRun shape (name, exit_code, passed, tests_ran, output tail, note).
- It is stored as part of the job's records (a new record kind or a field on an existing record), written with the job's other records; the exact schema is the producer's choice, following the pattern of the existing records in crates/athena-data/src/records.rs.
- It is recorded for every capability with configured checks (change and study), taken at the base commit before the first producer invocation; skip_when_only is not applied, since no change list exists yet.
- A baseline check that fails or times out is recorded as failing and never blocks or aborts the job. If the baseline cannot run at all, the job proceeds and the record says so.
- Nothing in verify.rs or routing.rs changes, and verdict logic is untouched. athena.toml and defaults.toml are not changed.
- Changing the records schema may require updating the docs and the record-validation or policy tests that enumerate record kinds.

## Acceptance criteria

1. [behavior] A test run by `cargo test --locked` drives a job with a fake or failing check on the base commit (for example `sh -c 'exit 1'`) and asserts that the job's records contain a baseline entry for that check with passed=false and its exit code, and passed=true for a check that succeeds.
2. [behavior] A test run by `cargo test --locked` shows the baseline is taken before the first producer invocation and from the base commit, not from the candidate. For example, the entry's SHA equals the job's base and its position precedes the producer's invocation record.
3. [behavior] A test run by `cargo test --locked` shows a baseline that fails, times out, or cannot start does not stop the job or change its verdict. The job continues and the record notes the failure.
4. [behavior] The existing verification tests still pass unchanged with no edits to the verdict logic. `git diff` shows no changes to crates/athena-core/src/verify.rs judgment code or to the routing of checks, and `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo fmt --all --check` all pass.
5. [content] The baseline record is part of the job's records: it lists each configured check by name with passed, exit code and a redacted output tail capped at the existing 4 KiB, and it is registered wherever record kinds are validated, so the records check accepts it.
6. [content] The docs or CHANGELOG state in a sentence or two that the baseline is recorded only and that part 2 will use it at verification.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)

## Out of scope

- Using the baseline at verification, or any change to how the verifier judges checks (part 2).
- Changing the checks themselves or their configuration in athena.toml.
- Fixing the checks that already fail on the base.
- Changing the producer's brief or the planner and critic.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is in Rust in the athena-core and athena-data crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests cover the new recording, and cargo test and clippy stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that a failing base check shows up as failed in the job's records, not only that a type exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Recording only; no consumption by the verifier (that is part 2).
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the design/CHANGELOG note are written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier can run.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a failing base check is recorded as failing.
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
