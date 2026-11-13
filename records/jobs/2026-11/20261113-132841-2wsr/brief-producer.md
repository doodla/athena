# Job 20261113-132841-2wsr: Add behavior-test design check to reading critic rules (retry kjvt)

## Request (verbatim)

> try again on 20261113-112906-kjvt

## Reading (accepted by the user)

Retry of job 20261113-112906-kjvt, which was dispatched but never verified and recorded no findings, so nothing is kept as verified and the work is redone from base. Add one reading check in athena-core, wired after check_code_survey, that refuses a code or tests reading with a behavior criterion unless it states (a) entry point, gate order and stop gate, (b) reused existing function, (c) an accepted record kind or round-trip deliverable, (d) a direct check per property. Goal: stop the repeated reading failures in introspections 20261028-055651-dhh0 and 20261028-082240-77as; audience: the reader agent and maintainers judging refusals.

Assumptions:
- Code survey: read crates/athena-core/src/reading/checks.rs (the step! pipeline and check_code_survey order), crates/athena-core/src/reading/surveys.rs (changes_code_with_behavior_criterion, names_record_evidence), crates/athena-core/src/reading/replay.rs, crates/athena-core/src/verify.rs, crates/athena-core/tests/refusal_corpus.rs and crates/athena-cli/src/main.rs (athena check-reading). Entry point is check_reading_text; the gate order is its step! sequence, and the new check is tested to stop at its own step after check_code_survey. The records allow-list is AnyRecord and HistoryRecord in athena-data (check_record_file); a new typed record needs a schema round-trip test deliverable. The property is measured directly by counting refusals from check_reading_text on fixture readings, not via a skip switch.
- Job 20261113-112906-kjvt: it was never verified and recorded no findings, so there is nothing to fix finding by finding. Its 9 changed files (CHANGELOG.md, athena-cli main.rs, briefs.rs, briefs/reader.rs, reading.rs, reading/checks.rs, reading/surveys.rs, verify.rs, refusal_corpus.rs) are all kept as deliverables, and the work restarts from base. Its approach (labelled clauses of the Code survey assumption, Output: and Direct check: criterion N clauses) is reused as the design.
- Existing code: crates/athena-core/src/reading/surveys.rs, crates/athena-core/src/reading/checks.rs and crates/athena-core/src/reading/replay.rs; the new check extends changes_code_with_behavior_criterion and names_record_evidence and reuses the replay harness rather than duplicating them.
- The briefs.rs and briefs/reader.rs edits are Rust brief code, not prompt files; the reader prompt has only 2 tokens headroom, so prompt files are untouched.
- replay.rs and status.rs were allowed by the earlier scope but not changed by its candidate; they change only if the replay test or status listing needs it.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/surveys.rs defines a new pub check function (built on changes_code_with_behavior_criterion and names_record_evidence) whose refusal error names which of (a) gate order and stop gate, (b) reused existing function, (c) record kind or round-trip deliverable, (d) direct check is missing; crates/athena-core/src/reading/checks.rs wires it with one step! line directly after the check_code_survey step (check the diff).
2. [behavior] Fixture tests in crates/athena-core/tests/refusal_corpus.rs and unit tests in crates/athena-core/src/reading.rs show four otherwise complete readings, each omitting exactly one of (a)-(d), are refused with the missing part named, and one complete reading is accepted. Run `cargo test -p athena-core`.
3. [behavior] Fixture tests show contradicting readings are refused: a new record deliverable with no accepted record kind and no round-trip test deliverable, and a direct check that is a skip switch (skip_probe, ATHENA_NO_PROBE) or a read of an internal helper; a negated mention is accepted; a routine criterion saying only fmt, clippy and test pass needs no Direct check clause. Run `cargo test -p athena-core`.
4. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 30% of newly refused readings, because the check now asks for new required labels that past readings never used. A replay test over recent accepted readings (crates/athena-core/src/reading/replay.rs) shows the newly refused share within 30% plus 5 points; if above, markers are not loosened and the measured share and newly refused job ids are reported. Run `cargo test -p athena-core`.
5. [behavior] Every caller of check_reading_text is updated: `athena check-reading` still accepts its class 'feature' fixture reading, and a new CLI case shows a code reading with a short survey prints `refuse:` naming the new check. Run `cargo test -p athena-cli`.
6. [behavior] The prompt budgets test and the banned-word boundary test still pass and no file under crates/athena-context/prompts/ is edited. Run `cargo test -p athena-context`.
7. [content] CHANGELOG.md has an entry for the change including any stated limit of skip-switch matching; athena.toml, defaults.toml, .github/workflows/ and records/ are unchanged (check git diff).
8. [behavior] No unrelated behavior changes: cargo fmt --check, cargo clippy and cargo test pass on the final tree, as a recorded Athena check on the same tree.

## Expected deliverables

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/replay.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing records/ (including records/failure-classes/2026-10/20261028-055702-0t5w.json) or the cited introspections.
- Editing any prompt file under crates/athena-context/prompts/, including the critic prompt.
- Changes to athena.toml, defaults.toml or CI workflows.
- Adding a pub SCHEMA_TAGS constant to athena-data.
- Adding a planner repair round or other planning-pipeline changes.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check refusals and acceptances on fixtures and replay, not file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion plus fixture tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Each behavior criterion names a cargo test command.
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

- path: <state>/jobs/20261113-132841-2wsr/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-132841-2wsr/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading/surveys.rs defines a new pub check function (built on changes_code_with_behavior_criterion and names_record_evidence) whose refusal error names which of (a) gate order and stop gate, (b) reused existing function, (c) record kind or round-trip deliverable, (d) direct check is missing; crates/athena-core/src/reading/checks.rs wires it with one step! line directly after the check_code_survey step (check the diff).
Fixture tests in crates/athena-core/tests/refusal_corpus.rs and unit tests in crates/athena-core/src/reading.rs show four otherwise complete readings, each omitting exactly one of (a)-(d), are refused with the missing part named, and one complete reading is accepted. Run `cargo test -p athena-core`.
Fixture tests show contradicting readings are refused: a new record deliverable with no accepted record kind and no round-trip test deliverable, and a direct check that is a skip switch (skip_probe, ATHENA_NO_PROBE) or a read of an internal helper; a negated mention is accepted; a routine criterion saying only fmt, clippy and test pass needs no Direct check clause. Run `cargo test -p athena-core`.
The change refuses more readings and the higher refusal rate is intended: expected 30% of newly refused readings, because the check now asks for new required labels that past readings never used. A replay test over recent accepted readings (crates/athena-core/src/reading/replay.rs) shows the newly refused share within 30% plus 5 points; if above, markers are not loosened and the measured share and newly refused job ids are reported. Run `cargo test -p athena-core`.
Every caller of check_reading_text is updated: `athena check-reading` still accepts its class 'feature' fixture reading, and a new CLI case shows a code reading with a short survey prints `refuse:` naming the new check. Run `cargo test -p athena-cli`.
The prompt budgets test and the banned-word boundary test still pass and no file under crates/athena-context/prompts/ is edited. Run `cargo test -p athena-context`.
CHANGELOG.md has an entry for the change including any stated limit of skip-switch matching; athena.toml, defaults.toml, .github/workflows/ and records/ are unchanged (check git diff).
No unrelated behavior changes: cargo fmt --check, cargo clippy and cargo test pass on the final tree, as a recorded Athena check on the same tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
