# Job 20261111-122637-aawj: Make role-usage measures count all run sources, with a records test

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
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261111-105017-5k6m (job 20261029-233051-jq6k); prevention it proposed: Any job that adds a usage, need or retire measure must replay it over the real records before it is judged. The replay must be checked against a list of roles known to be active. A role whose runs are not in the measured source must be show...
> Introspection 20261111-122618-q1ha (job 20261029-233058-fveq); prevention it proposed: Add a code check that every new status or measure stage is run over the real records and must give both outcomes: it must not be true for all items or false for all items, unless a stated reason is recorded. It must also be compared with kn...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Fixes the root cause of jobs 20261029-233051-jq6k and 20261029-233058-fveq: a usage/retire measure read only per-job invocations and never saw steward, audience and auditor runs kept in separate lists. Adds one Records accessor over every run source, builds the OLY-9-11 role-need measure on it, and tests it over real-shaped Records with known-active roles.

Assumptions:
- Goal/audience: Athena maintainers reading `athena status`; the status must never advise retiring a role that is actively running.
- Code survey: crates/athena-core/src/store.rs (Records struct lines ~239-251 keeps steward_runs, audience_runs, auditor_runs apart from jobs; split at ~421-456), crates/athena-core/src/status.rs (StatusView, STATUS_MEASURE_LABELS, improvement_cycle_line already present), crates/athena-data/src/records/introspection.rs (improvement_cycle already present with tests), crates/athena-cli/src/display.rs. No new record or data file is added, so check_record_file/AnyRecord is not touched.
- Existing code: crates/athena-core/src/store.rs Records and its steward_runs/audience_runs/auditor_runs fields, and crates/athena-core/src/status.rs StatusView, are extended, not replaced.
- Prior job 20261029-233051-jq6k (rejected): role_need.rs is absent from the repo, so its work is re-done here. Findings addressed: blocker (steward/audience/auditor invisible) fixed by counting every run source; minor missing compute test fixed by a Records-level test; minor unwindowed class history fixed by using only the newest N closed jobs per class; nit fixed by rendering through status() rather than the section formatter.
- Prior job 20261029-233058-fveq (rejected): its improvement_cycle code is already in the repo (introspection.rs, status.rs, display tests exist), so it is kept and not modified; its findings on that measure are out of scope here.
- Cause stated: a measure was written against one data source and never replayed over real records against a list of known-active roles. The information the stage lacked is the complete set of run sources, supplied by one accessor so a new measure cannot pick a partial source.
- No refusal rule is added: role-need candidates are advisory status lines only, so no refused-reading share applies (expected 0% of newly refused readings).

## Acceptance criteria

1. [content] crates/athena-core/src/store.rs has one Records method that yields invocations from jobs plus steward_runs, audience_runs and auditor_runs; crates/athena-core/src/role_need.rs uses it and not `invocations` alone (Grep).
2. [content] crates/athena-core/src/role_need.rs is declared in lib.rs, windows class history to the newest N closed jobs, and OLY-9-11 in docs/spec.toml names it as mechanism with a measure ending `status: role need`.
3. [behavior] A test in role_need.rs builds Records with more than eight closed jobs holding only reader/producer/verifier runs plus recent steward, audience and auditor runs, and asserts none of those three is a retire candidate; it fails when compute reads only job invocations. Run `cargo test -p athena-core role_need`.
4. [behavior] A test renders a StatusView through status() and asserts the `role need` line prints only when a candidate exists, and that mixed outcomes (Failed/Rejected/TimedOut counted failed; Declined/Closed excluded) are mapped correctly. Run `cargo test`.
5. [behavior] A test over Records asserts the measure yields both outcomes (some roles retired-or-not, some not) rather than all-true or all-false on a mixed fixture. Run `cargo test -p athena-core`.
6. [behavior] cargo fmt --check, cargo clippy and cargo test all pass.
7. [content] CHANGELOG.md names the root cause (usage measure blind to roles whose runs are outside job records) and the fix.

## Expected deliverables

- crates/athena-core/src/role_need.rs (create)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)

## Out of scope

- Automatically creating or retiring roles
- OLY-9-12 (value of a new role against a simpler alternative)
- Changes to improvement_cycle in introspection.rs and its status/display code (job 20261029-233058-fveq work kept as is)
- crates/athena-cli/tests/status_build_measurement.rs (unchanged)
- Changes to prompts, athena.toml or defaults.toml
- A generic refusal rule or gate on new measures

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that active roles are not flagged, not just that code exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test required.
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

- path: <state>/jobs/20261111-122637-aawj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-122637-aawj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/store.rs has one Records method that yields invocations from jobs plus steward_runs, audience_runs and auditor_runs; crates/athena-core/src/role_need.rs uses it and not `invocations` alone (Grep).
crates/athena-core/src/role_need.rs is declared in lib.rs, windows class history to the newest N closed jobs, and OLY-9-11 in docs/spec.toml names it as mechanism with a measure ending `status: role need`.
A test in role_need.rs builds Records with more than eight closed jobs holding only reader/producer/verifier runs plus recent steward, audience and auditor runs, and asserts none of those three is a retire candidate; it fails when compute reads only job invocations. Run `cargo test -p athena-core role_need`.
A test renders a StatusView through status() and asserts the `role need` line prints only when a candidate exists, and that mixed outcomes (Failed/Rejected/TimedOut counted failed; Declined/Closed excluded) are mapped correctly. Run `cargo test`.
A test over Records asserts the measure yields both outcomes (some roles retired-or-not, some not) rather than all-true or all-false on a mixed fixture. Run `cargo test -p athena-core`.
cargo fmt --check, cargo clippy and cargo test all pass.
CHANGELOG.md names the root cause (usage measure blind to roles whose runs are outside job records) and the fix.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
