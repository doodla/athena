# Job 20261111-055238-5har: Add design note: land jobs directly on host main via merge_tree

## Request (verbatim)

> Priority: high
> 
> Throughput design: Landing and rebuild is 2984.3 of the 2990.6 median minutes per landed job (THROUGHPUT.md). The cause
> 
> Athena's throughput review 20261111-055051-tjym named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: Landing and rebuild is 2984.3 of the 2990.6 median minutes per landed job (THROUGHPUT.md). The cause is the one long-lived local landing branch. Every pull request is cut from it, so it carries all earlier unlanded jobs' commits again, and ...
> Stage: landing and rebuild
> Proposed change: Stop using the local branch as a staging area for landings. When a job's verifier passes, fetch host main and build the job's single landing commit directly on it: a three-way merge with Git::merge_tree (base = the job's recorded base, ours...
> Predicted saving: 5.0 minutes per landed job; the median was 2990.6 minutes per landed job when the review ran.
> Why: Measured in THROUGHPUT.md: landing and rebuild is 2984.3 of the 2990.6 median minutes per landed job; reading, planning, producer, checks and verifier add up to about 28.5. The stage has grown in every review: 440.1 (continuous-landing.md),...
> 
> A change that implements this design must cite throughput review 20261111-055051-tjym in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for the change from throughput review 20261111-055051-tjym: stop using the long-lived local landing branch as a staging area, and build each job's single landing commit directly on fetched host main with Git::merge_tree. The note covers the mechanism, alternatives, a predicted saving of 5.0 minutes per landed job, and how the next throughput review measures it. Audience: Athena maintainers who will implement and judge the change.

Assumptions:
- Goal: a design note only; the implementation is a later job. Audience: Athena maintainers and the next throughput review.
- Existing code: crates/athena-git/src/lib.rs (Git::merge_tree), crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, docs/design/direct-main-landing.md, docs/design/per-job-main-landing.md, docs/design/continuous-landing.md, docs/design/per-job-landing-branches.md.
- Environment: the design involves fetching host main and pushing or opening PRs on the git host; this job only writes docs, with no network or push.
- The request text is truncated; the producer should read THROUGHPUT.md and the cited earlier landing designs to fill details and not invent figures.
- The filename is chosen by the producer; the one in deliverables is my guess.
- Governance files are not changed; no config deliverable, so no Consumers line is needed.

## Acceptance criteria

1. [content] A new note under docs/design/ names throughput review 20261111-055051-tjym and cites THROUGHPUT.md figures (2984.3 of 2990.6 median minutes in landing and rebuild) as evidence.
2. [content] The note explains the mechanism: on verifier pass, fetch host main and build the job's single landing commit on it with Git::merge_tree (base = the job's recorded base). It says why this removes the carried-over unlanded commits that the single long-lived landing branch causes. A reader can check the cause-to-fix link in the text.
3. [content] The note has an 'Existing code:' section naming the existing paths it reuses or extends (crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, docs/design/direct-main-landing.md, docs/design/per-job-main-landing.md). It says how it differs from those earlier designs and why they did not remove the wait.
4. [content] The note lists alternatives considered and rejected, conflict handling (merge_tree conflict, host main moving during landing), and rollback or migration from the local branch.
5. [content] The note states the predicted saving of 5.0 minutes per landed job (from a median of 2990.6) and how the next throughput review measures it: the same THROUGHPUT.md landing-and-rebuild median, compared with the prediction, for jobs landed after the change. It also says a change implementing the design must cite review 20261111-055051-tjym in its request text.
6. [content] Only docs (and optionally CHANGELOG.md) change: git diff shows no change to crates/, athena.toml, defaults.toml or .github/workflows/, and cargo build and cargo test still pass unchanged.

## Expected deliverables

- docs/design/direct-main-merge-tree-landing.md (create)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Implementing the landing change in crates/
- Editing THROUGHPUT.md or records/
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The note cites THROUGHPUT.md numbers and labels the 5.0-minute saving as a prediction.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note includes alternatives and the existing-code reuse.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion checks the measurement method for the next review.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 17 cases in 17 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check before dispatch, and again before verification. (1) When a request or reading cites job ids as evidence of a failure, the check loads each cited record and confirms the stated facts (counts, deliverables, the failing path) against it. A mismatch refuses the reading and prints the re...
- 12 cases in 12 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required survey step to every design job before the note is drafted. The producer writes an Existing code inventory: for each proposed change, the file and function that does it today, or a statement that none exists. For each rule attributed to a doc, it gives the quoted line. For the scoring...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

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

- path: <state>/jobs/20261111-055238-5har/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-055238-5har/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new note under docs/design/ names throughput review 20261111-055051-tjym and cites THROUGHPUT.md figures (2984.3 of 2990.6 median minutes in landing and rebuild) as evidence.
The note explains the mechanism: on verifier pass, fetch host main and build the job's single landing commit on it with Git::merge_tree (base = the job's recorded base). It says why this removes the carried-over unlanded commits that the single long-lived landing branch causes. A reader can check the cause-to-fix link in the text.
The note has an 'Existing code:' section naming the existing paths it reuses or extends (crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, docs/design/direct-main-landing.md, docs/design/per-job-main-landing.md). It says how it differs from those earlier designs and why they did not remove the wait.
The note lists alternatives considered and rejected, conflict handling (merge_tree conflict, host main moving during landing), and rollback or migration from the local branch.
The note states the predicted saving of 5.0 minutes per landed job (from a median of 2990.6) and how the next throughput review measures it: the same THROUGHPUT.md landing-and-rebuild median, compared with the prediction, for jobs landed after the change. It also says a change implementing the design must cite review 20261111-055051-tjym in its request text.
Only docs (and optionally CHANGELOG.md) change: git diff shows no change to crates/, athena.toml, defaults.toml or .github/workflows/, and cargo build and cargo test still pass unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
