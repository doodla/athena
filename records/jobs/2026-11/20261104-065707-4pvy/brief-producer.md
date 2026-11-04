# Job 20261104-065707-4pvy: Fix firstmate study Reopened state: cited return to Queued, tests green

## Request (verbatim)

> try again on 20261104-062715-yv4m

## Reading (accepted by the user)

Retry of job 20261104-062715-yv4m: make 'Reopened' in the firstmate study a non-terminal state that returns to Queued, with only Done ending the task. Add a cited claim for the Queued return, fix the t12 guard-'none' test failure, and fix the fmt failure in xtask/src/site.rs.

Assumptions:
- Job 20261104-062715-yv4m findings are addressed: (1) the t12 guard-'none' test failure is fixed by giving the return a non-'none' evaluated guard or by pointing t9 at queued; (2) the uncited Queued return gets a new claim in claims.json; (3) the cargo fmt failure at xtask/src/site.rs:963 is fixed; (4) validation.json is updated if needed; (5) the behavior criterion names the commands to run. The earlier t9/t12 double-event unsoundness is removed.
- xtask/src/site.rs exists at the base commit. The earlier candidate changed it, and that is kept as a deliverable: it is limited to the tests that asserted Reopened's old tag or counted transitions or cards, and it must be formatted. Its other changes in CHANGELOG.md, intro.json and model.json are kept and reworked as described.
- Goal: a correct 'Reopened' state on the firstmate study page, so the post-landing source check stops refusing it. The audience is the study page reader and that check. The reading is docs-class: study data is edited and Rust tests are only adjusted to match.

## Acceptance criteria

1. [content] In studies/firstmate-work-handling/model.json the 'reopened' state no longer has kind 'terminal-success' (Grep confirms); it is a non-terminal kind with a way back to 'queued', matching bin/fm-teardown.sh sending the backlog row back to Queued.
2. [content] studies/firstmate-work-handling/claims.json gains a new claim citing the pinned fm-teardown.sh lines (around 3894-3896) that show the backlog row returning to Queued. Every new or changed Queued-return text in model.json and intro.json cites that claim, not only c23-c25.
3. [content] Reopened's way out of the model is sound: t9 still reaches Reopened and the return to Queued is either t9 pointing straight at queued or a transition with a distinct cited trigger. It is not the t9 event repeated with guard 'none'. Any guard of 'none' has no guard_evaluated_by value, as the study rule test requires.
4. [content] studies/firstmate-work-handling/intro.json: the 'states' lead-in says only Done ends the task (Grep for 'ends the task' shows no Reopened). The 'reopened' glossary definition says the item goes back to Queued for the captain's answer and cites the new claim.
5. [content] studies/firstmate-work-handling/validation.json is updated to cover the new claim or transition if its structure requires it. CHANGELOG.md has an entry. Changes to xtask/src/site.rs are limited to the tests that asserted the old 'ends the task' tag on Reopened or counted transitions or cards. They are rustfmt-clean, including the starts_with call near line 963.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`, and `cargo test --locked`. All pass, including athena_cap_study rules::tests::a_guard_of_none_is_never_said_to_be_evaluated_by_code and the xtask site_page and policy tests, and the verifier sees the commands run.

## Expected deliverables

- studies/firstmate-work-handling/model.json (modify)
- studies/firstmate-work-handling/intro.json (modify)
- studies/firstmate-work-handling/claims.json (modify)
- studies/firstmate-work-handling/validation.json (modify)
- CHANGELOG.md (modify)
- xtask/src/site.rs (modify)

## Out of scope

- Changing the firstmate source code or the pinned commit
- Other states or study mechanisms
- Changing the study rules in crates/ or the source-check logic

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check Reopened's kind, its cited Queued return, and the full fmt, clippy and test run.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The new Queued-return claim cites the pinned source lines.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the model, the intro and the uncited Queued return, not just the tag.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior criterion names the commands to run.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261104-062715-yv4m raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 4 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`
> blocker (studies/firstmate-work-handling/model.json (t12, guard_evaluated_by)): The new transition t12 breaks a study rule test: a guard of "none" may not be said to be evaluated by code. cargo test -p athena-cap-study --lib a_guard_of_none_is_never_said_to_be_evaluated_by_code fails: 'assertion `left != right` failed: t12'. The CI test run fails on the same test.
> major (studies/firstmate-work-handling/intro.json (states lead-in, t12, reopened glossary); model.json t12): The Queued return is a new claim, but it cites only c23-c25, which do not support it, and claims.json was not updated. c23-c25 cite fm-teardown.sh lines 30-42 and fm-backlog-transition-lib.sh lines 15 and 54, about reopen versus close, the retain-mode replay and --force. None mentions Queued or lines 3894-3896. The source check is likely to refuse the new text for the same reason it refused the old text. claims.json is a listed deliverable and was left unchanged.
> major (xtask/src/site.rs:963): The format check fails on the changed test code. cargo fmt --all --check shows a diff in xtask/src/site.rs:963: the starts_with call should be on one line.

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

In studies/firstmate-work-handling/model.json the 'reopened' state no longer has kind 'terminal-success' (Grep confirms); it is a non-terminal kind with a way back to 'queued', matching bin/fm-teardown.sh sending the backlog row back to Queued.
studies/firstmate-work-handling/claims.json gains a new claim citing the pinned fm-teardown.sh lines (around 3894-3896) that show the backlog row returning to Queued. Every new or changed Queued-return text in model.json and intro.json cites that claim, not only c23-c25.
Reopened's way out of the model is sound: t9 still reaches Reopened and the return to Queued is either t9 pointing straight at queued or a transition with a distinct cited trigger. It is not the t9 event repeated with guard 'none'. Any guard of 'none' has no guard_evaluated_by value, as the study rule test requires.
studies/firstmate-work-handling/intro.json: the 'states' lead-in says only Done ends the task (Grep for 'ends the task' shows no Reopened). The 'reopened' glossary definition says the item goes back to Queued for the captain's answer and cites the new claim.
studies/firstmate-work-handling/validation.json is updated to cover the new claim or transition if its structure requires it. CHANGELOG.md has an entry. Changes to xtask/src/site.rs are limited to the tests that asserted the old 'ends the task' tag on Reopened or counted transitions or cards. They are rustfmt-clean, including the starts_with call near line 963.
Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`, and `cargo test --locked`. All pass, including athena_cap_study rules::tests::a_guard_of_none_is_never_said_to_be_evaluated_by_code and the xtask site_page and policy tests, and the verifier sees the commands run.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
