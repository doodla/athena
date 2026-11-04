# Job 20261104-062715-yv4m: Fix the firstmate study's Reopened state: not terminal, back to Queued

## Request (verbatim)

> Fix the firstmate study's 'Reopened' state. The source check after each landing has refused the study page three times today on the same claim: firstmate-work-handling/page-text/model.json marks 'reopened' as a terminal-success state that ends the task, and intro.json says 'Done and Reopened both carry the tag "ends the task"'. The code does not end the item; it sends the backlog row back to Queued (bin/fm-teardown.sh around lines 3894-3896). Correct the model and the intro text so the source check passes. Acceptance: the source check passes on the study page for the reopened state.

## Reading (accepted by the user)

The firstmate study wrongly marks 'reopened' as terminal-success and says Done and Reopened both "end the task". Per the code, teardown reopens the backlog item and it returns to Queued. I'll correct model.json and intro.json (the audience is the reader of the study page and the post-landing source check) so the check passes.

Assumptions:
- The task's 'source check' is the study validation run after landing; I did not read its code, so the exact rule (state kind vs claim text) is unconfirmed.
- I did not read bin/fm-teardown.sh in the pinned source, so lines 3894-3896 are taken from the request. If the claims need a new cited source for the Queued return, claims.json changes too.
- The model may require non-terminal states to have an outgoing transition, so a Reopened->Queued transition may be needed in model.json.

## Acceptance criteria

1. [content] In studies/firstmate-work-handling/model.json the 'reopened' state no longer has kind 'terminal-success'; it is a non-terminal kind (e.g. waiting), or the item returns to 'queued', consistent with bin/fm-teardown.sh sending the backlog row back to Queued.
2. [content] studies/firstmate-work-handling/intro.json 'states' lead-in no longer says Done and Reopened both carry 'ends the task'. It says only Done ends the task, and that Reopened sends the item back to Queued for the captain's answer. Grep for 'ends the task' confirms.
3. [content] The 'reopened' glossary definition in intro.json, and any transition out of Reopened added to model.json, match the cited claims c23-c25 and the Queued return, with no new uncited claims.
4. [behavior] Run cargo test (the study/record validation tests over studies/) and cargo clippy: they pass with the edited study files. The source check on the study page for the reopened state passes, as shown in the recorded check output.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the firstmate source code or the pinned commit
- Other states or study mechanisms

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The final criterion requires the source check to pass on the reopened state.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes both the model and the intro text that repeat the wrong claim.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused edit to two study files.
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
- 9 cases in 7 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Make "fails without the fix" a check that Athena runs itself. For a bug-fix job, Athena would run the new or changed tests against the base tree's production code, with the job's test files overlaid, and record the failing output as a named check. It would then run them on the final tree. The verifi...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

26 more qualifying classes are left out.

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

In studies/firstmate-work-handling/model.json the 'reopened' state no longer has kind 'terminal-success'; it is a non-terminal kind (e.g. waiting), or the item returns to 'queued', consistent with bin/fm-teardown.sh sending the backlog row back to Queued.
studies/firstmate-work-handling/intro.json 'states' lead-in no longer says Done and Reopened both carry 'ends the task'. It says only Done ends the task, and that Reopened sends the item back to Queued for the captain's answer. Grep for 'ends the task' confirms.
The 'reopened' glossary definition in intro.json, and any transition out of Reopened added to model.json, match the cited claims c23-c25 and the Queued return, with no new uncited claims.
Run cargo test (the study/record validation tests over studies/) and cargo clippy: they pass with the edited study files. The source check on the study page for the reopened state passes, as shown in the recorded check output.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
