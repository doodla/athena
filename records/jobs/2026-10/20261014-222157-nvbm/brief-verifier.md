# Job 20261014-222157-nvbm: verify "Make readers list needed governance files; producers stop on unlisted on"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 32d61134a544 → candidate 943148632980. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |  5 +++++
 crates/athena-context/prompts/producer.md | 13 +++++++------
 crates/athena-context/prompts/reader.md   | 22 +++++++++++-----------
 crates/athena-context/src/lib.rs          | 24 ++++++++++++++++++++++++
 crates/athena-core/src/admission.rs       | 28 ++++++++++++++++++++++++++++
 5 files changed, 75 insertions(+), 17 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (592 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
