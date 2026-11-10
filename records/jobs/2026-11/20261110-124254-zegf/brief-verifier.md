# Job 20261110-124254-zegf: verify "Show cited jobs' recorded deliverables in the reader brief"

## Request (verbatim)

> Priority: high
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
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261109-090744-yxah (job 20261109-075120-7zqw); prevention it proposed: When a request or reading adds a refusal rule that cites past jobs, Athena should resolve each cited job id to its recorded reading and deliverables at reading time. The criteria should then require the rule's test to load those recorded sh...
> Introspection 20261110-124233-20rr (job 20261029-111121-fzx0); prevention it proposed: Add a code check that runs any new reading-refusal or matching rule over the recorded past requests and accepted readings, with the old and new builds, before the producer finishes. It reports newly refused readings that cover all bullets a...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader brief's "Referenced jobs" section resolves a cited job id only to its request and findings, so readings (and the refusal rules and fixtures they propose) invent the cited jobs' shape. Fix at the source in crates/athena-context: print each referenced job's recorded reading summary, criteria and deliverables with a count. Job 20261109-075120-7zqw and job 20261029-111121-fzx0 are the earlier failures; their candidates are dropped and this change does not redo them.

Assumptions:
- Code survey: read crates/athena-context/src/briefs/reader.rs (referenced_jobs_section prints only request and findings; earlier_attempt_section already prints reading_summary, reading_criteria and reading_deliverables), crates/athena-core/src/reading/topics.rs (referenced_jobs resolves ids via records.jobs and carry()) and crates/athena-context/src/briefs.rs tests (reader_briefs_carry_referenced_jobs_only_when_set). No record or data file is added, so check_record_file and AnyRecord are untouched. Property is measured directly by rendering reader_brief.
- Goal and audience: the reader (and the verifier who reads its reading) must see what a cited job really changed, so thresholds and fixtures come from recorded shapes, not assumed ones.
- Prefer information over refusal: this change adds no refusal rule, so no expected-refusal share applies.
- Job 20261109-075120-7zqw: its candidate (check_job_size in checks.rs, ZAJ2_SHAPE fixture) is dropped. Its findings (invented zaj2 fixture, false 'twelve deliverables over four groups' claim) are addressed by showing the recorded deliverables to the reader.
- Job 20261029-111121-fzx0: its candidate (coverage.rs, fake.rs, drain.rs, reading.rs, checks.rs, refusal_corpus.rs, CHANGELOG.md) is dropped, not kept. Its refusal rule is not redone.
- The existing Previous-result section is left as is; referenced jobs not retried by this request are the gap.
- Governance files (athena.toml, defaults.toml, workflows) need no change.

## Acceptance criteria

1. [content] The root cause is named in the doc comment of referenced_jobs_section in crates/athena-context/src/briefs/reader.rs: cited job ids were resolved only to request and findings, not to recorded deliverables.
2. [behavior] Run `cargo test -p athena-context referenced`: a new test builds a reader brief with a referenced job that has reading_summary, reading_criteria and reading_deliverables set, and asserts the Referenced jobs section lists each deliverable and a deliverable count; the test fails on the old code and passes with the change.
3. [behavior] Run `cargo test -p athena-context referenced`: a test with referenced jobs having no recorded reading asserts the section says 'no reading recorded' for them, and the existing reader_briefs_carry_referenced_jobs_only_when_set assertions still hold.
4. [behavior] Run `cargo test -p athena-context`: a test shows a referenced job that is also the retried job or followed up is not printed twice with deliverables, and no new line starts with 'Job ' beyond the one per job.
5. [content] The brief tells the reader to take sizes, thresholds and test fixtures for rules citing those jobs from the listed recorded deliverables; the wording is in reader.rs and stays inside the reader prompt token budget (no prompt file edit).
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
7. [content] CHANGELOG.md has an Unreleased entry naming the cause and the fix.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/reading/topics.rs (modify)

## Out of scope

- Any new refusal rule or size check (check_job_size, request coverage) from jobs 20261109-075120-7zqw and 20261029-111121-fzx0; those candidates are dropped
- Changes to reader prompt files, athena.toml, defaults.toml and .github/workflows
- Replaying past readings against a rule
- Auto-splitting jobs
- Changing crates/athena-agent/src/fake.rs or crates/athena-cli/tests/drain.rs (touched only by the dropped candidate)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the brief shows recorded deliverables, not that code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix is in the brief's input, where the missing information originates.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4ffa62d5bba3 → candidate 8dec65268437. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   4 ++
 crates/athena-context/src/briefs.rs        | 112 +++++++++++++++++++++++++++++
 crates/athena-context/src/briefs/reader.rs |  35 ++++++++-
 3 files changed, 149 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-124254-zegf/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-124254-zegf/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42839251642 for 8dec6526843708b03806034ac0ca8d1685d23350)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42839251642 for 8dec6526843708b03806034ac0ca8d1685d23350)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42839251642 for 8dec6526843708b03806034ac0ca8d1685d23350)
- ci-result: `not a check` passed (Check results for 8dec6526843708b03806034ac0ca8d1685d23350 come from the CI run https://github.com/doodla/athena/actions/runs/42839251642 (604s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
