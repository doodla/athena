# Job 20261014-070345-v28a: verify "Add read-only `athena strategy` command and a strategy-view drift test"

## Request (verbatim)

> Make the strategic view generated so it cannot go stale: add a read-only 'athena strategy' command that reads docs/design/requirements-map.md and prints coverage per goal (rows, covered, partial, not covered) and the not-covered rows of each goal, and add a test that fails when the counts in docs/design/strategy-view.md differ from what the command computes from the map.

## Reading (accepted by the user)

Add `athena strategy` (with `--json`), which parses the map's tables for row statuses, takes each goal's row IDs from the goal sections of strategy-view.md (the map has no goal column), and prints rows, covered, partial and not covered per goal plus each goal's not-covered row IDs. Add a test that fails when the "Coverage at a glance" table in strategy-view.md differs from the computed counts.

Assumptions:
- The map has no goal column, so goal membership comes from the per-goal Covered/Partial/Not covered bullet lists in strategy-view.md; those bullet lists are not themselves compared, only the table counts (and ID membership) are.
- The current table counts in strategy-view.md may already be wrong; if the test finds a mismatch, the producer corrects the table (and bullets) to the computed values.
- The test reads the real files via CARGO_MANIFEST_DIR-relative paths.
- No new crate is needed; a small hand parser suffices.

## Acceptance criteria

1. [content] A new strategy module in athena-cli parses the map's tables (rows whose first cell is an ID, status = first word of the Status cell: Covered, Partial, Not covered) and is exposed as a `Strategy` subcommand in main.rs that opens no writes and edits no file.
2. [content] Per goal the output has rows, covered, partial, not covered and a not-covered row list (or 'none'), plus a total line; goal membership is read from the goal sections of strategy-view.md; an ID in a goal list missing from the map, or a map row in no goal, is reported as an error.
3. [behavior] `cargo run -q -p athena-cli -- strategy` (or the package that holds the binary) exits 0 from the repository root and prints eight goals whose counts match the table in docs/design/strategy-view.md.
4. [behavior] `cargo test` passes, including a test that parses the 'Coverage at a glance' table of docs/design/strategy-view.md and asserts every goal row and the total equal the computed counts, with a message naming the goal and the differing numbers.
5. [behavior] Unit tests on inline map and view text show the drift test fails when a count is changed or a status flips, and show parsing handles Not covered with trailing text.
6. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
7. [content] strategy-view.md's intro no longer says counts are hand-counted and snapshot-only; it points to `athena strategy` and the test. CHANGELOG.md has an entry.

## Expected deliverables

- crates/athena-cli/src/strategy.rs (create)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- docs/design/strategy-view.md (modify)
- docs/design/requirements-map.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/roadmap.md (modify)
- docs/decisions.md (modify)
- crates/athena-cli/Cargo.toml (modify)

## Out of scope

- Generating strategy-view.md prose, bets or risks
- Changing statuses in the requirements map beyond fixing a miscount
- The roadmap's next-steps section
- CI workflow changes

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust in athena-cli.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests plus the drift test; clippy and fmt clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command, one module, one test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Staleness is fixed by a failing test tied to computed counts, not only by a recount.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: No crate fits a bespoke markdown table; a small parser is used.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the command and the tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base db45b4280df4 → candidate 0959b3ca796a. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                      |   8 +
 crates/athena-cli/src/lib.rs      |   1 +
 crates/athena-cli/src/main.rs     |  16 ++
 crates/athena-cli/src/strategy.rs | 419 ++++++++++++++++++++++++++++++++++++++
 docs/design/strategy-view.md      |  13 +-
 5 files changed, 452 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (536 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
