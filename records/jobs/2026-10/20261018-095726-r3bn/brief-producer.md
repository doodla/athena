# Job 20261018-095726-r3bn: Dispatch jobs from a stable ref so merges don't disturb running jobs

## Request (verbatim)

> Let the next job start while the previous job's pull request is still in CI. Today merging a landing rebuilds the branch that jobs are dispatched from, so the next job has to wait for the merge. Dispatch jobs so that a merge cannot change the ground under a running job, and keep one landed job per merge.

## Reading (accepted by the user)

Today, after a squash merge, Athena rebuilds the local branch (resync_onto_merge) and jobs dispatch from it, so the next job waits. I read this as: cut each job from a pinned commit/ref that a merge never rewrites, so the next job can start while the previous PR is in CI. Each merge must still land exactly one job as one commit. The audience is the maintainer running Athena.

Assumptions:
- Jobs dispatch from the local branch, which a squash merge forces Athena to rebuild; I did not read the dispatch code in run.rs closely.
- Existing stale and merged-unverified protections (P5) stay in force.
- No change to governance files is needed.

## Acceptance criteria

1. [behavior] A new test shows that after a pull request is squash-merged while a second job is already dispatched, the second job's base and worktree are unchanged and its candidate still lands cleanly. Run cargo test.
2. [behavior] A test shows that two jobs dispatched back to back, before the first merges, each land as exactly one commit per merge, with no extra rebuild or merge commits. Run cargo test.
3. [behavior] cargo test passes, cargo clippy is clean, and cargo fmt --check passes.
4. [content] The dispatch source no longer depends on resync_onto_merge having run. The design doc and decisions.md explain the new base rule, the alternatives rejected, and how a job stale against a moved base is handled.
5. [content] The maintainer can read the docs and see that the next job starts while the earlier pull request is in CI, and that one merge lands one job.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- docs/design/parallel-jobs.md (modify)
- docs/design/native-pr-landing.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Automatic merging or re-verification of conflicting jobs
- Changing the merge method or the merge_via_rest default

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for the dispatch and landing behavior.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design doc and decision record updated.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise the merge-during-run scenario.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to dispatch and landing.
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
