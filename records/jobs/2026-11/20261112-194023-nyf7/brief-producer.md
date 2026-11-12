# Job 20261112-194023-nyf7: Show the reader the rules that judge criteria, in its brief

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Athena builds criteria from the request text and does not test each one against the decisions that g
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: criterion-contradicts-athena-default-or-invariant (20261023-183245-dd7r)
> Cause: Athena builds criteria from the request text and does not test each one against the decisions that govern how criteria are judged. D43 says what kind of evidence a behavior criterion may use. Nothing makes the reading stage check a criterio...
> Seen 4 time(s) in 4 jobs. The introspections that record it:
> Introspection 20261024-045710-yn3g (job 20261024-045548-7qht); prevention it proposed: Add a check at reading acceptance. For every criterion it reads the declared type (behavior or content) and the named evidence. It compares them with the rules from D43 and the other criterion-judging defaults. It rejects a reading, with th...
> Introspection 20261102-013736-k4jv (job 20261102-005332-8gkv); prevention it proposed: At reading acceptance, run a code check that scans criteria and deliverables for edits to append-only files (docs/decisions.md existing entries, records/) and refuses or rewrites them as appending a new decision that supersedes or amends D8...
> Introspection 20261106-062030-6pqw (job 20261106-041052-wzy7); prevention it proposed: Add a check at reading acceptance. Any criterion that names an existing line of docs/decisions.md for edit or annotation is refused, with a message saying to append a new superseding entry that cites the old one. The check should read the s...
> Introspection 20261112-135855-a39s (job 20261030-013721-4gwn); prevention it proposed: Add a code check run when the reading is accepted, before planning. It compares every criterion that forbids touching a file against the default-required deliverables (CHANGELOG.md from the changelog default) and the admission rules in admi...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reader brief never states the decisions that govern how criteria are judged, so readings write criteria that contradict them. Examples: a behavior criterion citing evidence D43 won't accept, or an in-place edit to the append-only docs/decisions.md. Fix by adding a "Rules that judge criteria" section, built from live facts, to the reader brief in crates/athena-context/src/briefs/reader.rs, with a test. No new refusal rule. Prior jobs 20261024-045548-7qht, 20261102-005332-8gkv and 20261106-041052-wzy7 are addressed as noted in the assumptions.

Assumptions:
- Goal and audience: the reading stage (the reader agent) is given the facts that decide whether a criterion can be met. Maintainers judge the result by whether readings stop producing contradictory criteria.
- Code survey: read crates/athena-context/src/briefs/reader.rs (reference_and_rules_section, verifier_can_run_section, READING_RULES, CHECKABLE_CRITERIA_RULE), crates/athena-core/src/reading/checks.rs and reading/surveys.rs (check_criteria_evidence, check_governance_coverage), and crates/athena-core/src/mirrors.rs (append_only_problem). The entry point is the reader brief builder. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are untouched. The property is measured directly by building the brief and asserting its text.
- Existing code: extends crates/athena-context/src/briefs/reader.rs and reuses the READING_RULES pattern. It states D43 as implemented in crates/athena-core/src/verify.rs, the append-only rule from crates/athena-core/src/mirrors.rs, and the governance and default-deliverable rules from crates/athena-core/src/admission.rs.
- Prior-job handling: job 20261024-045548-7qht is landed and passed. Its D43/D115 change in verify.rs, run.rs, decisions.md, decision-links.toml and CHANGELOG.md is kept as is, and its two nits are not reopened. The reader brief should state the D115 rule, that a recorded passing check on the same tree counts as evidence. Job 20261102-005332-8gkv was rejected for editing D87 in place; the fix is to tell the reader that decision entries are append-only, and its gh/watcher/records work is out of scope here and not redone. Job 20261106-041052-wzy7 was rejected because a criterion demanded annotating D10 in place; this change keeps its retry-on-stale work out of scope and tells the reader to write 'record in a new appended decision plus a decision-links.toml link' instead. This change touches none of those jobs' candidate files, so they stay dropped from this change.
- Expected refusal rate: 0% of past accepted readings are newly refused, because no refusal rule is added. The change only adds information to the brief.
- The brief section is generated from the rule constants or text already used by the checks where possible, so it cannot drift from them. Otherwise a test pins it to the check's wording.
- CHANGELOG.md is touched because the changelog default requires an entry. docs/decisions.md gets a new appended decision, never an edit to an existing entry.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-context`. A new test builds the reader brief and asserts it contains a 'Rules that judge criteria' section that states D43's evidence rule (including that a recorded passing Athena check on the same tree counts), that docs/decisions.md and records/ entries are append-only (new decision, not an in-place edit), and that the default-required CHANGELOG.md deliverable cannot be forbidden. The test fails on the base commit and passes with the change.
2. [behavior] A second new test in crates/athena-context shows the section's rule text matches the constants or wording used by the enforcing code, so it fails if the enforcing rule changes and the brief does not. It covers the cause class, not one job.
3. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`; all pass for the workspace.
4. [content] crates/athena-context/src/briefs/reader.rs adds the section to reference_and_rules_section or a sibling function. A diff grep shows no new refusal function in crates/athena-core/src/reading/.
5. [content] A criterion states 'expected 0% of newly refused readings': no refusal rule is added, and the brief gives the information instead.
6. [content] CHANGELOG.md has an entry that names the root cause (the reader was never shown the rules that judge criteria) and states only what the diff does. docs/decisions.md ends with a new appended decision, and no earlier byte changed. docs/decision-links.toml gets any needed link.
7. [content] The reader prompt files are not edited. Prompt token budgets are unchanged, since the facts are added in code.

