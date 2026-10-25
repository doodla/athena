# Job 20261016-174510-bwhh: Ignore shared append-only files when grouping plan items in parallel

## Request (verbatim)

> Every planned item touches CHANGELOG.md (and many docs/decisions.md), so the plan's parallel groups are all single items. Shared append-only files should not count as a clash, as docs/design/parallel-jobs.md section 9 proposes.

## Reading (accepted by the user)

Plan grouping (plan.rs `disjoint`) treats two items touching the same CHANGELOG.md or docs/decisions.md as a clash, so every parallel group is a single item. Change it so those shared append-only files are excluded from the overlap test, as parallel-jobs.md section 9 proposes. Audience: the maintainer running `plan`/drain who wants parallel groups.

Assumptions:
- The fix goes in plan.rs overlap/disjoint logic, not in the pool serialization of section 9 rule 3, which is a different mechanism.
- Shared files are CHANGELOG.md, docs/decisions.md and README.md, as listed in section 9.
- An entry whose only touched paths are shared files counts as naming no other path, so it stays ungrouped (conservative).
- CHANGELOG.md is touched per the standing changelog default.

## Acceptance criteria

1. [behavior] A new plan.rs unit test shows two entries that touch disjoint code files plus the same CHANGELOG.md and docs/decisions.md land in one parallel group; run `cargo test -p athena-core plan`.
2. [behavior] A test shows two entries that both touch the same non-shared file (e.g. crates/core/src/x.rs) still land in separate groups, and an entry touching only CHANGELOG.md is still not grouped with another (nothing else known); run `cargo test -p athena-core plan`.
3. [behavior] cargo test and cargo clippy --all-targets pass, and cargo fmt --check is clean.
4. [content] The shared append-only list (CHANGELOG.md, docs/decisions.md, README.md) is one named constant in plan.rs with a doc comment citing parallel-jobs.md section 9, and the module docs describe the exception.
5. [content] The change stays within plan grouping; the docs say landing-check authority for conflicting shared-file edits is unchanged.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- docs/design/parallel-jobs.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Implementing the docs-job-writes-shared-files rule or decision-number allocation
- Changing landing-check or merge behavior
- Changing max_concurrency defaults
- Changes to athena.toml or defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust in plan.rs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the new grouping; cargo test and clippy pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the overlap rule that makes all groups singletons, not a per-item workaround.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check items actually grouping together.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in one function.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test.
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
