# Job 20261027-160815-k3wz: Limit previous-result check to the job a retry or follow-up continues

## Request (verbatim)

> Priority: urgent
> The previous-result check added in commit d510d97 refuses a reading whenever the request text names an earlier job id but the reading does not restate that job's result. It was meant for retries and follow-ups of a job, but it also fires on requests that cite earlier jobs only as evidence, such as every root-cause request the error review writes (they cite the introspections of past jobs). On 2026-10-27 it refused the reading of a root-cause request twice and stopped the drain; the reader is not given those jobs' results, so it cannot satisfy the check. Fix: apply the check only to the job a retry or follow-up continues (its previous_job link, or a request that says it follows up a named job), and give the reader those jobs' recorded results; requests that cite jobs as evidence are read as before.
> Acceptance:
> - A test reads a root-cause request that cites two past jobs' introspections as evidence and shows the reading is accepted without restating their results.
> - A test shows a retry whose reading omits the previous attempt's result is still refused, and one that includes it is accepted.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/reading.rs, CHANGELOG.md

## Reading (accepted by the user)

The previous-result check in reading.rs currently runs over every job id cited in the request text. It should run only for the retried job (previous_job) and for jobs a request explicitly says it follows up. Jobs cited as evidence, such as the introspections in error-review root-cause requests, are shown to the reader as before and are not checked. The audience is the drain operator, who needs the drain to stop refusing those readings.

Assumptions:
- Follow-up detection is a small, documented phrase match on the request text, such as 'follows up job <id>' or 'follow-up of <id>'. It names the continued job. Other cited ids are evidence.
- The retried job (the `attempt` argument of read_for, from previous_job) is always checked and its recorded result is given to the reader as now.
- A followed-up job is carried to the reader with its recorded result, as the existing referenced_jobs carrying does. Evidence jobs are still carried but are not passed to check_prior_result_and_scope.
- The scope check (deliverables and out_of_scope) keeps applying only when there is at least one continued job.
- Only reading.rs and CHANGELOG.md change. No prompt files change, because the reader prompt has 3 tokens of headroom.

## Acceptance criteria

1. [behavior] A new test in reading.rs reads a root-cause request that cites two past jobs' introspections as evidence, with no previous_job and no follow-up wording. The reading does not restate those jobs' results and is accepted. Run `cargo test -p athena-core reading`.
2. [behavior] A new test shows that a retry whose reading omits the previous attempt's job id and result is still refused with 'the previous result'. A second test shows that the same retry with a reading that names the previous job is accepted. Run `cargo test -p athena-core reading`.
3. [behavior] A test shows that a request saying it follows up a named job is checked for that job only. A reading that omits the followed-up job is refused. A different job cited in the same request as evidence is not required to be named.
4. [content] read_for passes check_prior_result_and_scope only the retried job and the explicitly followed-up jobs. Evidence-only referenced jobs are still given to the reader in the brief. The doc comments describe this distinction.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including the existing prior-result tests.
6. [content] CHANGELOG.md has an entry describing that the previous-result check applies only to the continued job and no longer refuses evidence citations.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the reader prompt files or the token budgets
- Changing how the error review writes its requests
- Changing the scope check (deliverables and out_of_scope) rules for genuine retries
- Changing the MAX_REFERENCED_JOBS limit or the redaction of carried jobs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests are added, and tests, clippy and fmt must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The tests exercise accepted and refused readings rather than checking that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is scoped to the job being continued, not special-cased for root-cause requests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in one source file plus the changelog.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that the drain-stopping reading is accepted.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier can run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

16 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in reading.rs reads a root-cause request that cites two past jobs' introspections as evidence, with no previous_job and no follow-up wording. The reading does not restate those jobs' results and is accepted. Run `cargo test -p athena-core reading`.
A new test shows that a retry whose reading omits the previous attempt's job id and result is still refused with 'the previous result'. A second test shows that the same retry with a reading that names the previous job is accepted. Run `cargo test -p athena-core reading`.
A test shows that a request saying it follows up a named job is checked for that job only. A reading that omits the followed-up job is refused. A different job cited in the same request as evidence is not required to be named.
read_for passes check_prior_result_and_scope only the retried job and the explicitly followed-up jobs. Evidence-only referenced jobs are still given to the reader in the brief. The doc comments describe this distinction.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including the existing prior-result tests.
CHANGELOG.md has an entry describing that the previous-result check applies only to the continued job and no longer refuses evidence citations.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
