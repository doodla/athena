# Job 20261104-211248-rgdn: Check athena.toml cap sums at reading time; fixture job_usd in tests

## Request (verbatim)

> Make athena.toml cap changes safe to land: (1) when a reading or plan touches [budget] or agent cap fields in athena.toml, run the admission check (job_usd >= sum of reader, all planning rounds, producer and verifier caps) on the proposed values before dispatch, and report the minimum job_usd needed; (2) change the tests in admission.rs, run_pin_tests.rs, reading.rs and config.rs that depend on the live athena.toml job_usd value to use their own fixture config, so a config-only change to job_usd does not need test edits. Acceptance: a test where the proposed caps exceed job_usd is refused at reading time and names the required job_usd; raising job_usd in athena.toml alone passes the full test suite.
> 
> Acceptance:
> - The change is made as described: Raising caps in athena.toml failed twice: cap sum over job_usd refused at admission, and tests pin job_usd from the live config
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: athena.toml, admission.rs, run_pin_tests.rs, reading.rs, config.rs

## Reading (accepted by the user)

Add a reading-time check: a reading or plan that touches [budget] or agent cap fields in athena.toml is refused when the proposed caps need more than job_usd, and the refusal names the minimum job_usd. Replace the live-athena.toml job_usd dependence in tests in admission.rs, run_pin_tests.rs, reading.rs and config.rs with their own fixture config. Then raising job_usd alone passes the suite. The audience is Athena operators landing cap changes.

Assumptions:
- Code survey: read crates/athena-data/src/config.rs (Config::budget_required at ~936 sums reader_usd + planning_usd(max_rounds) + producer_usd + verifier_usd; Config::check_budget at ~948 returns ConfigError::BudgetBelowReserve with the required figure; tests at ~1006-1152 and REPO_CONFIG include_str of athena.toml), crates/athena-core/src/admission.rs (admit_without_secrets ~930-958 already runs check_budget on the candidate athena.toml after production; test a_job_usd_below_the_stage_caps_is_a_configuration_violation ~2018 replaces 'job_usd = 50.00' in BASE_CONFIG), crates/athena-core/src/reading.rs (budget_reserve ~128, check_budget/preflight ~480-515, tests shipped_budget_covers_the_reserve ~1658 and required_budget_follows_the_configured_caps ~1733 using the live config; check_reading_acceptance holds the reading-rule chain), crates/athena-core/src/run_pin_tests.rs (config() includes ../../../athena.toml; lines ~974, 1001, 1032 replace 'job_usd = 50.00'). The new check belongs in the reading-acceptance chain in reading.rs, calling Config::budget_required on a config with the proposed values. No record or data file is added, so check_record_file/AnyRecord are unaffected.
- Field meaning: [budget] job_usd is documented at reading.rs:158 as 'The job's whole budget, [budget] job_usd'; config: live athena.toml line 47 job_usd = 50.00. Cap fields are reader_usd, producer_usd, verifier_usd, planner_usd, critic_usd and [job_plan] max_rounds. No new config key is added.
- The reading-time check applies only to readings whose deliverables or text name athena.toml and a cap field. It parses the proposed values from the reading (or the plan's text) and overlays them on the base Config. Where the proposed values cannot be recovered, it falls back to the existing admission check and does not refuse.
- Test fixtures: each of the four files gets its own const fixture config text (a copy of the needed [budget] and [job_plan] tables) in place of include_str!('../../../athena.toml') wherever a test asserts or replaces job_usd. Tests that deliberately check the shipped athena.toml parses may keep REPO_CONFIG but must not assert a job_usd value.
- Consumers: athena.toml is read by athena-data Config::parse, athena-core admission and reading, and the cli. Whether athena.toml needs a change: none is expected, because the fix is in code and tests. It is listed at a low probability only in case the producer needs to raise job_usd to demonstrate the criterion.
- Expected refusal rate: the new rule refuses only readings that touch cap fields with a sum above job_usd, expected 1% of newly refused readings.
- Files named in the request are admission.rs, run_pin_tests.rs, reading.rs and config.rs, under crates/athena-core/src and crates/athena-data/src. CHANGELOG.md is a default-written deliverable.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/reading.rs builds a reading that touches athena.toml with proposed caps whose sum exceeds job_usd. It asserts that check_reading_acceptance refuses the reading before dispatch and that the message names the required minimum job_usd, as a '$N.NN' figure equal to Config::budget_required of the proposed values. Run cargo test -p athena-core.
2. [behavior] A companion test shows a reading that proposes caps within job_usd is accepted, and a reading that does not touch athena.toml is unaffected. Run cargo test -p athena-core.
3. [behavior] After editing only job_usd in athena.toml, for example 50.00 to 60.00, cargo test passes in full. Check by running cargo test with that single-line change and confirming no test asserts the live job_usd value.
4. [content] A Grep for include_str of athena.toml and for 'job_usd = 50.00' in crates/athena-core/src/admission.rs, run_pin_tests.rs, reading.rs and crates/athena-data/src/config.rs shows the job_usd-dependent tests use a local fixture config, with no remaining dependence on the live job_usd value.
5. [content] The reading-time check reuses Config::budget_required / check_budget in crates/athena-data/src/config.rs; it does not re-derive the sum. The existing admission check in admission.rs is unchanged in behavior.
6. [behavior] No unrelated behavior changes: the existing tests pass. The recorded cargo fmt --check, cargo clippy and cargo test results on the final tree all pass.
7. [content] CHANGELOG.md records the change. Any behavior change to the reading rules is noted there; docs/decisions.md is touched only by a new appended decision with a Provenance bullet, if one is added.
8. [content] The change refuses more readings by design: expected 1% of newly refused readings.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)

