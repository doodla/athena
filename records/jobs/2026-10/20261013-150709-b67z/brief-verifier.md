# Job 20261013-150709-b67z: verify "Fix athena land --pr base branch and work-branch contents"

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

## The change

Base 9d0449dce7eb → candidate dd50c2bbee4c. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |   5 +
 crates/athena-core/src/gh.rs     | 211 ++++++++++++++++++++++++++++++++++-----
 crates/athena-git/src/lib.rs     |  96 ++++++++++++++++++
 docs/design/native-pr-landing.md |   8 +-
 4 files changed, 291 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (349 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
