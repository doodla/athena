# Job 20261013-154735-te8h: Make land --pr push the landed commit; atomic lease on replace

## Request (verbatim)

> Retry 20261013-152648-1ds0 with the verifier's findings fixed: replace the work branch with an atomic git push --force-with-lease=refs/heads/<branch>:<expected> (no check-then-force), and when the remote head is unknown locally or the ancestry check errors, report the error instead of forcing.

## Reading (accepted by the user)

The repo has none of the earlier attempt, so this redoes job 1ds0 in full with the fixes. 'athena land --pr' pushes the job's locally landed commit (trailers and records) instead of the bare candidate. A rerun on an open PR replaces athena/<job> with one atomic `git push --force-with-lease=refs/heads/<branch>:<expected>`, and reuses the PR. An unknown remote head or an ancestry error is reported, never forced.

Assumptions:
- The workspace is at the state before the failed attempt: gh.rs still pushes work_commit (candidate plus a records commit), so the whole feature is rebuilt, not just patched.
- The commit to push is the job's locally landed commit, found from the job's landing record. If the job is not landed locally, the command reports that and does not push.
- Replacement applies only to the job's own branch athena/<job> (or its stealth name), and only when a PR link is already stored.
- The lease value is the remote head Athena just observed. If the remote head is not in the local object store, or is_ancestor errors, the command reports the error and does not push.
- Remote trait gains a lease-based replace method; the plain push stays non-forcing.
- The old work_commit helper becomes unused and is removed, with its tests replaced.

## Acceptance criteria

1. [content] athena-git has a push method that runs a single `git push --force-with-lease=refs/heads/<branch>:<expected> <remote> <commit>:refs/heads/<branch>`. It does no ls-remote check before the push and never uses a bare `+` or `--force`.
2. [content] In gh.rs, `athena land --pr` pushes the job's locally landed commit, which carries the Athena-Job trailer and the records, not the bare candidate. The choice and the reasons are recorded in docs/design/native-pr-landing.md.
3. [content] On a rerun with an open PR link whose stored head differs from the commit to push, land_pr replaces the branch head using the lease, reuses the stored PR number without opening a second PR, and updates the stored link head.
4. [content] When the remote head is unknown to the local object store, or the ancestry check returns an error, land_pr returns that error and makes no push of any kind. The `unwrap_or(false)` fallback is not used on the replace path.
5. [content] The replace path is refused for any branch other than the job's own work branch.
6. [behavior] Run `cargo test`. New tests cover: pushing the landed commit; replacing a bare-candidate head with the landed commit using a lease; a lease failure, where the remote moved after observation, is reported and leaves the remote unchanged; an ancestry error causes no push; an unknown remote head causes no push; the PR is reused. All pass.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
8. [content] CHANGELOG.md has an entry for the change.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/run.rs (modify)
- README.md (modify)

## Out of scope

- Changing athena check --history or CI workflows.
- Repairing PR #39 by hand.
- Force-pushing any branch other than athena/<job>.
- Changing records/ or governance files.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust; the docs are markdown.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests with a real git remote and fake host; cargo test and clippy must pass.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The approach and its alternatives are recorded in the existing native-pr-landing design doc.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the PR landing path.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The no-force rule is enforced by the lease and the guard in code, not by prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked through cargo test against a real local bare remote.

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
