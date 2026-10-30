# Job 20261029-142849-04d9: verify "Add [drain.pipeline] config table with validation and safe defaults"

## Request (verbatim)

> Priority: urgent
> Async drain build job A2 (docs/design/async-drain.md, decision D145). Add the [drain.pipeline] config table (design section 11 and 12, job 2): schema, validation and defaults that keep today's behavior (max_in_flight = 1), with tests. With max_in_flight = 1 drain behaves as today.
> Acceptance:
> - Tests show the described behavior through its real entry point, and with max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-data/src/config.rs, CHANGELOG.md

## Reading (accepted by the user)

Add a `drain` section with a `pipeline` table (max_in_flight, max_check_slots, read_ahead, reserve_disk_gb, max_conflict_returns) to athena-data's Config, with design section 11 defaults (1,1,1,10,2), validation, tests, and a CHANGELOG entry. Config-only: nothing reads the values yet, so drain behaves as today; audience is the later build jobs and reviewers of the async-drain design.

Assumptions:
- Code survey: read crates/athena-data/src/config.rs. Entry point is Config::parse (toml::from_str with deny_unknown_fields, then Config::validate, which returns ConfigError variants); reading_inputs resets non-reading sections (parallel, landing, job_plan, disk) before digesting. No probing/resolution order applies beyond parse then validate.
- Code survey: no record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved; the records allow-list is untouched.
- Design: docs/design/async-drain.md sections 11 and 12 (job 2). Defaults: max_in_flight=1, max_check_slots=1, read_ahead=1, reserve_disk_gb=10, max_conflict_returns=2. Absent [drain] must equal those defaults.
- Following the `disk`/`job_plan` pattern: the new `drain` field is Option, skipped when None in serialization and reset in reading_inputs, so existing config digests and readings stay valid.
- Validation bounds are not given by the design. I assume max_in_flight 1..=8 (matching MAX_CONCURRENCY_CEILING), max_check_slots >= 1 and <= max_in_flight-independent small ceiling, read_ahead >= 1, max_conflict_returns >= 1, reserve_disk_gb any non-negative u64; a ConfigError variant names the offending key.
- Config-only: no drain runtime code is changed and athena.toml is not edited (job 10 raises max_in_flight). Higher refusal rate: the change refuses only configs that newly contain a [drain.pipeline] table with out-of-range values, so expected 0% of newly refused past readings.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-data`: tests through Config::parse show an athena.toml with no [drain] section yields pipeline values 1,1,1,10,2, and an explicit [drain.pipeline] with those values parses equal to it.
2. [behavior] Run `cargo test -p athena-data`: tests through Config::parse show max_in_flight = 0 and above the ceiling, max_check_slots = 0, read_ahead = 0, max_conflict_returns = 0 and an unknown key under [drain.pipeline] are each rejected with an error naming the key.
3. [behavior] Run `cargo test -p athena-data`: a test shows Config::reading_inputs is identical with and without a non-default [drain.pipeline], and the repository's real athena.toml still parses with max_in_flight resolving to 1.
4. [behavior] Run `cargo test --workspace`: the existing drain and config tests pass with no edits to existing test bodies.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`: both succeed.
6. [content] CHANGELOG.md has a new entry describing the [drain.pipeline] table, its defaults, and that max_in_flight = 1 keeps today's behavior.
7. [content] Each new public item in config.rs has a doc comment stating its meaning and default, citing docs/design/async-drain.md section 11; the change expects 0% of newly refused readings because absent or default tables stay valid.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)

## Out of scope

- Reading or acting on the new values in the drain runtime (later build jobs)
- Editing athena.toml or raising max_in_flight to 3 (build job 10)
- Disk-guard, check-slot, read-ahead or conflict-return behavior
- Changes to defaults.toml or .github/workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust in athena-data.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; cargo test and clippy -D warnings required.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise Config::parse outcomes, not struct existence.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to config.rs plus the changelog.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments cite the design section for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check parsed defaults and refusals, not activity.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): Each new public item in config.rs has a doc comment stating its meaning and default, citing docs/design/async-drain.md section 11; the change expects 0% of newly refused readings because absent or default tables stay valid.

## The change

Base 1548ad457a65 → candidate 81c76a3e3ed4. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   6 ++
 crates/athena-data/src/config.rs | 150 +++++++++++++++++++++++++++++++++++++++
 docs/design/async-drain.md       |   5 ++
 3 files changed, 161 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1675 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --workspace`: the existing drain and config tests pass with no edits to existing test bodies.; Run `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`: both succeed.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
