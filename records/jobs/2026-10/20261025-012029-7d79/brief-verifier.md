# Job 20261025-012029-7d79: verify "Push attribution checkpoints to the remote records branch"

## Request (verbatim)

> Checkpoint records never reach the remote: athena check --attribution on main and athena land write the checkpoint commit to the local athena/records only (for example 5c314d0 on 2026-10-25), and the records writer resets a local ref that is ahead of the remote before its next write, so the checkpoint is dropped and CI always replays in full. Push the checkpoint with the same writer, compare-and-swap and retry as other records.
> Acceptance:
> - A test shows a checkpoint written by a passing check is on the remote records branch afterwards.
> - A later landing keeps it (the branch only fast-forwards).
> - CI's incremental run then judges only commits after it.
> Files: crates/athena-gate/src/attribution.rs, crates/athena-git/src/records_branch.rs, crates/athena-core/src/gh.rs, CHANGELOG.md

## Reading (accepted by the user)

write_checkpoint in attribution.rs appends only to the local records branch via RecordsWriter::append, so the next write resets it away. Change it to use RecordsWriter::append_to_remote (compare-and-swap, retry) so a passing check's checkpoint reaches the remote and later landings fast-forward past it. Audience: maintainers and CI, whose incremental run should judge only commits after the checkpoint.

Assumptions:
- Code survey: read crates/athena-gate/src/attribution.rs: write_checkpoint (line ~727) builds a Checkpoint and calls RecordsWriter::append, which writes only the local ref; read_newest_checkpoint (~549) is the reader, and tests passing_check_writes_checkpoint, failing_check_writes_no_checkpoint, checkpoint_write_failure_keeps_verdict and put_checkpoint already exist. Read crates/athena-git/src/records_branch.rs: append_to_remote (~398) syncs from the remote, builds on the remote tip, pushes with push_if_remote_at (expected-tip guard), retries on rejection up to max_attempts, then returns RecordsError::Contended. Resolution order: sync_from(remote), fetch tip, plan, build_commit, push. Records validation: no new record or data file is added; the checkpoint is the existing HistoryRecord::Checkpoint under CHECKPOINT_DIR, already accepted by check_record_file / AnyRecord in crates/athena-data/src/records.rs, and the existing round-trip tests there cover it. The property is measured directly by reading the remote records ref in a test repo after the check, not via the local ref.
- Environment: the push goes to the configured git remote's records branch (a local bare repo in tests); write permission to the records branch; reads the remote tip. No CI workflow files change.
- The request lists crates/athena-core/src/gh.rs, which does not exist in the repository (no gh* under crates/athena-core/src); it is treated as not needing change, and the remote name is obtained from the existing caller context. If a remote name needs plumbing it goes through attribution.rs.
- A failed push stays a reported checkpoint_error and keeps the verdict, as checkpoint_write_failure_keeps_verdict requires.
- athena land also writes a checkpoint through the same write_checkpoint path, so one fix covers both; no athena.toml, defaults.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] A new test in crates/athena-gate/src/attribution.rs, run with `cargo test -p athena-gate`, shows that after a passing check the checkpoint file exists on the remote records branch (read from the bare remote, not the local ref).
2. [behavior] A test run by `cargo test -p athena-gate` shows that a later landing/records write after the checkpoint leaves the checkpoint on the remote and the remote tip is a descendant of the checkpoint commit (fast-forward only, no reset).
3. [behavior] A test run by `cargo test -p athena-gate` shows an incremental attribution run, starting from the remote checkpoint, judges only commits after its main_sha rather than replaying in full.
4. [behavior] A test in crates/athena-git/src/records_branch.rs or attribution.rs shows a checkpoint push that loses a race is retried on the new remote tip and still lands; a persistent push failure keeps the verdict and sets checkpoint_error.
5. [content] Grep of crates/athena-gate/src/attribution.rs shows write_checkpoint calls append_to_remote rather than append.
6. [content] CHANGELOG.md has an entry describing that checkpoints are now pushed to the remote records branch.
7. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Expected deliverables

- crates/athena-gate/src/attribution.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)

## Out of scope

