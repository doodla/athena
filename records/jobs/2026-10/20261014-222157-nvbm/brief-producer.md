# Job 20261014-222157-nvbm: Make readers list needed governance files; producers stop on unlisted on

## Request (verbatim)

> When the reader writes a reading, have it check whether the outcome needs a change to a governance file (athena.toml, defaults.toml, .github/workflows/*). If it does, list that file as a deliverable and name it in an acceptance criterion. Also have the producer stop and report, rather than edit, when it finds it needs a governance file the reading does not list. Add a test that a reading for a workflow change lists the workflow file.

## Reading (accepted by the user)

Edit the reader and producer prompt frames so the reader checks whether the outcome needs a governance file (athena.toml, defaults.toml, .github/workflows/*) and lists and names it, and the producer stops and reports instead of editing an unlisted one. Add prompt-wording tests in athena-context, including one for a workflow change. The audience is the Athena owner and the reader and producer agents.

Assumptions:
- The reader's existing line 15 already states the list-and-name rule; the change makes it an explicit step ('check whether the outcome needs...') and adds the producer stop-and-report rule, which is absent today.
- 'A test that a reading for a workflow change lists the workflow file' cannot run a live model reader, so it is a deterministic test: prompt-wording checks plus an admission-level check on a reading for a workflow path.
- Prompt changes must stay within the existing reader and producer token budgets (checked by existing tests).
- The producer's stop-and-report applies through the shared frame in athena-context, not the per-capability section, so it covers change and study.

## Acceptance criteria

1. [content] crates/athena-context/prompts/reader.md tells the reader to check whether the outcome needs a change to athena.toml, defaults.toml or a file under .github/workflows/, and if so to list that file as a deliverable and name it in an acceptance criterion.
2. [content] crates/athena-context/prompts/producer.md tells the producer that when it finds it needs a governance file the reading does not list, it stops and reports it instead of editing the file.
3. [behavior] A new test in crates/athena-context/src/lib.rs checks that the reader frame carries the governance-check wording, names .github/workflows/, and that the producer frame carries the stop-and-report wording. Run `cargo test -p athena-context`.
4. [behavior] A new test shows that a reading for a workflow change (for example a deliverable of .github/workflows/ci.yml) is accepted by governance_change_is_listed only when the workflow is listed as a deliverable and named in a criterion, and that is_governance recognizes the workflow path. Run `cargo test -p athena-core governance`.
5. [behavior] The reader and producer frames still fit their token budgets and the whole workspace passes: run `cargo test --locked`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`.
6. [content] No governance file (athena.toml, defaults.toml, workflows, Cargo lints) is modified by this change.

## Expected deliverables

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Changing admission rules that already refuse unlisted governance changes
- Editing athena.toml, defaults.toml or any workflow file
- Changing the study capability's prompts
- Changing how the verifier judges governance changes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the prompt wording and the workflow-listing rule; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The existing admission rule is the enforcement; the prompt edits are marked as guidance and tested by wording checks.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to two prompt frames plus tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Prompt and test wording is plain for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The reader is told to catch the need up front so admission refusals do not happen later.
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
