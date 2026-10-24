# Job 20261024-170225-89rk: Retry from the rejected candidate's tree when only content criteria

## Request (verbatim)

> A retry starts from the base and redoes all the work, even when the rejected candidate passed every check and failed only a content criterion such as a missing change description: job 20261024-161559-rxjg passed fmt, clippy and 1094 tests and was rejected only for criterion 7 (a change description listing callers), and its retry costs about 40 minutes and 3 dollars to redo the code (job 20261023-234313-h49d was the same, rejected over one CHANGELOG line). When every check passed and the unmet criteria are content criteria, let the retry start from the rejected candidate's tree and send the producer only the findings, keeping the full verification of the new candidate.

## Reading (accepted by the user)

When a rejected job's checks (fmt, clippy, tests) all passed and the only unmet criteria are content criteria, the retry starts its producer snapshot from the rejected candidate's tree. The producer's brief carries only the verifier's findings. The new candidate gets the full verification. Any other rejection still restarts from the base. The audience is the Athena operator, who pays the roughly 40 minutes and 3 dollars of redone work.

Assumptions:
- Content criteria are criteria of kind 'content' in the accepted reading. A rejection qualifies only when every check recorded for the rejected candidate passed and every unmet criterion is of that kind.
- The rejected candidate's tree is available from the candidate record (candidate_tree/candidate_commit), and the retry's base commit is the same as the original job's.
- A retry whose rejection does not qualify (a failed check, an unmet behavior criterion, no candidate tree, a failed or interrupted job) behaves exactly as today and starts from the base.
- Today's behavior: run.rs produce_and_verify builds the producer snapshot with SnapshotRepo::Base from the base, and the retry brief says 'the retry starts from the base' (briefs.rs ~951). Both change only for the qualifying case.
- The new candidate is still computed as a change list against the base and fully verified (fmt, clippy, full tests, verifier). No verification is skipped.
- The verifier's judgment of unmet criteria is read from the rejected job's verification record. The spec of what counts as 'unmet' comes from existing records, not a new model call.

## Acceptance criteria

1. [behavior] A test that retries a rejected job whose checks all passed and whose only unmet criterion is a content criterion shows the producer's working snapshot containing the rejected candidate's files, not the base's. Run: cargo test --locked -p athena-core -p athena-cli (the new test by name).
2. [behavior] A test shows that the producer brief for such a retry includes the verifier's findings and states that the work starts from the rejected candidate. It does not claim the retry starts from the base. Run: cargo test --locked -p athena-context.
3. [behavior] Tests show a retry starts from the base, as before, when a check failed, when a behavior criterion is unmet, when the rejected job has no candidate tree, and when the job failed or was interrupted. Run: cargo test --locked.
4. [behavior] A test shows the new candidate from a tree-seeded retry still goes through the full check suite and the verifier, and its change list is computed against the original base so landing works unchanged. Run: cargo test --locked -p athena-core.
5. [behavior] A test shows that a tree-seeded retry whose producer changes nothing fails as an empty candidate, as today. It does not pass for inheriting the rejected tree. Run: cargo test --locked -p athena-core.
6. [behavior] cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked all pass.
7. [content] The qualifying rule (all checks passed, only content criteria unmet) is implemented in one named function with a doc comment. Its inputs are records, not strings parsed from prose.
8. [content] The change has a CHANGELOG [Unreleased] entry and a docs/decisions.md decision. Both say what changed, list each caller or writer touched with its covering test, and record the commands run. The decision names the alternatives (always from base, always from the candidate) and says any CHANGELOG or decision text that said retries always start from the base is superseded.
9. [content] The retry-related documentation and brief text (README or docs describing retry, briefs.rs wording) no longer says every retry starts from the base.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/pin.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- README.md (modify)

## Out of scope

- Skipping or weakening any verification of the new candidate.
- Changing the retry rule for failed or interrupted jobs, or for rejections caused by failed checks or unmet behavior criteria.
- Changing the producer's send-back loop or the full-suite policy.
- Edits to athena.toml, defaults.toml or the workflows.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Each new path has passing tests, with clippy and fmt clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the snapshot contents and brief text through tests, not that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria test that the redone work is avoided and that verification still runs in full.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule is general over any all-checks-passed, content-only rejection, not the two named jobs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision and CHANGELOG entry are written for an outside reader.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: A decision with alternatives is recorded in docs/decisions.md.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the retry seeding.
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

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A test that retries a rejected job whose checks all passed and whose only unmet criterion is a content criterion shows the producer's working snapshot containing the rejected candidate's files, not the base's. Run: cargo test --locked -p athena-core -p athena-cli (the new test by name).
A test shows that the producer brief for such a retry includes the verifier's findings and states that the work starts from the rejected candidate. It does not claim the retry starts from the base. Run: cargo test --locked -p athena-context.
Tests show a retry starts from the base, as before, when a check failed, when a behavior criterion is unmet, when the rejected job has no candidate tree, and when the job failed or was interrupted. Run: cargo test --locked.
A test shows the new candidate from a tree-seeded retry still goes through the full check suite and the verifier, and its change list is computed against the original base so landing works unchanged. Run: cargo test --locked -p athena-core.
A test shows that a tree-seeded retry whose producer changes nothing fails as an empty candidate, as today. It does not pass for inheriting the rejected tree. Run: cargo test --locked -p athena-core.
cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked all pass.
The qualifying rule (all checks passed, only content criteria unmet) is implemented in one named function with a doc comment. Its inputs are records, not strings parsed from prose.
The change has a CHANGELOG [Unreleased] entry and a docs/decisions.md decision. Both say what changed, list each caller or writer touched with its covering test, and record the commands run. The decision names the alternatives (always from base, always from the candidate) and says any CHANGELOG or decision text that said retries always start from the base is superseded.
The retry-related documentation and brief text (README or docs describing retry, briefs.rs wording) no longer says every retry starts from the base.

## Report

End with the producer report as your structured output.
