# Job 20261113-205925-c3n6: Escalate planner model and effort on revision rounds via new keys

## Request (verbatim)

> try again on 20261113-194336-scfc

## Reading (accepted by the user)

Retry of job 20261113-194336-scfc, following the introspection's proposed request. Add optional [models] planner_revision_model and planner_revision_effort. After a critic revise or mechanical plan-rule revision, the planner uses them only if set and not weaker than the current planner setting. Do not reuse hard_producer_*. Round 1 is unchanged. The owner who runs Athena can check this in tests.

Assumptions:
- Prior job 20261113-194336-scfc: it failed in planning with no candidate, so nothing is kept. Its findings are addressed as follows. (a) Reusing hard_producer_* was a no-op here and could lower the setting: new dedicated keys are used and a weaker value is ignored. (b) The effect in this repo was misstated: the plan and CHANGELOG must state the effective planner model and effort with and without the change. (c) The criterion 1 wording about producer_setting no longer applies, because producer_setting is not used. Its out_of_scope items 'no new config keys' and 'no athena.toml change' are superseded by this request, which names new keys and an athena.toml entry. Its other out_of_scope items stay.
- Code survey: read crates/athena-data/src/config.rs. Models has hard_producer_effort at line 331. producer_setting is at line 346. effort_for is at line 830, and it maps the planner to verifier_effort. Key validation is around line 669. Not read in this session, taken from the earlier job's findings: crates/athena-core/src/run/planning.rs (planning_round, lines 138-155), crates/athena-core/src/reading.rs (agent_spec, line 443), and the fake-agent helpers in crates/athena-core/src/run/tests.rs. The entry is Athena::go. No new record or data file is added, so check_record_file and AnyRecord are not involved. The property is measured by the recorded planner invocation model and effort per round.
- Existing code: crates/athena-data/src/config.rs (Models, validation, effort_for), crates/athena-core/src/run/planning.rs (planning_round), docs/decisions.md and docs/spec.toml are extended.
- Consumers: planner_revision_* are read only by planning_round in crates/athena-core/src/run/planning.rs and validated in crates/athena-data/src/config.rs. The athena.toml entry is the only user-set value.
- Assumption: 'not weaker' is judged for effort by position in EFFORT_LEVELS (config.rs:337). For the model, it is judged by position in producer_allowed, which is assumed to be ordered weakest to strongest. The producer reads this ordering and states it in a doc comment. A key that would lower either value is ignored, and only that part falls back.
- Assumption: both keys must be in the allowed lists when set (model in producer_allowed, effort in EFFORT_LEVELS), matching how the hard_producer_* keys are validated. Unset keys leave behavior unchanged.
- Assumption: athena.toml gets a commented or actual entry for the new keys. The producer picks values stronger than the current planner settings so the change takes effect here, and the CHANGELOG says what the keys do in this repo.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/tests.rs drives Athena::go with the fake agent and a critic revise. It shows that the first planner invocation records the ordinary model and effort, and that the next one records planner_revision_model and planner_revision_effort. Run `cargo test -p athena-core` and cite the test name.
2. [behavior] New tests show that a mechanical plan-rule revision also escalates the next planner call. They also show that a key weaker than the current planner model or effort is ignored, and that with both keys unset every round keeps the same model and effort. Run `cargo test -p athena-core`.
3. [behavior] New config.rs unit tests show that the keys parse as optional, that a value outside the allowed lists is rejected, and that an existing config without the keys still loads. Run `cargo test -p athena-data`.
4. [content] In crates/athena-data/src/config.rs, Models has optional planner_revision_model and planner_revision_effort. They are not derived from hard_producer_model or hard_producer_effort. Grep planning.rs for a diff showing no use of producer_setting or hard_producer_* in the planner path.
5. [behavior] The owner can see the escalation. The planner line printed by planning_round, or the invocation record, shows the model used on the revision round. A test in crates/athena-cli/tests/drain.rs or in the core tests asserts it. Run `cargo test -p athena-cli` if drain.rs is changed.
6. [content] athena.toml has the new planner_revision_* entry under [models]. CHANGELOG.md has an entry that states the effective planner model and effort in this repo with and without the change, and what the keys do. docs/spec.toml, where it describes planning, mentions the revision setting.
7. [content] docs/decisions.md gets a new appended decision with a Provenance bullet that records why dedicated keys were chosen over hard_producer_*. No existing entry is edited.
8. [behavior] `cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace` pass. Each is named as its own simple command, or cited from the Athena-recorded passing checks on the final tree.

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/spec.toml (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Escalating the critic, reader or verifier.
- Changing defaults.toml or any governance file other than the athena.toml [models] entry.
- Changing crates/athena-core/src/status/planning.rs or any records measure.
- Changing max_rounds or MAX_MECHANICAL_PASSES, or adding new retry limits.
- Adding a repair round for mechanical findings in planning.
- Reusing hard_producer_model or hard_producer_effort for the planner.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the planner's recorded model and effort per round, not that keys exist.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and integration tests, plus fmt, clippy and test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The no-weaker rule is enforced in code and tested.
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

- path: <state>/jobs/20261113-205925-c3n6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-205925-c3n6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test in crates/athena-core/src/run/tests.rs drives Athena::go with the fake agent and a critic revise. It shows that the first planner invocation records the ordinary model and effort, and that the next one records planner_revision_model and planner_revision_effort. Run `cargo test -p athena-core` and cite the test name.
New tests show that a mechanical plan-rule revision also escalates the next planner call. They also show that a key weaker than the current planner model or effort is ignored, and that with both keys unset every round keeps the same model and effort. Run `cargo test -p athena-core`.
New config.rs unit tests show that the keys parse as optional, that a value outside the allowed lists is rejected, and that an existing config without the keys still loads. Run `cargo test -p athena-data`.
In crates/athena-data/src/config.rs, Models has optional planner_revision_model and planner_revision_effort. They are not derived from hard_producer_model or hard_producer_effort. Grep planning.rs for a diff showing no use of producer_setting or hard_producer_* in the planner path.
The owner can see the escalation. The planner line printed by planning_round, or the invocation record, shows the model used on the revision round. A test in crates/athena-cli/tests/drain.rs or in the core tests asserts it. Run `cargo test -p athena-cli` if drain.rs is changed.
athena.toml has the new planner_revision_* entry under [models]. CHANGELOG.md has an entry that states the effective planner model and effort in this repo with and without the change, and what the keys do. docs/spec.toml, where it describes planning, mentions the revision setting.
docs/decisions.md gets a new appended decision with a Provenance bullet that records why dedicated keys were chosen over hard_producer_*. No existing entry is edited.
`cargo fmt --check`, `cargo clippy --workspace --all-targets` and `cargo test --workspace` pass. Each is named as its own simple command, or cited from the Athena-recorded passing checks on the final tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
