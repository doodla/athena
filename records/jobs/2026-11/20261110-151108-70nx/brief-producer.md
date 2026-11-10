# Job 20261110-151108-70nx: Resolve every cited past job id to its recorded reading in the reader

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The reading cited past jobs by id but nothing resolved those ids to their recorded deliverables befo
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: new-size-rule-fixtures-invented-not-taken-from-recorded-failed-jobs (20261109-090758-mrgx)
> Cause: The reading cited past jobs by id but nothing resolved those ids to their recorded deliverables before the reading was accepted. The thresholds (3 groups, 30 deliverables) and the test fixture were then built from an assumed shape. Nothing ...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261109-090744-yxah (job 20261109-075120-7zqw); prevention it proposed: When a request or reading adds a refusal rule that cites past jobs, Athena should resolve each cited job id to its recorded reading and deliverables at reading time. The criteria should then require the rule's test to load those recorded sh...
> Introspection 20261110-124233-20rr (job 20261029-111121-fzx0); prevention it proposed: Add a code check that runs any new reading-refusal or matching rule over the recorded past requests and accepted readings, with the old and new builds, before the producer finishes. It reports newly refused readings that cover all bullets a...
> Introspection 20261110-144807-kchg (job 20261110-124307-bn3c); prevention it proposed: Make a replay over the recorded real requests a required step of any job that adds or changes a refusal or matching rule. The step runs the old and new builds over all recorded requests and writes a record with the measured newly-refused co...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader brief resolves job ids (to recorded deliverables, criteria and changed files) only for ids written in the request text. Ids cited only by the retry's introspection (earlier cases, cause and prevention text) are never resolved, so readings invent fixtures and thresholds from an assumed shape. The change resolves those ids too, so the reader sees their recorded shape; it adds no refusal rule. It also addresses retried jobs 20261109-075120-7zqw, 20261029-111121-fzx0 and 20261110-124307-bn3c by dropping their candidates.

Assumptions:
- Goal and audience: the reader stage of Athena, and whoever judges its readings, must be given the recorded deliverables of every past job a request or its retry cause cites, instead of inventing a shape. The fix is information given to the reader, not a refusal rule, so no refusal rate applies.
- Code survey: crates/athena-core/src/reading/topics.rs (referenced_jobs resolves only job_ids_in(request) and followed_up_ids(request), capped by MAX_REFERENCED_JOBS, via carry()); crates/athena-core/src/reading.rs read_once (builds ReaderInputs, passes attempt.introspection through earlier_attempt); crates/athena-context/src/briefs/reader.rs (EarlierAttempt already holds reading_deliverables, reading_criteria and changed_files; RetryCause.earlier_cases carry job ids; retry_cause_section prints only text for them; prior_result_section/referenced_jobs_section render the carried jobs). No record or data file is added under crates/, so check_record_file/AnyRecord are not affected.
- Previous job 20261109-075120-7zqw (size check, rejected): its candidate's check_job_size, invented ZAJ2_SHAPE fixture and wrong doc claims are dropped, not kept; the finding that fixtures were invented is exactly this root cause, now fixed by resolving cited ids to recorded readings. The size-rule finding (bound too coarse) is out of scope as a separate rule.
- Previous job 20261029-111121-fzx0 (acceptance-bullet coverage check, rejected) and 20261110-124307-bn3c (its retry, rejected): the coverage.rs module, the fake.rs and drain.rs edits and the checks.rs wiring are dropped. Their findings (word-stem scoring, code-fence parsing, false positives) concern a refusal rule not requested here, and no longer apply because that rule is not in this change. The fixtures they needed (recorded readings of cited jobs) are what this change supplies to the reader.
- Referenced job ids are carried through the existing carry() path, redacted by the existing Redactor and bounded by MAX_REFERENCED_JOBS; jobs cited only by the introspection are listed after those named in the request so the cap never drops a request-named job. They are shown to the reader; continued_jobs still decides which must be accounted for, so the accounting rule check_prior_result_and_scope is unchanged.
- Existing code: crates/athena-core/src/reading/topics.rs, crates/athena-context/src/briefs/reader.rs and crates/athena-core/src/reading.rs; the change extends referenced_jobs and ReaderInputs.referenced_jobs rather than adding a new resolver.
- Consumers: none found (no config deliverable).

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/reading/topics.rs builds Records with a job A cited only in a retry's introspection earlier_cases (job field) or in its cause/prevention text, not in the request text, and asserts referenced_jobs (or its replacement) returns A with its recorded reading_deliverables. Run `cargo test -p athena-core referenced` to check; the test fails on the base code, which ignores such ids, and passes with the change.
2. [behavior] A test in crates/athena-context/src/briefs.rs builds a reader brief for a retry whose introspection cites a past job only by id, and asserts the brief contains that job's recorded deliverables, criteria and changed files under a heading for cited jobs. Run `cargo test -p athena-context brief` to check; it fails without the change.
3. [behavior] A test shows the cause is generic: with two different cited jobs (different ids, different deliverable lists) both appear in the brief with their own recorded deliverables, ids from the request text come first, and the MAX_REFERENCED_JOBS cap never drops a request-named job. Run `cargo test -p athena-core -p athena-context`.
4. [content] The doc comment on referenced_jobs in crates/athena-core/src/reading/topics.rs names the root cause (ids cited by introspection were never resolved to recorded deliverables, so fixtures and thresholds were invented), and the CHANGELOG.md Unreleased entry states it and lists jobs 20261109-075120-7zqw, 20261029-111121-fzx0 and 20261110-124307-bn3c as the instances. Check with Grep on both files.
5. [content] The change adds no new refusal or matching rule: `git diff` shows no new step! in refusing_check in crates/athena-core/src/reading/checks.rs and no new check function, so no newly refused share applies. check_prior_result_and_scope and continued_jobs behave as before; their existing tests still pass.
6. [content] The 'Referenced jobs' and 'Previous result and change scope' brief sections for cited-only jobs print the recorded deliverables, with no line starting with `Job ` added inside carried_work, and the reader prompt files are unchanged (git diff shows no prompt file edit; the reader prompt set has 2 tokens of headroom).
7. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.

