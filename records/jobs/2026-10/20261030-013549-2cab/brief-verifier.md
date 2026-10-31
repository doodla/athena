# Job 20261030-013549-2cab: verify "Create and remove a per-job worktree and target dir with a disk bound"

## Request (verbatim)

> Priority: high
> Async drain build job A4 (docs/design/async-drain.md, decision D145). Per-job worktree start (job 4): create and remove a worktree and its target directory per job, with the disk bound of section 5 computed from job A1's measured T and S. Depends on 20261030-013143-as0j and 20261029-142844-ynv3 and 20261029-210827-18fc (A1 needs its measurement). With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/disk.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A4 (D145, section 5): each job gets its own git worktree and CARGO_TARGET_DIR, created at producer start and removed when the job ends. The number of jobs allowed in flight is capped by a disk bound computed from the latest measured T and S (job A1's measurement record from 20261029-210827-18fc), with max_in_flight = 1 unchanged. The audience is Sri and the drain verifier.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs (guard, sweep_idle_targets, CargoBuilder, measure_build, BuildMeasurement use), crates/athena-core/src/run/mod.rs (Athena::go -> go_with_stages, the real entry point; kept_work_dir), crates/athena-core/src/run/producer.rs (producer_snapshot uses StateDir::work_dir, jobs/<id>/work), crates/athena-data/src/config.rs (PipelineConfig max_in_flight, reserve_disk_gb = 10). The disk bound formula is design section 5: in_flight = floor((free_gb - reserve_disk_gb - max_check_slots x T) / (T + S)). T and S come from the latest athena.build-measurement/1 record; with none, the design estimates T about 6 GB, S about 0.2 GB are used. No new record type or data file is added, so the records allow-list (check_record_file, AnyRecord round-trip) is not touched.
- Job 20261030-013143-as0j (A3, in-flight table) landed and is kept as is. The worktree it records is StateDir::work_dir; this job makes that directory a real git worktree and removes it. Its minor finding (a moved row keeps the stage verifier) and its nit (section number) are not in this job's files and stay out of scope.
- Job 20261029-142844-ynv3 (A1, rejected, no numbers measured) is dropped: its docs-only candidate changed CHANGELOG.md, docs/decisions.md and docs/design/async-drain.md and none of it is redone here. Its need for numbers was met by 20261029-210827-18fc (landed, kept as is), whose `athena measure-build` writes the T and S record this job reads. Its minor findings (extra store.rs, display.rs and test files; sccache stats not isolated) are not touched. If no record exists the bound uses the labelled design estimates.
- Worktree creation and removal run through git worktree add/remove on the state directory path jobs/<id>/work with a per-job CARGO_TARGET_DIR; removal also runs on failure and when a job ends, so a finished job leaves neither directory.
- With max_in_flight = 1 the drain runs as today, existing drain tests unchanged. Only a bound that would allow zero in-flight jobs on a small disk is applied, and it holds the start rather than failing the job.

## Acceptance criteria

1. [behavior] A test through Athena::go (the real entry point) shows a job's producer runs in a fresh git worktree with its own target directory under the job's state directory, and that after the job ends both directories are gone, on success and on a failed or rejected job. Run: cargo test -p athena-core -p athena-cli.
2. [behavior] A test in crates/athena-core/src/disk.rs shows the in-flight limit equals floor((free_gb - reserve_disk_gb - check_slots x T) / (T + S)) using T and S from the latest build-measurement record, and that with no record the design estimates (6 GB, 0.2 GB) are used. Run: cargo test -p athena-core disk.
3. [behavior] A test shows that when the bound allows fewer in-flight jobs than max_in_flight, the extra start waits rather than failing, and that with max_in_flight = 1 behavior is unchanged and the existing tests in crates/athena-cli/tests/drain.rs and lifecycle.rs pass without edits to their assertions. Run: cargo test.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
5. [content] CHANGELOG.md has an entry for per-job worktree start naming the disk bound and the source of T and S.
6. [content] The diff changes no file under athena.toml, defaults.toml, .github/workflows or records/, and does not change crates/athena-data/src/config.rs.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/design/async-drain.md (modify)

## Out of scope

