# Job 20261031-031744-c5yx: verify "Add FIFO LandingQueue to Drain landing, without rebase"

## Request (verbatim)

> Priority: urgent
> Async drain A6, first half: an ordered landing queue in the drain, with no rebase
> 
> Build job A6 (docs/design/async-drain.md, decision D145) failed planning twice for size (20261030-013555-f5qm, 20261031-030119-tw3v). It is split. This half adds only the queue; the rebase before landing is a separate request that depends on this one.
> 
> Add a first-in-first-out landing queue to `Drain` in crates/athena-cli/src/drain.rs, as a small pure `LandingQueue` struct. A worker joins the queue when its run passes, keeps its place through the existing landing retry (the two land calls in `attempts`), and leaves on every exit through a drop guard. Only the queue head takes the `landing` mutex and lands. Landing itself is unchanged: no rebase, and a landing that fails behaves as today. Queue order is the order runs pass, not the order landing commits were stacked; say so in a short note in docs/design/async-drain.md. The `[parallel] max_concurrency` > 1 path is otherwise unchanged.
> 
> Acceptance:
> - Unit tests of `LandingQueue`: joins are served in order; a worker that leaves (drop) while not at the head is removed; the head keeps its place across a retry.
> - A drain test with two passed workers shows they land in the order they passed.
> - With max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, docs/design/async-drain.md, CHANGELOG.md

## Reading (accepted by the user)

Retry of failed jobs 20261030-013555-f5qm and 20261031-030119-tw3v, cut to the queue only: a pure FIFO `LandingQueue` in drain.rs, joined when a run passes, held through the two land calls in `attempts`, left via a drop guard; only the head takes the `landing` mutex. No rebase, no landing change. Audience: Sri and the verifier of the build jobs.

