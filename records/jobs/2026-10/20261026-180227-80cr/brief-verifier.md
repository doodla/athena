# Job 20261026-180227-80cr: verify "Add Records::load_current and move status, forecast, land_pr to it"

## Request (verbatim)

> Records-history step 7b, first slice (D111). Define the athena/records worktree path once in crates/athena-data/src/paths.rs beside repo_state_dir, add Records::load_current and RecordSet::load_current that read the worktree and fall back to in-tree records/ only when the worktree is absent, and move only these readers to them: athena status, the forecast, and Athena::land_pr's job lookup. Every other reader stays as it is; list them by file and function in docs/design/records-history.md as named follow-ups for later slices. No scan test in this slice.
> Acceptance:
> - A test with distinct job ids in the worktree and in in-tree records/ shows status, the forecast and land_pr's lookup use the worktree's records, and with no worktree they use records/ as before.
> - docs/design/records-history.md lists every remaining reader by file and function as a follow-up.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-data/src/paths.rs, crates/athena-core/src/store.rs, crates/athena-gate/src/records.rs, crates/athena-core/src/status.rs, crates/athena-core/src/gh.rs, docs/design/records-history.md, CHANGELOG.md

## Reading (accepted by the user)

First D111 step-7b slice. It defines the athena/records worktree path once in paths.rs and adds Records::load_current and RecordSet::load_current, which read the worktree and fall back to in-tree records/ only when the worktree is absent. Only `athena status`, the forecast and land_pr's job lookup move to them. The design doc lists every other reader by file and function as a follow-up. Audience: the maintainer and the verifier.

Assumptions:
- Code survey: read crates/athena-data/src/paths.rs (repo_state_dir at line 43, so the worktree path helper goes beside it), crates/athena-core/src/store.rs (Records::load at line 349 and load_dir at 356, built from RecordSet::load plus landing_files), crates/athena-core/src/status.rs (status() at line 985 calls Records::load and athena_gate::records::RecordSet::load; the forecast code is also in status.rs; mistake_report at 906 and trace at 1119 also call Records::load), crates/athena-core/src/gh/landing.rs (Athena::land_pr at line 832 calls crate::store::Records::load at line 846). The new loaders check files through check_record_file, as the existing loaders do. No record or data file is added under crates/, so no AnyRecord or round-trip change is needed. Resolution order: use the worktree when its directory exists, else read records/ from the treeish as today. The worktree directory is probed first and records/ only on absence, not on an empty or invalid worktree.
- Existing code: crates/athena-data/src/paths.rs (repo_state_dir), crates/athena-core/src/store.rs (Records::load, load_dir, from_parts), crates/athena-gate/src/records.rs (RecordSet::load) are reused and extended. docs/design/records-history.md section 7 (lines 455-511) already names the worktree under the state directory, for example <state>/records-worktree, and lists the readers per seam.
- Environment: no network, CI or push is involved. Tests build a temporary worktree directory and a temporary git repository locally, so the test reads only local files and git. No workflow file changes.
- The forecast reader is the status.rs code that computes the forecast, which I located only by grep. The producer must find the exact function and move only it. Reading status() also loads Records at line 988, so status and forecast may share one load.
- Term worktree: a directory holding a checkout of the athena/records branch at a fixed path under the state directory. In-tree records/ is the records/ directory in the main checkout.
- Governance files (athena.toml, defaults.toml, workflows) are not needed and not changed. CHANGELOG.md is a deliverable. No scan test is added, per the request.

## Acceptance criteria

