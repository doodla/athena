# Job 20261010-154005-jsdj: Trailer says no verifier ran for jobs rejected by a check

## Request (verbatim)

> Give a job that failed its checks before any verifier ran an Athena-Verifier trailer saying that no verifier ran (n2, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

Review note n2 says a job rejected by a check before any verifier ran still gets `Athena-Verifier: <model> fail`, which reads as if the verifier judged it. The closing commit should carry `Athena-Verifier: none (check <name> failed)` instead. The audience is anyone reading a job's commit trailers.

Assumptions:
- In run.rs (~line 1476-1488) the trailer is emitted from final_verdict whenever a verification exists, whatever Verification.verifier_invocation says. That is the root cause.
- A verifier ran if and only if verification.verifier_invocation is Some.
- With no verifier run, the first failed check in verification.checks (passed == false) names the check. If none failed, the form falls back to `none (no verifier ran)`.
- Several failed checks: name the first, or join the names with commas. Either is acceptable if documented.
- The extraction of trailer construction into a small pure function is allowed so it can be unit tested.

## Acceptance criteria

1. [behavior] A new unit test in athena-core builds the closing trailers for a verification with verifier_invocation None and a failed check named fmt. It asserts the Athena-Verifier value is exactly `none (check fmt failed)` and does not contain the verifier model name. Run `cargo test -p athena-core`.
2. [behavior] A test asserts that when verifier_invocation is Some, the trailer is still `<verifier-model> pass` for a pass verdict and `<verifier-model> fail` for a fail verdict. Run `cargo test -p athena-core`.
3. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.
4. [content] In crates/athena-core/src/run.rs the Athena-Verifier trailer is chosen by whether a verifier invocation exists. The model name appears only when a verifier ran.
5. [content] docs/records.md documents the `Athena-Verifier: none (check <name> failed)` form beside the `<model> pass` line, so a reader of a rejected job's commit can tell that no verifier judged it.
6. [content] The change is focused. No governance files change. CHANGELOG.md gets an entry only if the repo's convention requires one.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)

## Out of scope

- Changing how verdicts are computed or recorded in verification.json
- Changing the gate's reading of records
- Editing the review note itself

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests, with cargo test and clippy passing.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Choose the trailer by whether a verifier invocation exists, not just by the verdict.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Update docs/records.md so outside readers can interpret the trailer.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert the actual trailer values.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in run.rs and docs/records.md.
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
