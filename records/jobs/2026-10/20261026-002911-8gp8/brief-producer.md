# Job 20261026-002911-8gp8: Hold jobs at dispatch on low disk; fail fast before checks

## Request (verbatim)

> try again on 20261017-001109-qfax

## Reading (accepted by the user)

Retry of 20261017-001109-qfax, following the user's latest decision. Before dispatch, check free disk against a minimum (default 10G in config.rs). If low, clean closed and idle workspaces' target/. If still low, leave the job queued with 'environment: low disk' and the free and required amounts, and do not dispatch or reject it. Before the checks, low space gives a clear, retryable environment failure, not a hold, so nothing mid-run needs resuming. Audience: the user running drain.

Assumptions:
- The user's decision overrides the 'proposed next request': the free-disk hold happens only at dispatch, and a job is never left Dispatched with a hold marker, so no mid-run resume route is needed.
- A held job stays in its queued (Proposed) state and a later drain picks up the same job when space is free; no new job is created.
- Before the checks, low space after cleanup fails the job with a retryable environment reason, not a rejection; the existing retry route applies.
- Free space is read through a small injectable function so tests can simulate low and recovered disk.
- athena.toml is not edited. The 10G default lives in config.rs and applies when the setting is absent. If the config digest covers the new field, it is reset as the repository requires.
- The deletion of a closed job's target/ already landed in 173d12e and is not redone. The idle sweep reuses the existing remover and job-lock helpers, and skips locked workspaces.
- The user-facing text says 'the user', never 'operator'.

## Acceptance criteria

1. [behavior] With free disk simulated below the minimum, a test shows cleanup of closed and idle workspaces' target/ runs before dispatch and before the checks. Run `cargo test --locked`.
2. [behavior] A test shows that when space is still low after cleanup at dispatch, the job is not dispatched, stays queued (not rejected) with status 'environment: low disk', and the free and required amounts appear in status and drain output. Run `cargo test --locked`.
3. [behavior] A drain test through the real step mapping, not only a stub, shows a low-disk hold is not counted as a failure. Once the fake reports enough space, a later drain runs the same job (same id) to a verdict. Run `cargo test --locked`.
4. [behavior] A test shows that when space is low before the checks and still low after cleanup, the job fails with a clear retryable environment reason, is not rejected, and no hold marker is left. When space is freed, verification goes ahead. Run `cargo test --locked`.
5. [behavior] A test shows the idle-workspace sweep removes target/, leaves source files, and skips workspaces whose job lock is held. Run `cargo test --locked`.
6. [content] config.rs defines a 10G minimum-free-disk default used when the setting is absent, and a test shows it parses. athena.toml is unchanged. README and CHANGELOG name the setting and the 'environment: low disk' status and say 'the user', never 'operator'.
7. [behavior] `cargo clippy --all-targets --locked -- -D warnings` is clean and `cargo fmt --all --check` passes.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/disk.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Editing athena.toml
- Mid-run holds and a resume route for a job left Dispatched
- Re-running job 20261016-044847-hem1
- Deleting a closed job's target/, which landed in 173d12e
- Changing `athena retry` semantics

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New disk tests are added; cargo test, clippy and fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check hold, cleanup order and the later run through tests that simulate disk.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and messages are plain and name the setting and status.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The mid-run hold is dropped by the user's decision, so the change is small.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The minimum is enforced in code before dispatch and before the checks, not by prose.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests assert the outcome (held, not rejected, later runs), not only that the code exists.
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

With free disk simulated below the minimum, a test shows cleanup of closed and idle workspaces' target/ runs before dispatch and before the checks. Run `cargo test --locked`.
A test shows that when space is still low after cleanup at dispatch, the job is not dispatched, stays queued (not rejected) with status 'environment: low disk', and the free and required amounts appear in status and drain output. Run `cargo test --locked`.
A drain test through the real step mapping, not only a stub, shows a low-disk hold is not counted as a failure. Once the fake reports enough space, a later drain runs the same job (same id) to a verdict. Run `cargo test --locked`.
A test shows that when space is low before the checks and still low after cleanup, the job fails with a clear retryable environment reason, is not rejected, and no hold marker is left. When space is freed, verification goes ahead. Run `cargo test --locked`.
A test shows the idle-workspace sweep removes target/, leaves source files, and skips workspaces whose job lock is held. Run `cargo test --locked`.
config.rs defines a 10G minimum-free-disk default used when the setting is absent, and a test shows it parses. athena.toml is unchanged. README and CHANGELOG name the setting and the 'environment: low disk' status and say 'the user', never 'operator'.
`cargo clippy --all-targets --locked -- -D warnings` is clean and `cargo fmt --all --check` passes.

## Report

End with the producer report as your structured output.
