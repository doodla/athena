# CI routing: wall-time baseline

**Status:** build-plan item 2 of `docs/design/ci-routing.md` (section 10). It adds the computation and records
what the "before" figures are. It changes no routing. The design itself is still not accepted; its review
(`docs/design/reviews/ci-routing-review-1.md`) is not done.

**Reader:** the owner and the design reviewers. Paths are relative to the repository root.

Every number below is marked **measured** (computed by a command in this repository), **given** (stated in the
request) or **estimate** (not measured).

## 1. The measure

Wall time per verified result is the time from a job's `dispatched_at` (in `records/jobs/<month>/<job>/job.json`)
to the `at` of its outcome record (`outcome*.json` in the same directory). A job is verified when its last
outcome is `landed`, or `blocked` (verified, landing refused by local edits). Jobs whose outcome is `failed` or
`rejected` are reported separately as time per job, so hard jobs cannot be dropped to improve the number. Other
outcomes (`stale`, `dropped`, `declined`, `abandoned`, `closed`) are counted but not timed. Jobs are grouped by
`class_of_work`; a job without one is `unclassified`.

The records hold no per-phase timestamps (producer, CI wait, verifier, checks, landing), so only the whole
dispatch-to-outcome time is computed. Per-phase times are **unavailable**; adding them is left to a later job.

## 2. The command

```
cargo run --locked -p xtask -- walltime --capability change
```

Flags: `--root <dir>` (default: the repository root), `--last <N>` (default 20), `--capability <name>` (default:
all capabilities). It reads the last N finished jobs by dispatch time, and prints for each job kind and for all
kinds: the count, the median and the 90th percentile (nearest rank), once for verified results and once for
failed or rejected jobs. If fewer than N jobs exist it prints a `CAVEAT` line. If the records have no usable
timestamps, or no jobs, it prints `UNAVAILABLE` and the reason; it never prints a zero or an empty table.

The unit tests are in `xtask/tests/walltime.rs`; they run the command on sample records.

## 3. Before

| Figure | Value | Mark |
|---|---|---|
| Median wall time per job, producer ~12 min, verifier ~6 min, checks ~6 min | about 25 min | **given** (the request; not measured) |
| Median and p90 from job records, last 20 change jobs | not computed in this job | The job's working copy has no `records/` directory (records are left out of it by design), so the command prints `UNAVAILABLE`. **Measured:** that output. |
| Per-phase times | unavailable | The records hold no phase timestamps. |
| Cold and warm CI build time, target cache size | not measured | Needs CI runs on the default branch after the cache layers land. |

To fill the second row, run the command in a checkout that has the job records, before the cache layers are
in use, and paste its output here. Until then the only "before" figure is the given 25 minutes.

## 4. After

Same command, same flags, N = 20, over the first 20 change jobs dispatched after the cache layers (and later
the routing) are in use:

```
cargo run --locked -p xtask -- walltime --capability change --last 20
```

Because the window is "the last N", run it when exactly 20 jobs have finished since the build, or filter the
records by date. Compare per job kind: the after median against the before median, and the after p90 against the
before p90, as design section 9.4 says. The strength guard of 9.4 (checks per job do not fall) is not computed
by this command. N = 20 is an **estimate** of a workable size, not derived from variance.

The after figures are not measured in this job: no jobs have run since the cache layers were added. Build-plan
item 6 records them.
