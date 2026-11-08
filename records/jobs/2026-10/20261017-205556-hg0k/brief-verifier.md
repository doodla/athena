# Job 20261017-205556-hg0k: verify "Add deterministic design-surface inventory, diff and record"

## Request (verbatim)

> Measure Athena's design surface, so we can tell whether the design is settling. Sri, 2026-10-17: "Let's also have Athena reconsider its own design every few jobs to see if we arrive at a stable design" and "I'm not sold the API surface should be considered settled". Before any review runs, Athena needs a deterministic inventory of its surface and a diff between two commits.
> Plan: a function in athena-core, computed by code with no model call, that lists at a given commit: the CLI commands and their flags (from the clap definitions in crates/athena-cli/src/main.rs), the athena.toml sections and keys the config schema accepts (crates/athena-data/src/config.rs), the record schemas and their versions (crates/athena-data/src/records.rs), the role definitions and versions (roles.rs), and the workspace crates. A second function diffs two inventories into added, removed and changed items per kind, plus totals. Record the result as a design-surface record so later reviews can compare against it.
> Acceptance:
> - cargo test: in a test repository with two commits where a command is added, a config section is removed and a record schema moves to version 2, the diff reports exactly those three changes and nothing else.
> - cargo test: the inventory of this repository's HEAD lists every top-level command that `athena --help` prints, and every [section] that the repository's athena.toml uses.
> - cargo test: the inventory is identical when computed twice at the same commit.
> - clippy with -D warnings and fmt --check are clean; CHANGELOG.md has an entry; docs/records.md describes the new record.
> Out of scope: the review role, proposing changes, and renaming anything.

## Reading (accepted by the user)

Add to athena-core a model-free function that inventories Athena's design surface at a commit (CLI commands/flags, athena.toml sections/keys, record schemas+versions, roles+versions, workspace crates) and a function diffing two inventories into added/removed/changed per kind with totals, plus a design-surface record type. Audience: Sri and later design reviews, who need a stable baseline to judge whether the design is settling.

Assumptions:
- Code survey: read crates/athena-data/src/records.rs (AnyRecord is a serde-tagged enum keyed by schema strings like athena.role/1 and athena.candidate/2; check_record_file at line ~1366 is the validation allow-list entry, used by crates/athena-gate/src/records.rs; a new record needs an AnyRecord variant, a SCHEMA const, a path, arms in the schema/validate matches near lines 1308 and 1354, and a round-trip test like a_code_health_baseline_has_a_path_a_schema_and_round_trips), crates/athena-data/src/roles.rs (ROLE_SCHEMA, steward(), audience() definitions), crates/athena-data/src/config.rs (config schema), crates/athena-cli/src/main.rs (clap definitions), and crates/athena-core/src/introspect.rs and code_health.rs as the nearest precedents for a computed-then-recorded measurement.
- Inventory is computed from file contents at a commit read through athena-git (not the working tree), with sorted, deterministic output; clap commands/flags are extracted from main.rs source text (or syn if a dependency is justified under buy-first), config keys from config.rs, schemas from the AnyRecord serde renames, roles from roles.rs, crates from workspace Cargo.toml.
- The new record is added as an AnyRecord variant so check_record_file accepts it and the round-trip test covers it; no new CLI command is added (that would itself change the surface); recording is via a function, not a new subcommand.
- Config: the function needs no new athena.toml section, but if the producer adds one to config.rs, athena.toml must set it so the schema test passes; athena.toml is therefore a low-probability deliverable.
- A new dependency is not expected; if one is added it is declared in crates/athena-core/Cargo.toml and Cargo.lock.
- Existing HEAD tests derive the expected command list from the clap Command (athena --help equivalent) and the section list from parsing athena.toml, not hard-coded lists.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a test builds a temporary git repository with two commits (a CLI command added, an athena.toml section removed, a record schema moving to version 2) and asserts the diff reports exactly those three changes (one added command, one removed config section, one changed record schema) and nothing else, with totals of 3.
2. [behavior] Run `cargo test`: a test computes the inventory at this repository's HEAD and asserts it contains every top-level subcommand of the clap Command that `athena --help` prints and every [section] header that the repository's athena.toml uses.
3. [behavior] Run `cargo test`: a test computes the inventory twice at the same commit and asserts the two results are equal and serialize to identical bytes.
4. [behavior] Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: both are clean; `cargo test` passes as a whole including the records round-trip test that covers the new design-surface AnyRecord variant and check_record_file accepting it.
5. [content] athena.toml is changed only if crates/athena-data/src/config.rs gains a new section or key; in that case athena.toml sets it, and the existing config schema test in `cargo test` passes. If config.rs is unchanged, athena.toml is unchanged.
6. [content] Any new dependency appears in crates/athena-core/Cargo.toml (and Cargo.lock) and `cargo build` passes; otherwise those files are unchanged.
7. [content] CHANGELOG.md has an entry describing the design-surface inventory and diff, and docs/records.md has a section describing the design-surface record: its schema string, fields, and how it is compared across commits.
8. [content] The diff has added, removed and changed lists per kind (commands, flags, config keys, record schemas, roles, crates) plus totals; Grep of the change shows no model/agent call in the inventory or diff code.

## Expected deliverables

- crates/athena-core/src/design_surface.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/paths.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- Cargo.toml (modify)

## Out of scope

- The design review role and scheduling reviews every few jobs
- Proposing design changes
- Renaming anything
- Adding a new CLI command
- Changing defaults.toml, .github/workflows, or any file under records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests; cargo test passes, clippy -D warnings and fmt --check clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the diff and inventory and check results, not existence of types.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the diff reports exactly the injected changes and the inventory is stable.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; no CLI command, no review role.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Determinism and completeness enforced by tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 01d40732aa5d → candidate 080eae96d9ad. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                     |   13 +
 crates/athena-cli/tests/design_surface.rs        |  110 +++
 crates/athena-core/src/design_surface.rs         | 1076 ++++++++++++++++++++++
 crates/athena-core/src/lib.rs                    |    4 +
 crates/athena-core/src/store.rs                  |    7 +
 crates/athena-data/src/records.rs                |   42 +
 crates/athena-data/src/records/design_surface.rs |   77 ++
 docs/records.md                                  |   31 +
 xtask/shard-timings.txt                          |    1 +
 9 files changed, 1361 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261017-205556-hg0k/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261017-205556-hg0k/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42455186657 for 080eae96d9ad65af38291d9b738005e7c25cb7b5)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42455186657 for 080eae96d9ad65af38291d9b738005e7c25cb7b5)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42455186657 for 080eae96d9ad65af38291d9b738005e7c25cb7b5)
- ci-result: `not a check` passed (Check results for 080eae96d9ad65af38291d9b738005e7c25cb7b5 come from the CI run https://github.com/doodla/athena/actions/runs/42455186657 (567s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: both are clean; `cargo test` passes as a whole including the records round-trip test that covers the new design-surface AnyRecord variant and check_record_file accepting it.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
