# Job 20261107-165953-rhxx: Check new refusal rules against every cited job before verdict

## Request (verbatim)

> Add a check that runs before the final verdict on any job that adds or changes a reading-refusal rule. It runs the rule on the recorded request and reading of every job id the request cites. It uses the same inputs the real flow passes in, including attempt.request when the job is a retry. The job fails unless the rule refuses every cited job, or the job lists the ones it does not refuse and says why. Any count in the changelog or docs must match the result. Acceptance: (1) a test shows a rule that ignores attempt.request fails this check on the retry case of job 20261025-190606-eebk; (2) a rule that catches every cited job passes; (3) a mismatched changelog count fails. Commit c2c671f fixed only the delivery-verb rule, and job 20261030-060835-ryvc was rejected after it landed (commit 35f0202), so this has to be a general check.
> 
> Acceptance:
> - The change is made as described: Two jobs that added reading-refusal rules were rejected because each rule caught only some of the jobs it said it fixed. Both rules were written from the request's wording and never run on those jobs.
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-11/20261107-132816-3vzf.json

## Reading (accepted by the user)

Add a pre-verdict check: when a job adds or changes a reading-refusal rule, run the rule on the recorded request, reading and attempt.request of each cited job id, and fail unless all are refused or the job lists the exceptions with reasons and any changelog/doc count matches. This also addresses jobs 20261025-190606-eebk (retry case; its secrets.rs/checks.rs speed-up work dropped) and 20261030-060835-ryvc (its speed-timeline rule kept as-is; its retry-blind gap is what this check detects). Audience: Athena maintainers and the verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading/checks.rs (refusing_check at line 521 takes reading, request, attempt, referenced, repo_files; the gate runs step! lines in order), crates/athena-core/src/reading/replay.rs (replay_cases, replay, touches_reading_rules, replay_declaration_of), crates/athena-core/src/run/mod.rs (replay_stage before the final verdict), and crates/athena-core/tests/refusal_corpus.rs (Case struct, corpus_cases_get_the_expected_outcome). Entry point is the replay stage run before the verdict; the new check hooks there via touches_reading_rules. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured by calling refusing_check itself with the real inputs, not a stand-in.
- Existing code: crates/athena-core/src/reading/replay.rs and crates/athena-core/src/reading/checks.rs (refusing_check) are extended; the check reuses refusing_check and the same attempt/referenced inputs the real flow passes, rather than a second copy of the rules.
- Environment: none beyond reading recorded requests and readings of cited jobs through the existing Records API; no CI, push or PR. When a cited job record is absent (as in this reader's copy), the check reports it as unverifiable rather than passing (no-empty-success).
- The 'Files: records/introspections/2026-11/20261107-132816-3vzf.json' line names the source introspection; records never change, so it is not a deliverable.
- Cited job ids are taken from the request text of the job under check (job-id pattern YYYYMMDD-HHMMSS-xxxx), the same extraction used for referenced jobs.
- The exception list is a declaration in the job's reading (an out_of_scope or criterion line naming the job id with 'not refused because ...'), parsed like replay_declaration. Changelog/docs counts are matched by comparing a stated 'N of M cited jobs' phrase to the check result.
- Refusal-rate: this adds a job-level check, not a new refusal of past readings, so expected 0% of newly refused readings.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new test builds a rule that ignores attempt.request and shows the cited-jobs check fails on the retry case of job 20261025-190606-eebk (request 'try again on 20261025-183644-t67q', attempt.request from its earlier job), naming that job in the failure.
2. [behavior] Run `cargo test -p athena-core`: a new test shows a rule that refuses every cited job (including the retry case with attempt.request passed) passes the check.
3. [behavior] Run `cargo test -p athena-core`: a new test shows a CHANGELOG.md/docs count that differs from the check result (e.g. says 3 of 3 when 2 are refused) fails, and a matching count passes.
4. [behavior] Run `cargo test -p athena-core`: a new test shows a job that lists an unrefused cited job with a 'because' reason passes, and one listing it without a reason fails; a missing cited record is reported as unverifiable, not passed.
5. [behavior] Run `cargo test -p athena-core`: a test shows the check runs only for jobs where touches_reading_rules is true, is wired in the replay/pre-verdict stage in crates/athena-core/src/run/mod.rs, and calls refusing_check with the same arguments as the real flow (grep the diff for the call).
6. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: the corpus gains a case 'try again on <speed job>' with no timeline that the speed-timeline rule refuses, covering the 20261030-060835-ryvc gap; the check applied to the delivery-verb rule of c2c671f and to ryvc's rule is covered by a test, so the check is general and not specific to one rule.
7. [content] CHANGELOG.md has an entry for the check and docs/decisions.md has a new appended decision with a Provenance bullet citing jobs 20261025-190606-eebk and 20261030-060835-ryvc and commits c2c671f and 35f0202; no existing decision entry is edited.
8. [content] The changes of 20261025-190606-eebk to crates/athena-gate/src/secrets.rs and crates/athena-core/src/checks.rs are not present in the diff (dropped), and no file outside the deliverables changes.
9. [behavior] Athena's recorded `cargo fmt --check`, `cargo clippy` and `cargo test` checks pass on the final tree and existing tests still pass.

## Expected deliverables

- crates/athena-core/src/reading/replay.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Keeping the changes of job 20261025-190606-eebk to crates/athena-gate/src/secrets.rs, crates/athena-core/src/checks.rs, docs/decisions.md and CHANGELOG.md (speed-up work, dropped; changelog/decision text is rewritten fresh)
- Speeding up any test or check, nextest, sccache
- Changing the speed-timeline rule's cue list beyond the corpus case
- Changing athena.toml, defaults.toml or CI workflow files
- Editing records/ or existing entries of docs/decisions.md
- Changing planner, critic or other prompts

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check refusal of real cited jobs, not that a function exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria included.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced as a gate step in code with tests, not prose.
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

37 more qualifying classes are left out.

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

- path: <state>/jobs/20261107-165953-rhxx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-165953-rhxx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## How the verifier checks each criterion

The verifier sees only the diff and the commands it runs itself. Before you report, check each criterion yourself the way its line below says, and give that evidence in its criteria_check entry. This is guidance: nothing here refuses or holds your work.

Criterion 1: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 2: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 3: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 4: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 5: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 6: a command that ran: cite the exact command the verifier can run in a fresh snapshot, and what it printed.
Criterion 7: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 8: a named file or diff line: cite the path, and the diff line, that shows it.
Criterion 9: Athena's recorded checks show it (fmt, clippy, the full test suite): begin the evidence with "left to Athena's checks"; the verifier judges it from the recorded checks.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core`: a new test builds a rule that ignores attempt.request and shows the cited-jobs check fails on the retry case of job 20261025-190606-eebk (request 'try again on 20261025-183644-t67q', attempt.request from its earlier job), naming that job in the failure.
Run `cargo test -p athena-core`: a new test shows a rule that refuses every cited job (including the retry case with attempt.request passed) passes the check.
Run `cargo test -p athena-core`: a new test shows a CHANGELOG.md/docs count that differs from the check result (e.g. says 3 of 3 when 2 are refused) fails, and a matching count passes.
Run `cargo test -p athena-core`: a new test shows a job that lists an unrefused cited job with a 'because' reason passes, and one listing it without a reason fails; a missing cited record is reported as unverifiable, not passed.
Run `cargo test -p athena-core`: a test shows the check runs only for jobs where touches_reading_rules is true, is wired in the replay/pre-verdict stage in crates/athena-core/src/run/mod.rs, and calls refusing_check with the same arguments as the real flow (grep the diff for the call).
Run `cargo test -p athena-core --test refusal_corpus`: the corpus gains a case 'try again on <speed job>' with no timeline that the speed-timeline rule refuses, covering the 20261030-060835-ryvc gap; the check applied to the delivery-verb rule of c2c671f and to ryvc's rule is covered by a test, so the check is general and not specific to one rule.
CHANGELOG.md has an entry for the check and docs/decisions.md has a new appended decision with a Provenance bullet citing jobs 20261025-190606-eebk and 20261030-060835-ryvc and commits c2c671f and 35f0202; no existing decision entry is edited.
The changes of 20261025-190606-eebk to crates/athena-gate/src/secrets.rs and crates/athena-core/src/checks.rs are not present in the diff (dropped), and no file outside the deliverables changes.
Athena's recorded `cargo fmt --check`, `cargo clippy` and `cargo test` checks pass on the final tree and existing tests still pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
