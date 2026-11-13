# Job 20261113-073013-c3eb: Add critique ledger mechanism for OLY-B-S6-01 (retry c6s3)

## Request (verbatim)

> try again on 20261030-044539-c6s3

## Reading (accepted by the user)

Retry of rejected job 20261030-044539-c6s3: add docs/critique-ledger.toml with a strict validating parser in athena-core, a test, and a spec entry for OLY-B-S6-01. This time regenerate docs/design/strategy-view.md whenever the requirements-map row S6-1 changes, so the stale-view test passes.

Assumptions:
- Goal and audience: the verifier and spec-gap readers need OLY-B-S6-01 to stop being a gap, with a mechanism that a test proves.
- Previous job 20261030-044539-c6s3 is addressed by keeping its approach and fixing each finding: (1) blocker/minor test failure: strategy_view_table_matches_the_map failed because requirements-map.md row S6-1 changed without regenerating docs/design/strategy-view.md; fix is to run `athena strategy` (cargo run -p athena-cli -- strategy) and commit the regenerated view, which is now a deliverable; (2) major D43: cite only commands that ran, so run cargo test, clippy and fmt --check on the full workspace; (3) minor shard-timings.txt: xtask/shard-timings.txt is now an explicit deliverable, because the new test file must be registered there; (4) minor weak check_standard: kept as designed and disclosed, with the limit noted in CHANGELOG and the map, since the accepted reading allows it.
- Files from the earlier candidate: CHANGELOG.md, critique_ledger.rs, lib.rs, tests/critique_ledger.rs, docs/critique-ledger.toml, requirements-map.md and spec.toml are all kept as deliverables; xtask/shard-timings.txt is kept as a deliverable; docs/design/strategy-view.md is added.
- Code survey: read crates/athena-core/src/lib.rs (module registration) and docs/spec.toml lines ~3021 (OLY-B-S6-01 has mechanism "none", owner Athena). The new parser lives in crates/athena-core/src/critique_ledger.rs and is called from crates/athena-core/tests/critique_ledger.rs. The ledger is a docs/ TOML file, not a record file under crates/, so check_record_file and AnyRecord do not apply. The property is measured directly by parsing the real ledger and the real spec entry.
- The producer should check how the strategy test (crates/athena-cli strategy::tests::strategy_view_table_matches_the_map) derives counts, and regenerate rather than hand-edit.
- No new refusal rule applies to readings; the validator only governs the ledger file.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-S6-01 has a mechanism naming crates/athena-core/src/critique_ledger.rs and docs/critique-ledger.toml (not "none"), owner "Athena", and a measure ending `status: not shown` that names `cargo test -p athena-core --test critique_ledger`.
2. [content] docs/critique-ledger.toml has [[entry]] tables covering critique, improvement and open_question, each with id, kind, subject, status, text and source, and at least one entry with subject athena.
3. [content] crates/athena-core/src/lib.rs registers the critique_ledger module with a doc line; the parser rejects unknown keys, unknown kinds or statuses, blank text and duplicate ids.
4. [behavior] Run `cargo test -p athena-core --test critique_ledger`: it parses the real ledger and spec entry and passes; negative cases (duplicate id, bad kind, no athena entry, missing kind coverage) fail validation, so the test fails if the ledger or validator is removed.
5. [behavior] Run `cargo test -p athena-cli --lib strategy_view_table_matches_the_map`: it passes, because docs/design/strategy-view.md was regenerated with `athena strategy` and matches docs/design/requirements-map.md row S6-1.
6. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` for the workspace, each as its own command; each passes, or Athena's recorded same-tree pass (D115) is cited.
7. [content] xtask/shard-timings.txt lists the new critique_ledger test; CHANGELOG.md has an entry for this job; no existing entry of docs/decisions.md and nothing under records/ is edited.

## Expected deliverables

- docs/spec.toml (modify)
- docs/critique-ledger.toml (create)
- crates/athena-core/src/critique_ledger.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/critique_ledger.rs (create)
- CHANGELOG.md (modify)
- docs/design/requirements-map.md (modify)
- docs/design/strategy-view.md (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- Adding an `athena status` line for the ledger
- Changing athena.toml, defaults.toml or CI workflows
- Editing existing entries of docs/decisions.md or anything under records/
- Automatically generating ledger entries from reviews or runs
- Adding severity, impact or effort fields to the ledger

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Behavior criteria run the named tests and cite commands.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Workspace cargo test, clippy and fmt --check are criteria.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261030-044539-c6s3 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 5 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`
> blocker (docs/design/requirements-map.md (row S6-1); docs/design/strategy-view.md not regenerated): The workspace test suite fails: the requirements-map edit makes docs/design/strategy-view.md stale. `cargo test -p athena-cli --lib strategy_view_table_matches_the_map` fails with "docs/design/strategy-view.md is out of date; run `athena strategy`: G6 ... table shows [23, 0, 7, 16], the map gives [23, 0, 8, 15]". The CI run for this commit also fails on this test. The fix is to run `athena strategy` and commit the regenerated view, or to leave the map row unchanged.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261113-073013-c3eb/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-073013-c3eb/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml entry OLY-B-S6-01 has a mechanism naming crates/athena-core/src/critique_ledger.rs and docs/critique-ledger.toml (not "none"), owner "Athena", and a measure ending `status: not shown` that names `cargo test -p athena-core --test critique_ledger`.
docs/critique-ledger.toml has [[entry]] tables covering critique, improvement and open_question, each with id, kind, subject, status, text and source, and at least one entry with subject athena.
crates/athena-core/src/lib.rs registers the critique_ledger module with a doc line; the parser rejects unknown keys, unknown kinds or statuses, blank text and duplicate ids.
Run `cargo test -p athena-core --test critique_ledger`: it parses the real ledger and spec entry and passes; negative cases (duplicate id, bad kind, no athena entry, missing kind coverage) fail validation, so the test fails if the ledger or validator is removed.
Run `cargo test -p athena-cli --lib strategy_view_table_matches_the_map`: it passes, because docs/design/strategy-view.md was regenerated with `athena strategy` and matches docs/design/requirements-map.md row S6-1.
Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` for the workspace, each as its own command; each passes, or Athena's recorded same-tree pass (D115) is cited.
xtask/shard-timings.txt lists the new critique_ledger test; CHANGELOG.md has an entry for this job; no existing entry of docs/decisions.md and nothing under records/ is edited.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
