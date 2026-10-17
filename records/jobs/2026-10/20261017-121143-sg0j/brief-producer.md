# Job 20261017-121143-sg0j: Version scoring rules so older scores verify under their own rules

## Request (verbatim)

> PR #129 (job 20261010-154004-60r4) fails CI's 'the full history' check: its new rule marks a bound-only cost score provisional, and 'athena check --history' finds 8 older score files (e.g. records/jobs/2026-10/20261013-141005-hrdp/score.json) that predate the rule. Only fix that: version the scoring rules so older scores verify under the rules they were written with, without editing any record. Gating future rule changes on history is a separate request.

## Reading (accepted by the user)

The scorer has one RULES string, and the gate refuses or mis-recomputes any score written under older rules. PR #129's bound-only cost rule changed scores, so 8 older score files no longer recompute. I'll make the rules versioned: v1 keeps the old behavior, a new version holds the provisional-cost rule, and the gate recomputes each score under the version it records. No record is edited. The audience is the maintainer running `athena check --history` and CI.

Assumptions:
- Goal: `athena check --history` passes on the full history, so PR #129's CI check goes green. The gate runs the scorer with the one current RULES string, so scores written before the bound-only cost rule cannot match.
- Score files carry a `rules` string ending in 'v1' today (athena-score RULES). The new rule bumps the version to v2, and v1 is kept as an explicit, recomputable behavior.
- I could not read the 8 score files because records/ is absent from my copy, so which exact rules string each carries is inferred.
- I read athena-score/src/lib.rs, athena-gate/src/jobs.rs and history.rs. The producer should confirm the exact failing path (rules mismatch versus a recomputation difference) by running `athena check --history`.
- The listed paths are inferred. Callers of score_records in athena-core, such as scoring.rs, may need a signature or version-argument update.
- The existing rescore and re-score logic must also honor the recorded version.

## Acceptance criteria

1. [behavior] Run `cargo test`. A new test recomputes a score that records the v1 rules string, using a killed or bound-only cost invocation, under v1. It matches the recorded score with the cost quantity not provisional, and the gate's job-record check accepts it.
2. [behavior] Run `cargo test`. A test confirms a score written under the current rules still marks a bound-only cost provisional, and that the gate rejects a score whose rules string is not a known version.
3. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`. All pass.
4. [content] The scorer defines each rule version explicitly: v1 keeps the pre-#129 behavior with no provisional cost for a bound, and the current version holds the bound-only rule. Each is selected by the rules string recorded in a score, and Score.rules is written from the version used.
5. [content] The gate's job-record check in crates/athena-gate/src/jobs.rs recomputes a score under the version it records and no longer demands equality with the latest RULES. An unknown version is still refused.
6. [content] No file under records/ is modified, and the diff touches no athena.toml, defaults.toml or workflow file. The change adds no gate that fails on future rule changes.
7. [content] The change is documented in CHANGELOG.md and in the scoring or records docs: what a rules version is, and that older scores verify under their own version.

## Expected deliverables

- crates/athena-score/src/lib.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- docs/records.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Gating future rule changes on history, which the request calls a separate request
- Editing any file under records/
- Showing provisional cost in the status display (display.rs)
- The kill-switch reason coverage noted in job 60r4's findings

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the version dispatch; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Version the rules instead of special-casing the 8 files.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the versioning; no future-change gating.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run old-version and new-version scores through the gate.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog explain rule versions to outside readers.
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