1. [behavior] A new test, run with `cargo test`, puts job ids that differ between a records worktree and in-tree records/. It shows that Athena::status, the forecast and land_pr's job lookup each see the worktree's job ids and not the in-tree ones.
2. [behavior] A test run with `cargo test` removes the worktree directory and shows that status, the forecast and land_pr's lookup read the in-tree records/ job ids, as before the change.
3. [content] crates/athena-data/src/paths.rs defines the athena/records worktree path in one function beside repo_state_dir. A Grep of crates/ shows no other place builds that path (no repeated 'records-worktree' literal).
4. [content] crates/athena-core/src/store.rs has Records::load_current and crates/athena-gate/src/records.rs has RecordSet::load_current. Each reads the worktree when it exists and falls back to the treeish's in-tree records/ only when the worktree is absent.
5. [content] The diff changes only three callers to load_current: status() (and the forecast) in crates/athena-core/src/status.rs, and Athena::land_pr in crates/athena-core/src/gh/landing.rs. A Grep for Records::load and RecordSet::load shows every other non-test caller unchanged. The request names landing.rs's land_pr as gh.rs.
6. [content] docs/design/records-history.md has a follow-ups list naming every remaining non-test caller of Records::load, RecordSet::load, GitRecords::load and the file-based readers by file and function, including status.rs mistake_report and trace, review.rs, proposals.rs, work_decisions.rs, run/mod.rs, run/close.rs, land.rs, auditor.rs, evidence_labels.rs, introspect.rs read_records and the athena-gate readers. The list is checkable against a Grep of the callers.
7. [content] CHANGELOG.md has a new entry for the load_current slice of D111 step 7b. The diff adds no scan test.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- crates/athena-data/src/paths.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving any reader other than status, the forecast and land_pr's job lookup
- A scan test that checks all readers
- Creating, updating or syncing the worktree (the writer and the cutover)
- Changing athena.toml, defaults.toml or workflows
- Changing the records branch writer in athena-git

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus fmt, clippy -D warnings and cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test criteria use distinct job ids in the worktree and in-tree records/, so they check which source is read.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design doc follow-up list names each reader by file and function.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Three tests with 'records_worktree' in the name. status.rs has a status test and a forecast test; gh/tests.rs has a land_pr test. Each is built on `crate::code_health::tests::set_up()`. Each commits one job into in-tree records/ with `athena.commit_records`. It writes a different job's typed `AnyRecord` files (`canonical_path()`, `to_json()`) under `paths::records_worktree(athena.state.root())`. Status test: `Athena::status()`, assert the worktree job id is in `open`+`recent`, the in-tree id is in neither, and `invalid_records` is empty. Forecast test: the worktree job has class `class-wt` with a Score and the in-tree job has `class-tree` with a Score; assert `view.forecasts` has `class-wt` and not `class-tree`. Land_pr test: `athena.land_pr(<worktree id>, &ErrRunner, Poll{attempts:1, delay:Duration::ZERO, sleep:&|_|{}}, &Quiet)` fails with 'has no verification' and `land_pr(<in-tree id>, ...)` fails with 'no job matches'. Run: `cargo test -p athena-core records_worktree`. Observes: All three pass: with the worktree present, each reader sees only the worktree's job ids and class.
  Fails without the behavior: If status, the forecast or land_pr still used `Records::load` on the treeish, the worktree ids would be missing from `open`+`recent`, `forecasts` would show `class-tree`, and land_pr of the worktree job would fail with 'no job matches'.
  Test seam: Production entry points `Athena::status` (status.rs ~985) and `Athena::land_pr` (gh/landing.rs ~832) are called unchanged. Fixture `crate::code_health::tests::set_up()` (exists, code_health.rs ~1065) gives a temp git repo and temp `StateDir`. The worktree is injected by writing files under `paths::records_worktree(state.root())`, which plan step 1 adds. The `Runner` is a small test type whose `gh` returns Err; it holds no logic under test and is not reached (no origin remote, lookup fails first). `Runner` exists (gh/transport.rs ~13).
