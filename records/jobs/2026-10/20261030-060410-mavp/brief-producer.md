# Job 20261030-060410-mavp: Defer introspections while the kill switch is on; run on start

## Request (verbatim)

> When the kill switch is on, have Athena defer an introspection (keep it pending and run it after `athena start`) instead of starting the model and recording status failed. Rerun the two introspections that already failed this way, for jobs 20261029-142858-5c0p and 20261017-155259-7mjc. I read both records: each failed with 'the kill switch is on', and neither job has any other introspection. Acceptance: a test shows an introspection triggered while the kill switch is on is left pending, not failed, and runs once the switch is off; fmt, clippy -D warnings and the tests pass.
> 
> Acceptance:
> - The change is made as described: Introspections start while the kill switch is on, are recorded as failed, and are never rerun, so two failed jobs have no diagnosis
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-10/20261030-013540-p86y.json

## Reading (accepted by the user)

When the kill switch is on, `Athena::introspect` queues the event in the state directory instead of invoking the model and committing a failed record. `athena start` then runs the queued introspections. This also reruns the two kill-switch failures, for jobs 20261029-142858-5c0p and 20261017-155259-7mjc, through that queue.

Assumptions:
- Code survey: I read crates/athena-core/src/introspect.rs. `introspect_job` and `introspect_intervention` go through `introspect`, which builds a Failed record and commits it even when `driver.invoke` errors. I read crates/athena-core/src/manage.rs, where `start()` calls `state.registry().start()`, and crates/athena-cli/src/commands.rs, where `start()` is the `athena start` entry point. I read crates/athena-data/src/records/introspection.rs, where `IntrospectionStatus` has only Recorded and Failed. `registry().stopped()` is already used in status.rs. The existing test file is crates/athena-cli/tests/introspection.rs. Measurement is direct: a test with the kill switch on and a fake driver counts model calls and committed records.
- Design: no new record status. A deferred event is a pending file under the state root (`introspect-pending/`), not a record, so the records allow-list, `AnyRecord`, `check_record_file` and the round-trip test are untouched. `athena start`, only when the switch was on, drains the queue by calling `introspect` per event and deletes each entry after it is committed. An entry whose run fails stays recorded as failed, as today, and is not requeued, so a failing item cannot loop.
- Interpretation: the 'kill switch is on' check is read from `state.registry().stopped()` before the model is invoked. The same deferral applies to job and intervention events. A trigger that is already queued is not queued twice.
- Term pending: an introspection event that was triggered, has no introspection record yet, and is waiting in the state-directory queue.
- Retry of 20261029-142858-5c0p: its planning stopped because the kill switch was turned on, so it produced no candidate and has no files to keep. It is addressed by queueing its introspection for rerun, not by redoing its build work. Retry of 20261017-155259-7mjc: it was rejected because `cargo test --locked` failed with no exit code. Its introspection is rerun the same way. Its seven changed CI-routing files are dropped and excluded in out_of_scope.
- The rerun of the two failed introspections needs the model, which the producer cannot call. It is done by the owner running `athena start` after the change lands, and the change provides a way to enqueue those two events. The producer does not edit records/ (records/introspections/2026-10/20261030-013540-p86y.json is read only), and the old failed records stay as history.
- Environment: a local state directory, read and write for the producer. No network, push, PR or CI workflow is used. The tests use the fake driver in crates/athena-agent, and git commits are made inside the test repository only.
- Consumers: none. No config file changes.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/introspection.rs triggers `introspect_job` (a rejected or failed job) while the kill switch is on. It asserts that the fake driver was not invoked, no introspection record was committed with status failed, and one event is pending. Evidence: `cargo test` shows it passing.
2. [behavior] The same test, or a second one, turns the switch off through the real `start` path (`Athena::start` or the `athena start` command). It asserts that the pending event now yields exactly one committed introspection record with status recorded, and that the queue is then empty. Evidence: `cargo test`.
3. [behavior] A test covers an intervention-triggered introspection while the switch is on. It is deferred and then run on start, and a second `athena start` does not run it again. Evidence: `cargo test`.
4. [content] The diff to crates/athena-core/src/introspect.rs (and the `start` path in crates/athena-core/src/manage.rs or crates/athena-cli/src/commands.rs) shows the stopped check happening before `driver.invoke`, with a queue of pending events and a drain run when the switch is turned off.
5. [content] CHANGELOG.md has a new entry saying that introspections are deferred while the kill switch is on and run after `athena start`.
6. [behavior] No unrelated behavior changes. The existing tests in crates/athena-cli/tests/introspection.rs and the other crates pass unchanged, and `the_kill_switch_blocks_new_work` still passes. Evidence: `cargo test`.
7. [behavior] `cargo fmt --check`, `cargo clippy` (with -D warnings as configured) and `cargo test` pass, as recorded by Athena on the final tree.
8. [content] The change does not edit anything under records/, docs/decisions.md or athena.toml. If a decision is recorded, it is a new appended decision with a Provenance bullet.

## Expected deliverables

- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Any edit to records/, including records/introspections/2026-10/20261030-013540-p86y.json. The old failed records stay.
- The seven files the rejected job 20261017-155259-7mjc changed (ci_result.rs, gh.rs, routing.rs, run.rs, run_ci_tests.rs, docs/design/ci-routing.md, CHANGELOG.md CI entry) are dropped, not kept.
- The build work of job 20261029-142858-5c0p (per-job worktree start, disk bound). It produced no candidate and is not redone here.
- Changing how the kill switch blocks agents and checks, or deferring anything other than introspections.
- A new IntrospectionStatus variant or any change to the record schema.
- Changes to athena.toml, defaults.toml or .github/workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new passing tests plus cargo test and clippy
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check pending-then-run behavior through the real start path
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, with no new record type
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The deferral is enforced in code and tested, not left as prose
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

- path: <state>/jobs/20261030-060410-mavp/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060410-mavp/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-cli/tests/introspection.rs triggers `introspect_job` (a rejected or failed job) while the kill switch is on. It asserts that the fake driver was not invoked, no introspection record was committed with status failed, and one event is pending. Evidence: `cargo test` shows it passing.
The same test, or a second one, turns the switch off through the real `start` path (`Athena::start` or the `athena start` command). It asserts that the pending event now yields exactly one committed introspection record with status recorded, and that the queue is then empty. Evidence: `cargo test`.
A test covers an intervention-triggered introspection while the switch is on. It is deferred and then run on start, and a second `athena start` does not run it again. Evidence: `cargo test`.
The diff to crates/athena-core/src/introspect.rs (and the `start` path in crates/athena-core/src/manage.rs or crates/athena-cli/src/commands.rs) shows the stopped check happening before `driver.invoke`, with a queue of pending events and a drain run when the switch is turned off.
CHANGELOG.md has a new entry saying that introspections are deferred while the kill switch is on and run after `athena start`.
No unrelated behavior changes. The existing tests in crates/athena-cli/tests/introspection.rs and the other crates pass unchanged, and `the_kill_switch_blocks_new_work` still passes. Evidence: `cargo test`.
`cargo fmt --check`, `cargo clippy` (with -D warnings as configured) and `cargo test` pass, as recorded by Athena on the final tree.
The change does not edit anything under records/, docs/decisions.md or athena.toml. If a decision is recorded, it is a new appended decision with a Provenance bullet.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
