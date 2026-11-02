# Job 20261031-031939-wz33: verify "Commit records from the landing queue after each landing"

## Request (verbatim)

> Priority: urgent
> Async drain build job A8 (docs/design/async-drain.md, decision D145). Records commit ordering (job 8): commit records from the landing queue after each landing, keeping the append-only rule, with several jobs in flight. With max_in_flight = 1 drain behaves as today.
> Depends on 20261030-013555-f5qm
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/store.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A8 (D145, job 8): with several jobs in flight, the drain commits each landed job's records from the landing queue right after its landing, in landing order, keeping the append-only record rule. The prerequisite job 20261030-013555-f5qm (A6, landing queue) never produced a candidate; this job's scope is only records ordering, and the A6 findings are addressed as stated in the assumptions. With max_in_flight = 1 the drain behaves as today.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (Landing trait at ~513, drain_concurrent at ~2482, the Athena-backed land at ~2790, doc header on max_in_flight 1 at ~46) and crates/athena-core/src/store.rs (Records::load, JobRecords, has_record_id) plus crates/athena-cli/tests/drain.rs (commit_records_to helper at ~4233). Entry point is drain_concurrent; the tests drive it, not helpers. No new record type or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched.
- Prior job 20261030-013555-f5qm (A6, landing queue) ended failed with no candidate and no changed files, so there is no work to keep or drop. Its findings are addressed as follows: the max_concurrency > 1 finding is addressed by a test through drain_concurrent with max_in_flight 1 pinning today's behaviour; the old-job state finding, the Rebase to Rebased mapping and the thrash finding concern the queue's rebase and conflict return (A6), so they do not apply to this job, and this job does not widen into them.
- A6's work is assumed to land first, as the dependency says; if the queue is absent, the producer wires the records commit to the existing landing step and says so.
- The audience is the maintainer running the drain and the verifier running cargo test.
- Records are committed through the existing commit path used by the drain; records/ files themselves are never edited by hand, and existing records are never rewritten, only appended.
- Changing store.rs is only to add what the ordering needs, such as a helper to commit a given job's records; it is not a change to record formats.

## Acceptance criteria

1. [behavior] A test through drain_concurrent with several jobs in flight shows each landed job's records are committed right after its landing, in landing order, never before the landing; run cargo test -p athena-cli --test drain and the new test passes.
2. [behavior] A test shows the append-only rule holds with interleaved jobs: after the drain, earlier record files are unchanged byte for byte (git diff shows only added record files) and no record is committed for a job that did not land.
3. [behavior] With max_in_flight = 1 the existing drain tests in crates/athena-cli/tests/drain.rs and crates/athena-cli/src/drain.rs pass unchanged (no edits to existing test bodies, shown by git diff); a test through drain_concurrent with [parallel] max_concurrency 2 and max_in_flight 1 pins today's records commit behaviour.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass across the workspace.
5. [content] CHANGELOG.md has an entry describing that records are now committed from the landing queue after each landing, with no behaviour change at max_in_flight = 1.
6. [content] Outcome check: a maintainer reading the new tests can see the records commit sequence versus the landing sequence for at least three jobs, not only that functions exist.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/design/async-drain.md (modify)

## Out of scope

