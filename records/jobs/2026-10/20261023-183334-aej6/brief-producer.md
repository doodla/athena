# Job 20261023-183334-aej6: Track operator mistakes as their own intervention category

## Request (verbatim)

> Sri (BRIEF.md addendum of 2026-10-23, 'Track every mistake'): 'Start tracking every mistake you make as work Athena should start being intelligent about as well'. Interventions whose text starts 'Mistake:' are mistakes of the role doing Athena's thinking for now (eleven back-filled on 2026-10-23, category other). Give them a category of their own (mistake) in the closed list, have status and the audit count mistakes per class and per cause, and have introspection of a mistake propose the check or knowledge that would have prevented it, so repeat causes become work.

## Reading (accepted by the user)

Add a `mistake` variant to the closed intervention category list. Status and the audit then count mistakes per class and per cause. Introspection of a mistake proposes the check or knowledge that would have prevented it. The audience is Sri, who wants repeat mistake causes to become Athena work.

Assumptions:
- The category list is the Rust enum InterventionCategory in crates/athena-data/src/records.rs, with parse and ALL.
- 'Class' means class_of_work of the affected job. 'Cause' is derived from the mistake record's text or an existing introspection field. If no cause field exists, the producer picks the smallest such field.
- Interventions with text starting 'Mistake:' but category `other` count as mistakes, so the back-filled eleven are included. Records stay immutable.
- No change to athena.toml or defaults.toml is needed.

## Acceptance criteria

1. [behavior] Run `cargo test`: new tests pass. They show that an intervention logged with category `mistake` is accepted by the parser and `InterventionCategory::ALL`, and that an unknown category is still rejected with the list including `mistake`.
2. [behavior] Run `cargo test`: a test with several mistake interventions of different classes and causes shows that status and the audit output report mistake counts per class and per cause, and that the JSON has those breakdowns separate from `interventions_by_category`.
3. [behavior] Run `cargo test`: a test shows that introspecting a `mistake` intervention produces a proposal naming the check or knowledge that would have prevented it. Repeat causes aggregate into one proposal.
4. [content] Existing records with category `other` and text starting 'Mistake:' (the eleven back-filled on 2026-10-23) are counted as mistakes in status and the audit. Records are not rewritten. The text-prefix rule is documented in a code comment or doc.
5. [behavior] `cargo clippy` is clean and `cargo fmt --check` passes.
6. [content] Outcome-check: a reader of the status or audit output can see how many mistakes occurred, which classes and causes recur, and which prevention work was proposed. Output is not only a type or field addition.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- README.md (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Rewriting or re-categorizing existing records
- Automatically implementing the proposed prevention checks
- Changing athena.toml, defaults.toml, or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the category, the counts, and the proposal; clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check counts and proposals in output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criterion checks that mistakes are visible and become work.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change on the existing intervention pipeline.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Prefix-marked records are counted too, not only new ones.
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
