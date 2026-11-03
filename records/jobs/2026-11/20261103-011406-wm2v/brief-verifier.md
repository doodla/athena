# Job 20261103-011406-wm2v: verify "Split athena-core --lib into filtered parts across CI test shards"

## Request (verbatim)

> try again on 20261103-005128-nfvz

## Reading (accepted by the user)

Retry of job 20261103-005128-nfvz (dispatched, never verified, no findings, no candidate files): teach `xtask test-shard` to split one test binary into parts by test-name module prefix, and split athena-core's `--lib` (821 tests, 255.11s) into parts that land on at least two of the six shards, each test running exactly once. Audience: CI maintainers and the verifier judging shard balance and exactly-once coverage.

Assumptions:
- Retry of job 20261103-005128-nfvz: it left no findings and no candidate files, so there is nothing to fix or keep; the work starts from the base and its six deliverables and out_of_scope items are carried over unchanged.
- Code Survey: Code survey: crates/athena-core/src/lib.rs (the --lib binary whose test names are module paths such as admission::tests::..., gh::tests::records_flow::..., run_pin_tests::...; run_pin_tests and truncate_tests are top-level modules). Entry point is xtask/src/shard.rs: plan_units builds Units from cargo metadata, assign balances them by timings, command runs `cargo test --locked -p <pkg> <selector>`. No record or data file is added under crates/, so check_record_file/AnyRecord are not touched.
- Design: a Unit gains an optional filter (module prefixes, or a catch-all that skips the other parts' prefixes via `--skip`). The filter is part of Unit::line() so timings keys, --dry-run and the run command all agree; parts are sorted deterministically. The part list for athena-core --lib is declared in xtask/src/shard.rs, and plan_units expands that one unit into parts.
- Prefix grouping is chosen so that part estimates (from the given 255.11s total, apportioned by test count per module group) are balanced; all estimates are labeled as estimates from run 41469096430. Largest modules by test count (gh, reading, run, hestia, verify, plan, disk, checks) go in separate parts; a skip-based catch-all part covers everything else including future modules.
- The coverage test in xtask/tests/shard_parts.rs obtains names from `cargo test -p athena-core --lib -- --list` (or an embedded snapshot of that list) and checks each name matches exactly one part's filter under libtest substring/--skip semantics; a negative test mutates the part set (drops or duplicates a prefix) and expects rejection.
- Other xtask/tests/ci_shards.rs checks are updated so they accept filtered units. No workflow file or athena.toml change is needed because the number of shards stays six.

## Acceptance criteria

1. [content] xtask/src/shard.rs lets a unit carry a test-name filter (module prefixes plus a skip-based catch-all), prints it in the --dry-run line, and runs it as `cargo test --locked -p athena-core --lib -- <filter>`; Grep of xtask/shard-timings.txt shows athena-core --lib as at least two part lines, not one.
2. [behavior] Run `cargo run -p xtask -- test-shard K 6 --dry-run` for K=0..5: athena-core --lib parts appear in at least two different shards, no part appears twice, and every other target appears exactly once across the shards.
3. [behavior] Run `cargo test -p xtask --test shard_parts` and `cargo test -p xtask --test ci_shards`: a test checks every name from `cargo test -p athena-core --lib -- --list` matches exactly one part's filter, and a negative test shows a dropped or duplicated prefix is rejected.
4. [content] xtask/shard-timings.txt lists the athena-core parts with estimated seconds summing to about 255s, notes they are estimates from run 41469096430, and no shard's estimated total exceeds roughly the mean plus the largest remaining single unit (lifecycle, 143.22s).
5. [content] CHANGELOG.md has an entry with the before numbers: athena-core --lib one binary of 821 tests at 255.11s (97% of shard 0's test time), shard 0 test step 5m45s vs shard 4 1m45s, and the next slowest binaries lifecycle 143.22s, drain 79.32s, retry 49.45s, job_plan 46.35s.
6. [behavior] Run `cargo test -p xtask`: unit tests in xtask/src/shard.rs for filtered units (assignment keeps every part exactly once, deterministic part ordering, timings round-trip with a filtered selector) pass.

## Expected deliverables

- xtask/src/shard.rs (modify)
- xtask/shard-timings.txt (modify)
- xtask/tests/ci_shards.rs (modify)
- CHANGELOG.md (modify)
- xtask/tests/shard_parts.rs (create)
- xtask/src/main.rs (modify)

## Out of scope

- Changing the number of shards or .github/workflows/ci.yml
- Adopting cargo-nextest
- Speeding up or editing the athena-core tests themselves
- Fetching anything from GitHub or re-measuring timings
- Splitting other binaries such as lifecycle.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests in shard.rs and the new shard_parts.rs test cover the change.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run test-shard --dry-run and the coverage test rather than checking that wiring exists.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Timings are labeled estimates from run 41469096430.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the sharding tool; other binaries are not split.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the CHANGELOG entry explain the filters for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 521f470c36a8 → candidate f23cf0586848. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md               |  12 ++
 xtask/shard-timings.txt    |  24 +++-
 xtask/src/main.rs          |   3 +-
 xtask/src/shard.rs         | 266 +++++++++++++++++++++++++++++++++++++++++----
 xtask/tests/ci_shards.rs   |  65 +++++++++--
 xtask/tests/shard_parts.rs | 154 ++++++++++++++++++++++++++
 6 files changed, 487 insertions(+), 37 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41560938422 for f23cf0586848a33ba32a12e4cd37fc84dcc3a026)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41560938422 for f23cf0586848a33ba32a12e4cd37fc84dcc3a026)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41560938422 for f23cf0586848a33ba32a12e4cd37fc84dcc3a026)
- ci-result: `not a check` passed (Check results for f23cf0586848a33ba32a12e4cd37fc84dcc3a026 come from the CI run https://github.com/doodla/athena/actions/runs/41560938422 (467s): fmt success, clippy failure, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
