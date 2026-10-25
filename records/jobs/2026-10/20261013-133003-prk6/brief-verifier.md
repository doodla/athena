# Job 20261013-133003-prk6: verify "Build slice 1 of native PR landing: owned-mode core, answering N1-N4"

## Request (verbatim)

> Start building native PR landing (docs/design/native-pr-landing.md, accepted by review 3), owned mode first, answering review 3's N1-N4. Split it into slices if it is too big for one job.

## Reading (accepted by the user)

Read as slice 1 of several: amend the design to answer review 3's N1-N4, and build a tested library module for owned mode (PR link store, push guard, merged-content check). No CLI command, no `gh` calls, and no stealth mode yet; those are later slices.

Assumptions:
- The design is too big for one job, so this job is slice 1. Later slices are the `athena land --pr` command with `gh` calls, stealth mode, and the roadmap order change.
- N2 is answered by keeping the PR number, host, repository and pushed head in the state directory in both modes. The repository's records schema does not change.
- N1 is answered by comparing the merged commit's tree with the verified tree (the fingerprint). A mismatch leaves the job in a named 'merged-unverified' state. The user can re-verify that job on the new tip as a new job. The diff branch is dropped.
- N3 and N4 are stated in the design text. The stealth body drops the job id and Athena vocabulary, the leak check covers the body, and stealth pushes a neutral branch name. For a multi-job squash in stealth mode, the earlier jobs get the per-job checks over the local store.
- The first slice is pure library code with the git and host calls behind traits or fakes, so it can be tested without a network.
- I read only the design, review 3, and the land.rs header. I did not read the existing record schema or the state-directory layout, so the module's exact home is a guess.

## Acceptance criteria

1. [content] docs/design/native-pr-landing.md gains text for N1: the exact comparison (merged tree against the verified fingerprint), the named state of a merged-but-refused job, and how the user moves it out of that state.
2. [content] The design states for N2 that the PR number, host, repository and pushed head live in the state directory in both modes. The status line no longer says 'Not yet re-reviewed' and cites review 3.
3. [content] The design states for N3 what the stealth PR body keeps and drops, that the leak check covers the body, and that stealth pushes a neutral branch name. It also states how the push restriction to the job's own branch is checked.
4. [content] The design states for N4 which per-job checks run over the local store when several jobs are squashed in stealth mode.
5. [content] A new owned-mode module exists in athena-core. It stores and loads a PR link record (PR number, host, repository, head commit id, job id) under the job's state directory, outside the repository's records/.
6. [content] The module has a push guard that refuses any branch other than the job's work branch and any non-fast-forward push. It has no force option.
7. [content] The module has a merged-content check that returns a 'merged but not verified' result on a tree mismatch. The merge is found by the stored PR number and never by parsing trailers.
8. [behavior] Run `cargo test`. Tests pass for the guard refusing a non-fast-forward and a wrong branch, for a lookup by PR number on a merged commit with a trailing Co-authored-by paragraph and no Athena trailer, and for the mismatch result on a disjoint-file case.
9. [behavior] Run `cargo clippy` and `cargo fmt --check`. Both are clean.

## Expected deliverables

- crates/athena-core/src/pr.rs (create)
- crates/athena-core/src/lib.rs (modify)
- docs/design/native-pr-landing.md (modify)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-data/src/paths.rs (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- The `athena land --pr` CLI command and any call to `gh`
- Stealth mode implementation
- Multi-job pull requests
- Changing the roadmap order
- Editing records/, athena.toml, defaults.toml, or CI workflows
- Auto-merge

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Split into slices and build only owned-mode core logic plus the design answers.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The new module ships with passing tests, clippy clean.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design is amended for N1-N4 before the code relies on it.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The push guard is enforced in code, not prose.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Host calls will use the existing `gh` client in a later slice; no new crate is needed now.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and comments are written for an outside reader.

## The change

Base 2ade5a5dddbe → candidate df4d400a53c4. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |  10 +
 crates/athena-core/src/lib.rs    |   2 +
 crates/athena-core/src/pr.rs     | 450 +++++++++++++++++++++++++++++++++++++++
 docs/decisions.md                |  20 ++
 docs/design/native-pr-landing.md |  92 ++++++--
 docs/enforcement.md              |   4 +-
 6 files changed, 560 insertions(+), 18 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (332 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
