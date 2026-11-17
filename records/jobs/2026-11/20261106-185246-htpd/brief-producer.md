# Job 20261106-185246-htpd: Add per-mode demonstration convention to defaults, with a reading test

## Request (verbatim)

> Add a reading and planning convention: when a job changes or records values in a flow that has more than one mode (for drain: serial drain(), max_concurrency = 1, and max_concurrency > 1), each behavior criterion needs one demonstration per shipped mode, with expected values stated. Record this convention in the defaults and add a test that a reading for such a request lists one demonstration per mode.
> 
> Acceptance:
> - The change is made as described: Recording added to drain was built and tested for the parallel path only; the default serial path recorded wrong values twice
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-11/20261106-022422-jkfa.json

## Reading (accepted by the user)

Add a standing default "every-mode" to defaults.toml: when a job changes or records values in a flow with several modes (drain: serial drain(), max_concurrency = 1, max_concurrency > 1), each behavior criterion needs one demonstration per shipped mode, with expected values stated. Teach the reader brief to apply it, and add a test that a reading for such a request lists one demonstration per mode. Audience: the reader, producer and verifier agents, and the user judging readings.

Assumptions:
- Code survey: I read defaults.toml and crates/athena-data/src/defaults.rs. The tests there parse REPO_DEFAULTS, and render() shows each default as '- id [applies_to]: first sentence'. I also read crates/athena-context/src/briefs.rs, whose reader-brief tests start around line 4557, and crates/athena-cli/tests/budgets.rs, where the per-set token budgets are checked. I did not read the drain code. The entry point is DefaultsModel::parse followed by render, which feeds the reader brief. No data or record file is added under crates/, so check_record_file and AnyRecord are not touched.
- Existing code: crates/athena-data/src/defaults.rs, crates/athena-context/src/briefs.rs and crates/athena-context/prompts/reader.md are extended, not created. The new default follows the existing [[default]] entries.
- Reading: the reader prompt set has 2 tokens of room (1998 of 2000), and the defaults are composed into it. A new default adds roughly 25 tokens, so the producer must keep the first sentence very short or raise the reader budget where budgets.rs and the athena-context budget code set it, then keep every_role_set_is_within_its_budget passing. I did not locate the budget constant.
- The 'test that a reading lists one demonstration per mode' is read as a deterministic Rust test, not a live model call. It checks that the reader brief and the rendered defaults carry the rule, with the drain example of three modes. It also checks a validator on a fixture reading: one with one demonstration per mode and expected values passes, and one covering only the parallel mode fails. The producer chooses where the validator lives. It could be a check beside check_criteria_evidence in athena-core, or a prose-only rule with a brief test. The default is a prose rule backed by a brief test plus a small fixture check.
- The default's applies_to is [code, tests, config, build]. Its source cites the introspection record records/introspections/2026-11/20261106-022422-jkfa.json: recording added to drain was built and tested for the parallel path only, and the default serial path recorded wrong values twice. That record is only read, never edited.
- Prose-only parts are marked as such, following the mechanism default. If the validator is built, it is the mechanism. Governance files: defaults.toml is a deliverable. athena.toml changes only if the budget lives there, and I found no reader budget in it. Consumers of defaults.toml: crates/athena-data/src/defaults.rs and crates/athena-context/src/briefs.rs.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry (id such as 'every-mode') whose statement says that when a job changes or records values in a flow with more than one mode (for drain: serial drain(), max_concurrency = 1, max_concurrency > 1), each behavior criterion needs one demonstration per shipped mode with expected values stated. Its source cites records/introspections/2026-11/20261106-022422-jkfa.json. Verify with git diff on defaults.toml.
2. [content] crates/athena-context/prompts/reader.md (or the defaults rendering) tells the reader to list one demonstration per shipped mode with expected values. The producer's choice of where the rule lives is checked by git diff.
3. [behavior] A new test in crates/athena-data/src/defaults.rs or crates/athena-context/src/briefs.rs fails without the change and passes with it. It shows that the default is present and rendered for a code or tests request, and that a reading for a drain-recording request lists one demonstration per mode (serial drain(), max_concurrency = 1, max_concurrency > 1) with expected values. The same check refuses a parallel-only fixture reading. Run cargo test.
4. [behavior] The change makes no unrelated behavior change: all existing tests pass, including the reader prompt budget test (every_role_set_is_within_its_budget in crates/athena-cli/tests/budgets.rs) and the defaults tests. Evidence: the recorded check for cargo test.
5. [behavior] cargo fmt --all --check passes. Evidence: the recorded fmt check.
6. [behavior] cargo clippy --all-targets --locked -- -D warnings is clean. Evidence: the recorded clippy check.
7. [content] CHANGELOG.md records the change. The outcome-check default applies, and the new convention is written so a fresh reader can follow it, with the drain example as the worked case.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Changing the drain code or the recording behavior itself.
- Editing or annotating anything under records/, including the cited introspection record.
- Changes to planner, critic, producer or verifier prompts.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The convention is written for a newcomer reader and carries a worked drain example.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test added; fmt, clippy and test criteria are named.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is backed by a brief test and a fixture check, with any prose-only part marked.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test must fail on a parallel-only reading, not just check that the text exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The root cause is readings that cover only one mode, so the convention is added to the reading step.
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

- path: <state>/jobs/20261106-185246-htpd/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-185246-htpd/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## How the verifier checks each criterion

The verifier sees only the diff and the commands it runs itself. Before you report, check each criterion yourself the way its line below says, and give that evidence in its criteria_check entry. This is guidance: nothing here refuses or holds your work.

Criterion 1: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 2: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 3: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 4: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 5: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 6: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.
Criterion 7: a named file or diff line: cite the path, and the diff line, that shows it.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

defaults.toml has a new [[default]] entry (id such as 'every-mode') whose statement says that when a job changes or records values in a flow with more than one mode (for drain: serial drain(), max_concurrency = 1, max_concurrency > 1), each behavior criterion needs one demonstration per shipped mode with expected values stated. Its source cites records/introspections/2026-11/20261106-022422-jkfa.json. Verify with git diff on defaults.toml.
crates/athena-context/prompts/reader.md (or the defaults rendering) tells the reader to list one demonstration per shipped mode with expected values. The producer's choice of where the rule lives is checked by git diff.
A new test in crates/athena-data/src/defaults.rs or crates/athena-context/src/briefs.rs fails without the change and passes with it. It shows that the default is present and rendered for a code or tests request, and that a reading for a drain-recording request lists one demonstration per mode (serial drain(), max_concurrency = 1, max_concurrency > 1) with expected values. The same check refuses a parallel-only fixture reading. Run cargo test.
The change makes no unrelated behavior change: all existing tests pass, including the reader prompt budget test (every_role_set_is_within_its_budget in crates/athena-cli/tests/budgets.rs) and the defaults tests. Evidence: the recorded check for cargo test.
cargo fmt --all --check passes. Evidence: the recorded fmt check.
cargo clippy --all-targets --locked -- -D warnings is clean. Evidence: the recorded clippy check.
CHANGELOG.md records the change. The outcome-check default applies, and the new convention is written so a fresh reader can follow it, with the drain example as the worked case.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
