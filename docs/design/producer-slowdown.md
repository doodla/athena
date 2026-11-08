# Why the producer stage grew, and what was removed

Audience: Athena maintainers reading the next throughput review.

The cause below is read from the code and from dated history. The job's snapshot held no `records/`, so no
timing of a single invocation was measured here. Every figure that is not quoted from the request is labeled an
estimate.

## 1. The figures (measured, quoted from the request)

Median minutes per landed job, previous review to this one:

| Stage | Before | Now |
| --- | --- | --- |
| producer | 12.7 | 23.5 |
| planning | 1.7 | 9.0 |
| checks | 0.3 | 0.7 |
| verifier | 3.4 | 3.6 |
| all stages (median total) | 918.1 | 1604.8 |

The producer grew by 10.8 minutes, the most of any stage. Planning grew by 7.3 minutes; it is out of scope
here unless it shares the cause, and nothing below says it does.

## 2. How the stage is measured

`role_minutes` (`crates/athena-core/src/status.rs:933`) sums, for one job, the wall time of every invocation in
the given roles (`started_at` to `ended_at`). So the producer stage is the sum of the producer invocations of a
landed job. Two consequences:

- Everything the producer agent does inside an invocation counts, including every `cargo` command it runs.
- A send-back is a whole new producer invocation, so it adds its full length. The in-process checks that
  Athena runs between invocations (the mirror replay in `crates/athena-core/src/run/producer.rs`) are outside
  any invocation and are not in the producer stage.

This job changes none of that: `status.rs`, `store.rs` and `main.rs` only sum recorded durations.

## 3. Timeline (from the repository history and `CHANGELOG.md`)

- 2026-11-04: the previous throughput review.
- 2026-11-05: `max_concurrency = 2` (`athena.toml:195`), two jobs build at a time.
- 2026-11-06: four changes that add work to the producer path: a decision gate, an allowed-paths check, the
  producer brief's instruction to run `cargo test -p athena-cli --test mirrors` before finishing, and the
  mirror replay in the producer loop.
- 2026-11-07: the producer's cargo wrapper and the build-slot queue.

## 4. The cause

The producer brief told every producer, on every job, to run `cargo test -p athena-cli --test mirrors` before
it finished. The text was the constant `MIRROR_CHECK_INSTRUCTION`
(`crates/athena-context/src/briefs.rs`, rendered by `mirror_check_section` into the "Files kept by hand"
section of `producer_brief`).

Why that adds minutes to the producer stage:

- `athena-cli` depends on the agent, cap-change, cap-study, core, data, gate and git crates, and has the core
  `test-support` feature and the testkit as dev-dependencies (`crates/athena-cli/Cargo.toml`). An integration
  test run also builds the package's binary. So the command compiles most of the workspace in the test profile.
- The producer builds into its own cold directory: `CARGO_TARGET_DIR = "{job_dir}/target/{side}"`
  (`athena.toml:80` and `:111`). Nothing from an earlier job is reused, so the whole build is paid on each job.
- It runs inside the producer invocation, so `role_minutes` counts it.
- It applies to every job, docs-only jobs included. A cost that every job pays moves the median; a trigger
  that fires on some jobs would move it less.
- The date fits: the instruction landed on 2026-11-06, after the previous review of 2026-11-04.

The instruction bought no coverage. Two checks already catch the same drift:

- The producer loop compares the hand-kept files with their sources in-process after every report, with no
  build (`mirror_drift_in`, `crates/athena-core/src/run/producer.rs:76`, called from the loop at about line
  362). A drift is sent back once, with the file and the fix.
- The configured test check runs the whole `mirrors` target at verification (decision D158, "Tradeoff").

Size of the cost: the only measurements at hand come from this job's own work, in a target directory that was
already partly built by earlier commands: `cargo test -p athena-cli --test mirrors` took 30.8 seconds to compile
there (measured), and a build of `athena-core` and its tests took 44 seconds from a colder start (measured). On a
fully cold directory, with two jobs building at once, a producer that ran it more than once, or a producer
queued behind a slot, the cost is higher (estimate: one to several minutes per run). So the build alone may
explain only part of the 10.8 minutes; this is why section 5 lists the other candidates and section 8 does not
promise a full return to 12.7.

