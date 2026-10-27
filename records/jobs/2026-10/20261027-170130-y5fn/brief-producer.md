# Job 20261027-170130-y5fn: Require planner demos to name a verified test seam; keep budgets

## Request (verbatim)

> try again on 20261027-054203-wkp5

## Reading (accepted by the user)

Retry of job 20261027-054203-wkp5. Fix the root cause (the planning stage never requires a demonstration to name and confirm its test seam) in planner.md, critic.md and the planner brief in briefs.rs, and this time update docs/system-facts.md budget lines and the lib.rs frame-size guard so budget tests pass, with the producer citing the commands it ran.

Assumptions:
- Goal: planner and critic prompts and the planner brief make each demonstration name its seam; audience is the planning stage and the verifier of this change.
- Job 20261027-054203-wkp5 findings: (1) budgets tests failed because docs/system-facts.md planner/critic lines were stale: fixed by recomputing and updating those two lines (never the budget); docs/system-facts.md is now a deliverable and named in a criterion. (2) The lib.rs frame-size threshold edit was outside the deliverables: now listed, and the producer should prefer shortening wording to fit the existing 500-token guard, raising it only if justified in CHANGELOG. (3) Missing command evidence: the producer's report must cite cargo test, clippy and fmt commands run with results.
- Producer first measures remaining planner (1300) and critic (1300) room, adds the seam text, and pays for it by shortening existing wording of equal meaning where needed; budgets themselves are not raised.
- Sizes are ceil(bytes/3); facts lines must equal the computed values exactly, which the budgets test checks.

## Acceptance criteria

1. [content] planner.md requires each demonstration to name the production entry point it calls, its guards and preconditions, the seam/fake/fixture and how it is injected, and whether the seam exists today (checked by reading the code) or which plan step adds it; it forbids putting the logic under test inside the test double.
2. [content] critic.md tells the critic to raise a major finding when a demonstration names no seam, names one the code does not provide, or tests only a double.
3. [content] The planner brief rendered by planner_brief in briefs.rs repeats the seam requirement in its Report section.
4. [content] docs/system-facts.md budget planner and budget critic lines state the computed sizes of the changed prompts, budgets (of 1300) are unchanged, and CHANGELOG.md names the root cause and that it is fixed in the planning stage, not at one job.
5. [behavior] A new unit test in crates/athena-context asserts the planner brief and the planner and critic prompts contain the seam requirement; `cargo test -p athena-context` passes, and the test fails on the base commit.
6. [behavior] `cargo test -p athena-cli --test budgets` passes, including system_facts_budgets_match_computed and system_facts_drift_is_rejected.
7. [behavior] `cargo fmt --check`, `cargo clippy --locked` and `cargo test --locked` all pass, and the producer's report cites each command it ran with its result.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/system-facts.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-context/src/lib.rs (modify)

## Out of scope

- Changing the three-round cap or the critic's verdict logic
- Adding fields to the Demonstration record or bumping its schema
- Redoing the landing, records or PR changes from the referenced jobs
- Changing CI workflows or governance files
- Raising the token budgets themselves

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test plus full cargo test, clippy and fmt.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that prompts and brief carry the seam rule and that budgets tests pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name the exact cargo commands to run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in planning stage prompts and brief, not one job.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused prompt change plus its dependent size figures.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261027-054203-wkp5 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 5 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`
> blocker (crates/athena-cli/tests/budgets.rs:128,138; docs/system-facts.md:25-26): Longer planner and critic prompts break the system-facts budget tests, and the facts given to planner and critic are now wrong In my `cargo test --locked --no-fail-fast -q` run, system_facts_budgets_match_computed and system_facts_drift_is_rejected failed. The message was 'planner: the facts file says 788 of 1300, the computed value is 952 of 1300; update docs/system-facts.md'. Byte arithmetic shows 788 (planner) and 742 (critic) were correct for the base prompts, so this change caused the failures. The critic line is also stale: its computed size is now about 849, an estimate from file sizes. docs/system-facts.md says to update the size line, never the budget. That file is not on the deliverables list and the request says 'Files: crates/', so the plan should have listed it or kept the prompts the same length.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

16 more qualifying classes are left out.

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

planner.md requires each demonstration to name the production entry point it calls, its guards and preconditions, the seam/fake/fixture and how it is injected, and whether the seam exists today (checked by reading the code) or which plan step adds it; it forbids putting the logic under test inside the test double.
critic.md tells the critic to raise a major finding when a demonstration names no seam, names one the code does not provide, or tests only a double.
The planner brief rendered by planner_brief in briefs.rs repeats the seam requirement in its Report section.
docs/system-facts.md budget planner and budget critic lines state the computed sizes of the changed prompts, budgets (of 1300) are unchanged, and CHANGELOG.md names the root cause and that it is fixed in the planning stage, not at one job.
A new unit test in crates/athena-context asserts the planner brief and the planner and critic prompts contain the seam requirement; `cargo test -p athena-context` passes, and the test fails on the base commit.
`cargo test -p athena-cli --test budgets` passes, including system_facts_budgets_match_computed and system_facts_drift_is_rejected.
`cargo fmt --check`, `cargo clippy --locked` and `cargo test --locked` all pass, and the producer's report cites each command it ran with its result.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
