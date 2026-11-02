# Job 20261102-073012-0w60: verify "Balance xtask test-shard by recorded per-target timings"

## Request (verbatim)

> try again on 20261031-172034-fx4m

## Reading (accepted by the user)

Retry of job 20261031-172034-fx4m (timed out at 3000s, no candidate, no changes kept). Change `xtask test-shard` to place units greedily longest first by durations in a checked-in timings file under xtask/, with the old sorted index-modulo split for untimed units, plus an xtask command that rewrites the file. Audience: CI maintainers who want the slowest shard near the mean. Keep it small to avoid another timeout.

Assumptions:
- Code survey: I read xtask/src/shard.rs, xtask/src/main.rs, xtask/tests/ci_shards.rs and crates/athena-data/src/records.rs. Entry point: main.rs dispatches `test-shard` to shard::command, which runs `cargo metadata`, calls plan_units (sorted, deduped Unit list), then select(units,k,n) (sorted index i, shard i % n), then dry-run prints or runs `cargo test -p <pkg> <selector>` per unit. Order for the change: plan_units first, then load timings (new), then timed units are placed greedily longest first on the least-loaded shard (ties broken by sorted order), then untimed units keep the index-modulo split over the sorted untimed list, then the shard k slice is returned. Every shard computes the same full assignment, so the partition holds. ci_shards.rs checks the partition through the built `xtask test-shard k n --dry-run` output against a metadata oracle; the real timings file is read by that path.
- Code survey: the timings file is xtask/shard-timings.txt, not under crates/ and not under records/. check_record_file in crates/athena-data/src/records.rs accepts only canonical records/*.json files and a fixed list of job Markdown files, and returns UnknownFile otherwise. A file under xtask/ is not passed to it, so no AnyRecord variant, check_record_file path or round-trip test is added or needed.
- Code survey: the balance property is measured directly in a shard.rs unit test. It calls the real selection function for n=6 over a fixture timings map, sums each shard's fixture durations, and compares the maximum with the mean. It also counts how often each unit appears across the shards. It does not use a stand-in for the selection.
- The change adds no refusal rule to the Athena reading or records gate, so no refusal-rate criterion is needed. The `test-shard` error on a malformed timings line is a user-facing parse error.
- Job 20261031-172034-fx4m is addressed by redoing its accepted reading from the base: its only recorded finding was the 3000s producer deadline, so there is no verifier finding to fix. The aim is to avoid a timeout by keeping the change small and the test fixtures simple.
- The timings file format is one line per unit: `<package> <selector> <seconds>`. The seconds come from the run-41221582236 totals, or from a local run if the producer can make one. If a measured value is unavailable, a clearly labeled estimate is acceptable. The update command may time each unit with `cargo test`.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`. A shard.rs unit test calls the real selection with a fixture timings map for n=6 and asserts that the largest per-shard duration total is at most 1.25 times the mean, and that each fixture unit appears in exactly one shard.
2. [behavior] Run `cargo test -p xtask`. A shard.rs unit test includes a unit absent from the fixture timings and asserts that it is assigned to exactly one shard, with every unit still covered.
3. [behavior] Run `cargo test -p xtask --test ci_shards`. The existing partition tests pass, including `shards_in_ci_yml_partition_every_workspace_test_target`, using the real checked-in xtask/shard-timings.txt.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. All pass.
5. [behavior] Run `cargo test -p xtask`. A shard.rs unit test shows a malformed timings line (missing field or non-numeric seconds) returns an error that names the line.
6. [content] xtask/src/shard.rs sorts timed units by descending duration and places each on the currently least-loaded shard. Untimed units use the existing sorted index-modulo split. A new xtask subcommand that rewrites xtask/shard-timings.txt from a local run is wired in xtask/src/main.rs and listed in the usage text.
7. [content] xtask/shard-timings.txt exists and has a line for every current workspace test unit that `cargo run -p xtask -- test-shard 0 1 --dry-run` lists, so the real shards are balanced rather than all falling back.
8. [content] CHANGELOG.md has an entry describing timing-balanced sharding, the timings file and the update command.

## Expected deliverables

- xtask/src/shard.rs (modify)
- xtask/shard-timings.txt (create)
- xtask/src/main.rs (modify)
- CHANGELOG.md (modify)
- xtask/tests/ci_shards.rs (modify)

## Out of scope

- Changing the shard count or .github/workflows/ci.yml
- Adding a sharding tool such as cargo-nextest
- Splitting individual tests within a target
- Changing athena.toml or defaults.toml
- Any record or data file under crates/ or records/
- Keeping work from job 20261031-172034-fx4m: it left no candidate, so nothing is kept or dropped

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; fmt, clippy -D warnings and cargo test are criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier runs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check balance and partition by running tests, not that files exist.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: No sharding crate or tool is added; the existing cargo metadata approach is extended.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change inside xtask plus the changelog.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the timings format and the fallback.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Timings are measured or labeled as estimates.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ac8c6f86ed2e → candidate 197bdcaf59ac. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md             |   8 ++
 xtask/shard-timings.txt  |  63 ++++++++++
 xtask/src/main.rs        |   5 +-
 xtask/src/shard.rs       | 320 +++++++++++++++++++++++++++++++++++++++++++----
 xtask/tests/ci_shards.rs |  13 ++
 5 files changed, 381 insertions(+), 28 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41409510508 for 197bdcaf59acbd89d1dcdbb2e3c7ce2577766786)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41409510508 for 197bdcaf59acbd89d1dcdbb2e3c7ce2577766786)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41409510508 for 197bdcaf59acbd89d1dcdbb2e3c7ce2577766786)
- ci-result: `not a check` passed (Check results for 197bdcaf59acbd89d1dcdbb2e3c7ce2577766786 come from the CI run https://github.com/doodla/athena/actions/runs/41409510508 (518s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. All pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