- Running more than one producer at once or raising max_in_flight above 1 (build job 10)
- Rebase at landing time, check slots, read-ahead, conflict return
- Acting on leftover in-flight rows (recovery)
- Fixing A3's moved-row stage finding or A1/measure-build findings
- Changes to athena.toml, defaults.toml, .github/workflows, crates/athena-data/src/config.rs
- Re-running or changing the build measurement itself

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for worktree lifecycle and the bound; fmt, clippy, test criterion.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run through cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check dirs created and removed, and the computed limit.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to run and disk modules.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration tests in crates/athena-cli/tests/drain.rs, named per_job_worktree_*, call the production entry point `Athena::go(job, AcceptedBy::User, &lines)` on a proposed job. The Athena comes from the existing `Fixture::athena(script)`, with `athena.state.with_free_space(simulated_free)` as go_with_disk does. Both exist today. The script wraps `Scenario::default().script()`. For Role::Producer, the wrapper records `spec.workdir`, `spec.env["CARGO_TARGET_DIR"]`, whether `<workdir>/.git` exists, and whether the target dir exists and is empty at that moment; it then delegates to the inner script. The wrapper observes only. The wrapper is the only new seam, added in this step. Preconditions: the config is test_config() (min_free_gb = 0, max_in_flight default 1). Five runs, each a fresh Fixture: (a) success: Landed; (b) rejected: a failing check from config_with_change_checks, so a check fails, `target/produce` is kept during the run, and the state is Rejected; (c) failed: the producer makes no change, so the state is Failed; (d) stale start: before `go`, the test plants `jobs/<id>/work/stale` and `jobs/<id>/target/produce/stale`, and the run still succeeds and the producer sees neither stale file; (e) error escape: the wrapper, for the producer only, deletes `<workdir>/.git` before returning, so a later `snapshot.change_list()?` in candidate_stage returns Err out of produce_and_verify. The test asserts `go` returns Err (not a closed job) and that `work_dir(id)` and `produce_dir(id)` are both gone. Without the guard that exit leaves both behind, because clean_job never runs. Unit tests in crates/athena-core/src/state.rs: dropping a JobWorktree removes both directories and a stale dir is cleared on creation; job_lock_held is true for a job whose lock another handle holds, false when no job.lock exists (and it creates none), and false once released. Run: `cargo test -p athena-cli --test drain per_job_worktree && cargo test -p athena-core state`. Observes: During the producer, in (a)-(e): workdir == <state>/jobs/<id>/work, `<workdir>/.git` exists, CARGO_TARGET_DIR == <state>/jobs/<id>/target/produce, a different path under the same job directory, empty at start, and no stale file is visible. After `go` returns, for success, rejected and failed, neither `work` nor `target/produce` exists. In (e) `go` returned Err and neither exists. The state.rs drop test also shows both are removed on an early return.
  Fails without the behavior: Without the guard and the fresh-start clearing, the stale `work` and `target/produce` planted before the run are still present when the producer starts (Snapshot::create refuses an existing directory, so the run errors). The `?`-error case leaves both directories behind after `go` returns Err, so its final assertion fails. The state.rs drop test also fails. If no isolated work dir existed, the in-run assertion that the producer's workdir is its own repository root under jobs/<id> fails.
- Criterion 2: Unit tests in the tests module of crates/athena-core/src/disk.rs call the pure public functions `Figures::from_latest(Option<&BuildMeasurement>)` and `in_flight_limit(free_bytes, reserve_gb, check_slots, &Figures)` directly. Nothing is faked. A further test builds `BuildMeasurement` values the way status.rs's build_measurement_tests does and calls `BuildMeasurement::latest` on a list of two. The seam is the functions themselves; step 3 adds them. Another test calls `Athena::start_hold` and checks that it reads the pipeline settings and the latest record. It uses a repository whose records contain a measurement and `StateDir::with_free_space` with a fixed fn, as the existing disk.rs tests do. Run: `cargo test -p athena-core disk`. Observes: Free 60 GB, reserve 10, 1 slot, T 6 GB, S 0.2 GB gives 7 (the design example). A record with T 4 GB and S 1 GB at free 60 GB gives floor((60-10-4)/5) = 9. With two records the figures are the later one's. With no record the figures are exactly 6_000_000_000 and 200_000_000 bytes and the source says 'design estimate'. Free below reserve + slots x T gives 0 (saturating). 2 slots lowers the 7 case to floor((60-10-12)/6.2) = 6. T + S = 0 does not divide by zero.
  Fails without the behavior: A different formula, an ignored record, other fallback constants, or the first record in place of the latest would each change the exact integers asserted below. A limit that is not floor-saturating would fail the below-reserve case.
