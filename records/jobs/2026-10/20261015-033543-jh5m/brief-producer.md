# Job 20261015-033543-jh5m: Send unmet acceptance criteria back to the producer before verifying

## Request (verbatim)

> Before final verification, have the producer stage check the work against each acceptance criterion in the accepted reading. Report every unmet criterion back to the producer while it can still fix it. Add a test showing that a criterion unmet before final verification is reported to the producer and not first found by the verifier.

## Reading (accepted by the user)

Extend the producer report with a per-criterion self-check against the accepted reading's acceptance criteria. In the producer loop in run.rs, any unmet or missing criterion goes back to the producer once, in the same working directory, before the candidate reaches the verifier. A lifecycle test shows an unmet criterion is reported to the producer and fixed before verification. The audience is the Athena maintainer, who judges whether the verifier stops being the first to find unmet criteria.

Assumptions:
- The check is self-reported: the producer's report gains a criteria_check list, one entry per accepted criterion (criterion text, status met/unmet, evidence). Athena checks the list is complete and has no unmet entries, and it does not judge whether the evidence is true.
- It reuses the existing send-back loop in produce_and_verify (PRODUCER_SEND_BACKS) and does not add a new stage. An unmet or missing criterion after the allowed send-backs fails the job before verification, the same as a missing full test run.
- Every unmet criterion is listed in the send-back brief in one message.
- A criterion is matched by its exact text from the accepted reading; a missing entry counts as unmet.
- Pinned studies, which have no tests, keep their exemption from the test send-back; the criteria check applies to them only if they have acceptance criteria, and the same rule applies to both capabilities.
- The send-back budget is shared: one send-back round covers both the test and the criteria problems, and both are reported in the same brief.
- The new report field is required in the JSON schema, so the fake agent, the test helpers and the literal in run_pin_tests.rs must emit it.

## Acceptance criteria

1. [content] ProducerOutput in crates/athena-context/src/outputs.rs has a criteria_check field (criterion, status, evidence per entry). producer_schema() includes it and the existing schema test covers it.
2. [content] crates/athena-context/prompts/producer.md tells the producer to check its work against each accepted acceptance criterion and to report the result in criteria_check, and says that an unmet criterion is sent back once and then fails the job.
3. [content] produce_and_verify in crates/athena-core/src/run.rs sends a report back to the producer when any accepted criterion is unmet or missing from criteria_check. The send-back brief names every such criterion, and the producer's UI line says so.
4. [behavior] Run `cargo test --locked`. A new lifecycle test has the scripted producer first report one criterion unmet and then report all criteria met. It asserts that the producer runs twice before the first Verifier invocation, that the send-back brief text contains the unmet criterion, that the job lands, and that the verifier is never the first to see the unmet criterion.
5. [behavior] Run `cargo test --locked`. A second new test has the producer still report a criterion unmet after the send-back. It asserts that the job ends Failed with a reason naming the criterion and that no Verifier invocation happened.
6. [behavior] Run `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --check`. All three pass, and the existing lifecycle tests still pass after the fake agent and test helpers were updated to emit criteria_check.
7. [content] CHANGELOG.md has an entry describing the pre-verification criteria check.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- docs (modify)

## Out of scope

- Changing the verifier's own checking or its prompt.
- Having Athena independently judge whether the producer's evidence for a criterion is true.
- More than one send-back round, or changes to the test send-back budget.
- Changes to athena.toml, defaults.toml or the CI workflows.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and the tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The change adds lifecycle tests, and cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in the producer loop, not only stated in the prompt.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: It generalizes the existing send-back path, not a single case.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that reuses the existing loop.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by running the new tests.
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