## Out of scope

- Changing job_usd or any cap value in athena.toml as a goal of this job
- Tests in other files that read the live config, such as crates/athena-cli/tests and run/tests.rs, beyond what the full suite needs
- Changing the existing candidate-time admission check semantics
- Edits to existing entries in docs/decisions.md or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the refusal and acceptance paths; fmt, clippy and test are criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The refusal criterion runs the real acceptance path and checks the named required job_usd.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in code in the reading-acceptance chain, not in prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix covers both failure causes: the missing reading-time check and the tests pinned to live config.
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

36 more qualifying classes are left out.

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

- path: <state>/jobs/20261104-211248-rgdn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261104-211248-rgdn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new unit test in crates/athena-core/src/reading.rs builds a reading that touches athena.toml with proposed caps whose sum exceeds job_usd. It asserts that check_reading_acceptance refuses the reading before dispatch and that the message names the required minimum job_usd, as a '$N.NN' figure equal to Config::budget_required of the proposed values. Run cargo test -p athena-core.
A companion test shows a reading that proposes caps within job_usd is accepted, and a reading that does not touch athena.toml is unaffected. Run cargo test -p athena-core.
After editing only job_usd in athena.toml, for example 50.00 to 60.00, cargo test passes in full. Check by running cargo test with that single-line change and confirming no test asserts the live job_usd value.
A Grep for include_str of athena.toml and for 'job_usd = 50.00' in crates/athena-core/src/admission.rs, run_pin_tests.rs, reading.rs and crates/athena-data/src/config.rs shows the job_usd-dependent tests use a local fixture config, with no remaining dependence on the live job_usd value.
The reading-time check reuses Config::budget_required / check_budget in crates/athena-data/src/config.rs; it does not re-derive the sum. The existing admission check in admission.rs is unchanged in behavior.
No unrelated behavior changes: the existing tests pass. The recorded cargo fmt --check, cargo clippy and cargo test results on the final tree all pass.
CHANGELOG.md records the change. Any behavior change to the reading rules is noted there; docs/decisions.md is touched only by a new appended decision with a Provenance bullet, if one is added.
The change refuses more readings by design: expected 1% of newly refused readings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Checks before the verifier

After you report, Athena usually runs the configured checks (fmt, clippy and the test suite) on your work before the verifier does. A check that fails there, and did not already fail on the base, is sent back to you once, with its command, exit status and output tail, in the same send-back as any unmet criteria. A check still failing after that send-back goes to the verifier as flagged, and the verifier's own run of the checks is the verdict. The run is skipped, with a line saying so, when disk space is low, when no build slot is free, when it hits an error, or when your change holds check-tool configuration, which verification replaces with the base's.

Athena also runs each behavior criterion that names real records and gives a `status --snapshot` command, on your work tree, and records the command and its output for the verifier. It sends you back only when a line that holds the criterion's phrase prints a figure outside the stated range, or a parenthesised list of three or more job ids that all show the same value; other output is only recorded. Do not change what a command prints to satisfy this check.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
