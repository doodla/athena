# Job 20261110-163344-3gaa: Print recorded deliverable counts in the reader brief for carried jobs

## Request (verbatim)

> try again on 20261110-151108-70nx

## Reading (accepted by the user)

Retry of job 20261110-151108-70nx. Its stated cause (ids cited only by introspection were never resolved) was contradicted by the records: 7zqw named its jobs in its own request, the base brief already showed their deliverables, and the reader still wrote 12 for a list of 11. The narrowed fix gives the reader the count the record holds, so it copies the number and does not count.

Assumptions:
- Job 20261110-151108-70nx is addressed by retraction, not by repair: finding 'root cause contradicted by records' (major) and 'reading does not match request' (major) are fixed by dropping the introspection-id resolution premise, which the records disprove (7zqw was not a retry; base referenced_jobs and prior_result_section already carry request-named ids and print deliverables). The finding about deliverables printed twice no longer applies: no new list is printed, only a count is added to existing lines. The retry.rs finding no longer applies: that file's work is dropped. The nit about compile-error fails-without no longer applies: no signature change.
- Root cause stated for this job: the reader is shown cited jobs' deliverables as a bare list and must count it, so it wrote 'zaj2 (12 deliverables)' where the record has 11 (evidence: the verification of earlier attempt 20261110-124254-zegf, as quoted in the 70nx findings). The information the stage lacked is the recorded count; give it in the brief instead of adding any refusal rule.
- Code survey: read crates/athena-context/src/briefs/reader.rs (prior_result_section at line 135, lines 160-189 print 'Job N: end state ...' then 'Allowed change scope, deliverables:' as a quoted list; referenced_jobs_section at line 436) and crates/athena-core/tests/refusal_corpus.rs plus the list of files mentioning replay (crates/athena-core/src/reading/replay.rs exists already). Entry point is the reader brief builder in briefs.rs; no records validation allow-list or data file under crates/ is added, so check_record_file and AnyRecord are untouched.
- Kept from the 70nx candidate: nothing is kept. Its seven changed files (CHANGELOG.md, crates/athena-cli/tests/retry.rs, crates/athena-context/src/briefs.rs, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/reading/topics.rs, crates/athena-core/tests/refusal_corpus.rs) are re-based; of these only CHANGELOG.md, briefs.rs and briefs/reader.rs are deliverables again, with new content. reading.rs, topics.rs, retry.rs and refusal_corpus.rs are excluded (dropped).
- The audience is the reader agent (which must quote counts correctly) and the verifier who checks a rendered brief.
- No new refusal rule is added, so no newly-refused share applies. The reader prompt set has 2 tokens of headroom, so no prompt file changes.
- Jobs 7zqw, fzx0 and bn3c are not claimed as instances of this gap; only the count-miscopy seen in 7zqw's reader is.

## Acceptance criteria

