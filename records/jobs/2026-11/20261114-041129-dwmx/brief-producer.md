# Job 20261114-041129-dwmx: Rank Athena self-fix requests ahead of feature work in plan and drain

## Request (verbatim)

> Requested by Sri (the owner). Rank requests that fix Athena itself (from the steward or Hestia: root-cause, repeated-failure, blind-spot and gate-review requests) ahead of feature and content work of the same priority in athena plan and drain. Website and study content requests from the steward are not fixes for this rule. Today 52 steward requests filed in the week to 14 Nov never ran because older feature items came first.

## Reading (accepted by the user)

Within the same effective priority tier, plan and drain order requests that fix Athena itself (steward or Hestia root-cause, repeated-failure, blind-spot and gate-review items) before feature and content items. Steward website and study content requests do not count as fixes. Goal: the steward's filed requests stop starving behind older feature items. Audience: the owner, who runs `athena plan` and `athena drain`.

Assumptions:
- Code survey: crates/athena-core/src/plan.rs (Plan::build_with picks the ready items with the best effective priority, then sorts them by `(e.rung, age)` at about line 783; PlanEntry struct and PlanEntry::bare; module doc 'Order' paragraph; proposal_priority), crates/athena-core/src/reading/topics.rs (plan_entry builds entries from JobRecords), crates/athena-cli/src/drain.rs (around line 4340 builds the entry and finds the approved work item through work_decisions::decision_of; calls proposal_priority), crates/athena-data/src/records/work.rs (WorkItem has role, kind BlindSpot|RepeatedFailure|Proposal, key, title, source; WorkItemKind). The entry point is `athena plan` / `athena drain` calling Plan::build_with. No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are untouched; any new PlanEntry field is serde-skipped when false so existing serialized plans are unchanged. Order is measured directly by the plan order vector in unit tests, not through a stand-in.
- Term ladder: 'tier' means a priority level (urgent, high, normal, low); 'rung' means the requirement ladder rung used as the second sort key in Plan::build_with.
- Interpretation: a request is a fix when it comes from an approved work item whose role is the steward or Hestia and whose kind is BlindSpot or RepeatedFailure, or whose key or title marks a root-cause or gate-review fix. A steward Proposal that touches only website or studies/ content is not a fix. The new key is applied only inside the best effective priority tier, so an explicit priority record, Priority: line or urgent/high cap always wins over it. Fix items sort by (not-fix, rung, age), so rung then age order is preserved within each group.
- Docs: the 'Order' paragraph in the plan.rs module doc is updated. A decision recording the new rule is appended to docs/decisions.md as a new entry with a Provenance bullet; no existing entry is edited. athena.toml, defaults.toml and workflows are not changed (Consumers: none found).
- Previous-result check: no earlier job is named in the request, so there is nothing to keep or exclude.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/plan.rs builds a plan with an older normal feature entry and a newer normal fix entry that are both ready; the plan order puts the fix entry first. Check by running `cargo test -p athena-core`.
2. [behavior] A unit test shows an urgent or high feature item still comes before a normal fix item, and an item that depends on another still comes after it, so the fix flag never overrides priority or dependencies. Check by running `cargo test -p athena-core`.
3. [behavior] A unit test shows a steward request for website or studies content is not flagged as a fix and keeps its age-ordered place behind older items of the same tier. Check by running `cargo test -p athena-cli`.
4. [behavior] A drain test in crates/athena-cli/tests/drain.rs (or a unit test in crates/athena-cli/src/drain.rs) has an approved steward blind-spot or repeated-failure work item and an older feature request, both normal priority. The drain dispatches the steward job first. Check by running `cargo test -p athena-cli`.
5. [content] The plan.rs module doc 'Order' paragraph describes the fix-first rule as applying inside a priority tier. The diff shows Plan::build_with changing the `ready.sort_by_key` key to place fix entries before others.
6. [content] docs/decisions.md gains a new appended entry stating the rule, with a Provenance bullet citing the owner's request. No existing entry changes. CHANGELOG.md records the job.
7. [behavior] `cargo test`, `cargo clippy` and `cargo fmt --check` pass on the final tree, using Athena's recorded checks on the same tree.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/reading/topics.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing how steward or Hestia raise or classify work items
- Changing priority levels, urgent/high caps, or the `athena priority` command
- Editing existing entries in docs/decisions.md or anything under records/
- Changes to athena.toml, defaults.toml or CI workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and drain tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the plan order and the dispatch order, not that a field exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The sort key in Plan::build_with is changed, not individual items re-prioritised.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in the sort key and covered by tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 21 cases in 21 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: A code check at reading acceptance should require that any new refusal rule is replayed against the recorded criteria of the failed jobs the request cites. It should fail if the named instance, here the criterion of job 20261102-005332-8gkv, is not refused. A second check should compare the reading'...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

35 more qualifying classes are left out.

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

## Run the existing tests that read what you change

Before you report, name each output your change alters: status, plan, display or any other printed text. Grep `tests/` and the `#[cfg(test)]` modules for a distinctive line of that output and for the function that prints it. Run each test file found with `cargo test -p <crate> --test <name>` (for example `cargo test -p athena-cli --test mirrors`), or `cargo test -p <crate> <module>` for a unit test. Then run the whole crate with `cargo test -p <crate>`. A section printed unconditionally breaks a test that compares whole output, so look for those first. Cite each command, and the name and assertion of any failing test, in your `criteria_check` evidence.

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

- path: <state>/jobs/20261114-041129-dwmx/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-041129-dwmx/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/plan.rs builds a plan with an older normal feature entry and a newer normal fix entry that are both ready; the plan order puts the fix entry first. Check by running `cargo test -p athena-core`.
A unit test shows an urgent or high feature item still comes before a normal fix item, and an item that depends on another still comes after it, so the fix flag never overrides priority or dependencies. Check by running `cargo test -p athena-core`.
A unit test shows a steward request for website or studies content is not flagged as a fix and keeps its age-ordered place behind older items of the same tier. Check by running `cargo test -p athena-cli`.
A drain test in crates/athena-cli/tests/drain.rs (or a unit test in crates/athena-cli/src/drain.rs) has an approved steward blind-spot or repeated-failure work item and an older feature request, both normal priority. The drain dispatches the steward job first. Check by running `cargo test -p athena-cli`.
The plan.rs module doc 'Order' paragraph describes the fix-first rule as applying inside a priority tier. The diff shows Plan::build_with changing the `ready.sort_by_key` key to place fix entries before others.
docs/decisions.md gains a new appended entry stating the rule, with a Provenance bullet citing the owner's request. No existing entry changes. CHANGELOG.md records the job.
`cargo test`, `cargo clippy` and `cargo fmt --check` pass on the final tree, using Athena's recorded checks on the same tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
