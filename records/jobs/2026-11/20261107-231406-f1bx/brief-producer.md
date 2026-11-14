# Job 20261107-231406-f1bx: Use earliest landing record for a job's landing and rebuild end

## Request (verbatim)

> Fix the 'landing and rebuild' stage in the throughput figures. Today `landing_record_time` in crates/athena-core/src/status.rs takes the NEWEST landings/ record that carries a job's landing commit. On a stacked branch, later landings carry earlier jobs' commits, so this pushes each job's landing end later every time a new job lands. Measured: the five landed jobs npqp, pg2d, yp1e, abbg and 4z98 have main-branch CI push runs 5 to 12 minutes after their landed outcomes. Yet review 20261107-224336-2v6c reports a landing median of 1,327.8 minutes. Change the code to use the earliest landing record that carries the job's commit, or the main merge time if one is recorded. Acceptance: (1) a test fixture where one job's commit is carried by two landing records shows the stage ending at the first one; (2) running `athena status` on the real records gives landing durations within 15 minutes of outcome-to-main-push for those five jobs; (3) the changelog states the old and new landing medians. Do not queue or build a landing redesign on the old figure until this lands.
> 
> Acceptance:
> - The change is made as described: Throughput reviews keep naming 'landing and rebuild' as the bottleneck (median 440 to 1,328 min), but GitHub shows recent jobs reached main within about 5 to 12 minutes of their landed records
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/status.rs, landings/

## Reading (accepted by the user)

landing_record_time (now in crates/athena-core/src/status/throughput.rs, not status.rs) takes the newest landings/ record carrying a job's job_landing commit; change it to the earliest, so stacked later landings stop pushing the end later. Audience: the owner reading throughput reviews. A landing's created_at is its merge-time record; no separate main-merge field exists, so none is added.

Assumptions:
- Code survey: read crates/athena-core/src/status/throughput.rs (landing_record_time at ~391 uses .max() over records.landings filtered on JobLanding commits with job id; landing_end at ~407 takes outcome.max(record time); used by busy_intervals, ranked_landings and the landing figures) and crates/athena-data/src/history.rs (Landing has created_at, host_commit, commits: Vec<LandingCommit>; no main-merge timestamp field). No new record or data file under crates/ is added, so check_record_file/AnyRecord/round-trip are unaffected. The property is measured directly as the landing_end minus outcome time per landed job.
- Field meaning: Landing.created_at is documented at crates/athena-data/src/history.rs:224 as 'When the landing was recorded', written after the merge; it is used as the main-merge time because no field records a separate merge time.
- Environment: the verifier reads only the repository snapshot with cargo and git log/show, no network; the independent range comes from commit times of the five jobs' landings on main in git log of the snapshot, and GitHub CI push-run times quoted in the request (5 to 12 minutes after landed outcomes). The records directory used for status is the real records snapshot available to the verifier; if absent, the verifier reports it.
- The request names status.rs but the function lives in status/throughput.rs; that file is changed. The 'landings/' path in the request is records data and is not edited (records are append-only).
- Old and new landing medians in the changelog are measured by running athena status before and after the change on the real records; the old figure is the review's 1,327.8 minutes unless reproduced differently.

## Acceptance criteria

1. [content] In crates/athena-core/src/status/throughput.rs, landing_record_time selects the earliest (.min()) created_at among landings/ records carrying the job's job_landing commit, and its doc comment says earliest.
2. [content] A new unit test in crates/athena-core/src/status/throughput.rs (or its tests module) builds a fixture where one job's commit is carried by two landing records and asserts landing_end equals the first record's time, not the second.
3. [behavior] Run `cargo test -p athena-core` : the new fixture test and all existing tests pass.
4. [behavior] Run `cargo run -p athena-cli -- status --snapshot <records directory>` on the real records: for jobs npqp, pg2d, yp1e, abbg and 4z98 the landing duration printed is within 15 minutes of the outcome-to-main-push gap, the expected range being 5 to 12 minutes after the landed outcome (up to 27 minutes tolerance) taken from main-branch commit times in git log, an independent source the change does not compute; the landing median is far below 1,327.8 minutes.
5. [content] CHANGELOG.md has an entry stating the old landing median (1,327.8 minutes as reported, or as measured before) and the new landing median measured after the change, and that landing figures use the earliest landing record.
6. [behavior] No unrelated behavior changes: `git diff` touches only throughput.rs and CHANGELOG.md; cargo fmt --check, cargo clippy and cargo test pass.

## Expected deliverables

- crates/athena-core/src/status/throughput.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Queuing or building a landing redesign
- Editing anything under records/ or landings/
- Adding a new main-merge timestamp field to landing records
- Changing other throughput stages

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check status output on real records against independent commit times.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture test added; fmt, clippy and test required.
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

- path: <state>/jobs/20261107-231406-f1bx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-231406-f1bx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In crates/athena-core/src/status/throughput.rs, landing_record_time selects the earliest (.min()) created_at among landings/ records carrying the job's job_landing commit, and its doc comment says earliest.
A new unit test in crates/athena-core/src/status/throughput.rs (or its tests module) builds a fixture where one job's commit is carried by two landing records and asserts landing_end equals the first record's time, not the second.
Run `cargo test -p athena-core` : the new fixture test and all existing tests pass.
Run `cargo run -p athena-cli -- status --snapshot <records directory>` on the real records: for jobs npqp, pg2d, yp1e, abbg and 4z98 the landing duration printed is within 15 minutes of the outcome-to-main-push gap, the expected range being 5 to 12 minutes after the landed outcome (up to 27 minutes tolerance) taken from main-branch commit times in git log, an independent source the change does not compute; the landing median is far below 1,327.8 minutes.
CHANGELOG.md has an entry stating the old landing median (1,327.8 minutes as reported, or as measured before) and the new landing median measured after the change, and that landing figures use the earliest landing record.
No unrelated behavior changes: `git diff` touches only throughput.rs and CHANGELOG.md; cargo fmt --check, cargo clippy and cargo test pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