- Raising max_in_flight above 1 or running several producers at once (build job 10)
- The landing queue's rebase, conflict return and thrash stop (A6, job 20261030-013555-f5qm)
- Disk guard, check slots, read-ahead and recovery of leftover in-flight rows
- Changes to athena.toml, defaults.toml, .github/workflows, crates/athena-data/src/config.rs
- Changes to record formats, records/ and crates/athena-core/src/gh/landing.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are run via cargo test through the real entry point.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check commit order and append-only results, not wiring.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Last criterion checks the ordering outcome.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only records ordering; queue work stays out of scope.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The append-only rule is enforced by a test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and doc comments are written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A new test in the appended `records_order` module drives `drain::drain_concurrent` with bound 3 over independent jobs a, b, c. Barrier-style start (as in `parallel::Shared`) puts all three runs in flight. Run times are staggered by at least 50 ms so runs pass in the order c, a, b. The stub appends `run X`, `land X` and `records X` to one log. Its `commit_records` makes a real gated commit with a Priority record via `Committer::commit_records`. The hook of `c` waits for a flag set at the end of `a`'s run, then 100 ms, so `a` is queued behind `c`. Asserts: the landing and records log is exactly `land c, records c, land a, records a, land b, records b`; `git log` subjects in the fixture repo run c, a, b; no `records X` precedes `land X`. Run: `cargo test -p athena-cli --test drain records_order::records_commit_after_each_landing -- --nocapture`. Observes: Each landed job's records commit directly follows its own landing and precedes the next landing. The records commit sequence equals the landing sequence (c, a, b), not the start order (a, b, c).
  Fails without the behavior: Without the `commit_records` call in `Drain::land`, no `records X` entry is logged and the exact-sequence assertion fails. A call made after the slot is released would let `land a` appear before `records c`. A call that runs for the wrong job fails the sequence. The test does not claim to detect a call moved just after `Drain::land` returns but before `attempts` returns, since that still keeps the order.
  Test seam: Production entry point: `drain::drain_concurrent(&steps, limits(9.0), &Quiet, 3)` (exists, drain.rs ~2482). Guards: independent jobs with disjoint paths, a budget large enough, the stub answers `Landing::Landed`, and `LandingQueue` and `await_head` are the real ones inside the drain. Fixture: a `Steps + Sync` recording stub in `records_order` (added by step 5) holding a `Git` and `StateDir` for a `Fixture` repo (tests/common/mod.rs, exists). It is injected as the `&dyn Steps` argument. `Steps::commit_records` does not exist today; step 2 adds it.
- Criterion 2: A new test in `records_order`. The `Fixture` repo first gets an earlier Priority record for an old job through the existing `commit_records_to` helper (tests/drain.rs:4233). The test notes HEAD, the file's blob id and its bytes. The three-job drain of criterion 1 then runs with bound 3, plus a fourth job `d` listed in the stub's `failing` list, whose run is rejected, so it never reaches `Drain::land`. The test runs `git diff --name-status <base>..HEAD` through `fx.git` and asserts exactly three lines, all starting with `A`. It asserts the earlier file's blob id and bytes are unchanged and that there are exactly three commits since base. It loads `Records::load(&fx.git, "HEAD")` and asserts that a, b and c each have their priority record and `d` has none. Run: `cargo test -p athena-cli --test drain records_order::append_only_with_interleaved_jobs -- --nocapture`. Observes: The diff holds only added record files, the earlier file is byte-identical, exactly the three landed jobs gained records, and the rejected job gained none. All of this goes through the real committer and canonical paths.
  Fails without the behavior: If the hook ran for every finished job, `d` would gain a record, and the commit count and the `Records::load` assertion would fail. If a records commit rewrote an earlier file, the diff would show `M` and the assertion would fail.
  Test seam: Production entry point: `drain::drain_concurrent` (exists). Guard: `d` ends `Rejected`, so `Drain::land` is not reached for it. Fixture: the same recording stub (step 5) and `Fixture` repo; `commit_records_to` (tests/drain.rs:4233, exists) makes the earlier record; `Records::load` (store.rs, exists) reads the result. Injected as `&dyn Steps`. The gate's own refusal of rewritten records is a production check that the real committer also runs.
