# Job 20261023-214328-hp1b: Add mistake category with per-class and per-cause counts

## Request (verbatim)

> try again on 20261023-183334-aej6

## Reading (accepted by the user)

Retry of job 20261023-183334-aej6: add a `mistake` intervention category. Status, the audit and the JSON count mistakes per class and per cause, apart from interventions_by_category. Introspecting a mistake proposes the preventing check or knowledge. This time a test must read the audit output, and the audit must print mistakes even when no job audit is produced. The audience is Sri.

Assumptions:
- The request as typed is 'try again' on the original mistakes job, so the original request is the goal. The pre-handover evidence check proposed by the introspection is a separate pipeline change and is not built here.
- The producer maps every criterion to a named test before finishing; this is the lesson of the earlier rejection, kept as a working practice and not as a new mechanism.
- Mistakes are interventions with category `mistake`, plus legacy interventions with category `other` whose text starts 'Mistake:'. Existing records are not rewritten.
- The previous attempt's file list is the starting point. It adds crates/athena-cli/tests/auditor.rs for the missing audit test and docs/design/athena-v0.md, which lists the category set.
- The audit's mistake report must not depend on a job audit being produced, so it prints before the early return in Command::Audit in main.rs.

## Acceptance criteria

1. [behavior] Run `cargo test`: new tests show an intervention logged with category `mistake` is accepted by the parser and appears in `InterventionCategory::ALL`, and an unknown category is still rejected with an error listing `mistake`.
2. [behavior] Run `cargo test`: a test with several mistake interventions of different classes and causes shows `athena status` output and its JSON report mistake counts per class and per cause, in fields separate from `interventions_by_category`.
3. [behavior] Run `cargo test`: a CLI test in crates/athena-cli/tests (auditor.rs or introspection.rs) runs the audit command and asserts its output reports mistakes per class and per cause. A second case asserts the mistake counts still print when no job audit is produced.
4. [behavior] Run `cargo test`: a test shows that introspecting a `mistake` intervention yields a proposal naming the check or knowledge that would have prevented it, and that mistakes with the same cause aggregate into one proposal.
5. [content] Existing records with category `other` and text starting 'Mistake:' are counted as mistakes in status and the audit. Records are not rewritten. The text-prefix rule is documented in a code comment or doc.
6. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean, and `cargo test` passes in full.
7. [content] Outcome-check: a reader of the status or audit output can see how many mistakes occurred, which classes and causes recur, and which prevention work was proposed. README.md, CHANGELOG.md and the category list in docs/design/athena-v0.md mention the `mistake` category.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/auditor.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/athena-v0.md (modify)

## Out of scope

- Building the pre-handover evidence gate in the job pipeline, which the introspection proposed as a separate next request
- Rewriting or migrating existing intervention records
- Changing athena.toml, defaults.toml or workflow files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Every new behavior has a named test; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check counts appearing in status and audit output, not just the new type.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires a reader to see mistake counts, recurring causes and proposed prevention.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The pipeline evidence gate is left out and the change stays on the mistake category.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust and Markdown.
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
