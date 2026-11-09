# Job 20261109-020722-xy57: Redo per-job main landing design note with premise check

## Request (verbatim)

> try again on 20261109-014709-kxrd

## Reading (accepted by the user)

Retry of job 20261109-014709-kxrd: write docs/design/per-job-main-landing.md for throughput review 20261109-014617-5cg0, first checking that the 1842.1-minute landing stage is mostly a measurement effect (repeated landing listings per agent-and-build-slots.md §6.1), and claiming only real savings. Audience: Athena maintainers and the next throughput review that scores the prediction.

Assumptions:
- Retry of job 20261109-014709-kxrd (rejected, verdict fail). Its candidate added docs/design/per-job-main-landing.md; that file is kept as the base for the rewrite (a deliverable, modified in place). Each of its 7 findings is addressed as a checklist item: (1) the 1842.1-minute stage is mostly repeated listings, not real waiting, and the saving is restated in a real quantity; (2) per-job-landing-branches.md gets a pointer to the new note so the citation duty is not conflicting; (3) the CHANGELOG/decisions inconsistency is resolved by making both deliverables with criteria; (4) the misread premise is fixed by citing §6.1.
- Existing code: docs/design/agent-and-build-slots.md, docs/design/per-job-landing-branches.md, docs/design/squash-chain-landing.md, docs/design/native-pr-landing.md, docs/design/continuous-landing.md, crates/athena-core/src/land.rs, crates/athena-core/src/status/throughput.rs
- Premise check: the reader did not open throughput.rs or §6.1 here; the producer must, per the verifier's cited facts: landing_end (throughput.rs ~lines 405-413) uses the later of the outcome and landing_record_time, which takes the newest landing record listing the job; job 20261104-054754-b4f5 appears in 60 of 215 landing files, first 4.3 min after verifier, newest about 6,900 min later (verifier's computation, to be re-measured). So the 1842.1 figure is largely a measurement effect.
- Goal: a note that proposes how to cut landing wait and claims only real savings, measured in the same quantity as the problem (minutes from verifier pass to first landing record). Real wait is labeled with source or estimate; the 1830.0 figure is not used unless records show it is real.
- Environment: none needed; the note describes a PR-based flow but the job only edits markdown in the repository and reaches no remote.
- Docs-only change; CHANGELOG.md and docs/decisions.md are changed deliberately, listed as deliverables and named in criteria, which resolves the prior contradiction with the scope-limit criterion.

## Acceptance criteria

1. [content] docs/design/per-job-main-landing.md exists (ls docs/design) and cites throughput review 20261109-014617-5cg0 and the figures 1842.1 and 1903.5.
2. [content] The design has an Existing code section that reuses or extends docs/design/per-job-landing-branches.md, docs/design/squash-chain-landing.md, docs/design/native-pr-landing.md and crates/athena-core/src/land.rs, saying what is kept and what changes, with file and function references describing how landing works today before the proposal.
3. [content] A premise section cites docs/design/agent-and-build-slots.md §6.1 and crates/athena-core/src/status/throughput.rs landing_end, states what the 1842.1 figure measures, and separates real elapsed wait per job from the measurement effect of repeated landing listings, each number sourced or labeled as an estimate (Grep for '6.1' and 'landing_end').
4. [content] The claimed saving is stated in the same real quantity (minutes from verifier pass to first landing record) and does not claim 1830.0 minutes as saved unless the note shows records that it is real wait; the problem-size, mechanism and ranking sections are consistent with the corrected size.
5. [content] The note lists alternatives, including measuring from the first landing record, with rejection reasons, plus failure cases (merge conflict, main moved, verified content differing), and says the next review reports both the unchanged landing stage and the first-landing stage and how it measures the saving.
6. [content] The note states an implementing change must cite review 20261109-014617-5cg0 in its request text.
7. [content] docs/design/per-job-landing-branches.md has a pointer to the new note that resolves which review an implementing change must cite (git diff).
8. [content] CHANGELOG.md has a new entry for the new note and docs/decisions.md has a recorded decision (git diff).
9. [content] Only docs/design/per-job-main-landing.md, docs/design/per-job-landing-branches.md, docs/decisions.md and CHANGELOG.md change (git diff --stat); no crates/, athena.toml, defaults.toml, workflows, THROUGHPUT.md or records/ change.
10. [behavior] Run cargo test: the policy tests that scan docs pass with the new note, and cargo fmt --check and cargo clippy are clean.

