# Job 20261023-234149-jcd2: Tell verifiers to reuse recorded fmt, clippy and test results

## Request (verbatim)

> try again on 20261019-013309-nvbb

## Reading (accepted by the user)

Retry the failed job narrowly: edit the verifier prompt so verifiers cite Athena's recorded fmt, clippy and test results rather than rerunning the full suite, running only targeted tests for new behavior, and note it in the changelog. No code changes; the producer must avoid repeated full-suite runs.

Assumptions:
- Goal: Athena's verifiers (and the operator reading verdicts) stop rerunning the full suite and reuse recorded fmt/clippy/test results.
- Per the user's reply, scope stays prompt-only plus changelog; the watchdog/resume proposal is not part of this retry.
- Producer should run only cargo test -p athena-context while editing and the full suite at most once at the end (or not at all).

## Acceptance criteria

1. [content] crates/athena-context/prompts/verifier.md tells the verifier that the fmt, clippy and test results in the 'Checks Athena ran on this tree' section are recorded for the candidate and must be cited, not rerun, and that the full `cargo test` suite is not to be rerun.
2. [content] The same prompt says a criterion such as 'cargo test passes' is satisfied by citing the recorded check, and that new behavior is exercised only with targeted commands such as `cargo test -p <crate> <filter>`.
3. [content] The prompt keeps that a failed or missing recorded check is not a pass (the verifier may then run that one check).
4. [content] CHANGELOG.md has an entry naming the 2026-10-18 verifier timeouts (full suite ~11 min) as the reason.
5. [content] The diff touches only the verifier prompt and CHANGELOG.md: no Rust source, no watchdog or retry machinery.
6. [behavior] Run `cargo test -p athena-context`; it passes, showing the prompt edit did not break the brief snapshot tests.

## Expected deliverables

- crates/athena-context/prompts/verifier.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)

## Out of scope

- Producer checkpoints, early-stop watchdog, timed-out status and resumable retry
- Rust source changes
- Reader prompt and brief rendering changes

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single prompt file plus changelog; no code.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that verifiers cite recorded results, not that text exists only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test -p athena-context run to confirm snapshots still pass.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data.
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
