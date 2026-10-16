# Job 20261016-200605-xh54: verify "Build bounded parallel drain with schema and athena.toml landing togethe"

## Request (verbatim)

> try again on 20261016-185517-qcrz

## Reading (accepted by the user)

Retry of the parallel-jobs build: the drain runs items of one plan group concurrently up to a new `[parallel] max_concurrency` setting (default 1). The `parallel` field is added to the deny_unknown_fields Config in config.rs in the same change as the athena.toml section, so the file parses at every commit. Audience: the Athena user running a drain, who wants shorter wall time with serial behavior unchanged by default.

Assumptions:
- The new request is the same as the failed job's; the only change is ordering: schema field and athena.toml section land in one change.
- Config uses deny_unknown_fields (read in config.rs), so the field must exist before the section is parsed.
- Submit-time config-parse refusal from the introspection is a separate change and out of scope here.

## Acceptance criteria

1. [behavior] A test loads the repository's real athena.toml (with its [parallel] section) through Config and passes; a config without [parallel] parses and yields max_concurrency 1; a config with an unknown top-level section is rejected. Run `cargo test`.
2. [behavior] A drain test with bound 1 shows items run strictly in the existing order, and all existing drain tests pass unchanged. Run `cargo test -p athena-cli`.
3. [behavior] A drain test with bound 2 and two independent items in one group has both in flight at once (barrier; fails if serial), and an item depending on another starts only after it finishes. Run `cargo test -p athena-cli`.
4. [behavior] Tests show one item's failure or refusal does not stop its siblings, the worker bound is never exceeded, and Stop starts no further item. Run `cargo test`.
5. [behavior] Ordering and retry rules are unchanged; `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.
6. [content] athena.toml gains [parallel] only together with its schema field in crates/athena-data/src/config.rs in the same change; the field, default (1) and ceiling are documented where defined, and the athena.toml change is named and justified in the final report.
7. [content] Drain and Report docs no longer say the loop runs one job at a time; docs/design/parallel-jobs.md and CHANGELOG.md state what this slice built and what remains (multi-job `athena go`, --pr phase).

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- docs/design/parallel-jobs.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/plan.rs (modify)

## Out of scope

- Submit/enqueue-time config parse refusal mechanism
- Multi-job `athena go`
- --pr phase

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for config parsing and drain concurrency; cargo test, clippy, fmt pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check concurrency via barrier test, not wiring.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test parses real athena.toml so schema/config drift fails the gate.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused slice; remaining parts deferred.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 96cbf077f77d → candidate a191824acd98. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |  14 ++
 README.md                           |   4 +-
 athena.toml                         |   5 +
 crates/athena-cli/src/drain.rs      | 474 +++++++++++++++++++++++++++---------
 crates/athena-cli/src/main.rs       |  10 +-
 crates/athena-cli/tests/drain.rs    | 310 +++++++++++++++++++++++
 crates/athena-core/src/admission.rs |  15 +-
 crates/athena-core/src/gh.rs        |   2 +-
 crates/athena-core/src/jev.rs       |  26 +-
 crates/athena-core/src/plan.rs      |   5 +-
 crates/athena-data/src/config.rs    |  75 ++++++
 docs/design/parallel-jobs.md        |  19 ++
 docs/design/roles-and-ladder.md     |   3 +-
 13 files changed, 828 insertions(+), 134 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (777 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
