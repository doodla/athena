# Job 20261109-055645-r796: Design removing the local landing branch from landing

## Request (verbatim)

> Priority: high
> 
> Throughput design: Landing and rebuild takes 1996.9 of the 2063.6 median minutes per landed job (THROUGHPUT.md, last 20
> 
> Athena's throughput review 20261109-055516-tzwd named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: Landing and rebuild takes 1996.9 of the 2063.6 median minutes per landed job (THROUGHPUT.md, last 20 landings); the other five stages together take about 37. The design notes show that almost all of this time is not real waiting. Every veri...
> Stage: landing and rebuild
> Proposed change: Remove the local landing branch from the pipeline. Today the path is: verifier, then a fast-forward onto a shared local branch, then a pull request cut from that stack. Instead, take each verified job straight from the verifier to its own p...
> Predicted saving: 5.0 minutes per landed job; the median was 2063.6 minutes per landed job when the review ran.
> Why: Sources: THROUGHPUT.md and the design notes; nothing was re-measured here. THROUGHPUT.md: the median is 2063.6 minutes per landed job, of which landing and rebuild is 1996.9. This stage has grown with each review: 440.1 minutes (continuous-...
> 
> A change that implements this design must cite throughput review 20261109-055516-tzwd in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for taking each verified job straight to its own pull request, removing the shared local landing branch. It cites throughput review 20261109-055516-tzwd, states the 5.0 min/job predicted saving, and says how the next review measures it.

Assumptions:
- Goal and audience: the maintainer and the next throughput review, who judge whether the design removes the landing-and-rebuild wait.
- Existing code: crates/athena-core/src/land.rs, crates/athena-core/src/pr.rs, docs/design/continuous-landing.md, docs/design/per-job-main-landing.md, docs/design/native-pr-landing.md, docs/design/squash-chain-landing.md, docs/design/per-job-landing-branches.md (all exist; the job creates only a new note).
- Environment: the note discusses pull requests but nothing is pushed or run; read-only repository docs and code, no network or CI reached.
- Design only: no code, config or workflow changes; the note is a new file docs/design/ named for the change, e.g. per-job-pr-landing.md.
- Numbers (1996.9 of 2063.6 minutes, 5.0 predicted saving) are taken from the request and THROUGHPUT.md; the note labels them as not re-measured.

## Acceptance criteria

1. [content] A new file under docs/design/ describes taking each verified job straight to its own pull request without the shared local landing branch, and explains how that removes the landing-and-rebuild wait (Grep for 'landing branch' and 'pull request' in the new file).
2. [content] The note states the predicted saving of 5.0 minutes per landed job against the 2063.6 median, and says how the next throughput review measures it (Grep for '5.0' and 'next throughput review').
3. [content] The note cites throughput review 20261109-055516-tzwd (Grep for that id in the new file).
4. [content] The design has an Existing code section that reuses or extends crates/athena-core/src/land.rs, crates/athena-core/src/pr.rs and docs/design/continuous-landing.md, saying what each contributes.
5. [content] The note lists alternatives considered (e.g. docs/design/per-job-main-landing.md, squash-chain-landing.md) with reasons for rejecting them, and cites THROUGHPUT.md for measured numbers or labels them as estimates.
6. [content] git diff shows only added files under docs/design/; no other paths change.

## Expected deliverables

- docs/design/per-job-pr-landing.md (create)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Implementing the change in crates/ code
- Changes to athena.toml, defaults.toml or workflows
- Re-measuring throughput

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note records alternatives and is a design doc to be reviewed before building.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cited to THROUGHPUT.md or labeled estimates.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria require the saving and its measurement by the next review.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 12 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must size the slice to reach the limit. The producer must also run...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

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

- path: <state>/jobs/20261109-055645-r796/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-055645-r796/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new file under docs/design/ describes taking each verified job straight to its own pull request without the shared local landing branch, and explains how that removes the landing-and-rebuild wait (Grep for 'landing branch' and 'pull request' in the new file).
The note states the predicted saving of 5.0 minutes per landed job against the 2063.6 median, and says how the next throughput review measures it (Grep for '5.0' and 'next throughput review').
The note cites throughput review 20261109-055516-tzwd (Grep for that id in the new file).
The design has an Existing code section that reuses or extends crates/athena-core/src/land.rs, crates/athena-core/src/pr.rs and docs/design/continuous-landing.md, saying what each contributes.
The note lists alternatives considered (e.g. docs/design/per-job-main-landing.md, squash-chain-landing.md) with reasons for rejecting them, and cites THROUGHPUT.md for measured numbers or labels them as estimates.
git diff shows only added files under docs/design/; no other paths change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
