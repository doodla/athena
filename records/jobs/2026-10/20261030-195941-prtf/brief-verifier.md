# Job 20261030-195941-prtf: verify "Retry job d103: read-only evidence snapshot and athena status --snapshot"

## Request (verbatim)

> try again on 20261030-070002-d103

## Reading (accepted by the user)

Retry of job 20261030-070002-d103, which failed only on low disk before the verifier (environment, not the work) and was never verified. Redo its change: at dispatch write a read-only snapshot of records/ and main's first-parent history outside the working copy, name it in producer and verifier briefs, and add `athena status --snapshot <dir>`. Audience: producers and verifiers who must check real-records criteria.

Assumptions:
- Previous result, job 20261030-070002-d103: it ended failed with no verdict because free disk was 9.3 GB against the 10.0 GB required before the verifier. Its one recorded reason is environmental, so there is no verifier finding to fix. Its candidate was discarded and this retry starts from base, redoing the same snapshot feature. Its 19 changed files are each addressed in deliverables (kept as the expected touch set) or out_of_scope.
- Disk hygiene (auto-cleanup, hold-as-waiting-for-disk) from introspection 20261030-195920-9eng is a separate follow-up and is out of scope here; this job only needs the disk to be free when it runs. That prevention was already tried in a prior case, so it is not re-proposed.
- Code survey: read crates/athena-core/src/status.rs (status body and throughput section), crates/athena-core/src/store.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/disk.rs (existing free-disk guard, untouched), crates/athena-core/src/run/ (dispatch writes the snapshot before the producer is spawned, briefs built afterwards), crates/athena-context/src/briefs.rs, crates/athena-cli/src/main.rs. The snapshot is runtime state under the state dir, not a record file under crates/, so check_record_file and AnyRecord do not apply. Throughput is measured by calling the same status function on the snapshot and on the main checkout and comparing the section text directly.
- The snapshot lives at <state>/jobs/<id>/evidence/ holding records/, landings/ and history.json (last 200 first-parent commits: hash, commit time, subject, trailers), locked read-only. Stored briefs use a <state> placeholder so no machine path is committed.
- Files outside the original scope (crates/athena-agent/src/lib.rs, policy.rs, claude.rs, crates/athena-core/src/reading.rs, run/planning.rs, run/verification.rs, admission.rs, tests/live.rs) are kept as likely needed: agent read policy must allow the evidence path, and the planner, verifier and reader briefs must carry it. They change only as needed for that; if the producer finds them unneeded they stay untouched.

## Acceptance criteria

1. [behavior] A test through the real dispatch path (run `cargo test -p athena-core`) shows the snapshot directory exists at the path the briefs name, is read-only (a write attempt fails), contains the records copied at dispatch, and contains main's first-parent history (hash, commit time, subject, trailers) at dispatch.
2. [behavior] A test (run `cargo test -p athena-core status`) shows `athena status` run against the snapshot gives a throughput section identical to the main checkout's for the same records and history.
3. [behavior] A CLI test (run `cargo test -p athena-cli`) shows `athena status --snapshot <dir>` reads records and history from the snapshot and works with no records/ directory in the current directory.
4. [behavior] Tests (run `cargo test -p athena-context`) show the producer and verifier briefs both state the snapshot path and say real-records checks use it; the working copy still has no records/.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has a new entry describing the evidence snapshot and the `athena status --snapshot` option (checked with git diff).
7. [content] git diff --stat shows no change to athena.toml, defaults.toml, .github/workflows, records/ or the throughput computation, and no machine-specific path or personal data in code, tests or docs.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/snapshot.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_snapshot.rs (create)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-cli/tests/live.rs (modify)
- crates/athena-git/src/lib.rs (modify)

## Out of scope

