# Job 20261014-075345-x9cx: Forbid verifiers from presenting snapshot runs as real-repo results

## Request (verbatim)

> Verifiers have reported runs in their snapshot, which has no records/, as results on the real repository; that false claim reached main in PR #59. Stop that from happening.

## Reading (accepted by the user)

Read as a prompt fix: the verifier's snapshot lacks records/, history and private files, so any run there (cargo test, policy checks) describes only the snapshot. The verifier prompt must say this and require findings to label such runs as snapshot results and never claim anything about records/ or the real repository. A test pins the wording.

Assumptions:
- The false claim came from verifier prompt wording, not from code that fabricates results.
- Fix is a prompt rule plus a pinning test; no new gate that scans verdicts for such claims.
- PR #59 content itself is not rewritten; that is history.

## Acceptance criteria

1. [content] crates/athena-context/prompts/verifier.md states that the verifier's working directory is a snapshot without records/, owner files, or history, and that anything it ran there is a result about the snapshot only.
2. [content] The prompt tells the verifier to say so explicitly in evidence and findings whenever a command or its output depends on records/ or other absent paths, and never to report such a run as a result on the real repository.
3. [content] A unit test in athena-context asserts that VERIFIER_FRAME contains the new snapshot-scope wording, so removing it fails the test.
4. [behavior] Run `cargo test` from the repository root: it passes, including the prompt budget tests and the new test.
5. [behavior] Run `cargo clippy` and `cargo fmt --check`: both are clean.

## Expected deliverables

- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Editing records/ or already-merged content from PR #59
- A mechanism that automatically detects false claims in verdicts
- Changing the snapshot contents

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the verifier prompt that lets snapshot runs be reported as real-repo results, not just the one claim.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add a test pinning the new wording; cargo test and clippy pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One prompt change and a test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces the prompt wording; the rule itself stays prose for the model.
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
