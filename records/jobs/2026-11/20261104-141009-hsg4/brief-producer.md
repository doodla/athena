# Job 20261104-141009-hsg4: Throughput review: flag median slowdowns and queue a request

## Request (verbatim)

> Make the throughput review catch slowdowns and keep moving. The median time per landed job rose from 25.4 minutes (review of 29 Oct) to about 60 minutes (1 to 4 Nov) and nothing flagged it: the review proposes one design request and then waits until that request closes.
> 
> Files changed: the throughput review code and its tests.
> 
> Acceptance:
> - each review run compares the latest median minutes per landed job with the previous review's and names the stages whose minutes grew, even while a design request is open
> - when the median grew by more than a quarter, the review queues a request at high priority naming the stage that grew most and its measured minutes
> - a test where the median rose from 25 to 60 minutes with a design request open: the grown stages are named and a request is queued
> - a test where the median is flat: nothing is queued

## Reading (accepted by the user)

Today the throughput review (crates/athena-core/src/hestia.rs) queues one model-proposed design request and does nothing while it is open, so the 25.4 to 60 minute rise went unflagged. I read it as: every review run compares the latest median with the previous review's and names the stages that grew, even with a design request open. When the median grew by more than 25%, it also queues a high-priority request naming the biggest-growing stage and its measured minutes.

Assumptions:
- Goal and audience: the operator reading throughput reviews and the queue wants a slowdown flagged and acted on, not hidden behind an open design request.
- Code survey: read crates/athena-core/src/hestia.rs (throughput_due ~1236, throughput_review ~1438, open_throughput_request, throughput_request, throughput_followups, tests from ~3893), crates/athena-core/src/status.rs (Throughput::compute, landed_jobs, StageMinutes::named, median, THROUGHPUT_WINDOW) and crates/athena-data/src/records.rs (ThroughputReview, ThroughputTrigger, ThroughputProposal; AnyRecord::ThroughputReview validate and schema). The entry point is Athena::throughput_review, called from review_passes. It loads records, calls Throughput::compute, then throughput_due, then either notes that a request is open or asks the model. The median is measured directly from landed jobs' stage minutes, not through a stand-in.
- The previous review stores only window_median_minutes and landings, not per-stage medians. Plan: recompute the previous per-stage medians from landed_jobs(records) cut to the first `previous.landings` jobs, using the newest THROUGHPUT_WINDOW of them, and compare with the current Throughput::median_stages. This needs no record schema change, so records.rs, its schema and the AnyRecord round-trip test stay unchanged. If a producer finds this unworkable and adds a record field, it must update the schema and the round-trip test (check_record_file/AnyRecord).
- A new pure function (for example `stage_growth`) returns the stages whose median minutes grew, largest first. The deterministic slowdown request is queued by Athena code through the existing StewardRequest/self.queue path at Priority::High, without needing the model, and is queued regardless of an open Throughput design request. The review note and the review's CLI output line name the grown stages.
- The 25% threshold is a new constant (for example SLOWDOWN_RATIO = 1.25). The existing 20% WORSEN_RATIO trigger stays, so a review still runs between 20% and 25%, names the stages, and queues nothing.
- To avoid queuing a duplicate slowdown request on every run, a still-open request with a distinct title marker for slowdowns suppresses another one. Each comparison is against the previous review, so a persistent slowdown is flagged once per review.

## Acceptance criteria

1. [behavior] Test (run `cargo test -p athena-core hestia`): the previous review recorded a median of 25 minutes, the current median is 60, and an open request whose text has a line starting 'Throughput design: ' exists. The review result or note names every stage whose minutes grew, and a new request is queued at Priority::High. The test asserts the stage names and the queued request text.
2. [behavior] Test (run `cargo test -p athena-core hestia`): the median is flat against the previous review, so no request is queued and no stage is reported as grown. The test asserts that the queue holds no new request.
3. [behavior] Test (run `cargo test -p athena-core hestia`): a rise above 20% but not above 25% names the grown stages and queues no slowdown request. A rise just over 25% queues one.
4. [content] The queued slowdown request text, as built by the new code, names the stage that grew most and its measured median minutes (and the previous minutes) and carries priority high. Grep of the request builder in crates/athena-core/src/hestia.rs shows both values interpolated, and the test asserts them.
5. [behavior] Test: a review with an open design request still compares stage medians against the previous review and keeps the existing behavior of asking the model for no second proposal. Existing throughput tests still pass (`cargo test`).
6. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass. Changes are limited to the throughput review code and its tests, and no governance file changes.
7. [content] The module doc comment in hestia.rs, and docs/design/hestia.md if it describes the trigger, state the new comparison, the 25% rule and the stage naming. This matches outcome-check: the check is the slowdown being flagged, not the review merely running.

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/design/hestia.md (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the model prompt or its proposal schema, or the code-health and error reviews.
- Changing the throughput measurement itself (stage definitions, THROUGHPUT_WINDOW).
- Tuning the review cadence (REVIEW_EVERY) or the 20% trigger.
- Governance files (athena.toml, defaults.toml, workflows).

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the slowdown and flat cases, and cargo test and clippy stay clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check stage naming and queueing in tests with an open request, not only that the functions exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is a slowdown flagged and a request queued, not the review merely running.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in hestia.rs, with the previous stage medians derived from existing records and no schema change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The 25% rule is enforced in code and tested, not left as prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the rule for an outside reader.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
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
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

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

Test (run `cargo test -p athena-core hestia`): the previous review recorded a median of 25 minutes, the current median is 60, and an open request whose text has a line starting 'Throughput design: ' exists. The review result or note names every stage whose minutes grew, and a new request is queued at Priority::High. The test asserts the stage names and the queued request text.
Test (run `cargo test -p athena-core hestia`): the median is flat against the previous review, so no request is queued and no stage is reported as grown. The test asserts that the queue holds no new request.
Test (run `cargo test -p athena-core hestia`): a rise above 20% but not above 25% names the grown stages and queues no slowdown request. A rise just over 25% queues one.
The queued slowdown request text, as built by the new code, names the stage that grew most and its measured median minutes (and the previous minutes) and carries priority high. Grep of the request builder in crates/athena-core/src/hestia.rs shows both values interpolated, and the test asserts them.
Test: a review with an open design request still compares stage medians against the previous review and keeps the existing behavior of asking the model for no second proposal. Existing throughput tests still pass (`cargo test`).
`cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass. Changes are limited to the throughput review code and its tests, and no governance file changes.
The module doc comment in hestia.rs, and docs/design/hestia.md if it describes the trigger, state the new comparison, the 25% rule and the stage naming. This matches outcome-check: the check is the slowdown being flagged, not the review merely running.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