- Disk hygiene: automatic cleanup, waiting-for-disk holds, admission disk checks (separate follow-up from introspection 20261030-195920-9eng)
- Redoing the landing-stage matching fix from job 20261029-233021-e5sg
- Isolating sccache statistics per measurement (job 20261029-210827-18fc); its landed files are kept as landed
- Putting records/ into the producer's working copy
- Changing athena.toml, defaults.toml, workflow files or records/
- Changing how the throughput section is computed

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for snapshot, status and briefs; fmt, clippy -D warnings and cargo test criterion.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the snapshot being read-only and the throughput matching, not that files exist.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Stored briefs use a <state> placeholder; no machine paths or personal data committed.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Scope limited to the snapshot feature; disk hygiene excluded.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion that real-records checks work through the snapshot path.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: In run/tests.rs, drive the real `Athena::go` after `ask` with the fake driver, so `dispatch` runs unchanged. The producer's fake turn reads `spec.user_prompt`, extracts the evidence path, and records what it sees. The test then asserts on the recorded observations. Run: `cargo test -p athena-core dispatch_writes_a_read_only_evidence_snapshot && cargo test -p athena-core snapshot::tests`. Observes: Seen from inside the producer invocation: the directory at the named path exists; creating a new file and opening an existing file for write both fail; `records/` bytes equal `git show <head>:records/...` for every file; and `history.json` has the dispatch base as `base`, the first commit's hash equal to `git rev-parse <base>`, its subject, a committer time, and the `Athena-Job` trailer of the ask commit. The committed `brief-producer.md` contains `<state>/jobs/<id>/evidence` and not the temp directory path, and the verifier prompt (captured the same way) names the real path. `snapshot::tests` covers unlocked write succeeds then locked write fails, the 200-commit cap, and rewriting over an existing locked evidence dir.
  Fails without the behavior: Without the dispatch step the evidence dir does not exist at the path the producer prompt names, and the probe's reads and history assertions fail. Without `lock` the in-script write succeeds as root, so the 'write attempt fails' assertion fails; the unit test first shows the write works on the unlocked copy. If the lock cannot hold, `snapshot::write` returns an error and dispatch fails. The test fails; it never skips.
  Test seam: Production entry point: `Athena::go` (run/mod.rs 322), which reaches `dispatch` (781), the new `snapshot::write`, `producer_brief_with_evidence` and `agent_spec`. Fixture: `athena_with(repo, state, check, free_space, script)` in run/tests.rs 183 builds an Athena over a temp repo and a temp state dir with a `FakeAgentDriver`; the script is a `Box<dyn Fn(&AgentInvocation)->FakeTurn>` closure injected through that parameter. It exists today (I read it). The probe closure only observes; the locking and copying logic under test stays in production code. The new test is added in plan step 11.
