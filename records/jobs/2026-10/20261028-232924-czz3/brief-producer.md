# Job 20261028-232924-czz3: Treat a model usage limit as a hold at every model-call stage

## Request (verbatim)

> try again on 20261027-160633-t5sh

## Reading (accepted by the user)

Retry of job 20261027-160633-t5sh: recognise the driver's usage-limit reply once, record it as limited at zero cost, and make the drain stop with a 'usage limit' reason naming the reset time. This now includes the request-reading step, which the earlier attempt missed. The goal is that the maintainer running the queue sees a hold and not 'repeated failure'.

Assumptions:
- Prior job 20261027-160633-t5sh was rejected on one major finding: drain still stopped as 'repeated failure' when the limit hit during the read step (reading.rs turns any non-Success outcome into a reader error, drain.rs counts it as a failure). This retry fixes it by carrying a typed limit result from the reader to drain. Its two minor findings are addressed: the three extra files are now listed as deliverables, and drain gets a structured limit flag instead of parsing the closing-reason text.
- The candidate of 20261027-160633-t5sh is not in the repository; the producer redoes its work from the base, keeping the accepted design: a Limited invocation outcome, zero cost, reset time parsed from the text (UTC), job kept retryable with its candidate, and a status line showing the limit while it holds.
- Code survey: read crates/athena-agent/src/claude.rs (output_from_result at line 249 turns driver results into invocation outcomes), crates/athena-core/src/reading.rs (read_queued at line 812 runs the reader), crates/athena-data/src/records.rs (invocation outcome enum and record validation; a new Limited variant must round-trip through the existing record tests and AnyRecord), crates/athena-core/src/run/mod.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/status.rs and crates/athena-cli/src/drain.rs. No Limited outcome exists yet. The limit is measured directly from the reply text, not through a stand-in.
- Model-call stages to cover, each with a test: reading (including approval and retry reads), planning, critic, producer, verifier. The change description should list each stage and its test.
- This does not add a refusal rule for readings; the gate only gains acceptance of a zero-cost limited record.

## Acceptance criteria

1. [behavior] Run `cargo test`: a claude.rs test feeds a fake driver reply with "You've hit your weekly limit · resets Oct 29, 6pm (UTC)" and no usage, and asserts it is recognised as a usage limit with the reset time parsed.
2. [behavior] Run `cargo test`: a test per model-call stage (reading in crates/athena-core/src/reading.rs, planning, critic, producer, verifier) injects the limit reply and asserts the invocation record is limited at zero cost, passes the gate, the job stays retryable with its candidate kept, and no failure is counted.
3. [behavior] Run `cargo test`: a drain test where the limit hits at the reading step of a queued item asserts drain stops with stop reason 'usage limit' naming the reset time, not 'repeated failure', and the failure counter is unchanged. A second drain test does the same for the run step.
4. [behavior] Run `cargo test`: a status.rs test asserts `athena status` shows the usage limit and its reset time while it holds, and shows neither after the reset time has passed.
5. [behavior] Run `cargo test`: a test shows an ordinary non-limit driver error at the reading step still records as an error and counts as a failure.
6. [content] Drain learns of the limit from a structured field or typed result, not by parsing the closing-reason text; a Grep of crates/athena-cli/src/drain.rs finds no parsing of the 'environment: usage limit' prefix.
7. [content] CHANGELOG.md has an entry describing the usage-limit handling, the 'usage limit' stop reason and the status display.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Site-rating work of job 20261027-123701-77zh (xtask/src/site.rs, studies/, docs/design/explainer-site.md)
- Automatic waiting or sleeping until the reset time
- Changing the gate's rules for other records
- Records under records/, athena.toml, defaults.toml and workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check drain stopping as a hold with unchanged failure count, not just that types exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Per-stage tests plus fmt, clippy -D warnings and cargo test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Classify the limit once at the shared point and route it to every stage.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Each behavior criterion names a cargo test that exercises the limit reply.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261027-160633-t5sh raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-core/src/reading.rs:483; crates/athena-cli/src/drain.rs:798-809, 1075): Drain still stops with 'repeated failure' when the usage limit is hit while it reads a request, the first model call it makes for each queued item. Found by reading the code; this scenario was not run. Drain notices a limit only through the run pipeline's closing reason (usage_limit_of in run_job). The reader call is not covered. For a queued item, drain's read step calls athena.go, which calls read_queued and runs the reader. reading.rs:483 turns any non-Success outcome, Limited included, into an ordinary reader error. drain.rs:798-809 turns that into RunStep::Failed, so failures is incremented, the item is read again, and after a second refusal drain stops as RepeatedFailure. Approval reads (drain.rs:1075) and retry reads go through the same reader. A weekly limit lasts days, so a drain started before the reset will most likely hit the limit at the reader first. That is the symptom the request asked to remove ('have drain stop with a usage limit stop reason ... instead of repeated failure'). Simpler, general alternative: have the reader report a Limited invocation as its own read outcome, or have drain check the job's records for a Limited invocation, and route it to RunStep::UsageLimit like the run step.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 32 cases in 32 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Classify the usage-limit error once, at the single point where model-call errors are turned into results, as a typed "hold" outcome. Every caller (reading, planning, critic, producer, verifier) must receive that type, and the drain's failure counter must not count it. Add a test that injects the lim...
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

18 more qualifying classes are left out.

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

Run `cargo test`: a claude.rs test feeds a fake driver reply with "You've hit your weekly limit · resets Oct 29, 6pm (UTC)" and no usage, and asserts it is recognised as a usage limit with the reset time parsed.
Run `cargo test`: a test per model-call stage (reading in crates/athena-core/src/reading.rs, planning, critic, producer, verifier) injects the limit reply and asserts the invocation record is limited at zero cost, passes the gate, the job stays retryable with its candidate kept, and no failure is counted.
Run `cargo test`: a drain test where the limit hits at the reading step of a queued item asserts drain stops with stop reason 'usage limit' naming the reset time, not 'repeated failure', and the failure counter is unchanged. A second drain test does the same for the run step.
Run `cargo test`: a status.rs test asserts `athena status` shows the usage limit and its reset time while it holds, and shows neither after the reset time has passed.
Run `cargo test`: a test shows an ordinary non-limit driver error at the reading step still records as an error and counts as a failure.
Drain learns of the limit from a structured field or typed result, not by parsing the closing-reason text; a Grep of crates/athena-cli/src/drain.rs finds no parsing of the 'environment: usage limit' prefix.
CHANGELOG.md has an entry describing the usage-limit handling, the 'usage limit' stop reason and the status display.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
