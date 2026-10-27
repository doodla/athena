# Job 20261027-085401-0qgf: Order approved steward proposals in the plan like other items

## Request (verbatim)

> Priority: urgent
> athena drain works approved steward proposals before the plan: on 2026-10-27 its first drain approved proposal 20261018-000648-ne91 (a 9-day-old explainer-page fix), spent both of its job slots on it (failed twice) and never reached the urgent items at the head of athena plan. Approved proposals must enter the plan like any other item and be ordered by the same rules (dependency, then priority, then rung, then age), with a proposal's priority defaulting to normal unless its role or a priority record says otherwise.
> Acceptance:
> - A drain test with an approved in-limit proposal and an urgent queued request works the urgent request first.
> - athena plan lists approved proposals in their ordered place.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/plan.rs, CHANGELOG.md

## Reading (accepted by the user)

Drain currently approves in-limit steward proposals up front (approve_all) and works them before the plan, starving urgent queued requests. Make approved proposals plan entries ordered by dependency, priority, rung, age, with priority defaulting to normal unless the role or a priority record says otherwise; drain then follows the plan, and `athena plan` lists them in place. Audience: operators running `athena drain` and `athena plan`.

Assumptions:
- Approval still happens in drain, but approved proposals' jobs are merged into the plan and ordered rather than worked first.
- Priority for a proposal comes from its role or a priority record; absent either it is normal.
- No governance files need to change.

## Acceptance criteria

1. [behavior] A new drain test (run with cargo test -p athena-cli drain) with an approved in-limit proposal and an urgent queued request works the urgent request first, and fails on the old behavior.
2. [behavior] A test shows the plan (Plan::build / the plan command path) lists an approved proposal at its ordered place by dependency, priority, rung, then age, defaulting to normal priority, with role or priority-record overrides honored.
3. [content] Drain no longer works approved proposals ahead of the plan; approved proposals are ordered by the same rules as other items and job-limit/budget handling still applies.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.
5. [content] CHANGELOG.md has an entry describing the fix for a reader outside the project.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Fixing the explainer-page proposal 20261018-000648-ne91 itself
- Changing proposal approval limits or rung rules
- Changing the job limit semantics

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for drain order and plan listing; fmt, clippy -D warnings, tests pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check order of work and plan output.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix ordering by putting proposals into the plan, not special-casing the one proposal.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Urgent request is worked first.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

A new drain test (run with cargo test -p athena-cli drain) with an approved in-limit proposal and an urgent queued request works the urgent request first, and fails on the old behavior.
A test shows the plan (Plan::build / the plan command path) lists an approved proposal at its ordered place by dependency, priority, rung, then age, defaulting to normal priority, with role or priority-record overrides honored.
Drain no longer works approved proposals ahead of the plan; approved proposals are ordered by the same rules as other items and job-limit/budget handling still applies.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.
CHANGELOG.md has an entry describing the fix for a reader outside the project.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
