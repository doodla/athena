# Job 20261105-163159-vsfy: Find and remove the cause of the longer verifier stage

## Request (verbatim)

> Priority: high
> 
> Throughput slowdown: verifier grew from 3.1 to 3.5 min per landed job
> 
> Athena's throughput review found that the median minutes per landed job rose from 616.3 to 1032.3, more than 25%, since the previous throughput review. Find why the stage that grew most now takes longer and remove the cause.
> 
> Stage that grew most: verifier, a median of 3.5 minutes per landed job now, 3.1 before.
> Stages whose median minutes grew, in minutes per landed job before to now: verifier 3.1 to 3.5.
> 
> The medians are over the newest landed jobs that Athena measured. The status section of athena status prints the same stage medians.
> 
> Acceptance:
> - The cause of the longer verifier stage is found and stated in the change or in a note under docs/design/
> - The change shortens that stage, and the next throughput review shows the median minutes per landed job falling
> 
> Files: crates/, docs/design/

## Reading (accepted by the user)

The verifier stage median rose from 3.1 to 3.5 min per landed job. The producer finds the cause in the verifier path, fixes it so the stage gets shorter, and states the cause in a docs/design note. The audience is the maintainer reading the next throughput review.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs. It defines the stage medians (the verifier column is role_minutes over Role::Verifier, around lines 651-931), and `athena status` prints them. The producer must also survey the verifier run path in crates/athena-core/src/run/ and the checks the verifier runs, to find where time is spent. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- The verifier time is measured directly by role_minutes on the verifier invocations, not through a stand-in.
- Cause is not yet known. The producer must measure it from the code (for example repeated or cold builds, redundant checks, a larger prompt or diff context for the verifier) and not guess.
- The change makes no new refusal rule, so no refused-reading percentage applies.
- Out of scope: rewording the other stages, and changing records/ or the throughput review thresholds.

## Acceptance criteria

1. [content] A note under docs/design/ (or the commit message) states the cause of the longer verifier stage and cites the code path or measurement that shows it, and the verifier can read it in the diff.
2. [content] The diff removes the stated cause in crates/ code in the verifier path, for example by removing redundant work, and does not only change how the median is reported.
3. [behavior] Run `cargo test` from the repository root: it passes, including a new or updated test that exercises the changed verifier behavior, and it fails if the cause is reintroduced.
4. [behavior] Run `cargo clippy` and `cargo fmt --check`: both are clean.
5. [content] The change is a single focused fix with no unrelated edits, and the note says how the next throughput review should show verifier minutes per landed job falling below 3.5.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/design/verifier-stage-slowdown.md (create)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changes to the other stages' timing
- Changes to records/ or to the throughput review thresholds

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause must be found and removed, not hidden.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check the fix and a test, not just that files exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The goal is that verifier minutes fall.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design note is written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The note cites measured sources.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-163159-vsfy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-163159-vsfy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A note under docs/design/ (or the commit message) states the cause of the longer verifier stage and cites the code path or measurement that shows it, and the verifier can read it in the diff.
The diff removes the stated cause in crates/ code in the verifier path, for example by removing redundant work, and does not only change how the median is reported.
Run `cargo test` from the repository root: it passes, including a new or updated test that exercises the changed verifier behavior, and it fails if the cause is reintroduced.
Run `cargo clippy` and `cargo fmt --check`: both are clean.
The change is a single focused fix with no unrelated edits, and the note says how the next throughput review should show verifier minutes per landed job falling below 3.5.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
