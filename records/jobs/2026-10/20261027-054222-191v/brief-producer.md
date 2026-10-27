# Job 20261027-054222-191v: Show readers the token headroom of every prompt set in their brief

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Readings choose where a prompt edit goes without seeing how much token budget each prompt set has le
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: reading-edits-prompt-without-checking-token-budget (20261024-002642-bm4c)
> Cause: Readings choose where a prompt edit goes without seeing how much token budget each prompt set has left. The shared verifier prompt is composed into several sets, so an addition there costs every set that includes it, and one set had 4 token...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261024-002725-eqs1 (job 20261024-002644-1as8); prevention it proposed: Add a pre-dispatch check for any job whose reading touches prompt files. It computes the token headroom of every prompt set that composes each target file, the same figure the budgets test prints, and adds it to the reading. If the planned ...
> Introspection 20261024-104248-8p9c (job 20261024-093833-vava); prevention it proposed: Before a reading is accepted, run a check that computes the remaining token room for every composed prompt (producer per capability, verifier, and so on) and puts the table into the reading stage's context. Add a check that any reading or p...
> Introspection 20261025-090447-2zm3 (job 20261025-034452-wqp8); prevention it proposed: Add a check at reading admission. It takes each prompt file the reading or plan says it will grow, and compares the planned growth with the remaining budget the budgets test computes. It rejects the reading, naming the file and the room lef...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (failure class reading-edits-prompt-without-checking-token-budget, seen in jobs 20261024-002644-1as8, 20261024-093833-vava and 20261025-034452-wqp8): the reader's brief never says how many tokens each composed prompt set has left. The shared verifier.md, reader.md and producer.md therefore get planned for growth that overruns a set's budget. The fix is to compute the headroom per set in code and put it in the reader brief, with the files each set composes, so the reading picks a target file that fits and lists no overrun.

Assumptions:
- Goal: readings stop planning prompt edits that overflow a budget. The audience is the reader agent, and the maintainers who judge its readings.
- Job 20261024-002644-1as8 (landed, pass, 0 findings): it kept its change inside crates/athena-cap-change/prompts/verifier.md because the shared verifier prompt was full. Nothing is left to fix; this change makes that headroom visible up front. Its scope stays untouched.
- Job 20261024-093833-vava (landed, 2 findings: run_pin_tests.rs changed outside the listed deliverables; a nit on D116 wording). Neither finding applies here, since this change touches neither file. The same outcome-check applies: its out-of-scope items 'adding to the change verifier prompt (at 1298 of 1300 tokens)' and 'growing the shared producer.md beyond its budget' stay out of scope, and this change grows no prompt.
- Job 20261025-034452-wqp8 (rejected, 5 findings: D122 inserted inside D121; no test for urgent_reason parse path; unlisted files changed). Its decisions.md finding applies to this change: the new decision is appended as a fresh heading at the end and no existing decision is edited. The other findings concern the urgent priority feature, which is not redone here. Its out-of-scope items 'growing reader.md beyond the token budget' and 'changing athena.toml or defaults.toml' stay out of scope.
- The headroom is computed from the same compose and check_budget logic that crates/athena-cli/tests/budgets.rs uses, so the figures match what that test prints. Sets: contract, reader, planner, critic, and producer and verifier per enabled capability. Each row names its component ids (for example frame.verifier, capability.change.verifier) so the reader can see which sets a shared file feeds.
- The figures go in the reader's user brief (ReaderInputs), not in reader.md, which is near its own budget. The reader.md wording is left unchanged, and the brief heading says plainly that an edit to a listed file costs every set that includes it.
- Pre-dispatch rejection of over-budget readings, as two of the introspections suggested, is not built here. The reader is shown the numbers and the existing budgets test is the backstop, which keeps the change small.

## Acceptance criteria

1. [content] The reader brief has a 'Prompt token headroom' section listing, for each budgeted set (contract, reader, planner, critic, and producer and verifier per enabled capability), the component ids it composes, the estimated tokens used, the budget and the tokens left. It says an edit to a shared file costs every set that includes it.
2. [content] The headroom is computed in code by one function in crates/athena-context/src/lib.rs, using compose and the budget values already there. reading.rs passes the rows into ReaderInputs, and no budget number is hard-coded in the brief.
3. [behavior] Run `cargo test -p athena-context`. A new briefs test builds a ReaderInputs whose rows show a set with 2 tokens left, and asserts the brief names that set, its component ids and '2'. Another asserts the section is absent when no rows are given. Both tests fail without the change.
4. [behavior] Run `cargo test -p athena-cli --test budgets`. A new test asserts the computed headroom rows equal the figures behind computed_rows(), so the brief cannot drift from the budgets test. It also asserts a shared-file row, such as the verifier frame, appears in every verifier set that composes it.
5. [behavior] Run `cargo test -p athena-core reading`. A test shows the reader invocation's user prompt (via the fake driver) contains the headroom section with the real verifier and reader figures.
6. [content] CHANGELOG.md has an entry. docs/decisions.md gets one new decision appended as its own heading, naming the root cause, citing the three introspections and the alternatives (pre-dispatch rejection, a note in reader.md), without editing any existing decision.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, including the system-facts budgets test, which shows no prompt file was grown.

## Expected deliverables

- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Any change to the job 20261024-002644-1as8 deliverables: crates/athena-cap-change/prompts/verifier.md and the shared verifier prompt
- Growing or editing any prompt file, including reader.md, producer.md and verifier.md, and the change verifier at 1298 of 1300 tokens
- Rejecting readings automatically at admission for exceeding a budget
- Changing budget values or the token estimator
- Re-doing the urgent-priority feature of job 20261025-034452-wqp8
- Editing existing entries in docs/decisions.md, records/, athena.toml or defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in athena-context, athena-cli budgets and athena-core reading; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the source: the reader's brief lacks the headroom data, so the data is added there for the whole class of prompt edits.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the rendered brief and the reader's actual prompt, not that types exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is a reader that sees headroom per set. Criteria check that and not the activity.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change. Pre-dispatch rejection is left out.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test ties the shown figures to the budgets computation, so they cannot drift.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the decision record are written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs the named cargo commands.
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

The reader brief has a 'Prompt token headroom' section listing, for each budgeted set (contract, reader, planner, critic, and producer and verifier per enabled capability), the component ids it composes, the estimated tokens used, the budget and the tokens left. It says an edit to a shared file costs every set that includes it.
The headroom is computed in code by one function in crates/athena-context/src/lib.rs, using compose and the budget values already there. reading.rs passes the rows into ReaderInputs, and no budget number is hard-coded in the brief.
Run `cargo test -p athena-context`. A new briefs test builds a ReaderInputs whose rows show a set with 2 tokens left, and asserts the brief names that set, its component ids and '2'. Another asserts the section is absent when no rows are given. Both tests fail without the change.
Run `cargo test -p athena-cli --test budgets`. A new test asserts the computed headroom rows equal the figures behind computed_rows(), so the brief cannot drift from the budgets test. It also asserts a shared-file row, such as the verifier frame, appears in every verifier set that composes it.
Run `cargo test -p athena-core reading`. A test shows the reader invocation's user prompt (via the fake driver) contains the headroom section with the real verifier and reader figures.
CHANGELOG.md has an entry. docs/decisions.md gets one new decision appended as its own heading, naming the root cause, citing the three introspections and the alternatives (pre-dispatch rejection, a note in reader.md), without editing any existing decision.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass, including the system-facts budgets test, which shows no prompt file was grown.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