- Changing the records record schema or adding record files
- Changing CI workflow files
- Creating crates/athena-core/src/gh.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy, fmt pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the write path so checkpoints are pushed, not the reset symptom.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests read the remote branch directly.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test/clippy/fmt.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New unit test `attribution::tests::checkpoint_reaches_the_remote_records_branch`. Build a repo with a bare `origin` (a helper `add_remote(&git)` that runs `init --bare` and `remote add origin <path>`), land two commits with `land()`, call `publish(&git)`, then run `run_check(&git, &head, now(), &incremental())`. Read the bare repo directly: `bare.rev_parse(RECORDS_REF)` equals the local tip and `bare.show_file(remote_tip, "checkpoints/<cp.id>.json")` parses to `HistoryRecord::Checkpoint(cp)` with `main_sha` = head and `records_sha` = the tip before the write. The remote tip's only parent is the old tip. Run: `cargo test -p athena-gate attribution::tests::checkpoint_reaches_the_remote_records_branch`. Observes: The checkpoint file exists on the bare remote's `athena/records` tip, read from the remote and not the local ref. `run.checkpoint_written` is Some and `checkpoint_error` is None.
  Fails without the behavior: With `RecordsWriter::append` the bare remote has no `checkpoints/` file and the show_file/remote-tip assertion fails.
  Test seam: Production entry point `athena_gate::attribution::run_check` with `Options::new(Mode::Incremental, true)` (remote defaults to "origin"). The fixture is a local bare git repo added as `origin` by the new test helper `add_remote` (step 5), which also adds `publish`. `run_check`, the `Options.remote` field and the `append_to_remote` call are added by steps 1-3. The bare repo is real git, not a double.
- Criterion 2: New test `attribution::tests::a_later_landing_fast_forwards_past_the_remote_checkpoint`. After the criterion-1 setup, write a landing record through the production writer: `RecordsWriter::new(&git).append_to_remote("origin", "landing", &[Change::put("landings/x.json", ..)], &[])`. Read the bare remote: `checkpoints/<cp.id>.json` is still in the tip's tree, `bare.is_ancestor(checkpoint_commit, new_tip)` is true, `git.parents(new_tip) == [checkpoint_commit]`, and `audit_history(&bare, new_tip)` is empty. Also assert the local ref equals the remote tip (not reset). Run: `cargo test -p athena-gate attribution::tests::a_later_landing_fast_forwards_past_the_remote_checkpoint`. Observes: The remote tip descends from the checkpoint commit with the checkpoint file intact, and nothing was reset.
  Fails without the behavior: With the old local `append`, `append_to_remote`'s `sync_from` resets the local ref to the remote tip, the checkpoint file is absent from the remote, and the file-present assertion fails.
  Test seam: Production entry points `run_check` then `RecordsWriter::append_to_remote`, which exists (records_branch.rs:398). The fixture is the bare `origin` from `add_remote` (step 5). The checkpoint commit id is the return of the check's remote tip read right after `run_check`.
- Criterion 3: New test `attribution::tests::ci_incremental_run_from_the_remote_checkpoint_judges_only_later_commits`. Land 3 commits and check them (checkpoint pushed). Land 2 more with the landing records written by `append_to_remote`. Simulate CI's fresh state: `git update-ref -d refs/heads/athena/records`, then `git fetch origin athena/records:athena/records` (the same fetch ci.yml:646 runs). Run `run_check(&git, &head, now(), &Options::new(Mode::Incremental, false))`. Assert `mode == Incremental`, `started_from.main_sha` equals the third commit, `commits_judged() == 2`, `records_judged() == 4` (checkpoints are excluded at attribution.rs:338), and `fallback_reason` is None. A second run on the same repo with the checkpoint absent (a control remote with no checkpoint) is a full replay of 5 commits. Run: `cargo test -p athena-gate attribution::tests::ci_incremental_run_from_the_remote_checkpoint_judges_only_later_commits`. Observes: The incremental run starts from the checkpoint that came from the remote and judges only the two later commits and their four records, not all five.
  Fails without the behavior: Without the push, the fetched local ref has no checkpoint, so `mode` is FullReplay, `fallback_reason` is "no checkpoint on the records branch" and `commits_judged` is 5, which fails the assertions.
  Test seam: Production entry point `run_check` with write off (CI mode), reading the local ref fetched from the bare remote. The fixture is the bare `origin` from `add_remote` and the fetch command above, run through `git.run`. The fetch is test setup and the logic under test (`run_check`) is not doubled. `add_remote` and `publish` are added in step 5.
