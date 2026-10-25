# Job 20261017-003902-d5gk: Add SHA-keyed CI result reuse with local fallback (ci-routing item 4)

## Request (verbatim)

> Finish the CI routing build: items 4-7 of docs/design/ci-routing.md section 10 (result reuse keyed by commit, per-job routing of what runs locally vs in CI, and the after measurement with xtask walltime against the baseline from #117).

## Reading (accepted by the user)

Per the user's reply, only item 4 of ci-routing.md section 10 is in scope. Add a CI result record keyed by commit SHA, the SHA check and missing/red/stale rules of 5.3, and a local fallback, in athena-core with tests, then mark item 4 in section 10. The audience is the owner, who judges by tests that fail without the SHA check or the fallback.

Assumptions:
- Per the user's reply, the scope is item 4 only; this overrides the earlier readings' scope.
- The record is read by Athena from the platform API for the exact SHA, as in 5.2 and 6.2.
- New logic is a pure, testable module in athena-core, with the runner and fetch injected so tests need no network.
- The wait bound for an unfinished run is a parameter; its value comes from later measurement data.
- No governance files change.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: new tests pass showing a CI result whose SHA differs from the candidate's is ignored (stale) and never yields a verified outcome; each test fails if the SHA comparison is removed.
2. [behavior] Run `cargo test --locked`: tests show a missing, unfinished, cancelled, skipped or timed-out CI result selects the local fallback, which runs the three checks once and records them as local, keyed by SHA.
3. [behavior] Run `cargo test --locked`: a red CI result for the matching SHA is a failed check and the fallback is not run to hide it; a green run with a workflow file changed in the same candidate is not trusted and falls back locally.
4. [behavior] Run `cargo test --locked`: when neither CI nor the local run can produce a result, the outcome is blocked with a named reason, not verified (no empty success).
5. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --check`: both are clean.
6. [content] The record holds the run address, full SHA, per-step conclusions and duration, and Athena (not the verifier) is what reads it; the verifier command set in athena.toml is unchanged.
7. [content] Section 10 item 4 of docs/design/ci-routing.md states what was built and what remains; items 5-7 are untouched, and no speedup is claimed.
8. [content] CHANGELOG.md has an entry for the result-reuse change.

## Expected deliverables

- crates/athena-core/src/ci_result.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- docs/design/ci-routing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Item 5: per-job-kind routing from section 3.2
- Items 6-7: after-measurement with xtask walltime and the final docs job
- Changing athena.toml, defaults.toml or .github/workflows/
- Letting the verifier fetch the CI result itself

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests plus clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check stale, fallback, red and blocked behavior through tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only item 4 is read.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A criterion requires blocked rather than verified when no result exists.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
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
