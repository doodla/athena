# Job 20261106-042847-ab51: verify "Raise drain max_concurrency ceiling from 8 to 16"

## Request (verbatim)

> try again on 20261106-022423-vdtn

## Reading (accepted by the user)

Retry of job 20261106-022423-vdtn: raise MAX_CONCURRENCY_CEILING in athena-data config from 8 to 16 so max_concurrency accepts 1 to 16 and 17 is refused with a 1-to-16 message, updating help (derived from the constant), docs and CHANGELOG. The prior xtask test failures looked unrelated (CI claude --version); this attempt runs full cargo test and fixes anything the ceiling change breaks. Audience: the operator running athena drain and reviewers.

Assumptions:
- Code survey: crates/athena-data/src/config.rs defines MAX_CONCURRENCY_CEILING (line 269, =8), used by the [parallel] max_concurrency check (~722), the error attribute (~588), and the max_in_flight check (~745); MAX_CHECK_SLOTS_CEILING (line 91) is a separate constant and stays 8. crates/athena-cli/src/main.rs (~553-563) fills the drain --help ceiling from the constant, with a test at ~1853. No record or data file is added, so check_record_file/AnyRecord are not involved. The ceiling is measured directly by Config::parse in tests.
- Job 20261106-022423-vdtn: its candidate changed CHANGELOG.md, athena.toml, config.rs, and the three design docs; all are kept as work to redo from base. Its findings are addressed: (a) CHANGELOG nit: the entry goes under 'Changed', not 'Added'; (b) the xtask repin test failure (claude --version failed in CI, xtask untouched) is environmental and unrelated, so xtask is not changed; the producer runs the full cargo test and reports any test the ceiling change genuinely breaks; (c) the test check failure is otherwise unreadable, so no further cause is assumed.
- max_in_flight shares MAX_CONCURRENCY_CEILING, so its range also becomes 1 to 16; docs stating 'max_in_flight outside 1 to 8' (async-drain.md ~264) are updated, but max_check_slots stays 1 to 8.
- Prior candidate's athena.toml change was only the range comment (line 192); value max_concurrency = 2 stays. No README.md or crates/athena-cli/src/main.rs edit is expected since help is derived from the constant.
- Changing Athena to store failing test names from test output (the introspection's proposal) is a separate request and is not done here.
- Historical CHANGELOG entries and docs/decisions.md records that say 1 to 8 are history and stay.

## Acceptance criteria

1. [behavior] Run cargo test -p athena-data: the range test shows max_concurrency = 16 loads and 17 is refused with an error containing '1 to 16'; all pass.
2. [behavior] Run cargo test -p athena-cli: help test passes and states the ceiling as 16 (stated via the constant); cargo run -- drain --help shows 16 and not 8.
3. [behavior] Run cargo test --locked from the repo root, cargo clippy --all-targets and cargo fmt --check: all clean; any test newly failing because of the ceiling is fixed in this diff, and xtask is untouched unless a test genuinely depends on the ceiling.
4. [content] Grep of docs/design/async-drain.md, docs/design/parallel-jobs.md and athena.toml finds no remaining 'max_concurrency'/max_in_flight range stated as 1 to 8; they say 1 to 16. max_check_slots keeps 1 to 8.
5. [content] athena.toml diff changes only the comment; max_concurrency = 2 is unchanged.
6. [content] CHANGELOG.md has one new entry under 'Changed' that states only what the diff does (ceiling 8 to 16, error and help text, docs) and does not mention plan group size recording; older entries are untouched.
7. [content] docs/design/agent-and-build-slots.md step 6 text reflects the ceiling now being 16 where it describes the built drain, without implementing other steps.
8. [content] No code or docs add plan group size recording.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- docs/design/async-drain.md (modify)
- docs/design/parallel-jobs.md (modify)
- docs/design/agent-and-build-slots.md (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/src/main.rs (modify)
- README.md (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- Recording plan group sizes
- Changing max_concurrency = 2 in athena.toml
- Rewriting historical CHANGELOG entries or docs/decisions.md records
- Other build steps of agent-and-build-slots.md
- Changing MAX_CHECK_SLOTS_CEILING
- Changing Athena to store failing test names from test output
- Editing xtask code (the repin test failure was environmental)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust constant plus tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Full cargo test, clippy and fmt required.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test and drain --help.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check 16 loads and 17 refused, not just that the constant changed.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the ceiling; no group-size recording.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Help and error derive from the constant; docs grep covers all restatements.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and docs written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f0c92648fa5d → candidate 781b25f0b2c7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         | 6 ++++++
 athena.toml                          | 2 +-
 crates/athena-data/src/config.rs     | 6 +++++-
 docs/design/agent-and-build-slots.md | 4 ++--
 docs/design/async-drain.md           | 6 +++---
 docs/design/parallel-jobs.md         | 2 +-
 6 files changed, 18 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-042847-ab51/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-042847-ab51/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42163314540 for 781b25f0b2c7815ac25ecfb8785e2d1fe4a4725d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42163314540 for 781b25f0b2c7815ac25ecfb8785e2d1fe4a4725d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42163314540 for 781b25f0b2c7815ac25ecfb8785e2d1fe4a4725d)
- ci-result: `not a check` passed (Check results for 781b25f0b2c7815ac25ecfb8785e2d1fe4a4725d come from the CI run https://github.com/doodla/athena/actions/runs/42163314540 (473s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo test --locked from the repo root, cargo clippy --all-targets and cargo fmt --check: all clean; any test newly failing because of the ceiling is fixed in this diff, and xtask is untouched unless a test genuinely depends on the ceiling.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
