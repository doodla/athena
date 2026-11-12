# Why the verifier stage grew from 3.1 to 3.5 minutes per landed job

Written for the maintainer who reads the next throughput review.

## What was measured

The throughput section of `athena status` gives the verifier stage as the median, over the newest landed jobs,
of `role_minutes(job, [Role::Verifier])` (`crates/athena-core/src/status.rs`, `landed_jobs`). That is the wall
time of the verifier agent's invocations and nothing else: checks, the replay and the demonstration runs are
other stages. The review that opened this job read 3.1 before and 3.5 now. Running
`athena status --snapshot <evidence dir>` on this job's evidence snapshot printed `verifier 3.4` over the last
20 measured landed jobs (measured, one run). So the growth is in what the verifier agent does inside its run.

## Cause

The verifier's prompt tells it to exercise each behavior criterion by running commands, and its grant allows
`cargo build`, `check`, `test`, `clippy`, `fmt`, `run` (`athena.toml`, `verifier_commands`). Those commands
build in the verifier's own build path, `CARGO_TARGET_DIR = {job_dir}/target/verify`.

Before build slots owned warm directories, the checks built in that same path, so by the time the verifier
started, the third-party crates were compiled there. The warm-target change (CHANGELOG, "Warm target
directories owned by build slots") moved the checks' builds into a slot's directory and, in
`SnapExec::exec` (`crates/athena-core/src/run/checks.rs`), drops the link after every check command. Nothing
linked the verifier's path again, so `target/verify` was empty when the verifier started
(`verification_stage` in `crates/athena-core/src/run/verification.rs` invoked the agent with no link). Every
`cargo test` or `cargo clippy` the verifier ran to show a behavior criterion compiled all third-party crates
from nothing. The more criteria that ask for a command, the more cold builds; that is the added verifier time.

This is read from the code path; the per-command build time was not measured separately, because the records
hold the verifier's wall time but not its commands' durations.

## Change

`Athena::link_verifier_build` and `release_verifier_build` (`crates/athena-core/src/run/checks.rs`), called
around the verifier invocation in `verification_stage`:

- before the agent starts, take a build slot and link the verifier's `CARGO_TARGET_DIR` to the slot's warm
  target directory (the same directory and the same size limit, `slot_max_gb`, as the checks use);
- after the agent ends and the snapshot is unlocked, remove the link, run `cargo clean -p` for the workspace
  members and free the slot, exactly as after a check.

Any failure (no slot within the wait bound, a directory that cannot be linked) prints a warning and the
verifier builds cold as before; it never holds or fails a job. The slot is held for the length of the verifier
run, so with `max_check_slots = 1` the checks of another job wait for it; the checks of the same job are done
by then.

## Test

`the_verifier_builds_in_the_slots_warm_target_directory` (`crates/athena-core/src/run/tests.rs`) has the check
write a third-party file into the slot and has the verifier's fake agent read its own `CARGO_TARGET_DIR`: it
must be a link to the slot's directory holding that file, the link must be gone afterwards and the slot free.
It fails on the old code, where the path is not a link and the directory is empty. The over-limit slot test
now sees the deletion at the verifier's use, since the verifier is the next use of the slot.

## What the next throughput review should show

The verifier column should fall below 3.5 minutes per landed job and move back toward 3.1, for jobs landed
after this change. Read it with `athena status` (the verifier figure in the throughput line) or the review's
stage table, over at least 20 landed jobs after the change, since the median covers the newest 20. If it does
not fall, the cold build was not the main cost, and the next step is to record the duration of each command
the verifier runs.
