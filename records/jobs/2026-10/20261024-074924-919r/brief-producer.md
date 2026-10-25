# Job 20261024-074924-919r: Tell verifier to put cited Athena checks in the commands list

## Request (verbatim)

> In crates/athena-cap-change/prompts/verifier.md, the instruction to cite the fmt, clippy and test results under 'Checks Athena ran on this tree' does not say where: job 20261024-033935-c4w4's verifier quoted them in its evidence text with an empty commands list, and D43 (verify.rs) reads only the commands list, so the criterion was rejected. Say that each recorded check cited goes in that criterion's commands list exactly as shown under 'Checks Athena ran on this tree'. Keep the change verifier inside its token budget (cargo test --locked -p athena-cli --test budgets -- --nocapture prints the sizes) and edit only that prompt and CHANGELOG.md.

## Reading (accepted by the user)

Edit the verifier prompt so each recorded fmt/clippy/test check it cites goes in that criterion's commands list, exactly as shown under "Checks Athena ran on this tree", not only in evidence text. Add a CHANGELOG entry. Audience: the change verifier and the maintainer reading rejections.

Assumptions:
- The existing sentence is edited in place with a short addition; no other prompt text changes.
- The prompt's wording on 'a failed or missing record' stays as is.

## Acceptance criteria

1. [content] verifier.md says that each recorded check cited goes in that criterion's commands list, exactly as shown under 'Checks Athena ran on this tree', and that quoting it only in evidence text is not enough (D43 reads only the commands list).
2. [content] CHANGELOG.md has an entry under Unreleased naming the cause (job 20261024-033935-c4w4, empty commands list, D43) and the prompt change.
3. [content] Only crates/athena-cap-change/prompts/verifier.md and CHANGELOG.md differ from the base (git diff --stat).
4. [behavior] The budgets test passes and shows the change verifier inside its token budget: run cargo test --locked -p athena-cli --test budgets -- --nocapture.
5. [behavior] Existing tests that read the verifier prompt still pass: run cargo test -p athena-cap-change.

## Expected deliverables

- crates/athena-cap-change/prompts/verifier.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing verify.rs or D43 logic
- Other prompts
- Design docs

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One sentence added to one prompt plus a changelog entry.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Budget and crate tests are behavior criteria.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion checks the instruction names the commands list, which is what D43 reads.
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