## Expected deliverables

- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Adding any new refusal or check at reading acceptance.
- Editing existing entries of docs/decisions.md or anything under records/.
- Redoing or changing the candidates of jobs 20261024-045548-7qht, 20261102-005332-8gkv and 20261106-041052-wzy7, or their files: verify.rs, run.rs, gh/*, watcher.rs, records.rs, land.rs, main.rs and README.md.
- Changes to athena.toml, defaults.toml, .github/workflows, or prompt files.
- Changing how the verifier judges D43.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the brief section; fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the brief text by running a test, not that the wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the reader brief, where the missing information should be.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and decision entries are written for an outside reader.
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

- path: <state>/jobs/20261112-194023-nyf7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261112-194023-nyf7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-context`. A new test builds the reader brief and asserts it contains a 'Rules that judge criteria' section that states D43's evidence rule (including that a recorded passing Athena check on the same tree counts), that docs/decisions.md and records/ entries are append-only (new decision, not an in-place edit), and that the default-required CHANGELOG.md deliverable cannot be forbidden. The test fails on the base commit and passes with the change.
A second new test in crates/athena-context shows the section's rule text matches the constants or wording used by the enforcing code, so it fails if the enforcing rule changes and the brief does not. It covers the cause class, not one job.
Run `cargo fmt --check`, `cargo clippy` and `cargo test`; all pass for the workspace.
crates/athena-context/src/briefs/reader.rs adds the section to reference_and_rules_section or a sibling function. A diff grep shows no new refusal function in crates/athena-core/src/reading/.
A criterion states 'expected 0% of newly refused readings': no refusal rule is added, and the brief gives the information instead.
CHANGELOG.md has an entry that names the root cause (the reader was never shown the rules that judge criteria) and states only what the diff does. docs/decisions.md ends with a new appended decision, and no earlier byte changed. docs/decision-links.toml gets any needed link.
The reader prompt files are not edited. Prompt token budgets are unchanged, since the facts are added in code.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## If you leave the tree unchanged

A run that leaves the tree unchanged still fails. Set `no_change` in your report, and Athena records it as the job's failure reason: `already_done` with the evidence (the file and symbol you found in place), `blocked` with the blocker, or `gave_up` with the reason. Set it to null when you changed files.

## Report

End with the producer report as your structured output.
