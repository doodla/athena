# Job 20261024-002644-1as8: Tell change verifiers to cite recorded checks, not rerun the suite

## Request (verbatim)

> try again on 20261023-234149-jcd2

## Reading (accepted by the user)

Add one short instruction (under ~100 tokens) to the change-capability verifier prompt: cite the recorded fmt/clippy/test results under "Checks Athena ran on this tree" instead of rerunning the full suite, running them only if missing or failed. Add a CHANGELOG entry citing the 2026-10-18 verifier timeouts. Audience: verifiers and the maintainer reading the diff.

Assumptions:
- Per the user's reply, the instruction goes only in the change verifier prompt because the study set is at 1296/1300 tokens.
- The producer runs only targeted tests while editing and the full suite at most once.

## Acceptance criteria

1. [content] crates/athena-cap-change/prompts/verifier.md tells the verifier that the fmt, clippy and test results under 'Checks Athena ran on this tree' are recorded for this candidate: cite them and do not rerun the full cargo test suite; 'cargo test passes' is met by citing the record.
2. [content] The prompt says a failed or missing recorded check is not a pass and the verifier may then run just that check; new behavior is exercised with targeted commands like `cargo test -p <crate> <filter>`.
3. [content] CHANGELOG.md has an entry naming the 2026-10-18 verifier timeouts (full suite about 11 minutes) as the reason.
4. [content] The diff touches only crates/athena-cap-change/prompts/verifier.md and CHANGELOG.md: not the shared crates/athena-context/prompts/verifier.md, no Rust source, no config.
5. [behavior] Run `cargo test --locked -p athena-cli --test budgets -- --nocapture`; it passes and the change set stays within its 1300-token budget (was 1147).
6. [behavior] Run `cargo test -p athena-context`; it passes, showing brief snapshot tests are unbroken.

## Expected deliverables

- crates/athena-cap-change/prompts/verifier.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Shared crates/athena-context/prompts/verifier.md and the study verifier prompt
- Producer watchdog, checkpoint or retry machinery
- Rust source or config changes

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One prompt sentence plus a changelog line.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Budgets and athena-context tests named as behavior criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the prompt text and budget test outcomes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog entry cites the dated cause.
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
