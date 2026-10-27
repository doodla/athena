# Job 20261027-031444-60zg: Add Hestia error review and lineage-based scope guard counting

## Request (verbatim)

> Priority: urgent
> Give a role the duty of thinking about errors across jobs, not one at a time (Sri, 2026-10-27: "Is anyone looking at the errors and thinking about them?"). Today every failure gets an introspection with a cause and prevention, but nothing groups them by cause or acts on repeats; the operator catches patterns by hand, late (four planning-cap stops before a fix; five brde attempts before a stop). Add an error review to Hestia (docs/design/hestia.md): after each introspection is recorded, group it with earlier introspections by cause (the introspection's cause and class, matched by the existing class registry or by Jev's class answer). When a cause has occurred in two or more distinct jobs and no open or landed request cites it, queue one root-cause request at urgent priority through the shared steward request builder, citing the introspection ids, and show the open repeated causes with their counts in athena status. Also make the scope guard count attempts per line of work across retries and re-asks of the same request (follow previous_job and request text lineage), not per job id.
> Acceptance:
> - A test records two introspections with the same cause on different jobs and shows exactly one urgent root-cause request is queued citing both; a third does not queue another while that request is open.
> - A test shows athena status lists repeated causes with their counts and the request that addresses each.
> - A test shows the scope guard counts a retry and a re-ask of the same request as one line, and pauses at its threshold.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry; docs/design/hestia.md describes the error review.
> Files: crates/athena-core/src/hestia.rs, crates/athena-core/src/introspect.rs, crates/athena-core/src/scope_guard.rs, crates/athena-core/src/status.rs, docs/design/hestia.md, CHANGELOG.md

## Reading (accepted by the user)

Add an error review to Hestia that groups introspections by cause/class, queues one urgent root-cause request when a cause repeats across two or more distinct jobs without an open or landed request citing it, and shows repeated causes in athena status. Make the scope guard count attempts per line of work (previous_job and request text lineage). Audience: the operator (Sri) who catches repeats by hand today.

Assumptions:
- Grouping key is the introspection cause plus class, with the class registry or Jev's class answer resolving the class.
- The error review runs right after an introspection is recorded.
- The scope guard already has line grouping (group_lines, line_attempts); the work is to ensure counting follows previous_job and request text lineage, and to test it.
- No athena.toml or workflow change is needed.

## Acceptance criteria

1. [behavior] Test (cargo test): two introspections with the same cause on different jobs queue exactly one urgent root-cause request citing both introspection ids via the shared steward request builder; a third introspection with that cause does not queue another while that request is open or landed.
2. [behavior] Test (cargo test): a single-job repeat, or causes already cited by an open or landed request, queue nothing.
3. [behavior] Test (cargo test): athena status output lists each open repeated cause with its count and the request addressing it.
4. [behavior] Test (cargo test): the scope guard counts a retry (previous_job) and a re-ask of the same request text as one line of work and pauses at its threshold, not per job id.
5. [behavior] cargo fmt --check, cargo clippy with -D warnings, and cargo test all pass.
6. [content] CHANGELOG.md has an entry and docs/design/hestia.md describes the error review (grouping key, trigger, dedupe, status display).

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/design/hestia.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how introspections are written beyond what grouping needs
- Automatically fixing the root causes
- Changing scope guard thresholds

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests and clippy/fmt/test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check queued requests and status output.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Review is code-enforced, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Queued requests are root-cause requests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior verified by running tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 30 cases in 30 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Before the verifier is dispatched, a code check confirms that the producer's report exists, is non-empty, and is included in the verifier's input. If any acceptance criterion refers to the report's content, the job is held at handover when the report is absent, and is not sent to the verifier to be ...
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

Test (cargo test): two introspections with the same cause on different jobs queue exactly one urgent root-cause request citing both introspection ids via the shared steward request builder; a third introspection with that cause does not queue another while that request is open or landed.
Test (cargo test): a single-job repeat, or causes already cited by an open or landed request, queue nothing.
Test (cargo test): athena status output lists each open repeated cause with its count and the request addressing it.
Test (cargo test): the scope guard counts a retry (previous_job) and a re-ask of the same request text as one line of work and pauses at its threshold, not per job id.
cargo fmt --check, cargo clippy with -D warnings, and cargo test all pass.
CHANGELOG.md has an entry and docs/design/hestia.md describes the error review (grouping key, trigger, dedupe, status display).

## Report

End with the producer report as your structured output.
