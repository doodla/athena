# Job 20261105-070552-kx1y: Design continuous landing inside the drain (review 20261105-070420-vq95)

## Request (verbatim)

> Priority: high
> 
> Throughput design: Landing and rebuild takes a median of 440.1 of the 464.5 minutes per landed job; every other stage t
> 
> Athena's throughput review 20261105-070420-vq95 named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: Landing and rebuild takes a median of 440.1 of the 464.5 minutes per landed job; every other stage together takes about 21. The measured span runs from the verifier's end to the landing record, which is written only after the merge has been...
> Stage: landing and rebuild
> Proposed change: Make landing a continuous step inside the drain instead of a separate, later one. As soon as the verifier passes, the job goes onto the one ordered landing queue (async-drain.md sections 6 and 7). There its pull request is merged automatica...
> Predicted saving: 420.0 minutes per landed job; the median was 464.5 minutes per landed job when the review ran.
> Why: Measured (THROUGHPUT.md): landing and rebuild has a median of 440.1 min of the 464.5 min per landed job. Reading, planning, producer, checks and verifier together take about 21 min. Idle time is about 44 min per landed job (16070.4 / 366), ...
> 
> A change that implements this design must cite throughput review 20261105-070420-vq95 in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a new design note under docs/design/ that makes landing a continuous step of the drain: after the verifier passes, the job joins the one ordered landing queue, its PR auto-merges, and the rebuild leaves the critical path. It targets the maintainers who read the next throughput review, and it cites review 20261105-070420-vq95 and the predicted 420.0 min saving per landed job.

Assumptions:
- Goal: a reviewable design note, not an implementation. The audience is the Athena maintainers and the next throughput review, which scores the prediction.
- Existing code: docs/design/async-drain.md (sections 6 and 7, the ordered landing queue), docs/design/native-pr-landing.md, docs/design/squash-chain-landing.md and docs/design/warm-baseline-build.md. The note reuses or extends these and creates none of them.
- The note is a new file, docs/design/continuous-landing.md. The exact name is the producer's choice, but it must sit directly under docs/design/.
- THROUGHPUT.md is cited as the source of the measured numbers. If it is absent from the snapshot, the note cites the request text instead.
- No governance files (athena.toml, defaults.toml, workflows) change, because this is a design note only.

## Acceptance criteria

1. [content] A new markdown file directly under docs/design/ describes making landing a continuous drain step: after the verifier passes, the job joins the one ordered landing queue, and its PR merges automatically.
2. [content] The note has an Existing code section that reuses or extends docs/design/async-drain.md (sections 6 and 7) and docs/design/native-pr-landing.md. It says which parts it reuses and which it changes.
3. [content] The note explains, with the measured figures (440.1 of 464.5 min in landing and rebuild, about 21 min for all other stages), how the change removes the bottleneck. Grep finds the figures in the note.
4. [content] The note states the predicted saving of 420.0 minutes per landed job. It also names how the next throughput review measures it: the median minutes per landed job from the verifier's end to the landing record, compared with 464.5.
5. [content] The note cites throughput review 20261105-070420-vq95 by that id, and says a change implementing the design must cite it in its request text.
6. [content] The note lists alternatives considered, risks (ordering, rebuild failure, merge conflicts) and what is out of scope.
7. [behavior] Run cargo test from the repository root. It passes, because any docs-consistency tests still hold with the new file.
8. [content] Only files under docs/design/ change, and no governance file changes.

## Expected deliverables

- docs/design/continuous-landing.md (create)
- docs/design/async-drain.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/ or any code
- Changing athena.toml, defaults.toml or workflows
- Re-running or editing the throughput review itself

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note lists alternatives considered and the reasons for the choice.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The note cites measured numbers (440.1, 464.5, 21, 16070.4/366) to their source and labels the 420.0 min saving as a prediction.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires the note to name the measured minutes per landed job as the outcome the next review checks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The note is written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One note and no code changes.
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
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

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

- path: <state>/jobs/20261105-070552-kx1y/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-070552-kx1y/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new markdown file directly under docs/design/ describes making landing a continuous drain step: after the verifier passes, the job joins the one ordered landing queue, and its PR merges automatically.
The note has an Existing code section that reuses or extends docs/design/async-drain.md (sections 6 and 7) and docs/design/native-pr-landing.md. It says which parts it reuses and which it changes.
The note explains, with the measured figures (440.1 of 464.5 min in landing and rebuild, about 21 min for all other stages), how the change removes the bottleneck. Grep finds the figures in the note.
The note states the predicted saving of 420.0 minutes per landed job. It also names how the next throughput review measures it: the median minutes per landed job from the verifier's end to the landing record, compared with 464.5.
The note cites throughput review 20261105-070420-vq95 by that id, and says a change implementing the design must cite it in its request text.
The note lists alternatives considered, risks (ordering, rebuild failure, merge conflicts) and what is out of scope.
Run cargo test from the repository root. It passes, because any docs-consistency tests still hold with the new file.
Only files under docs/design/ change, and no governance file changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
