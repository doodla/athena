# Job 20261105-234147-88tw: Add bug-evidence default and criterion for bug requests

## Request (verbatim)

> Priority: high
> 
> Add a default expectation for bug, wrong-output and regression requests: the reading adds a criterion that the change names the root cause, shows it from data the job can read (records, code path, a failing case), and shows why the wrong output or the metric change follows from that cause, before any fix. A guess labelled as an estimate does not meet it. Add a test that a bug request's reading gets this criterion.
> 
> Introspections that record the repeated cause: 20261104-054700-e6js, 20261105-051344-q9fv, 20261105-101121-qw9r, 20261105-205835-hk09
> 
> Acceptance:
> - The change is made as described: Bug and slowdown jobs fix or explain the symptom without a data-backed root cause and are rejected at the final verdict (3 jobs)
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-11/20261104-054711-kb90.json

## Reading (accepted by the user)

Add a "bug-evidence" default to defaults.toml for bug, wrong-output and regression requests, and make the reader's pipeline ensure such a reading carries a criterion that the change names the root cause, shows it from readable data, and shows why the symptom follows, before any fix. Add a test that a bug request's reading gets the criterion. Audience: the verifier who rejects data-less root-cause claims at the final verdict.

Assumptions:
- Code survey: crates/athena-core/src/reading.rs build_reading (about line 685-848) builds the Reading from reader output and already calls ensure_changelog (line 281) to add a deliverable and a defaults_applied entry mechanically; crates/athena-data/src/defaults.rs parses defaults.toml (REPO_DEFAULTS) and render() shows only each default's first sentence; no record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched.
- Design: add a new default id `bug-evidence` (applies_to ["*"], source cites the four introspections) whose first sentence is short, rather than editing `root-cause`, whose text already says to fix the root cause. Add an analogous mechanical step beside ensure_changelog that, for a change reading from a bug, wrong-output, regression or slowdown request, appends the root-cause-evidence criterion and a defaults_applied entry once. The request keyword detection must be simple and tested.
- The reader prompt has 3 tokens of headroom, so the new default's rendered line may overflow its 2000-token budget; the producer keeps the first sentence minimal and, only if the budget check fails, raises the reader budget in athena.toml.
- The cited file records/failure-classes/2026-11/20261104-054711-kb90.json is evidence only and records/ never changes.
- Existing tests must keep passing, including defaults.rs tests that assert rendered lines; no unrelated behavior changes.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] `bug-evidence` applying to bug, wrong-output and regression requests. It requires that the change names the root cause, shows it from data the job can read (records, code path, a failing case) and shows why the wrong output or metric change follows, before any fix. It says a guess labelled as an estimate does not meet it, and it cites the four introspection ids in its source.
2. [content] The reading builder in crates/athena-core/src/reading.rs adds the root-cause-evidence criterion and a defaults_applied entry exactly once for bug, wrong-output, regression and slowdown requests, and adds nothing for other requests.
3. [behavior] A new unit test in crates/athena-core/src/reading.rs builds a reading from a bug request and asserts the criterion is present. A second test asserts a non-bug request gets none and that running the step twice adds it once. Run `cargo test -p athena-core` to check.
4. [behavior] A defaults.rs test asserts the `bug-evidence` default parses from the repo defaults.toml and its first-sentence line renders. Run `cargo test -p athena-data` to check.
5. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass across the workspace, with existing tests unchanged in outcome and prompt token budgets still within limits.
6. [content] Governance files: defaults.toml is changed and named here. athena.toml changes only if the reader prompt budget check requires it, and then only that budget value.
7. [content] Standing defaults hold. outcome-check: a newcomer reading the new default and the criterion text understands what evidence is required. root-cause: the change description names why bug jobs were being rejected. no-empty-success and thrash-stop apply unchanged.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/defaults.rs (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-context/src/briefs.rs (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Changing records/ or any owner file, including records/failure-classes/2026-11/20261104-054711-kb90.json
- Changing the verifier's final-verdict rejection logic
- Rewording the existing root-cause default
- Fixing the individual bug and slowdown jobs that were rejected

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the criterion and the default; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The criterion is added by code in the reading builder rather than left to prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is that readings for bug requests carried no criterion demanding data-backed root-cause evidence.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show the reading gains the criterion.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One default plus one builder step.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The default and criterion text are plain for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criterion text must be clear to a newcomer reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier runs the tests.
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
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

29 more qualifying classes are left out.

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

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-234147-88tw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-234147-88tw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

defaults.toml has a new [[default]] `bug-evidence` applying to bug, wrong-output and regression requests. It requires that the change names the root cause, shows it from data the job can read (records, code path, a failing case) and shows why the wrong output or metric change follows, before any fix. It says a guess labelled as an estimate does not meet it, and it cites the four introspection ids in its source.
The reading builder in crates/athena-core/src/reading.rs adds the root-cause-evidence criterion and a defaults_applied entry exactly once for bug, wrong-output, regression and slowdown requests, and adds nothing for other requests.
A new unit test in crates/athena-core/src/reading.rs builds a reading from a bug request and asserts the criterion is present. A second test asserts a non-bug request gets none and that running the step twice adds it once. Run `cargo test -p athena-core` to check.
A defaults.rs test asserts the `bug-evidence` default parses from the repo defaults.toml and its first-sentence line renders. Run `cargo test -p athena-data` to check.
`cargo fmt --check`, `cargo clippy` and `cargo test` pass across the workspace, with existing tests unchanged in outcome and prompt token budgets still within limits.
Governance files: defaults.toml is changed and named here. athena.toml changes only if the reader prompt budget check requires it, and then only that budget value.
Standing defaults hold. outcome-check: a newcomer reading the new default and the criterion text understands what evidence is required. root-cause: the change description names why bug jobs were being rejected. no-empty-success and thrash-stop apply unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