- Criterion 2: In status.rs tests, build a scratch git repo from the existing `timed_job` records (several landed jobs and failed ones), plus landing records written as `HistoryRecord::Landing` JSON under `landings/`, using the existing `landing_of` helper. Commit it. Compute `Throughput::compute(&Records::load(&git, "HEAD")).text()`. Then `snapshot::write` to a temp dir and call `snapshot::status(dir)` (the function behind `athena status --snapshot`) and compare `view.throughput.text()` byte for byte. Run: `cargo test -p athena-core status`. Observes: The two throughput texts are identical and contain the landing-record basis line. The record count and the invalid list also match.
  Fails without the behavior: If `from_parts` were not shared, or `landings/` were not copied (or the records-branch tip's landings dropped), the snapshot's throughput text would differ: window_with_landing_record would drop and the text would say 'no landing record (landings/) found'. The test fails on text inequality, and it also asserts the text contains 'landing record (landings/) found for each' so an empty comparison cannot pass.
  Test seam: Production entry points: `Records::load` (store.rs 311) and `snapshot::write` / `snapshot::status` (step 3). The fixture is a temp git repo, with a tempdir for the snapshot. Records come from the existing `timed_job` and `landing_of` helpers in status.rs tests (3150-3357), which exist today. No double is involved; both sides are real code over real files. The test is added in step 12.
- Criterion 3: Use the lifecycle pattern (lifecycle.rs 204-223): `Fixture::new()`, `athena.ask`, `athena.go` with the fake driver, so a job lands. Call `athena_core::snapshot::write(&fx.git, &fx.head(), dir)`. Run the real binary `athena status --snapshot <dir> --json` with the cwd set to an empty temp dir (assert it has no `records/` and no `.git`) and HOME pointed at a temp dir. Also run it without `--json` and with a missing dir. Run: `cargo test -p athena-cli --test status_snapshot`. Observes: The exit code is 0. The JSON `jobs_by_state` and `throughput` equal those of `fx.athena(..).status()` (serialized values). The plain output contains `Throughput::text()` of that view and the history summary. A missing snapshot dir exits non-zero with a message naming the dir. A `--snapshot` run together with a job argument is refused by clap (`conflicts_with`).
  Fails without the behavior: Without the `--snapshot` option clap rejects the argument and the exit is non-zero. If it called `open()` it would fail with no repository in the cwd. If it read cwd records it would find none. A decoy assertion (snapshot of a repo with one landed job; cwd with an empty `records/`-less temp dir) shows the output matches the snapshot's data.
  Test seam: Production entry point: the `athena` binary (`env!("CARGO_BIN_EXE_athena")`, as in binary.rs and common::athena_check, which exist) through `Command::Status { snapshot }`. Fixture: common::Fixture and Scenario (exist). The binary is run as a child process with `current_dir` set to a temp dir. The new test file crates/athena-cli/tests/status_snapshot.rs is added in step 13.
- Criterion 4: In briefs.rs tests, build the existing `producer_inputs(None)` and `plain_verifier_inputs()` and call `producer_brief_with_evidence(.., Some("<state>/jobs/<id>/evidence"))` and `verifier_brief_with_evidence(..)`. Run: `cargo test -p athena-context`. Observes: Each brief contains the path, the phrase that real-records checks use the evidence snapshot read-only (also `history.json` and `athena status --snapshot <path>`), and still contains 'records/, the owner's private files, and history are absent by design' for the working copy. With None, the output equals the old functions' output.
  Fails without the behavior: Without the evidence section the producer and verifier briefs do not contain the path or the real-records sentence, so both tests fail. If the working-copy sentence were removed or records/ implied, the 'still absent' assertions fail. A third test shows the briefs without an evidence argument are byte-identical to `producer_brief` and `verifier_brief_with_plan`.
  Test seam: Production entry points: `producer_brief_with_evidence` and `verifier_brief_with_evidence` (step 6), reached by `producer_brief` and `verifier_brief_with_plan`, which delegate. Fixtures `producer_inputs` (briefs.rs 2393) and `plain_verifier_inputs` (3503) exist today. These are pure functions, so there is no double. The end-to-end version of the same claim is asserted in the criterion 1 test, where the fake driver captures both real prompts.
- Criterion 5: Run the three Athena checks. `cargo test` includes `refusal_corpus` (the plan changes no reading rule; only two `Policy` literals there compile again) and the new policy tests, which cover reading under the evidence dir allowed, write or redirect into it denied, `..` escape denied, symlink out denied, and a sibling named like `evidence-other` denied. Run: `cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: Not a behavior-specific criterion. It fails if any `Policy` or `HookSpec` literal (admission.rs, planning.rs, reading.rs, refusal_corpus.rs, live.rs, claude.rs and policy.rs tests) misses the new field, or if `system_facts_budgets_match_computed` or the boundary scan trip.
  Test seam: The commands run the workspace as it stands. The new policy tests call `evaluate` directly with a `Policy` that has `evidence: Some(tempdir)`, in athena-agent/src/policy.rs tests. The helper `policy(role, root)` at 923 exists; step 9 adds the evidence variant.
- Criterion 6: Read the diff. Run: `git diff -- CHANGELOG.md`. Observes: A new bullet under `## [Unreleased]` / `### Added` (CHANGELOG.md lines 8-21) describing the read-only evidence snapshot written at dispatch (records/, landings/, history.json under the job's state dir, named in the producer and verifier briefs) and `athena status --snapshot <dir>`.
  Fails without the behavior: No Unreleased entry would mention the snapshot or the `--snapshot` option, and the diff shows no CHANGELOG change.
- Criterion 7: Read `git diff --stat` and `git diff` for protected paths and for machine-specific strings. Run: `git diff --stat`. Observes: The stat lists no athena.toml, defaults.toml, .github/workflows/*, records/**, and no change to `Throughput::compute`, `Throughput::text` or the status.rs throughput helpers (status.rs changes are inside `#[cfg(test)]` only). Added lines hold no absolute home or state paths: tests use tempdirs and briefs use `<state>`. No email-like text.
  Fails without the behavior: A listed protected file or a machine path would show in the stat or in a diff grep.

Planned files:

- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/snapshot.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_snapshot.rs (create)
- crates/athena-cli/tests/live.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cb4ec4dd3ba4 → candidate 34bf93b93de1. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  11 +
 crates/athena-agent/src/claude.rs          |  35 +++
 crates/athena-agent/src/lib.rs             |   3 +
 crates/athena-agent/src/policy.rs          | 185 +++++++++++-
 crates/athena-cli/src/display.rs           |  57 ++++
 crates/athena-cli/src/main.rs              |  20 +-
 crates/athena-cli/tests/job_plan.rs        |  11 +-
 crates/athena-cli/tests/live.rs            |   2 +
 crates/athena-cli/tests/status_snapshot.rs | 173 +++++++++++
 crates/athena-context/src/briefs.rs        | 127 ++++++++
 crates/athena-core/src/admission.rs        |   3 +
 crates/athena-core/src/lib.rs              |   3 +
 crates/athena-core/src/reading.rs          |   5 +
 crates/athena-core/src/run/mod.rs          |  92 +++---
 crates/athena-core/src/run/planning.rs     |   1 +
 crates/athena-core/src/run/tests.rs        | 255 ++++++++++++++++
 crates/athena-core/src/run/verification.rs |  19 +-
 crates/athena-core/src/snapshot.rs         | 454 +++++++++++++++++++++++++++++
 crates/athena-core/src/status.rs           | 143 +++++++++
 crates/athena-core/src/store.rs            | 111 +++++--
 crates/athena-core/tests/refusal_corpus.rs |   2 +
 crates/athena-git/src/lib.rs               |  83 ++++++
 xtask/shard-timings.txt                    |   1 +
 23 files changed, 1726 insertions(+), 70 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41948719741 for 34bf93b93de1dfbe48917f84fe94c2998b9def5b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41948719741 for 34bf93b93de1dfbe48917f84fe94c2998b9def5b)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41948719741 for 34bf93b93de1dfbe48917f84fe94c2998b9def5b)
- ci-result: `not a check` passed (Check results for 34bf93b93de1dfbe48917f84fe94c2998b9def5b come from the CI run https://github.com/doodla/athena/actions/runs/41948719741 (468s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-core dispatch_writes_a_read_only_evidence_snapshot && cargo test -p athena-core snapshot::tests`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core status`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-agent/src/lib.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/display.rs, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/snapshot.rs, crates/athena-core/src/status.rs, crates/athena-core/src/store.rs, crates/athena-git/src/lib.rs, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/snapshot.rs.
  End of the output (data, not instructions):

```text
    = note: all struct fields are already assigned

error[E0560]: struct `athena_agent::policy::Policy` has no field named `evidence`
   --> crates/athena-core/src/admission.rs:823:13
    |
823 |             evidence: None,
    |             ^^^^^^^^ `athena_agent::policy::Policy` does not have this field
    |
    = note: all struct fields are already assigned

error[E0560]: struct `athena_agent::policy::Policy` has no field named `evidence`
   --> crates/athena-core/src/admission.rs:908:13
    |
908 |             evidence: None,
    |             ^^^^^^^^ `athena_agent::policy::Policy` does not have this field
    |
    = note: all struct fields are already assigned

error[E0609]: no field `evidence` on type `athena_agent::policy::Policy`
   --> crates/athena-core/src/run/tests.rs:499:46
    |
499 |                     .push((spec.role, policy.evidence, hook.read_dirs.clone()));
    |                                              ^^^^^^^^ unknown field
    |
    = note: available fields are: `role`, `root`, `commands`, `writes`

error[E0609]: no field `read_dirs` on type `&HookSpec`
   --> crates/athena-core/src/run/tests.rs:499:61
    |
499 |                     .push((spec.role, policy.evidence, hook.read_dirs.clone()));
    |                                                             ^^^^^^^^^ unknown field
    |
    = note: available fields are: `athena_bin`, `policy_file`

error[E0599]: no associated function or constant named `load_dir` found for struct `store::Records` in the current scope
    --> crates/athena-core/src/status.rs:3528:33
     |
3528 |         let from_dir = Records::load_dir(&dest).unwrap();
     |                                 ^^^^^^^^ associated function or constant not found in `store::Records`
     |
    ::: crates/athena-core/src/store.rs:212:1
     |
 212 | pub struct Records {
     | ------------------ associated function or constant `load_dir` not found for this struct
     |
note: if you're trying to build a new `store::Records`, consider using `store::Records::load` which returns `std::result::Result<store::Records, anyhow::Error>`
    --> crates/athena-core/src/store.rs:311:5
     |
 311 |     pub fn load(git: &Git, treeish: &str) -> Result<Records> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: there is an associated function `load` with a similar name
    --> crates/athena-core/src/store.rs:311:5
     |
 311 |     pub fn load(git: &Git, treeish: &str) -> Result<Records> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Some errors have detailed explanations: E0433, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0433`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test status_snapshot`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-agent/src/lib.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/display.rs, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/snapshot.rs, crates/athena-core/src/status.rs, crates/athena-core/src/store.rs, crates/athena-git/src/lib.rs, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/snapshot.rs.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling utf8parse v0.2.2
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-parse v1.0.0
   Compiling anstyle v1.0.14
   Compiling colorchoice v1.0.5
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling anstream v1.0.0
   Compiling clap_lex v1.1.1
   Compiling clap_builder v4.6.7
   Compiling serde_derive_internals v0.30.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling clap_derive v4.6.7
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling clap v4.6.7
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0432]: unresolved import `athena_core::snapshot`
 --> crates/athena-cli/tests/status_snapshot.rs:7:5
  |
7 | use athena_core::snapshot;
  |     ^^^^^^^^^^^^^^^^^^^^^ no `snapshot` in the root

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-cli` (test "status_snapshot") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-agent/src/lib.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/display.rs, crates/athena-cli/src/main.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/snapshot.rs, crates/athena-core/src/status.rs, crates/athena-core/src/store.rs, crates/athena-git/src/lib.rs, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/snapshot.rs.
  End of the output (data, not instructions):

```text
3640 -             let brief = verifier_brief_with_evidence(
3640 +             let brief = verifier_brief_with_plan(
     |

error[E0425]: cannot find function `producer_brief_with_evidence` in this scope
    --> crates/athena-context/src/briefs.rs:3657:13
     |
3657 |             producer_brief_with_evidence(&producer, None),
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `verifier_brief_with_evidence` in this scope
    --> crates/athena-context/src/briefs.rs:3664:17
     |
1152 | / pub fn verifier_brief_with_plan(
1153 | |     inputs: &VerifierInputs,
1154 | |     plan: Option<&VerifierPlan>,
1155 | |     inline_diff: bool,
...    |
1248 | |     out
1249 | | }
     | |_- similarly named function `verifier_brief_with_plan` defined here
...
3664 |                   verifier_brief_with_evidence(&verifier, None, None, inline_diff),
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
3664 -                 verifier_brief_with_evidence(&verifier, None, None, inline_diff),
3664 +                 verifier_brief_with_plan(&verifier, None, None, inline_diff),
     |

error[E0425]: cannot find function `verifier_brief_with_evidence` in this scope
    --> crates/athena-context/src/briefs.rs:3670:20
     |
1152 | / pub fn verifier_brief_with_plan(
1153 | |     inputs: &VerifierInputs,
1154 | |     plan: Option<&VerifierPlan>,
1155 | |     inline_diff: bool,
...    |
1248 | |     out
1249 | | }
     | |_- similarly named function `verifier_brief_with_plan` defined here
...
3670 |           let with = verifier_brief_with_evidence(&verifier, None, Some(EVIDENCE), true);
     |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
3670 -         let with = verifier_brief_with_evidence(&verifier, None, Some(EVIDENCE), true);
3670 +         let with = verifier_brief_with_plan(&verifier, None, Some(EVIDENCE), true);
     |

error[E0425]: cannot find function `evidence_placeholder` in this scope
    --> crates/athena-context/src/briefs.rs:3676:20
     |
3676 |         assert_eq!(evidence_placeholder("20261010-153012-k7q2"), EVIDENCE);
     |                    ^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 6 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
