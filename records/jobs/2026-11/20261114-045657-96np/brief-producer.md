# Job 20261114-045657-96np: Run real-records status criteria before the verifier and record evidence

## Request (verbatim)

> Priority: high
> 
> Five status- or usage-measure jobs failed between 2026-11-11 and 2026-11-12, all with failure class 20261111-105030-a2bc. The reading gate (check_records_measure) already makes these readings include a criterion that runs the measure on the real records against an independent range. But nothing runs that criterion before the final verdict. Producers ship measures that were checked only on fixtures, or evidence that cites no command that ran, and the verifier rejects the whole job. After the producer finishes and before the verifier starts, have Athena itself run each behavior criterion that names real records (the status --snapshot command the reading gives) against the records snapshot. Record the command and its output as evidence that the producer and verifier can see. If the measure gives the same answer for every item, or falls outside the stated range, send it back to the producer. If the snapshot cannot be reached from the producer's workspace, end the job early with that reason rather than at the verdict. Add a test using the fake producer, and a CHANGELOG.md entry.
> 
> Introspections that record the repeated cause: 20261111-105017-5k6m, 20261111-122618-q1ha, 20261111-145341-v9p4, 20261111-212939-722z, 20261112-074421-cpkp, 20261112-180919-sydm, 20261112-190037-pkhv, 20261113-000027-1saa
> 
> Acceptance:
> - The change is made as described: New status-measure jobs keep failing at the final verdict because nothing runs the real-records criterion before the verifier does
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: CHANGELOG.md

## Reading (accepted by the user)

After the producer's report and before the verifier, Athena runs each behavior criterion that gives a `status --snapshot` command against the evidence snapshot and records the command and its output as evidence for the producer and verifier. A constant measure or an out-of-range value is sent back to the producer; an unreachable snapshot ends the job early with that reason. Audience: Athena operators and the verifier, who stop seeing these jobs fail at the final verdict.

Assumptions:
- Code survey: crates/athena-core/src/run/producer.rs (producer_loop: report, format_stage, change_list, loop_replay, loop_check_failures, then break to the verifier; send-backs via PRODUCER_SEND_BACKS and problems/send_back_text), crates/athena-core/src/run/mod.rs (the evidence snapshot is written at ~line 1153 via crate::snapshot::evidence_dir and substituted into the producer brief), crates/athena-core/src/snapshot.rs (`athena status --snapshot <dir>` reads it with no repository), crates/athena-core/src/reading/surveys.rs and reading.rs (the real-records criterion text), crates/athena-core/src/run/tests.rs and crates/athena-agent/src/fake.rs (fake producer). Run order: producer report, format, self-checks, then the new records-criteria run, then verifier. The measure is read directly from the command's printed output on the snapshot. No record or data file is added under crates/, so check_record_file and AnyRecord are untouched unless a new record kind is chosen, which is not planned; the output is kept as check evidence in the existing form.
- Existing code: crates/athena-core/src/snapshot.rs (evidence_dir, write, status over a snapshot dir), crates/athena-core/src/run/producer.rs (loop_check_failures, append_check_failures, send-back text) and run/checks.rs (check evidence the verifier sees); the change extends these rather than adding a parallel path.
- Selection: a behavior criterion whose text contains `status --snapshot` with an evidence directory; Athena runs it from its own process on the evidence snapshot directory, not inside the producer sandbox.
- Same answer for every item means the measure's printed per-item values are all identical when there is more than one item; the stated range is parsed from the criterion text. If the range cannot be parsed, the output is recorded and shown but does not send back.
- Environment: reads the local evidence snapshot directory Athena writes per job (read-only copy of records/, landings/ and main first-parent history); no network, push, PR or CI. `Snapshot unreachable from the producer's workspace` means the path is missing or not readable by the producer's grant; then the job ends early with that reason via self.failure.
- Send-backs reuse the existing PRODUCER_SEND_BACKS budget; after it is spent the failure reaches the verifier as a flagged gap, like check failures.
- Field meaning: no existing field, CLI argument or config key is reused; new behavior uses existing status --snapshot CLI as documented in crates/athena-core/src/lib.rs:28.

