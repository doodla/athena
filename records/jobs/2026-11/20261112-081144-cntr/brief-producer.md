# Job 20261112-081144-cntr: Give OLY-B-O5-01 a mechanism with an accepted restarts status label

## Request (verbatim)

> try again on 20261030-013416-weaa

## Reading (accepted by the user)

Retry of rejected job 20261030-013416-weaa: record `athena restart` and the `restarts` line of `athena status` as the mechanism for OLY-B-O5-01, this time adding ("restarts", "restarts") to STATUS_MEASURE_LABELS so the measure ending `status: restarts` passes the spec checker. A new test fails on the base tree (label rejected, requirement still a gap) and passes after. Audience: the owner reading the spec gap list.

Assumptions:
- Prior job 20261030-013416-weaa (rejected) is addressed finding by finding: (a) the label `restarts` was rejected by the spec checker -> fixed by adding the entry to STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs (line 305, which today lists 16 labels and no `restarts`); (b) its test passed on the base tree -> replaced by a test in status.rs that fails on the base; (c) CHANGELOG claimed a rejected measure -> entry rewritten after the real-spec test passes; (d) behavior evidence not tied to commands that ran -> criteria name the exact cargo commands.
- Kept from that candidate: docs/spec.toml and CHANGELOG.md edits (redone). Dropped: its crates/athena-cli/tests/lifecycle.rs edit, because a base lifecycle test (around line 2940-2963) already records a restart via athena.restart and asserts status.restarts == 1; adding another proves nothing new. Its planned crates/athena-cli/tests/restart.rs does not exist at the base and is not created; crates/athena-core/src/manage.rs is not changed either.
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at 305; `pub restarts: usize` at 475; `restarts: records.restarts.len()` at 1322; existing forecasts-requirement test at ~1630-1656 and `every_measure_label_names_a_status_view_field_and_a_printed_line` at 1659, which requires a StatusView field `restarts` and a printed string starting `"restarts `), crates/athena-cli/src/display.rs:1411 (`restarts    {}` line), crates/athena-core/src/hestia.rs (measure_problem ~1219-1230 and spec_gaps, which reject unlisted labels), crates/athena-core/src/store.rs (Restart records load into records.restarts), crates/athena-cli/tests/lifecycle.rs (existing restart/status test). No new record or data file is added, so check_record_file/AnyRecord need no change. The property is measured directly by spec_gaps on the real docs/spec.toml requirement, not a stand-in.
- Existing code: docs/spec.toml (requirement OLY-B-O5-01 at line 2265, mechanism currently "none", measure "The spec gap list; status: requirement"), crates/athena-core/src/status.rs, crates/athena-core/src/hestia.rs, crates/athena-cli/src/display.rs, crates/athena-core/src/manage.rs (`restart`) are reused; only the label table, spec entry, a test and the changelog change.
- Consumers: none found beyond those named. Governance files athena.toml, defaults.toml and workflows are not changed.
- Spec mechanism text names crates/athena-core/src/manage.rs (`Manager::restart`, the `athena restart` command) and crates/athena-core/src/status.rs plus crates/athena-cli/src/display.rs (the `restarts` line); the mechanism records state of the work coming from Athena. Per-milestone or efficiency-drop enforcement stays prose-only and the measure says so.

## Acceptance criteria