Assumptions:
- Prior jobs 20261030-013555-f5qm and 20261031-030119-tw3v produced no candidate (failed in planning for size); nothing from them is kept, so no files of theirs are carried over. Their findings are addressed by the split: 20261030-013555-f5qm's rebase, send_back/drop_landed, thrash and landing.rs findings no longer apply because this half has none of that; its max_concurrency > 1 finding is addressed by leaving landing unchanged and keeping that path otherwise as is. 20261031-030119-tw3v's left-behind records-loss finding and gh/mod.rs finding no longer apply (no rebase, no landing.rs or gh changes); its pass-order-vs-stacking-order finding is fixed by the design-doc note the request asks for; its size finding is addressed by the small scope.
- Code survey: crates/athena-cli/src/drain.rs: `Drain` has `landing: Mutex<()>` (line ~1041, built ~1859); `Drain::land` (~1330) takes the mutex then calls `steps.land`; `attempts` (~1376) loops, sets `passed` on `RunStep::Passed` and calls `self.land` once per loop pass, up to ATTEMPTS, which are the two land calls. The queue is joined once `passed` is set, kept through both calls, and left through a drop guard on every exit of the section. Waiting uses the existing `wake` condvar or a new Condvar paired with the state mutex. No record or data file is added, so no records validation or AnyRecord change.
- Queue order is the order runs pass (join order), not landing-commit stacking order; stated in a short note in docs/design/async-drain.md section 6, which also marks that the rebase is a later request.
- The drain test uses the existing test Steps fakes in drain.rs's tests module and drives two workers via the real drain entry, with the second worker's run passing first or gating by channel so pass order is deterministic.
- No change to athena.toml, defaults.toml or workflows is needed.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli drain`: new unit tests of `LandingQueue` pass, showing joins are served in order, a member that leaves (drop guard) while not at the head is removed without blocking others, and the head keeps its place across a retry (the same slot serves both land calls).
2. [behavior] Run `cargo test -p athena-cli drain`: a new drain test with two workers that pass one after the other shows the two landings happen in the order the runs passed, even if the later-started worker is ready first.
3. [behavior] Run `cargo test -p athena-cli`: with max_in_flight = 1 every pre-existing drain test passes unchanged; the diff in drain.rs tests only adds tests, and no existing test body is edited.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
5. [content] In crates/athena-cli/src/drain.rs `LandingQueue` is a small pure struct (no mutex or threads inside); the drop guard releases the slot on every exit of `attempts`; the diff leaves `Steps::land`, the lander and crates/athena-core untouched, and adds no rebase.
6. [content] docs/design/async-drain.md has a short note that queue order is the order runs pass, not the order landing commits were stacked, and that rebase comes in a later request.
7. [content] CHANGELOG.md has an entry for the ordered landing queue that says it does not rebase.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Rebase before landing, conflict return to the producer, send_back/drop_landed, thrash stop (the separate dependent request)
- Any change to crates/athena-core (gh/landing.rs, gh/mod.rs, run/) or how landing itself works
- Raising max_in_flight above 1 (build job 10) and the `[parallel] max_concurrency` path beyond the queue
- athena.toml, defaults.toml, .github/workflows, crates/athena-data/src/config.rs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and drain tests; fmt, clippy -D warnings and cargo test criterion.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check landing order and unchanged serial behavior by running the tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Order, drop and retry-place criteria run real code through tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Queue only; rebase left to a later request.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc note and CHANGELOG entry written for an outside reader.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in `landing_queue_tests`. Pure tests call `LandingQueue::join/is_head/leave`. Guard tests build a real `Drain::new(&fake, &ui, caps, limits)`, join through `Drain::enter_queue` and read `d.st().queue`. (a) Join three tickets: only the first is head; after it leaves the second is, then the third. (b) Join three, drop the middle guard: head and last stay in order, the middle ticket is gone, nobody is blocked. (c) The head's guard is held across two simulated land calls: `is_head` stays true for it and false for the one behind; dropping it makes the next one head. (d) A guard dropped by `catch_unwind(AssertUnwindSafe(..))` around a panic also leaves. (e) `leave` twice is harmless. The drain-level proof that `attempts` keeps one slot across both land calls is the retry test under criterion 2. Run: `cargo test -p athena-cli landing_queue_tests`. Observes: Joins are served in order. A member that leaves while not at the head is removed and the others keep their order. The head's place holds until its guard drops, including on panic.
  Fails without the behavior: Without the queue the module does not compile. If `leave` only popped the front, (b) leaves the dropped middle ticket and fails. If the guard did not release on drop, (c) and (d) fail.
  Test seam: Production entry points `LandingQueue::{join,is_head,leave}`, `Drain::enter_queue` and the `LandingSlot` drop, all added by plan steps 1 and 2. `Drain::new` exists (~1859). The only doubles are the small `Steps` fake and recording `Ui` added by plan step 5. They hold no queue logic and are injected as `&S` and `&U` into `Drain::new`.
- Criterion 2: Three tests through the public `drain_concurrent(&fake, limits, &ui, 2)`. The fake's `Steps` methods and the recording `Ui` append to one shared ordered log. The `Ui` records only lines containing `waits in the landing queue`. Every gate polls the log with a 5 s cap and, on timeout, appends `gate timed out`. (1) `a_waiting_worker_does_not_land_between_the_heads_two_land_calls`: items a and b with disjoint touches form one group, two workers. `run(a)` passes at once. First `land(a)` logs `land a`, waits for the wait line for b, then returns `NotLanded`. `run(b)` waits until the log holds the first `land a`, then passes. Second `land(a)` logs `land a` and lands. `land(b)` logs `land b` and lands. Assert the whole log equals [`land a`, `b waits in the landing queue`, `land a`, `land b`] and `report.landed == 2`. (2) `two_workers_land_in_the_order_they_passed`: a starts first but `run(a)` waits for `run(b)` to end. b passes first, becomes head and enters `land(b)`, whose first call waits for the wait line for a, and then returns Landed. a passes later, waits, then lands. Assert the whole log equals [`land b`, `a waits in the landing queue`, `land a`] with `landed == 2`, so a, the earlier-started worker, lands second. (3) `a_failed_head_releases_its_place`: both `land(a)` calls return `NotLanded` while b waits. b then lands, `landed == 1`, and the log has no `gate timed out`. Run: `cargo test -p athena-cli landing_queue_tests`. Observes: The head keeps its place through the retry: b passes while a is between its two land calls, b waits and says so, and b lands only after a's second call. The worker that passed first lands first although it started second, and the later worker is held back and says so. A failed head's guard releases its place.
  Fails without the behavior: On the base code no wait line is ever printed, so the gate in tests (1) and (2) always times out. The log then contains `gate timed out` and lacks the wait entry, so the whole-log assertion fails every time. This does not depend on how the unfair `landing` mutex is won. If a failed head did not release its place, test (3)'s b would wait forever and fail at the 5 s cap, with `gate timed out` or `landed` wrong. If the head lost its place between its two land calls, b would land between them and the log in (1) would differ.
  Test seam: Production entry point `drain_concurrent` (public, exists ~2202). It uses `Drain::attempts`, `enter_queue` and `await_head` (added by steps 2 and 3). The `Steps` fake and the recording `Ui` are new in the test module (step 5), passed as `&(dyn Steps + Sync)` and `&(dyn Ui + Sync)` and sharing one `Mutex<Vec<String>>`. They only log, poll the log and return scripted outcomes; the queue logic stays in production code. Guard: `max_concurrency` 2 gives two workers, and the two items must form one group (step 5 copies the disjoint-touch setup from `read_ahead_tests`; a wrong grouping makes the gates time out and the tests fail loudly).
- Criterion 3: Run the whole athena-cli suite, including the in-file drain tests and tests/drain.rs. They use `drain` with one worker, so the queue has one member that is always head, never waits and never prints. Then read `git diff` for drain.rs and check that existing test modules show only added lines and that tests/drain.rs is absent from the diff. Run: `cargo test -p athena-cli`. Observes: Every pre-existing test passes unchanged. The test diff is additions only.
  Fails without the behavior: If the queue blocked a lone worker, or the slot were not released after Landed or Failed, existing drain tests would hang or fail.
  Test seam: Existing entry points `drain` and `drain_concurrent` with the existing fakes (exist today). Nothing is added.
- Criterion 4: Run the three CI checks together. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: Exit 0 for all three. Clippy raises no dead-code, needless-lifetime or mut warning for `LandingQueue` or `LandingSlot`.
  Fails without the behavior: A fmt or clippy failure, a dead method on `LandingQueue`, or a failing test makes the command fail.
  Test seam: None. These are repository gates.
- Criterion 5: Read the diff. `LandingQueue` has only `u64` and `VecDeque<u64>` fields and no `Mutex`, `Condvar` or thread. `LandingSlot` is carried in the `passed` tuple in `attempts`, so it drops on every return. Nothing changes in crates/athena-core, `Steps::land` (~477, impl ~2996) or `Drain::land` (~1330), and no rebase code is added. Run: `git diff --stat`. Observes: Only drain.rs, async-drain.md and CHANGELOG.md appear. A search of the added code lines finds no `rebase`. The queue struct is pure.
- Criterion 6: Read the diff of the design doc. Under section 6 a short note says queue order is the order runs pass, not the order landing commits were stacked, and that the rebase before landing comes in a later request. Run: `git diff -- docs/design/async-drain.md`. Observes: The note exists, is short, and names both points.
- Criterion 7: Read the diff of the changelog. Under `## [Unreleased]` there is an entry for the ordered landing queue saying it lands first in, first out in pass order, that the head keeps its place across the landing retry, and that it does not rebase. Run: `git diff -- CHANGELOG.md`. Observes: An entry exists in Unreleased and its text says there is no rebase.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 23177cdcacb6 → candidate b728d6ac6261. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                   |   8 +
 crates/athena-cli/src/drain.rs | 423 ++++++++++++++++++++++++++++++++++++++++-
 docs/design/async-drain.md     |   6 +
 3 files changed, 429 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1834 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
