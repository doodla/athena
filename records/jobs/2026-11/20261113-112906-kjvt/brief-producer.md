# Job 20261113-112906-kjvt: Add reading check for gate order, reuse, record kind, direct checks

## Request (verbatim)

> try again on 20261030-060843-f4s2

## Reading (accepted by the user)

Retry of job 20261030-060843-f4s2, which failed in planning with no candidate, so nothing is kept and the work restarts from base. Add one reading check in athena-core, wired after check_code_survey, that refuses a code or tests reading with a behavior criterion unless it states entry point and gate order with the stop gate, reused existing functions, an accepted record kind or a round-trip deliverable, and a direct check per property. This time every caller of check_reading_text is a listed dependent, including `athena check-reading`. Goal: stop the repeated reading failures in introspections 20261028-055651-dhh0 and 20261028-082240-77as. Audience: the reader agent and the maintainers who judge refusals.

Assumptions:
- Previous result, job 20261030-060843-f4s2: it failed in planning round 5 with no candidate and no changed files, so no earlier work is kept. Each finding is addressed: (1) verify.rs is now a listed deliverable; (2) the `athena check-reading` command is a listed dependent with its tests; (3) when the in-loop replay reports a share above the limit, the producer does not loosen the markers and reports the measured share and newly refused job ids, or stops with no_change: blocked; (4) the skip-switch and internal-helper rule also catches identifier forms such as skip_x, no_x, disable, bypass and all-caps environment-variable tokens; (5) routine criteria such as 'fmt, clippy and test pass' or 'existing tests pass' are exempt from the direct-check clause; (6) the record kind is read from AnyRecord/HistoryRecord schema accessors where possible, else from the serde tag list pinned by a test, with a SCHEMA_TAGS constant in athena-data noted as a follow-up only.
- Code survey: read crates/athena-core/src/reading/surveys.rs (code_surveys, names_record_evidence, changes_code_with_behavior_criterion, existing_code_assumptions), crates/athena-core/src/reading/checks.rs (step! order; the entry point is check_reading_text, and gates run in that order, so the new step goes directly after check_code_survey), crates/athena-core/src/verify.rs (split_clauses, clause_words, is_negator), crates/athena-core/src/reading/replay.rs (replay_cases, which runs `check-reading` on base and candidate builds), crates/athena-core/src/run/mod.rs (loop_replay), crates/athena-data/src/records.rs (AnyRecord) and history.rs (HistoryRecord), crates/athena-cli/src/main.rs (check_reading_command and its tests), and crates/athena-context/src/briefs/reader.rs. The records check accepts only AnyRecord and HistoryRecord schema tags. A new typed record needs a round-trip test deliverable. The property is measured by counting planner, producer or model calls, or by asserting on the stop gate's refusal text, never by reading an internal helper or adding a skip switch.
- Existing code: crates/athena-core/src/reading/surveys.rs (names_record_evidence, changes_code_with_behavior_criterion, existing_code_assumptions), crates/athena-core/src/verify.rs (split_clauses, clause_words, is_negator) and crates/athena-core/src/reading/replay.rs (replay_cases) are reused rather than duplicated.
- Dependents of the new step: check_reading_text callers, namely crates/athena-cli/src/main.rs check_reading_command (class 'feature' fixture, so the new step does not apply to it; one added CLI case shows a code reading with a short survey printing `refuse:`; run `cargo test -p athena-cli check_reading`), run/mod.rs replays, replay.rs, and the reading tests in reading.rs and tests/refusal_corpus.rs.
- Labels parsed by the check: `Entry point:`, `Gate order:`, `Stop gate:`, `Reuses:` and `Direct check:`. The Direct check clause must name the entry point, a gate from Gate order, or a planner, producer or model call. The refusal error names which of (a)-(d) is missing.
- The prompt files under crates/athena-context/prompts/ are not edited, because the reader prompt has 2 tokens of headroom. The guidance goes in crates/athena-context/src/briefs/reader.rs, only as far as the briefs tests allow.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/surveys.rs defines a new pub check function, built on changes_code_with_behavior_criterion and names_record_evidence. Its refusal error names which of (a) gate order and stop gate, (b) reused existing function, (c) record kind or round-trip deliverable, (d) direct check is missing. crates/athena-core/src/reading/checks.rs wires it with a step! line directly after the check_code_survey step.
2. [behavior] Fixture tests in crates/athena-core/tests/refusal_corpus.rs and unit tests in crates/athena-core/src/reading.rs show four otherwise complete readings, each omitting exactly one of (a)-(d), are each refused with the missing part named, and one complete reading is accepted. Run `cargo test -p athena-core`.
3. [behavior] Fixture tests show a reading that contradicts the code is refused: a new record deliverable with no accepted record kind and no round-trip test deliverable, and a direct check that is a skip switch (words or identifiers like skip_probe, ATHENA_NO_PROBE) or a read of an internal helper. A negated mention is accepted. Run `cargo test -p athena-core`.
4. [behavior] A routine behavior criterion that only says fmt, clippy and test pass, or existing tests pass, needs no Direct check clause; a fixture shows it is accepted. Run `cargo test -p athena-core`.
5. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 30% of newly refused readings, because the check now asks for new required labels that past readings never used. A test or recorded refusal-rule replay (replay.rs) over recent accepted readings shows the newly refused share within the declared range plus 5 points. If the in-loop replay reports a share above that, the markers are not loosened and the measured share and newly refused job ids are reported. Run `cargo test -p athena-core`.
6. [behavior] The `athena check-reading` command still accepts its class 'feature' fixture reading, and a new CLI case shows a code reading with a short survey prints `refuse:` naming the new check. Run `cargo test -p athena-cli`.
7. [behavior] The prompt budgets test and the banned-word boundary test still pass, and no file under crates/athena-context/prompts/ is edited. Run `cargo test -p athena-context`.
8. [content] CHANGELOG.md has an entry for the change, including any stated limit of the skip-switch matching. athena.toml, defaults.toml, .github/workflows/ and records/ are unchanged.
9. [behavior] No unrelated behavior changes: cargo fmt --check, cargo clippy and cargo test pass on the final tree, as a recorded Athena check on the same tree.

