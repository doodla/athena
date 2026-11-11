# Job 20261111-105048-n7bm: Add role-need measure to athena status for OLY-9-11 (retry)

## Request (verbatim)

> try again on 20261029-233051-jq6k

## Reading (accepted by the user)

Retry of job 20261029-233051-jq6k: add a role-need measure in athena-core, printed by `athena status` as a `role need` line and recorded as OLY-9-11's mechanism and measure. It reads each role's runs from where they are recorded (job invocations plus steward_runs, audience_runs and auditor_runs), so steward, audience and auditor are never flagged for retirement. Audience: Athena owner and the Hestia spec pass.

Assumptions:
- Code survey: crates/athena-core/src/store.rs (Records at ~239-251 keeps steward_runs, audience_runs, auditor_runs apart from request_jobs invocations; built at ~421-456), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at ~305, checked at ~1597), crates/athena-core/src/hestia.rs (~1166, ~1230 validates that a measure ends with a known `status: <label>`), crates/athena-cli/src/display.rs (status rendering). The measure is computed directly from Records.
- Existing code: extend crates/athena-core/src/status.rs (StatusView, STATUS_MEASURE_LABELS) and crates/athena-cli/src/display.rs; follow the class_trend pattern of recent windows; reuse the Records accessors in crates/athena-core/src/store.rs without changing them.
- Job 20261029-233051-jq6k findings addressed: (blocker) retire rule blind to steward/audience/auditor: compute now reads runs from job invocations AND the three separate run lists; a role with no readable source is shown as `not measured`, never `retire`. (minor) compute untested: add tests over Records covering outcome mapping (Failed/Rejected/TimedOut count as failed; Declined/Closed excluded), ordering and role visibility. (minor) unbounded class history: role-needed uses only the newest N closed jobs per class. (nit) display test renders a StatusView through status(), not the section helper.
- Files from job 20261029-233051-jq6k's candidate: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/role_need.rs, crates/athena-core/src/status.rs and docs/spec.toml are all kept as deliverables and redone from the base with the fixes above.
- Term role need: per role, a status of needed (recent failure concentration in a class), in use, retire candidate (no runs in a recent window across all run sources) or not measured. Advisory only; no automatic role changes.
- No new record or data file is added under crates/, so the records validation allow-list and AnyRecord are unaffected. No prompts change, so the token budgets are unaffected. No governance files change.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-9-11 has mechanism naming an existing path crates/athena-core/src/role_need.rs, owner Athena, and a measure ending `status: role need`; the label `role need` is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
2. [behavior] A test over Records built with steward_runs, audience_runs and auditor_runs (and few or no job invocations for them) shows RoleNeed::compute does not flag steward, audience or auditor for retirement; replaying over a real-shape fixture lists them as not flagged. Run `cargo test -p athena-core role_need`.
3. [behavior] Tests in role_need.rs show a role truly unused in the recent window, with its source measured, is flagged retire; a role with no recorded source is shown `not measured`; a class with repeated recent failures yields role-needed; thin history yields nothing; outcome mapping (Failed/Rejected/TimedOut failed, Declined/Closed excluded) and only the recent window per class are honoured. Tests fail if compute/analyze is stubbed. Run `cargo test -p athena-core role_need`.
4. [behavior] A display test in crates/athena-cli/src/display.rs renders a StatusView through status() and shows a line starting `role need` for every role in ALL_ROLES (including not measured ones), and no candidate advice when none. Run `cargo test -p athena-cli`.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` (including the OLY spec test in xtask/tests/spec.rs) pass.
6. [content] The diff changes only docs/spec.toml, crates/athena-core/src/role_need.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs and CHANGELOG.md (entry added); athena.toml, defaults.toml and prompts are unchanged.

## Expected deliverables

- crates/athena-core/src/role_need.rs (create)
- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- OLY-9-12 (value of a new role against a simpler alternative)
- Automatically creating or retiring roles
- Changes to prompts, athena.toml or defaults.toml
- Changing how store.rs records steward, audience or auditor runs

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the replayed behavior (steward, audience, auditor not flagged), not file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests fail without the mechanism; fmt, clippy and test required.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix addresses the multi-source blindness, not just the three roles.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261029-233051-jq6k raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> blocker (crates/athena-core/src/role_need.rs RoleNeed::compute (ALL_ROLES and the j.invocations scan)): The retire rule always flags the steward, audience and auditor, which are in active use, because it never sees their runs. compute takes roles only from `records.request_jobs()[..].invocations`. store.rs:239-251 says steward, audience and auditor invocations are "kept apart from `jobs`" in `steward_runs`, `audience_runs` and `auditor_runs`. In the evidence snapshot every steward invocation is a standalone `01-steward.json` with no outcome: 300 files, none at position 02-09. The most recent are 20261111-061048-8bdq and 20261111-055051-pxja. Audience runs also appear repeatedly on Nov 10-11. Request jobs such as 20261110-191311-r241 hold only reader, producer and verifier invocations, plus planner and critic in some jobs. Given more than eight closed jobs, `athena status` will therefore always print "retire candidate: steward/audience/auditor", which is false advice that will appear in every status output. A better rule counts each role's runs in a time window across both the job invocations and the separate run lists, or restricts retirement to roles that run inside jobs. A RoleNeed::compute test over Records with a steward run would catch this.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 18 cases in 18 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: When a criterion says a test must assert a property of rendered output, the producer's brief should require two things before the job is done. The test must run against the real rendered page or the real source data. And it must be shown to fail on the known bad input, here the current duplicated te...
- 13 cases in 13 jobs:
> Title: design-ignores-existing-code-caught-only-at-final-verdict
> Prevention: Add a required design-job step before the note is finished: every claim about what the code or host does (a refusal, a conflict, a race outcome) must name the code path or ref semantics that produces it, and a critic must check each such claim against the landing code. Each rule the note changes mus...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...

34 more qualifying classes are left out.

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

- path: <state>/jobs/20261111-105048-n7bm/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-105048-n7bm/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml entry OLY-9-11 has mechanism naming an existing path crates/athena-core/src/role_need.rs, owner Athena, and a measure ending `status: role need`; the label `role need` is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
A test over Records built with steward_runs, audience_runs and auditor_runs (and few or no job invocations for them) shows RoleNeed::compute does not flag steward, audience or auditor for retirement; replaying over a real-shape fixture lists them as not flagged. Run `cargo test -p athena-core role_need`.
Tests in role_need.rs show a role truly unused in the recent window, with its source measured, is flagged retire; a role with no recorded source is shown `not measured`; a class with repeated recent failures yields role-needed; thin history yields nothing; outcome mapping (Failed/Rejected/TimedOut failed, Declined/Closed excluded) and only the recent window per class are honoured. Tests fail if compute/analyze is stubbed. Run `cargo test -p athena-core role_need`.
A display test in crates/athena-cli/src/display.rs renders a StatusView through status() and shows a line starting `role need` for every role in ALL_ROLES (including not measured ones), and no candidate advice when none. Run `cargo test -p athena-cli`.
`cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` (including the OLY spec test in xtask/tests/spec.rs) pass.
The diff changes only docs/spec.toml, crates/athena-core/src/role_need.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs and CHANGELOG.md (entry added); athena.toml, defaults.toml and prompts are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
