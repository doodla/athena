# Job 20261114-112705-q134: Hold kill-switch-stopped jobs as resumable, not failed

## Request (verbatim)

> try again on 20261030-113229-kgrg

## Reading (accepted by the user)

Retry of job 20261030-113229-kgrg (it failed only because the 600s planner deadline hit, nothing was kept). When the kill switch interrupts planning or blocks the producer from starting, record the job as stopped and resumable, keep its reading and plan, exclude it from failure-class counts, and resume it after `athena start` with no reader rerun. The reading is kept small so planning fits the deadline.

Assumptions:
- Goal and audience: the person running Athena, who judges this by the two kill-switch tests, status failure counts and resume behavior.
- Previous result: job 20261030-113229-kgrg failed with no candidate (planner deadline of 600s reached, not verified); its changes are none, so nothing is kept or dropped. This retry redoes the work from the base, with the same deliverable list, trimmed of files only if the planner finds them unneeded.
- Job 20261030-060410-mavp (introspections) is not redone: its landed changes to introspect.rs, manage.rs and its minor findings do not apply to jobs. Deadline handling for the planner (introspection 20261114-112627-efkg proposal) is a separate request and not addressed here.
- Code survey: read crates/athena-core/src/run/planning.rs (RoundEnd::Stopped returned at many sites, lines 66-483, enum at 531; planning returns Ok(None) on stop), run/producer.rs (line 207 Formatted::Stopped(note) returns the stop note), run/mod.rs, run/tests.rs, state.rs, status.rs, lib.rs. The shared point is the Stopped outcome from planning and producer start; the job end state is set in run/mod.rs. No new record or data file under crates/ is added, so check_record_file/AnyRecord are unaffected unless a new state variant is serialized; if so the existing round-trip tests must cover it.
- Existing code: crates/athena-core/src/run/planning.rs, run/producer.rs, run/mod.rs, state.rs, status.rs and the introspection deferral in introspect.rs/manage.rs are reused or extended; the job path follows the introspection's stopped-and-resumable pattern.
- Field meaning: any reused job end-state value 'stopped' follows the introspection deferral's meaning (held, resumable, not failed); the producer must quote it from state.rs and add a new variant only if none fits.
- Consumers: none found for config; athena.toml, defaults.toml and .github/workflows are not changed.

## Acceptance criteria

1. [behavior] A test in crates/athena-core/src/run/tests.rs turns the kill switch on during planning; after the run the job is recorded as stopped and resumable (not failed), and its accepted reading and any plan are kept. Run `cargo test`.
2. [behavior] A test turns the kill switch on before the producer starts; the job is stopped and resumable, not failed, and keeps its reading and plan. Run `cargo test`.
3. [behavior] In both tests, after `athena start` (start_with) the job resumes and proceeds, and a counting fake reader shows zero new reader invocations after the stop. Run `cargo test`.
4. [behavior] In both tests, the failure-class counts in status do not include the stopped job, asserted through the status counting function. Run `cargo test`.
5. [behavior] Non-kill-switch failures (a planner error, a producer failure with the switch off) are still recorded as failed and counted; a test covers this and all existing tests pass. Run `cargo test`.
6. [content] Both the planning and producer-start stops reach the stopped-and-resumable record at one shared point in crates/athena-core/src/run/, found by Grep for the shared stop reason, not duplicated text matching.
7. [content] docs/decisions.md has a new appended entry with a Provenance bullet citing job 20261030-060410-mavp as the introspection counterpart; no existing entry is edited; CHANGELOG.md has an entry; nothing under records/, athena.toml, defaults.toml or .github/workflows changes (check `git diff --stat`).
8. [behavior] The recorded `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` checks pass on the final tree.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Any edit to records/ (the 'Files: records/' line in the earlier request is not a deliverable).
- Introspection deferral (job 20261030-060410-mavp): landed changes kept as they are; its minor findings are not addressed.
- Changing how the kill switch blocks agents and checks, or deferring anything other than jobs.
- Changes to athena.toml, defaults.toml or .github/workflows.
- Rewriting records of jobs already recorded as failed by the kill switch.
- Planner deadline handling, draft-plan persistence and retry-from-draft (separate request from introspection 20261114-112627-efkg).

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check stopped, resumable and zero-reader-rerun behavior by tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt required.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by tests at one shared stop point.
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

- path: <state>/jobs/20261114-112705-q134/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-112705-q134/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A test in crates/athena-core/src/run/tests.rs turns the kill switch on during planning; after the run the job is recorded as stopped and resumable (not failed), and its accepted reading and any plan are kept. Run `cargo test`.
A test turns the kill switch on before the producer starts; the job is stopped and resumable, not failed, and keeps its reading and plan. Run `cargo test`.
In both tests, after `athena start` (start_with) the job resumes and proceeds, and a counting fake reader shows zero new reader invocations after the stop. Run `cargo test`.
In both tests, the failure-class counts in status do not include the stopped job, asserted through the status counting function. Run `cargo test`.
Non-kill-switch failures (a planner error, a producer failure with the switch off) are still recorded as failed and counted; a test covers this and all existing tests pass. Run `cargo test`.
Both the planning and producer-start stops reach the stopped-and-resumable record at one shared point in crates/athena-core/src/run/, found by Grep for the shared stop reason, not duplicated text matching.
docs/decisions.md has a new appended entry with a Provenance bullet citing job 20261030-060410-mavp as the introspection counterpart; no existing entry is edited; CHANGELOG.md has an entry; nothing under records/, athena.toml, defaults.toml or .github/workflows changes (check `git diff --stat`).
The recorded `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` checks pass on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
