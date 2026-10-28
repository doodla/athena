# Job 20261027-160633-t5sh: Treat a model usage limit as a hold, not a job failure

## Request (verbatim)

> Priority: urgent
> Treat a model usage limit as the environment, not a job failure. On 2026-10-27 the verifier call for job 20261027-123701-77zh returned in 2 seconds with "You've hit your weekly limit · resets Oct 29, 6pm (UTC)" and no usage; Athena recorded an error invocation with measured tokens but no usage for the requested model, its own gate refused that record, the job ended with no outcome, and drain counted it as a failure twice and stopped. Fix: recognise a usage-limit reply from the driver (the limit text and its reset time), record the invocation as limited with zero cost (so the gate accepts it), keep the job and its candidate for retry, and have drain stop with a 'usage limit' stop reason naming the reset time instead of 'repeated failure'. athena status shows the limit and its reset time while it holds.
> Acceptance:
> - A test with a fake driver reply carrying the limit text shows the invocation record passes the gate, the job stays retryable with its candidate kept, and no failure is counted.
> - A drain test shows the stop reason 'usage limit' with the reset time.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-agent/src/claude.rs, crates/athena-core/src/run/mod.rs, crates/athena-cli/src/drain.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

Recognise the driver's usage-limit reply (limit text plus reset time), record the invocation as limited at zero cost so the gate accepts it, keep the job and its candidate retryable, and make drain stop with 'usage limit' naming the reset time. Job 20261027-123701-77zh, which hit the limit, is retried by this fix and is not re-done; its site-rating work is out of scope.

Assumptions:
- Code survey: read crates/athena-agent/src/claude.rs (the driver reply parse and the invocation record built from it), crates/athena-cli/src/drain.rs (Stop::RepeatedFailure maps to 'repeated failure' near line 504; the drain loop counts failures), crates/athena-core/src/run/mod.rs (run pipeline, the verifier call and the gate on invocation records) and crates/athena-core/src/status.rs. No new record or data file under crates/ is added; if the invocation record gains a 'limited' state, the existing records validation and round-trip test must still accept it.
- Job 20261027-123701-77zh (site rating fixes) never got a verdict and changed no files; its candidate is kept and retried, not dropped. Its own deliverables (xtask/src/site.rs, studies/, docs/design/explainer-site.md) are not part of this change.
- The audience is Athena maintainers running drain; they judge the change by the stop reason, status output and a retryable job.
- The limit text has the form "You've hit your weekly limit · resets Oct 29, 6pm (UTC)"; the parser tolerates other limit kinds (session, daily) that carry the same 'limit' and 'resets' wording.

## Acceptance criteria

1. [behavior] Run `cargo test`: a claude.rs test feeds a fake driver reply with the text "You've hit your weekly limit · resets Oct 29, 6pm (UTC)" and no usage, and asserts the reply is recognised as a usage limit with the reset time parsed.
2. [behavior] Run `cargo test`: a run-level test with a fake driver returning the limit reply asserts the invocation record is marked limited with zero cost, passes the gate, the job stays retryable with its candidate kept, and no failure is counted.
3. [behavior] Run `cargo test`: a drain test with a job hitting the limit asserts drain stops with stop reason 'usage limit' that names the reset time, not 'repeated failure', and the failure counter is unchanged.
4. [behavior] Run `cargo test`: a status.rs test asserts `athena status` output shows the usage limit and its reset time while the limit holds, and shows neither once the reset time has passed.
5. [content] A genuine driver error without limit text still records as an error and counts as a failure, shown by an existing or new test, so ordinary failure handling is unchanged.
6. [content] CHANGELOG.md has an entry describing the usage-limit handling, with the 'usage limit' stop reason and the status display.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- The site-rating changes of job 20261027-123701-77zh (xtask/src/site.rs, studies/, docs/design/explainer-site.md)
- Automatic waiting or sleeping until the reset time
- Changing the gate's rules for other records
- Records under records/, athena.toml, defaults.toml and workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-driver, drain and status tests plus fmt, clippy -D warnings and cargo test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Handles the limit reply at the driver and record level rather than patching drain's failure count alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise the limit reply, gate, retry and stop reason.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The gate accepts the limited record and drain enforces the stop through code.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A limited job is never counted as success; it stays retryable.
- thrash-stop: If the same item fails twice in a loop, the loop stops and reports thrashing. Prose-only: no loop code detects it yet. Here: A limit hit stops drain at once with its own reason, rather than as a repeated failure.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry and messages are written for an outside reader.
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

Run `cargo test`: a claude.rs test feeds a fake driver reply with the text "You've hit your weekly limit · resets Oct 29, 6pm (UTC)" and no usage, and asserts the reply is recognised as a usage limit with the reset time parsed.
Run `cargo test`: a run-level test with a fake driver returning the limit reply asserts the invocation record is marked limited with zero cost, passes the gate, the job stays retryable with its candidate kept, and no failure is counted.
Run `cargo test`: a drain test with a job hitting the limit asserts drain stops with stop reason 'usage limit' that names the reset time, not 'repeated failure', and the failure counter is unchanged.
Run `cargo test`: a status.rs test asserts `athena status` output shows the usage limit and its reset time while the limit holds, and shows neither once the reset time has passed.
A genuine driver error without limit text still records as an error and counts as a failure, shown by an existing or new test, so ordinary failure handling is unchanged.
CHANGELOG.md has an entry describing the usage-limit handling, with the 'usage limit' stop reason and the status display.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