- Criterion 4: Three tests. (a) records_branch.rs `a_checkpoint_shaped_push_that_loses_the_race_is_rebuilt_on_the_new_remote_tip_and_lands`: `append_to_remote_with_hook` with `Change::put("checkpoints/cp.json", ..)`, where hook attempt 1 has a rival clone push, then assert 2 attempts, the commit's parent is the rival tip, and both files are on the bare remote. (b) attribution.rs `checkpoint_lands_on_a_remote_that_moved_ahead_of_the_checked_tip`: a rival clone pushes a record to the bare remote after the local clone last synced, then `run_check` passes with `checkpoint_written` Some. The remote tip's parent is the rival tip, `records_sha` is the older judged tip, and the rival record is kept. (c) attribution.rs `checkpoint_write_failure_keeps_verdict` rewritten: after `git config remote.origin.pushurl <missing path>`, `run_check` returns `report.passed()` true, `checkpoint_written` None, `checkpoint_error` Some, and the bare remote tip is unchanged. A CLI test in binary.rs does the same through the binary: `athena check --attribution` exits 0 and prints "checkpoint not written". Run: `cargo test -p athena-git records_branch::tests::a_checkpoint_shaped_push_that_loses_the_race_is_rebuilt_on_the_new_remote_tip_and_lands && cargo test -p athena-gate attribution::tests::checkpoint_lands_on_a_remote_that_moved_ahead_of_the_checked_tip && cargo test -p athena-gate attribution::tests::checkpoint_write_failure_keeps_verdict && cargo test -p athena-cli --test binary incremental::check_attribution_push_failure_keeps_the_verdict`. Observes: A lost push race is retried on the new remote tip and the checkpoint lands. A remote that moved ahead is built upon. A persistent push failure leaves the verdict and exit code unchanged and sets `checkpoint_error`, with the remote untouched.
  Fails without the behavior: (a) without the rebuild-and-retry in `append_to_remote` the second attempt would not occur and the checkpoint would not land. (b) without the remote-tip build, the checkpoint would be based on the stale local tip and the push would be rejected. (c) without an error path, `checkpoint_error` would be None, or the verdict would flip.
  Test seam: (a) Existing private `append_to_remote_with_hook` (records_branch.rs:410) with a rival `RecordsWriter` on a second clone and the existing helpers `bare_remote`/`clone_of`, as in the existing test at line 871. (b) and (c) use `run_check` and the bare `origin` from `add_remote`; the pushurl is set with `git config` in the test (step 5). For the CLI, the binary runs against the Fixture plus a bare remote added by the new `remote_for` helper in binary.rs (step 7).
- Criterion 5: Inspect the diff of write_checkpoint. Run: `git diff -- crates/athena-gate/src/attribution.rs`. Observes: Inside `fn write_checkpoint` the call is `RecordsWriter::new(git).append_to_remote(remote, "checkpoint", ...)`; the `.append(` call is gone and the doc comment no longer says "local".
  Test seam: None. This is a content criterion.
- Criterion 6: Inspect the changelog diff. Run: `git diff -- CHANGELOG.md`. Observes: A new bullet under `## [Unreleased]` / `### Fixed` (or Added if there is no Fixed heading under Unreleased) states that the passing-check and `athena land` checkpoints are pushed to the remote `athena/records` with compare-and-swap and retry. It also states that the check refuses to write when its judged records tip is not on the remote and reports `checkpoint_error`, and that a later landing fast-forwards past it so CI's incremental run judges only later commits. It names the root cause: the local-only append was reset by the next remote write.
  Test seam: None. This is a content criterion.