## Acceptance criteria

1. [behavior] A new fake-producer test in crates/athena-core/src/run/tests.rs: a reading with a `status --snapshot` behavior criterion whose measure is constant across items is sent back to the producer before the verifier runs, and the recorded evidence holds the command and its output. Run `cargo test -p athena-core` and name the test.
2. [behavior] A fake-producer test shows a measure inside the stated range passes: the command and output are recorded as evidence in the verifier's brief and the verifier then runs. Run `cargo test -p athena-core`.
3. [behavior] A fake-producer test shows a measure outside the stated range is sent back, and an unreachable snapshot ends the job early with a reason that names the snapshot, before any verifier invocation. Run `cargo test -p athena-core`.
4. [content] crates/athena-core/src/run/producer.rs (or a new run module it calls) runs the criterion after loop_check_failures and before `break Some(report)`, in Athena's own process, and the diff shows the command and output added to the evidence the producer's send-back and the verifier's brief carry.
5. [content] CHANGELOG.md has a new entry describing the pre-verifier real-records run, its send-back conditions and the early end when the snapshot is unreachable.
6. [content] If a new appended decision is added to docs/decisions.md it has a Provenance bullet; no existing entry or anything under records/ is edited.
7. [behavior] No unrelated behavior changes: criteria that name no real records run nothing new, and the existing tests pass. Run `cargo test`.
8. [behavior] Run `cargo fmt --check`.
9. [behavior] Run `cargo clippy --all-targets`; no warnings.

## Expected deliverables

- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/records_criteria.rs (create)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the check_records_measure reading gate or its wording in reading/surveys.rs
- Editing existing entries in docs/decisions.md or anything under records/
- Changing athena.toml, defaults.toml or workflows
- Running criteria that do not name real records (fixture-only criteria)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion checks that a status-measure job is stopped or sent back before the verifier, not just that code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-producer test plus fmt, clippy and test criteria.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG.md entry and doc comments written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Time budget

Each producer run, the first and any send-back, is stopped after 86400 seconds of wall-clock time. A stopped run sends no report, so what is on disk when it stops must already be a usable change.

Pace the work by its order. Do the smallest change that meets every acceptance criterion first, one file at a time, and leave extras for afterwards; report as soon as the criteria are met. Leave no half-edited file and no broken build between steps. Use targeted tests while you work (`cargo test -p <crate> <filter>`). The full suite runs at most once, and Athena runs it again on your result (see "Checks before the verifier" below), so a second full run only spends this limit.

You have no clock of your own. If the commands Bash may run (below) include `date`, note the time when you start and read it between steps; without it, count steps against the order above.

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

- path: <state>/jobs/20261114-045657-96np/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045657-96np/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new fake-producer test in crates/athena-core/src/run/tests.rs: a reading with a `status --snapshot` behavior criterion whose measure is constant across items is sent back to the producer before the verifier runs, and the recorded evidence holds the command and its output. Run `cargo test -p athena-core` and name the test.
A fake-producer test shows a measure inside the stated range passes: the command and output are recorded as evidence in the verifier's brief and the verifier then runs. Run `cargo test -p athena-core`.
A fake-producer test shows a measure outside the stated range is sent back, and an unreachable snapshot ends the job early with a reason that names the snapshot, before any verifier invocation. Run `cargo test -p athena-core`.
crates/athena-core/src/run/producer.rs (or a new run module it calls) runs the criterion after loop_check_failures and before `break Some(report)`, in Athena's own process, and the diff shows the command and output added to the evidence the producer's send-back and the verifier's brief carry.
CHANGELOG.md has a new entry describing the pre-verifier real-records run, its send-back conditions and the early end when the snapshot is unreachable.
If a new appended decision is added to docs/decisions.md it has a Provenance bullet; no existing entry or anything under records/ is edited.
No unrelated behavior changes: criteria that name no real records run nothing new, and the existing tests pass. Run `cargo test`.
Run `cargo fmt --check`.
Run `cargo clippy --all-targets`; no warnings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