## Expected deliverables

- crates/athena-core/src/reading/topics.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any new refusal rule: the size check (check_job_size), the acceptance-bullet coverage check (coverage.rs) and a replay harness; the dropped candidates of jobs 20261109-075120-7zqw, 20261029-111121-fzx0 and 20261110-124307-bn3c are discarded, including crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/coverage.rs, crates/athena-agent/src/fake.rs and crates/athena-cli/tests/drain.rs.
- Changing reader prompts, athena.toml, defaults.toml or .github/workflows.
- Redoing the work of the earlier jobs (size bounds, bullet coverage); these stay separate requests.
- Auto-splitting jobs or changing the small-job default.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed where the ids are resolved (the reader inputs), not for one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail on the base and pass with the change.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, no refusal rule.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 15 cases in 15 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a reading-stage check for bug-style or "show X in the brief" requests. The reader must name the code path that currently produces or omits the behavior, and the reading is accepted only if it cites a function and line showing the gap. If the behavior already exists, the reading must say so and n...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

34 more qualifying classes are left out.

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

- path: <state>/jobs/20261110-151108-70nx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-151108-70nx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/reading/topics.rs builds Records with a job A cited only in a retry's introspection earlier_cases (job field) or in its cause/prevention text, not in the request text, and asserts referenced_jobs (or its replacement) returns A with its recorded reading_deliverables. Run `cargo test -p athena-core referenced` to check; the test fails on the base code, which ignores such ids, and passes with the change.
A test in crates/athena-context/src/briefs.rs builds a reader brief for a retry whose introspection cites a past job only by id, and asserts the brief contains that job's recorded deliverables, criteria and changed files under a heading for cited jobs. Run `cargo test -p athena-context brief` to check; it fails without the change.
A test shows the cause is generic: with two different cited jobs (different ids, different deliverable lists) both appear in the brief with their own recorded deliverables, ids from the request text come first, and the MAX_REFERENCED_JOBS cap never drops a request-named job. Run `cargo test -p athena-core -p athena-context`.
The doc comment on referenced_jobs in crates/athena-core/src/reading/topics.rs names the root cause (ids cited by introspection were never resolved to recorded deliverables, so fixtures and thresholds were invented), and the CHANGELOG.md Unreleased entry states it and lists jobs 20261109-075120-7zqw, 20261029-111121-fzx0 and 20261110-124307-bn3c as the instances. Check with Grep on both files.
The change adds no new refusal or matching rule: `git diff` shows no new step! in refusing_check in crates/athena-core/src/reading/checks.rs and no new check function, so no newly refused share applies. check_prior_result_and_scope and continued_jobs behave as before; their existing tests still pass.
The 'Referenced jobs' and 'Previous result and change scope' brief sections for cited-only jobs print the recorded deliverables, with no line starting with `Job ` added inside carried_work, and the reader prompt files are unchanged (git diff shows no prompt file edit; the reader prompt set has 2 tokens of headroom).
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
