# Job 20261027-054157-e8z4: Make planning enforce owner priority order, with a check

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Planning treats every rung as the same kind of work and sequences by default, not by the owner's sta
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-order-ignores-owner-priority (20261016-174525-zw7r)
> Cause: Planning treats every rung as the same kind of work and sequences by default, not by the owner's stated priority. Owner priorities from addenda are not an input to plan ordering, and there is no check that the plan's order matches the lates...
> Seen 6 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261016-174515-crny (job 20261016-071159-s6tr); prevention it proposed: Make owner priority a first-class field. Record it from requests and addenda, and have the planner sort rungs by it before the default order. Add a code check that fails plan creation when the top stated priority is not in the first positio...
> Introspection 20261016-182927-a4cf (job 20261016-071202-q0xy); prevention it proposed: Make the owner's priorities, recorded from requests and addenda, a required input to plan ordering. Give each plan rung a rank derived from them. Add a code check that fails plan creation, or flags the plan, when the rung order contradicts ...
> Introspection 20261017-044416-s1q3 (job 20261017-003902-d5gk); prevention it proposed: Make owner priority an input to planning and to the queue. Let an addendum or operator mark a rung with a priority. Have the planner sort by that priority ahead of default rung order. Add a check that fails plan generation when the top stat...
> Introspection 20261017-052636-m3tt (job 20261017-044557-3htd); prevention it proposed: Make the owner's priority a recorded input to planning, and have the queue honor it. The owner states an ordered priority list, or marks a series such as CI routing as top priority. The plan sorts by that list before default rung type. When...
> Introspection 20261017-130032-60w8 (job 20261016-152050-4kgd); prevention it proposed: Make planning use recurrence of operator interventions as an input. When the same gap has produced N or more hand rescues (for example 3), or when the owner marks a priority, the job that fixes the gap moves to the front of the plan automat...
> Introspection 20261017-152740-zbj3 (job 20261017-152733-efyd); prevention it proposed: Add an owner-priority input to the plan and to the queue. The owner records an ordered priority list, such as speed via CI. The planner ranks any job tagged to a top priority ahead of default-ordered work, and keeps a multi-item series toge...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: plan ordering in crates/athena-core/src/plan.rs only honours a per-request `Priority:` line or priority record, so owner priorities stated in BRIEF.md addenda never reach the plan, and nothing checks the built order against them. Fix at plan construction: an owner-priority input ranks rungs ahead of default order, and a check flags a plan whose top-priority item is not in the first wave. The retried jobs (s6tr, q0xy, d5gk) are only the failure instances: they are cited as evidence and none is redone.

Assumptions:
- Audience: the owner and operator who read the plan and queue, and the verifier. Goal: a stated owner priority reliably moves work to the front.
- Repository already has Priority (urgent/high/normal/low), an effective-priority fixpoint in Plan::build, and `athena priority` records. The gap is the addendum-derived input and the order-versus-priority check. I did not read BRIEF.md or the addendum parsing, so that is an assumption.
- Job 20261016-071159-s6tr (landed, pass): its findings were about xtask deps, the walltime measure and a cache nit. They are unrelated to the cause and no longer apply, so they are not touched.
- Job 20261016-071202-q0xy (failed): its athena.toml `[parallel]` parse refusal is a separate config-schema failure and is not part of this cause. It is left unaddressed here.
- Job 20261017-003902-d5gk (landed, pass): its findings concern CI result parsing and the unwired resolve. They are unrelated to ordering and are not touched.
- Tests are added in plan.rs's own test module, so no governance files change.

## Acceptance criteria

1. [content] The change description names the root cause (owner priority from addenda is not an input to plan ordering and the order is never checked against it) and the fix sits in crates/athena-core/src/plan.rs, not in one job's request.
2. [behavior] Run `cargo test -p athena-core plan`: a new test builds a plan where an addendum-stated top priority belongs to a newer, later-rung item and asserts it is in the first wave. The same test fails when the change is reverted.
3. [behavior] Run `cargo test -p athena-core plan`: a new test asserts the order check reports a plan whose top-priority item is not first, and passes a correctly ordered plan. It covers the general case with several items, not one job's ids.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass, with the existing plan ordering tests (dependencies first, effective priority) unchanged.
5. [content] CHANGELOG.md has an entry that states the cause and the new check, for an outside reader.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- The CI routing build and walltime work from jobs s6tr and d5gk
- The `[parallel]` config schema failure from job q0xy
- Parallel job execution
- Rewriting existing past plans or records

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at plan construction and its check, covering the whole class.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail before the change and pass after.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, mostly in plan.rs.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check that order follows owner priority.
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

The change description names the root cause (owner priority from addenda is not an input to plan ordering and the order is never checked against it) and the fix sits in crates/athena-core/src/plan.rs, not in one job's request.
Run `cargo test -p athena-core plan`: a new test builds a plan where an addendum-stated top priority belongs to a newer, later-rung item and asserts it is in the first wave. The same test fails when the change is reverted.
Run `cargo test -p athena-core plan`: a new test asserts the order check reports a plan whose top-priority item is not first, and passes a correctly ordered plan. It covers the general case with several items, not one job's ids.
Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass, with the existing plan ordering tests (dependencies first, effective priority) unchanged.
CHANGELOG.md has an entry that states the cause and the new check, for an outside reader.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
