# Job 20261029-233051-jq6k: Give OLY-9-11 a role-need mechanism shown in athena status

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-9-11 a mechanism
> 
> Requirement OLY-9-11 in docs/spec.toml is active and has no mechanism.
> Requirement: Phase 6 extends specialization, delegation and role evolution based on observed need.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-9-11
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-9-11 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a role-need measure in athena-core, computed from records and printed by `athena status` as a new `role need` line, so Phase 6 role evolution (add or retire roles) is driven by observed need. Record it as OLY-9-11's mechanism and measure in docs/spec.toml, with a test that fails without it. Audience: Athena owner and the Hestia spec pass.

Assumptions:
- Code survey: read crates/athena-core/src/class_trend.rs (the OLY-9-10 model: analyze, ClassTrend::compute, lines), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at ~305, StatusView fields, compute calls at ~1170 and ~1278), crates/athena-core/src/status/planning.rs (specialization at 488), crates/athena-core/src/learning.rs, crates/athena-cli/src/display.rs (trend_section ~1103, call ~1330), xtask/tests/spec.rs (spec validator requiring mechanism, owner, and a measure ending `status: <label>`).
- Existing code: extend crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView), reuse specialization in crates/athena-core/src/status/planning.rs and the class_trend.rs pattern; print via crates/athena-cli/src/display.rs; spec entry in docs/spec.toml.
- Design: a new module crates/athena-core/src/role_need.rs with a pure analyze function over per-class and per-role job counts and outcomes from Records. It reports a class of work with enough jobs and a repeated failure or slow median as a role-needed candidate, and a role with no invocations over a window as a retire candidate. It reports nothing below a minimum job count, like MIN_CLASS_JOBS. Candidates are advisory only; the mechanism does not create or retire roles.
- The new label is `role need`, added to STATUS_MEASURE_LABELS mapped to a new StatusView field, and printed at the start of a line by athena status. The record-file allow-list (check_record_file, AnyRecord) is not touched because no new record or data file is added; the measure is computed from existing records.
- The measure is read directly from records, not through a stand-in. OLY-9-12 is left untouched.
- Adds no refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-9-11 has a mechanism naming an existing path (crates/athena-core/src/role_need.rs), owner Athena, and a measure ending `status: role need`; the `role need` label is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
2. [behavior] Unit tests in crates/athena-core/src/role_need.rs fail when analyze is stubbed to return None and pass with it: a class with repeated failures above the minimum job count yields a role-needed candidate, an unused role yields a retire candidate, and a thin history yields nothing. Run `cargo test -p athena-core role_need`.
3. [behavior] A display test in crates/athena-cli/src/display.rs shows that StatusView with role-need candidates prints a line starting `role need`, and prints no such line when there is no candidate. Run `cargo test -p athena-cli`.
4. [behavior] `cargo test -p xtask` (the spec test in xtask/tests/spec.rs) passes with the updated OLY-9-11 entry; `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.
5. [content] The diff changes only docs/spec.toml, crates/athena-core/src/role_need.rs, lib.rs, status.rs, crates/athena-cli/src/display.rs, and CHANGELOG.md if the repo convention requires it; no governance files (athena.toml, defaults.toml) change.

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

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and display tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check printed output and candidate results, not that files exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused module following the class_trend pattern.
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

- path: <state>/jobs/20261029-233051-jq6k/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233051-jq6k/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml entry OLY-9-11 has a mechanism naming an existing path (crates/athena-core/src/role_need.rs), owner Athena, and a measure ending `status: role need`; the `role need` label is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
Unit tests in crates/athena-core/src/role_need.rs fail when analyze is stubbed to return None and pass with it: a class with repeated failures above the minimum job count yields a role-needed candidate, an unused role yields a retire candidate, and a thin history yields nothing. Run `cargo test -p athena-core role_need`.
A display test in crates/athena-cli/src/display.rs shows that StatusView with role-need candidates prints a line starting `role need`, and prints no such line when there is no candidate. Run `cargo test -p athena-cli`.
`cargo test -p xtask` (the spec test in xtask/tests/spec.rs) passes with the updated OLY-9-11 entry; `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.
The diff changes only docs/spec.toml, crates/athena-core/src/role_need.rs, lib.rs, status.rs, crates/athena-cli/src/display.rs, and CHANGELOG.md if the repo convention requires it; no governance files (athena.toml, defaults.toml) change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
