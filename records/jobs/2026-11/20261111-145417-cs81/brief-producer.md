# Job 20261111-145417-cs81: Retry 20261111-122637-aawj: role-need measure over all run sources

## Request (verbatim)

> try again on 20261111-122637-aawj

## Reading (accepted by the user)

Retries job 20261111-122637-aawj (rejected): rebuild the OLY-9-11 role-need measure on one all-sources Records accessor, and address the verifier's findings. Specifically, never flag on-demand or out-of-job roles (steward, audience, auditor) for retirement, document the split-source trap where writers see it, and test with known-active roles.

Assumptions:
- Goal and audience: the Athena maintainer reading `athena status`, and future writers of usage/need measures, who must not be misled about which roles are active.
- Job 20261111-122637-aawj is addressed finding by finding. (1) Replay major: the measure advised retiring the auditor on real-shaped records. Fixed by a KNOWN_ACTIVE/on-demand list in role_need.rs (steward, audience, auditor) that are reported as 'not measured', never as retire candidates, and by a test replaying compute over Records with those roles. (2) Opt-in accessor major: doc comments on JobRecords::invocations and Records state that steward, audience and auditor runs are not in it; docs/system-facts.md gains a note naming the split and pointing to Records::all_invocations; a store test over Records::from_parts asserts every Invocation record loaded appears in all_invocations, so a future fourth split list fails. (3) Misread-request major: criteria now check the class (known-active roles unflagged, both outcomes, store coverage), not one instance. (4) Minor counts_as_work: Dropped, stale and abandoned jobs that never ran a model are excluded from the window and totals. (5) Minor extra files: crates/athena-cli/tests/status_role_need.rs and xtask/shard-timings.txt are kept as deliverables since they register the new test binary.
- Existing code: crates/athena-core/src/store.rs (Records, JobRecords, from_parts, steward_runs/audience_runs/auditor_runs), crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, docs/spec.toml OLY-9-11 (mechanism = none, measure 'status: not shown'); the criteria extend these, none is replaced.
- Code survey: read crates/athena-core/src/store.rs (JobRecords.invocations at line 29; Records steward_runs/audience_runs/auditor_runs lines 239-251; from_parts line 376 routes invocations to the separate lists at 421-427). `.invocations` is used in 24 files under crates/, so making it private is a large churn; this reading keeps it public but documented and covered by a coverage test, and the owner can ask for privacy as a follow-up. No new record or data file is added under crates/, so check_record_file/AnyRecord/round-trip are unaffected. The property is measured directly by calling RoleNeed::compute on Records, not via a stand-in.
- The previous candidate's role_need.rs, store.rs all_invocations, status.rs, display.rs, lib.rs, spec.toml and CHANGELOG changes are kept as the base where sound; the producer starts from the base and reapplies them with the fixes above.
- No refusal rule is added: the measure only advises on the status line and does not refuse work, so no refused-share figure applies.
- Governance files athena.toml, defaults.toml and workflows are unchanged and not needed for this outcome.

## Acceptance criteria

1. [content] crates/athena-core/src/store.rs has Records::all_invocations covering jobs' invocations plus steward_runs, audience_runs and auditor_runs, and doc comments on JobRecords::invocations and Records say those three roles' runs are not in the per-job list (Grep 'all_invocations' and the doc text).
2. [content] docs/system-facts.md names the steward, audience and auditor runs as kept outside job records and points usage measures to Records::all_invocations (Grep the file).
3. [content] crates/athena-core/src/role_need.rs reads usage only via all_invocations (Grep: no `.invocations` use there), is declared in lib.rs, treats steward/audience/auditor as known-active or on-demand ('not measured', never retire candidates), and excludes jobs that never ran a model (dropped, stale, abandoned, Declined, Closed) from the window and class totals.
4. [behavior] A store test over Records::from_parts asserts every Invocation record loaded appears in all_invocations, so a new split list would fail it. Run `cargo test -p athena-core store`.
5. [behavior] A role_need test builds Records with more than eight closed jobs holding only reader/producer/verifier runs and recent steward, audience and auditor runs, replays RoleNeed::compute, and asserts none of those three is a retire candidate while a role with no runs in any source is; it fails if compute reads only job invocations. Run `cargo test -p athena-core role_need`.
6. [behavior] A test over a mixed fixture asserts the measure yields both outcomes (some roles flagged, some not), not all-true or all-false; and a status() render test asserts the `role need` line prints only when a candidate exists, with Failed/Rejected/TimedOut counted failed. Run `cargo test`.
7. [content] docs/spec.toml OLY-9-11 sets mechanism to a real mechanism naming role_need and a measure ending `status: role need`; CHANGELOG.md names the root cause (usage measure blind to roles whose runs are outside job records) and the fix.
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.

## Expected deliverables

- crates/athena-core/src/role_need.rs (create)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- crates/athena-cli/tests/status_role_need.rs (create)
- xtask/shard-timings.txt (modify)

## Out of scope

