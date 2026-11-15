# Job 20261114-045744-tj6b: Tell the verdict which check failures the job's paths could not cause

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Nothing separates a failure the job caused from one already on the base, or from a clash with a conc
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: failing-check-blocks-all-criteria-unjudged-no-baseline (20261017-075355-rwrh)
> Cause: Nothing separates a failure the job caused from one already on the base, or from a clash with a concurrent config and test change. The check stage has no baseline run and no mapping from tests to the change. A failing check also ends the jo...
> Seen 6 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261017-075336-m1y1 (job 20261017-044408-wcwy); prevention it proposed: Add a code check at admission and before handover. Admission would run the full test suite on the base commit and record which tests fail. Before handover the producer must run the same suite, and the system would compare the two. A test th...
> Introspection 20261017-184719-0s4h (job 20261017-182453-a5sf); prevention it proposed: Before the producer starts, run the repo's full check suite on the base commit and record which checks already fail. At verification, compare the failures after the job to that baseline. Failures that already existed are reported separately...
> Introspection 20261029-084637-aa8f (job 20261026-232434-6wfq); prevention it proposed: Run the same test command on the base commit when a job is admitted and record which tests fail. When a check fails after the job, compare the two runs and say whether each failing test is new or already failing. Store the names and output ...
> Introspection 20261108-031125-np1m (job 20261108-023653-70wy); prevention it proposed: Before dispatch, run the test suite on the unmodified base and record the failing tests as a baseline. At verification, judge only tests that fail now and passed in the baseline. Report a baseline failure as an environment or main-health pr...
> Introspection 20261108-034239-naz9 (job 20261108-023645-456v); prevention it proposed: Before dispatching a job, run the workspace checks on the base commit and record which tests already fail. Judge the "tests pass" criterion against that baseline: a failure present on the base counts as pre-existing and does not make the cr...
> Introspection 20261113-194925-8bkq (job 20261113-190327-f794); prevention it proposed: When a check fails on a job whose diff touches only docs, Athena should automatically run the same failing test on the base tree. It should record the result: failing on base means pre-existing, and the job is judged on its criteria with th...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the baseline only separates failures the base also had; a failure from a concurrent merge or an unrelated config/doc change reaches the verdict with no link between the failing test's files and the job's changed paths (jobs 20261017-044408-wcwy, 20261017-182453-a5sf, 20261026-232434-6wfq and the other three). The change gives the verdict inputs and the verifier's brief that link, as a finding, and adds no refusal rule.

Assumptions:
- Code survey: read crates/athena-core/src/verify.rs (is_pre_existing, failure_split, mark_pre_existing, blocking_checks at ~447) and crates/athena-core/src/run/checks.rs (take_baseline, baseline_for). A per-test baseline already exists; the gap is concurrent-change failures not in the baseline. No record or data file under crates/ is added, so check_record_file/AnyRecord need no change.
- Goal and audience: the verifier and the owner reading a rejected job's findings; they should see at once whether a red check could come from the job.
- Previous jobs (all rejected on a red check, none to be re-run): 20261017-044408-wcwy (auditor role; its failing drain test was caused by a concurrent athena.toml change; its work is dropped here), 20261017-182453-a5sf (ontology; failure from concurrent docs/design/job-planning.md; dropped), 20261026-232434-6wfq (R5 producer split; the failing test output was cut off; dropped). Their findings are addressed by the new finding, not by redoing their work.
- Environment: none; no CI, push or PR is used.
- The 'tests that fail now' output carries file:line paths in panic messages; paths are parsed from it. Failures with no paths yield no finding.

## Acceptance criteria

1. [content] The change description names the root cause: the check stage has a baseline of base-commit failures but no mapping from a failing test's reported file paths to the paths the job changed, so a failure from a concurrent change is indistinguishable from the job's own.
2. [content] crates/athena-core/src/verify.rs gains a function that extracts repository paths from a failing check's output and reports which of them the job changed and which it did not; the check's verdict reasons are unchanged (no new refusal rule; expected 0% of newly refused readings, because the finding is Minor and informational).
3. [behavior] A new unit test in crates/athena-core/src/verify.rs builds a failing check whose output cites a path the job did not change (e.g. docs/design/job-planning.md:6) and a baseline without it; the judgment still fails, and carries a finding naming the path as not touched by the job. It fails on the base tree and passes with the change. Run `cargo test --locked -p athena-core verify`.
4. [behavior] A second unit test shows that when the cited path is in the job's changed paths, the finding says the job touched it. Run `cargo test --locked -p athena-core verify`.
5. [content] The verifier brief (built in crates/athena-core/src/run/verification.rs) includes the failing check's untouched-path finding so criteria are still judged; a test in the same crate asserts the brief text contains it.
6. [content] CHANGELOG.md has an entry for the change; docs/decisions.md is untouched except possibly a new appended decision with a Provenance bullet.
7. [behavior] cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass (recorded Athena checks on the final tree).

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- The auditor role and every file job 20261017-044408-wcwy changed (roles.rs, records.rs, auditor.rs, fake.rs, display.rs, main.rs, tests/auditor.rs, docs/records.md, roles-and-ladder.md and the rest): dropped
- The ontology work of job 20261017-182453-a5sf (docs/ontology.toml, tests/ontology.rs, architecture.md): dropped
- The R5 producer split of job 20261026-232434-6wfq (run/producer.rs, run/mod.rs, structure-pass.md): dropped
- Any new rule that refuses or holds work; the verdict on a failing check is unchanged
- Editing athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: unit tests added and fmt, clippy, test criterion
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: root cause named in the change description
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: tests check the finding appears, not that wiring exists
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

- path: <state>/jobs/20261114-045744-tj6b/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045744-tj6b/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The change description names the root cause: the check stage has a baseline of base-commit failures but no mapping from a failing test's reported file paths to the paths the job changed, so a failure from a concurrent change is indistinguishable from the job's own.
crates/athena-core/src/verify.rs gains a function that extracts repository paths from a failing check's output and reports which of them the job changed and which it did not; the check's verdict reasons are unchanged (no new refusal rule; expected 0% of newly refused readings, because the finding is Minor and informational).
A new unit test in crates/athena-core/src/verify.rs builds a failing check whose output cites a path the job did not change (e.g. docs/design/job-planning.md:6) and a baseline without it; the judgment still fails, and carries a finding naming the path as not touched by the job. It fails on the base tree and passes with the change. Run `cargo test --locked -p athena-core verify`.
A second unit test shows that when the cited path is in the job's changed paths, the finding says the job touched it. Run `cargo test --locked -p athena-core verify`.
The verifier brief (built in crates/athena-core/src/run/verification.rs) includes the failing check's untouched-path finding so criteria are still judged; a test in the same crate asserts the brief text contains it.
CHANGELOG.md has an entry for the change; docs/decisions.md is untouched except possibly a new appended decision with a Provenance bullet.
cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked pass (recorded Athena checks on the final tree).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