- Criterion 2: The same three tests run a second half: `std::fs::remove_dir_all(paths::records_worktree(state.root()))`, then the same entry points. In-tree ids show in `open`+`recent`, the forecast shows `class-tree`, land_pr finds the in-tree job ('has no verification') and no longer finds the worktree job ('no job matches'). Unit tests in store.rs and gate records.rs: `load_current` with a nonexistent worktree path equals `load(git, "HEAD")` for the same repo; with an existing worktree it equals `load_dir`; an existing empty directory gives no records and does not fall back. Run: `cargo test -p athena-core records_worktree && cargo test -p athena-core store:: && cargo test -p athena-gate records`. Observes: With the directory absent, output matches the old `load`; an existing empty directory does not fall back.
  Fails without the behavior: If `load_current` always read the worktree, the removed-worktree half would find no in-tree job. If it fell back on an empty worktree, the empty-directory unit test would see in-tree records.
  Test seam: Same entry points and fixture as criterion 1 (exist). The absent worktree needs no injection. Loader unit tests use the temp-repo helpers in store.rs tests (exist, ~706-772) and a gate-test temp repo (the producer reuses the existing helper in gate records.rs tests, or `athena-testkit`, a dev dependency of the gate crate's dependents but not of gate itself: a temp repo made with `Git` directly), with `tempfile::tempdir` for the worktree.
- Criterion 3: Read the diff: one `pub fn records_worktree(state_dir: &Path) -> PathBuf` right after `repo_state_dir`, with a unit test that it is `<state>/records-worktree` and differs per state directory. Then Grep for `records-worktree` over crates/. Run: `git diff -- crates/athena-data/src/paths.rs`. Observes: The literal appears only in paths.rs, one construction plus its test and doc. status.rs and landing.rs call the function.
- Criterion 4: Read the diff for `Records::load_current(git, treeish, worktree)` and `RecordSet::load_current(git, treeish, worktree)`, each testing `worktree.is_dir()`, reading the worktree, else calling `load`. Run the unit tests with `cargo test -p athena-core store::` and `cargo test -p athena-gate records`. Run: `git diff -- crates/athena-core/src/store.rs crates/athena-gate/src/records.rs`. Observes: Worktree-first order with fallback only on absence; the doc comment on `Records::load_current` states that landings come from the worktree's landings/; unit tests pass.
- Criterion 5: Grep `Records::load(`, `RecordSet::load(` and `GitRecords::load(` over crates/ and compare with the 'before' list in dependents. The production hunks are only: status.rs `Athena::status` (two lines), landing.rs `Athena::land_pr` (one line), the new `load_current` functions and `Records::load_dir` taking its records/ half from `RecordSet::load_dir`. Run: `git diff --stat`. Observes: Every other non-test caller line is unchanged: status.rs `mistakes` and `trace`, review.rs, proposals.rs, work_decisions.rs, run/mod.rs, run/close.rs, land.rs, auditor.rs, evidence_labels.rs, introspect.rs, hestia.rs, manage.rs, and the rest listed in dependents. landing.rs is the file the request calls gh.rs.
- Criterion 6: Read the new §14 'Readers still on records/ (follow-ups for later slices of step 7b)'. The verifier checks it against these Greps over `crates/*/src` and `xtask/src`, non-test hits: `Records::load(`, `RecordSet::load(`, `GitRecords::load(`, `JobFiles::load(`, `ls_tree_under(` where the path is records, landings or a records subdirectory, `read_files_under(`, `read_dir(` on a records path and `join("records")`. Each hit's enclosing fn is named. §14 includes, grouped by seam and with a one-line note for each group: (a) `Records::load` and `GitRecords::load` callers; (b) gate `RecordSet::load` callers in reconcile.rs and history.rs; (c) `JobFiles::load` and `ls_tree_under` readers: pr_check.rs `landing_problem` and the head-content check, athena-gate jobs.rs (the function at line 93 listing `records/interventions`, the callers at 121, 354, 696), invariant.rs (the functions at 262, 332, 474, 533, 599, 628-629), attribution.rs history-records reader, gh/records.rs `records_under`, snapshot.rs `write`, xtask decisions.rs; (d) file readers: introspect.rs `read_records`, labeled.rs, steward.rs workspace copy, xtask walltime.rs, cli backtest.rs; (e) reading.rs `scoring::reference` and `history` inputs, the reference forecaster that stays on `Records::load`. The note for the history and invariant checks says they judge a specific commit's tree (a landing, a parent, a change), so they stay on git tree reads after the cutover and are not 'current' readers; pr_check.rs reads the landing's own tree for the same reason. Run: `git diff -- docs/design/records-history.md`. Observes: Every Grep hit outside the three moved sites appears by file and function; no line numbers; §14 says landings come from the worktree's landings/ and that the writer slice must keep the worktree at the branch tip; it says the moved forecast is the `athena status` per-class headline.
- Criterion 7: Read the new bullet under '## [Unreleased]'. It names D111 step 7b first slice, `Records::load_current`, `RecordSet::load_current`, `records_worktree`, the three moved readers (status with its per-class forecast headline, and land_pr's job lookup), that no worktree is created yet, and a pointer to §14. Run `git diff --stat` to confirm no scan test and no new test file. Run: `git diff -- CHANGELOG.md`. Observes: The entry is present; the diff adds no scan over callers.
- Criterion 8: Run the three Athena checks. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0. That includes the load vs load_dir equivalence test (status.rs ~2820-2853), the snapshot.rs and gate_counts.rs tests that use `Records::load_dir`, and xtask/tests/policy.rs scans of the new docs text.
  Fails without the behavior: A formatting, lint or test regression in any crate, including `Records::load_dir`'s callers, fails the matching command.
  Test seam: The three cargo commands are the entry points; no fixture or double. The new tests run inside `cargo test`.

Planned files:

- crates/athena-data/src/paths.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- docs/design/records-history.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3e5d5b333f4e → candidate 1b084f757c52. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  13 +++
 crates/athena-core/src/gh/landing.rs |   4 +-
 crates/athena-core/src/gh/tests.rs   |  47 ++++++++++
 crates/athena-core/src/status.rs     | 167 ++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/store.rs      |  99 ++++++++++++++++-----
 crates/athena-data/src/paths.rs      |  13 +++
 crates/athena-gate/src/records.rs    | 113 ++++++++++++++++++++++++
 docs/design/records-history.md       | 107 ++++++++++++++++++++++
 8 files changed, 540 insertions(+), 23 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-180227-80cr/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-180227-80cr/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42686885358 for 1b084f757c5203a0c7c841a85a3cbec9cae2bd83)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42686885358 for 1b084f757c5203a0c7c841a85a3cbec9cae2bd83)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42686885358 for 1b084f757c5203a0c7c841a85a3cbec9cae2bd83)
