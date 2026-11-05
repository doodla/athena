# Job 20261104-192155-3b3e: Design continuous drain-owned landing to remove post-verifier wait

## Request (verbatim)

> Priority: high
> 
> Throughput design: The wait after the verifier finishes and before the landing record exists takes a median of 499.1 of
> 
> Athena's throughput review 20261104-192053-745k named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The wait after the verifier finishes and before the landing record exists takes a median of 499.1 of the 528.2 minutes per landed job (THROUGHPUT.md, last 20 landed jobs). All other stages together take about 32 minutes. A verified job sits...
> Stage: landing and rebuild
> Proposed change: Make landing one continuous step that Athena owns, run by the drain's ordered landing queue, so a job no longer waits for a later command to notice its merge. When the verifier passes, the queue does the whole sequence in one slot: rebase o...
> Predicted saving: 470.0 minutes per landed job; the median was 528.2 minutes per landed job when the review ran.
> Why: Measured: THROUGHPUT.md gives a median of 528.2 minutes per landed job, of which 499.1 is landing and rebuild. Reading (0.8), planning (4.1), producer (24.2), checks (0.0) and verifier (2.9) add up to about 32, so no change inside those sta...
> 
> A change that implements this design must cite throughput review 20261104-192053-745k in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for throughput review 20261104-192053-745k: make landing one continuous step owned by the drain's ordered landing queue, removing the ~499 min median wait between verifier pass and landing record. Audience is the Athena maintainer and the next throughput review.

Assumptions:
- Existing code: docs/design/async-drain.md, docs/design/native-pr-landing.md, docs/design/squash-chain-landing.md, docs/design/parallel-jobs.md, crates/athena-git/src/lib.rs and crates/athena-gate/src/history.rs; the design builds on these and creates none of them.
- Goal: a design note only; no code is implemented here. A later change implementing it must cite review 20261104-192053-745k.
- The request text is truncated; the note takes the quoted bottleneck, 470.0 min predicted saving, and 528.2 / 499.1 baseline figures as given from THROUGHPUT.md.
- No governance files (athena.toml, defaults.toml, workflows) need to change for a design note.

## Acceptance criteria

1. [content] A new file under docs/design/ describes the continuous, drain-owned landing step (rebase, verify, merge, landing record in one queue slot) and explains how it removes the wait between verifier pass and landing record.
2. [content] The design has an Existing code section that reuses or extends docs/design/async-drain.md, docs/design/native-pr-landing.md and crates/athena-git/src/lib.rs, saying what each already provides.
3. [content] The note states the predicted saving of 470.0 minutes per landed job (baseline median 528.2, of which 499.1 is landing and rebuild, citing THROUGHPUT.md) and states how the next throughput review measures the minutes after the change landed.
4. [content] The note cites review 20261104-192053-745k and says an implementing change must cite it in its request text.
5. [content] The note lists alternatives considered, risks (rebase conflicts, ordering, failed landing) and an explicit rollback or failure path.
6. [content] The diff changes only files under docs/design/ (plus optionally CHANGELOG.md); no athena.toml, defaults.toml or workflow file changes.

## Expected deliverables

- docs/design/continuous-landing.md (create)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the landing change in crates/
- Changes to athena.toml, defaults.toml or .github/workflows
- Editing THROUGHPUT.md or records/

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note lists alternatives considered and the chosen design.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cite THROUGHPUT.md and are labeled measured or estimated.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Note is written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the note states the saving and how the next review measures it.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new design document.
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

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new file under docs/design/ describes the continuous, drain-owned landing step (rebase, verify, merge, landing record in one queue slot) and explains how it removes the wait between verifier pass and landing record.
The design has an Existing code section that reuses or extends docs/design/async-drain.md, docs/design/native-pr-landing.md and crates/athena-git/src/lib.rs, saying what each already provides.
The note states the predicted saving of 470.0 minutes per landed job (baseline median 528.2, of which 499.1 is landing and rebuild, citing THROUGHPUT.md) and states how the next throughput review measures the minutes after the change landed.
The note cites review 20261104-192053-745k and says an implementing change must cite it in its request text.
The note lists alternatives considered, risks (rebase conflicts, ordering, failed landing) and an explicit rollback or failure path.
The diff changes only files under docs/design/ (plus optionally CHANGELOG.md); no athena.toml, defaults.toml or workflow file changes.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
