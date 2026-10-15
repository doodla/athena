# Job 20261015-194312-jpe3: verify "Fix snapshot lock flake from vanishing files during tree walk"

## Request (verbatim)

> CI fails intermittently on PRs #103 and #104: snapshot::tests::a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree panics at crates/athena-git/src/snapshot.rs:544 because the second lock() returns Spawn(NotFound, 'No such file or directory'); the same commit passes on the other CI run. Find and fix the cause of the flake.

## Reading (accepted by the user)

The flaky error is an io NotFound (wrapped as GitError::Spawn) from lock()'s recursive set_tree_writable, which lists .git entries and then stats them; a transient git file (lock file or auto-gc/maintenance leftover from the preceding commit) vanishes in between. The fix makes the walk tolerate entries that disappear, and stops git from spawning background maintenance in snapshot commands. The audience is CI maintainers who need PRs #103 and #104 to pass reliably.

Assumptions:
- Spawn(NotFound) is a misleading label: GitError::Spawn wraps any io::Error via #[from], and lock() spawns nothing but chattr (whose error is discarded), so the NotFound comes from symlink_metadata/read_dir/set_permissions in set_tree_writable.
- The race is with a transient file in .git left by the preceding git add/commit in try_writes (index.lock, or detached gc --auto), which exists at read_dir time and is gone at stat time. I did not reproduce this.
- Fixing the tolerance in the walk is the root cause fix; disabling auto-gc is a complementary measure.

## Acceptance criteria

1. [content] set_tree_writable in snapshot.rs treats an entry that disappears between read_dir and stat/chmod (io NotFound) as already handled, instead of returning an error; other errors still propagate.
2. [content] A regression test in snapshot.rs shows the cause: a walk over a tree where an entry is removed mid-walk (or an equivalent deterministic simulation) succeeds, rather than relying on timing.
3. [content] Git commands run in snapshots no longer start background gc/maintenance (e.g. gc.auto=0, maintenance.auto=false), with the reasoning commented, if the producer confirms this is a source of the vanishing files.
4. [behavior] Run `cargo test -p athena-git` (including a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree) and it passes; the tests repeat passing across several runs.
5. [behavior] Run `cargo clippy --workspace --all-targets` and `cargo fmt --check`; both are clean.

## Expected deliverables

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing CI workflows or retry settings
- Rewriting the lock design or the immutable-attribute handling
- Other flaky tests unrelated to snapshot locking

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the unguarded walk race and the background git activity, not add a test retry.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Regression test plus cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the walk surviving a vanished entry and the test passing repeatedly.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Confined to the athena-git crate.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 95c0da8f0c14 → candidate 993729fffe51. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                      |  7 ++++++
 crates/athena-git/src/lib.rs      |  5 +++++
 crates/athena-git/src/snapshot.rs | 47 +++++++++++++++++++++++++++++++++++++--
 3 files changed, 57 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (707 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
