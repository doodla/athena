# Job 20261024-033933-ta60: verify "Add athena/records branch writer with CAS and bounded retry"

## Request (verbatim)

> Build the records-branch writer of the records-history plan (docs/design/records-history.md section 8.2, decision D111, job 1 part 2): in athena-git, a writer that commits opaque record files to the branch athena/records (its own root, never merged into main) with compare-and-swap and bounded retry. Tests: two simultaneous writers both land, each as its own path, fast-forward only; a rejected update refetches, replays and succeeds; a write that would modify, delete or rename an existing path is refused and the tip is unchanged; the branch has no parent in common with main. Nothing in the job pipeline calls it yet.

## Reading (accepted by the user)

Add a new module in athena-git that appends opaque record files to the orphan branch athena/records. It uses a compare-and-swap ref update, fast-forward only, and retries a bounded number of times by refetching and replaying. It refuses any write that would modify, delete or rename an existing path. Nothing in the job pipeline calls it. The audience is the maintainer and the verifier, who check it by running the tests.

Assumptions:
- The writer works on a local repository's ref, and optionally on a remote through the existing guarded push (push_if_remote_at). The local ref update uses `git update-ref <ref> <new> <old>`, which takes git's lock and acts as the compare-and-swap. The 'simultaneous writers' test uses two threads on one repository, or two clones pushing to a bare remote.
- Whether to also implement the remote variant is the producer's call. Local plus a bare-remote test is the minimum that matches §8.2 'Local repositories use the same ref update'.
- Records are opaque bytes: the writer does not validate schemas, because the athena-data record types are job 1 part 1 and are out of this change.
- The retry count defaults to 5 (the design's example) and can be set as a parameter.
- The first write creates a root commit with no parents, using an empty base tree. The commit uses a fixed Athena identity, like Git::isolated, with signing configurable through the existing `sign` flag.
- The branch name is the constant `athena/records`.
- The goal is the writer plus its tests, and the maintainer will build the later jobs (importer, landing) on it.
- Existing-path refusal is checked against the latest fetched tip on each attempt, so a replay after rejection also refuses if the winner took the same path.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-git`. A test starts two writers at once (threads) against one repository, with different record paths. Both land, each path is present on the final tip, every update was a fast-forward (each tip descends from the previous one), and no commit was lost.
2. [behavior] Run `cargo test -p athena-git`. A test makes the ref move between the writer's read and its update (a stale expected tip). The first update is rejected, the writer refetches and replays its new files on the new tip, and the write succeeds. The earlier commit and the new files are both in the final tree.
3. [behavior] Run `cargo test -p athena-git`. Tests show that a write that modifies an existing path, deletes one, or renames one (a delete plus an add of the same content) is refused with an error. In each case the branch tip after the call equals the tip before. A write with identical bytes at an existing path is also tested, and its result is documented.
4. [behavior] Run `cargo test -p athena-git`. A test shows that `athena/records` shares no history with main: `git merge-base main athena/records` finds nothing, and the first records commit has no parent. A second test shows the retry bound is enforced. When every attempt is rejected, the writer returns a clear error after the bounded count and does not loop.
5. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean. `cargo test` passes for the workspace.
6. [content] A writer API exists in athena-git (a new public module exported from lib.rs). It takes opaque bytes keyed by path, with doc comments that cite §8.2 and D111. It uses the existing Git wrapper, temp index and commit_tree helpers, with no new crate. A repository search finds no caller of it outside athena-git, so the pipeline is untouched.
7. [content] No governance file is changed (athena.toml, defaults.toml, workflows, lint tables). No file under records/ is touched.

## Expected deliverables

- crates/athena-git/src/records_branch.rs (create)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-git/Cargo.toml (modify)

## Out of scope

- New record types in athena-data
- The importer, landing intent and reconcile, and the pre-merge status (jobs 2 to 4)
- Wiring the writer into the job pipeline
- Confirming host branch rules (no force-push, no deletion) on the real host
- The history check that rejects commits that modify records

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only, inside the athena-git crate.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for each requested behavior; cargo test and clippy must pass.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing Git wrapper, TempIndex and commit_tree helpers and the git CLI's atomic update-ref; add no crate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new module plus its export; no pipeline changes.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the concurrency, retry, refusal and no-common-ancestor behaviors instead of checking that the code exists.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite the design section and decision.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Tests use synthetic identities and no personal data.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test: two threads with their own Git values on one temp repo, released by a Barrier, each writing a different path via RecordsWriter::write (max 5). Join, inspect branch. Run: `cargo test -p athena-git records_branch`. Observes: Both writes Ok; both paths in ls_tree of final tip; 3 commits (root + 2); each commit's parent is the previous one; both writers' commits in the chain.
  Fails without the behavior: Without CAS one commit is dropped (path missing or commit absent); without retry the loser returns an error.
- Criterion 2: Unit test using the before_update hook: on attempt 1 the hook advances the ref with another write after the writer read the tip. Run: `cargo test -p athena-git records_branch`. Observes: First update rejected, outcome reports 2 attempts, final tree holds the hook's file and the writer's file, final commit's parent is the hook's commit.
  Fails without the behavior: Without refetch and replay the call errors or overwrites the hook's commit, so the earlier file is missing.
- Criterion 3: Unit tests: seed a path, save tip; write (a) different bytes at it, (b) Delete, (c) Delete plus Put of same bytes at new path (rename), (d) identical bytes, (e) file/dir clash. Assert Err(Refused) and unchanged tip. Run: `cargo test -p athena-git records_branch`. Observes: Each case errors and rev_parse of the branch equals the saved tip. Case (d) is refused; test comment and module docs state identical bytes count as an existing path.
  Fails without the behavior: Without the existing-path check, modify, delete and rename succeed and move the tip, failing the tip-equality assertions.
- Criterion 4: Test 1: repo_with a main commit, write one record, run `git merge-base` via Git::exit and check parents of the first records commit. Test 2: max_attempts 3, hook moves the ref every attempt and counts calls. Run: `cargo test -p athena-git records_branch`. Observes: Test 1: merge-base exits 1 with empty output; first commit has no parents. Test 2: error naming the attempt count after exactly 3 hook calls; no loop; tip is the hook's last commit without the writer's files.
  Fails without the behavior: A writer branching from HEAD shares history so merge-base prints a commit; an unbounded loop hangs or exceeds 3 hook calls.
- Criterion 5: Run the format, lint and full test commands as three separate whole commands. The new tests are part of the workspace run, so the test run executes the behaviors of criteria 1 to 4; the lint run covers the new module under -D warnings. Run: `cargo fmt --all --check; cargo clippy --all-targets --locked -- -D warnings; cargo test --locked`. Observes: Each exits 0 with no warnings, and the test run lists the records_branch tests as passed.
  Fails without the behavior: Unformatted code fails fmt --check; any warning in the new module (e.g. unused code, needless clones) fails clippy -D warnings; a broken or missing records_branch test, or a regression elsewhere in the workspace from the lib.rs export, fails cargo test.
- Criterion 6: Inspect the diff and search for callers. Run: `git diff --stat`. Observes: lib.rs gains only `pub mod records_branch;`; module docs cite §8.2 and D111; only Git, TempIndex and commit_tree are used; Cargo.toml adds no dependency; a Grep for records_branch and RecordsWriter finds hits only under crates/athena-git.
- Criterion 7: Check the changed paths. Run: `git status`. Observes: Changed files are limited to records_branch.rs, lib.rs, CHANGELOG.md, docs/design/records-history.md and crates/athena-git/Cargo.toml (description only). No athena.toml, defaults.toml, workflow, lint table or records/ file appears.

Planned files:

- crates/athena-git/src/records_branch.rs (create)
- crates/athena-git/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-git/Cargo.toml (modify)

## The change

Base 61386f6fdac6 → candidate 6add67d57e05. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   8 +
 crates/athena-git/Cargo.toml            |   2 +-
 crates/athena-git/src/lib.rs            |   1 +
 crates/athena-git/src/records_branch.rs | 479 ++++++++++++++++++++++++++++++++
 docs/design/records-history.md          |   5 +
 5 files changed, 494 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1027 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