- Criterion 3: New tests in crates/athena-cli/tests/drain.rs (names starting `in_flight_bound_`) call `Athena::go` on a Fixture whose config is test_config() plus `\n[drain.pipeline]\nmax_in_flight = 2\n` (the TOML shape is DrainConfig/PipelineConfig). Defaults are reserve_disk_gb = 10 and max_check_slots = 1. Free space is set per test by `set_free` through the existing `simulated_free`. The other in-flight job is made real: the test creates `jobs/<other>/work/x` and holds `athena.state.try_lock_job(&other)` for the whole call. Both seams exist today. Cases: (a) 25 GB free, so the limit is floor((25-10-6)/6.2) = 1 with one running: `go` returns Err that downcasts to `LowDisk`, the job is still Proposed with no Job record, and the hold marker exists. (b) The lock is released: the same job now runs to Landed. (c) 40 GB: limit 3, min(2,3) = 2 > 1 running, so the job starts and lands. (d) 12 GB (limit 0) with nothing running: not held, lands. (e) the default config (max_in_flight = 1), 25 GB and a running other job: not held, lands as today. (f) Drain level: modelled on the_real_steps_hold_a_job_on_low_disk..., with max_in_flight = 2 and an in-flight job held, `drain::drain` reports outcome 'environment: low disk' with failed_item None and the job still queued. After release, the second drain lands the same job. The existing assertions of drain.rs and lifecycle.rs are not edited and run in the same `cargo test`. Run: `cargo test --locked`. Observes: The extra start returns a hold, not a failure, and the same job starts later. A lone job and max_in_flight = 1 are not held. The existing drain.rs and lifecycle.rs tests pass with no assertion changed (`git diff` on those files shows only added tests).
  Fails without the behavior: Without start_hold in dispatch, case (a) returns a finished job instead of Err(LowDisk). A hold that fails the job would leave it Closed, which the test rejects. Without the lone-job exemption, case (d) is held. Without the max_in_flight = 1 skip, case (e) is held.
- Criterion 4: Run the three checks Athena runs, in the order given. The producer runs `cargo fmt --all` first. All run through the real cargo tooling over the workspace. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0 with no new warning, and none of the new tests is #[ignore]d.
  Fails without the behavior: A formatting difference, any clippy warning (for example an unused import or a needless clone in the new code) or any failing test makes the command exit non-zero.
- Criterion 5: Read the CHANGELOG.md diff. Run: `git diff -- CHANGELOG.md`. Observes: One new entry under `## [Unreleased]` for per-job worktree start. It names the bound floor((free - reserve_disk_gb - max_check_slots x T) / (T + S)). It says T and S are read from the latest `athena.build-measurement/1` record (`athena measure-build`), with 6 GB and 0.2 GB used when none exists. It says that max_in_flight = 1 is unchanged and that a lone job is never held. It says the per-job worktree is the producer's isolated snapshot repository, not a linked `git worktree`, and why (it keeps history and records/ out of the producer's reach).
  Fails without the behavior: Not applicable (content criterion).
- Criterion 6: List the changed paths. Run: `git diff --stat`. Observes: Only crates/athena-core/src/{run/mod.rs,run/producer.rs,disk.rs,state.rs}, crates/athena-cli/tests/drain.rs, docs/design/async-drain.md and CHANGELOG.md. There is nothing under athena.toml, defaults.toml, .github/workflows or records/, not crates/athena-data/src/config.rs, and not run/tests.rs.
  Fails without the behavior: Not applicable (content criterion).

Planned files:

- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e8addf827f73 → candidate dac20593197f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  14 ++
 crates/athena-cli/tests/drain.rs       | 338 +++++++++++++++++++++++++++++++++
 crates/athena-core/src/disk.rs         | 286 ++++++++++++++++++++++++++++
 crates/athena-core/src/run/mod.rs      |  14 +-
 crates/athena-core/src/run/producer.rs |   3 +-
 crates/athena-core/src/state.rs        | 188 ++++++++++++++++++
 docs/design/async-drain.md             |  32 +++-
 7 files changed, 871 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1780 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
