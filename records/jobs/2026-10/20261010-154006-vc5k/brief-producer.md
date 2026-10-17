# Job 20261010-154006-vc5k: Tell the verifier records/ and private paths are absent by design

## Request (verbatim)

> Tell the verifier that its snapshot leaves out records/ and owner-private paths by design (n3, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

Review note n3: the verifier prompt and brief lack the 'records/, private files, and history are absent by design' statement that the producer and reader prompts carry. Add it to verifier.md and verifier_brief so the verifier stops hunting for them or reporting their absence as a finding. Audience: the verifier model and maintainers reading its findings.

Assumptions:
- Verifier gets the same sentence as the producer brief (briefs.rs ~line 222), adapted for verification.
- CHANGELOG.md is touched only if the repo convention records prompt changes.
- No governance files change.

## Acceptance criteria

1. [content] crates/athena-context/prompts/verifier.md states that records/, private files, and history are absent from the verifier's snapshot by design, and that their absence is not a finding, in wording like producer.md and reader.md.
2. [content] verifier_brief in crates/athena-context/src/briefs.rs emits a sentence that records/, the owner's private files, and history are absent by design and that their absence is not a finding. Producer and reader briefs and prompts are unchanged.
3. [behavior] A new unit test in briefs.rs asserts verifier_brief output contains the 'absent by design' statement naming records/ and private files. Run `cargo test -p athena-context` and see it pass.
4. [behavior] `cargo test --workspace`, `cargo clippy --workspace --all-targets`, and `cargo fmt --check` pass, including any tests that pin prompt text.

## Expected deliverables

- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing producer or reader prompts or briefs
- Changing the design doc or review file
- Changing what the verifier snapshot contains

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test, with cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts the brief output contains the statement.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the verifier prompt, brief, and changelog change.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Both the prompt and the brief get the statement, not just one.
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