## Expected deliverables

- docs/design/per-job-main-landing.md (create)
- docs/design/per-job-landing-branches.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/
- Editing athena.toml, defaults.toml or workflows
- Editing THROUGHPUT.md or records/
- Changing how throughput.rs measures landing_end (only proposed in the note)

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every number in the note carries a source file or an estimate label.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Note written for an outside reader with explicit terms.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note includes existing code, alternatives and failure cases.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Addresses the measurement effect, not just the headline stage.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Saving is stated in the measured quantity and checked by the next review.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check note content via grep and diff.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261109-014709-kxrd raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (docs/design/per-job-main-landing.md sections 2, 3 ('Why this removes the wait') and 5): The note treats the 1842.1-minute stage as real waiting. The request's own cited evidence (agent-and-build-slots.md §6.1) and the records show it is mostly a measurement effect, so the 1830-minute 'saving' would be scored as real throughput. throughput.rs landing_end (lines 405-413) uses the later of the outcome and landing_record_time, and §6.1 says landing_record_time takes the newest of every landing record listing the job. In the evidence snapshot, the string '"job": "20261104-054754-b4f5"' appears in 60 of 215 landing files. The first is 20261104-062243-y5nm, 4.3 min after the verifier per §2.3; the newest is 20261109-014430-574k, about 6,900 min later (computed from file names). The note never cites §6.1, even though the request's 'Why' does. It explains the saving as stack-replay cost, conflicts and waiting for main, and its scoring rule would credit about 1830 minutes 'saved' that were mostly repeated listings of the same commit. The earlier note (per-job-landing-branches.md line 22) at least said the figure is 'not real waiting'. Simpler and more correct: say that the stage is inflated by commits being listed again in later pull requests. Add 'measure from the first landing record' as an alternative or a companion fix. Have the next review report both the unchanged stage and the first-landing stage, so the real wait is separated from the measurement effect.
> major: The reading does not match the request, so passing criteria do not show the request was met The request's "Why" cites agent-and-build-slots.md §6.1, measured on the records, as evidence. That section says the landing-and-rebuild figure is largely a measurement effect: landing_end takes the newest landing record that lists a job's commit, and stacked pull requests list each commit many times. The reading and criterion 3 frame the 1842.1 minutes as a real wait on a stacked branch. So the note never deals with the document the request cites, and its prediction scores the removal of repeated listings as minutes saved.

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

- path: <state>/jobs/20261109-020722-xy57/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-020722-xy57/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/design/per-job-main-landing.md exists (ls docs/design) and cites throughput review 20261109-014617-5cg0 and the figures 1842.1 and 1903.5.
The design has an Existing code section that reuses or extends docs/design/per-job-landing-branches.md, docs/design/squash-chain-landing.md, docs/design/native-pr-landing.md and crates/athena-core/src/land.rs, saying what is kept and what changes, with file and function references describing how landing works today before the proposal.
A premise section cites docs/design/agent-and-build-slots.md §6.1 and crates/athena-core/src/status/throughput.rs landing_end, states what the 1842.1 figure measures, and separates real elapsed wait per job from the measurement effect of repeated landing listings, each number sourced or labeled as an estimate (Grep for '6.1' and 'landing_end').
The claimed saving is stated in the same real quantity (minutes from verifier pass to first landing record) and does not claim 1830.0 minutes as saved unless the note shows records that it is real wait; the problem-size, mechanism and ranking sections are consistent with the corrected size.
The note lists alternatives, including measuring from the first landing record, with rejection reasons, plus failure cases (merge conflict, main moved, verified content differing), and says the next review reports both the unchanged landing stage and the first-landing stage and how it measures the saving.
The note states an implementing change must cite review 20261109-014617-5cg0 in its request text.
docs/design/per-job-landing-branches.md has a pointer to the new note that resolves which review an implementing change must cite (git diff).
CHANGELOG.md has a new entry for the new note and docs/decisions.md has a recorded decision (git diff).
Only docs/design/per-job-main-landing.md, docs/design/per-job-landing-branches.md, docs/decisions.md and CHANGELOG.md change (git diff --stat); no crates/, athena.toml, defaults.toml, workflows, THROUGHPUT.md or records/ change.
Run cargo test: the policy tests that scan docs pass with the new note, and cargo fmt --check and cargo clippy are clean.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
