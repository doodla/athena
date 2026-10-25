# Job 20261017-103521-xypa: Version scoring rules so old scores still verify; gate rule changes on h

## Request (verbatim)

> PR #129 (job 20261010-154004-60r4) fails CI's 'the full history' check: its new rule marks a bound-only cost score provisional, and 'athena check --history' now finds 8 existing score files (e.g. records/jobs/2026-10/20261013-141005-hrdp/score.json) that do not follow from their records because they predate the rule. Make the rule hold without invalidating history written before it, without hand-editing records, and make a change to scoring rules get checked against the full history before it lands.

## Reading (accepted by the user)

The goal is to let PR #129's provisional-cost rule stand while the 8 older score files still check, without editing any record. The audience is the maintainer and CI's 'the full history' job. Scores record their rules string, so the new rule becomes a new rules version. Scores written under v1 are recomputed with the v1 behaviour (a bound-only cost is not provisional), and a check on scoring changes runs against the full history before landing.

Assumptions:
- PR #129's branch is the base for the work: the provisional-cost rule (cost_is_only_a_bound, cost_provisional) already exists in crates/athena-score/src/lib.rs. I only read the base tree, where that rule is partly present, so this is inferred.
- Score records carry a `rules` string, now '...; v1'. The gate refuses any score whose rules differ from athena_score::RULES (crates/athena-gate/src/jobs.rs:287). The fix is to bump RULES to v2 and have the gate recompute a v1 score under v1 behaviour, where a bound-only cost is firm.
- 'Checked against the full history before it lands' means the local and CI pre-landing check (xtask ci) also runs `athena check --history`, or an equivalent test recomputes every committed score. The 'history' CI job already exists but runs separately and failed only after the change was written.
- Edits to .github/workflows/ci.yml and athena.toml are not needed. The existing [history] exempt list is not used, because it would excuse the old commits rather than make them verify.
- The three minor findings from job 60r4 (display.rs shows provisional cost as firm; kill-switch reason untested) are out of scope unless trivially adjacent.

## Acceptance criteria

1. [behavior] Run `cargo run -p athena-cli -- check --history` from the repository root (or the equivalent `cargo run` of the athena binary). It reports no violation for the 8 existing score files, including records/jobs/2026-10/20261013-141005-hrdp/score.json, and no record file is modified (git status shows no change under records/).
2. [behavior] Run `cargo test` for athena-score and athena-gate. A new test shows a v1-rules score with a bound-only cost still follows from its records (cost not provisional), and a v2-rules score marks the bound-only cost provisional.
3. [behavior] A gate test shows a score that claims the current rules but has the wrong provisional flag is still refused. The check was not loosened for current scores.
4. [content] athena_score::RULES ends in v2, and the score code keeps a documented legacy path for v1 (with its rules string) so old scores recompute as they were written.
5. [behavior] `cargo run -p xtask -- ci` now includes the full-history check, so a scoring-rules change fails locally before landing if it breaks old records. Show it by running that command, or by a test of the task's step list.
6. [behavior] `cargo fmt --check` and `cargo clippy` are clean, and `cargo test` passes for the workspace.
7. [content] CHANGELOG.md or the relevant design note says that a scoring-rules change must bump the rules version and keep the previous version recomputable, with the reason.

## Expected deliverables

- crates/athena-score/src/lib.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- xtask/src/main.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-cli/tests/binary.rs (modify)

## Out of scope

- Hand-editing or rewriting any file under records/
- Adding the 8 commits to the [history] exempt list in athena.toml
- Showing the provisional flag in the status display (crates/athena-cli/src/display.rs)
- Tests for the kill-switch reason wording in job 60r4
- Changing the CI workflow file

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests for v1 and v2 recomputation; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Version the rules so any future scoring change has the same escape, instead of special-casing the 8 files.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The history check is wired into the pre-landing task, not left as prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the history check over the real records and check the outcome.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A verifier runs the build and tests and the history check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: rules versioning plus one gate wiring.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the legacy-rules path to an outside reader.
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