- Criterion 7: Run the three checks Athena runs on the result. Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`. Observes: All pass, including the changed tests in athena-gate, athena-git and athena-cli (binary.rs, main.rs unit tests) and the budgets and boundary tests in athena-cli and xtask.
  Fails without the behavior: A missed test adaptation (a local `put` followed by a checkpoint-expecting run with no `publish`, or a CLI test with no `origin`) fails in cargo test.
  Test seam: None. These are the repository's own checks.

Planned files:

- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-git/src/records_branch.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base edf46a1e5edf → candidate 09fb59dd2603. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  12 +
 crates/athena-cli/src/main.rs           |  32 ++-
 crates/athena-cli/tests/binary.rs       |  72 +++++-
 crates/athena-gate/src/attribution.rs   | 389 +++++++++++++++++++++++++++++---
 crates/athena-git/src/records_branch.rs |  55 ++++-
 docs/design/records-history.md          |   9 +-
 6 files changed, 527 insertions(+), 42 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261025-012029-7d79/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261025-012029-7d79/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42431109610 for 09fb59dd26033e7ee7bfe36585d3666d457eb98b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42431109610 for 09fb59dd26033e7ee7bfe36585d3666d457eb98b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42431109610 for 09fb59dd26033e7ee7bfe36585d3666d457eb98b)
- ci-result: `not a check` passed (Check results for 09fb59dd26033e7ee7bfe36585d3666d457eb98b come from the CI run https://github.com/doodla/athena/actions/runs/42431109610 (765s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy --all-targets and cargo fmt --check all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-gate attribution::tests::checkpoint_reaches_the_remote_records_branch`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/main.rs, crates/athena-gate/src/attribution.rs, crates/athena-git/src/records_branch.rs, docs/design/records-history.md.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling quote v1.0.47
   Compiling version_check v0.9.5
   Compiling serde_core v1.0.229
   Compiling typenum v1.20.1
   Compiling generic-array v0.14.7
   Compiling libc v0.2.190
   Compiling memchr v2.8.3
   Compiling syn v3.0.7
   Compiling cfg-if v1.0.5
   Compiling getrandom v0.3.4
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling zmij v1.0.23
   Compiling thiserror v2.0.21
   Compiling digest v0.10.7
   Compiling serde v1.0.229
   Compiling rustix v1.1.5
   Compiling cpufeatures v0.2.17
   Compiling serde_json v1.0.151
   Compiling winnow v1.0.4
   Compiling sha2 v0.10.9
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling bitflags v2.13.2
   Compiling itoa v1.0.18
   Compiling jiff-core v0.1.1
   Compiling linux-raw-sys v0.12.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling tempfile v3.27.0
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling regex-automata v0.4.18
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling regex v1.13.1
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
error[E0425]: cannot find value `RECORDS_BRANCH` in this scope
    --> crates/athena-gate/src/attribution.rs:1118:35
     |
1118 |         git.push_branch("origin", RECORDS_BRANCH, &records_tip(git))
     |                                   ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `RECORDS_BRANCH` in this scope
    --> crates/athena-gate/src/attribution.rs:1570:35
     |
1570 |         git.push_branch("origin", RECORDS_BRANCH, first.trim())
     |                                   ^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-gate` (lib test) due to 2 previous errors
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-gate attribution::tests::a_later_landing_fast_forwards_past_the_remote_checkpoint`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/main.rs, crates/athena-gate/src/attribution.rs, crates/athena-git/src/records_branch.rs, docs/design/records-history.md.
  End of the output (data, not instructions):

```text
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
error[E0425]: cannot find value `RECORDS_BRANCH` in this scope
    --> crates/athena-gate/src/attribution.rs:1118:35
     |
1118 |         git.push_branch("origin", RECORDS_BRANCH, &records_tip(git))
     |                                   ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `RECORDS_BRANCH` in this scope
    --> crates/athena-gate/src/attribution.rs:1570:35
     |
1570 |         git.push_branch("origin", RECORDS_BRANCH, first.trim())
     |                                   ^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-gate` (lib test) due to 2 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-gate attribution::tests::ci_incremental_run_from_the_remote_checkpoint_judges_only_later_commits`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/main.rs, crates/athena-gate/src/attribution.rs, crates/athena-git/src/records_branch.rs, docs/design/records-history.md.
  End of the output (data, not instructions):

```text
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
error[E0425]: cannot find value `RECORDS_BRANCH` in this scope
    --> crates/athena-gate/src/attribution.rs:1118:35
     |
1118 |         git.push_branch("origin", RECORDS_BRANCH, &records_tip(git))
     |                                   ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `RECORDS_BRANCH` in this scope
    --> crates/athena-gate/src/attribution.rs:1570:35
     |
1570 |         git.push_branch("origin", RECORDS_BRANCH, first.trim())
     |                                   ^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-gate` (lib test) due to 2 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-git records_branch::tests::a_checkpoint_shaped_push_that_loses_the_race_is_rebuilt_on_the_new_remote_tip_and_lands && cargo test -p athena-gate attribution::tests::checkpoint_lands_on_a_remote_that_moved_ahead_of_the_checked_tip && cargo test -p athena-gate attribution::tests::checkpoint_write_failure_keeps_verdict && cargo test -p athena-cli --test binary incremental::check_attribution_push_failure_keeps_the_verdict`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
