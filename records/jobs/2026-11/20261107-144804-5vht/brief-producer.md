# Job 20261107-144804-5vht: Require a measured stage timeline in readings of speed or delay requests

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The request named a fix (move the call earlier) and a timing figure, but nothing required the readin
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: speed-change-requested-without-checking-measured-stage-timeline (20261025-110542-wa68)
> Cause: The request named a fix (move the call earlier) and a timing figure, but nothing required the reading or planner to check that figure against the real stage order and durations before planning around it. The class already exists for a speed...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261025-110857-eqhd (job 20261025-110525-8hzb); prevention it proposed: For any request that claims a delay or speed-up, the reading stage must first produce a timeline from the code and the recorded run timings: each stage in order, its measured duration, and where the claimed delay sits. The reading must stat...
> Introspection 20261025-183009-xw7y (job 20261025-034733-c453); prevention it proposed: Add a measure-first step for any request to speed up a pipeline or check. Before the reading is accepted, Athena runs or reads a timing breakdown (compile versus run, per test binary or stage) and puts it in the reading. The reading must th...
> Introspection 20261025-190659-jvd7 (job 20261025-190606-eebk); prevention it proposed: Add a required measurement step to any request or reading whose goal is speed. Before the reading fixes a method, it must record a per-stage or per-binary timeline from a real run. It must also try each candidate lever, such as a different ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: no reading rule makes a request that claims a delay or speed-up (jobs 20261025-110525-8hzb, 20261025-034733-c453, 20261025-190606-eebk) be checked against the real stage order and measured durations. I will add a refusal check, `check_timing_claim`, beside `check_code_survey` and `check_records_measure`. It refuses a speed or delay reading that has no `Timeline:` assumption naming crates/ paths, stage order and measured or estimated durations. The three earlier jobs are not retried; their candidates are dropped.

Assumptions:
- Goal and audience: Athena's reader, planner and the user who writes speed requests. A request that claims a delay or speed-up with a timing figure must have the figure checked against the real stage order before planning.
- Code survey: crates/athena-core/src/reading/checks.rs (the ordered `step!` list at about line 539, where the new check is wired in after check_records_measure), crates/athena-core/src/reading/surveys.rs (check_code_survey at 199 and check_records_measure at 538 are the models for the lexical trigger and the error text), crates/athena-core/src/reading.rs (in-file tests for check order at about 8234 and for check_code_survey at 7240), crates/athena-core/tests/refusal_corpus.rs (the refusal corpus). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- Existing code: crates/athena-core/src/reading/surveys.rs, crates/athena-core/src/reading/checks.rs and the helpers changes_code_with_behavior_criterion, has_phrase and straight_lowercase are reused; the new check extends them and does not duplicate them.
- Term: `timeline` means the `Timeline:` assumption listing each stage in order, its duration (measured from the code or recorded run timings, or labeled as an estimate), and the stage where the claimed delay sits.
- The trigger is lexical on the request and reading text: words like speed up, faster, delay, earlier, minutes or seconds with a stage name. A reading that names no such claim is unaffected. The check refuses more readings and the higher refusal rate is intended: expected 4% of newly refused readings. This is an estimate, not measured: 3 of about 1000 scored jobs recorded this failure class, plus other speed requests.
- Environment: none. The check works on the reading and repository snapshot only, with no CI, push, PR or records reads. The verifier replays past readings with the existing `replay`, which reads local records.
- Prompt files are not changed, because the reader prompt has 2 tokens left; the rule is enforced by mechanism in code, and the refusal message tells the reader what to add.
- Consumers: none found for config; no athena.toml or defaults.toml change is needed.

## Acceptance criteria

1. [content] Root cause named in CHANGELOG.md and a new appended decision in docs/decisions.md with a Provenance bullet: nothing required a reading of a delay or speed-up request to check the claimed figure against the real stage order and durations. It cites jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk. No existing decisions.md entry is edited.
2. [content] crates/athena-core/src/reading/surveys.rs defines `pub fn check_timing_claim`, and crates/athena-core/src/reading/checks.rs calls it with `step!(check_timing_claim(...))` in the check list. The error text names the missing `Timeline:` assumption and gives an example.
3. [behavior] Run `cargo test -p athena-core`: a new unit test shows a reading for the 8hzb-style request (move a call earlier to save about 12 minutes) with no `Timeline:` assumption is refused by check_timing_claim. It passes when a `Timeline:` assumption names crates/ paths, stage order and durations. The refusal test fails if the step! line is removed. A second test shows a reading with no speed or delay claim is not refused.
4. [behavior] Run `cargo test -p athena-core`: the existing check-order test and the refusal corpus test still pass, with one new corpus case for the refused speed-claim reading.
5. [content] The change declares `expected 4% of newly refused readings` in a criterion or the doc comment of check_timing_claim, with limits of the lexical trigger stated, as done for check_records_measure.
6. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass, as recorded by Athena's checks on the final tree.

## Expected deliverables

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Retrying the three earlier jobs' own changes: job 20261025-110525-8hzb (run.rs, gh.rs early pull request move), job 20261025-034733-c453 (nextest and test speed-up) and job 20261025-190606-eebk (test speed-up); their candidates are dropped. Job 20261025-190606-eebk's candidate changed CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-gate/src/secrets.rs and docs/decisions.md; checks.rs and secrets.rs changes are dropped, and CHANGELOG.md and docs/decisions.md get only this job's new entries.
- Editing prompt files under crates/*/prompts (token headroom is 2 to 12 tokens for reader and planner)
- Changing athena.toml, defaults.toml or CI workflows
- Actually speeding up any pipeline or test suite
- Editing existing decisions.md or records entries

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a reading-stage refusal rule where the unchecked figure enters, not a per-job workaround.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a refusal check in code, with tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests that fail without the step line.
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

- path: <state>/jobs/20261107-144804-5vht/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-144804-5vht/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Root cause named in CHANGELOG.md and a new appended decision in docs/decisions.md with a Provenance bullet: nothing required a reading of a delay or speed-up request to check the claimed figure against the real stage order and durations. It cites jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk. No existing decisions.md entry is edited.
crates/athena-core/src/reading/surveys.rs defines `pub fn check_timing_claim`, and crates/athena-core/src/reading/checks.rs calls it with `step!(check_timing_claim(...))` in the check list. The error text names the missing `Timeline:` assumption and gives an example.
Run `cargo test -p athena-core`: a new unit test shows a reading for the 8hzb-style request (move a call earlier to save about 12 minutes) with no `Timeline:` assumption is refused by check_timing_claim. It passes when a `Timeline:` assumption names crates/ paths, stage order and durations. The refusal test fails if the step! line is removed. A second test shows a reading with no speed or delay claim is not refused.
Run `cargo test -p athena-core`: the existing check-order test and the refusal corpus test still pass, with one new corpus case for the refused speed-claim reading.
The change declares `expected 4% of newly refused readings` in a criterion or the doc comment of check_timing_claim, with limits of the lexical trigger stated, as done for check_records_measure.
Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass, as recorded by Athena's checks on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
