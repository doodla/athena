# Job 20261112-180947-td6x: Give OLY-B-NOTE-01 a specialization mechanism, measure and tests

## Request (verbatim)

> try again on 20261030-013725-ap0q

## Reading (accepted by the user)

Retry of job 20261030-013725-ap0q: record the existing `specialization` status figure as the mechanism for OLY-B-NOTE-01 in docs/spec.toml, keep its snapshot print, and add tests for both `athena status` and `status --snapshot`. Audience: the owner reading `athena status`. The earlier candidate is kept (CHANGELOG.md, display.rs, routing_evidence.rs, snapshot.rs, spec.toml, shard-timings.txt), with the snapshot test gap fixed and real-records evidence cited from commands that ran.

Assumptions:
- Addresses job 20261030-013725-ap0q (rejected): its major finding (behavior criterion 3 evidence cites a command not seen running) is fixed by a criterion the verifier runs itself on real records against independent counts; its minor finding (snapshot path has no test) is fixed by a test calling display::snapshot_status; the out-of-scope-files finding is fixed by listing snapshot.rs and xtask/shard-timings.txt as deliverables (candidate work kept); the nit about 'unrecorded' effort counting as a setting is left alone, it predates the change.
- Code survey: read crates/athena-core/src/status/planning.rs (`specialization` figure, SPECIALIZATION_MIN_JOBS), crates/athena-cli/src/display.rs (`status` and `snapshot_status` print the block), crates/athena-core/src/snapshot.rs (SnapshotView, specialization field added by the earlier candidate). The change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are untouched. The figure is measured directly from producer invocations per class of work, not through a stand-in. The records validation test reads docs/spec.toml and must still pass.
- Existing code: crates/athena-core/src/status/planning.rs (`specialization`), crates/athena-cli/src/display.rs, crates/athena-core/src/snapshot.rs, and docs/spec.toml are reused or extended, not rebuilt.
- Environment: the real-records check runs `cargo run -p athena-cli -- status --snapshot <evidence directory>` read-only (no writes, no network) over the evidence snapshot holding history.json and the recorded invocations; the independent counts come from those files, which the change does not compute.
- Goal and audience: requirement OLY-B-NOTE-01 is shown to the owner through the `specialization` heading in `athena status`; the measure ends `status: specialization`.
- Governance: athena.toml, defaults.toml and .github/workflows are not changed. xtask/shard-timings.txt is a data file the ci_shards test needs for the new test file, not a governance file.
- Shared files: shard-timings.txt gains a line for routing_evidence.rs only if ci_shards requires it.

## Acceptance criteria

1. [content] In docs/spec.toml, requirement OLY-B-NOTE-01 has a mechanism that is no longer "none" and names crates/athena-core/src/status/planning.rs (`specialization`) and crates/athena-cli/src/display.rs, both found by Grep; owner stays Athena; measure says how it is shown and ends with `status: specialization`, a heading display.rs prints.
2. [behavior] Run `cargo test`: crates/athena-cli/tests/routing_evidence.rs builds fixture records where a non-baseline model/effort setting with at least SPECIALIZATION_MIN_JOBS jobs has a higher landed share, and asserts `display::status` prints the `specialization` line flagging it as helping; a second case with fewer jobs asserts no flag. Removing the figure or its display makes it fail.
3. [behavior] Run `cargo test`: a test in routing_evidence.rs calls `display::snapshot_status` on a fixture snapshot and asserts the `specialization` block appears; it fails if the snapshot print line in display.rs or the SnapshotView field in snapshot.rs is removed.
4. [behavior] Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records: the `specialization` block lists each class of work and its per-setting job counts sum to the expected range, which is the number of dispatched request jobs with a producer invocation counted from history.json and the recorded invocations (a source the change does not compute), within 0 difference. Every class flagged as helping has at least 3 jobs in that setting.
5. [behavior] Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` before and after on the real records: the landing time, throughput, keep rule and planning cost lines are unchanged; the landing time is within the expected range of a few minutes taken from commit times in history.json.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including existing planning, display, records validation and xtask ci_shards tests.
7. [content] CHANGELOG.md has an entry for the OLY-B-NOTE-01 mechanism and the snapshot print; the diff changes only the listed deliverables.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/snapshot.rs (modify)
- crates/athena-cli/tests/routing_evidence.rs (create)
- CHANGELOG.md (modify)
- xtask/shard-timings.txt (modify)
- crates/athena-core/src/status/planning.rs (modify)

## Out of scope

- Automatically changing the configured model or effort per role from the evidence.
- Changing athena.toml, defaults.toml or CI workflows.
- Changing landing time, throughput, keep rule or planning cost figures (checked by the before/after real-records criterion).
- Excluding 'unrecorded' effort from the flag or adding a margin to the helping rule.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Real-records run compared with independent counts from history.json.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criterion plus new tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the status command and tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261030-013725-ap0q raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: Behavior criterion 3 is unmet: its evidence must cite commands that ran (D43) cited but not seen running: `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013725-ap0q/evidence`

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

- path: <state>/jobs/20261112-180947-td6x/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-180947-td6x/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

In docs/spec.toml, requirement OLY-B-NOTE-01 has a mechanism that is no longer "none" and names crates/athena-core/src/status/planning.rs (`specialization`) and crates/athena-cli/src/display.rs, both found by Grep; owner stays Athena; measure says how it is shown and ends with `status: specialization`, a heading display.rs prints.
Run `cargo test`: crates/athena-cli/tests/routing_evidence.rs builds fixture records where a non-baseline model/effort setting with at least SPECIALIZATION_MIN_JOBS jobs has a higher landed share, and asserts `display::status` prints the `specialization` line flagging it as helping; a second case with fewer jobs asserts no flag. Removing the figure or its display makes it fail.
Run `cargo test`: a test in routing_evidence.rs calls `display::snapshot_status` on a fixture snapshot and asserts the `specialization` block appears; it fails if the snapshot print line in display.rs or the SnapshotView field in snapshot.rs is removed.
Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` on the real records: the `specialization` block lists each class of work and its per-setting job counts sum to the expected range, which is the number of dispatched request jobs with a producer invocation counted from history.json and the recorded invocations (a source the change does not compute), within 0 difference. Every class flagged as helping has at least 3 jobs in that setting.
Run `cargo run -p athena-cli -- status --snapshot <evidence directory>` before and after on the real records: the landing time, throughput, keep rule and planning cost lines are unchanged; the landing time is within the expected range of a few minutes taken from commit times in history.json.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, including existing planning, display, records validation and xtask ci_shards tests.
CHANGELOG.md has an entry for the OLY-B-NOTE-01 mechanism and the snapshot print; the diff changes only the listed deliverables.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