1. [behavior] A new test in crates/athena-context/src/briefs.rs builds a reader brief for a follow-up whose carried job records 11 deliverables and asserts the text contains the count 11 next to 'Allowed change scope, deliverables' for that job, and a second job with 3 deliverables shows 3. Run `cargo test -p athena-context brief`; it fails on base, where no count is printed.
2. [behavior] The count is computed from reading_deliverables.len() for every job in prior_result_section and in referenced_jobs_section, not typed per job: a test with differing lengths (0, 1, 11) shows each count, and 0 still prints the existing 'none recorded' line. Run `cargo test -p athena-context`.
3. [content] Grep of the diff shows only crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/briefs.rs and CHANGELOG.md changed; crates/athena-core/src/reading.rs, reading/topics.rs, reading/checks.rs, crates/athena-cli/tests/retry.rs, refusal_corpus.rs and all prompt files are unchanged, and no new step! or check function appears in refusing_check.
4. [content] CHANGELOG.md Unreleased has an entry that names the cause (reader had to count a bare deliverable list and miscounted, e.g. 12 for 11 in job 7zqw) and says the fix prints the recorded count; it does not list fzx0 or bn3c as instances. Check with Grep.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, and existing tests such as a_follow_up_brief_carries_the_result_and_scope_of_each_named_job still pass.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Resolving ids cited only by a retry's introspection (dropped from the 70nx candidate): crates/athena-core/src/reading/topics.rs, crates/athena-core/src/reading.rs, crates/athena-cli/tests/retry.rs, crates/athena-core/tests/refusal_corpus.rs are not changed.
- Any new refusal rule, replay harness, size check or bullet-coverage check; the dropped candidates of 7zqw, fzx0 and bn3c stay discarded.
- Reader prompts, athena.toml, defaults.toml, .github/workflows.
- Verifying root-cause claims before dispatch (the proposed next request from introspection jskc); a separate request.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the rendered brief shows the correct count, not that code exists.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the brief text.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion included.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261110-151108-70nx raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-core/src/reading/topics.rs:565-568; CHANGELOG.md Unreleased ### Fixed): The named root cause is contradicted by the recorded instances, so the change would not prevent the failure class it cites. Evidence snapshot, records/jobs/2026-11/20261109-075120-7zqw/request.json: 7zqw was not a retry, and its request text names jobs 20261025-183644-t67q and 20261017-205554-zaj2 directly. Base referenced_jobs already resolves ids in the request text through job_ids_in and carry(). Base prior_result_section chains referenced_jobs and prints 'Allowed change scope, deliverables' (base test a_follow_up_brief_carries_the_result_and_scope_of_each_named_job, briefs.rs:3896). The verification of the earlier attempt at this same fix (records/jobs/2026-11/20261110-124254-zegf/verification.json) found that 7zqw's reader saw those deliverables and still wrote 'zaj2 (12 deliverables)', where the record has 11. Introspections 20261110-124233-20rr (fzx0) and 20261110-144807-kchg (bn3c) say those jobs failed because a word-matching or parsing rule was never run over recorded requests; neither failure involves an unresolved job id. For bn3c, the new code would carry job 7zqw (via 20rr's earlier case yxah), a size-check job unrelated to bn3c's code-fence flaw. The originating gap is still open: nothing makes a rule's fixtures and thresholds come from the recorded shapes, and nothing runs the rule over recorded requests. Simpler and more correct: fix that gap, for example by requiring fixtures to be loaded from records, or by running a new rule over recorded requests before the verdict.
> major: The reading does not match the request, so passing criteria do not show the request was met The request asks for the cause to be found where it originates, starting from what information the failing stage lacked. The reading says the gap is that ids cited only by a retry's introspection were never resolved. The records contradict this. Job 7zqw was not a retry, and the jobs it cited were named in its own request text. The base code already resolves such ids and already shows their recorded deliverables to the reader, and the reader saw them and still miscounted. Jobs fzx0 and bn3c failed because rules were never run over recorded requests, not because of unresolved ids. So the reading fixes a real but different gap, and the CHANGELOG wrongly lists the three jobs as instances of it. This repeats the misdiagnosis for which the earlier attempt at this fix (job 20261110-124254-zegf) was rejected.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 16 cases in 16 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a check at reading acceptance for requests of the root-cause or bug kind. The reading must list the request's stated cause as a claim. Each claim must cite the code location that shows the cause (file and lines) or the recorded instances that show it. A step in the reader or critic stage checks ...
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

- path: <state>/jobs/20261110-163344-3gaa/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-163344-3gaa/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-context/src/briefs.rs builds a reader brief for a follow-up whose carried job records 11 deliverables and asserts the text contains the count 11 next to 'Allowed change scope, deliverables' for that job, and a second job with 3 deliverables shows 3. Run `cargo test -p athena-context brief`; it fails on base, where no count is printed.
The count is computed from reading_deliverables.len() for every job in prior_result_section and in referenced_jobs_section, not typed per job: a test with differing lengths (0, 1, 11) shows each count, and 0 still prints the existing 'none recorded' line. Run `cargo test -p athena-context`.
Grep of the diff shows only crates/athena-context/src/briefs/reader.rs, crates/athena-context/src/briefs.rs and CHANGELOG.md changed; crates/athena-core/src/reading.rs, reading/topics.rs, reading/checks.rs, crates/athena-cli/tests/retry.rs, refusal_corpus.rs and all prompt files are unchanged, and no new step! or check function appears in refusing_check.
CHANGELOG.md Unreleased has an entry that names the cause (reader had to count a bare deliverable list and miscounted, e.g. 12 for 11 in job 7zqw) and says the fix prints the recorded count; it does not list fzx0 or bn3c as instances. Check with Grep.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, and existing tests such as a_follow_up_brief_carries_the_result_and_scope_of_each_named_job still pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