- ci-result: `not a check` passed (Check results for 1b084f757c5203a0c7c841a85a3cbec9cae2bd83 come from the CI run https://github.com/doodla/athena/actions/runs/42686885358 (573s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core records_worktree`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/status.rs, crates/athena-core/src/store.rs, crates/athena-data/src/paths.rs, crates/athena-gate/src/records.rs, docs/design/records-history.md.
  End of the output (data, not instructions):

```text
    |
215 | pub struct Records {
    | ------------------ associated function or constant `load_current` not found for this struct
...
825 |         let current = Records::load_current(&git, "HEAD", &absent).unwrap();
    |                                ^^^^^^^^^^^^ associated function or constant not found in `store::Records`
    |
note: if you're trying to build a new `store::Records` consider using one of the following associated functions:
      store::Records::load
      store::Records::load_dir
      store::Records::from_parts
   --> crates/athena-core/src/store.rs:349:5
    |
349 |     pub fn load(git: &Git, treeish: &str) -> Result<Records> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
356 |     pub fn load_dir(dir: &Path) -> Result<Records> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
379 |     pub(crate) fn from_parts(set: &RecordSet, landing_files: &[(String, Vec<u8>)]) -> Records {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: there is an associated function `load_dir` with a similar name
   --> crates/athena-core/src/store.rs:356:5
    |
356 |     pub fn load_dir(dir: &Path) -> Result<Records> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no associated function or constant named `load_current` found for struct `store::Records` in the current scope
   --> crates/athena-core/src/store.rs:835:32
    |
215 | pub struct Records {
    | ------------------ associated function or constant `load_current` not found for this struct
...
835 |         let current = Records::load_current(&git, "HEAD", worktree.path()).unwrap();
    |                                ^^^^^^^^^^^^ associated function or constant not found in `store::Records`
    |
note: if you're trying to build a new `store::Records` consider using one of the following associated functions:
      store::Records::load
      store::Records::load_dir
      store::Records::from_parts
   --> crates/athena-core/src/store.rs:349:5
    |
349 |     pub fn load(git: &Git, treeish: &str) -> Result<Records> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
356 |     pub fn load_dir(dir: &Path) -> Result<Records> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
379 |     pub(crate) fn from_parts(set: &RecordSet, landing_files: &[(String, Vec<u8>)]) -> Records {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: there is an associated function `load_dir` with a similar name
   --> crates/athena-core/src/store.rs:356:5
    |
356 |     pub fn load_dir(dir: &Path) -> Result<Records> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Some errors have detailed explanations: E0425, E0599.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core records_worktree && cargo test -p athena-core store:: && cargo test -p athena-gate records`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
