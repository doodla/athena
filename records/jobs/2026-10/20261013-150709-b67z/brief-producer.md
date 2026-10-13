# Job 20261013-150709-b67z: Fix athena land --pr base branch and work-branch contents

## Request (verbatim)

> Fix two bugs in 'athena land --pr' (crates/athena-core/src/gh.rs and its caller). First, the pull request's base must be the repository's default branch on the remote (as GitHub reports it), not the current local branch; today a local branch that is not on the remote makes GitHub answer HTTP 422. Second, the pushed work branch must carry the whole job, including its verification, outcome and other records, not only the candidate commit, so that merging the PR lands everything 'athena check --history' expects on main. Add tests for both. Keep merging behind the existing off-by-default opt-in.

## Reading (accepted by the user)

The PR base becomes the remote's default branch as GitHub reports it (a REST read of the repository), not the local current branch. The pushed athena/<job> branch becomes a commit that carries the candidate plus the job's records (verification, outcome, etc.), so merging it lands what `athena check --history` expects. Both get tests; merge stays behind `merge_via_rest`.

Assumptions:
- Records are committed on the user's local branch (HEAD), so the work branch should be the candidate combined with the records from local HEAD (e.g. a merge or records-tree commit on top of the candidate); the exact construction is the producer's choice if the fingerprint stays equal to the verified one.
- The default branch is read from `gh api repos/{owner}/{repo}` field `default_branch`.
- Pushed head/link.head and the checks lookup should use the new pushed commit rather than the bare candidate.

## Acceptance criteria

1. [content] PrHost (or GhClient) gains a call that reads the repository's default branch via `gh api repos/{owner}/{repo}` (`default_branch` field); Athena::land_pr uses it as NewPr.base and no longer uses git current_branch for the base.
2. [content] A detached HEAD or a local branch absent from the remote no longer affects the PR base or errors.
3. [content] The commit pushed to athena/<job> contains the candidate's content plus the job's records (verification, outcome, candidate and other records) from the local history, and is a descendant of the verified candidate so the push guard and fingerprint check still hold.
4. [content] The merge call remains reachable only when `merge_via_rest` is true and checks are green; that gate and its default (off) are unchanged.
5. [behavior] Run `cargo test -p athena-core gh`: a new test using fake Runner/PrHost shows the base is the host-reported default branch (e.g. `trunk`) while the local branch differs; a new test shows the pushed commit includes the records files, not just the candidate commit.
6. [behavior] Run `cargo test` , `cargo clippy --all-targets` and `cargo fmt --check`: all pass; existing land_pr tests still pass.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)

## Out of scope

- Enabling merge by default or changing the merge_via_rest setting
- Supporting hosts other than github.com
- Changes to athena check --history itself
- Force pushes or other branches than athena/<job>

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests for both fixes; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in gh.rs and its caller, with a git helper only if needed.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checked with cargo test, clippy and fmt --check.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the records-on-local-HEAD layout and fake-based test pattern from the repository.

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