## Expected deliverables

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/replay.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing records/ (including records/failure-classes/2026-10/20261028-055702-0t5w.json) or the cited introspections.
- Growing or editing any prompt file under crates/athena-context/prompts/, including the critic prompt; the reader prompt has 2 tokens of headroom.
- Changes to athena.toml, defaults.toml or CI workflows.
- Adding a pub SCHEMA_TAGS constant to athena-data (a follow-up only).
- Adding a planner repair round or other planning-pipeline changes.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check refusals and acceptances by running fixture tests, not that the function exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture and unit tests are listed and cargo fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Each behavior criterion names a cargo test command.
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

- path: <state>/jobs/20261113-112906-kjvt/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-112906-kjvt/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

crates/athena-core/src/reading/surveys.rs defines a new pub check function, built on changes_code_with_behavior_criterion and names_record_evidence. Its refusal error names which of (a) gate order and stop gate, (b) reused existing function, (c) record kind or round-trip deliverable, (d) direct check is missing. crates/athena-core/src/reading/checks.rs wires it with a step! line directly after the check_code_survey step.
Fixture tests in crates/athena-core/tests/refusal_corpus.rs and unit tests in crates/athena-core/src/reading.rs show four otherwise complete readings, each omitting exactly one of (a)-(d), are each refused with the missing part named, and one complete reading is accepted. Run `cargo test -p athena-core`.
Fixture tests show a reading that contradicts the code is refused: a new record deliverable with no accepted record kind and no round-trip test deliverable, and a direct check that is a skip switch (words or identifiers like skip_probe, ATHENA_NO_PROBE) or a read of an internal helper. A negated mention is accepted. Run `cargo test -p athena-core`.
A routine behavior criterion that only says fmt, clippy and test pass, or existing tests pass, needs no Direct check clause; a fixture shows it is accepted. Run `cargo test -p athena-core`.
The change refuses more readings and the higher refusal rate is intended: expected 30% of newly refused readings, because the check now asks for new required labels that past readings never used. A test or recorded refusal-rule replay (replay.rs) over recent accepted readings shows the newly refused share within the declared range plus 5 points. If the in-loop replay reports a share above that, the markers are not loosened and the measured share and newly refused job ids are reported. Run `cargo test -p athena-core`.
The `athena check-reading` command still accepts its class 'feature' fixture reading, and a new CLI case shows a code reading with a short survey prints `refuse:` naming the new check. Run `cargo test -p athena-cli`.
The prompt budgets test and the banned-word boundary test still pass, and no file under crates/athena-context/prompts/ is edited. Run `cargo test -p athena-context`.
CHANGELOG.md has an entry for the change, including any stated limit of the skip-switch matching. athena.toml, defaults.toml, .github/workflows/ and records/ are unchanged.
No unrelated behavior changes: cargo fmt --check, cargo clippy and cargo test pass on the final tree, as a recorded Athena check on the same tree.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
