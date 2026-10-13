# Job 20261013-162640-9h2j: verify "Fix athena land --pr rerun after merge; record merge-commit landing"

## Request (verbatim)

> Three fixes to 'athena land --pr', from landing PRs #39 and #40. (1) After the PR merges, rerunning it failed with 'git ls-tree: fatal: not a tree object' until I ran git fetch; it must fetch the merged commit itself before inspecting it. (2) When the job's PR link already exists, rerunning must first ask the host whether the PR merged and confirm the landing from that, even when the job's landed commit is not on the current local branch; today it errors with 'not landed on this branch'. (3) Record merge commits (what the opt-in merge already requests) as the landing shape for pull requests, in the design doc and docs/decisions.md. Evidence: squash-merging #39 broke ancestry, so the next PR conflicted, while a simulated history of two merge-commit landings passes 'athena check --history' with 0 violations. Add tests for (1) and (2).

## Reading (accepted by the user)

Make `athena land --pr` fetch the merged commit before fingerprinting it. On a rerun with an existing PR link, check the host for the merge before requiring the landed commit on the local branch. Record merge commits as the PR landing shape in the design doc and a new decisions.md entry. Tests cover fixes (1) and (2).

Assumptions:
- Fix (1) goes in `GitContent::fingerprint_of` (crates/athena-core/src/pr.rs). It runs `git fetch origin <sha>` when the commit is not present locally, using a new Git method in athena-git.
- Fix (2) goes in `Athena::land_pr`: when a stored PR link exists and `landed_commit` fails, the code still runs the host merge check. The head and replace flow need the landed commit. A job that is neither landed locally nor merged on the host keeps the 'not landed on this branch' error.
- Once the host confirms the merge and the fingerprint matches, the job reports as landed, even with the landed commit off the current branch.
- The design doc is docs/design/native-pr-landing.md, with a short note in docs/design/athena-v0.md section 7.3 if it describes the squash shape for PRs. The decision goes in a new entry D87 in docs/decisions.md.
- Tests use the existing fakes in gh.rs and pr.rs, plus a real-git test for the fetch if the athena-git test helpers support a second repository.
- No change to the merge request itself: `merge_method` is already `merge`.

## Acceptance criteria

1. [behavior] A test shows that when the merged commit is absent from the local object store but present on the remote, the merged-content check fetches it and returns the fingerprint instead of failing with 'not a tree object'. Run `cargo test -p athena-core`.
2. [behavior] A test shows that `land_pr` with an existing open PR link, a host that reports the PR merged, and a landed commit not reachable from the local head confirms the landing and does not return 'not landed on this branch'. Run `cargo test -p athena-core`.
3. [behavior] A test shows that with no stored link and no landed commit on the branch, `land_pr` still errors with 'not landed on this branch'. Run `cargo test -p athena-core`.
4. [behavior] `cargo test --workspace`, `cargo clippy --workspace --all-targets`, and `cargo fmt --check` all pass.
5. [content] docs/design/native-pr-landing.md states that merge commits are the landing shape for pull requests, and that squash merges break ancestry.
6. [content] docs/decisions.md has a new entry after D86 that records merge commits as the PR landing shape. It cites the #39 squash conflict and the simulated two-merge-commit history passing `athena check --history` with 0 violations.
7. [content] The fetch uses the existing Git wrapper and does not move any local branch or ref other than fetching the object.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/pr.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- docs/decisions.md (modify)
- docs/design/native-pr-landing.md (modify)
- docs/design/athena-v0.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)

## Out of scope

- Changing the merge method or the `merge_via_rest` setting
- Changing the history check or the gate's landing rules
- Exempting any squash commit from the history check
- Rewriting existing PR history

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests, plus markdown docs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for fixes (1) and (2); cargo test and clippy must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The decision entry cites the PR #39 squash ancestry break and the simulated two-merge-commit `athena check --history` run with 0 violations.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the land --pr path and its docs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test for the new tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the decision entry are written for an outside reader.

## The change

Base 948c4ce93884 → candidate d5deb7dca5dc. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |   6 ++
 crates/athena-core/src/gh.rs     | 164 ++++++++++++++++++++++++++++++++++-----
 crates/athena-core/src/pr.rs     |  48 +++++++++++-
 crates/athena-git/src/lib.rs     |  41 ++++++++++
 docs/decisions.md                |  20 +++++
 docs/design/athena-v0.md         |   4 +
 docs/design/native-pr-landing.md |  18 +++++
 7 files changed, 280 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (364 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
