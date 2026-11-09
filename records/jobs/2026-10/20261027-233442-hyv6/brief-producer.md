# Job 20261027-233442-hyv6: Fix reopened in firstmate study; queue a fix when source check fails

## Request (verbatim)

> Priority: high
> A failed source check of the explainer queues nothing, so the same contradiction is reported on every drain and never fixed. On 2026-10-27 drain twice reported: the firstmate-work-handling study's model marks state 'reopened' as terminal-success (t9 ready -> reopened), intro.json says Done and Reopened both end the task, and the glossary calls reopened 'the final state', but the code returns a reopened item to its queue (bin/fm-teardown.sh). Fix both: (1) correct the study's model, intro lead-in and glossary entry so reopened is a non-terminal state that returns the item to the queue, citing the source lines; (2) when drain's source check fails, queue one fix request through the shared steward request builder that cites each contradicted or omitted statement and its source, unless an open request already cites it.
> Acceptance:
> - cargo run -p xtask -- site builds the study and the source check of drain passes on the corrected study, with reopened shown as returning the item to the queue.
> - A drain test with a fake source check that reports a contradiction shows one fix request queued citing it, and none on a second run while it is open.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: studies/firstmate-work-handling/model.json, studies/firstmate-work-handling/intro.json, studies/firstmate-work-handling/glossary.json, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Correct the firstmate-work-handling study so 'reopened' is non-terminal and returns the item to its queue (model.json, and intro.json, which holds both the lead-in and the glossary entry), citing bin/fm-teardown.sh lines. Make drain queue one fix request via StewardRequest when the source check fails, skipping any contradiction an open request already cites.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (source check at ~2438-2464 stores check.detail in held_back and queues nothing; check_page is the step) and crates/athena-core/src/steward.rs (StewardRequest::new builder, also used by hestia.rs and code_health.rs). No record or data file is added under crates/.
- The request lists studies/firstmate-work-handling/glossary.json, but that file does not exist at the base commit; the glossary lives in intro.json, so the glossary fix is made there and no glossary.json is created.
- Existing code: extends crates/athena-cli/src/drain.rs and reuses crates/athena-core/src/steward.rs StewardRequest; duplicate detection checks open queued requests for the cited statement.
- Audience: the drain operator and study readers, who need the contradiction fixed once and not reported every drain.
- Governance files (athena.toml, defaults.toml, workflows) are not changed.

## Acceptance criteria

1. [content] studies/firstmate-work-handling/model.json no longer marks state 'reopened' terminal-success (t9 ready -> reopened) and shows reopened returning the item to the queue, citing bin/fm-teardown.sh lines; check with git diff.
2. [content] studies/firstmate-work-handling/intro.json lead-in no longer says Done and Reopened both end the task, and its glossary entry no longer calls reopened 'the final state'; it says reopened returns the item to its queue, with a source citation. Check with git diff and Grep for 'final state'.
3. [behavior] Run `cargo run -p xtask -- site`: it builds the study without error and the drain source check passes on the corrected study, showing reopened returning the item to the queue.
4. [behavior] A new drain.rs test with a fake source check reporting a contradiction shows exactly one fix request queued citing the contradicted statement and its source, built through StewardRequest, and none on a second run while it is open. Run `cargo test -p athena-cli`.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry covering the study correction and the drain fix-request queueing.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- studies/firstmate-work-handling/claims.json (modify)
- crates/athena-core/src/steward.rs (modify)

## Out of scope

- Other studies' source checks
- Changing the source check model call itself
- athena.toml, defaults.toml and workflow files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: drain test plus fmt, clippy and test suite criteria
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: site build and source check pass is a criterion
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 14 cases in 14 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Before a reading is accepted, run a step that, for each file or setting the request would delete or change, lists the gate checks, admission rules and tests that read it. If any of them refuse or depend on the change, the reading must either include the code change or split into a prerequisite code ...
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

- path: <state>/jobs/20261027-233442-hyv6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-233442-hyv6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

studies/firstmate-work-handling/model.json no longer marks state 'reopened' terminal-success (t9 ready -> reopened) and shows reopened returning the item to the queue, citing bin/fm-teardown.sh lines; check with git diff.
studies/firstmate-work-handling/intro.json lead-in no longer says Done and Reopened both end the task, and its glossary entry no longer calls reopened 'the final state'; it says reopened returns the item to its queue, with a source citation. Check with git diff and Grep for 'final state'.
Run `cargo run -p xtask -- site`: it builds the study without error and the drain source check passes on the corrected study, showing reopened returning the item to the queue.
A new drain.rs test with a fake source check reporting a contradiction shows exactly one fix request queued citing the contradicted statement and its source, built through StewardRequest, and none on a second run while it is open. Run `cargo test -p athena-cli`.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
CHANGELOG.md has an entry covering the study correction and the drain fix-request queueing.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
