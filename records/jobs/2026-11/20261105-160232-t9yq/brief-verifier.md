# Job 20261105-160232-t9yq: verify "Raise [parallel] max_concurrency to 2 and pin tests to a fixture value"

## Request (verbatim)

> try again on 20261105-103552-6s9v

## Reading (accepted by the user)

Retry of 20261105-103552-6s9v: set max_concurrency = 2 in athena.toml, make the athena-cli test fixture config pin max_concurrency = 1 so drain tests stop reading the live value, and add a CHANGELOG entry. Audience: whoever runs batches and the verifier running the full suite.

Assumptions:
- Prior job 20261105-103552-6s9v: its athena.toml value change is kept; its findings are fixed: the three drain tests (blocker) via the fixture pin, the missing CHANGELOG entry (major) by adding it, the stale 'Left at 1' comment (minor) by rewording it, and the misread scope (major) because the scope now allows the test and changelog files.
- Code survey: crates/athena-cli/tests/common/mod.rs test_config builds from include_str!(athena.toml), which is why drain.rs tests at 3162, 7706, 7839 broke; pinning the value there is the smallest root-cause fix. Production code under crates/ is untouched and no record or data file is added (check_record_file/AnyRecord not involved).
- Default concurrency 2 is accepted by the schema (1 to 8), and the disk check already holds jobs on low disk.
- Other tests that rely on serial drain use test_config and so keep seeing 1.

## Acceptance criteria

1. [content] Grep of athena.toml finds max_concurrency = 2 under [parallel] and no max_concurrency = 1; the comment above [parallel] no longer says the value is left at 1.
2. [content] crates/athena-cli/tests/common/mod.rs test_config sets max_concurrency = 1 in the fixture it returns (replacing whatever the live athena.toml holds), so the 'max_concurrency = 1' replaces in crates/athena-cli/tests/drain.rs (near lines 3162, 7706, 7839) act on a fixture, not the live value.
3. [content] CHANGELOG.md has an entry under [Unreleased] ### Changed (the first one, line ~64) saying the default [parallel] max_concurrency is now 2.
4. [content] git diff touches only athena.toml, CHANGELOG.md, crates/athena-cli/tests/common/mod.rs and, if needed, crates/athena-cli/tests/drain.rs; no production code, config key or schema changes.
5. [behavior] Run cargo test --locked -p athena-cli --test drain and cargo test --locked -p athena-data; both pass, including a_change_to_max_concurrency_alone_does_not_invalidate_a_reading and the unreadable-config tests.
6. [behavior] Run cargo test --locked and cargo clippy --locked --all-targets; the full suite passes and clippy is clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the parallel drain code or its maximum of 8
- Changing the disk check
- Adding config keys or changing the config schema
- defaults.toml and .github/workflows
- Adding a new lint that fails tests reading live config values

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Pin the fixture value instead of editing each test's literal.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Full test suite must pass.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Four files at most, no production code.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 734b3ea364ff → candidate 4354c3b4421a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          | 3 +++
 athena.toml                           | 6 +++---
 crates/athena-cli/tests/common/mod.rs | 6 ++++++
 3 files changed, 12 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-160232-t9yq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-160232-t9yq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42061572964 for 4354c3b4421adb75a39b1cd15cc62a22658745d4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42061572964 for 4354c3b4421adb75a39b1cd15cc62a22658745d4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42061572964 for 4354c3b4421adb75a39b1cd15cc62a22658745d4)
- ci-result: `not a check` passed (Check results for 4354c3b4421adb75a39b1cd15cc62a22658745d4 come from the CI run https://github.com/doodla/athena/actions/runs/42061572964 (511s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo test --locked and cargo clippy --locked --all-targets; the full suite passes and clippy is clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
