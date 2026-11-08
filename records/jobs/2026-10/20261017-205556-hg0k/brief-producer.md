# Job 20261017-205556-hg0k: Add deterministic design-surface inventory, diff and record

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

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261017-205556-hg0k/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261017-205556-hg0k/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`: a test builds a temporary git repository with two commits (a CLI command added, an athena.toml section removed, a record schema moving to version 2) and asserts the diff reports exactly those three changes (one added command, one removed config section, one changed record schema) and nothing else, with totals of 3.
Run `cargo test`: a test computes the inventory at this repository's HEAD and asserts it contains every top-level subcommand of the clap Command that `athena --help` prints and every [section] header that the repository's athena.toml uses.
Run `cargo test`: a test computes the inventory twice at the same commit and asserts the two results are equal and serialize to identical bytes.
Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: both are clean; `cargo test` passes as a whole including the records round-trip test that covers the new design-surface AnyRecord variant and check_record_file accepting it.
athena.toml is changed only if crates/athena-data/src/config.rs gains a new section or key; in that case athena.toml sets it, and the existing config schema test in `cargo test` passes. If config.rs is unchanged, athena.toml is unchanged.
Any new dependency appears in crates/athena-core/Cargo.toml (and Cargo.lock) and `cargo build` passes; otherwise those files are unchanged.
CHANGELOG.md has an entry describing the design-surface inventory and diff, and docs/records.md has a section describing the design-surface record: its schema string, fields, and how it is compared across commits.
The diff has added, removed and changed lists per kind (commands, flags, config keys, record schemas, roles, crates) plus totals; Grep of the change shows no model/agent call in the inventory or diff code.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
