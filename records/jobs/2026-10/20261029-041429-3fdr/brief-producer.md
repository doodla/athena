# Job 20261029-041429-3fdr: Make Athena's own writers avoid boundary words in requests and records

## Request (verbatim)

> Priority: high
> Athena's own writers produce text its boundary scan then refuses. On 2026-10-29 a steward proposal described a recurring duty with a boundary word for the person running Athena; approving it failed at admission because the request carried that word, so the proposal could never be worked. Every writer of text that later becomes a request or a record (the steward and its proposals, Hestia's error review and root-cause requests, the introspection writer) must check its text against the boundary word list before writing and use the plain alternative (for example 'the person running Athena').
> Acceptance:
> - A test feeds each writer a fixture whose source text contains a boundary word and shows the written proposal or request contains none and passes the admission scan.
> - A test shows approve-work on such a proposal succeeds.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/steward.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/introspect.rs, CHANGELOG.md

## Reading (accepted by the user)

The steward, Hestia's error review and root-cause requests, and the introspection writer must run their text through the boundary scan before writing. Any boundary word is replaced with a plain alternative such as 'the person running Athena', so written proposals and requests pass admission and approve-work succeeds. The audience is the person running Athena, who approves proposals, and the admission scan that judges them.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (boundary_words, boundary_fragments, boundary_scan, which matches whole words case-insensitively plus fragments and returns BoundaryHit line and term), crates/athena-core/src/work_decisions.rs (approve_work builds the request from the work item and goes through normal admission), crates/athena-core/src/steward.rs (ProposedChange::from_change, render, blind_spot_change), crates/athena-core/src/hestia.rs (root_cause_request) and crates/athena-core/src/introspect.rs. Admission runs check_boundary_and_repo_files in crates/athena-core/src/reading.rs.
- The fix is one shared sanitiser, built on boundary_scan and boundary_words, that rewrites boundary words to plain alternatives (the person running Athena for the first word in the list, and similar for the others). Each writer calls it before text is written; writers that cannot rewrite safely return an error that names the word.
- No new record or data file is added under crates/, so the records allow-list (check_record_file, AnyRecord) is not touched. The shared helper may live in one of the three listed files or in athena-context next to boundary_scan, which is an extra deliverable.
- Hestia's error review writer lives in hestia.rs, and the introspection writer in introspect.rs. Fixtures must build the boundary word from fragments so no tracked file contains it, because the repository policy scan fails on tracked files.
- This change makes writers refuse or rewrite more text but adds no new admission rule, so no replay refusal-rate criterion is needed: the admission scan itself is unchanged.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`. A test per writer (steward proposal, Hestia error review, Hestia root-cause request, introspection writer) feeds a fixture whose source text holds a boundary word (built from fragments) and asserts the written text has no hits from boundary_scan and says 'the person running Athena' or another plain alternative.
2. [behavior] Run `cargo test -p athena-core`. A test seeds a steward proposal from a source with a boundary word, calls approve_work, and asserts it succeeds and the submitted request passes check_boundary_and_repo_files.
3. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including `cargo test -p xtask --test policy`.
4. [content] CHANGELOG.md has an entry describing that writers now keep boundary words out of proposals and requests, with no boundary word in the entry itself.
5. [content] steward.rs, hestia.rs and introspect.rs each call the shared boundary check before writing text that becomes a request or record; a Grep of the diff shows the call in all three.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)

## Out of scope

- Changing the boundary word list or the admission scan's rules
- Changing athena.toml, defaults.toml or workflows
- Rewriting records already written

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Per-writer tests and an approve-work test; cargo test and clippy -D warnings must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the written text against boundary_scan and approve_work succeeding.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared check used by every writer, instead of fixing only the steward proposal that failed.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Tests enforce that each writer's output passes the scan.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog entry are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 35 cases in 35 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Have the producer and checks code emit typed stage events (an enum with Producer, Checks and so on) at each transition, including the send-back transition, and have the in-flight table record only those. Do not parse display strings. Add a test that runs the real drain path through checks failing an...
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

20 more qualifying classes are left out.

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

Run `cargo test -p athena-core`. A test per writer (steward proposal, Hestia error review, Hestia root-cause request, introspection writer) feeds a fixture whose source text holds a boundary word (built from fragments) and asserts the written text has no hits from boundary_scan and says 'the person running Athena' or another plain alternative.
Run `cargo test -p athena-core`. A test seeds a steward proposal from a source with a boundary word, calls approve_work, and asserts it succeeds and the submitted request passes check_boundary_and_repo_files.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass, including `cargo test -p xtask --test policy`.
CHANGELOG.md has an entry describing that writers now keep boundary words out of proposals and requests, with no boundary word in the entry itself.
steward.rs, hestia.rs and introspect.rs each call the shared boundary check before writing text that becomes a request or record; a Grep of the diff shows the call in all three.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