- Making JobRecords::invocations private across the 24 files that use it (follow-up)
- A standing gate or refusal rule on new measures
- Automatically creating or retiring roles
- OLY-9-12 (value of a new role against a simpler alternative)
- Changes to improvement_cycle in introspection.rs and its status/display code
- Changes to prompts, athena.toml or defaults.toml

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria replay the measure on known-active roles and require both outcomes.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261111-122637-aawj raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major (crates/athena-core/src/role_need.rs RoleNeed::compute / shown): The measure was not replayed over the real records. On them it would advise retiring the auditor, which the request names as known-active. Evidence snapshot, read with Glob and Grep; this is a result about the snapshot, not the live repository. The newest 8 closed jobs that count as work are 20261110-191432-ysr1, 20261111-022136-czdw, 055238-5har, 061100-smxs, 064834-q749, 075222-nsk3, 105048-n7bm and 122652-1k8z (landed, rejected and failed outcomes). Steward runs (20261111-055051, 061048), audience runs, and planner and critic runs (191432-ysr1, 064834-q749) fall inside the window. The only auditor invocations are jobs/2026-10/20261017-201222-3a9h and 20261017-200939-4ynn. So status would print "role need  retire candidates: auditor ...". The request's cause lists the auditor among roles "known" to be active and says the missing step was replay "on the real records" and comparison "with known facts". The auditor runs on demand (`athena audit`), so a fixed 8-job window may not show whether it is needed. The fix needs either a replay against the known-active list or a "not measured" / on-demand case, as introspection 20261111-105017-5k6m proposed. `status --snapshot` does not print role need, so the snapshot cannot show the line.
> major (crates/athena-core/src/store.rs:29 (JobRecords::invocations), docs/system-facts.md): The class fix is an opt-in accessor. Nothing points a future measure to it or stops one from using the partial source, so the class is not prevented. `pub invocations: Vec<Invocation>` on JobRecords still has no doc comment saying the steward, audience and auditor runs are not in it. The only pointer is on the new all_invocations, which a writer reaching for job.invocations will not see. docs/system-facts.md has no mention of steward, audience, auditor or invocation (Grep: no matches). The request prefers giving the stage this information in its brief or system facts. Introspection 5k6m names "The producer was not told where each role's runs are recorded". No test or check requires a new measure to be replayed over real records, and no store-level test checks that all_invocations covers every loaded invocation (one would catch a future fourth split list). Simpler and stronger options: document the split on JobRecords::invocations and in the system facts, and add a test over Records::from_parts that every Invocation record appears in all_invocations.
> major: The reading does not match the request, so passing criteria do not show the request was met The request asks for the cause to be fixed where it originates "so the whole class is prevented", preferably by giving the failing stage the missing information "in its brief, in the system facts it is shown, or in the inputs its reader receives". The cause it quotes includes "Nothing required it to be run on the real records, or compared with known facts such as the steward, audience and auditor being active". The reading narrowed this to one accessor plus one measure and claimed the accessor means "a new measure cannot pick a partial source". That is not true: JobRecords::invocations is still public and undocumented, nothing points a measure author to all_invocations, the system facts are unchanged, and no replay over real records against known-active roles is required or done. The criteria therefore check one fixed instance, not prevention of the class.

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

- path: <state>/jobs/20261111-145417-cs81/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-145417-cs81/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/store.rs has Records::all_invocations covering jobs' invocations plus steward_runs, audience_runs and auditor_runs, and doc comments on JobRecords::invocations and Records say those three roles' runs are not in the per-job list (Grep 'all_invocations' and the doc text).
docs/system-facts.md names the steward, audience and auditor runs as kept outside job records and points usage measures to Records::all_invocations (Grep the file).
crates/athena-core/src/role_need.rs reads usage only via all_invocations (Grep: no `.invocations` use there), is declared in lib.rs, treats steward/audience/auditor as known-active or on-demand ('not measured', never retire candidates), and excludes jobs that never ran a model (dropped, stale, abandoned, Declined, Closed) from the window and class totals.
A store test over Records::from_parts asserts every Invocation record loaded appears in all_invocations, so a new split list would fail it. Run `cargo test -p athena-core store`.
A role_need test builds Records with more than eight closed jobs holding only reader/producer/verifier runs and recent steward, audience and auditor runs, replays RoleNeed::compute, and asserts none of those three is a retire candidate while a role with no runs in any source is; it fails if compute reads only job invocations. Run `cargo test -p athena-core role_need`.
A test over a mixed fixture asserts the measure yields both outcomes (some roles flagged, some not), not all-true or all-false; and a status() render test asserts the `role need` line prints only when a candidate exists, with Failed/Rejected/TimedOut counted failed. Run `cargo test`.
docs/spec.toml OLY-9-11 sets mechanism to a real mechanism naming role_need and a measure ending `status: role need`; CHANGELOG.md names the root cause (usage measure blind to roles whose runs are outside job records) and the fix.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
