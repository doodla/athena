# Job 20261015-141831-hyb4: Add behavior-evidence default with protection-test rule and examples

## Request (verbatim)

> try again on 20261015-095214-y8kr

## Reading (accepted by the user)

Redo rejected job 20261015-095214-y8kr from the base (its edits were discarded): add a new defaults.toml entry requiring acceptance criteria to check the requested behavior, with protection/denial tests shown unprotected-as-root then refused and failing rather than skipping. Add worked examples and a test. Keep the diff minimal: no reader.md rewording and every CHANGELOG claim true. Audience: the reader, producer and verifier receiving rendered defaults.

Assumptions:
- The earlier changes were discarded, so the retry rebuilds the whole feature, not just the two repairs.
- The defaults.toml entry is the only prompt-facing change; reader.md is left byte-identical unless the reader token budget in athena-context forces a change.
- If reader.md must change, only the intended lines change, each instruction stays its own bullet, and the CHANGELOG says why.
- The 'applies_to' of the new entry is ["*"] and its first sentence carries the core rule, since the rendered brief shows only the first sentence.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry (e.g. id "behavior-evidence") with applies_to ["*"], a source citing this request, and a statement whose FIRST sentence says criteria must check the requested behavior happening, not that types, files or wiring exist.
2. [content] The statement's later sentences require that a protection or denial test first shows the action succeeds without the protection, as the same user and environment the real component runs as (root), then shows it is refused, and that it fails, not skips or passes silently, if the environment cannot enforce the protection.
3. [behavior] A new unit test in crates/athena-data/src/defaults.rs parses the repository defaults.toml and asserts the new entry exists and its render() line contains the core rule. Run `cargo test -p athena-data`: it passes along with root_cause_default_leads_with_the_core_rule.
4. [content] docs/design/defaults-loop.md has a worked example for a 'make X read-only' request whose criteria are: a write to X as root succeeds before protection; the same root write is refused after; the test fails, not skips, if the environment cannot enforce it (a root-bypassed file mode is not proof).
5. [content] The same doc has a worked example for 'role calls a model' whose criteria run the role end to end and check the model request is observed (e.g. through the fake client) and its output is used, not merely that a client or config is wired in.
6. [content] Outcome check for the reader: the rendered reader brief for the defaults set includes the new default line with the core rule, so a reader writing criteria for a read-only request sees it. Check by running the athena-context tests, which must still pass within the reader token budget.
7. [content] `git diff` shows no collateral edits: crates/athena-context/prompts/reader.md is unchanged, or, if changed, differs only by the intended lines with every instruction still its own bullet. Each CHANGELOG.md claim under Unreleased / Added names only things present in the diff, with no claim about the reader prompt unless reader.md contains it.
8. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` all pass from the repository root.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- docs/design/defaults-loop.md (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Building a pre-verification diff or changelog checker (the introspection's proposed code-check).
- Changing how prompts are built beyond the defaults rendering.
- Rewording existing reader prompt guidance.
- Editing records/ or the owner's files.
- Changing existing defaults.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A unit test in defaults.rs asserts the new entry and its rendered first-sentence line; cargo test, clippy and fmt must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule goes into the shared defaults every role receives, not into one prompt, and the earlier failure's cause (collateral edits and untrue changelog claims) is avoided by keeping the diff minimal.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: defaults entry, test, example doc and one changelog line.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria require running cargo test and checking the diff for collateral changes.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Entry and examples are written for an outside reader.
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
