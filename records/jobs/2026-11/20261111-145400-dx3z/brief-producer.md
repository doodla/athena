# Job 20261111-145400-dx3z: Make usage measures see every role's runs; replay-test against active

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The retire rule was written against one data source, the per-job invocation list. Nothing required i
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: usage-measure-blind-to-roles-whose-runs-are-not-in-job-records (20261111-105030-a2bc)
> Cause: The retire rule was written against one data source, the per-job invocation list. Nothing required it to be run on the real records, or compared with known facts such as the steward, audience and auditor being active, before the job was jud...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261111-105017-5k6m (job 20261029-233051-jq6k); prevention it proposed: Any job that adds a usage, need or retire measure must replay it over the real records before it is judged. The replay must be checked against a list of roles known to be active. A role whose runs are not in the measured source must be show...
> Introspection 20261111-122618-q1ha (job 20261029-233058-fveq); prevention it proposed: Add a code check that every new status or measure stage is run over the real records and must give both outcomes: it must not be true for all items or false for all items, unless a stated reason is recorded. It must also be compared with kn...
> Introspection 20261111-145341-v9p4 (job 20261111-122637-aawj); prevention it proposed: Add a standing code check that runs every role-usage or need measure over the real job and non-job run records. It must fail when any role the records show as active (steward, audience, auditor) would be flagged for retirement. Make the all...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the retire rule (OLY-9-11) read only the per-job invocation list, so the steward, audience and auditor, whose runs live in separate Records lists, looked unused; nothing told the author or checked the measure against known-active roles. Retry of jobs 20261029-233051-jq6k, 20261029-233058-fveq and 20261111-122637-aawj: keep the all_invocations source and on-demand handling, and add the missing information (system facts, field doc) and a class-level replay test.

Assumptions:
- Goal and audience: maintainers writing or judging usage/need measures; they must be told where each role's runs are recorded and have a test that fails when a measure misses a known-active role.
- Code survey: read crates/athena-core/src/store.rs (JobRecords::invocations doc at line ~32, Records::all_invocations at ~524, test all_invocations_covers_every_loaded_invocation at ~713), crates/athena-core/src/role_need.rs (compute, analyze, source-scan test at ~433), status.rs and crates/athena-cli/src/display.rs, docs/spec.toml OLY-9-11. No new record or data file is added, so check_record_file/AnyRecord are not touched. The property is measured directly by running RoleNeed::compute over Records built via Records::from_parts.
- Existing code: crates/athena-core/src/store.rs, crates/athena-core/src/role_need.rs, docs/system-facts.md and docs/spec.toml are extended, not recreated; the criteria reuse Records::all_invocations and RoleNeed::compute.
- Job 20261029-233051-jq6k (rejected: steward/audience/auditor blind, no compute test, unwindowed class history, display test bypassing status()): addressed by the current role_need.rs, which reads all_invocations, is windowed, and treats on-demand roles as not measured; its files role_need.rs, status.rs, lib.rs, display.rs, spec.toml, CHANGELOG.md are kept as deliverables.
- Job 20261029-233058-fveq (rejected: improvement_cycle evaluated stage always true, doc comment misplaced, loose chain): its introspection.rs, status_build_measurement.rs, status.rs work is a different requirement (OLY-10-02) and is out of scope here; not changed, not reverted.
- Job 20261111-122637-aawj (rejected: no replay vs known-active roles, auditor flagged, opt-in accessor undocumented, dropped/stale jobs counted, extra files): kept store.rs, role_need.rs, status.rs, lib.rs, display.rs, spec.toml, CHANGELOG.md, status_role_need.rs, xtask/shard-timings.txt; the remaining findings are fixed here by documentation in system-facts, a replay test with a known-active role list, and excluding dropped/stale jobs from the window.
- No refusal rule is added; the fix gives information (docs, field docs) and a test, so no share of past readings is refused (expected 0% of newly refused readings).
- Environment: none; no CI, push or records access.

## Acceptance criteria

1. [content] docs/system-facts.md has a section stating that steward, audience and auditor runs are stored outside a job's invocations and that any usage/need/retire measure must read Records::all_invocations and be replayed over real records (Grep for 'all_invocations' in docs/system-facts.md).
2. [content] JobRecords::invocations in crates/athena-core/src/store.rs carries a doc comment naming the steward, audience and auditor as absent and pointing to Records::all_invocations.
3. [behavior] Run `cargo test -p athena-core role_need`: a new test builds Records via from_parts with a standalone steward, audience and auditor run (and none of them in any job's invocations) and asserts none is a retire candidate and each is reported active or not measured; it fails if compute reads job.invocations only, and passes with the change.
4. [behavior] Run `cargo test -p athena-core role_need`: a test over a list of known-active roles asserts the measure yields both outcomes on a mixed record set (some role flagged, some not) and never flags a known-active role.
5. [behavior] Run `cargo test -p athena-core`: a test shows dropped, stale and abandoned jobs do not enter the closed-job window or the class totals; the existing all_invocations_covers_every_loaded_invocation test still passes.
6. [behavior] Run `cargo test -p athena-cli status_role_need`: rendering through status() prints the role need line only when a candidate exists and never names the auditor, steward or audience as a candidate.
7. [content] CHANGELOG.md has an entry naming the root cause (usage measure read a partial run source) and the fix; docs/spec.toml OLY-9-11 mechanism and measure still end with `status: role need`.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Expected deliverables

- docs/system-facts.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/role_need.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_role_need.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)

## Out of scope

- OLY-10-02 improvement_cycle in crates/athena-data/src/records/introspection.rs and crates/athena-cli/tests/status_build_measurement.rs (job 20261029-233058-fveq work, neither changed nor reverted)
- Automatically creating or retiring roles
- OLY-9-12 (value of a new role against a simpler alternative)
- A generic refusal rule or gate on new measures
- Changes to prompts, athena.toml or defaults.toml

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that known-active roles are never flagged, via a test over records.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run named cargo tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test pass criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the source accessor and its documentation, not one measure.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

- path: <state>/jobs/20261111-145400-dx3z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-145400-dx3z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/system-facts.md has a section stating that steward, audience and auditor runs are stored outside a job's invocations and that any usage/need/retire measure must read Records::all_invocations and be replayed over real records (Grep for 'all_invocations' in docs/system-facts.md).
JobRecords::invocations in crates/athena-core/src/store.rs carries a doc comment naming the steward, audience and auditor as absent and pointing to Records::all_invocations.
Run `cargo test -p athena-core role_need`: a new test builds Records via from_parts with a standalone steward, audience and auditor run (and none of them in any job's invocations) and asserts none is a retire candidate and each is reported active or not measured; it fails if compute reads job.invocations only, and passes with the change.
Run `cargo test -p athena-core role_need`: a test over a list of known-active roles asserts the measure yields both outcomes on a mixed record set (some role flagged, some not) and never flags a known-active role.
Run `cargo test -p athena-core`: a test shows dropped, stale and abandoned jobs do not enter the closed-job window or the class totals; the existing all_invocations_covers_every_loaded_invocation test still passes.
Run `cargo test -p athena-cli status_role_need`: rendering through status() prints the role need line only when a candidate exists and never names the auditor, steward or audience as a candidate.
CHANGELOG.md has an entry naming the root cause (usage measure read a partial run source) and the fix; docs/spec.toml OLY-9-11 mechanism and measure still end with `status: role need`.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`; all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
