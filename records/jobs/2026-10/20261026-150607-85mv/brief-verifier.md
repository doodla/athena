# Job 20261026-150607-85mv: verify "Move all records/ readers to the athena/records worktree, with fallback"

## Request (verbatim)

> try again on 20261026-145955-s7e2

## Reading (accepted by the user)

Define the athena/records worktree path once in athena-data paths.rs beside repo_state_dir. Add load_current forms to Records::load and RecordSet::load, and move every live reader to them, including gh.rs land_pr and the introspect.rs directory walks. Readers fall back to in-tree records/ only when the worktree is absent. The site generator reads studies/ and stays unchanged. Audience: Athena operators and the verifier.

Assumptions:
- Goal: after the D111 cutover, main's HEAD has no records/, so every live reader must read the worktree of athena/records in Athena's state directory.
- Decided up front from the earlier critic findings: gh.rs land_pr (gh.rs:2513, qualified crate::store::Records::load) moves to load_current. The introspect.rs walks (~300/318/339/360) move to the records root. Neither goes on the allow-list.
- A first grep of the snapshot (Records::load, RecordSet::load, join("records"), read_records) found hits in about 44 files, many of them tests. The producer must enumerate non-test hits from a real scan and classify each as move, allow-listed with a reason, or out of scope. Every moved file is a deliverable.
- Allow-list candidates are limited to citation strings (steward.rs, work_decisions.rs), the gate invariant BOOTSTRAP_OUTCOME path, treeish judges that take a given ref (gate history, attribution, reconcile, content), the fallback itself, and paths.rs.
- The scan matches the Records::load( / RecordSet::load( suffix whatever the path prefix is, so crate::store::Records::load( is caught.
- The site generator xtask/src/site.rs reads studies/ and is unchanged. It is shown to build identically with records/ present and absent.
- The producer may touch other files the inventory finds, such as the gate reconcile, attribution, history and content modules. Only the ones that turn out to be live readers change.

## Acceptance criteria

1. [behavior] A test puts different records in the worktree and in the in-tree records/ and checks that each moved reader returns the worktree's records: Records::load_current, RecordSet::load_current, read_records, the xtask walltime reader, introspect walks, status, forecasting, scoring, retry carry, auditor, audit, and the other moved readers in the inventory. Run cargo test --locked.
2. [behavior] A real-git fixture whose HEAD has no records/ loads records through the worktree. A negative control shows the same assertion fails with the old HEAD-based load. Run cargo test --locked.
3. [behavior] A test calls land_pr on a checkout whose HEAD has no records/ and shows it resolves the job's verification and candidate from the worktree. Run cargo test --locked.
4. [behavior] With no worktree present, each moved reader falls back to records/ and returns the same result as before. athena status and a forecast succeed on a scratch checkout whose working tree has no records/, with output matching the records/ run. Run cargo test --locked.
5. [content] The worktree path is defined once, in crates/athena-data/src/paths.rs beside repo_state_dir. No non-test reader joins records/ itself outside paths.rs and the fallback.
6. [behavior] An xtask scan test named records_reader_scan fails on planted reads (join("records"), read_records(<repo dir>), Records::load(git, head), and crate::store::Records::load(git, head)). It passes on the real tree with an allow-list in which every entry has a reason. The scan was first run against the real tree to produce the call-site inventory. Run cargo test -p xtask --locked records_reader_scan and confirm a nonzero number of tests ran.
7. [behavior] xtask/src/site.rs is untouched, and the site output is identical with records/ present and absent. Run cargo run -p xtask -- site both ways and compare the outputs.
8. [behavior] cargo fmt --check, cargo clippy --all-targets and the full cargo test --locked pass, with existing tests unchanged.
9. [content] docs/decisions.md and CHANGELOG.md record the reader move, the fallback, the allow-list, and that the site generator is out of scope, citing D111.

## Expected deliverables

- crates/athena-data/src/paths.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-core/src/audience.rs (modify)
- crates/athena-core/src/review.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/records_import.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/audit.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/records_branch_readers.rs (create)
- xtask/src/main.rs (modify)
- xtask/src/walltime.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- xtask/src/site.rs and the study site generator, which reads studies/ and stays unchanged
- Writers of records/ and the cutover itself (job 20261025-001232-vesh)
- Citation strings in steward.rs and work_decisions.rs
- Changes to athena.toml, defaults.toml or CI workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture, fallback, land_pr and scan tests are added, and fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check readers returning worktree records, not that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that readers still work when HEAD has no records/.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared load_current path plus a scan covers all call sites, not only the ones found so far.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria use commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Allow-list reasons and decision docs are written for an outside reader.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: The scan test run must confirm a nonzero number of tests ran.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused move; the site generator is excluded.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration tests in crates/athena-cli/tests/records_branch_readers.rs, one per moved reader or family. Each puts record A in in-tree records/ and record B (different job ids) in the worktree, and asserts the reader returns B. Run: `cargo test --locked`. Observes: Each reader test passes with B returned and A absent.
  Fails without the behavior: A reader still calling the HEAD-based load or joining the repo root returns A, so the assertion for B fails.
- Criterion 2: Real-git fixture whose HEAD has no records/ and a worktree with records. Assert load_current returns the worktree's jobs; a negative-control test asserts old Records::load(git, "HEAD") returns none. Run: `cargo test --locked`. Observes: load_current finds the job; the old load finds nothing.
  Fails without the behavior: Without the worktree branch, load_current equals load and returns no records.
- Criterion 3: A land_pr test on a checkout whose HEAD has no records/, with verification (Pass) and candidate only in the worktree, using the existing runner doubles. Run: `cargo test --locked`. Observes: land_pr passes the no-verification and no-candidate checks and acts on the worktree's candidate.
  Fails without the behavior: With the old Records::load at gh.rs:2513, land_pr fails with an unknown job or 'has no verification'.
- Criterion 4: Fallback tests with no worktree: each moved reader equals the old result. Run the built binary for status and a forecast on a scratch checkout whose working tree has no records/ (records at HEAD), and compare with a run with records/ present. Run: `cargo test --locked`. Observes: Results are equal; status and forecast exit 0 with matching output.
  Fails without the behavior: If load_current required the worktree or fell back to empty, the equality and exit-code assertions fail.
- Criterion 5: Read paths.rs for the three helpers beside repo_state_dir; the scan finds no other non-test join of records/ outside paths.rs and the allow-listed fallback. Run: `cargo test -p xtask --locked records_reader_scan`. Observes: The scan passes on the real tree with a nonzero test count.
  Fails without the behavior: A second non-test join("records") would be reported by the scan and fail the test.
- Criterion 6: Tests run the scan on the real tree (every allow-list entry has a non-empty reason) and on planted text: join("records"), read_records(<repo dir>), Records::load(git, head), crate::store::Records::load(git, head). The producer first runs the scan on the unmodified tree and records the inventory. Run: `cargo test -p xtask --locked records_reader_scan`. Observes: Real tree has no findings; every planted read is flagged; N passed is greater than 0.
  Fails without the behavior: A scan missing a planted pattern, such as the qualified crate::store::Records::load, fails its planted test.
- Criterion 7: git diff shows site.rs untouched. Run the site generator with records/ present and again with it removed (or a test in xtask/src/main.rs doing so on a temp root) and compare outputs. Run: `cargo run -p xtask -- site`. Observes: Outputs are byte-identical.
  Fails without the behavior: If the generator read records/, the two outputs would differ.
- Criterion 8: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked; git diff shows no existing test edited or removed (new tests only in new functions and the new file). The full existing suite exercising unchanged readers through the new seams is the regression guard. Run: `cargo test --locked`. Observes: All three pass with the existing tests unchanged.
  Fails without the behavior: If a seam changed old behavior (for example the fallback returned a different result), an existing unchanged test would fail; a formatting or lint slip fails fmt or clippy.
- Criterion 9: Add a docs/decisions.md entry and a CHANGELOG.md Unreleased entry, each citing D111 and naming the reader move, the fallback, the allow-list and the site exclusion. The xtask policy tests scan these files. Run: `cargo test --locked`. Observes: Both files carry the four points and the policy tests pass.

Planned files:

- crates/athena-data/src/paths.rs (modify)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/auditor.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-core/src/audience.rs (modify)
- crates/athena-core/src/review.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/watcher.rs (modify)
- crates/athena-core/src/records_import.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-gate/src/attribution.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/decisions.rs (modify)
- crates/athena-cli/src/backtest.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/records_branch_readers.rs (create)
- xtask/src/main.rs (modify)
- xtask/src/walltime.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base 73841ea1bf19 → candidate 98f5a9d87ac0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                      |   7 +
 crates/athena-cli/src/backtest.rs                 |   2 +-
 crates/athena-cli/src/drain.rs                    |   2 +-
 crates/athena-cli/tests/records_branch_readers.rs | 352 ++++++++++++++++++++++
 crates/athena-core/src/audience.rs                |   2 +-
 crates/athena-core/src/auditor.rs                 |   2 +-
 crates/athena-core/src/code_health.rs             |   4 +-
 crates/athena-core/src/gh.rs                      |   2 +-
 crates/athena-core/src/hestia.rs                  |   4 +-
 crates/athena-core/src/introspect.rs              |  19 +-
 crates/athena-core/src/manage.rs                  |  12 +-
 crates/athena-core/src/proposals.rs               |  15 +-
 crates/athena-core/src/reading.rs                 |  14 +-
 crates/athena-core/src/review.rs                  |   2 +-
 crates/athena-core/src/run.rs                     |   6 +-
 crates/athena-core/src/scope_guard.rs             |   2 +-
 crates/athena-core/src/status.rs                  |   8 +-
 crates/athena-core/src/steward.rs                 |   4 +-
 crates/athena-core/src/store.rs                   |  11 +-
 crates/athena-core/src/watcher.rs                 |   2 +-
 crates/athena-core/src/work_decisions.rs          |   8 +-
 crates/athena-data/src/lib.rs                     |   3 +-
 crates/athena-data/src/paths.rs                   |  41 +++
 crates/athena-gate/src/records.rs                 |  58 ++++
 docs/decisions.md                                 |  38 +++
 xtask/src/main.rs                                 | 166 ++++++++++
 xtask/src/walltime.rs                             |  43 ++-
 27 files changed, 773 insertions(+), 56 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1347 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets and the full cargo test --locked pass, with existing tests unchanged.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
