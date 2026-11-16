# Job 20261106-074601-f88a: Add regression test: config-only scope missing a dependent test is

## Request (verbatim)

> When a reading writes a criterion that limits which files a job may change, check that list against the repo first. Search for every file that reads the changed file, such as tests that load the real athena.toml. Add the files the defaults require, such as CHANGELOG.md. Then either widen the criterion to allow test-only edits to those files, or flag the conflict before the reading is accepted. Add a test where a config-only scope that leaves out a dependent test is caught.
> 
> Acceptance:
> - The change is made as described: Reading limits which files a job may edit to the files the request names, and leaves out tests and defaults that need to change too
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261018-045551-4h0h.json

## Reading (accepted by the user)

The mechanism (D166, `check_scope_limit`) already refuses limits that omit CHANGELOG.md or config consumers. This job adds a regression test replaying the failure class: a limit naming only athena.toml with a consumer test (e.g. crates/athena-data/tests/repo_config.rs) left out must be refused at reading acceptance. Audience: maintainers judging that the failure class is caught. athena.toml is not changed.

Assumptions:
- Code survey: read crates/athena-core/src/reading/checks.rs (check_scope_limit, refusing_check), crates/athena-core/src/reading.rs tests (scope_limit tables, gate_refuser, config_reading, limited, governed_reading ~lines 9140-9400), crates/athena-context/src/briefs.rs (change_limit), docs/decisions.md D166. Entry point is refusing_check; the check reads only criteria. No record or data file is added under crates/, so check_record_file/AnyRecord are not touched. The property is measured directly by gate_refuser on a constructed reading.
- Consumers: none found to change; athena.toml is not a deliverable. Tests that load the real athena.toml include crates/athena-data/tests/repo_config.rs and crates/athena-cli/tests/*.rs.
- The request asks for a mechanism that D166 already implements; the remaining gap is a test replaying the exact failure class (config-only scope omitting a dependent test), so no new rule is added.
- athena.toml is named in the request only as the example config; this job does not modify it or defaults.toml.
- The records/failure-classes file named in the request is evidence only and is never changed.

## Acceptance criteria

1. [content] crates/athena-core/src/reading.rs gains a new unit test beside the existing scope_limit tests: a reading with athena.toml as a deliverable, a limiting criterion naming only athena.toml and CHANGELOG.md, and a Consumers: assumption naming crates/athena-data/tests/repo_config.rs (a real athena.toml loader, not a deliverable) is asserted refused by check_scope_limit via gate_refuser.
2. [content] The same test asserts the repaired reading is accepted: the consumer test is added to the deliverables and the limit widened to allow test-only edits to it (or Consumers: states it is unchanged), so the test fails if the check is removed.
3. [content] The test also asserts the refusal error text names athena.toml and Consumers:, checked in the diff of crates/athena-core/src/reading.rs.
4. [behavior] Run `cargo test` and confirm the new scope-limit test passes and all existing tests pass, including refusal_corpus.
5. [behavior] Run `cargo clippy` and confirm it is clean.
6. [behavior] Run `cargo fmt --check` and confirm it passes.
7. [content] Only the test is added: the diff to check_scope_limit, change_limit and other non-test code is empty, so no reading is newly refused (expected 0% of newly refused readings).
8. [content] CHANGELOG.md has a new entry recording the change; the job's diff touches only the deliverables.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)

## Out of scope

- athena.toml and defaults.toml are not changed
- records/ including the failure-class record
- Changing check_scope_limit logic or the reader prompt (no token headroom)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test; cargo test, clippy and fmt criteria.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One test plus changelog entry.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts the refusal happens through the real gate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

36 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-074601-f88a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-074601-f88a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## How the verifier checks each criterion

The verifier sees only the diff and the commands it runs itself. Before you report, check each criterion yourself the way its line below says, and give that evidence in its criteria_check entry. This is guidance: nothing here refuses or holds your work.

Criterion 1: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 2: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 3: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 4: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 5: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 6: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 7: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 8: a named file or diff line: cite the path, and the diff line, that shows it.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading.rs gains a new unit test beside the existing scope_limit tests: a reading with athena.toml as a deliverable, a limiting criterion naming only athena.toml and CHANGELOG.md, and a Consumers: assumption naming crates/athena-data/tests/repo_config.rs (a real athena.toml loader, not a deliverable) is asserted refused by check_scope_limit via gate_refuser.
The same test asserts the repaired reading is accepted: the consumer test is added to the deliverables and the limit widened to allow test-only edits to it (or Consumers: states it is unchanged), so the test fails if the check is removed.
The test also asserts the refusal error text names athena.toml and Consumers:, checked in the diff of crates/athena-core/src/reading.rs.
Run `cargo test` and confirm the new scope-limit test passes and all existing tests pass, including refusal_corpus.
Run `cargo clippy` and confirm it is clean.
Run `cargo fmt --check` and confirm it passes.
Only the test is added: the diff to check_scope_limit, change_limit and other non-test code is empty, so no reading is newly refused (expected 0% of newly refused readings).
CHANGELOG.md has a new entry recording the change; the job's diff touches only the deliverables.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