- Criterion 3: Run all of tests/drain.rs and the drain.rs unit tests. Read the diff and show that existing test bodies have no removed or changed lines, only added lines. New tests in `records_order`: (a) `records_follow_landing_with_one_job_at_a_time` runs the recording stub through `drain_concurrent(.., 1)` and through `drain::drain` over a, b, c. It asserts the log is `run a, land a, records a, run b, land b, records b, run c, land c, records c` and that peak concurrency is 1. (b) `a_default_steps_sees_no_records_call_at_bound_two` runs `drain_concurrent(.., 2)` over a dependent pair with a default-hook stub copied into the module, with no `commit_records` override, and asserts the log is the old sequence with no records entry and that no commit was made in its repository. Run: `cargo test -p athena-cli --test drain && cargo test -p athena-cli --lib drain && git diff -- crates/athena-cli/tests/drain.rs crates/athena-cli/src/drain.rs`. Observes: Existing tests pass untouched. At bound 1 each landing's records commit follows it, one job at a time. A stub without the override sees nothing new, which is today's behaviour. [parallel] max_concurrency 2 with a stub that does not override the hook pins today's behaviour.
  Fails without the behavior: If the default hook changed behaviour for a stub that does not override it, an existing test in tests/drain.rs or the drain.rs unit tests would fail. If a landing and its records commit were reordered at bound 1, the pin test would fail.
  Test seam: Production entry points: `drain::drain` and `drain::drain_concurrent(.., 1)` and `(.., 2)` (exist). The bound is the fourth argument, `[parallel] max_concurrency` today; `max_in_flight` is not a drain parameter (checked in the code read). Fixture: the recording stub (step 5) and a default-hook stub in the module, injected as `&dyn Steps`. The `Steps::commit_records` default is added by step 2.
- Criterion 4: Run the three checks Athena runs on the result, across the workspace. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A formatting slip, a clippy lint in the new code (unused import, needless clone) or any failing test makes one command exit non-zero.
  Test seam: Whole-workspace cargo commands (exist). No fixture.
- Criterion 5: Read the diff. The entry sits under `### Added` of `[Unreleased]` (heading at CHANGELOG.md:197; the section runs from line 8 to 2897). It says what is built and what is not: the drain commits a landed job's records from the landing slot right after its landing, in landing order, only for landed jobs and append-only; the real steps stay on the default and still commit a job's records at close; moving the close-time commit into the queue is later work; nothing changes at max_in_flight = 1 (a bound of 1 behaves as today). It also does not say production records follow landing order. Run: `git diff -- CHANGELOG.md`. Observes: Added lines only, under Unreleased, with the narrower, accurate wording and the no-change statement at max_in_flight = 1.
  Fails without the behavior: Without the entry the diff is empty and the check fails.
  Test seam: No code seam; a content check of a file that exists.
- Criterion 6: The `records_order` tests keep one log vector holding `land X` and `records X`. They build the expected list from the landing order c, a, b, which differs from the start order a, b, c, compare whole sequences, and print the start, landing and records sequences in the assert message and with `println!`. The verifier runs with `--nocapture` and reads the printed lines for three jobs. Run: `cargo test -p athena-cli --test drain records_order -- --nocapture`. Observes: The landing sequence and the records commit sequence appear side by side for three jobs, match, and differ from the start order.
  Fails without the behavior: A test that only checks `commit_records` is called, or a count of 3, would not compare the order and would pass for a wrong order.
  Test seam: Same as criterion 1: `drain::drain_concurrent` with the recording stub (added by step 5).

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 6564c5bb8cd1 → candidate e145713f3913. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  11 +
 crates/athena-cli/src/drain.rs   |  35 +++
 crates/athena-cli/tests/drain.rs | 513 +++++++++++++++++++++++++++++++++++++++
 docs/design/async-drain.md       |  21 ++
 4 files changed, 580 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41355570950 for e145713f39130e8c3d0e964c2fc7558e66101261)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41355570950 for e145713f39130e8c3d0e964c2fc7558e66101261)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41355570950 for e145713f39130e8c3d0e964c2fc7558e66101261)
- ci-result: `not a check` passed (Check results for e145713f39130e8c3d0e964c2fc7558e66101261 come from the CI run https://github.com/doodla/athena/actions/runs/41355570950 (576s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass across the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
