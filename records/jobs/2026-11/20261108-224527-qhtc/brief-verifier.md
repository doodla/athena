# Job 20261108-224527-qhtc: verify "Make both snapshot locks attempt a write at every path"

## Request (verbatim)

> try again on 20261027-054321-rxa7

## Reading (accepted by the user)

Retry of job 20261027-054321-rxa7 (failed in planning; no candidate kept). Fix the class in both copies of the two-place-probe lock, `Snapshot::lock` in athena-git and `lock` in athena-core's snapshot.rs, via one shared routine that attempts a write at every path and names the writable one. The audience is the Athena operator, who should not get verdicts voided by a partly locked snapshot.

Assumptions:
- Code survey: read crates/athena-git/src/snapshot.rs (Snapshot::lock line 161, probe_write line 182, set_write_bits, set_immutable) and crates/athena-core/src/snapshot.rs (lock line 133, probe_write line 155); both probe only a root file and one other file. Callers: crates/athena-core/src/run/verification.rs already turns a lock error into Incomplete before the verifier runs; that stays unchanged. No records or data files are added, so no record validation is touched.
- Job 20261027-054321-rxa7 failed at plan critique round 1 with no candidate, so nothing is kept; its findings are addressed: (a) the second copy in athena-core is fixed here; (b) tests bind root by building the partial lock with immutable/attempted-write or an injected writable path, not mode bits alone; (c) a no-leftover-probe check is made on the real lock then unlock path; (d) the error distinguishes 'accepted a write' from 'could not be checked'; (e) the CHANGELOG states only what the diff does and does not blame earlier jobs.
- Shared routine: `writable_paths` is a pub function in athena-git (athena-core depends on athena-git) that both locks call; athena-core's probe_write is deleted.
- The suite may run as root, so tests must use an attempted write and fail against the old two-place probe.
- Verification.rs, run_pin_tests.rs and the rerun policy are dropped from scope unless a compile fix needs them; the earlier-listed deliverables there are not kept as planned changes.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-git`: a new test leaves one nested path (e.g. .git/objects) writable in an otherwise locked tree, constructed so it binds root too (attempted write or injected writable path, not mode bits alone), and asserts `Snapshot::lock` returns an error naming exactly that path. It fails against the old two-place probe.
2. [behavior] Run `cargo test -p athena-core`: a new test in crates/athena-core/src/snapshot.rs does the same for the evidence-snapshot `lock`, asserting the error names the nested writable path, and it fails against the old probe when run as root.
3. [content] crates/athena-core/src/snapshot.rs `lock` calls the shared athena-git routine, and its own `probe_write` is deleted (Grep finds no `fn probe_write` there); athena-git's `lock` uses the same routine over every directory and file including .git.
4. [behavior] Run `cargo test -p athena-git`: a test shows that after a successful `lock()` then `unlock()` no `.athena-lock-probe` file exists anywhere in the tree and the work-tree tree hash equals its value before the lock.
5. [behavior] Run `cargo test -p athena-git`: a test shows the lock error text differs between a path that accepted a write and a path that could not be checked (e.g. 'could not be listed').
6. [behavior] Run `cargo test -p athena-git` and `cargo test -p athena-core`: existing tests still pass, including `a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree` and the snapshot-change rerun tests in run_pin_tests.rs.
7. [content] CHANGELOG.md [Unreleased] has an entry stating that both locks now attempt a write at every path and fail naming the writable one; it does not assert that the earlier two-place probe caused jobs 20261015-031159-xx7n or 20261015-033631-x3a1.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Expected deliverables

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/snapshot.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Changing the rerun-once policy or tree comparison in verification.rs beyond surfacing a lock failure
- Hashing gitignored files such as target/
- studies/firstmate-work-handling/*, xtask/src/site.rs, docs/design/explainer-site.md (earlier candidate changes dropped)
- records/, governance files (athena.toml, defaults.toml, workflows) and prompts

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Both lock copies fixed via one shared routine.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests attempt real writes, as root too.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check a partly locked snapshot is refused.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Two source files plus CHANGELOG.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in crates/athena-git/src/snapshot.rs: a partly locked snapshot is refused and the error names the one reopened path. Run: `cargo test -p athena-git a_lock_with_one_nested_path_left_writable_is_refused_naming_it`. Observes: Calls `Snapshot::lock_with` on `verifier_snapshot(..)`. Before locking, the test creates the empty dir `.git/objects/leaf`. The hook runs the existing `unlock_dir` on `snap.dir().join(".git/objects/leaf")`, which clears chattr +i and restores the owner write bit, so root can write there. Asserts `Err(GitError::Refused(msg))` where msg contains the backticked relative path `.git/objects/leaf` and exactly two backticks (so one path is named), and does not contain 'could not be checked'. Asserts the snapshot is unlocked afterwards (lock unlocks on failure): `try_writes` gives (true, true, true).
  Fails without the behavior: With the old probe (create in root, append .git/HEAD) every checked path is still locked, so `lock_with` returns Ok and `unwrap_err` panics. The write is really attempted, so this holds as root too.
  Test seam: Production entry point: `Snapshot::lock_with` (which `lock` calls with a no-op). Seam: the `after_apply` closure running the real `unlock_dir` on a leaf directory; no double replaces the probe. Added by plan step 2. `verifier_snapshot` and `unlock_dir` exist today.
- Criterion 2: Unit test in crates/athena-core/src/snapshot.rs for the evidence-snapshot lock. Run: `cargo test -p athena-core an_evidence_lock_with_one_nested_path_left_writable_is_refused_naming_it`. Observes: Builds a snapshot with the existing `repo(..)` helper and `write(&git,"HEAD",&dest)`, then `unlock(&dest)`, creates the empty dir `records/a/leaf`, and calls `lock_with(&dest, hook)`. The hook calls `athena_git::snapshot::unlock_dir(dest/records/a/leaf)`. Asserts the error from `format!("{:#}", err)` contains 'evidence snapshot lock does not hold' and the backticked path `records/a/leaf` with exactly two backticks. The `Unlock` guard cleans up.
  Fails without the behavior: The old probe checked the root and `history.json` only, so `lock_with` would return Ok for the reopened leaf and `unwrap_err` panics, as root too.
  Test seam: Production entry point: `snapshot::lock_with` (called by `lock` and `write`). Seam: the hook closure with the real `unlock_dir`. Added by plan step 4. `repo` and `Unlock` exist today.
- Criterion 3: Show the diff and run the test that exercises the shared routine over the whole tree. Run: `git diff -- crates/athena-core/src/snapshot.rs crates/athena-git/src/snapshot.rs`. Observes: The diff removes `fn probe_write` from crates/athena-core/src/snapshot.rs (Grep for `fn probe_write` there finds nothing) and shows `lock_with` there calling `athena_git::snapshot::writable_paths`. In athena-git it shows `Snapshot::lock_with` calling `writable_paths(&self.dir)`, a recursive walk that includes `.git`. `cargo test -p athena-git writable_paths_tries_a_write_at_every_directory_and_file` also asserts that, on an unlocked verifier snapshot, the accepted list contains the root, `.git`, `.git/objects`, `.git/HEAD` and `src/main.rs`.
- Criterion 4: Unit test on the real lock then unlock path in crates/athena-git/src/snapshot.rs. Run: `cargo test -p athena-git a_lock_then_unlock_leaves_no_probe_file_and_the_tree_unchanged`. Observes: Uses `verifier_snapshot`. Records `work_tree_tree()`, calls the public `lock().expect(..)` then `unlock()`. Walks `snap.dir()` recursively, `.git` included, and asserts no entry is named `.athena-lock-probe`. Asserts `work_tree_tree()` equals the earlier value. A second part calls `writable_paths` on the unlocked snapshot, so probes are created and removed in every directory, and asserts the same two facts.
  Fails without the behavior: A probe that is created but not removed (for instance an early return after the create) leaves the file in the walk and also changes the tree hash.
  Test seam: Production entry points: `Snapshot::lock`, `Snapshot::unlock`, `writable_paths` (added by step 1). No double; fixture is the real `verifier_snapshot` helper, which exists today.
- Criterion 5: Unit test comparing the two kinds of error text. Run: `cargo test -p athena-git the_lock_error_text_differs_for_an_accepted_write_and_an_unchecked_path`. Observes: Part one: `writable_paths` on a plain tempdir holding a writable file gives accepted paths and no unchecked ones; its `refusal("the snapshot", root)` text contains 'accepted' and not 'could not be checked'. Part two (cfg(unix)): a tempdir containing a Unix socket bound with `std::os::unix::net::UnixListener` and nothing else is locked with `set_tree_writable(false)` and best-effort `set_immutable`. `writable_paths` lists the socket under unchecked, and its refusal text contains 'could not be checked' and the socket's relative path. The assertion is on the socket's presence in unchecked, not on accepted being empty, so it does not depend on chattr's handling of sockets. The test asserts the two texts differ.
  Fails without the behavior: If every finding were worded the same, or unchecked paths were counted as locked, the 'could not be checked' assertion fails.
  Test seam: Production entry points: `writable_paths` and `WriteFindings::refusal`, which both locks use. Fixture: a real socket file in a real tempdir, no double. Both functions are added by step 1; the test is added by step 3.
- Criterion 6: Run both crate suites, which include the existing lock and rerun tests, unchanged. Run: `cargo test -p athena-git -p athena-core`. Observes: `a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree` still passes: `lock()` succeeds on a fully locked verifier snapshot including its .git, because the new walk finds nothing writable. The evidence test `an_unlocked_copy_accepts_a_write_and_the_locked_one_refuses_it`, the status.rs and run/tests.rs evidence tests, and the snapshot-change rerun tests in run_pin_tests.rs (via `faulty_verifier`) pass; the rerun tests never call the probe.
  Fails without the behavior: If the walk reported a path git leaves writable, or could not check a normal path (a regular .git file, an empty directory), `lock()` would return Err and the existing lock test and every verifier run in run_pin_tests.rs would fail.
  Test seam: Existing tests, unchanged and with no double; production entry points `Snapshot::lock` and `snapshot::write`. The seam exists today (read in the code); the walk behind it is added by steps 1, 2 and 4.
- Criterion 7: Read the CHANGELOG diff. Run: `git diff -- CHANGELOG.md`. Observes: One bullet under [Unreleased] / the first `### Fixed`: both snapshot locks (`Snapshot::lock` in athena-git and the evidence snapshot `lock` in athena-core) now attempt a write at every directory and file and fail naming the writable path, saying 'accepted a write' versus 'could not be checked'. It does not mention jobs 20261015-031159-xx7n or 20261015-033631-x3a1 or say the earlier probe caused them.
- Criterion 8: Run the three project checks, each as its own whole command, on the whole workspace: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, and the command below. All must exit 0. Run: `cargo test --locked`. Observes: The test run, with the new tests included, exits 0; the verifier also runs `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`, and each exits 0, so no formatting, lint or test failure comes from the new code.
  Fails without the behavior: A formatting, lint or test failure in the new code (for instance an unused `accepted` field, an unformatted test, or a failing new test) makes the matching command exit non-zero.
  Test seam: Production entry points: the cargo fmt, clippy and test commands over the workspace (Athena's own fmt, clippy and test checks). Fixture: the producer's working tree as changed by steps 1 to 5; no fake or injection, nothing to fake. The seam exists today; step 6 runs the same three commands.

Planned files:

- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/snapshot.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 609a766e3452 → candidate 86c97ff49807. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   7 +
 crates/athena-core/src/snapshot.rs |  69 +++++---
 crates/athena-git/src/snapshot.rs  | 319 +++++++++++++++++++++++++++++++++----
 docs/decision-links.toml           |   7 +
 docs/decisions.md                  |  17 ++
 5 files changed, 367 insertions(+), 52 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-224527-qhtc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-224527-qhtc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42535345006 for 86c97ff498079b016106d90fef218e6cac34488a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42535345006 for 86c97ff498079b016106d90fef218e6cac34488a)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42535345006 for 86c97ff498079b016106d90fef218e6cac34488a)
