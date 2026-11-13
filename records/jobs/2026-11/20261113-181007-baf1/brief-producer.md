# Job 20261113-181007-baf1: Raise planner effort after planning thrash via [job_plan] thrash_effort

## Request (verbatim)

> try again on 20261113-151949-mjpz

## Reading (accepted by the user)

Retry of job 20261113-151949-mjpz ("when the plan thrashing, increase intelligence"): after a critic revise (round > 1) or a mechanical re-plan, the next planner call runs at the higher of its configured effort and a new `[job_plan] thrash_effort` key (default xhigh). The first planner call and the critic are unchanged. The goal is fewer planning stops for the operator reading invocation records.

Assumptions:
- Job 20261113-151949-mjpz was dispatched, never verified, and recorded no findings. This retry redoes the same feature from the base, so there are no findings to fix. Its 7 changed files are all kept as deliverables below, and the work is redone from scratch.
- Dropped from the earlier deliverables: crates/athena-core/src/reading.rs. The earlier candidate did not change it, and nothing here needs it.
- Code survey: crates/athena-core/src/run/planning.rs (the planning loop calls self.agent_spec at about lines 140 and 343; the Attempt struct holds round and mechanical_passes; MAX_MECHANICAL_PASSES = 2), crates/athena-core/src/run/producer.rs (line 339 shows the existing pattern of overriding spec.effort), crates/athena-data/src/config.rs (JobPlanConfig, which is Copy and has deny_unknown_fields, so effort needs a small Copy enum), crates/athena-core/src/run/tests.rs (existing mechanical-pass test near line 1450). The change adds no record or data file, so check_record_file, AnyRecord and the round-trip test are not touched. Effort is measured directly from the effort recorded on each planner invocation.
- Terms: thrash means a critic revise (round > 1) or a mechanical re-plan (mechanical_passes > 0) in planning. The raise is the higher of the configured effort and thrash_effort, so it never lowers an effort such as max.
- Consumers: JobPlanConfig is read by crates/athena-core/src/run/planning.rs and set in athena.toml. No other reader of the new key was found.
- The change adds a default, not a refusal rule, except that config validation refuses an unknown effort value. Config files that currently load without the key keep loading.
- Audience: the operator who reads invocation records and judges whether planning stops less often.

## Acceptance criteria

1. [content] crates/athena-data/src/config.rs: JobPlanConfig gains a `thrash_effort` field that defaults to xhigh when absent, and an unknown value is refused when the config loads. A unit test in that file covers the default, an explicit value and the refusal.
2. [content] crates/athena-core/src/run/planning.rs: the planner spec's effort is raised to the higher of the configured effort and thrash_effort when round > 1 or mechanical_passes > 0. It is left unchanged on the first attempt of round 1. The critic's spec is not touched.
3. [behavior] A test in crates/athena-core/src/run/tests.rs drives a job whose first plan gets a critic `revise` and then an accept. It asserts the first planner invocation record has the configured effort and the second has the thrash effort. Run `cargo test -p athena-core`; it passes.
4. [behavior] A second test in crates/athena-core/src/run/tests.rs drives a plan that breaks Athena's own plan rules once, so there is a mechanical re-plan in round 1. It asserts the re-planned call runs at the thrash effort. Run `cargo test -p athena-core`; it passes.
5. [behavior] A test shows a plan accepted in one pass records every planner invocation, and the critic, at the unchanged configured effort. Another shows a configured effort above thrash_effort is not lowered. Run `cargo test -p athena-core`; both pass.
6. [behavior] Run `cargo test -p athena-data`; the config tests pass.
7. [content] athena.toml [job_plan] sets `thrash_effort` with a comment saying what triggers it. CHANGELOG.md records the change. docs/design/job-planning.md describes the escalation under the planning loop. docs/decisions.md gets a new appended decision with a Provenance bullet, and no existing entry is edited.
8. [content] The diff changes no producer or verifier effort, max_rounds, planner or critic budgets or timeouts, MAX_MECHANICAL_PASSES, or models; the deliverables and CHANGELOG.md are the only changed files.
9. [behavior] Athena's recorded `cargo fmt --check`, `cargo clippy` and `cargo test` runs pass on the final tree (D115).

## Expected deliverables

- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/job-planning.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the producer or verifier effort, or the hard-job escalation of the producer.
- Changing max_rounds, the planner or critic budgets or timeouts, or MAX_MECHANICAL_PASSES.
- Escalating the model, because the verifier's model may not produce.
- Editing existing records or decision entries.
- crates/athena-core/src/reading.rs: the earlier reading listed it, but its candidate did not change it, so this retry drops it.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Behavior criteria check the effort recorded on each planner invocation, not that the config key exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in config.rs and tests.rs; fmt, clippy and test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The escalation is enforced in planning.rs code and tested, not left as prose.
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

- path: <state>/jobs/20261113-181007-baf1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-181007-baf1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-data/src/config.rs: JobPlanConfig gains a `thrash_effort` field that defaults to xhigh when absent, and an unknown value is refused when the config loads. A unit test in that file covers the default, an explicit value and the refusal.
crates/athena-core/src/run/planning.rs: the planner spec's effort is raised to the higher of the configured effort and thrash_effort when round > 1 or mechanical_passes > 0. It is left unchanged on the first attempt of round 1. The critic's spec is not touched.
A test in crates/athena-core/src/run/tests.rs drives a job whose first plan gets a critic `revise` and then an accept. It asserts the first planner invocation record has the configured effort and the second has the thrash effort. Run `cargo test -p athena-core`; it passes.
A second test in crates/athena-core/src/run/tests.rs drives a plan that breaks Athena's own plan rules once, so there is a mechanical re-plan in round 1. It asserts the re-planned call runs at the thrash effort. Run `cargo test -p athena-core`; it passes.
A test shows a plan accepted in one pass records every planner invocation, and the critic, at the unchanged configured effort. Another shows a configured effort above thrash_effort is not lowered. Run `cargo test -p athena-core`; both pass.
Run `cargo test -p athena-data`; the config tests pass.
athena.toml [job_plan] sets `thrash_effort` with a comment saying what triggers it. CHANGELOG.md records the change. docs/design/job-planning.md describes the escalation under the planning loop. docs/decisions.md gets a new appended decision with a Provenance bullet, and no existing entry is edited.
The diff changes no producer or verifier effort, max_rounds, planner or critic budgets or timeouts, MAX_MECHANICAL_PASSES, or models; the deliverables and CHANGELOG.md are the only changed files.
Athena's recorded `cargo fmt --check`, `cargo clippy` and `cargo test` runs pass on the final tree (D115).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
