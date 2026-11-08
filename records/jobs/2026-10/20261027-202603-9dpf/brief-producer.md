# Job 20261027-202603-9dpf: Refuse to queue stale approved proposals; ask for fresh approval

## Request (verbatim)

> Before queuing a job from an approved proposal, check the outcomes the proposal cites. If an earlier job from the same source was closed as done, or a newer audience rating of the same target exists, do not queue it as is. Mark it stale, show the user why with references to the closing job and the newer rating, and ask for a fresh approval. Add a test with a proposal whose cited job was closed as done, and assert that no job is queued.
> 
> Acceptance:
> - The change is made as described: An approved audience proposal from an old rating ran after its items had landed; the job and its retry both failed
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/jobs/2026-10/20261027-074457-bx8e/request.json

## Reading (accepted by the user)

Retry of job 20261027-074457-bx8e (rejected: t3 wording uncited, weak site test). That job's site-page work is dropped; the real defect was that approve_work queued a proposal whose cited job was closed Done or whose rating had a newer one. The change adds a staleness check before approved work is queued, with a test.

Assumptions:
- Code survey: read crates/athena-core/src/work_decisions.rs (approve_work calls find_open, then approve_request, then ask; approve_in_limit wraps it), crates/athena-cli/src/drain.rs (approve calls approve_in_limit), crates/athena-data/src/records/work.rs (WorkItem source/evidence are Reference{Job|Path|Commit}; AudienceRating has target and at) and crates/athena-core/src/store.rs (Records.audience_ratings). No new record or data file under crates/ is added, so no AnyRecord or check_record_file change is needed.
- Closed as done means a cited job whose state is closed with an outcome reason that, after stripping 'closed: ', starts with 'Done'; this matches job 20261018-000747-w4v7. A newer rating means another audience rating with the same target and a later time than the rating the proposal's source or evidence path names.
- Marking stale is done by refusing at approval with an explanatory error; no new decision record schema is added, and the proposal stays open so a person can reject or re-raise it. The audience is the person approving work and the drain.
- Retry of 20261027-074457-bx8e: its verifier findings (t3 wording, text-only decider test, changelog claims) concern work that was already landed in 9884df5 and bca96fc and is dropped, not fixed; this refusal rule is the root cause it exposed.
- The change adds a refusal rule; expected 5% of newly refused readings (only approvals of proposals already carried out or superseded).

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core work_decisions`: a new test builds a proposal whose evidence cites a job closed as done and asserts that approve_work/the staleness check queues no job (no job, no approval record) and returns an error.
2. [behavior] A second new test in work_decisions.rs (run with cargo test -p athena-core work_decisions) has a proposal whose source rating has a newer rating of the same target and asserts it is refused; an unrelated fresh proposal is still approved as before.
3. [content] The refusal message, in crates/athena-core/src/work_decisions.rs, names the closing job id and the newer rating id, says the proposal is stale, and asks for a fresh approval; a test asserts those ids appear in the message.
4. [content] The check runs in work_decisions.rs before `self.ask` is called in approve_work, so approve_in_limit and the drain path (crates/athena-cli/src/drain.rs approve) are covered; diff shows no other behavior changes.
5. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass, including existing tests.
6. [content] Job 20261027-074457-bx8e's candidate (CHANGELOG.md, studies model.json, xtask/src/site.rs) is dropped: git diff shows no change to model.json, site.rs or claims.json; only CHANGELOG.md gains an entry for this change.

## Expected deliverables

- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- Restructuring the front page around Athena
- studies/firstmate-work-handling/model.json, claims.json and xtask/src/site.rs (the rejected job's candidate edits are dropped)
- A new work-decision record schema or a stale state
- New findings about firstmate
- Re-doing items already landed in 9884df5 and bca96fc

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy and fmt pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert that no job is queued and the message cites both records.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check sits in the shared approve path, not in one caller.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in work_decisions.rs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check a stale proposal is not queued.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 10 cases in 10 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then required to re-measure the file at the end and report the fi...

33 more qualifying classes are left out.

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

- path: <state>/jobs/20261027-202603-9dpf/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-202603-9dpf/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-core work_decisions`: a new test builds a proposal whose evidence cites a job closed as done and asserts that approve_work/the staleness check queues no job (no job, no approval record) and returns an error.
A second new test in work_decisions.rs (run with cargo test -p athena-core work_decisions) has a proposal whose source rating has a newer rating of the same target and asserts it is refused; an unrelated fresh proposal is still approved as before.
The refusal message, in crates/athena-core/src/work_decisions.rs, names the closing job id and the newer rating id, says the proposal is stale, and asks for a fresh approval; a test asserts those ids appear in the message.
The check runs in work_decisions.rs before `self.ask` is called in approve_work, so approve_in_limit and the drain path (crates/athena-cli/src/drain.rs approve) are covered; diff shows no other behavior changes.
`cargo fmt --check`, `cargo clippy` and `cargo test` pass, including existing tests.
Job 20261027-074457-bx8e's candidate (CHANGELOG.md, studies model.json, xtask/src/site.rs) is dropped: git diff shows no change to model.json, site.rs or claims.json; only CHANGELOG.md gains an entry for this change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
