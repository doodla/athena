# Job 20261014-222157-bk5x: Require producer to run full cargo test --locked; return not-run

## Request (verbatim)

> Change the producer prompt (line 8 says the producer 'may' run the checks) so the producer must run the full `cargo test --locked` before it finishes, even for docs, TOML or data-only changes. Policy and config tests read tracked non-Rust files: boundary words, unlisted file kinds, defaults.toml, and study model.json. A candidate whose full test run is 'not-run' should be sent back to the producer instead of going on to verification.

## Reading (accepted by the user)

Replace line 8 of the producer frame prompt with a must-run rule for `cargo test --locked` on every change, including docs, TOML and data-only ones. Add a gate in the job flow: if the producer's report shows that test run as `not-run`, re-invoke the producer once rather than passing the candidate to verification. Audience: the Athena maintainer, who judges whether docs-only candidates stop reaching verification with an unrun test suite.

Assumptions:
- Today no code path re-invokes the producer: produce_and_verify in crates/athena-core/src/run.rs calls it once, and the candidate then goes straight to checks and verification. 'Sent back to the producer' therefore needs a small bounded retry in that function, which I assume the user wants.
- The retry is limited to one extra producer invocation, to bound cost. If it is still not-run, the job fails with a clear reason and does not reach verification.
- The gate keys on the producer's reported checks_run: an entry whose command is `cargo test --locked` with result not-run, or no such entry. Athena's own check runner is unchanged, though its skip_when_only patterns for docs and *.md remain; athena.toml line 79 already says the test check never skips.
- The producer's Bash grant must allow `cargo test --locked`. athena.toml already lists cargo test as allowed, so no governance file changes. If the grant needs a change, that is a separate decision.
- Only the shared producer frame (crates/athena-context/prompts/producer.md) changes. The capability-specific producer.md files are left alone.

## Acceptance criteria

1. [content] crates/athena-context/prompts/producer.md no longer says the producer 'may' run the checks. It requires the producer to run the full `cargo test --locked` before finishing, for every change including docs, TOML and data-only ones. It names the reason: policy and config tests read tracked non-Rust files (boundary words, unlisted file kinds, defaults.toml, study model.json).
2. [content] The prompt says a `not-run` full test result is not an acceptable finish, and that the producer must report the real result in checks_run.
3. [behavior] A candidate whose producer report lists the full `cargo test --locked` as `not-run` (or lists no such check) is not sent to the verifier. The producer is re-invoked with a message explaining why, and a bounded retry count applies. Run `cargo test` and check that the new test for this gate passes.
4. [behavior] A candidate whose report shows the full test run as pass or fail continues to verification as before. Run `cargo test` and check that the existing lifecycle and run tests still pass.
5. [behavior] `cargo fmt --check` and `cargo clippy` are clean, and prompt budget tests (crates/athena-cli/tests/budgets.rs) still pass with the longer prompt.
6. [content] Docs that describe the producer's checks (docs/design/athena-v0.md) match the new rule and the send-back behavior. The CHANGELOG records the change.

## Expected deliverables

- crates/athena-context/prompts/producer.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- docs/design/athena-v0.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing Athena's own check runner or the skip_when_only patterns in athena.toml
- Changing the verifier or reader prompts
- Changing the study capability's producer prompt
- Making the producer run clippy or fmt as a hard requirement

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new gate gets a test, and cargo test and clippy must stay clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The prompt rule and the gate that enforces it are both changed, so the rule is not left as an unenforced request.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The not-run rule is enforced in the job flow, not only stated in prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One retry, no new config keys, and no changes to the check runner.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The prompt text states the reason for the rule, so an outside reader understands why docs-only changes still need the full test run.
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
