# Job 20261030-212650-gh7t: Refuse real-records criteria unless the job depends on landed d103

## Request (verbatim)

> Before admission, have the reader flag or refuse any acceptance criterion that runs athena status, plan or similar commands against this repository's real records/, unless the job depends on the evidence-snapshot job 20261030-070002-d103 and that job has landed. Add a test built from the e5sg and sz3z request shapes.
> 
> Acceptance:
> - The change is made as described: Acceptance criteria tell the producer to run athena on the real records/ queue, which its working copy does not contain; this happened three times
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/

## Reading (accepted by the user)

Add a reading check that refuses an acceptance criterion telling the producer to run athena status, plan or similar against this repository's real records/, unless the request has a "Depends on 20261030-070002-d103" line. Admission holds a dependent job until d103 lands, so the dependency line is the landed condition. A test is built from the e5sg and sz3z request shapes. Job 20261030-070002-d103 is referenced, not retried: its candidate files are neither kept nor redone here.

Assumptions:
- Goal and audience: stop readers from admitting criteria the producer and verifier cannot meet (their working copy has no records/); this happened in jobs 20261029-233021-e5sg, sz3z and one more. Reading authors and the admission gate are the audience.
- Code survey: crates/athena-core/src/reading/checks.rs (refusing_check runs one step! per check, in order; check_reading_acceptance calls it with the reading and the request text), crates/athena-core/src/reading/surveys.rs (check_criteria_evidence is the model for a criterion-text check), crates/athena-core/src/reading/topics.rs (dependency_ids_in reads 'Depends on' lines and the retry link), crates/athena-core/tests/refusal_corpus.rs (it fails when a step! line has no corpus case where that check refuses). The change adds no record or data file under crates/, so check_record_file and AnyRecord are not touched.
- Existing code: the new check reuses dependency_ids_in in crates/athena-core/src/reading/topics.rs for the dependency test. It follows crates/athena-core/src/reading/surveys.rs check_criteria_evidence for its shape and is added as one step! line in crates/athena-core/src/reading/checks.rs.
- Landed condition: the reading check has no records access. Admission already holds an item whose 'Depends on' job has not landed (see the depends_on tests in crates/athena-core/src/reading.rs), so a Depends on 20261030-070002-d103 line in the request is enough for the check.
- Matching: a criterion is flagged when it names an athena command (status, plan, or similar subcommands such as retry, drain, queue) together with the real records/ queue or real records. Criteria that run a command on a fixture or a temp directory, or that say records/ is unchanged, are not flagged. The e5sg and sz3z shapes fix the exact phrases.
- Refusal rate: the change refuses more readings, and the higher refusal rate is intended. Expected 3% of newly refused readings, because few past criteria run athena on the real records. The corpus replay is the check.
- Field meaning: no existing field, argument, record key or config key is reused.
- Prompt headroom: the reader prompt has 2 tokens left, so a new rule line in crates/athena-context/src/briefs/reader.rs is added only if it replaces text or fits; the check's error message is the guidance. Risk: briefs.rs line 4055 lists check names and may need the new check name.
- Out of scope also covers the 'Files: records/' line of the request: records/ never changes, so it is not a deliverable. The d103 candidate files are not redone; d103 is a dependency, not a retry.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/surveys.rs defines a public check, called by name in one new step! line of refusing_check in crates/athena-core/src/reading/checks.rs. It takes the reading and the request, and refuses a criterion that runs athena status, plan or a similar command against the real records/ queue.
2. [content] The check passes when dependency_ids_in(request) contains 20261030-070002-d103, and the dependency is the only exemption. Its error names the criterion by number and text and says to depend on job 20261030-070002-d103 or to use a fixture instead.
3. [behavior] Run `cargo test -p athena-core`. A new unit test in crates/athena-core/src/reading.rs built from the e5sg shape ('run athena status on the real records') and the sz3z shape shows the criterion is refused without a Depends on line and accepted with a 'Depends on 20261030-070002-d103' line. It also shows criteria that run athena on a fixture or temp dir are not refused.
4. [behavior] Run `cargo test -p athena-core --test refusal_corpus`. A new corpus case in crates/athena-core/tests/refusal_corpus.rs has the new check as the refusing one. The corpus coverage test passes with the new step! line.
5. [behavior] Run `cargo test` for the workspace. All existing tests pass: no unrelated behavior changes. The newly refused share of the corpus is about 3%, the higher refusal rate is intended, and it stays at or below 8%.
6. [content] CHANGELOG.md has a new entry for the check. No file under records/, athena.toml, defaults.toml or .github/workflows/ changes.
7. [behavior] Run `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`. Both pass, or Athena's recorded check passes on the same final tree.

## Expected deliverables

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Building the evidence snapshot itself (job 20261030-070002-d103): its candidate files (run/mod.rs, status.rs, snapshot.rs, store.rs, briefs, main.rs, display.rs and the others) are neither kept nor redone here
- Any change under records/, which never changes
- Changing athena.toml, defaults.toml, workflow files or the throughput computation
- Refusing criteria that run athena against fixtures or temporary records
- Checking whether d103 has landed from inside the reading check; admission's existing Depends on hold covers that

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the refusal happens and the dependency exemption works, through tests and the refusal corpus.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and corpus tests; fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check sits in the shared reading gate, not in one job's reading.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a step! check in refusing_check, not by prose.
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

- path: <state>/jobs/20261030-212650-gh7t/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-212650-gh7t/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading/surveys.rs defines a public check, called by name in one new step! line of refusing_check in crates/athena-core/src/reading/checks.rs. It takes the reading and the request, and refuses a criterion that runs athena status, plan or a similar command against the real records/ queue.
The check passes when dependency_ids_in(request) contains 20261030-070002-d103, and the dependency is the only exemption. Its error names the criterion by number and text and says to depend on job 20261030-070002-d103 or to use a fixture instead.
Run `cargo test -p athena-core`. A new unit test in crates/athena-core/src/reading.rs built from the e5sg shape ('run athena status on the real records') and the sz3z shape shows the criterion is refused without a Depends on line and accepted with a 'Depends on 20261030-070002-d103' line. It also shows criteria that run athena on a fixture or temp dir are not refused.
Run `cargo test -p athena-core --test refusal_corpus`. A new corpus case in crates/athena-core/tests/refusal_corpus.rs has the new check as the refusing one. The corpus coverage test passes with the new step! line.
Run `cargo test` for the workspace. All existing tests pass: no unrelated behavior changes. The newly refused share of the corpus is about 3%, the higher refusal rate is intended, and it stays at or below 8%.
CHANGELOG.md has a new entry for the check. No file under records/, athena.toml, defaults.toml or .github/workflows/ changes.
Run `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings`. Both pass, or Athena's recorded check passes on the same final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
