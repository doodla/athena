# Job 20261013-050804-e6cj: verify "Skip cargo checks that a change's paths cannot affect"

## Request (verbatim)

> Retry 20261013-045315-qaph.

## Reading (accepted by the user)

Retry of the check-skipping job, fixing its three findings. Checks are skipped only when every changed path is in a configured "cannot affect" list, defined once and shared. Anything unlisted (studies/, *.json, Cargo files, any code) runs all checks, and the run loop is tested.

Assumptions:
- Safe direction: a check is skipped only if every changed path matches an inert list; an unmatched path runs the check. This fixes the blocker, where an allow-list of affecting paths missed studies/*.json.
- xtask/tests/policy.rs and other tests read docs files (docs/non-rust-files.md, docs/architecture.md, docs/enforcement.md), so those docs, and any file a test reads, must not be inert for the test check. Fmt and clippy can skip on any markdown-only change.
- The pattern list is defined once in athena.toml and shared by the change and study capabilities, e.g. a top-level table of named path sets that each check references, not copied per check.
- A skipped check is recorded as a passed CheckRun with a note 'not run: no changed path can affect it', as the earlier attempt did, so the record shows the skip.
- The nested-run skip stays as is; the new skip is evaluated from the job's change list (ChangeList::paths).
- athena.toml is a governance file and changes only governs later jobs; this job's own checks use its base config.

## Acceptance criteria

1. [content] athena.toml defines the 'cannot affect' path patterns once, and every check that uses them refers to that single definition; no pattern list appears more than once.
2. [content] A change touching only studies/**, any .json file, Cargo.toml/Cargo.lock, or any .rs file never skips the test check (covered by a unit test of the skip decision).
3. [content] Docs files that tests read (docs/non-rust-files.md, docs/architecture.md, docs/enforcement.md and any others found by grep for reads in tests) do not allow skipping the test check.
4. [content] Config parsing rejects or defaults safely: a check with no path set always runs; an empty change list runs every check.
5. [behavior] A run-loop test in athena-core shows that a job whose change list is docs-only records a CheckRun with the note 'not run: no changed path can affect it' for skipped checks, and still runs unaffected checks. Run: cargo test -p athena-core.
6. [behavior] A run-loop test shows a studies/*.json-only change runs the test check. Run: cargo test -p athena-core.
7. [behavior] cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings, and cargo test --locked pass.
8. [content] The change is documented: a decision entry in docs/decisions.md and a CHANGELOG.md line; athena.toml is named as a governance deliverable.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/enforcement.md (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)

## Out of scope

- Speeding up cargo itself (caching, incremental builds, shared target dirs).
- Changing what the verifier agent runs.
- Editing records/ or other governance files (defaults.toml, CI workflows, rust-toolchain.toml).
- Skipping the secrets or landing gates.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the skip decision and a run-loop test for the skip, answering the minor finding.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The skip rule is enforced by config plus code with tests, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the check runner and its config.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry and changelog written for an outside reader.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The retry's intent and failures were recovered from the quoted findings and the repository.

## The change

Base 84a0ffb70d97 → candidate b0c28375462d. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |   5 ++
 athena.toml                             |  14 ++++
 crates/athena-cap-change/src/lib.rs     |  63 +++++++++++++++---
 crates/athena-cap-study/src/lib.rs      |  11 +---
 crates/athena-core/src/capability.rs    |  33 +++++++++-
 crates/athena-core/src/checks.rs        |   4 ++
 crates/athena-core/src/run.rs           |  16 +++--
 crates/athena-core/src/run_pin_tests.rs | 110 ++++++++++++++++++++++++++++++--
 crates/athena-data/src/config.rs        | 102 +++++++++++++++++++++++++++++
 docs/decisions.md                       |  24 +++++++
 docs/enforcement.md                     |   1 +
 11 files changed, 351 insertions(+), 32 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (314 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