1. [content] docs/spec.toml OLY-B-O5-01 has a mechanism naming crates/athena-core/src/manage.rs and the `restarts` line of athena status (status.rs, display.rs) instead of "none", owner stays Athena, and the measure explains how the requirement is shown and ends with `status: restarts`. Check with Grep; every named path exists (ls).
2. [content] crates/athena-core/src/status.rs STATUS_MEASURE_LABELS contains ("restarts", "restarts"); the `restarts    ` line in crates/athena-cli/src/display.rs matches the label. Check with Grep.
3. [behavior] Run `cargo test -p athena-core --lib status` and `cargo test -p athena-core --lib hestia`: a new status.rs test loads the real docs/spec.toml requirement OLY-B-O5-01, asserts measure_problem is None, the measure ends `status: restarts`, and spec_gaps no longer lists it; with mechanism reset to "none" it is a gap again. The test fails on the base tree (label rejected) and passes now; the existing `the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed` and `every_measure_label_names_a_status_view_field_and_a_printed_line` tests pass.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`; all pass, including the xtask spec test.
5. [content] CHANGELOG.md has an entry stating the mechanism and measure `status: restarts` and the new label; the diff touches only docs/spec.toml, crates/athena-core/src/status.rs and CHANGELOG.md (the deliverables); status/planning.rs, manage.rs and lifecycle.rs are unchanged.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Enforcing a restart per milestone or on an efficiency drop with a new gate or refusal
- Changing crates/athena-core/src/manage.rs, crates/athena-core/src/status/planning.rs, or crates/athena-cli/tests/lifecycle.rs (dropped from the prior candidate)
- Creating crates/athena-cli/tests/restart.rs (does not exist at the base; not added)
- Changing athena.toml, defaults.toml or workflows
- Editing records/ or other requirements in docs/spec.toml

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the spec gap list on the real spec, not that wiring exists.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A new test fails on the base tree and names the cargo commands to run.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and cargo test criterion.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261030-013416-weaa raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 4 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo test --locked`
> blocker (docs/spec.toml (OLY-B-O5-01 measure); crates/athena-core/src/status.rs:305 STATUS_MEASURE_LABELS; crates/athena-core/src/hestia.rs:1219 measure_problem): The measure label `restarts` is not an accepted status measure label, so the real-spec test fails and OLY-B-O5-01 is still a spec gap. `cargo test -p athena-core --lib the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed` fails: "the measure names `restarts`, which `athena status` does not print". STATUS_MEASURE_LABELS has no `restarts` entry. The fix is to add ("restarts", "restarts") there, which the existing `every_measure_label_names_a_status_view_field_and_a_printed_line` test would then check, or to use `status: not shown` and name the test.
> major (crates/athena-cli/tests/lifecycle.rs:2966): No test fails without the change: the new lifecycle test also passes on the base tree, because the restart and status code already existed. The diff adds no production code, so the test pins behavior that already existed. The acceptance criterion "A test fails without the mechanism and passes with it" is met only in the sense that the test would fail if restart or status counting broke. Nothing ties the spec entry to a test, unlike the `oly_b_..._no_longer_appears_in_the_spec_gap_list` tests in hestia.rs, which would have caught the invalid label.
> major: The reading does not match the request, so passing criteria do not show the request was met The reading assumed that `restarts`, as a line `athena status` prints, is a valid measure label. In this repository a valid label is an entry of `STATUS_MEASURE_LABELS` (status.rs), and `restarts` is not listed. Either the reading had to plan for adding the label there, or it had to use `status: not shown` with the test. The request's condition that the label be "a line `athena status` prints" is enforced by that table, which the reading did not check.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 20 cases in 20 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs before the verifier, at reading acceptance and again after the producer finishes. It would (1) validate every measure label a job adds or edits in docs/spec.toml against STATUS_MEASURE_LABELS by running the real-spec test; (2) require each new behavior test to fail on the ...
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

- path: <state>/jobs/20261112-081144-cntr/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-081144-cntr/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

docs/spec.toml OLY-B-O5-01 has a mechanism naming crates/athena-core/src/manage.rs and the `restarts` line of athena status (status.rs, display.rs) instead of "none", owner stays Athena, and the measure explains how the requirement is shown and ends with `status: restarts`. Check with Grep; every named path exists (ls).
crates/athena-core/src/status.rs STATUS_MEASURE_LABELS contains ("restarts", "restarts"); the `restarts    ` line in crates/athena-cli/src/display.rs matches the label. Check with Grep.
Run `cargo test -p athena-core --lib status` and `cargo test -p athena-core --lib hestia`: a new status.rs test loads the real docs/spec.toml requirement OLY-B-O5-01, asserts measure_problem is None, the measure ends `status: restarts`, and spec_gaps no longer lists it; with mechanism reset to "none" it is a gap again. The test fails on the base tree (label rejected) and passes now; the existing `the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed` and `every_measure_label_names_a_status_view_field_and_a_printed_line` tests pass.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`; all pass, including the xtask spec test.
CHANGELOG.md has an entry stating the mechanism and measure `status: restarts` and the new label; the diff touches only docs/spec.toml, crates/athena-core/src/status.rs and CHANGELOG.md (the deliverables); status/planning.rs, manage.rs and lifecycle.rs are unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