## 5. Other candidates, and the record that settles each

In rough order of confidence after the cause above (all estimates):

- (a) Send-back passes. The decision gate, the allowed-paths check, the mirror replay and the instruction all
  landed on 2026-11-06. A send-back starts a fresh session with the whole brief; agent sessions are not
  resumed. They fire on some jobs only, so they move a median less. Settled by the number of producer
  invocations per landed job and the `self-check failed:` detail.
- (b) Build-slot waits. The wrapper (`crates/athena-cli/src/slot_exec.rs`, `crates/athena-core/src/slots.rs`)
  exists only since 2026-11-07, so most of the review window never waited on a slot. Settled by the wait
  recorded in the producer's command records.
- (c) CPU contention from two jobs building at once (`max_concurrency = 2`, since 2026-11-05). Settled by
  comparing the duration of the same command in jobs that overlapped and jobs that did not.
- (d) Heavier readings and plans. This fits planning rising from 1.7 to 9.0 and is out of scope here.

The record that confirms the chosen cause is `commands_run` of producer invocations from 2026-11-06 on
(`crates/athena-data/src/records.rs`): those that hold `cargo test -p athena-cli --test mirrors`, against their
duration and against invocations that do not hold it.

## 6. The change

`MIRROR_CHECK_INSTRUCTION` is replaced by `MIRROR_CHECK_NOTE`. It keeps the two facts the old text carried (a
new test file needs its line in `xtask/shard-timings.txt`; `docs/decisions.md` only gains text after its last
line) and says that Athena compares these files with their sources after each report and sends a drift back.
It names no command. Tests:

- `producer_briefs_do_not_tell_the_producer_to_build_for_the_mirror_check` (`briefs.rs`) takes the section from
  the real `producer_brief`, and fails on the old text because it holds `cargo` and `--test mirrors`.
- `mirrors_the_producer_is_not_asked_to_build_and_the_loop_still_sends_the_drift_back`
  (`crates/athena-core/src/run/tests.rs`) runs a whole job and checks the producer's first prompt: the facts are
  there and the command is not. It also checks that a missing timings line is still sent back, so no coverage
  is lost.

The trade: a producer that adds a test file and forgets the timings line now finds out from a send-back (one
more producer pass) instead of from a build it ran on every job. The loop still sends it back, and the
`mirrors` target still fails at verification. If the next review shows many mirror send-backs, the follow-up is
to give the instruction only to jobs whose planned paths touch a watched file; that needs the registry in
`athena-core`, which `athena-context` cannot depend on, so it is not done here.

Left alone on purpose: the boundary-scan instruction (`cargo test -p xtask --test policy`, in both review
windows), the send-back triggers and limit, the slot count and wrapper wait cap, `max_concurrency`, planning,
and all stage reporting and thresholds. Raising a timeout or changing the report would hide the time, not
remove it.

## 7. A stale sentence

`docs/decisions.md` D158 says "The producer brief gains a short note to run the target". That sentence no
longer holds. The log is append-only and outside this request's files, so it is left as it is; whether to log
the supersession as a new decision is the maintainer's call.

## 8. What the next review should show

A lower producer median than 23.5 minutes. The size is an estimate and may be smaller than 10.8 minutes,
because:

- the next window also holds jobs that waited on build slots (since 2026-11-07) and ran two at a time, which
  can raise the producer stage regardless of this change; and
- send-back passes and heavier plans (candidates (a) and (d)) are untouched.

Planning, checks and the verifier are not expected to move because of this change.

To check: read the stage medians from `athena status` and the stored throughput comparison, and confirm from
`commands_run` of recent producer invocations that none holds `cargo test -p athena-cli --test mirrors`. If the
producer median does not fall, read the invocation count per landed job and the slot waits next, as listed in
section 5.

## 9. Estimates and gaps

- The share of the 10.8 minutes due to the build is an estimate; no per-invocation timing was available.
- The share of review-window jobs that ran under the 2026-11-06 brief is unknown here.
- No `records/` was in the snapshot; the cause is read from code and dates, and section 5 names the records
  that would confirm or reject it.
