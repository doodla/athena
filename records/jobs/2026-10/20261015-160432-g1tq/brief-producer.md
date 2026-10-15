# Job 20261015-160432-g1tq: Reconcile invocation tokens with modelUsage, with end-to-end test

## Request (verbatim)

> try again on 20261015-145809-7565

## Reading (accepted by the user)

Retry of job 145809-7565: make the invocation's measured tokens and the gate's reconciliation rule use one definition (modelUsage), starting from the base because the earlier changes were discarded. This time add a test that runs a result event through parsing, the invocation record and check_invocation, plus a negative case. The audience is the Athena owner, who needs verified jobs to land.

Assumptions:
- The root cause is that the invocation's tokens come from the result event's top-level usage, while the gate compares them with modelUsage. The two differ when subagents run.
- The one definition is the sum of modelUsage over all models, stated in a doc comment on the usage field. If the producer finds the requested model's modelUsage fits the code better, that is acceptable if the doc comment, the invocation tokens and the gate agree.
- Older records in records/ that used several models may fail the new check. The producer should keep check_records tolerant of them, for example by comparing against the requested model's entry when the legacy shape applies, or by noting the limit. records/ is absent from the snapshot and is never changed.
- The earlier attempt's design was sound; the failure was the missing end-to-end test, so the retry repeats the fix and adds that test.
- The final report includes a table mapping each acceptance criterion to the named test or command that shows it is met.

## Acceptance criteria

1. [behavior] Run cargo test: a named end-to-end test feeds a result event through the stream parser and the claude driver (fake script) or the invocation constructor. Its top-level usage differs from modelUsage (main thread plus subagent usage on the requested model, using 90/52656/3634696/134508 against 6/3136/275219/3821). It then runs check_invocation on the resulting record and asserts there is no Reconciliation violation.
2. [behavior] Run cargo test: a named negative test builds the record from a result event whose totals really disagree with modelUsage, through the same path. It asserts that check_invocation returns a Reconciliation violation.
3. [content] One definition of an invocation's measured tokens is stated in a doc comment (on ResultEvent.usage and AgentOutput.usage in lib.rs). The invocation record and the gate's reconcile rule both use it.
4. [content] The job 2fsa numbers (90/52656/3634696/134508 and 6/3136/275219/3821) appear in a regression test fixture or comment, and CHANGELOG.md records the fix.
5. [behavior] Run cargo test, cargo clippy --all-targets and cargo fmt --check: all pass and are clean, including the existing stream, claude and landing tests.

## Expected deliverables

- crates/athena-agent/src/stream.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Adding a pre-handover criterion-check mechanism to the job runner (already tried for this failure class).
- Editing records/ or migrating older records.
- Changing governance files such as athena.toml or defaults.toml.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Align the recorded tokens and the gate on one definition rather than loosening the check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require tests that run a result event through to check_invocation, not tests that only construct pieces.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy --all-targets and fmt --check must be clean.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change on the usage path, with the changelog.
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
