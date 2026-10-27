# Job 20261027-054151-ace0: Carry prior result and change scope into retry readings; refuse

## Request (verbatim)

> Priority: urgent
> 
> Make the reading stage's input include the previous attempt's result and the allowed change scope for a retried or follow-up job. Then add a check that refuses a reading that leaves either one out. Check for overlap with the queued item reading-omits-prior-verifier-finding-or-end-state.
> 
> Introspections that record the repeated cause: 20261016-060515-0801, 20261024-020022-z573, 20261024-115134-0pna, 20261024-154638-19r5, 20261024-161644-yhq2, 20261024-170310-67x6, 20261025-022225-ffsv
> 
> Acceptance:
> - The change the request describes is made, with the root cause named
> - A test fails without the change and passes with it
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

For a retried or follow-up job, the reader's brief must always state the previous attempt's result (verdict, findings, end state) and the allowed change scope (deliverables and out-of-scope). A new check then refuses any reading that omits either. Audience: the reader agent and the verifier. Root cause: the brief carries these only partly and nothing enforces the reading to use them.

Assumptions:
- Existing machinery (EarlierAttempt in athena-context briefs.rs, earlier_attempt/referenced_jobs in athena-core reading.rs) already carries findings, reading criteria and deliverables; the gap is the end state and out-of-scope (change scope) for retries and follow-ups, and that no check refuses a reading that ignores them. I did not see the actual introspection records, which are absent.
- The queued item reading-omits-prior-verifier-finding-or-end-state is in records/, which I cannot read. I assume it overlaps on the prior finding and end-state half. This job covers it, adds the scope half and the refusal check, and names the overlap in its summary and the changelog; it does not edit the queue.
- The check lives where a reading is accepted after read_for in athena-core and fails the reading with a clear message, so the reader is asked again within the existing revision loop.
- A reading 'leaves out' the prior result if it neither addresses the earlier findings or end state in its summary, assumptions or criteria, nor states why they no longer apply. A deterministic check can test this by comparing the reading's text and criteria against the carried findings, deliverables and out-of-scope.

## Acceptance criteria

1. [content] The reader brief for a retry (try again on <job>) and for a follow-up job that names an earlier job includes a section with the previous attempt's result (verdict, findings or outcome reason, end state) and the allowed change scope (deliverable paths and out-of-scope items), with the scope present even if the earlier reading had none.
2. [content] A new check function in athena-core refuses a reading for a retried or follow-up job that does not reflect the prior result or does not state the change scope, returning an error that names which of the two is missing. It does not apply to a first-attempt job.
3. [content] The change summary or CHANGELOG states the root cause (the brief and the acceptance of a reading did not require the prior result and scope) and states the overlap with reading-omits-prior-verifier-finding-or-end-state: what it covers and what is left.
4. [behavior] New unit tests in crates/athena-context and crates/athena-core fail on the pre-change code and pass after. They cover the brief containing the result and scope, a reading missing the result refused, a reading missing the scope refused, a complete reading accepted, and a first attempt unaffected. Run `cargo test` and compare with the base.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root with no new warnings.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Editing or closing the queued item reading-omits-prior-verifier-finding-or-end-state (records/ is not changed)
- Changing the verifier, producer or landing behavior
- Changing athena.toml, defaults.toml or workflows
- Rewriting the introspection records

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New failing-then-passing tests plus cargo test and clippy criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The criteria require naming the root cause and adding a mechanism check, not only prompt text.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A code check enforces the rule instead of prose only.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The tests exercise the brief output and the refusal.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that a reading omitting the result or scope is refused.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the reading and brief code.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
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

The reader brief for a retry (try again on <job>) and for a follow-up job that names an earlier job includes a section with the previous attempt's result (verdict, findings or outcome reason, end state) and the allowed change scope (deliverable paths and out-of-scope items), with the scope present even if the earlier reading had none.
A new check function in athena-core refuses a reading for a retried or follow-up job that does not reflect the prior result or does not state the change scope, returning an error that names which of the two is missing. It does not apply to a first-attempt job.
The change summary or CHANGELOG states the root cause (the brief and the acceptance of a reading did not require the prior result and scope) and states the overlap with reading-omits-prior-verifier-finding-or-end-state: what it covers and what is left.
New unit tests in crates/athena-context and crates/athena-core fail on the pre-change code and pass after. They cover the brief containing the result and scope, a reading missing the result refused, a reading missing the scope refused, a complete reading accepted, and a first attempt unaffected. Run `cargo test` and compare with the base.
`cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass from the repository root with no new warnings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
