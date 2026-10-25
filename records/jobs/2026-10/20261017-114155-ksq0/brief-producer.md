# Job 20261017-114155-ksq0: Version scoring rules so old scores verify; gate rule changes on history

## Request (verbatim)

> try again on 20261017-111119-m988

## Reading (accepted by the user)

Retry of job 20261017-111119-m988: bump the scoring rules to v2 so a bound-only cost is provisional only under v2, recompute older scores under the rules string they carry, and add a mechanism that checks scoring changes against full history. Audience: maintainers and CI, who need `athena check --history` to pass on PR #129 with no record edited.

Assumptions:
- The typed request (retry the rejected job) takes precedence over the introspection's proposed new request; building pre-handover command-evidence recording in the run stage is a separate change and is not done here.
- The earlier rejection was only a missing-evidence finding for the cargo test/clippy/fmt criterion, so the earlier design (v1->v2 rules, per-version recompute, history guard, decision record) is kept.
- The producer must actually run cargo test, cargo clippy and cargo fmt --check at the end and report the command lines, exit codes and output tails, so the verifier can cite them.
- The test helper landing_with_bound_cost in crates/athena-gate/src/landing_tests.rs is a legitimate deliverable this time.
- The history guard test should fail, not silently pass, when it cannot find the git repository or HEAD in CI, or must say plainly that it checked nothing.

## Acceptance criteria

1. [behavior] A scorer test shows the same job records score a bound-only cost as provisional under the current (v2) rules string and not provisional under the v1 rules string. Run `cargo test -p athena-score`.
2. [behavior] A gate test shows a score file stamped with the v1 rules string, whose cost was a bound, passes the score-follows-records check. A second test shows a v2 score with a wrong provisional flag is rejected, and an unknown rules string is still refused. Run `cargo test -p athena-gate`.
3. [behavior] `cargo run -p athena-cli -- check --history` (or the repository's equivalent) over the real repository reports no violation for the 8 existing score files, including records/jobs/2026-10/20261013-141005-hrdp/score.json. `git diff` shows no file under records/ modified. Run `git diff --stat` and the check.
4. [content] New scores are written under a bumped v2 rules string. The gate recomputes each score under the rules string it carries.
5. [behavior] A mechanism makes a scoring change get checked against full history. A test or an xtask ci step recomputes every historical score with the working-tree scorer, and a test shows it fails when a rule changes under an unchanged version. The guard fails or states it checked nothing when git or HEAD is missing instead of passing silently, and the CI workflow runs it. Run `cargo test` and `cargo run -p xtask -- ci`.
6. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass. The producer's handover quotes each command line, its exit status and its output tail as evidence.
7. [content] docs/decisions.md records the call (versioned rules over rewriting history or exempting records) with alternatives, and CHANGELOG.md notes the rules version bump.

## Expected deliverables

- crates/athena-score/src/lib.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- xtask/src/main.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- crates/athena-data/src/records.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Recording command evidence in the run stage before handover (the introspection's proposed new feature); separate change.
- Editing or regenerating any file under records/.
- Exempting old records from the gate.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criterion requires cargo test, clippy and fmt --check to pass, with new tests for the new behavior.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name runnable commands so the verifier can exercise them.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that old scores verify and that a changed rule under an unchanged version fails, not that wiring exists.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The history check on scoring changes is a test or xtask ci step, not prose.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: The history guard must not pass silently when it checked zero scores or found no repository.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Versioned rules fix the cause (rules changing under old records), not only the 8 files.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision is recorded with alternatives in docs/decisions.md and CHANGELOG.md for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The evidence-recording run-stage feature is left out as a separate change.
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
