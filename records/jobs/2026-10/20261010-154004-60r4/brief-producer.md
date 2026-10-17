# Job 20261010-154004-60r4: Record kill-switch stops as killed; mark bound-only cost provisional

## Request (verbatim)

> Record a producer stopped by the kill switch as killed, and leave the cost score provisional when an execution cost is only a bound (m8, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

Fix m8: when the kill switch stops a producer before the driver's check, record the invocation as killed rather than error. Mark the cost_usd quantity score provisional when execution cost is only a bound. Audience is the maintainer and verifier reading records and scores.

Assumptions:
- Goal: fix minor m8 of the v0 acceptance review; audience is the maintainer and verifier reading invocation records and scores.
- Exact file paths in athena-score and athena-core are inferred; the producer locates them.
- No governance files need to change.

## Acceptance criteria

1. [behavior] A new driver test in crates/athena-agent/src/claude.rs shows a child ending with no result while the kill switch is on yields InvocationOutcome::Killed with a detail naming the kill switch, not Error. Run: cargo test -p athena-agent.
2. [behavior] A new scoring test in crates/athena-score builds a job whose execution cost is only a bound and shows the cost_usd quantity score is provisional; a job with exact costs is not provisional. Run: cargo test -p athena-score.
3. [behavior] An end-to-end test in crates/athena-cli/tests/lifecycle.rs shows a producer stopped by the kill switch closes with a killed invocation record and a provisional cost score, and the commit gate accepts the recomputed score. Run: cargo test -p athena-cli.
4. [behavior] Whole workspace passes cargo test and cargo clippy --all-targets with no warnings; cargo fmt --check is clean.
5. [content] A bound-only cost is no longer scored as a firm figure, and a provisional cost_usd score does not count in aggregates as a resolved value.
6. [content] Older score.json records without the new field still deserialize (serde default); existing history tests still pass.
7. [content] docs/records.md describes the provisional quantity field and kill-switch attribution, and CHANGELOG.md has an entry citing m8 of docs/design/reviews/athena-v0-acceptance-1.md.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-score/src/lib.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Other minor findings of the v0 acceptance review
- Changing the kill switch polling interval

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in agent, score, and cli crates; clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise killed and provisional outcomes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the attribution in the driver and the scoring of bound costs, not one record.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog updated for outside readers.
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
