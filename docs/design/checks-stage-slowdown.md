# Why the checks stage grew from 0.2 to 0.6 minutes per landed job

Throughput review finding: the median `checks` time per landed job rose from 0.2 to 0.6 minutes. This note
states the cause, the evidence and the change.

## What the stage measures

`status::check_minutes` (crates/athena-core/src/status.rs) sums the measured duration of every check in a
job's `verification.json`, except those whose basis is "not run". A check taken from a green CI run is stored
with a duration of 0 seconds ("recorded result"), so a job whose fmt, clippy and test results are reused adds
nothing. Time enters the stage only from checks that ran on this machine.

## Cause

Since the reuse of CI results, the three configured checks cost nothing for most jobs. What is left in the
stage is the planned "fails without the fix" demonstrations, which Athena runs itself
(`Athena::demonstration_runs` in crates/athena-core/src/run/checks.rs). Each is stored as a check named
`fails-without-<n>` with its measured duration, and up to `MAX_DEMONSTRATION_RUNS` (3) distinct commands run
one after the other in the same reverted scratch copy.

A demonstration very often fails because the tests do not compile once the fix is reverted: new functions
the tests call are gone. In that case every command of the same package fails at the same place, yet each
one started its own `cargo test` and compiled the same tree again.

Evidence, read from the job records (measured by Athena's monotonic clock, one job each):

- Job `20261104-141009-hsg4`: three demonstrations, all `cargo test -p athena-core ...`, all stopped by the
  same 43 compile errors, took 41.7 s, 16.8 s and 16.8 s. The last two repeat the first one's failure: 33.6 s
  of the 75.3 s.
- Job `20261104-124034-ysc4`: three demonstrations that took 43.6 s, 27.3 s and 5.5 s.
- Jobs with reused CI results record 0 s for fmt, clippy and test (for example `20261102-073012-0w60`).

This is the cause for the jobs that carry demonstrations. I did not recompute the median of the last 20
landed jobs from the records, so the share of the 0.4-minute rise that this accounts for is an estimate: a
repeat costs 17 s to 27 s, and the median job in the window spends 36 s in this stage.

## Change

`Athena::demonstration_runs` keeps the demonstrations that failed to build (`verify::failed_to_build`: a
nonzero exit, no test run, and "could not compile" in the output) together with their build scope
(`verify::build_scope`: the flags and their values, without the test-name filter and without anything after
`--`). A later demonstration with the same scope takes that result instead of starting cargo again. Its
record is still written: the command, exit code, output tail and criteria are those of the earlier run, with
a duration of 0 s and a note naming the run it repeats. The verifier's view and the verdict logic are
unchanged.

Commands with another scope (another package, `--lib`, other flags), runs that compiled, and runs that did not
finish are still run in full. A run that built is never reused, because a test filter selects different
tests.

## Test

`demonstrations_of_one_build_scope_compile_once_when_the_build_fails` (crates/athena-core/src/run/tests.rs)
plans three demonstrations, two for `-p demo` and one for `-p other`, whose command always fails to compile,
and counts the runs: two, not three. Without the change the count is three. Unit tests in
crates/athena-core/src/verify.rs cover the scope and the failed-build rule.

## Next review

The next throughput review should show `checks` falling back for jobs with demonstrations whose tests do not
compile without the fix. That is a prediction, not a measurement.
