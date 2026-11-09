# Job 20261027-184453-x7fn: Build plan groups from priority order so drain follows plan order

## Request (verbatim)

> Priority: high
> athena drain does not follow athena plan's order when groups are involved. On 2026-10-27 the plan's order began with three urgent items and a fourth urgent item, but drain dispatched a normal-priority item first, because it works plan groups in sequence and a normal item sat in an earlier group than the urgent ones. Groups exist to allow parallel work; they must not reorder work. Fix: build groups from the priority order (an item joins the earliest group after everything it depends on, taking items in plan order), so with one job at a time drain dispatches exactly in plan order, and a later group never holds an item of higher priority than an earlier group without a dependency reason.
> Acceptance:
> - A test plans a normal item and an urgent item with disjoint paths and no dependency, and shows drain with max_concurrency 1 dispatches the urgent item first.
> - A test shows a dependency still comes before its dependent whatever their priorities.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/plan.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Plan::build_with makes groups that can put a lower-priority item in an earlier group than a higher-priority one, so drain, which works groups in sequence, dispatches out of plan order. Rebuild groups from the final order: taking items in plan order, each joins the earliest group after all its dependencies' groups where it is path-disjoint with the members. Audience: operators who run `athena plan` and `athena drain` and expect them to agree.

Assumptions:
- Code survey: read crates/athena-core/src/plan.rs (Plan::build_with wave loop at ~L704-742 picks ready items of the best effective priority, then greedily groups by disjoint(); build_capped re-runs build_with on demoted copies; build_improving reorders `order` and regroups at ~L928-943) and crates/athena-cli/src/drain.rs (batches_for at ~L2916 only filters plan.groups in group order; drain_concurrent/run_serial work batch by batch). The entry point is the plan build used by Drain::prepare and `athena plan`. No record or data file is added, so no records validation allow-list or AnyRecord round-trip is affected. The property is measured directly: the order in which drain dispatches items, via the existing drain test harness, with max_concurrency 1.
- Root cause is in group construction, not in batches_for: groups must be derived from plan.order so flattening groups equals the order. The new rule: each item, in plan order, joins the earliest group that is after the groups of all its dependencies and where it is disjoint() with all members; otherwise it starts a new group. A group must never be placed before a group holding an earlier-ordered item that it cannot join, so flattened groups == order.
- Existing group tests in plan.rs (e.g. disjoint_independent_items_share_a_group_and_dependents_come_after, the ~L1747 test expecting [[b,a,c],[d]]) may need updating if the new grouping changes their expected output; update them only where the new rule justifies it.
- The improvement-turn regrouping in build_improving is kept consistent with the new rule; it is not redesigned.
- This change does not newly refuse any reading, so no refusal-rate criterion applies.

## Acceptance criteria

1. [behavior] A new test (in crates/athena-cli/src/drain.rs tests, via `cargo test`) plans a normal item and an urgent item with disjoint touched paths and no dependency, and shows drain with max_concurrency 1 dispatches the urgent item first.
2. [behavior] A new test in plan.rs (`cargo test -p athena-core`) shows a dependency's group comes before its dependent's group whatever their priorities, including a normal-priority dependency of an urgent item and a case where the dependent has a higher priority than its dependency.
3. [behavior] A new plan.rs test shows that for a mixed plan the flattened Plan::groups equals Plan::order, and no later group holds an item of higher priority than an earlier group unless a dependency explains it.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
5. [content] crates/athena-core/src/plan.rs builds groups from the ordered items (earliest group after all dependencies, plan order), and its doc comments describing groups and drain order match the new behavior; crates/athena-cli/src/drain.rs docs are updated where they describe group order.
6. [content] CHANGELOG.md has a new entry describing that drain now dispatches in plan order when groups are involved (git diff shows the added lines).

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing priority, tier-cap, or effective-priority rules
- Changing how the Steward improvement-turn promotion works
- Changing the parallel scheduling or landing check logic beyond group construction
- athena.toml, defaults.toml and workflow files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for both behaviors; fmt, clippy -D warnings and cargo test required.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix group construction rather than reordering in batches_for.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the dispatch order and group/order agreement.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 12 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check for size-reduction jobs. Before the reading is accepted, Athena measures each named file's production lines and its longest function in the current snapshot and puts the measured numbers into the reading. The reading must size the slice to reach the limit. The producer must also run...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

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

- path: <state>/jobs/20261027-184453-x7fn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-184453-x7fn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test (in crates/athena-cli/src/drain.rs tests, via `cargo test`) plans a normal item and an urgent item with disjoint touched paths and no dependency, and shows drain with max_concurrency 1 dispatches the urgent item first.
A new test in plan.rs (`cargo test -p athena-core`) shows a dependency's group comes before its dependent's group whatever their priorities, including a normal-priority dependency of an urgent item and a case where the dependent has a higher priority than its dependency.
A new plan.rs test shows that for a mixed plan the flattened Plan::groups equals Plan::order, and no later group holds an item of higher priority than an earlier group unless a dependency explains it.
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
crates/athena-core/src/plan.rs builds groups from the ordered items (earliest group after all dependencies, plan order), and its doc comments describing groups and drain order match the new behavior; crates/athena-cli/src/drain.rs docs are updated where they describe group order.
CHANGELOG.md has a new entry describing that drain now dispatches in plan order when groups are involved (git diff shows the added lines).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
