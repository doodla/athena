# Job 20261024-055549-0qcy: verify "Add records-branch writer to athena-git (retry of 20261024-033933-ta60)"

## Request (verbatim)

> try again on 20261024-033933-ta60

## Reading (accepted by the user)

Retry the rejected job: add a writer in athena-git that appends opaque record files to the orphan branch athena/records using compare-and-swap, fast-forward only, bounded retry (§8.2, D111). The prior attempt was rejected only because the gate criterion had no cited commands; this attempt must run and cite the gate commands.

Assumptions:
- Goal: a tested, uncalled library writer; audience is the maintainer and the verifier, who run the tests.
- The retry takes the original records-branch request as typed; the introspection's proposed pipeline pre-handover step is a separate job and is out of scope.
- The earlier design and test list were accepted; only the evidence gap (criterion 5) and the weak contention nit need fixing.
- The existing Git wrapper, temp index and commit_tree helpers in athena-git suffice; no new crate, so Cargo.toml likely stays unchanged.
- Writing identical bytes at an existing path is refused (or documented as a no-op); the producer picks one and documents it.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-git`. A test starts two writers on threads against one repository, with different paths. Both land, both paths are on the final tip, each update is a fast-forward of the previous tip, and no commit is lost.
2. [behavior] Run `cargo test -p athena-git`. A deterministic test makes the expected tip stale. The first update is rejected, the writer refetches and replays its files on the new tip, and the write succeeds with both the earlier and new files in the final tree. The two-thread test needs no forced contention; this test covers the reject path.
3. [behavior] Run `cargo test -p athena-git`. Tests show that modifying, deleting or renaming an existing path is refused with an error and the tip is unchanged. A same-bytes write at an existing path is tested and its result documented.
4. [behavior] Run `cargo test -p athena-git`. A test shows `git merge-base main athena/records` finds nothing and the first records commit has no parent. A second test shows that when every attempt is rejected, the writer returns a clear error after the bounded count.
5. [behavior] Run `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` from the repository root. All pass and are clean. The producer's final report quotes each command line and its result so the verifier can cite them (D43).
6. [content] A new public module in athena-git, exported from lib.rs, takes opaque bytes keyed by path. Its doc comments cite §8.2 and D111. It uses the existing Git wrapper and helpers with no new crate. No code outside athena-git calls it.
7. [content] The CHANGELOG [Unreleased] section has an entry, and docs/design/records-history.md notes the writer as built. No governance file (athena.toml, defaults.toml, workflows, lint tables) and nothing under records/ is changed.

## Expected deliverables

- crates/athena-git/src/records_branch.rs (create)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-git/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Any pre-handover evidence step or other change to the job pipeline
- Calling the writer from the pipeline
- Pushing or publishing athena/records to a remote
- Changes to governance files or records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The writer and its tests are Rust in athena-git.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Every new module has passing tests; cargo test and clippy are clean.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse existing helpers; add no crate.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check writers landing, replay, refusal and orphan history by running tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module plus export, changelog and design note.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite §8.2 and D111 for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The tests check the outcome: the branch state after each call.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Test two_writers_on_threads_both_land: in a testutil repo, spawn two scoped threads, each appending a different path. Then check the final tree and the commit chain. Run: `cargo test -p athena-git records_branch`. Observes: Both paths are in the tip tree. `rev_list_reverse` gives 2 commits in a linear chain. The first has no parent, and is_ancestor holds between consecutive tips. No commit is lost.
  Fails without the behavior: Without the CAS and retry, one writer's update would overwrite the other's or fail, and one path would be missing from the final tree.
- Criterion 2: Test stale_tip_is_rejected_then_replayed: use the private hook to advance the branch with another writer's commit after the first tip read. The writer's first swap fails, it refetches, and it succeeds on the second attempt. The hook counts attempts. Run: `cargo test -p athena-git records_branch`. Observes: The attempt count is 2. The final tree holds both the earlier file and the new file. The new commit's parent is the competing commit.
  Fails without the behavior: Without the expected-old-value check, the stale write would succeed on the first attempt and drop the competing file.
- Criterion 3: Tests for Put with different bytes, Delete and Rename on an existing path, each asserting Err(Refused). A same-bytes Put test asserts Ok with the tip unchanged. The doc comment records this behavior. Run: `cargo test -p athena-git records_branch`. Observes: Each refusal returns Refused, and rev_parse of the ref is identical before and after. The same-bytes write makes no new commit.
  Fails without the behavior: Without the append-only check, the tip would change or the tree content would be altered.
- Criterion 4: One test checks orphan history: after the first write, `git merge-base main athena/records` exits 1 with empty output (via Git::exit), and parents(first) is empty. A second test uses the hook to move the tip on every attempt, so every swap is rejected. With max attempts set to 3, it expects Contended{attempts:3} and a message naming the count. Run: `cargo test -p athena-git records_branch`. Observes: No merge base and no parent. The error is Contended with the bounded count, and the hook ran exactly 3 times.
  Fails without the behavior: Without the orphan logic, the first commit would have a parent and merge-base would find one. Without the bound, the loop would not terminate or would give a vague error.
- Criterion 5: The producer runs the three gate commands from the repository root and quotes each command line with its exit status and result summary in the final report. The verifier reruns them. Run: `cargo test`. Observes: Quote the lines `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check`, each with its result: all tests pass, no clippy warnings, and fmt is clean.
  Fails without the behavior: A failing test, a clippy warning or a formatting difference would make the corresponding command exit non-zero.
- Criterion 6: Inspect the files: lib.rs has `pub mod records_branch;` and the module docs cite §8.2 and D111. Search the other crates for callers. Run: `cargo build`. Observes: The build succeeds. A Grep for `records_branch` outside crates/athena-git returns nothing. `git diff` shows Cargo.toml and Cargo.lock unchanged.
- Criterion 7: Review the diff of CHANGELOG.md and docs/design/records-history.md, and list the changed paths. Run: `git status`. Observes: The changed files are only records_branch.rs, lib.rs, CHANGELOG.md and records-history.md. The CHANGELOG [Unreleased] section has an entry. records-history.md marks the writer built. Nothing in athena.toml, defaults.toml, workflows or records/ appears.

Planned files:

- crates/athena-git/src/records_branch.rs (create)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-git/Cargo.toml (modify)
- Cargo.lock (modify)

## The change

Base 03856d9226d8 → candidate ca59ef63db71. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   8 +
 crates/athena-git/src/lib.rs            |   4 +-
 crates/athena-git/src/records_branch.rs | 545 ++++++++++++++++++++++++++++++++
 docs/design/records-history.md          |  10 +-
 4 files changed, 565 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1033 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