- ci-result: `not a check` passed (Check results for 86c97ff498079b016106d90fef218e6cac34488a come from the CI run https://github.com/doodla/athena/actions/runs/42535345006 (673s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-git a_lock_with_one_nested_path_left_writable_is_refused_naming_it`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/snapshot.rs, crates/athena-git/src/snapshot.rs, docs/decision-links.toml, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling block-buffer v0.10.4
   Compiling syn v3.0.7
   Compiling linux-raw-sys v0.12.1
   Compiling thiserror v2.0.21
   Compiling bitflags v2.13.2
   Compiling digest v0.10.7
   Compiling cpufeatures v0.2.17
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling sha2 v0.10.9
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling thiserror-impl v2.0.21
   Compiling tempfile v3.27.0
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
error[E0425]: cannot find value `LOCK_PROBE` in this scope
   --> crates/athena-git/src/snapshot.rs:862:64
    |
862 |                 .filter(|p| p.file_name().is_some_and(|n| n == LOCK_PROBE))
    |                                                                ^^^^^^^^^^ not found in this scope

error[E0599]: no method named `lock_with` found for struct `snapshot::Snapshot` in the current scope
   --> crates/athena-git/src/snapshot.rs:818:14
    |
 48 |   pub struct Snapshot {
    |   ------------------- method `lock_with` not found for this struct
...
817 |           let err = snap
    |  ___________________-
818 | |             .lock_with(|dir| unlock_dir(&dir.join(".git/objects/leaf")).unwrap())
    | |             -^^^^^^^^^ method not found in `snapshot::Snapshot`
    | |_____________|
    |

error[E0425]: cannot find function `writable_paths` in this scope
   --> crates/athena-git/src/snapshot.rs:838:24
    |
838 |         let findings = writable_paths(snap.dir());
    |                        ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `writable_paths` in this scope
   --> crates/athena-git/src/snapshot.rs:871:18
    |
871 |         assert!(!writable_paths(snap.dir()).accepted.is_empty());
    |                  ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `writable_paths` in this scope
   --> crates/athena-git/src/snapshot.rs:881:24
    |
881 |         let accepted = writable_paths(open.path());
    |                        ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `writable_paths` in this scope
   --> crates/athena-git/src/snapshot.rs:901:25
    |
901 |         let unchecked = writable_paths(closed.path());
    |                         ^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-git` (lib test) due to 6 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core an_evidence_lock_with_one_nested_path_left_writable_is_refused_naming_it`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/snapshot.rs, crates/athena-git/src/snapshot.rs, docs/decision-links.toml, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling serde_core v1.0.229
   Compiling memchr v2.8.3
   Compiling libc v0.2.190
   Compiling proc-macro2 v1.0.107
   Compiling zmij v1.0.23
   Compiling quote v1.0.47
   Compiling syn v3.0.7
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling winnow v1.0.4
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling getrandom v0.3.4
   Compiling itoa v1.0.18
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling winnow v0.7.15
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling thiserror v2.0.21
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling tempfile v3.27.0
   Compiling schemars_derive v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling regex-automata v0.4.18
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling equivalent v1.0.2
   Compiling indexmap v2.14.2
   Compiling schemars v1.2.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `lock_with` in this scope
   --> crates/athena-core/src/snapshot.rs:371:19
    |
371 |         let err = lock_with(&dest, |dir| {
    |                   ^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-git a_lock_then_unlock_leaves_no_probe_file_and_the_tree_unchanged`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/snapshot.rs, crates/athena-git/src/snapshot.rs, docs/decision-links.toml, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
error[E0425]: cannot find value `LOCK_PROBE` in this scope
   --> crates/athena-git/src/snapshot.rs:862:64
    |
862 |                 .filter(|p| p.file_name().is_some_and(|n| n == LOCK_PROBE))
    |                                                                ^^^^^^^^^^ not found in this scope

error[E0599]: no method named `lock_with` found for struct `snapshot::Snapshot` in the current scope
   --> crates/athena-git/src/snapshot.rs:818:14
    |
 48 |   pub struct Snapshot {
    |   ------------------- method `lock_with` not found for this struct
...
817 |           let err = snap
    |  ___________________-
818 | |             .lock_with(|dir| unlock_dir(&dir.join(".git/objects/leaf")).unwrap())
    | |             -^^^^^^^^^ method not found in `snapshot::Snapshot`
    | |_____________|
    |

error[E0425]: cannot find function `writable_paths` in this scope
   --> crates/athena-git/src/snapshot.rs:838:24
    |
838 |         let findings = writable_paths(snap.dir());
    |                        ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `writable_paths` in this scope
   --> crates/athena-git/src/snapshot.rs:871:18
    |
871 |         assert!(!writable_paths(snap.dir()).accepted.is_empty());
    |                  ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `writable_paths` in this scope
   --> crates/athena-git/src/snapshot.rs:881:24
    |
881 |         let accepted = writable_paths(open.path());
    |                        ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `writable_paths` in this scope
   --> crates/athena-git/src/snapshot.rs:901:25
    |
901 |         let unchecked = writable_paths(closed.path());
    |                         ^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-git` (lib test) due to 6 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-git the_lock_error_text_differs_for_an_accepted_write_and_an_unchecked_path`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-git -p athena-core`). Result: not run: only the first few distinct commands are run.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
